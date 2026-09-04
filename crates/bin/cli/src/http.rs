//! A minimal HTTP/1.1 client.
//!
//! PH0 only, and deliberately dependency-free: the CLI talks to a Node on
//! localhost, so there is no TLS to get wrong and no reason to spend a
//! dependency-budget slot (see the workspace `Cargo.toml`). In PH1 this file is
//! deleted and replaced by the **generated** client from `fractal-schema`, which
//! is what makes CLI/API parity a build error rather than a promise (`docs/30`).

use std::fmt::Write as _;
use std::io::{Read as _, Write as _};
use std::net::TcpStream;
use std::time::Duration;

pub(crate) struct Response {
    pub(crate) status: u16,
    pub(crate) body: String,
}

/// Perform a request against `base` (host:port), returning status and body.
///
/// # Errors
/// Any transport failure. The caller maps these to exit code 8.
pub(crate) fn request(
    base: &str,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> anyhow::Result<Response> {
    let host = base.trim_start_matches("http://").trim_end_matches('/');
    let mut stream = TcpStream::connect(host)?;
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;

    let payload = body.unwrap_or("");
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\nAccept: application/json\r\nUser-Agent: fn/{}\r\n",
        env!("CARGO_PKG_VERSION")
    );
    if body.is_some() {
        req.push_str("Content-Type: application/json\r\n");
        let _ = writeln!(req, "Content-Length: {}\r", payload.len());
    }
    req.push_str("\r\n");
    req.push_str(payload);

    stream.write_all(req.as_bytes())?;
    stream.flush()?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)?;
    let text = String::from_utf8_lossy(&raw).into_owned();

    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((text.as_str(), ""));
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|c| c.parse::<u16>().ok())
        .unwrap_or(0);

    // Chunked responses arrive with size prefixes; strip them rather than
    // pretending the body is JSON when it is not.
    let body = if head
        .to_ascii_lowercase()
        .contains("transfer-encoding: chunked")
    {
        dechunk(body)
    } else {
        body.to_owned()
    };

    Ok(Response { status, body })
}

fn dechunk(input: &str) -> String {
    let mut out = String::new();
    let mut rest = input;
    while let Some((size_line, tail)) = rest.split_once("\r\n") {
        let Ok(size) = usize::from_str_radix(size_line.trim(), 16) else {
            break;
        };
        let Some(chunk) = tail.get(..size) else { break };
        if size == 0 {
            break;
        }
        out.push_str(chunk);
        rest = tail.get(size + 2..).unwrap_or("");
    }
    out
}
