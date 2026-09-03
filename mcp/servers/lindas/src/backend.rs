//! The one door to the outside: `https://lindas.admin.ch/query`, and
//! nothing else.
//!
//! Four backends, the shape the fedlex server proved (BQ/BS/BO′ —
//! `mcp/servers/fedlex/src/backend.rs`, the pattern adopted with
//! attribution under E15): `Live` asks the endpoint, `Fixtures` reads a
//! recorded answer by its SEMANTIC key, `Recording` does both (the
//! deliberate re-record pass) and `Counting` reads fixtures while
//! counting what a live run would have cost. The brake and the fixture
//! store come from `oh-mcp-common`; what lives here is this server's
//! own wording and its own host.
//!
//! Contract points this file carries (`TOOLSET-v0.md`): P31 (every
//! query binds a subject or a predicate — this module never builds an
//! unbound pattern), P32 (HTTP 406 is a typed refusal about the
//! request, not an outage), P33 (a value that came out of an answer is
//! normalised before it goes into the next request), P34 (the LIMIT is
//! the tool's), and §8's one host.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{bail, Context as _, Result};

pub use oh_mcp_common::fixtures::{fixture_file_name, index_line, key_file, now_rfc3339};
pub use oh_mcp_common::throttle::{
    busy_retry_after_ms, FrozenClock, UpstreamBusy, UpstreamThrottle, DEFAULT_UPSTREAM_BURST,
    DEFAULT_UPSTREAM_MAX_WAIT, DEFAULT_UPSTREAM_RATE,
};

/// The public LINDAS SPARQL endpoint — the ONE host this server ever
/// connects to (contract §8). Foreign IRIs are subjects this endpoint
/// is asked about; they are never fetched (P30).
pub const LINDAS_ENDPOINT: &str = "https://lindas.admin.ch/query";

/// The prefix every connect target must carry. A URL that does not is
/// refused before a socket is opened — the LINDAS counterpart of the
/// fedlex server's manifestation-host guard.
pub const ENDPOINT_HOST: &str = "https://lindas.admin.ch/";

/// The identifying agent, with the address that answers for it.
pub const USER_AGENT: &str =
    "oh-mcp-lindas/0.1 (+https://openhelvetia.swiss; base-tier domain server)";

/// A SELECT/ASK is bounded by the CALLER's patience, not the
/// endpoint's: the chat allows a tool call 15 s and the brake may
/// reserve 5 of them (J17.5's form, contract §8).
pub const SELECT_TIMEOUT: Duration = Duration::from_secs(15);

/// The heavier class — a description of one IRI can be 300 statements
/// wide (C6.3) — gets the fetch-equivalent bound.
pub const DESCRIBE_TIMEOUT: Duration = Duration::from_secs(30);

/// The refusal of the call path names its class and its bound…
pub fn select_class() -> String {
    format!("SPARQL select (timeout {} s)", SELECT_TIMEOUT.as_secs())
}

/// …the body half says so, with the same bound…
pub fn select_body_class() -> String {
    format!(
        "SPARQL result body (select, timeout {} s)",
        SELECT_TIMEOUT.as_secs()
    )
}

/// …and the describe class carries its own, with a body half of its
/// own too (BX′: a description was reading its body under the SELECT
/// class's wording, so half of every describe refusal named a bound
/// that had not been applied).
pub fn describe_class() -> String {
    format!("SPARQL describe (timeout {} s)", DESCRIBE_TIMEOUT.as_secs())
}

/// The body half of the describe class.
pub fn describe_body_class() -> String {
    format!(
        "SPARQL result body (describe, timeout {} s)",
        DESCRIBE_TIMEOUT.as_secs()
    )
}

/// The class PAIR a bound belongs to — the call half and the body
/// half. One place decides it, and a test can call it: the choice used
/// to be re-derived from a `Duration` at the call site and only for
/// the call half, which is how a describe came to report «select,
/// timeout 15 s» for a body it read under a 30 s bound.
pub fn classes_of(timeout: Duration) -> (String, String) {
    if timeout == DESCRIBE_TIMEOUT {
        (describe_class(), describe_body_class())
    } else {
        (select_class(), select_body_class())
    }
}

/// The refusal as this backend raises it (the wording names THIS
/// server's host; the mechanism is `oh-mcp-common`).
pub fn busy_message(throttle: &UpstreamThrottle, busy: UpstreamBusy) -> String {
    let ms = busy.retry_after.as_millis();
    format!(
        "upstream-busy: retry_after_ms={ms}: the polite brake against \
         lindas.admin.ch is saturated ({} live requests/s, burst {}); this request \
         would have waited longer than {} s — retry after {ms} ms",
        throttle.rate_per_second(),
        throttle.burst(),
        throttle.max_wait().as_secs_f64()
    )
}

/// How an answer was served, carried into the provenance of every
/// answer (contract §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Served {
    Live,
    Fixture,
}

impl Served {
    pub fn as_str(self) -> &'static str {
        match self {
            Served::Live => "live",
            Served::Fixture => "fixture",
        }
    }
}

/// One answer of the endpoint, with the moment it was retrieved (live
/// only — a fixture carries the moment of ITS recording, in
/// `INDEX.txt`).
pub struct Answer {
    pub value: serde_json::Value,
    pub served: Served,
    pub retrieved_at: Option<String>,
    /// The query that produced this answer, verbatim.
    ///
    /// **Why an answer carries its own question.** «Source:
    /// lindas.admin.ch» is a claim a reader cannot check; the query is
    /// the same claim made checkable — with it and the endpoint, one
    /// `curl` reproduces the figure, and a figure that cannot be
    /// reproduced is not evidence. A fixture carries it too: what it
    /// names is what a live run would send, which is what makes a
    /// recorded answer auditable rather than merely convenient.
    pub query: String,
}

pub enum Backend {
    /// Live queries against the one endpoint, braked.
    Live {
        endpoint: String,
        throttle: UpstreamThrottle,
    },
    /// Recorded answers under `dir/<key-hash>.json` — the test path.
    Fixtures { dir: PathBuf },
    /// Live, and every answer is also written as a fixture with its
    /// date in `INDEX.txt` (the deliberate recording pass). Braked.
    Recording {
        endpoint: String,
        dir: PathBuf,
        throttle: UpstreamThrottle,
    },
    /// Reads fixtures and COUNTS what a live run would have cost: the
    /// request budget of a recording pass is measured, never trusted
    /// (the fedlex pattern, BV A′).
    Counting {
        dir: PathBuf,
        selects: Arc<AtomicUsize>,
        seen_keys: Mutex<Vec<String>>,
        throttle: Option<UpstreamThrottle>,
    },
}

impl Backend {
    /// The live backend with the default polite brake (2/s, burst 4,
    /// five seconds of patience).
    pub fn live(endpoint: impl Into<String>) -> Self {
        Backend::Live {
            endpoint: endpoint.into(),
            throttle: UpstreamThrottle::default_polite(),
        }
    }

    /// The live backend with a brake of the caller's choosing.
    pub fn live_with_throttle(endpoint: impl Into<String>, throttle: UpstreamThrottle) -> Self {
        Backend::Live {
            endpoint: endpoint.into(),
            throttle,
        }
    }

    /// The recording pass: live, braked, every answer written.
    pub fn recording(endpoint: impl Into<String>, dir: impl Into<PathBuf>) -> Self {
        Backend::Recording {
            endpoint: endpoint.into(),
            dir: dir.into(),
            throttle: UpstreamThrottle::default_polite(),
        }
    }

    /// The counting double: fixtures in, requests counted.
    pub fn counting(dir: impl Into<PathBuf>) -> (Self, Arc<AtomicUsize>) {
        let selects = Arc::new(AtomicUsize::new(0));
        (
            Backend::Counting {
                dir: dir.into(),
                selects: selects.clone(),
                seen_keys: Mutex::new(Vec::new()),
                throttle: None,
            },
            selects,
        )
    }

    /// Every fixture key the counting double was asked for, in order.
    /// Empty for every other backend.
    pub fn seen_keys(&self) -> Vec<String> {
        match self {
            Backend::Counting { seen_keys, .. } => {
                seen_keys.lock().expect("seen lock not poisoned").clone()
            }
            _ => Vec::new(),
        }
    }

    /// The endpoint a reader would send this server's queries to.
    ///
    /// A fixture backend names the canonical one: a recording is a
    /// recording OF that endpoint, and the point of naming it is that
    /// somebody can go and ask it the same question themselves.
    pub fn endpoint(&self) -> &str {
        match self {
            Backend::Live { endpoint, .. } | Backend::Recording { endpoint, .. } => endpoint,
            Backend::Fixtures { .. } | Backend::Counting { .. } => LINDAS_ENDPOINT,
        }
    }

    fn throttle(&self) -> Option<&UpstreamThrottle> {
        match self {
            Backend::Live { throttle, .. } | Backend::Recording { throttle, .. } => Some(throttle),
            Backend::Counting { throttle, .. } => throttle.as_ref(),
            Backend::Fixtures { .. } => None,
        }
    }

    /// Takes a token from the brake before a live request — or raises
    /// the `upstream-busy` text the domain types.
    fn brake(&self) -> Result<()> {
        let Some(throttle) = self.throttle() else {
            return Ok(());
        };
        throttle
            .acquire()
            .map(|_| ())
            .map_err(|busy| anyhow::anyhow!("{}", busy_message(throttle, busy)))
    }

    /// Runs a SELECT or an ASK at the select bound (15 s). `key` is
    /// the stable semantic fixture key (`<tool>:<cube version
    /// IRI>:<arguments>`, contract §2); the query is what actually
    /// runs.
    pub fn select(&self, key: &str, query: &str) -> Result<Answer> {
        self.select_within(key, query, SELECT_TIMEOUT)
    }

    /// The same, at the DESCRIBE bound (30 s) — §8's «anything that
    /// reads a body», which in this server is the description PAGE:
    /// up to 400 statements with their labels.
    ///
    /// **Why this exists (BX').** The contract declares two timeout
    /// classes and the crate carries both constants, both refusal
    /// strings and a test that asserts all four — but until this
    /// method every query ran at 15 s, so `describe_class()` was a
    /// sentence no code could produce. The measurement is on the side
    /// of the wider bound: a description is the class where one
    /// subject carries 304 predicates (C6.3) and where the count is a
    /// DISTINCT aggregate over everything a subject says, so the code
    /// is made to match the declaration rather than the declaration
    /// trimmed to the code.
    pub fn describe_within(&self, key: &str, query: &str) -> Result<Answer> {
        self.select_within(key, query, DESCRIBE_TIMEOUT)
    }

    /// The one request path; the timeout is the CLASS of the query,
    /// and it is what the refusal names when the bound is reached.
    fn select_within(&self, key: &str, query: &str, timeout: Duration) -> Result<Answer> {
        match self {
            Backend::Live { endpoint, .. } => {
                self.brake()?;
                Ok(Answer {
                    value: live_select(endpoint, query, timeout)?,
                    served: Served::Live,
                    retrieved_at: Some(now_rfc3339()),
                    query: query.to_string(),
                })
            }
            Backend::Fixtures { dir } => Ok(Answer {
                value: read_fixture(dir, key)?,
                served: Served::Fixture,
                retrieved_at: None,
                query: query.to_string(),
            }),
            Backend::Counting {
                dir,
                selects,
                seen_keys,
                ..
            } => {
                self.brake()?;
                selects.fetch_add(1, Ordering::SeqCst);
                seen_keys
                    .lock()
                    .expect("seen lock not poisoned")
                    .push(key.to_string());
                Ok(Answer {
                    value: read_fixture(dir, key)?,
                    served: Served::Fixture,
                    retrieved_at: None,
                    query: query.to_string(),
                })
            }
            Backend::Recording { endpoint, dir, .. } => {
                self.brake()?;
                let value = live_select(endpoint, query, timeout)?;
                std::fs::create_dir_all(dir)?;
                let path = key_file(dir, key);
                let mut pretty = serde_json::to_string_pretty(&value)?;
                pretty.push('\n');
                std::fs::write(&path, pretty)?;
                // The key→file mapping stays human-auditable, with the
                // day the recording was made.
                index_line(dir, &path, key)?;
                Ok(Answer {
                    value,
                    served: Served::Live,
                    retrieved_at: Some(now_rfc3339()),
                    query: query.to_string(),
                })
            }
        }
    }

    /// The bindings of a SELECT answer, or an upstream fault.
    pub fn bindings(value: &serde_json::Value) -> Result<&Vec<serde_json::Value>> {
        value
            .get("results")
            .and_then(|r| r.get("bindings"))
            .and_then(|b| b.as_array())
            .ok_or_else(|| anyhow::anyhow!("upstream-unavailable: no bindings in the answer"))
    }

    /// The boolean of an ASK answer.
    pub fn boolean(value: &serde_json::Value) -> Result<bool> {
        value
            .get("boolean")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow::anyhow!("upstream-unavailable: no boolean in the answer"))
    }
}

/// A value that came out of an answer before it goes into the next
/// request: the endpoint's CSV is CRLF and its JSON can carry stray
/// whitespace, so nothing is pasted raw (P33, C12.4).
pub fn normalise_value(raw: &str) -> String {
    raw.trim_matches(|c: char| c == '\r' || c == '\n' || c == '\u{feff}')
        .trim()
        .to_string()
}

/// An IRI this server may put into a query: syntactically sound, and
/// carrying nothing that could break out of `<…>`. The HOST is not
/// checked — a foreign IRI is a legitimate SUBJECT (P30) — only the
/// shape is.
pub fn iri_safe(iri: &str) -> Result<&str> {
    let trimmed = iri.trim();
    if trimmed.is_empty() {
        bail!("an IRI is required");
    }
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        bail!("«{trimmed}» is no IRI (http/https expected)");
    }
    if trimmed
        .chars()
        .any(|c| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '{' | '}' | '\\' | '|' | '^'))
    {
        bail!("«{trimmed}» carries characters that cannot stand in an IRI");
    }
    Ok(trimmed)
}

/// The literal escape for a bound value inside a query.
pub fn literal_safe(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\n', '\r'], " ")
}

fn live_select(endpoint: &str, query: &str, timeout: Duration) -> Result<serde_json::Value> {
    // §8: ONE host. A target that is not it never reaches a socket.
    if !endpoint.starts_with(ENDPOINT_HOST) {
        bail!("invalid-input: «{endpoint}» is not the one host this server speaks to");
    }
    let (class, body_class) = classes_of(timeout);
    let mut response = ureq::post(endpoint)
        .config()
        .timeout_global(Some(timeout))
        .build()
        .header("accept", "application/sparql-results+json")
        .header("user-agent", USER_AGENT)
        .send_form([("query", query)])
        .map_err(|e| match e {
            // The endpoint REJECTED the request: a permanent answer
            // ABOUT the request — a malformed query (400) or a
            // serialisation it cannot serve (406, C12.2) — never an
            // outage. Kept apart so the domain types it invalid-input.
            ureq::Error::StatusCode(code) if (400..500).contains(&code) => {
                anyhow::anyhow!("bad-request: HTTP {code}")
            }
            other => anyhow::anyhow!("upstream-unavailable: {class}: {other}"),
        })?;
    response
        .body_mut()
        .read_json::<serde_json::Value>()
        .map_err(|e| anyhow::anyhow!("upstream-unavailable: {body_class}: {e}"))
}

fn read_fixture(dir: &std::path::Path, key: &str) -> Result<serde_json::Value> {
    let path = key_file(dir, key);
    let raw = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "fixture missing for key «{key}» ({}) — run the recording pass \
             (cargo test --test e2e record_fixtures -- --ignored)",
            path.display()
        )
    })?;
    Ok(serde_json::from_str(&raw)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bounds are the CALLER's patience, and every refusal names
    /// its class and its bound (contract §8, J17.5's form).
    #[test]
    fn the_timeout_constants_bound_every_live_request() {
        assert_eq!(SELECT_TIMEOUT, Duration::from_secs(15));
        assert_eq!(DESCRIBE_TIMEOUT, Duration::from_secs(30));
        assert!(SELECT_TIMEOUT >= DEFAULT_UPSTREAM_MAX_WAIT + Duration::from_secs(5));
        // The wide class is held to the same arithmetic (BX′): it is a
        // bound on ONE request, the brake may reserve five seconds
        // before it, and it is the wider of the two.
        assert!(DESCRIBE_TIMEOUT >= DEFAULT_UPSTREAM_MAX_WAIT + Duration::from_secs(5));
        assert!(DESCRIBE_TIMEOUT > SELECT_TIMEOUT);
        assert_eq!(select_class(), "SPARQL select (timeout 15 s)");
        assert_eq!(
            select_body_class(),
            "SPARQL result body (select, timeout 15 s)"
        );
        assert_eq!(describe_class(), "SPARQL describe (timeout 30 s)");
        assert_eq!(
            describe_body_class(),
            "SPARQL result body (describe, timeout 30 s)"
        );
        // Both halves of a bound belong to the same class — the defect
        // BX′ found was a describe reading its body under the select
        // class's wording.
        assert_eq!(
            classes_of(DESCRIBE_TIMEOUT),
            (describe_class(), describe_body_class())
        );
        assert_eq!(
            classes_of(SELECT_TIMEOUT),
            (select_class(), select_body_class())
        );
        assert!(USER_AGENT.contains("openhelvetia.swiss"));
        assert!(USER_AGENT.starts_with("oh-mcp-lindas/"));
    }

    /// The brake's refusal names THIS server's host and carries a
    /// machine-readable retry.
    #[test]
    fn the_busy_message_names_this_host_and_a_retry() {
        let brake = UpstreamThrottle::default_polite();
        let text = busy_message(
            &brake,
            UpstreamBusy {
                retry_after: Duration::from_millis(1500),
            },
        );
        assert!(text.contains("lindas.admin.ch"), "{text}");
        assert!(
            !text.contains("fedlex"),
            "the wording is this server's: {text}"
        );
        assert_eq!(busy_retry_after_ms(&text), Some(1500));
    }

    /// §8: a connect target that is not the one host never opens a
    /// socket — the check is before the request, not after it.
    #[test]
    fn only_one_host_is_ever_a_connect_target() {
        let refused = live_select("https://evil.example/query", "ASK {}", SELECT_TIMEOUT)
            .expect_err("a foreign host is refused");
        assert!(
            refused.to_string().contains("not the one host"),
            "{refused}"
        );
        assert!(ENDPOINT_HOST.starts_with("https://"));
        assert!(LINDAS_ENDPOINT.starts_with(ENDPOINT_HOST));
    }

    /// A value out of an answer is normalised before it is used again
    /// (P33): the endpoint's CSV is CRLF.
    #[test]
    fn a_value_from_an_answer_is_normalised_before_it_is_reused() {
        assert_eq!(
            normalise_value("https://politics.ld.admin.ch/fc/cube-president\r\n"),
            "https://politics.ld.admin.ch/fc/cube-president"
        );
        assert_eq!(normalise_value("  spaced  "), "spaced");
        assert_eq!(normalise_value("\u{feff}iri"), "iri");
    }

    /// An IRI is checked for SHAPE, never for host — a foreign IRI is
    /// a legitimate subject (P30).
    #[test]
    fn an_iri_is_checked_for_shape_and_not_for_host() {
        assert!(iri_safe("https://register.ld.admin.ch/i14y/concept/sex/2").is_ok());
        assert!(iri_safe("https://ld.admin.ch/canton/12").is_ok());
        assert!(iri_safe("not-an-iri").is_err());
        assert!(iri_safe("https://a.example/x y").is_err());
        assert!(iri_safe("https://a.example/x>y").is_err());
        assert!(iri_safe("").is_err());
    }
}
