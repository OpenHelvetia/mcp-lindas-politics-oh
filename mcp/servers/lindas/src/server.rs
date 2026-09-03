//! The rmcp tool router: the eight tools of `TOOLSET-v0.md` §3 under
//! the `lindas.` domain, results as pretty JSON text; a typed refusal
//! surfaces as an MCP tool error carrying the typed JSON — machines
//! branch on `error`, never on prose.
//!
//! **Stage-one lines (E16 two-stage discovery).** Every description
//! below is §5 of the contract, VERBATIM: at most 160 characters,
//! verb-first, saying when to use the tool, ending with the answer
//! class — and carrying the German trigger words that make a chat
//! model reach for this domain rather than the legal one (Abstimmung,
//! Referendum, Volksinitiative, Ständemehr, Kanton, Bundesrat,
//! Nationalratswahl, Interessenbindung). The inventory carries `id`,
//! `domain`, `summary` and `weight` and nothing else, which is why the
//! triggers live in the lines.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{tool, tool_handler, tool_router, ErrorData, ServerHandler};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::domain::{self, Ctx};

pub struct LindasServer {
    pub ctx: std::sync::Arc<Ctx>,
}

impl Clone for LindasServer {
    fn clone(&self) -> Self {
        Self {
            ctx: self.ctx.clone(),
        }
    }
}

fn emit(result: anyhow::Result<serde_json::Value>) -> Result<CallToolResult, ErrorData> {
    match result {
        Ok(value) => {
            let text = serde_json::to_string_pretty(&value)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
            if value.get("error").is_some() {
                Ok(CallToolResult::error(vec![ContentBlock::text(text)]))
            } else {
                Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
            }
        }
        Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
            "{{\"error\":\"internal\",\"detail\":\"{e:#}\"}}"
        ))])),
    }
}

/// The house rule for a stage-one line, as a checkable predicate:
/// ≤ 160 characters, starts with an upper-case verb, says when to use
/// the tool, ends with the answer kind. The fedlex server's rule,
/// re-used so both domains read alike in one catalogue.
pub fn summary_conforms(summary: &str) -> Result<(), String> {
    let n = summary.chars().count();
    if n > 160 {
        return Err(format!("{n} characters, the stage-one limit is 160"));
    }
    if !summary.chars().next().is_some_and(char::is_uppercase) {
        return Err("must begin with the verb, capitalised".into());
    }
    if !(summary.ends_with(" norm.") || summary.ends_with(" hint.")) {
        return Err("must end with «norm.» or «hint.»".into());
    }
    if !summary.contains("use ") {
        return Err("must say WHEN to use the tool («use when/for/to/before …»)".into());
    }
    Ok(())
}

// ---------------------------------------------------------------------
// Parameter shapes (shared with the gateway mount)
//
// Every struct is closed against unknown keys (`deny_unknown_fields`),
// and that is a MEASURED lesson, not tidiness: `{"cube": …, "filter":
// "hasCanton=…"}` — singular, a plausible near-miss of `filters` — was
// accepted, the key silently dropped, and the caller handed the whole
// of Switzerland while believing it had filtered one canton: 18'366
// rows where 26 were asked for, no complaint anywhere in the payload
// (audit of 01.09.2026, the highest-ranked tool-surface defect). With
// the closed shape the same call is a typed `bad-request` that names
// the stray key and the accepted ones, because serde's own message
// already does.
// ---------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListCubesParams {
    /// One of `fc`, `fch/apg`, `national-council-election`,
    /// `political-rights`; absent = all 44.
    pub family: Option<String>,
    /// Label language (de|fr|it|en|rm); default de. The answer names
    /// the language it served, and «und» where a name carries no tag.
    pub lang: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FindCubeParams {
    /// A word of the cube's name, 2–100 characters
    /// («Abstimmung», «Interessenbindung»).
    pub query: String,
    pub lang: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CubeParams {
    /// A cube IRI of the served scope, e.g.
    /// `https://politics.ld.admin.ch/political-rights/popular-vote/1`.
    pub cube: String,
    pub lang: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DimensionValuesParams {
    pub cube: String,
    /// The dimension IRI as `lindas.describe_cube` served it — never
    /// built by appending to the cube IRI.
    pub dimension: String,
    pub lang: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservationsParams {
    pub cube: String,
    /// Filters as `dimension=value` pairs; the dimension is the full
    /// IRI `lindas.describe_cube` served, the value an IRI or a plain
    /// literal. A dimension the shape does not declare is admitted
    /// when the cube's observations carry it, and the answer says so.
    pub filters: Option<Vec<String>>,
    /// The dimension IRIs to project — the cells a row comes back
    /// with. Absent = every cell the row carries, which is up to 51 in
    /// the vote cube; naming the three or four a question needs is one
    /// call instead of paging the same table at a smaller limit.
    pub dimensions: Option<Vec<String>>,
    pub lang: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VersionsParams {
    /// Any version of the family, e.g.
    /// `…/national-council-election/candidates/2019`.
    pub cube: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IriParams {
    /// Any IRI — the store is asked about it; no host is fetched.
    pub iri: String,
    pub lang: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LabelParams {
    /// Any IRI, of any host (canton, country, legal form, gender).
    pub iri: String,
    pub lang: Option<String>,
}

/// `dimension=value` → the pair the domain filters on. The FIRST `=`
/// separates them, because a value may carry one.
///
/// Public because the gateway mount parses the same argument shape:
/// one rule, one home — a second parser at the gateway would be a
/// second contract for the same string.
///
/// # Errors
///
/// The pair that carries no `=`, named, so the refusal can quote it.
pub fn split_filters(filters: &[String]) -> Result<Vec<(String, String)>, serde_json::Value> {
    filters
        .iter()
        .map(|f| {
            f.split_once('=')
                .map(|(d, v)| (d.trim().to_string(), v.trim().to_string()))
                .ok_or_else(|| {
                    // The REFUSAL itself, not a string a caller has to
                    // wrap (BY′): this is the first thing a guessing
                    // model meets, and both doors — this server and the
                    // gateway — must answer it with the shape.
                    domain::invalid_dimension(&format!("filter «{f}» must be «dimension=value»"))
                })
        })
        .collect()
}

#[tool_router(vis = "pub")]
impl LindasServer {
    #[tool(
        name = "lindas.list_cubes",
        description = "List the 44 political data cubes of the Confederation (Abstimmungen, Wahlen, Bundesrat, Interessenbindungen): use to see what data exists. norm.",
        annotations(read_only_hint = true)
    )]
    pub async fn list_cubes(
        &self,
        Parameters(p): Parameters<ListCubesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        emit(domain::list_cubes(
            &self.ctx,
            p.family.as_deref(),
            p.lang.as_deref(),
            p.limit,
            p.offset,
        ))
    }

    #[tool(
        name = "lindas.find_cube",
        description = "Find the cube behind a question by a word of its name («Volksinitiative», «Petition», «Parteienregister»): use before reading rows. hint.",
        annotations(read_only_hint = true)
    )]
    pub async fn find_cube(
        &self,
        Parameters(p): Parameters<FindCubeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        emit(domain::find_cube(
            &self.ctx,
            &p.query,
            p.lang.as_deref(),
            p.limit,
        ))
    }

    #[tool(
        name = "lindas.describe_cube",
        description = "Show a cube's declared dimensions and profile — the record may carry more: use to learn the filters a question needs. norm.",
        annotations(read_only_hint = true)
    )]
    pub async fn describe_cube(
        &self,
        Parameters(p): Parameters<CubeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        emit(domain::describe_cube(
            &self.ctx,
            &p.cube,
            p.lang.as_deref(),
            p.limit,
            p.offset,
        ))
    }

    #[tool(
        name = "lindas.dimension_values",
        description = "List the values one dimension takes (Kantone, Abstimmungstypen, Geschäftsstände): use to filter by IRI instead of by text. hint.",
        annotations(read_only_hint = true)
    )]
    pub async fn dimension_values(
        &self,
        Parameters(p): Parameters<DimensionValuesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        emit(domain::dimension_values(
            &self.ctx,
            &p.cube,
            &p.dimension,
            p.lang.as_deref(),
            p.limit,
        ))
    }

    #[tool(
        name = "lindas.observations",
        description = "Read a cube's rows with filters (Abstimmung, Referendum, Volksinitiative, Ständemehr, Kanton, Datum): use for the figures themselves. norm.",
        annotations(read_only_hint = true)
    )]
    pub async fn observations(
        &self,
        Parameters(p): Parameters<ObservationsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let filters = match split_filters(p.filters.as_deref().unwrap_or_default()) {
            Ok(f) => f,
            Err(refusal) => return emit(Ok(refusal)),
        };
        emit(domain::observations(
            &self.ctx,
            &p.cube,
            &filters,
            &p.dimensions.unwrap_or_default(),
            p.lang.as_deref(),
            p.limit,
            p.offset,
        ))
    }

    #[tool(
        name = "lindas.list_versions",
        description = "List the versions of a cube family (Nationalratswahl 2019/2023/2027): use before reading a year; nothing links old to new. norm.",
        annotations(read_only_hint = true)
    )]
    pub async fn list_versions(
        &self,
        Parameters(p): Parameters<VersionsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        emit(domain::list_versions(&self.ctx, &p.cube))
    }

    #[tool(
        name = "lindas.describe",
        description = "Show everything the holding says about one IRI (a cube, an observation, a Kanton): use to follow an address you were handed. norm.",
        annotations(read_only_hint = true)
    )]
    pub async fn describe(
        &self,
        Parameters(p): Parameters<IriParams>,
    ) -> Result<CallToolResult, ErrorData> {
        emit(domain::describe(
            &self.ctx,
            &p.iri,
            p.lang.as_deref(),
            p.limit,
            p.offset,
        ))
    }

    #[tool(
        name = "lindas.resolve_label",
        description = "Resolve an IRI to its label in one language with a fallback (Kanton, Partei, Gremium, Interessenbindung): use to name a value. hint.",
        annotations(read_only_hint = true)
    )]
    pub async fn resolve_label(
        &self,
        Parameters(p): Parameters<LabelParams>,
    ) -> Result<CallToolResult, ErrorData> {
        emit(domain::resolve_label(&self.ctx, &p.iri, p.lang.as_deref()))
    }
}

#[tool_handler(
    name = "oh-mcp-lindas",
    version = "0.1.0",
    instructions = "OpenHelvetia LINDAS domain server, base tier: the 44 political \
         data cubes of politics.ld.admin.ch — Volksabstimmungen with the \
         Ständemehr, Referendumsvorlagen and their Geschäftsstände, \
         Volksinitiativen, Nationalratswahlen 2019/2023/2027, the Bundesrat \
         and the Federal Chancellery's register of Interessenbindungen. \
         Read-only, stateless, one host. The loop: find the cube \
         (find_cube or list_cubes) → learn its filters (describe_cube, \
         dimension_values) → read the rows (observations) → name a value \
         (resolve_label) or follow an address (describe). What this server \
         never does: arithmetic across observations — the Ständemehr is READ \
         from the row (15.5 : 6.5 on 07.02.1971, where counting cantons gives \
         17) and an outcome is read from its own dimension, never derived \
         from the yes share. Every answer carries provenance (served, as_of, \
         licence «not stated at the source», access «public (I14Y)»); every \
         list is capped with returned/total/truncated; «not stated» is \
         answered for both forms the holding writes (an IRI and an empty \
         typed literal), never as 0 or «». A cube can be published and hold \
         nothing (placeholder: true) and 14 of 44 carry no status \
         (status_unset) — both are answers. The SHACL shape is what a cube \
         DECLARES; its observations may carry more, so a filter on an \
         undeclared dimension is admitted when a bound ASK finds it and the \
         answer says undeclared_dimensions. A vote title written like a legal \
         citation is handed over verbatim with resolution: open — no server \
         resolves a dated act title yet. Discovery is two-stage: stage one is \
         the one-line inventory, stage two the input schemas of the tools you \
         intend to call. Live requests pass a polite brake (2 a second, burst \
         4, at most 5 s): a call that would wait longer answers the typed \
         upstream-busy with retry_after_ms. Policy (auth, rate, budget) lives \
         at the platform gateway, not here."
)]
impl ServerHandler for LindasServer {}
