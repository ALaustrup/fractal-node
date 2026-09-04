//! `fn` — the Fractal Node command line.
//!
//! N3: the CLI is a first-class front end, not a wrapper. Everything the GUI can
//! do, this can do — checked at every release tag by the parity suite (P13).
//!
//! Grammar is `fn <noun> <verb>` (`docs/31 §3.1`), with nouns and verbs taken
//! verbatim from the canonical vocabulary. A command using a word that is not in
//! `docs/01` fails the schema lint.

#![allow(clippy::print_stdout, clippy::print_stderr)] // A CLI's output IS its interface.

mod http;
mod render;
mod theme;

use clap::{Parser, Subcommand};
use render::Format;
use std::io::IsTerminal as _;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(
    name = "fn",
    version,
    about = "Fractal Node",
    disable_help_subcommand = true
)]
struct Cli {
    /// Output contract. Defaults to `human` on a terminal and `json` when piped.
    #[arg(long, global = true, value_enum)]
    format: Option<Format>,

    /// The Node to talk to.
    #[arg(
        long,
        global = true,
        env = "FRACTAL_NODE",
        default_value = "127.0.0.1:8787"
    )]
    node: String,

    /// Evaluate without emitting anything. Consequential commands require it.
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Node and identity status.
    Status,
    /// The machine-readable command tree (`docs/31 §8`).
    Schema,
    /// Societies — the atomic container.
    #[command(subcommand)]
    Society(SocietyCmd),
}

#[derive(Subcommand, Debug)]
enum SocietyCmd {
    /// List the Societies this Node holds.
    List,
    /// Found a Society.
    Create {
        /// Display name.
        name: String,
        /// Globally unique handle, 3–24 characters.
        #[arg(long)]
        handle: String,
        /// Public | discoverable | private | sealed.
        #[arg(long, default_value = "discoverable")]
        visibility: String,
        /// Makes the command safe to retry.
        #[arg(long)]
        idempotency_key: Option<String>,
    },
    /// Read one Society.
    Get {
        /// `soc_…`
        society_id: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let format = Format::resolve(cli.format, std::io::stdout().is_terminal());
    let code = run(&cli, format);
    std::process::exit(code);
}

fn run(cli: &Cli, format: Format) -> i32 {
    match &cli.command {
        None => {
            boot(&cli.node, format);
            0
        }
        Some(Command::Status) => status(&cli.node, format),
        Some(Command::Schema) => {
            // Discoverability for agents: the whole surface, without scraping help.
            let tree = serde_json::json!({
                "cli_version": VERSION,
                "commands": [
                    { "path": ["status"], "operation": "node.status" },
                    { "path": ["schema"], "operation": "cli.schema" },
                    { "path": ["society", "list"],   "operation": "society.list" },
                    { "path": ["society", "create"], "operation": "society.create",
                      "args": ["name"], "flags": ["handle", "visibility", "idempotency-key"] },
                    { "path": ["society", "get"],    "operation": "society.get", "args": ["society_id"] }
                ],
                "global_flags": ["format", "node", "dry-run"],
                "exit_codes": {
                    "0": "success", "1": "generic failure", "2": "usage",
                    "3": "authentication required", "4": "capability denied",
                    "5": "not found", "6": "conflict", "7": "rate limited",
                    "8": "node unreachable", "9": "confirmation required",
                    "10": "dry run reported blocking violations", "70": "internal error"
                }
            });
            render::ok(format, &tree, || println!("{tree:#}"));
            0
        }
        Some(Command::Society(s)) => society(cli, format, s),
    }
}

/// The boot sequence (`docs/31 §6`).
///
/// Every line is a real check. A boot screen that lies about system state would
/// be the worst possible first impression for an instrument.
fn boot(node: &str, format: Format) {
    if format == Format::Json {
        let _ = status(node, format);
        return;
    }
    println!();
    println!("                        {}", theme::signal("◇"));
    println!();
    println!("            {}", theme::dim("F R A C T A L   N O D E"));
    println!("            {}", theme::signal("ACCESS TERMINAL"));
    println!();
    let reachable = http::request(node, "GET", "/health", None).ok();
    let runtime = reachable
        .as_ref()
        .and_then(|r| serde_json::from_str::<serde_json::Value>(&r.body).ok())
        .and_then(|v| {
            v.pointer("/data/runtime")
                .and_then(|s| s.as_str().map(String::from))
        });

    line("01 / CLI     ", &format!("fn {VERSION}"), "OK", true);
    match runtime {
        Some(v) => line("02 / RUNTIME ", &format!("{node} · core {v}"), "LIVE", true),
        None => line(
            "02 / RUNTIME ",
            &format!("{node} · no answer"),
            "OFFLINE",
            false,
        ),
    }
    line(
        "03 / IDENTITY",
        "not enrolled — arrives in PH1",
        "PENDING",
        false,
    );
    line(
        "04 / LEDGER  ",
        "internal — arrives in PH1",
        "PENDING",
        false,
    );
    println!();
    println!("  {} {}", theme::signal("⌁"), theme::muted("ready"));
    println!("  {}", theme::dim("try: fn society list"));
    println!();
}

fn line(index: &str, detail: &str, state: &str, good: bool) {
    let painted = if good {
        theme::signal(state)
    } else {
        theme::muted(state)
    };
    println!(
        "  {} {:<38} {}",
        theme::signal(&format!("[ {index}]")),
        theme::muted(detail),
        painted
    );
}

fn status(node: &str, format: Format) -> i32 {
    match http::request(node, "GET", "/health", None) {
        Ok(r) if r.status == 200 => {
            let parsed: serde_json::Value =
                serde_json::from_str(&r.body).unwrap_or_else(|_| serde_json::json!({}));
            let data = parsed.get("data").cloned().unwrap_or(serde_json::json!({}));
            render::ok(format, &data, || {
                println!(
                    "{} {}  {}",
                    theme::signal("●"),
                    theme::muted("LIVE"),
                    theme::dim(&format!(
                        "runtime {} · api {}",
                        data.get("runtime").and_then(|v| v.as_str()).unwrap_or("?"),
                        data.get("api_version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?")
                    ))
                );
            });
            0
        }
        Ok(r) => render::fail(
            format,
            "unreachable",
            "The Node answered, but not with health",
            &format!("HTTP {} from {node}", r.status),
            Some("Check that --node points at a Fractal Node Runtime."),
        ),
        Err(e) => render::fail(
            format,
            "unreachable",
            "No Node at that address",
            &format!("{node}: {e}"),
            Some("Start one with `fractal-node --listen 127.0.0.1:8787`, or pass --node."),
        ),
    }
}

fn society(cli: &Cli, format: Format, cmd: &SocietyCmd) -> i32 {
    match cmd {
        SocietyCmd::List => society_list(cli, format),
        SocietyCmd::Get { society_id } => society_get(cli, format, society_id),
        SocietyCmd::Create {
            name,
            handle,
            visibility,
            idempotency_key,
        } => society_create(
            cli,
            format,
            name,
            handle,
            visibility,
            idempotency_key.as_deref(),
        ),
    }
}

fn society_list(cli: &Cli, format: Format) -> i32 {
    match call(&cli.node, "GET", "/v1/societies", None, format) {
        Ok(data) => {
            render::ok(format, &data, || print_list(&data));
            0
        }
        Err(code) => code,
    }
}

fn print_list(data: &serde_json::Value) {
    let empty = Vec::new();
    let list = data
        .get("societies")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty);
    if list.is_empty() {
        // An empty state is a designed screen, not a fallback (docs/32 §6).
        println!("{}", theme::muted("No Societies on this Node yet."));
        println!(
            "{}",
            theme::dim("Found one: fn society create \"Oracle Hall\" --handle oracle_hall")
        );
        return;
    }
    println!(
        "{}",
        theme::dim(
            "  ID                              NAME                  HANDLE            MEMBERS"
        )
    );
    for s in list {
        let g = |k: &str| s.get(k).and_then(|v| v.as_str()).unwrap_or("").to_owned();
        let members = s
            .get("member_count")
            .map_or_else(|| "0".to_owned(), std::string::ToString::to_string);
        println!(
            "{} {:<31} {:<21} {:<17} {}",
            theme::rail(false),
            theme::dim(&g("society_id")),
            g("name"),
            theme::muted(&g("handle")),
            theme::electric(&members)
        );
    }
}

fn society_get(cli: &Cli, format: Format, society_id: &str) -> i32 {
    match call(
        &cli.node,
        "GET",
        &format!("/v1/societies/{society_id}"),
        None,
        format,
    ) {
        Ok(data) => {
            render::ok(format, &data, || print_society(&data));
            0
        }
        Err(code) => code,
    }
}

fn society_create(
    cli: &Cli,
    format: Format,
    name: &str,
    handle: &str,
    visibility: &str,
    idempotency_key: Option<&str>,
) -> i32 {
    if cli.dry_run {
        // docs/31 §4.5: a dry run evaluates fully and emits nothing.
        let effects = serde_json::json!({
            "dry_run": true,
            "effects": [{ "kind": "society.created.v1", "name": name, "handle": handle }],
            "emitted": 0,
        });
        render::ok(format, &effects, || {
            println!(
                "{} would found {}",
                theme::muted("dry run:"),
                theme::signal(name)
            );
            println!(
                "  {}",
                theme::dim("1 event · society.created.v1 · nothing written")
            );
        });
        return 0;
    }
    let body = serde_json::json!({
        "name": name,
        "handle": handle,
        "visibility": visibility,
        "idempotency_key": idempotency_key,
    });
    match call(
        &cli.node,
        "POST",
        "/v1/societies",
        Some(&body.to_string()),
        format,
    ) {
        Ok(data) => {
            render::ok(format, &data, || {
                println!("{} Society founded", theme::signal("⌁"));
                print_society(&data);
            });
            0
        }
        Err(code) => code,
    }
}

fn print_society(data: &serde_json::Value) {
    let Some(s) = data.get("society") else { return };
    let g = |k: &str| s.get(k).and_then(|v| v.as_str()).unwrap_or("—").to_owned();
    println!("  {:<12} {}", theme::dim("NAME"), g("name"));
    println!(
        "  {:<12} {}",
        theme::dim("HANDLE"),
        theme::muted(&g("handle"))
    );
    println!(
        "  {:<12} {}",
        theme::dim("ID"),
        theme::dim(&g("society_id"))
    );
    println!(
        "  {:<12} {}",
        theme::dim("STATUS"),
        theme::signal(&g("status"))
    );
    println!(
        "  {:<12} {}",
        theme::dim("VISIBILITY"),
        theme::muted(&g("visibility"))
    );
    println!(
        "  {:<12} {}",
        theme::dim("MEMBERS"),
        theme::electric(
            &s.get("member_count")
                .map_or("0".into(), std::string::ToString::to_string)
        )
    );
}

/// Call the Node and unwrap the envelope, mapping errors to exit codes.
fn call(
    node: &str,
    method: &str,
    path: &str,
    body: Option<&str>,
    format: Format,
) -> Result<serde_json::Value, i32> {
    let resp = http::request(node, method, path, body).map_err(|e| {
        render::fail(
            format,
            "unreachable",
            "No Node at that address",
            &format!("{node}: {e}"),
            Some("Start one with `fractal-node`, or pass --node."),
        )
    })?;

    let parsed: serde_json::Value = serde_json::from_str(&resp.body).map_err(|_| {
        render::fail(
            format,
            "internal",
            "The Node sent something that is not JSON",
            &resp.body.chars().take(200).collect::<String>(),
            Some("This is a bug. Report it with the command you ran."),
        )
    })?;

    if parsed.get("ok").and_then(serde_json::Value::as_bool) == Some(true) {
        return Ok(parsed.get("data").cloned().unwrap_or(serde_json::json!({})));
    }

    let err = parsed
        .get("error")
        .cloned()
        .unwrap_or(serde_json::json!({}));
    let get = |k: &str| err.get(k).and_then(|v| v.as_str()).unwrap_or("").to_owned();
    let code = get("code");
    let title = get("title");
    let title = if title.is_empty() {
        "Refused".to_owned()
    } else {
        title
    };
    let detail = get("detail");
    Err(render::fail(
        format,
        &code,
        &title,
        &detail,
        err.pointer("/remedy/human").and_then(|v| v.as_str()),
    ))
}
