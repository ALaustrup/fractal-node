//! The HTTP gateway.
//!
//! Everything here is transport: parse, authorise at the edge, call the
//! application layer, shape the response. `docs/30` fixes the envelope and the
//! error model; this crate implements exactly that and adds nothing.
//!
//! The error bodies follow `docs/33 §7.3`: **cause, then remedy, no apology.**
//! "Something went wrong" is a banned string, and there is no code path here
//! that could produce one.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use fractal_app_kernel::{IdempotencyKey, PolicyDenied};
use fractal_app_society::{CreateSocietyRequest, ServiceError, SocietyService};
use fractal_types::{Fnid, Handle, Principal, SocietyId, Visibility};
use std::sync::Arc;

pub const API_VERSION: &str = "v1";
pub const CLI_MIN_VERSION: &str = "0.0.1";

#[derive(Clone)]
pub struct AppState {
    pub societies: Arc<SocietyService>,
    pub runtime_version: &'static str,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("runtime_version", &self.runtime_version)
            .finish_non_exhaustive()
    }
}

/// Build the router. Every route here is also a CLI verb — P13 is checked by the
/// parity suite, not by memory.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/societies", get(list_societies).post(create_society))
        .route("/v1/societies/:society_id", get(get_society))
        .route("/v1/meta", get(meta))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Envelope (docs/30 §4.2)
// ---------------------------------------------------------------------------

fn ok_body(data: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "ok": true,
        "data": data.clone(),
        "meta": { "api_version": API_VERSION },
        "warnings": [],
    })
}

struct ApiError {
    status: StatusCode,
    code: &'static str,
    title: String,
    detail: String,
    remedy: Option<String>,
    retryable: bool,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "ok": false,
            "error": {
                "code": self.code,
                "title": self.title,
                "detail": self.detail,
                "retryable": self.retryable,
                "remedy": self.remedy.map(|human| serde_json::json!({ "human": human })),
                "docs": format!("https://docs.fractalnode.dev/errors/{}", self.code),
            },
            "meta": { "api_version": API_VERSION },
        });
        (self.status, Json(body)).into_response()
    }
}

impl From<ServiceError> for ApiError {
    fn from(e: ServiceError) -> Self {
        match e {
            // Every denial shape is mapped explicitly. The compiler enforces
            // this: a new PolicyDenied variant cannot ship with a vague message.
            ServiceError::Denied(PolicyDenied::NotHuman { .. }) => Self {
                status: StatusCode::FORBIDDEN,
                code: "capability_denied",
                title: "Refused".to_owned(),
                detail: e.to_string(),
                remedy: Some(
                    "Founding a Society enacts a Charter, and only a Citizen may sign one. \
                     Act as yourself rather than through an Agent."
                        .to_owned(),
                ),
                retryable: false,
            },
            ServiceError::Denied(PolicyDenied::CapabilityDenied { ref capability }) => Self {
                status: StatusCode::FORBIDDEN,
                code: "capability_denied",
                title: "Refused".to_owned(),
                detail: e.to_string(),
                remedy: Some(format!(
                    "Grant `{capability}` in Policy, or ask the Operator to widen the Envelope."
                )),
                retryable: false,
            },
            ServiceError::Denied(PolicyDenied::ConfirmationRequired { .. }) => Self {
                status: StatusCode::FORBIDDEN,
                code: "confirmation_required",
                title: "Confirmation needed".to_owned(),
                detail: e.to_string(),
                remedy: Some(
                    "This action cannot be undone, so a person has to approve it. \
                     Confirm it in the app, or run the command interactively."
                        .to_owned(),
                ),
                retryable: false,
            },
            ServiceError::Rejected(ref r) => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "rejected",
                title: "Refused".to_owned(),
                detail: r.to_string(),
                remedy: Some("Adjust the request and send it again.".to_owned()),
                retryable: false,
            },
            ServiceError::Append(_) => Self {
                status: StatusCode::CONFLICT,
                code: "conflict",
                title: "Conflict".to_owned(),
                detail: e.to_string(),
                remedy: Some("Reload and retry with the current version.".to_owned()),
                retryable: true,
            },
            ServiceError::Read { .. } | ServiceError::Corrupt { .. } => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "store_unavailable",
                title: "The log could not be read".to_owned(),
                detail: e.to_string(),
                remedy: Some("Retry shortly. If it persists, check the Node's storage.".to_owned()),
                retryable: true,
            },
        }
    }
}

fn bad_request(code: &'static str, detail: String, remedy: &str) -> ApiError {
    ApiError {
        status: StatusCode::BAD_REQUEST,
        code,
        title: "Refused".to_owned(),
        detail,
        remedy: Some(remedy.to_owned()),
        retryable: false,
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn health(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(ok_body(&serde_json::json!({
        "status": "ok",
        "runtime": s.runtime_version,
        "api_version": API_VERSION,
    })))
}

async fn meta(State(s): State<AppState>) -> Json<serde_json::Value> {
    // Machine-readable surface description. `docs/31 §8`: an agent discovers the
    // shape of the system rather than scraping help text.
    Json(ok_body(&serde_json::json!({
        "runtime": s.runtime_version,
        "api_version": API_VERSION,
        "cli_min_version": CLI_MIN_VERSION,
        "phase": "PH0",
        "operations": [
            { "id": "node.status",    "method": "GET",  "path": "/health",                    "cli": "fn status" },
            { "id": "society.list",   "method": "GET",  "path": "/v1/societies",              "cli": "fn society list" },
            { "id": "society.create", "method": "POST", "path": "/v1/societies",              "cli": "fn society create" },
            { "id": "society.get",    "method": "GET",  "path": "/v1/societies/{society_id}", "cli": "fn society get" }
        ]
    })))
}

async fn list_societies(State(s): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let list = s.societies.list()?;
    Ok(Json(ok_body(&serde_json::json!({ "societies": list }))))
}

async fn get_society(
    State(s): State<AppState>,
    Path(raw): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id: SocietyId = raw.parse().map_err(|_| {
        bad_request(
            "invalid_identifier",
            format!("`{raw}` is not a Society identifier."),
            "Society identifiers look like soc_01J… — copy it from a listing.",
        )
    })?;
    match s.societies.get(id)? {
        Some(v) => Ok(Json(ok_body(&serde_json::json!({ "society": v })))),
        None => Err(ApiError {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            title: "No such Society".to_owned(),
            detail: format!("{id} is not on this Node."),
            remedy: Some("List societies to see what this Node holds.".to_owned()),
            retryable: false,
        }),
    }
}

#[derive(serde::Deserialize)]
struct CreateBody {
    name: String,
    handle: String,
    #[serde(default)]
    visibility: Option<Visibility>,
    #[serde(default)]
    idempotency_key: Option<String>,
    /// PH0 only: identity lands in PH1 and the actor comes from the session then.
    #[serde(default)]
    founder_fnid: Option<String>,
    #[serde(default)]
    societies_founded: u32,
    #[serde(default)]
    founder_level: u16,
}

async fn create_society(
    State(s): State<AppState>,
    Json(body): Json<CreateBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let handle = Handle::parse(&body.handle).map_err(|e| {
        bad_request(
            "invalid_handle",
            e.to_string(),
            "Handles are 3–24 characters of a–z, 0–9 and underscore.",
        )
    })?;

    // PH0 stand-in for authentication. PH1 replaces this with the passkey session
    // (docs/12); the handler above it does not change when it does.
    let fnid = match body.founder_fnid.as_deref() {
        Some(raw) => raw.parse::<Fnid>().map_err(|e| {
            bad_request(
                "invalid_fnid",
                e.to_string(),
                "Copy the FNID exactly, including its checksum.",
            )
        })?,
        None => Fnid::sample(1),
    };

    let req = CreateSocietyRequest {
        actor: Principal::Citizen { fnid },
        name: body.name,
        handle,
        visibility: body.visibility.unwrap_or_default(),
        idempotency_key: body.idempotency_key.map(IdempotencyKey::new),
        societies_founded: body.societies_founded,
        founder_level: body.founder_level,
    };
    let view = s.societies.create(&req)?;
    Ok((
        StatusCode::CREATED,
        Json(ok_body(&serde_json::json!({ "society": view }))),
    ))
}
