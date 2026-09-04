//! The two `EventStore` implementations must be behaviourally identical.
//!
//! This is the test that makes P5 real. An abstraction with one implementation
//! is a guess about the future; an abstraction with two, held to the same
//! observable behaviour, is a boundary. When Postgres arrives in PH1 it joins
//! this test rather than replacing it.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use fractal_adapter_store_jsonl::JsonlEventStore;
use fractal_adapter_store_memory::MemoryEventStore;
use fractal_ports::{AppendError, EventEnvelope, EventKind, EventStore, Seq};
use fractal_types::{Fnid, Principal, SocietyId, Timestamp, Ulid};
use std::sync::Arc;

fn now() -> Arc<dyn Fn() -> Timestamp + Send + Sync> {
    Arc::new(|| Timestamp::from_millis(1_700_000_000_000))
}

fn society(n: u128) -> SocietyId {
    SocietyId::new(Ulid::from_u128(n))
}

fn event(society_id: SocietyId, n: u128) -> EventEnvelope {
    EventEnvelope {
        society_id,
        event_id: Ulid::from_u128(n),
        kind: EventKind::from_static("test.thing.happened.v1"),
        schema_version: 1,
        occurred_at: Timestamp::from_millis(1_000 + i64::try_from(n).unwrap_or(0)),
        actor: Principal::Citizen {
            fnid: Fnid::sample(1),
        },
        on_behalf_of: None,
        envelope_ref: None,
        correlation_id: Ulid::from_u128(900 + n),
        causation_id: None,
        payload: serde_json::json!({ "n": n }),
    }
}

/// Run the same script against both stores and require identical observations.
fn both<F>(name: &str, script: F)
where
    F: Fn(&dyn EventStore) -> Vec<String>,
{
    let dir = std::env::temp_dir().join(format!("fn-jsonl-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let disk = JsonlEventStore::open(&dir, now()).expect("open jsonl store");
    let mem = MemoryEventStore::new(now());

    let a = script(&mem);
    let b = script(&disk);
    assert_eq!(a, b, "memory and jsonl stores disagreed in `{name}`");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn appending_and_reading_agree() {
    both("append_read", |store| {
        let s = society(1);
        let mut out = vec![format!("head0={}", store.head(s).unwrap())];
        let stored = store
            .append(s, Seq::FIRST, vec![event(s, 1), event(s, 2)])
            .unwrap();
        out.push(format!("appended={}", stored.len()));
        out.push(format!(
            "seqs={:?}",
            stored.iter().map(|e| e.seq.get()).collect::<Vec<_>>()
        ));
        out.push(format!("head1={}", store.head(s).unwrap()));
        let read = store.read(s, Seq::FIRST, 10).unwrap();
        out.push(format!("read={}", read.len()));
        out.push(format!(
            "payloads={:?}",
            read.iter()
                .map(|e| e.envelope.payload.to_string())
                .collect::<Vec<_>>()
        ));
        out
    });
}

#[test]
fn optimistic_concurrency_agrees() {
    both("conflict", |store| {
        let s = society(2);
        store.append(s, Seq::FIRST, vec![event(s, 1)]).unwrap();
        // Someone else already wrote. Appending at the stale position must fail
        // the same way in both stores — never silently overwrite.
        let err = store.append(s, Seq::FIRST, vec![event(s, 2)]).unwrap_err();
        let shape = match err {
            AppendError::Conflict {
                expected, actual, ..
            } => {
                format!("conflict expected={expected} actual={actual}")
            }
            other => format!("unexpected {other}"),
        };
        vec![shape, format!("head={}", store.head(s).unwrap())]
    });
}

#[test]
fn reads_are_scoped_to_one_society() {
    // P1, in the storage layer: a read for one Society can never surface another's
    // events. This is the invariant the whole partitioning strategy rests on.
    both("scoping", |store| {
        let a = society(10);
        let b = society(11);
        store
            .append(a, Seq::FIRST, vec![event(a, 1), event(a, 2)])
            .unwrap();
        store.append(b, Seq::FIRST, vec![event(b, 3)]).unwrap();
        let read_a = store.read(a, Seq::FIRST, 100).unwrap();
        let read_b = store.read(b, Seq::FIRST, 100).unwrap();
        assert!(read_a.iter().all(|e| e.envelope.society_id == a));
        assert!(read_b.iter().all(|e| e.envelope.society_id == b));
        let mut ids = store.societies().unwrap();
        ids.sort_unstable();
        vec![
            format!("a={} b={}", read_a.len(), read_b.len()),
            format!("societies={ids:?}"),
        ]
    });
}

#[test]
fn reading_an_unknown_society_is_empty_not_an_error() {
    both("unknown", |store| {
        let s = society(99);
        vec![
            format!("read={}", store.read(s, Seq::FIRST, 10).unwrap().len()),
            format!("head={}", store.head(s).unwrap()),
        ]
    });
}

#[test]
fn paging_agrees() {
    both("paging", |store| {
        let s = society(3);
        let events: Vec<_> = (1..=7).map(|n| event(s, n)).collect();
        store.append(s, Seq::FIRST, events).unwrap();
        let mut out = Vec::new();
        let mut from = Seq::FIRST;
        loop {
            let page = store.read(s, from, 3).unwrap();
            if page.is_empty() {
                break;
            }
            out.push(format!(
                "page={:?}",
                page.iter().map(|e| e.seq.get()).collect::<Vec<_>>()
            ));
            from = Seq::new(page.last().map_or(0, |e| e.seq.get()) + 1);
        }
        out
    });
}

#[test]
fn a_jsonl_log_survives_reopening() {
    // The memory store cannot demonstrate this, so it is checked on disk alone:
    // durability is the reason this implementation exists.
    let dir = std::env::temp_dir().join(format!("fn-jsonl-reopen-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let s = society(42);
    {
        let store = JsonlEventStore::open(&dir, now()).unwrap();
        store
            .append(s, Seq::FIRST, vec![event(s, 1), event(s, 2)])
            .unwrap();
    }
    let reopened = JsonlEventStore::open(&dir, now()).unwrap();
    let read = reopened.read(s, Seq::FIRST, 10).unwrap();
    assert_eq!(read.len(), 2);
    assert_eq!(reopened.head(s).unwrap(), Seq::new(3));
    assert_eq!(reopened.societies().unwrap(), vec![s]);
    let _ = std::fs::remove_dir_all(&dir);
}
