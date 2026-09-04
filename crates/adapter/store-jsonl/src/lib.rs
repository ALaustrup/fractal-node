//! An `EventStore` backed by one append-only file per Society.
//!
//! `docs/10 §7` names Postgres as the Phase 1 `EventStore` and "per-society
//! segment files" as a later option. This is that shape, built now, for one
//! reason: **a port with one implementation is not a port.** Having two from
//! day zero is what proves the boundary is real, and it is why swapping in
//! Postgres in PH1 is a configuration change rather than a discovery.
//!
//! It also gives PH0 a durable store with no database dependency, which keeps
//! the phase inside its dependency budget (see the workspace `Cargo.toml`).
//!
//! Layout: `<root>/<society_id>.log`, one JSON object per line, position implied
//! by line number. Appends are `O_APPEND` writes followed by an explicit sync.

use fractal_ports::{AppendError, EventEnvelope, EventStore, ReadError, Seq, StoredEvent};
use fractal_types::{SocietyId, Timestamp};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

type NowFn = Arc<dyn Fn() -> Timestamp + Send + Sync>;

pub struct JsonlEventStore {
    root: PathBuf,
    now: NowFn,
    /// Serialises appends. Per-Society ordering is the invariant being protected
    /// (`docs/10 §4`); a single lock is correct and, at PH0 volumes, free.
    write_lock: Mutex<()>,
}

impl std::fmt::Debug for JsonlEventStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsonlEventStore")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl JsonlEventStore {
    /// # Errors
    /// If the root directory cannot be created.
    pub fn open(root: impl AsRef<Path>, now: NowFn) -> std::io::Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self {
            root,
            now,
            write_lock: Mutex::new(()),
        })
    }

    fn path_for(&self, society_id: SocietyId) -> PathBuf {
        self.root.join(format!("{society_id}.log"))
    }

    fn read_all(&self, society_id: SocietyId) -> Result<Vec<StoredEvent>, ReadError> {
        let path = self.path_for(society_id);
        let Ok(file) = File::open(&path) else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();
        for (i, line) in BufReader::new(file).lines().enumerate() {
            let seq = Seq::new(i as u64 + 1);
            let line = line.map_err(|e| ReadError::Corrupt {
                society_id,
                seq,
                detail: e.to_string(),
            })?;
            if line.trim().is_empty() {
                continue;
            }
            let event: StoredEvent =
                serde_json::from_str(&line).map_err(|e| ReadError::Corrupt {
                    society_id,
                    seq,
                    detail: e.to_string(),
                })?;
            out.push(event);
        }
        Ok(out)
    }
}

#[allow(clippy::cast_possible_truncation)]
impl EventStore for JsonlEventStore {
    fn append(
        &self,
        society_id: SocietyId,
        expected_seq: Seq,
        events: Vec<EventEnvelope>,
    ) -> Result<Vec<StoredEvent>, AppendError> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_| AppendError::Unavailable("write lock poisoned".to_owned()))?;

        let existing = self
            .read_all(society_id)
            .map_err(|e| AppendError::Unavailable(e.to_string()))?;
        let head = Seq::new(existing.len() as u64 + 1);
        if head != expected_seq {
            return Err(AppendError::Conflict {
                society_id,
                expected: expected_seq,
                actual: head,
            });
        }

        let recorded_at = (self.now)();
        let mut stored = Vec::with_capacity(events.len());
        let mut buf = String::new();
        for (i, envelope) in events.into_iter().enumerate() {
            let seq = Seq::new(head.get() + i as u64);
            let ev = StoredEvent {
                seq,
                recorded_at,
                envelope,
            };
            let line =
                serde_json::to_string(&ev).map_err(|e| AppendError::Unavailable(e.to_string()))?;
            buf.push_str(&line);
            buf.push('\n');
            stored.push(ev);
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.path_for(society_id))
            .map_err(|e| AppendError::Unavailable(e.to_string()))?;
        file.write_all(buf.as_bytes())
            .map_err(|e| AppendError::Unavailable(e.to_string()))?;
        // The log is the source of truth (P6). An unsynced append is a lie.
        file.sync_all()
            .map_err(|e| AppendError::Unavailable(e.to_string()))?;
        Ok(stored)
    }

    fn read(
        &self,
        society_id: SocietyId,
        from: Seq,
        limit: usize,
    ) -> Result<Vec<StoredEvent>, ReadError> {
        let all = self.read_all(society_id)?;
        let Ok(start) = usize::try_from(from.get().saturating_sub(1)) else {
            return Ok(Vec::new());
        };
        Ok(all.into_iter().skip(start).take(limit).collect())
    }

    fn head(&self, society_id: SocietyId) -> Result<Seq, ReadError> {
        Ok(Seq::new(self.read_all(society_id)?.len() as u64 + 1))
    }

    fn societies(&self) -> Result<Vec<SocietyId>, ReadError> {
        let entries =
            fs::read_dir(&self.root).map_err(|e| ReadError::Unavailable(e.to_string()))?;
        let mut out = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            let Some(stem) = name.strip_suffix(".log") else {
                continue;
            };
            if let Ok(id) = stem.parse::<SocietyId>() {
                out.push(id);
            }
        }
        out.sort_unstable();
        Ok(out)
    }
}
