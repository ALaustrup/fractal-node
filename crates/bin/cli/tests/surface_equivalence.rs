//! Acceptance criterion 5 (`docs/50 PH0`): *creating a Society via web, CLI and
//! API produces identical event streams.*
//!
//! This was "verified by hand" for the length of Phase 0, which is another way
//! of saying it was true once, on one machine, in front of one person. P13 is
//! the principle at stake — one core, many front ends, no interface a
//! second-class bolt-on — and a principle nothing measures is a slogan
//! (`docs/40 §7.7.1`).
//!
//! Each surface gets its OWN Node, its own port and its own log directory, so
//! nothing is shared and no ordering can hide a difference. Each is asked to
//! found the same Society. The three logs are then normalised — identifiers and
//! clocks are legitimately unequal — and everything that remains must match
//! exactly.
//!
//! What each surface actually exercises:
//!
//!   API   raw HTTP/1.1 against `POST /v1/societies`, spelled from the `OpenAPI`
//!         document. The contract as written.
//!   CLI   the real `fn` binary, as a subprocess. Its own argument parsing, its
//!         own defaults, its own request construction.
//!   WEB   the real `apps/web/app.js`, imported UNMODIFIED under `node` with a
//!         resolve hook and a minimal DOM, its real submit handler invoked.
//!         Not a paraphrase of the page — the page.
//!
//! The web leg needs `node` on PATH. It fails rather than skips when it is
//! absent: a criterion that quietly stops being checked is worse than one
//! openly unmet.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::io::{Read as _, Write as _};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const NAME: &str = "The First Hearth";
const HANDLE: &str = "firsthearth";
const VISIBILITY: &str = "discoverable";

/// The same intent, typed the way a person types. Surrounding whitespace is the
/// most ordinary input defect there is, and it is where these three surfaces
/// were actually found to disagree: the web GUI trimmed before sending, the CLI
/// and the API did not, so this founded a Society from the browser and was
/// refused everywhere else. Clean inputs never touch that path, which is why
/// the first version of this test passed while the divergence was live.
const UNTIDY_NAME: &str = "  The First Hearth  ";
const UNTIDY_HANDLE: &str = "  FirstHearth  ";

// ---------------------------------------------------------------------------
// Locating the pieces
// ---------------------------------------------------------------------------

/// `CARGO_BIN_EXE_` only covers this package's binaries, so the Node is found
/// beside the CLI rather than assumed to be on PATH.
fn bin_dir() -> PathBuf {
    let cli = PathBuf::from(env!("CARGO_BIN_EXE_fn"));
    cli.parent()
        .expect("the CLI binary has a parent")
        .to_path_buf()
}

/// Build the Runtime before using it.
///
/// This is not belt-and-braces, it is load-bearing. `CARGO_BIN_EXE_` only
/// covers the binaries of the package under test, and cargo rebuilds only what
/// that package needs — so `cargo test -p fractal-cli` leaves `fractal-node`
/// exactly as it was, however old that is. The first version of this file did
/// not do this, and the consequence was not theoretical: a deliberate bug
/// injected into the domain to check that this test could SEE it produced four
/// green ticks, because the Runtime under test had been compiled before the
/// bug existed.
///
/// A test that silently exercises a stale artefact is worse than no test. It
/// reports on a build nobody has any more.
fn build_the_runtime() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let status = Command::new(env!("CARGO"))
            .args([
                "build",
                "--quiet",
                "-p",
                "fractal-node",
                "-p",
                "fractal-cli",
            ])
            .current_dir(repo_root())
            .status()
            .expect("running cargo to build the Runtime");
        assert!(
            status.success(),
            "the Runtime failed to build; this test cannot run"
        );
    });
}

fn node_binary() -> PathBuf {
    build_the_runtime();
    let exe = if cfg!(windows) {
        "fractal-node.exe"
    } else {
        "fractal-node"
    };
    let p = bin_dir().join(exe);
    assert!(
        p.exists(),
        "{} is missing after a build — the Runtime binary is not where the CLI binary lives, \
         so this test cannot know which Runtime it is talking to.",
        p.display()
    );
    p
}

fn repo_root() -> PathBuf {
    // crates/bin/cli -> crates/bin -> crates -> root
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..3 {
        p = p
            .parent()
            .expect("the manifest is nested under the repository root")
            .to_path_buf();
    }
    p
}

// ---------------------------------------------------------------------------
// A Node, running
// ---------------------------------------------------------------------------

struct Node {
    child: Child,
    addr: SocketAddr,
    dir: PathBuf,
}

impl Drop for Node {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

impl Node {
    fn start(tag: &str) -> Self {
        // Ask the OS for a free port and release it immediately. A race is
        // possible in principle; in practice the window is microseconds and the
        // alternative is a hardcoded port, which races with every other test.
        let port = TcpListener::bind("127.0.0.1:0")
            .expect("a free port")
            .local_addr()
            .expect("the bound address")
            .port();
        let addr: SocketAddr = format!("127.0.0.1:{port}")
            .parse()
            .expect("a valid address");

        let dir = std::env::temp_dir().join(format!("fractal-surface-{tag}-{port}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("creating the log directory");

        let child = Command::new(node_binary())
            .arg("--listen")
            .arg(addr.to_string())
            .arg("--data-dir")
            .arg(&dir)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawning the Runtime");

        let node = Self { child, addr, dir };
        node.await_ready();
        node
    }

    fn await_ready(&self) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline {
            if http(self.addr, "GET", "/health", None).is_ok() {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!("the Runtime never became ready on {}", self.addr);
    }

    /// Every event this Node recorded, across every Society, normalised.
    fn events(&self) -> Vec<serde_json::Value> {
        let mut logs: Vec<PathBuf> = std::fs::read_dir(&self.dir)
            .expect("reading the log directory")
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "log"))
            .collect();
        logs.sort();

        let mut out = Vec::new();
        for log in logs {
            let body = std::fs::read_to_string(&log).expect("reading a log");
            for line in body.lines().filter(|l| !l.trim().is_empty()) {
                let v: serde_json::Value = serde_json::from_str(line).expect("a log line is JSON");
                out.push(normalise(&v));
            }
        }
        out
    }
}

/// Strip what is ALLOWED to differ, keep everything else.
///
/// Identifiers are ULIDs and timestamps come from a real clock, so three
/// independent Nodes cannot agree on them and should not. Everything that
/// survives this function is something the three surfaces genuinely must agree
/// on — and `society_id` is removed from the payload too, because leaving it
/// in would make every comparison fail for the one reason that proves nothing.
fn normalise(v: &serde_json::Value) -> serde_json::Value {
    let mut v = v.clone();
    let obj = v.as_object_mut().expect("an event is an object");
    for volatile in [
        "society_id",
        "event_id",
        "correlation_id",
        "recorded_at",
        "occurred_at",
    ] {
        obj.remove(volatile);
    }
    if let Some(p) = obj.get_mut("payload").and_then(|p| p.as_object_mut()) {
        p.remove("society_id");
    }
    v
}

// ---------------------------------------------------------------------------
// A minimal HTTP/1.1 client — the API surface, spelled by hand
// ---------------------------------------------------------------------------

fn http(addr: SocketAddr, method: &str, path: &str, body: Option<&str>) -> std::io::Result<String> {
    let mut stream = TcpStream::connect(addr)?;
    stream.set_read_timeout(Some(Duration::from_secs(20)))?;
    let payload = body.unwrap_or("");
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\
         Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{payload}",
        payload.len()
    );
    stream.write_all(req.as_bytes())?;
    stream.flush()?;
    let mut raw = String::new();
    stream.read_to_string(&mut raw)?;
    Ok(raw)
}

fn http_body(raw: &str) -> &str {
    raw.split_once("\r\n\r\n").map_or("", |(_, b)| b)
}

// ---------------------------------------------------------------------------
// The three surfaces
// ---------------------------------------------------------------------------

fn via_api(node: &Node) {
    via_api_with(node, NAME, HANDLE);
}

fn via_api_with(node: &Node, name: &str, handle: &str) {
    let body = serde_json::json!({
        "name": name,
        "handle": handle,
        "visibility": VISIBILITY,
    })
    .to_string();
    let raw = http(node.addr, "POST", "/v1/societies", Some(&body)).expect("the API call");
    assert!(
        raw.starts_with("HTTP/1.1 201"),
        "the API surface did not create a Society:\n{raw}"
    );
}

fn via_cli(node: &Node) {
    via_cli_with(node, NAME, HANDLE);
}

fn via_cli_with(node: &Node, name: &str, handle: &str) {
    let out = Command::new(env!("CARGO_BIN_EXE_fn"))
        .args([
            "society",
            "create",
            name,
            "--handle",
            handle,
            "--visibility",
            VISIBILITY,
            "--node",
            &node.addr.to_string(),
            "--format",
            "json",
        ])
        .output()
        .expect("running the CLI");
    assert!(
        out.status.success(),
        "the CLI surface did not create a Society:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn via_web(node: &Node) {
    via_web_with(node, NAME, HANDLE);
}

fn via_web_with(node: &Node, name: &str, handle: &str) {
    let root = repo_root();
    let drive = root.join("crates/bin/cli/tests/web/drive.mjs");
    let client = root.join("packages/api-client/dist/index.js");
    let app = root.join("apps/web/app.js");
    for p in [&drive, &client, &app] {
        assert!(
            p.exists(),
            "{} is missing — the web surface cannot be driven",
            p.display()
        );
    }

    let out = Command::new("node")
        .arg(&drive)
        .arg(&client)
        .arg(&app)
        .arg(format!("http://{}", node.addr))
        .args([name, handle, VISIBILITY])
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "could not run `node` ({e}). The web surface is driven by executing the real \
                 apps/web/app.js, so this test needs Node.js on PATH. It fails rather than \
                 skips on purpose: criterion 5 unchecked is worse than criterion 5 unmet."
            )
        });
    assert!(
        out.status.success(),
        "the web surface did not create a Society:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

// ---------------------------------------------------------------------------
// The criterion
// ---------------------------------------------------------------------------

#[test]
fn web_cli_and_api_write_identical_event_streams() {
    let api = Node::start("api");
    via_api(&api);
    let a = api.events();

    let cli = Node::start("cli");
    via_cli(&cli);
    let c = cli.events();

    let web = Node::start("web");
    via_web(&web);
    let w = web.events();

    assert_eq!(
        a.len(),
        1,
        "the API surface wrote {} events, expected exactly 1",
        a.len()
    );
    assert_eq!(
        c.len(),
        1,
        "the CLI surface wrote {} events, expected exactly 1",
        c.len()
    );
    assert_eq!(
        w.len(),
        1,
        "the web surface wrote {} events, expected exactly 1",
        w.len()
    );

    let pretty = |v: &serde_json::Value| serde_json::to_string_pretty(v).unwrap_or_default();

    assert_eq!(
        a[0],
        c[0],
        "the API and the CLI disagree (P13).\n--- api ---\n{}\n--- cli ---\n{}",
        pretty(&a[0]),
        pretty(&c[0])
    );
    assert_eq!(
        a[0],
        w[0],
        "the API and the web GUI disagree (P13).\n--- api ---\n{}\n--- web ---\n{}",
        pretty(&a[0]),
        pretty(&w[0])
    );
}

/// The comparison above is only worth anything if it can fail. A surface asked
/// for a DIFFERENT Society must produce a different normalised event — proving
/// that `normalise` has not quietly deleted the fields that carry the meaning.
#[test]
fn the_comparison_can_actually_fail() {
    let a = Node::start("neg-a");
    via_api(&a);

    let b = Node::start("neg-b");
    let body = serde_json::json!({
        "name": "Cartographers",
        "handle": "cartographers",
        "visibility": "public",
    })
    .to_string();
    let raw = http(b.addr, "POST", "/v1/societies", Some(&body)).expect("the API call");
    assert!(raw.starts_with("HTTP/1.1 201"), "setup failed:\n{raw}");

    assert_ne!(
        a.events()[0],
        b.events()[0],
        "two genuinely different Societies normalised to the same event — `normalise` is \
         stripping something load-bearing, and the equivalence test above proves nothing"
    );
}

/// The Runtime's warning channel must reach every surface, not just the one
/// that happened to be tested by hand. PH0 flags an unauthenticated founder;
/// a surface that cannot see it would ship a false sense of security.
#[test]
fn every_surface_receives_the_runtime_warnings() {
    let node = Node::start("warn");
    let body = serde_json::json!({ "name": NAME, "handle": HANDLE }).to_string();
    let raw = http(node.addr, "POST", "/v1/societies", Some(&body)).expect("the API call");
    let parsed: serde_json::Value =
        serde_json::from_str(http_body(&raw)).expect("the response is JSON");
    let warnings = parsed
        .get("warnings")
        .and_then(|w| w.as_array())
        .expect("every successful envelope carries a warnings array");
    assert!(
        warnings.iter().any(|w| w.as_str().is_some_and(|s| s.contains("unauthenticated"))),
        "PH0 accepts the founder identity from the caller and must say so on every create: {parsed}"
    );

    let generated = std::fs::read_to_string(repo_root().join("packages/api-client/dist/index.js"))
        .expect("the generated client");
    assert!(
        generated.contains("json.warnings"),
        "the generated client drops the envelope's warnings, so no front end can surface them"
    );
    let app = std::fs::read_to_string(repo_root().join("apps/web/app.js")).expect("the web app");
    assert!(
        app.contains("onWarning"),
        "the web GUI does not subscribe to the warning sink, so the Runtime's warnings are invisible there"
    );
}

/// The same criterion, with input a person would actually type.
///
/// This is the case that caught the real divergence, and it is separate from
/// the tidy one on purpose: when it fails, the failure says "the surfaces
/// disagree about normalisation" rather than "the surfaces disagree", and the
/// difference between those two sentences is an afternoon.
#[test]
fn the_surfaces_agree_on_untidy_input_too() {
    let api = Node::start("untidy-api");
    via_api_with(&api, UNTIDY_NAME, UNTIDY_HANDLE);
    let a = api.events();

    let cli = Node::start("untidy-cli");
    via_cli_with(&cli, UNTIDY_NAME, UNTIDY_HANDLE);
    let c = cli.events();

    let web = Node::start("untidy-web");
    via_web_with(&web, UNTIDY_NAME, UNTIDY_HANDLE);
    let w = web.events();

    let pretty = |v: &serde_json::Value| serde_json::to_string_pretty(v).unwrap_or_default();

    assert_eq!(
        a.len(),
        1,
        "the API surface refused untidy input the others accepted"
    );
    assert_eq!(
        c.len(),
        1,
        "the CLI surface refused untidy input the others accepted"
    );
    assert_eq!(
        w.len(),
        1,
        "the web surface refused untidy input the others accepted"
    );

    assert_eq!(
        a[0],
        c[0],
        "API and CLI normalise differently.\n--- api ---\n{}\n--- cli ---\n{}",
        pretty(&a[0]),
        pretty(&c[0])
    );
    assert_eq!(
        a[0],
        w[0],
        "API and web normalise differently.\n--- api ---\n{}\n--- web ---\n{}",
        pretty(&a[0]),
        pretty(&w[0])
    );

    // And the normalisation actually happened, rather than all three agreeing
    // on storing the mess.
    assert_eq!(
        a[0].pointer("/payload/handle").and_then(|v| v.as_str()),
        Some("firsthearth"),
        "the Handle was stored un-normalised"
    );
    assert_eq!(
        a[0].pointer("/payload/name").and_then(|v| v.as_str()),
        Some("The First Hearth"),
        "the name was stored un-trimmed"
    );
}
