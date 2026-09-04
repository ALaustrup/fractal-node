//! The single source of truth for the public contract.
//!
//! `docs/30` sets out why this exists and `docs/41 §12` sets out how it is
//! enforced. The short version: P13 says a feature is not shipped until it
//! exists in the API, the CLI and a GUI. That promise is unkeepable if the three
//! surfaces are written by hand — they drift, quietly, in the direction of
//! whichever one someone last touched.
//!
//! So they are not written by hand. They are **generated from this file**:
//!
//! ```text
//!                      crates/support/schema  (this crate — hand-authored)
//!                                 │
//!                cargo xtask codegen
//!         ┌───────────┬───────────┼───────────┬──────────────┐
//!         ▼           ▼           ▼           ▼              ▼
//!   schemas/      schemas/    api/http     bin/cli      packages/
//!   openapi/      events/     generated    generated    api-client/
//!   v1.json       *.json      .rs          .rs          index.ts
//! ```
//!
//! `cargo xtask codegen --check` regenerates and diffs. Drift is a build
//! failure, so "the CLI is behind the API" is a state the repository cannot
//! reach. That is the whole point of Phase 0 doing this before Phase 1 has
//! anything to keep in sync.
//!
//! **This crate is data, not logic.** It contains no I/O, no formatting and no
//! generator code — the generators live in `xtask`, where they belong, so that
//! adding an output target never touches the contract.

#![no_std]

// ---------------------------------------------------------------------------
// Vocabulary
// ---------------------------------------------------------------------------

/// An HTTP method. Closed: the API uses these four and no others.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Patch,
    Delete,
}

impl Method {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }
    /// Lowercase, as `OpenAPI` wants it.
    #[must_use]
    pub const fn openapi(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Patch => "patch",
            Self::Delete => "delete",
        }
    }
}

/// A field's type.
///
/// Deliberately small. A contract that can express anything expresses nothing
/// clearly, and every variant here has to be renderable in `OpenAPI`, JSON Schema,
/// TypeScript and a CLI flag — four targets, so four ways to get it wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    /// UTF-8 text.
    Str,
    /// A 64-bit signed integer.
    Int,
    Bool,
    /// Milliseconds since the Unix epoch. Rendered as an integer, named as a time.
    Timestamp,
    /// An amount of Fraction.
    ///
    /// Always a decimal STRING on the wire, never a JSON number: 1e18 is three
    /// orders past IEEE-754's exact range, so a number would silently round
    /// balances in every JavaScript client (`docs/61` X-Q).
    Quanta,
    /// A closed set of string values.
    Enum(&'static [&'static str]),
    /// A named type declared in [`TYPES`].
    Ref(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub struct Field {
    pub name: &'static str,
    pub ty: Ty,
    pub required: bool,
    pub doc: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct TypeDef {
    pub name: &'static str,
    pub doc: &'static str,
    pub fields: &'static [Field],
}

/// How an operation is reached from the command line.
///
/// Every operation MUST have one. An operation without a CLI binding fails
/// `cargo xtask codegen` — that refusal is P13 expressed as a build error rather
/// than as a review conversation someone has to remember to have.
#[derive(Debug, Clone, Copy)]
pub struct CliBinding {
    /// `fn <noun> <verb>`. Both come from `docs/01`; the terminology lint checks them.
    pub noun: &'static str,
    pub verb: &'static str,
    /// Positional arguments, in order.
    pub args: &'static [&'static str],
    /// Long flags, without the leading dashes.
    pub flags: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub struct Operation {
    /// Stable identifier, `<noun>.<verb>`. Used by the parity gate and by agents.
    pub id: &'static str,
    pub summary: &'static str,
    pub method: Method,
    pub path: &'static str,
    pub cli: CliBinding,
    /// The request body type, for operations that take one.
    pub request: Option<&'static str>,
    /// The response `data` type.
    pub response: &'static str,
    /// The capability required. `None` means "any authenticated principal".
    /// Populated properly in PH3 when Envelopes exist.
    pub capability: Option<&'static str>,
    /// Whether an `Idempotency-Key` makes this safe to retry.
    pub idempotent: bool,
    /// Whether `--dry-run` is meaningful (`docs/31 §4.5`).
    pub dry_runnable: bool,
    /// Error codes this operation can return, from [`ERRORS`].
    pub errors: &'static [&'static str],
}

/// A domain event kind.
#[derive(Debug, Clone, Copy)]
pub struct EventDef {
    /// `<boundary>.<noun>.<past-tense-verb>.v<N>` (`docs/10 §5`).
    pub kind: &'static str,
    pub version: u16,
    pub doc: &'static str,
    /// The payload type, declared in [`TYPES`].
    pub payload: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ErrorDef {
    pub code: &'static str,
    pub http: u16,
    /// The CLI exit code (`docs/31 §4.3`). Stable forever: scripts depend on it.
    pub exit: i32,
    pub retryable: bool,
    pub doc: &'static str,
}

// ---------------------------------------------------------------------------
// THE CONTRACT
// ---------------------------------------------------------------------------

pub const API_VERSION: &str = "v1";

/// The closed error registry (`docs/30 §5`).
///
/// Closed on purpose: an open-ended error vocabulary is how clients end up
/// pattern-matching on message strings.
pub const ERRORS: &[ErrorDef] = &[
    ErrorDef {
        code: "usage",
        http: 400,
        exit: 2,
        retryable: false,
        doc: "The request was malformed.",
    },
    ErrorDef {
        code: "invalid_identifier",
        http: 400,
        exit: 2,
        retryable: false,
        doc: "An identifier did not parse or failed its checksum.",
    },
    ErrorDef {
        code: "invalid_handle",
        http: 400,
        exit: 2,
        retryable: false,
        doc: "A Handle was the wrong length or shape.",
    },
    ErrorDef {
        code: "invalid_fnid",
        http: 400,
        exit: 2,
        retryable: false,
        doc: "An FNID did not parse or failed its checksum.",
    },
    ErrorDef {
        code: "unauthenticated",
        http: 401,
        exit: 3,
        retryable: false,
        doc: "No valid session. Arrives properly in PH1 with passkeys.",
    },
    ErrorDef {
        code: "capability_denied",
        http: 403,
        exit: 4,
        retryable: false,
        doc: "The principal's Envelope does not permit this. Names the missing capability.",
    },
    ErrorDef {
        code: "confirmation_required",
        http: 403,
        exit: 9,
        retryable: false,
        doc: "An irreversible action needs a live human confirmation (P4).",
    },
    ErrorDef {
        code: "not_found",
        http: 404,
        exit: 5,
        retryable: false,
        doc: "No such object on this Node.",
    },
    ErrorDef {
        code: "conflict",
        http: 409,
        exit: 6,
        retryable: true,
        doc: "Optimistic concurrency lost. Reload and retry.",
    },
    ErrorDef {
        code: "rejected",
        http: 422,
        exit: 1,
        retryable: false,
        doc: "The domain refused the command. The detail says why.",
    },
    ErrorDef {
        code: "rate_limited",
        http: 429,
        exit: 7,
        retryable: true,
        doc: "Too many requests. Honour Retry-After.",
    },
    ErrorDef {
        code: "store_unavailable",
        http: 500,
        exit: 8,
        retryable: true,
        doc: "The event log could not be read or written.",
    },
    ErrorDef {
        code: "unreachable",
        http: 0,
        exit: 8,
        retryable: true,
        doc: "Client-side only: no Node answered.",
    },
    ErrorDef {
        code: "internal",
        http: 500,
        exit: 70,
        retryable: false,
        doc: "A bug. Prints a correlation id and a report command.",
    },
];

pub const TYPES: &[TypeDef] = &[
    TypeDef {
        name: "Health",
        doc: "Liveness and version of a Node.",
        fields: &[
            Field { name: "status", ty: Ty::Enum(&["ok", "degraded"]), required: true,
                doc: "Whether the Runtime considers itself healthy." },
            Field { name: "runtime", ty: Ty::Str, required: true, doc: "Runtime version." },
            Field { name: "api_version", ty: Ty::Str, required: true, doc: "API major version." },
        ],
    },
    TypeDef {
        name: "Society",
        doc: "A Society — the atomic container (P1).",
        fields: &[
            Field { name: "society_id", ty: Ty::Str, required: true, doc: "`soc_` + ULID." },
            Field { name: "name", ty: Ty::Str, required: true, doc: "Display name, 1–64 characters." },
            Field { name: "handle", ty: Ty::Str, required: true, doc: "Globally unique, `@name`." },
            Field { name: "founder", ty: Ty::Str, required: true, doc: "FNID of the founding Citizen." },
            Field { name: "visibility", ty: Ty::Enum(&["public", "discoverable", "private", "sealed"]),
                required: true, doc: "Who may see and join it." },
            Field { name: "status", ty: Ty::Enum(&["forming", "active", "dormant", "fracturing", "dissolving", "archived"]),
                required: true, doc: "Lifecycle state (`docs/11 §4`)." },
            Field { name: "member_count", ty: Ty::Int, required: true, doc: "Members. Never below 1 while Active." },
            Field { name: "created_at", ty: Ty::Timestamp, required: true, doc: "When it was founded." },
            Field { name: "seq", ty: Ty::Int, required: true,
                doc: "Position in THIS Society's log. Per-Society, never global (`docs/10 §4`)." },
        ],
    },
    TypeDef {
        name: "CreateSocietyRequest",
        doc: "Found a Society.",
        fields: &[
            Field { name: "name", ty: Ty::Str, required: true, doc: "Display name, 1–64 characters." },
            Field { name: "handle", ty: Ty::Str, required: true, doc: "3–24 characters of a–z, 0–9, underscore." },
            Field { name: "visibility", ty: Ty::Enum(&["public", "discoverable", "private", "sealed"]),
                required: false, doc: "Defaults to discoverable." },
            Field { name: "idempotency_key", ty: Ty::Str, required: false,
                doc: "Makes the command safe to retry for 24 hours (`docs/10 §10`)." },
            Field { name: "founder_fnid", ty: Ty::Str, required: false,
                doc: "PH0 only. From PH1 the actor comes from the session (`docs/12`)." },
            Field { name: "societies_founded", ty: Ty::Int, required: false,
                doc: "PH0 only. How many this Citizen has already founded; the first-hearth rule reads it." },
            Field { name: "founder_level", ty: Ty::Int, required: false,
                doc: "PH0 only. Founding a second Society requires Level 3." },
        ],
    },
    TypeDef {
        name: "SocietyList",
        doc: "Every Society on this Node.",
        fields: &[Field { name: "societies", ty: Ty::Ref("Society"), required: true,
            doc: "Ordered by creation time." }],
    },
    TypeDef {
        name: "SocietyEnvelope",
        doc: "A single Society.",
        fields: &[Field { name: "society", ty: Ty::Ref("Society"), required: true, doc: "The Society." }],
    },
    TypeDef {
        name: "Meta",
        doc: "The machine-readable description of this Node's surface (`docs/31 §8`).",
        fields: &[
            Field { name: "runtime", ty: Ty::Str, required: true, doc: "Runtime version." },
            Field { name: "api_version", ty: Ty::Str, required: true, doc: "API major version." },
            Field { name: "phase", ty: Ty::Str, required: true, doc: "The phase this build implements." },
            Field { name: "operations", ty: Ty::Str, required: true,
                doc: "Every operation, generated from the schema. An agent plans against this rather than probing." },
        ],
    },
    TypeDef {
        name: "SocietyCreatedPayload",
        doc: "Payload of `society.created.v1`.",
        fields: &[
            Field { name: "society_id", ty: Ty::Str, required: true, doc: "`soc_` + ULID." },
            Field { name: "name", ty: Ty::Str, required: true, doc: "Display name as founded." },
            Field { name: "handle", ty: Ty::Str, required: true, doc: "Handle as claimed." },
            Field { name: "founder", ty: Ty::Str, required: true, doc: "FNID of the founding Citizen." },
            Field { name: "visibility", ty: Ty::Enum(&["public", "discoverable", "private", "sealed"]),
                required: true, doc: "Visibility at founding." },
            Field { name: "status", ty: Ty::Enum(&["active"]), required: true,
                doc: "A Society is Active from its first event: it has one member, its founder." },
            Field { name: "origin", ty: Ty::Enum(&["first_hearth", "founded", "crystallized", "fractured", "forked"]),
                required: true, doc: "How this Society came to exist (`docs/11 §2.2` Lineage)." },
        ],
    },
];

/// Every operation the platform offers.
///
/// Adding one here adds it to the `OpenAPI` document, the TypeScript client, the
/// CLI command tree and the Node's `/v1/meta` in the same commit. Forgetting one
/// of those is not possible.
pub const OPERATIONS: &[Operation] = &[
    Operation {
        id: "node.status",
        summary: "Liveness and version of this Node.",
        method: Method::Get,
        path: "/health",
        cli: CliBinding {
            noun: "node",
            verb: "status",
            args: &[],
            flags: &[],
        },
        request: None,
        response: "Health",
        capability: None,
        idempotent: true,
        dry_runnable: false,
        errors: &["unreachable"],
    },
    Operation {
        id: "node.meta",
        summary: "The machine-readable description of this Node's surface.",
        method: Method::Get,
        path: "/v1/meta",
        cli: CliBinding {
            noun: "node",
            verb: "meta",
            args: &[],
            flags: &[],
        },
        request: None,
        response: "Meta",
        capability: None,
        idempotent: true,
        dry_runnable: false,
        errors: &["unreachable"],
    },
    Operation {
        id: "society.list",
        summary: "Every Society this Node holds.",
        method: Method::Get,
        path: "/v1/societies",
        cli: CliBinding {
            noun: "society",
            verb: "list",
            args: &[],
            flags: &[],
        },
        request: None,
        response: "SocietyList",
        capability: Some("society.read"),
        idempotent: true,
        dry_runnable: false,
        errors: &["store_unavailable", "unreachable"],
    },
    Operation {
        id: "society.create",
        summary: "Found a Society.",
        method: Method::Post,
        path: "/v1/societies",
        cli: CliBinding {
            noun: "society",
            verb: "create",
            args: &["name"],
            flags: &["handle", "visibility", "idempotency-key"],
        },
        request: Some("CreateSocietyRequest"),
        response: "SocietyEnvelope",
        // Founding a Society enacts a Charter, and only a Citizen may sign one (P4).
        capability: Some("society.create"),
        idempotent: true,
        dry_runnable: true,
        errors: &[
            "invalid_handle",
            "invalid_fnid",
            "capability_denied",
            "rejected",
            "conflict",
            "store_unavailable",
            "unreachable",
        ],
    },
    Operation {
        id: "society.get",
        summary: "Read one Society.",
        method: Method::Get,
        path: "/v1/societies/{society_id}",
        cli: CliBinding {
            noun: "society",
            verb: "get",
            args: &["society_id"],
            flags: &[],
        },
        request: None,
        response: "SocietyEnvelope",
        capability: Some("society.read"),
        idempotent: true,
        dry_runnable: false,
        errors: &[
            "invalid_identifier",
            "not_found",
            "store_unavailable",
            "unreachable",
        ],
    },
];

/// Every domain event kind.
pub const EVENTS: &[EventDef] = &[EventDef {
    kind: "society.created.v1",
    version: 1,
    doc: "A Society was founded. The first event in every Society's log.",
    payload: "SocietyCreatedPayload",
}];

// ---------------------------------------------------------------------------
// Lookups
// ---------------------------------------------------------------------------

/// Find a type by name.
#[must_use]
pub fn type_def(name: &str) -> Option<&'static TypeDef> {
    let mut i = 0;
    while i < TYPES.len() {
        if let Some(t) = TYPES.get(i) {
            if str_eq(t.name, name) {
                return Some(t);
            }
        }
        i += 1;
    }
    None
}

/// Find an error by code.
#[must_use]
pub fn error_def(code: &str) -> Option<&'static ErrorDef> {
    let mut i = 0;
    while i < ERRORS.len() {
        if let Some(e) = ERRORS.get(i) {
            if str_eq(e.code, code) {
                return Some(e);
            }
        }
        i += 1;
    }
    None
}

fn str_eq(a: &str, b: &str) -> bool {
    a.as_bytes() == b.as_bytes()
}
