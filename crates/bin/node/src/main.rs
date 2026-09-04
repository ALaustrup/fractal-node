//! `fractal-node` — the Runtime.
//!
//! This is a composition root and nothing else: it is the only place in the
//! workspace where a concrete adapter may be named (`layers.toml`). All of the
//! behaviour lives in the layers below; if logic appears in this file, it is in
//! the wrong crate.

#![allow(clippy::print_stdout)] // A daemon's startup banner is its user interface.

use anyhow::Context as _;
use clap::Parser;
use fractal_adapter_ambient_system::{SystemClock, SystemIdGen};
use fractal_adapter_store_jsonl::JsonlEventStore;
use fractal_adapter_store_memory::MemoryEventStore;
use fractal_api_http::{router, AppState};
use fractal_app_kernel::CommandContext;
use fractal_app_society::SocietyService;
use fractal_ports::{Clock, EventStore};
use std::net::SocketAddr;
use std::sync::Arc;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(name = "fractal-node", version, about = "The Fractal Node Runtime")]
struct Args {
    /// Address to listen on.
    #[arg(long, env = "FRACTAL_LISTEN", default_value = "127.0.0.1:8787")]
    listen: SocketAddr,

    /// Where per-Society logs are kept. Omit for an in-memory Node that forgets
    /// everything on exit — useful for tests, never for anything you care about.
    #[arg(long, env = "FRACTAL_DATA_DIR")]
    data_dir: Option<std::path::PathBuf>,

    /// Serve the web GUI from this directory. PH0 serves static files; PH1
    /// replaces it with the built Vite bundle (docs/51) behind the same flag.
    #[arg(long, env = "FRACTAL_WEB_DIR")]
    web_dir: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let clock = Arc::new(SystemClock);

    let now_fn = {
        let c = Arc::clone(&clock);
        Arc::new(move || c.now())
    };

    let (store, backing): (Arc<dyn EventStore>, String) = match &args.data_dir {
        Some(dir) => {
            let s = JsonlEventStore::open(dir, now_fn)
                .with_context(|| format!("opening the log directory at {}", dir.display()))?;
            (Arc::new(s), format!("jsonl {}", dir.display()))
        }
        None => (
            Arc::new(MemoryEventStore::new(now_fn)),
            "memory (volatile)".to_owned(),
        ),
    };

    let ctx = Arc::new(CommandContext::new(clock, Arc::new(SystemIdGen)));
    let societies = Arc::new(SocietyService::new(Arc::clone(&store), ctx));
    let state = AppState {
        societies,
        runtime_version: VERSION,
    };

    let mut app = router(state);
    if let Some(dir) = &args.web_dir {
        // The GUI is a peer front end over the same public API (P3/P13), not a
        // privileged surface. It is served here purely so PH0 has one origin.
        let tokens = dir
            .join("..")
            .join("..")
            .join("packages/tokens/dist/tokens.css");
        app = app
            .route_service("/tokens.css", tower_http::services::ServeFile::new(tokens))
            .fallback_service(tower_http::services::ServeDir::new(dir));
    }

    let listener = tokio::net::TcpListener::bind(args.listen)
        .await
        .with_context(|| format!("binding {}", args.listen))?;
    let bound = listener.local_addr().unwrap_or(args.listen);

    println!("⌁ FRACTAL NODE // RUNTIME {VERSION}");
    println!("  [ 01 / STORE  ] {backing}");
    println!("  [ 02 / LISTEN ] http://{bound}");
    println!("  [ 03 / API    ] /v1/societies · /v1/meta · /health");
    match &args.web_dir {
        Some(d) => println!("  [ 04 / WEB    ] {} → http://{bound}/", d.display()),
        None => println!("  [ 04 / WEB    ] not served (pass --web-dir apps/web)"),
    }
    println!("  ready");

    axum::serve(listener, app).await.context("serving")?;
    Ok(())
}
