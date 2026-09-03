//! The eight tools of `TOOLSET-v0.md` §3, and the cross-cutting rules
//! of §2 as code.
//!
//! Every function here is a contract point with a number: the module is
//! the specification's implementation, and `tests/contract_table.rs`
//! holds the two against each other. Nothing in this file reads a
//! clock (the moment is injected as `Ctx::today`), nothing builds an
//! unbound query (P31), nothing computes across observations (P20) and
//! nothing constructs a dimension IRI (P12).

use anyhow::Result;
use serde_json::{json, Value};

use crate::backend::{iri_safe, literal_safe, normalise_value, Answer, Backend};
use crate::scope;

/// The prefixes every query of this server carries.
pub const PREFIXES: &str = "PREFIX cube: <https://cube.link/>\n\
     PREFIX sh: <http://www.w3.org/ns/shacl#>\n\
     PREFIX schema: <http://schema.org/>\n\
     PREFIX skos: <http://www.w3.org/2004/02/skos/core#>\n\
     PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
     PREFIX qudt: <http://qudt.org/schema/qudt/>\n\
     PREFIX dcat: <http://www.w3.org/ns/dcat#>\n\
     PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n";

/// «Not stated», in both the forms this holding writes (C3.1, P13).
pub const UNDEFINED: &str = "https://cube.link/Undefined";

/// The languages a label is looked for in, in order (P28).
pub const LABEL_FALLBACK: [&str; 5] = ["de", "fr", "it", "en", "rm"];

/// Caps (contract §3). Every list answer carries the original size.
pub const MAX_CUBES: usize = 100;
pub const MAX_HITS: usize = 50;
pub const MAX_DIMENSIONS: usize = 100;
pub const MAX_VALUES: usize = 200;
pub const MAX_OBSERVATIONS: usize = 200;
pub const MAX_STATEMENTS: usize = 400;
/// How many dimensions one call may project (BY′ point 3). Every
/// UNDECLARED one costs a bound `ASK` before the page is asked for, so
/// an uncapped list is a request multiplier a caller controls. The
/// widest row of this holding carries 51 cells and the widest shape
/// declares 14; two dozen is more than any question has needed, and it
/// is refused before a single request.
pub const MAX_PROJECTED: usize = 24;
/// And how many FILTERS one call may carry (BY′): the same multiplier
/// — every undeclared dimension a filter names costs a bound `ASK` —
/// plus a query string that grows with each clause. Capping one and
/// not the other would have been a cap in name only.
pub const MAX_FILTERS: usize = 24;

/// What every answer carries about where it came from (contract §2):
/// the injected moment, how it was served, and the two facts the entry
/// records about this source (P38 and the I14Y access right).
pub struct Ctx {
    pub backend: Backend,
    /// «Today» — injected, never read from a clock in this library.
    pub today: String,
}

/// The provenance block of every answer.
///
/// **Why the query is in here.** «Source: lindas.admin.ch» is a claim
/// about where a number came from, and a reader has no way to check
/// it: the store holds millions of statements and the answer holds
/// twelve. With the endpoint and the query, the same reader runs one
/// command and gets the same twelve — or does not, and has caught us.
/// That is the difference between citing a source and being auditable,
/// and it costs a few hundred bytes.
fn provenance(ctx: &Ctx, answer: &Answer) -> Value {
    json!({
        "source": "lindas.admin.ch/query",
        "served": answer.served.as_str(),
        "retrieved_at": answer.retrieved_at,
        "as_of": ctx.today,
        "licence": "not stated at the source",
        "access": "public (I14Y)",
        "endpoint": ctx.backend.endpoint(),
        "query": answer.query,
    })
}

// --- the four typed refusals (contract §2) --------------------------

pub fn not_found(subject: &str) -> Value {
    json!({"error": "not-found", "subject": subject})
}

fn not_found_dimension(cube: &str, dimension: &str) -> Value {
    json!({
        "error": "not-found",
        "subject": {"cube": cube, "dimension": dimension},
        "detail": "the cube's observations do not carry this dimension — asked with a bound ASK \
                   before answering (P12); lindas.describe_cube lists what the shape declares and \
                   samples what an observation carries"
    })
}

pub fn invalid(detail: &str) -> Value {
    json!({"error": "invalid-input", "detail": detail})
}

/// The refusal for a CUBE that is no served cube IRI (BY′ point 6).
///
/// The other place a guessing model starts: it costs no request, and
/// until now it said only that the string was no IRI.
pub fn invalid_cube(detail: &str) -> Value {
    json!({
        "error": "invalid-input",
        "detail": detail,
        "accepted": "<a cube IRI of the served list, as lindas.list_cubes serves it>",
        "note": "lindas.list_cubes serves the 44 cubes of this scope with their names and states, \
                 and lindas.find_cube finds one by a word of its name. A cube IRI is never built \
                 by hand: the version is the last segment where there is one, and nothing links \
                 an old version to a new one. This refusal costs no request"
    })
}

/// The refusal for a FILTER, with the shape a filter has (BY point 0,
/// from the first live measurement).
///
/// A model that guessed short names and quoted literals spent a
/// refusal — and, for an unknown dimension, an `ASK` — on every guess,
/// because the refusal said what was wrong and never what was right.
/// It says both now, and it is raised BEFORE any request: a dimension
/// that is not an IRI is decidable without asking the store anything.
pub fn invalid_dimension(detail: &str) -> Value {
    refusal_with_shape(
        detail,
        "{dimension: <full IRI as describe_cube served it>, value: <IRI or plain literal>}",
        "the dimension is the IRI lindas.describe_cube served under «dimensions», never a short \
         name and never a prefixed name; the value is an IRI (matched exactly) or a plain literal \
         (matched on its lexical form, so «1971-02-07» finds the date whichever of the three \
         datatypes it carries). lindas.dimension_values lists the values one dimension takes. \
         This refusal costs no request",
    )
}

/// The refusal for a PROJECTION — a list of bare dimension IRIs, not
/// `{dimension, value}` pairs (BY′: the cap refusal advertised the
/// filter's shape, and named «no IRI» as the cause where the cause was
/// the count).
pub fn invalid_projection(detail: &str) -> Value {
    refusal_with_shape(
        detail,
        "[<full IRI as describe_cube served it>, …] — bare dimension IRIs, at most 24",
        "«dimensions» is the list of cells a row should come back with, not a filter: no values, \
         no pairs. The IRIs are the ones lindas.describe_cube serves under «dimensions»; one the \
         shape does not declare is admitted the same way a filter is, by a bound ASK, which is \
         why the list is capped. This refusal costs no request",
    )
}

/// The refusal for a DIMENSION named on its own — `dimension_values`
/// takes a cube and one dimension IRI, and no value at all.
pub fn invalid_dimension_name(detail: &str) -> Value {
    refusal_with_shape(
        detail,
        "<full IRI as describe_cube served it> — one dimension IRI, no value",
        "lindas.describe_cube serves the dimension IRIs of a cube under «dimensions»; this tool \
         answers the values ONE of them takes, so it takes the cube and that IRI and nothing \
         else. This refusal costs no request",
    )
}

/// The refusal for a LIST THAT IS TOO LONG (BY″).
///
/// A cap rejects a NUMBER, and the refusal must say so: the first
/// version of this handed back the shape of a filter and named «no
/// IRI» as the cause, where every name given was a well-formed IRI and
/// the cause was the count. What is accepted is «at most N», and the
/// note says what each item would have cost.
pub fn too_many(detail: &str, cap: usize, what: &str, why: &str, helper: &str) -> Value {
    json!({
        "error": "invalid-input",
        "detail": format!("{detail} and at most {cap} may be"),
        // What a cap decides is a COUNT — and NOTHING else. The first
        // version added «every one of the names given was well formed»,
        // which the filter path had not looked at yet (BY‴): a refusal
        // may not report a check it did not make.
        "accepted": format!("at most {cap} {what} in one call — a COUNT: this refusal is about \
                             how many were named and about nothing else"),
        "note": format!("{why}. {helper} serves what a shorter list should name. This refusal \
                         costs no request: a count is decidable before the store is asked \
                         anything")
    })
}

/// The one shape of a refusal that says what it would have accepted.
fn refusal_with_shape(detail: &str, accepted: &str, note: &str) -> Value {
    json!({
        "error": "invalid-input",
        "detail": detail,
        "accepted": accepted,
        "note": note
    })
}

fn upstream(detail: String) -> Value {
    json!({"error": "upstream-unavailable", "detail": detail})
}

/// The backend's error text → this server's typed refusal. A 4xx is a
/// permanent answer ABOUT the request (P32, C12.2), the brake's refusal
/// carries its retry.
fn backend_refusal(error: &anyhow::Error) -> Value {
    let text = format!("{error:#}");
    if let Some(ms) = crate::backend::busy_retry_after_ms(&text) {
        return json!({
            "error": "upstream-busy",
            "detail": text,
            "retry_after_ms": ms,
        });
    }
    if text.contains("bad-request: HTTP 406") {
        return invalid(
            "the endpoint cannot serve this request in the serialisation asked for (HTTP 406) — \
             a permanent answer about the request, not an outage",
        );
    }
    if text.contains("bad-request: HTTP") {
        return invalid(&format!("the endpoint refused the request: {text}"));
    }
    upstream(text)
}

/// `ctx.backend.select` with the refusal already typed.
fn ask(ctx: &Ctx, key: &str, query: &str) -> std::result::Result<Answer, Value> {
    ctx.backend
        .select(key, query)
        .map_err(|e| backend_refusal(&e))
}

/// The same, at the DESCRIBE bound (§8's heavier class, 30 s): what a
/// subject says about itself is where one IRI carries 304 predicates
/// (C6.3), and the count is a DISTINCT aggregate over all of them.
fn ask_wide(ctx: &Ctx, key: &str, query: &str) -> std::result::Result<Answer, Value> {
    ctx.backend
        .describe_within(key, query)
        .map_err(|e| backend_refusal(&e))
}

// --- reading a SPARQL answer ----------------------------------------

/// One cell of an answer: what it is, and whether it is STATED at all.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub value: Option<String>,
    pub stated: bool,
    /// `iri` | `literal`, and for a not-stated cell WHICH of the two
    /// forms the holding used (C3.1 — the difference is carried, not
    /// interpreted).
    pub form: &'static str,
    pub datatype: Option<String>,
    pub lang: Option<String>,
}

/// One cell of an answer with the labels the store carries for it:
/// the dimension (or predicate), the value, and its labels by
/// language. Named because a tuple of three is where a type starts.
pub type LabelledCell = (String, Cell, Vec<(String, String)>);

/// Reads one binding variable into a [`Cell`], recognising all FOUR
/// shapes of «not stated»: the IRI `cube:Undefined`, an empty literal
/// typed `cube:Undefined`, an empty PLAIN literal, and an empty
/// literal typed plain `xsd:string` (P13; C3.1 as corrected by §17.2
/// of the rulebook — the first two are self-declaring, the other two
/// are silence served as a value unless this function refuses it).
pub fn cell(binding: &Value, var: &str) -> Option<Cell> {
    let node = binding.get(var)?;
    let kind = node
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("literal");
    let raw = node
        .get("value")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let datatype = node
        .get("datatype")
        .and_then(Value::as_str)
        .map(str::to_string);
    let lang = node
        .get("xml:lang")
        .and_then(Value::as_str)
        .filter(|l| !l.is_empty())
        .map(str::to_string);
    let is_iri = kind == "uri";
    if is_iri && raw == UNDEFINED {
        return Some(Cell {
            value: None,
            stated: false,
            form: "iri",
            datatype: None,
            lang: None,
        });
    }
    if datatype.as_deref() == Some(UNDEFINED) {
        return Some(Cell {
            value: None,
            stated: false,
            form: "literal",
            datatype: Some(UNDEFINED.to_string()),
            lang: None,
        });
    }
    // **An empty lexical form is «not stated», whatever its type.**
    // C3.1 named two shapes of missing and the holding has four
    // (§17.2, measured): beside the two cube:Undefined forms, an empty
    // PLAIN literal (`mmDate`, `mmId` on the national vote rows) and
    // an empty literal typed plain xsd:string (`pr:fax` in the party
    // register). Reading only the first two served the register's
    // silence as `stated: true, value: ""` — a blank rendered as a
    // value, the honesty counters wrong by exactly the cells that
    // matter, and `dimension_values` offering the blank as a
    // filterable value. The datatype it arrived with is kept, because
    // WHICH shape of empty a cell used is itself a fact of the store.
    if !is_iri && raw.is_empty() {
        return Some(Cell {
            value: None,
            stated: false,
            form: "literal",
            datatype,
            lang,
        });
    }
    Some(Cell {
        value: Some(raw.to_string()),
        stated: true,
        form: if is_iri { "iri" } else { "literal" },
        datatype,
        lang,
    })
}

/// The plain string of a binding variable, when it is stated.
pub fn text(binding: &Value, var: &str) -> Option<String> {
    cell(binding, var).and_then(|c| c.value)
}

/// A cell as the answer serves it (P13, P14: the lexical form, its
/// datatype, its unit — never a parsed number).
fn cell_json(dimension: &str, c: &Cell, label: Option<(&str, &str)>) -> Value {
    let mut out = json!({
        "dimension": dimension,
        "stated": c.stated,
        "form": c.form,
    });
    let map = out.as_object_mut().expect("object");
    if let Some(value) = &c.value {
        map.insert("value".into(), json!(value));
    }
    if let Some(datatype) = &c.datatype {
        map.insert("datatype".into(), json!(datatype));
    }
    if let Some(lang) = &c.lang {
        map.insert("lang".into(), json!(lang));
    }
    if let Some((label, lang)) = label {
        map.insert("label".into(), json!(label));
        map.insert("label_lang".into(), json!(lang));
    }
    out
}

/// The label to serve, and the language that answered (P28): the
/// caller's language first, then de → fr → it → en → rm, then any.
/// An untagged literal counts as `und` and is never dropped (P2).
pub fn choose_label(found: &[(String, String)], first: Option<&str>) -> Option<(String, String)> {
    let mut order: Vec<&str> = Vec::new();
    if let Some(f) = first {
        order.push(f);
    }
    order.extend(LABEL_FALLBACK.iter().copied().filter(|l| Some(*l) != first));
    for want in order {
        if let Some((lang, label)) = found.iter().find(|(lang, _)| lang == want) {
            return Some((label.clone(), lang.clone()));
        }
    }
    found.first().map(|(lang, label)| {
        (
            label.clone(),
            if lang.is_empty() {
                "und".to_string()
            } else {
                lang.clone()
            },
        )
    })
}

/// `VALUES ?cube { <…> }` over a list of IRIs — the bound form every
/// scope-wide query uses (P31: never a prefix filter over the store).
fn values_block(var: &str, iris: &[String]) -> String {
    let mut out = format!("VALUES ?{var} {{");
    for iri in iris {
        out.push_str(&format!(" <{iri}>"));
    }
    out.push_str(" }");
    out
}

/// The language a caller asked for, or `de`; anything outside the five
/// the holding writes is `invalid-input` — decidable without a request.
fn language(lang: Option<&str>) -> std::result::Result<&str, Value> {
    match lang.unwrap_or("de") {
        l if LABEL_FALLBACK.contains(&l) => Ok(LABEL_FALLBACK
            .iter()
            .find(|f| **f == l)
            .expect("just matched")),
        other => Err(refusal_with_shape(
            &format!("lang «{other}» is none of the five"),
            "de | fr | it | en | rm",
            "the five languages this holding writes (C4.2, C4.3); an untagged label answers as \
             «und». lindas.describe_cube serves a cube's names in the languages it has. This \
             refusal costs no request",
        )),
    }
}

/// A served cube IRI, or the refusal that says why not (P1).
fn served_cube(cube: &str) -> std::result::Result<String, Value> {
    let cube = match iri_safe(cube) {
        Ok(c) => normalise_value(c),
        Err(e) => return Err(invalid_cube(&format!("cube: {e:#}"))),
    };
    if !scope::is_served(&cube) {
        return Err(not_found(&cube));
    }
    Ok(cube)
}

/// The cap a caller asked for, clamped to this tool's own.
fn capped(limit: Option<u32>, max: usize, default: usize) -> usize {
    limit.map_or(default, |l| (l as usize).clamp(1, max))
}

// --- 3.1 list_cubes -------------------------------------------------

/// `lindas.list_cubes` — the served scope, and what each cube says
/// about itself (P1, P2, P3, P5, P7).
pub fn list_cubes(
    ctx: &Ctx,
    family: Option<&str>,
    lang: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Value> {
    let lang = match language(lang) {
        Ok(l) => l,
        Err(refusal) => return Ok(refusal),
    };
    let Some(cubes) = scope::of_family(family) else {
        return Ok(refusal_with_shape(
            &format!(
                "family «{}» is none of the four",
                family.unwrap_or_default()
            ),
            "fc | fch/apg | national-council-election | political-rights",
            "the four families of the served scope (C0.3); lindas.list_cubes without a family \
             serves all 44 and says which family each belongs to. This refusal costs no request",
        ));
    };
    let scope_name = family.unwrap_or("all");
    let profiles = match cube_profiles(ctx, &cubes, scope_name, lang) {
        Ok(p) => p,
        Err(refusal) => return Ok(refusal),
    };
    let counts = match observation_counts(ctx, &cubes, scope_name) {
        Ok(c) => c,
        Err(refusal) => return Ok(refusal),
    };
    let total = cubes.len();
    let limit = capped(limit, MAX_CUBES, 50);
    let offset = offset.unwrap_or(0) as usize;
    let page: Vec<Value> = cubes
        .iter()
        .skip(offset)
        .take(limit)
        .map(|cube| profiles.row(cube, counts.get(cube), lang))
        .collect();
    let returned = page.len();
    Ok(json!({
        "cubes": page,
        "family": family,
        "limit": limit,
        "offset": offset,
        "returned": returned,
        "total": total,
        "truncated": offset + returned < total,
        "kind": "norm",
        "provenance": profiles.provenance.clone(),
    }))
}

/// The metadata rows of a set of cubes, folded per cube (P16: the
/// endpoint may answer a key twice, so nothing is trusted to be
/// unique).
struct Profiles {
    names: std::collections::BTreeMap<String, Vec<(String, String)>>,
    descriptions: std::collections::BTreeMap<String, Vec<(String, String)>>,
    status: std::collections::BTreeMap<String, String>,
    status_labels: std::collections::BTreeMap<String, Vec<(String, String)>>,
    provenance: Value,
}

impl Profiles {
    /// One cube as `list_cubes` and `list_versions` serve it (P2, P3,
    /// P5).
    fn row(&self, cube: &str, observations: Option<&u64>, lang: &str) -> Value {
        let name = self
            .names
            .get(cube)
            .and_then(|found| choose_label(found, Some(lang)));
        let description = self
            .descriptions
            .get(cube)
            .and_then(|found| choose_label(found, Some(lang)));
        let status = self.status.get(cube);
        let status_label = status
            .and_then(|s| self.status_labels.get(s))
            .and_then(|found| choose_label(found, Some(lang)));
        let observations = observations.copied().unwrap_or(0);
        json!({
            "cube": cube,
            "name": name.as_ref().map(|(l, _)| l.clone()),
            "name_lang": name.as_ref().map(|(_, l)| l.clone()),
            "description": description.as_ref().map(|(l, _)| l.clone()),
            "description_lang": description.as_ref().map(|(_, l)| l.clone()),
            "family": scope::family_of(cube),
            "version": scope::version_of(cube),
            "versioned": scope::version_of(cube).is_some(),
            "status": status,
            "status_label": status_label.as_ref().map(|(l, _)| l.clone()),
            "status_label_lang": status_label.as_ref().map(|(_, l)| l.clone()),
            "status_unset": status.is_none(),
            "observations": observations,
            "placeholder": observations == 0,
        })
    }
}

/// One bound query for the metadata of a set of cubes.
fn cube_profiles(
    ctx: &Ctx,
    cubes: &[String],
    scope_name: &str,
    lang: &str,
) -> std::result::Result<Profiles, Value> {
    let query = format!(
        "{PREFIXES}SELECT ?cube ?name ?nameLang ?description ?descriptionLang ?status \
         ?statusLabel ?statusLabelLang WHERE {{\n\
         {values}\n\
         OPTIONAL {{ ?cube schema:name ?name . BIND(LANG(?name) AS ?nameLang) }}\n\
         OPTIONAL {{ ?cube schema:description ?description . \
         BIND(LANG(?description) AS ?descriptionLang) }}\n\
         OPTIONAL {{ ?cube schema:creativeWorkStatus ?status .\n\
         OPTIONAL {{ VALUES ?labelP {{ schema:name skos:prefLabel rdfs:label }}\n\
         ?status ?labelP ?statusLabel . BIND(LANG(?statusLabel) AS ?statusLabelLang) }} }}\n\
         }}",
        values = values_block("cube", cubes)
    );
    let answer = ask(ctx, &format!("list_cubes:{scope_name}:{lang}"), &query)?;
    let bindings = Backend::bindings(&answer.value).map_err(|e| upstream(format!("{e:#}")))?;
    let mut profiles = Profiles {
        names: Default::default(),
        descriptions: Default::default(),
        status: Default::default(),
        status_labels: Default::default(),
        provenance: provenance(ctx, &answer),
    };
    for binding in bindings {
        let Some(cube) = text(binding, "cube") else {
            continue;
        };
        if let Some(name) = text(binding, "name") {
            let lang = text(binding, "nameLang").unwrap_or_default();
            let entry = profiles.names.entry(cube.clone()).or_default();
            if !entry.iter().any(|(l, n)| *l == lang && *n == name) {
                entry.push((lang, name));
            }
        }
        if let Some(description) = text(binding, "description") {
            let lang = text(binding, "descriptionLang").unwrap_or_default();
            let entry = profiles.descriptions.entry(cube.clone()).or_default();
            if !entry.iter().any(|(l, d)| *l == lang && *d == description) {
                entry.push((lang, description));
            }
        }
        if let Some(status) = text(binding, "status") {
            profiles.status.insert(cube.clone(), status.clone());
            if let Some(label) = text(binding, "statusLabel") {
                let lang = text(binding, "statusLabelLang").unwrap_or_default();
                let entry = profiles.status_labels.entry(status).or_default();
                if !entry.iter().any(|(l, s)| *l == lang && *s == label) {
                    entry.push((lang, label));
                }
            }
        }
    }
    Ok(profiles)
}

/// One bound query for the observation counts of a set of cubes.
fn observation_counts(
    ctx: &Ctx,
    cubes: &[String],
    scope_name: &str,
) -> std::result::Result<std::collections::BTreeMap<String, u64>, Value> {
    let query = format!(
        "{PREFIXES}SELECT ?cube (COUNT(?obs) AS ?observations) WHERE {{\n\
         {values}\n\
         ?cube cube:observationSet ?set . ?set cube:observation ?obs .\n\
         }} GROUP BY ?cube",
        values = values_block("cube", cubes)
    );
    let answer = ask(ctx, &format!("observation_counts:{scope_name}"), &query)?;
    let bindings = Backend::bindings(&answer.value).map_err(|e| upstream(format!("{e:#}")))?;
    let mut counts = std::collections::BTreeMap::new();
    for binding in bindings {
        let (Some(cube), Some(count)) = (text(binding, "cube"), text(binding, "observations"))
        else {
            continue;
        };
        // P16: an aggregate may answer a key twice — fold, never trust.
        let parsed: u64 = count.parse().unwrap_or(0);
        let entry = counts.entry(cube).or_insert(0);
        *entry = (*entry).max(parsed);
    }
    Ok(counts)
}

// --- 3.2 find_cube --------------------------------------------------

/// How many words a name search may carry — the same reason the
/// fedlex title search has one (BY′ point 10): every word is one
/// `CONTAINS`, and the endpoint is shared.
pub const MAX_QUERY_WORDS: usize = 12;

/// Every WORD of the query, as a case-insensitive substring, in the
/// SAME cube name (BY′ point 10).
///
/// `find_cube` filtered the whole query as ONE contiguous substring —
/// the construction `fedlex.search_law` shed at BY point 0, for the
/// reason that applies here too: «Abstimmungen Kantone» is no
/// substring of «Eidgenössische Volksabstimmungen seit 1848», and this
/// is the ENTRANCE of two-stage discovery, where a model asks in its
/// own words. The window is the fixed 44-cube scope either way, so
/// word-wise costs nothing upstream. For ONE word the fragment is what
/// it always was.
pub fn name_filter(words: &[String]) -> String {
    if words.is_empty() {
        return "true".to_string();
    }
    words
        .iter()
        .map(|w| format!("CONTAINS(LCASE(STR(?name)), \"{}\")", literal_safe(w)))
        .collect::<Vec<_>>()
        .join(" && ")
}

/// The name query of `find_cube`: the 44 served cubes bound by
/// `VALUES`, and every WORD of the query in the same name.
///
/// A FUNCTION so the shape can be proven offline (BY′): the fixture
/// backends answer by semantic key — `find_cube:<query>:<lang>:<limit>`
/// — and never read the SPARQL, so a recorded answer cannot tell the
/// word-wise filter from the contiguous one it replaced.
pub fn find_cube_query(cubes: &[String], words: &[String]) -> String {
    format!(
        "{PREFIXES}SELECT ?cube ?name ?nameLang WHERE {{\n\
         {values}\n\
         ?cube schema:name ?name .\n\
         FILTER({filter})\n\
         BIND(LANG(?name) AS ?nameLang)\n\
         }}",
        values = values_block("cube", cubes),
        filter = name_filter(words)
    )
}

/// `lindas.find_cube` — a cube by a word in its label (P2, P7).
pub fn find_cube(ctx: &Ctx, query: &str, lang: Option<&str>, limit: Option<u32>) -> Result<Value> {
    let lang = match language(lang) {
        Ok(l) => l,
        Err(refusal) => return Ok(refusal),
    };
    let needle = query.trim();
    if needle.chars().count() < 2 || needle.chars().count() > 100 {
        return Ok(refusal_with_shape(
            "the query must be 2 to 100 characters",
            "a word of a cube's name — 2 to 100 characters, at most twelve words",
            "lindas.list_cubes serves the 44 names to choose a word from; every word given must \
             occur in the SAME name. This refusal costs no request",
        ));
    }
    // BY′ point 10: every WORD in the same name, not the phrase.
    let words: Vec<String> = needle
        .to_lowercase()
        .split_whitespace()
        .map(str::to_string)
        .collect();
    if words.len() > MAX_QUERY_WORDS {
        return Ok(too_many(
            &format!("words: {} were given", words.len()),
            MAX_QUERY_WORDS,
            "words",
            "every word must occur in the SAME cube name and becomes one CONTAINS in the query, \
             so a pasted sentence is refused; name the distinctive words",
            "lindas.list_cubes",
        ));
    }
    let limit = capped(limit, MAX_HITS, 10);
    let cubes = scope::all();
    let sparql = find_cube_query(&cubes, &words);
    let answer = match ask(ctx, &format!("find_cube:{needle}:{lang}:{limit}"), &sparql) {
        Ok(a) => a,
        Err(refusal) => return Ok(refusal),
    };
    let bindings = match Backend::bindings(&answer.value) {
        Ok(b) => b,
        Err(e) => return Ok(upstream(format!("{e:#}"))),
    };
    let mut hits: std::collections::BTreeMap<String, Vec<(String, String)>> = Default::default();
    for binding in bindings {
        let (Some(cube), Some(name)) = (text(binding, "cube"), text(binding, "name")) else {
            continue;
        };
        let lang = text(binding, "nameLang").unwrap_or_default();
        hits.entry(cube).or_default().push((lang, name));
    }
    // A search that found nothing has nothing to count (BY″, the
    // reason corrected at BY‴): the count query is ONE request over
    // the fixed 44-cube scope under a constant key
    // (`observation_counts:all`) — not one per hit — and its answer is
    // attached to the hits. With no hits there is nothing to attach it
    // to, so the request is spent on nothing. This tool is the
    // entrance, where a model's first guesses miss most often.
    let counts = if hits.is_empty() {
        Default::default()
    } else {
        match observation_counts(ctx, &cubes, "all") {
            Ok(c) => c,
            Err(refusal) => return Ok(refusal),
        }
    };
    let total = hits.len();
    let rows: Vec<Value> = hits
        .iter()
        .take(limit)
        .map(|(cube, names)| {
            let label = choose_label(names, Some(lang));
            let observations = counts.get(cube).copied().unwrap_or(0);
            json!({
                "cube": cube,
                "name": label.as_ref().map(|(l, _)| l.clone()),
                "name_lang": label.as_ref().map(|(_, l)| l.clone()),
                "family": scope::family_of(cube),
                "observations": observations,
                "placeholder": observations == 0,
            })
        })
        .collect();
    let returned = rows.len();
    Ok(json!({
        "query": needle,
        "hits": rows,
        "limit": limit,
        "returned": returned,
        "total": total,
        "truncated": returned < total,
        "kind": "hint",
        "provenance": provenance(ctx, &answer),
    }))
}

// --- 3.3 describe_cube ----------------------------------------------

/// One dimension as the shape declares it.
#[derive(Clone)]
pub struct ShapeDimension {
    pub dimension: String,
    pub names: Vec<(String, String)>,
    pub node_kind: Option<String>,
    pub datatype: Option<String>,
    pub scale: Option<String>,
    pub dimension_class: Option<String>,
    pub min_count: Option<String>,
    pub max_count: Option<String>,
    pub unit: Option<String>,
    /// Does the shape close this dimension with `sh:in`? The MEMBERS
    /// are not pulled here: one cube's `id` enumerates 2'880 values,
    /// and a shape answer that carried them would be a list nobody
    /// asked for. `dimension_values` fetches them, capped.
    pub enumerated: bool,
}

impl ShapeDimension {
    /// The kind a shape answer serves: the declared class, or the
    /// honest `unknown` (P10, C2.4).
    fn kind(&self) -> &str {
        match self.dimension_class.as_deref() {
            Some(c) if c.ends_with("KeyDimension") => "key",
            Some(c) if c.ends_with("MeasureDimension") => "measure",
            Some(c) if c.ends_with("AttributeDimension") => "attribute",
            _ => "unknown",
        }
    }

    fn json(&self, lang: &str) -> Value {
        let name = choose_label(&self.names, Some(lang));
        json!({
            "dimension": self.dimension,
            "name": name.as_ref().map(|(l, _)| l.clone()),
            "name_lang": name.as_ref().map(|(_, l)| l.clone()),
            "node_kind": self.node_kind,
            "datatype": self.datatype,
            "scale": self.scale,
            "dimension_kind": self.kind(),
            "optional": self.min_count.as_deref() == Some("0"),
            "max_count": self.max_count,
            "unit": self.unit,
            "enumerated": self.enumerated,
        })
    }
}

/// The declared shape of one cube, read once and shared by the tools
/// that need it (P12's step (a) is a shape lookup, not a request per
/// call).
fn cube_shape(
    ctx: &Ctx,
    cube: &str,
    lang: &str,
) -> std::result::Result<(Vec<ShapeDimension>, Answer), Value> {
    let query = format!(
        "{PREFIXES}SELECT ?dim ?name ?nameLang ?nodeKind ?datatype ?scale ?class ?minCount \
         ?maxCount ?unit ?inList WHERE {{\n\
         <{cube}> cube:observationConstraint ?shape . ?shape sh:property ?prop . \
         ?prop sh:path ?dim .\n\
         OPTIONAL {{ ?prop schema:name ?name . BIND(LANG(?name) AS ?nameLang) }}\n\
         OPTIONAL {{ ?prop sh:nodeKind ?nodeKind }}\n\
         OPTIONAL {{ ?prop sh:datatype ?datatype }}\n\
         OPTIONAL {{ ?prop qudt:scaleType ?scale }}\n\
         OPTIONAL {{ ?prop a ?class }}\n\
         OPTIONAL {{ ?prop sh:minCount ?minCount }}\n\
         OPTIONAL {{ ?prop sh:maxCount ?maxCount }}\n\
         OPTIONAL {{ ?prop qudt:hasUnit ?unit }}\n\
         OPTIONAL {{ ?prop sh:in ?inList }}\n\
         }} ORDER BY ?dim"
    );
    let answer = ask(ctx, &format!("describe_cube:shape:{cube}:{lang}"), &query)?;
    let bindings = Backend::bindings(&answer.value).map_err(|e| upstream(format!("{e:#}")))?;
    let mut by_dim: Vec<ShapeDimension> = Vec::new();
    for binding in bindings {
        let Some(dim) = text(binding, "dim") else {
            continue;
        };
        let existing = by_dim.iter().position(|d| d.dimension == dim);
        let index = match existing {
            Some(i) => i,
            None => {
                by_dim.push(ShapeDimension {
                    dimension: dim.clone(),
                    names: Vec::new(),
                    node_kind: None,
                    datatype: None,
                    scale: None,
                    dimension_class: None,
                    min_count: None,
                    max_count: None,
                    unit: None,
                    enumerated: false,
                });
                by_dim.len() - 1
            }
        };
        let entry = &mut by_dim[index];
        if let Some(name) = text(binding, "name") {
            let lang = text(binding, "nameLang").unwrap_or_default();
            if !entry.names.iter().any(|(l, n)| *l == lang && *n == name) {
                entry.names.push((lang, name));
            }
        }
        entry.node_kind = entry.node_kind.take().or_else(|| text(binding, "nodeKind"));
        entry.datatype = entry.datatype.take().or_else(|| text(binding, "datatype"));
        entry.scale = entry.scale.take().or_else(|| text(binding, "scale"));
        entry.min_count = entry.min_count.take().or_else(|| text(binding, "minCount"));
        entry.max_count = entry.max_count.take().or_else(|| text(binding, "maxCount"));
        entry.unit = entry.unit.take().or_else(|| text(binding, "unit"));
        if let Some(class) = text(binding, "class") {
            if class.starts_with("https://cube.link/") {
                entry.dimension_class = Some(class);
            }
        }
        if text(binding, "inList").is_some() {
            entry.enumerated = true;
        }
    }
    Ok((by_dim, answer))
}

/// `lindas.describe_cube` — the declared shape and the profile (P6,
/// P8, P9, P10, P11, P18, P36).
pub fn describe_cube(
    ctx: &Ctx,
    cube: &str,
    lang: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Value> {
    let lang = match language(lang) {
        Ok(l) => l,
        Err(refusal) => return Ok(refusal),
    };
    let cube = match served_cube(cube) {
        Ok(c) => c,
        Err(refusal) => return Ok(refusal),
    };
    let query = format!(
        "{PREFIXES}SELECT ?name ?nameLang ?description ?descriptionLang ?created ?published \
         ?modified ?status ?publisher ?theme ?viewer (COUNT(?obs) AS ?observations) WHERE {{\n\
         OPTIONAL {{ <{cube}> schema:name ?name . BIND(LANG(?name) AS ?nameLang) }}\n\
         OPTIONAL {{ <{cube}> schema:description ?description . \
         BIND(LANG(?description) AS ?descriptionLang) }}\n\
         OPTIONAL {{ <{cube}> schema:dateCreated ?created }}\n\
         OPTIONAL {{ <{cube}> schema:datePublished ?published }}\n\
         OPTIONAL {{ <{cube}> schema:dateModified ?modified }}\n\
         OPTIONAL {{ <{cube}> schema:creativeWorkStatus ?status }}\n\
         OPTIONAL {{ <{cube}> schema:publisher ?publisher }}\n\
         OPTIONAL {{ <{cube}> dcat:theme ?theme }}\n\
         OPTIONAL {{ <{cube}> schema:workExample ?viewer }}\n\
         OPTIONAL {{ <{cube}> cube:observationSet ?set . ?set cube:observation ?obs }}\n\
         }} GROUP BY ?name ?nameLang ?description ?descriptionLang ?created ?published ?modified \
         ?status ?publisher ?theme ?viewer"
    );
    let profile = match ask(ctx, &format!("describe_cube:{cube}:{lang}"), &query) {
        Ok(a) => a,
        Err(refusal) => return Ok(refusal),
    };
    let bindings = match Backend::bindings(&profile.value) {
        Ok(b) => b,
        Err(e) => return Ok(upstream(format!("{e:#}"))),
    };
    let mut names: Vec<(String, String)> = Vec::new();
    let mut descriptions: Vec<(String, String)> = Vec::new();
    let mut observations = 0u64;
    let mut first: Option<&Value> = None;
    for binding in bindings {
        if first.is_none() {
            first = Some(binding);
        }
        if let Some(name) = text(binding, "name") {
            let lang = text(binding, "nameLang").unwrap_or_default();
            if !names.iter().any(|(l, n)| *l == lang && *n == name) {
                names.push((lang, name));
            }
        }
        if let Some(description) = text(binding, "description") {
            let lang = text(binding, "descriptionLang").unwrap_or_default();
            if !descriptions
                .iter()
                .any(|(l, d)| *l == lang && *d == description)
            {
                descriptions.push((lang, description));
            }
        }
        if let Some(count) = text(binding, "observations").and_then(|c| c.parse::<u64>().ok()) {
            observations = observations.max(count);
        }
    }
    let name = choose_label(&names, Some(lang));
    let description = choose_label(&descriptions, Some(lang));
    let field = |var: &str| first.and_then(|b| text(b, var));
    let modified = field("modified");
    let (shape, _) = match cube_shape(ctx, &cube, lang) {
        Ok(s) => s,
        Err(refusal) => return Ok(refusal),
    };
    let sample = match carried_predicates(ctx, &cube) {
        Ok(s) => s,
        Err(refusal) => return Ok(refusal),
    };
    let dimensions_total = shape.len();
    let limit = capped(limit, MAX_DIMENSIONS, 50);
    let offset = offset.unwrap_or(0) as usize;
    let page: Vec<Value> = shape
        .iter()
        .skip(offset)
        .take(limit)
        .map(|d| d.json(lang))
        .collect();
    let returned = page.len();
    let declared: Vec<&str> = shape.iter().map(|d| d.dimension.as_str()).collect();
    let undeclared: Vec<&String> = sample
        .iter()
        .filter(|p| !declared.contains(&p.as_str()))
        .collect();
    Ok(json!({
        "cube": cube,
        "name": name.as_ref().map(|(l, _)| l.clone()),
        "name_lang": name.as_ref().map(|(_, l)| l.clone()),
        "description": description.as_ref().map(|(l, _)| l.clone()),
        "description_lang": description.as_ref().map(|(_, l)| l.clone()),
        "family": scope::family_of(&cube),
        "version": scope::version_of(&cube),
        "versioned": scope::version_of(&cube).is_some(),
        "publisher": field("publisher"),
        "theme": field("theme"),
        "viewer": field("viewer"),
        "dates": {
            "created": field("created"),
            "published": field("published"),
            "modified": modified.clone(),
            "granularity": modified.as_deref().map(|m| if m.contains('T') { "dateTime" } else { "date" }),
        },
        "status": field("status"),
        "status_unset": field("status").is_none(),
        "observations": observations,
        "placeholder": observations == 0,
        "dimensions": page,
        "dimensions_total": dimensions_total,
        "limit": limit,
        "offset": offset,
        "returned": returned,
        "truncated": offset + returned < dimensions_total,
        "declared_only": true,
        "carried_predicates_sample": sample,
        "undeclared_in_sample": undeclared,
        "sampled": true,
        "note": "these are the dimensions the SHACL shape DECLARES; the observations may carry \
                 more (measured: 14 declared against 51 carried, in the largest cube of the \
                 scope). \
                 carried_predicates_sample is one observation's predicates — a hint, not a \
                 census; a dimension is admitted by the bound ASK of lindas.observations",
        "kind": "norm",
        "provenance": provenance(ctx, &profile),
    }))
}

/// The predicates ONE observation of a cube carries — the sample
/// `describe_cube` serves beside the declared shape (P9, C14.1).
fn carried_predicates(ctx: &Ctx, cube: &str) -> std::result::Result<Vec<String>, Value> {
    let query = format!(
        "{PREFIXES}SELECT DISTINCT ?p WHERE {{\n\
         {{ SELECT ?obs WHERE {{ <{cube}> cube:observationSet ?set . ?set cube:observation ?obs }} \
         LIMIT 1 }}\n\
         ?obs ?p ?v .\n\
         }} ORDER BY ?p"
    );
    let answer = ask(ctx, &format!("describe_cube:sample:{cube}"), &query)?;
    let bindings = Backend::bindings(&answer.value).map_err(|e| upstream(format!("{e:#}")))?;
    Ok(bindings.iter().filter_map(|b| text(b, "p")).collect())
}

// --- P12: the three-step dimension rule ------------------------------

/// What the three steps of P12 decided about one dimension.
///
/// `Copy` since BY′: the decision is memoised per dimension, so a
/// caller that filters AND projects the same one asks the store once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    /// (a) the shape declares it — accepted, no request was made.
    Declared,
    /// (b) undeclared, and the bound ASK found it in the observations.
    Undeclared,
    /// (c) the bound ASK found nothing — `not-found`, because a request
    /// WAS made to find that out.
    Absent,
}

/// P12 in one place: declared → accepted; undeclared → ONE bound ASK;
/// the ASK's `false` → absent. Applied by every tool that takes a
/// dimension.
pub fn dimension_gate(
    ctx: &Ctx,
    cube: &str,
    dimension: &str,
    declared: &[ShapeDimension],
) -> std::result::Result<Gate, Value> {
    if declared.iter().any(|d| d.dimension == dimension) {
        return Ok(Gate::Declared);
    }
    let query = format!(
        "{PREFIXES}ASK {{ <{cube}> cube:observationSet ?set . ?set cube:observation ?o . \
         ?o <{dimension}> ?v }}"
    );
    let answer = ask(ctx, &format!("ask_dimension:{cube}:{dimension}"), &query)?;
    let carried = Backend::boolean(&answer.value).map_err(|e| upstream(format!("{e:#}")))?;
    Ok(if carried {
        Gate::Undeclared
    } else {
        Gate::Absent
    })
}

// --- 3.4 dimension_values -------------------------------------------

/// `lindas.dimension_values` — the values one dimension takes (P11,
/// P12, P13, P21).
pub fn dimension_values(
    ctx: &Ctx,
    cube: &str,
    dimension: &str,
    lang: Option<&str>,
    limit: Option<u32>,
) -> Result<Value> {
    let lang = match language(lang) {
        Ok(l) => l,
        Err(refusal) => return Ok(refusal),
    };
    let cube = match served_cube(cube) {
        Ok(c) => c,
        Err(refusal) => return Ok(refusal),
    };
    let dimension = match iri_safe(dimension) {
        Ok(d) => normalise_value(d),
        Err(e) => return Ok(invalid_dimension_name(&format!("dimension: {e:#}"))),
    };
    let limit = capped(limit, MAX_VALUES, 50);
    let (shape, _) = match cube_shape(ctx, &cube, lang) {
        Ok(s) => s,
        Err(refusal) => return Ok(refusal),
    };
    let gate = match dimension_gate(ctx, &cube, &dimension, &shape) {
        Ok(g) => g,
        Err(refusal) => return Ok(refusal),
    };
    if gate == Gate::Absent {
        return Ok(not_found_dimension(&cube, &dimension));
    }
    let undeclared: Vec<&str> = if gate == Gate::Undeclared {
        vec![dimension.as_str()]
    } else {
        Vec::new()
    };
    // (P11) A dimension the shape CLOSES with sh:in is answered from
    // that list — one bound query over the list itself, never a scan
    // of the observations. The members carry their labels, because a
    // state answer that names only an IRI is half an answer (C9.2).
    let enumerated = shape
        .iter()
        .any(|d| d.dimension == dimension && d.enumerated);
    if enumerated {
        let query = format!(
            "{PREFIXES}SELECT ?value ?label ?labelLang WHERE {{\n\
             <{cube}> cube:observationConstraint ?shape . ?shape sh:property ?prop .\n\
             ?prop sh:path <{dimension}> ; sh:in/rdf:rest*/rdf:first ?value .\n\
             OPTIONAL {{ VALUES ?labelP {{ schema:name skos:prefLabel rdfs:label }}\n\
             ?value ?labelP ?label . BIND(LANG(?label) AS ?labelLang) }}\n\
             }} ORDER BY ?value"
        );
        let answer = match ask(
            ctx,
            &format!("dimension_values:enum:{cube}:{dimension}:{lang}"),
            &query,
        ) {
            Ok(a) => a,
            Err(refusal) => return Ok(refusal),
        };
        let bindings = match Backend::bindings(&answer.value) {
            Ok(b) => b,
            Err(e) => return Ok(upstream(format!("{e:#}"))),
        };
        let mut members: Vec<(String, Vec<(String, String)>)> = Vec::new();
        for binding in bindings {
            let Some(value) = text(binding, "value") else {
                continue;
            };
            let index = match members.iter().position(|(v, _)| *v == value) {
                Some(i) => i,
                None => {
                    members.push((value.clone(), Vec::new()));
                    members.len() - 1
                }
            };
            if let Some(label) = text(binding, "label") {
                let lang = text(binding, "labelLang").unwrap_or_default();
                let labels = &mut members[index].1;
                if !labels.iter().any(|(l, t)| *l == lang && *t == label) {
                    labels.push((lang, label));
                }
            }
        }
        let total = members.len();
        let rows: Vec<Value> = members
            .iter()
            .take(limit)
            .map(|(value, labels)| {
                let label = choose_label(labels, Some(lang));
                json!({
                    "value": value,
                    "stated": true,
                    "form": "iri",
                    "label": label.as_ref().map(|(l, _)| l.clone()),
                    "label_lang": label.as_ref().map(|(_, l)| l.clone()),
                    "observations": Value::Null,
                })
            })
            .collect();
        let returned = rows.len();
        return Ok(json!({
            "cube": cube,
            "dimension": dimension,
            "values": rows,
            "limit": limit,
            "returned": returned,
            "total": total,
            "truncated": returned < total,
            "source": "enumeration",
            "note": "the shape closes this dimension with sh:in: the values come from that list, \
                     not from a scan of the observations, so «observations» is null per value",
            "undeclared_dimensions": undeclared,
            "kind": "hint",
            "provenance": provenance(ctx, &answer),
        }));
    }
    let query = format!(
        "{PREFIXES}SELECT ?value ?label ?labelLang (COUNT(?obs) AS ?observations) WHERE {{\n\
         <{cube}> cube:observationSet ?set . ?set cube:observation ?obs .\n\
         ?obs <{dimension}> ?value .\n\
         OPTIONAL {{ VALUES ?labelP {{ schema:name skos:prefLabel rdfs:label }}\n\
         ?value ?labelP ?label . BIND(LANG(?label) AS ?labelLang) }}\n\
         }} GROUP BY ?value ?label ?labelLang ORDER BY DESC(?observations) ?value"
    );
    let answer = match ask(
        ctx,
        &format!("dimension_values:{cube}:{dimension}:{lang}:{limit}"),
        &query,
    ) {
        Ok(a) => a,
        Err(refusal) => return Ok(refusal),
    };
    let bindings = match Backend::bindings(&answer.value) {
        Ok(b) => b,
        Err(e) => return Ok(upstream(format!("{e:#}"))),
    };
    struct ValueRow {
        cell: Cell,
        labels: Vec<(String, String)>,
        observations: u64,
    }
    let mut rows: Vec<(String, ValueRow)> = Vec::new();
    for binding in bindings {
        let Some(c) = cell(binding, "value") else {
            continue;
        };
        let key = c
            .value
            .clone()
            .unwrap_or_else(|| format!("not-stated:{}", c.form));
        let observations = text(binding, "observations")
            .and_then(|o| o.parse::<u64>().ok())
            .unwrap_or(0);
        let index = match rows.iter().position(|(k, _)| *k == key) {
            Some(i) => i,
            None => {
                rows.push((
                    key.clone(),
                    ValueRow {
                        cell: c.clone(),
                        labels: Vec::new(),
                        observations: 0,
                    },
                ));
                rows.len() - 1
            }
        };
        let row = &mut rows[index].1;
        // P16: an aggregate can answer the same key twice — fold.
        row.observations = row.observations.max(observations);
        if let Some(label) = text(binding, "label") {
            let lang = text(binding, "labelLang").unwrap_or_default();
            if !row.labels.iter().any(|(l, t)| *l == lang && *t == label) {
                row.labels.push((lang, label));
            }
        }
    }
    let total = rows.len();
    let page: Vec<Value> = rows
        .iter()
        .take(limit)
        .map(|(_, row)| {
            let label = choose_label(&row.labels, Some(lang));
            json!({
                "value": row.cell.value,
                "stated": row.cell.stated,
                "form": row.cell.form,
                "label": label.as_ref().map(|(l, _)| l.clone()),
                "label_lang": label.as_ref().map(|(_, l)| l.clone()),
                "observations": row.observations,
            })
        })
        .collect();
    let returned = page.len();
    Ok(json!({
        "cube": cube,
        "dimension": dimension,
        "values": page,
        "limit": limit,
        "returned": returned,
        "total": total,
        "truncated": returned < total,
        "source": "observations",
        "undeclared_dimensions": undeclared,
        "kind": "hint",
        "provenance": provenance(ctx, &answer),
    }))
}

// --- 3.5 observations -----------------------------------------------

/// `lindas.observations` — the rows, filtered, capped, honest (P12,
/// P13, P14, P15, P16, P17, P19, P20, P21, P25, P35).
/// One page of `observations`, with the cells of each row.
///
/// **The projection may not drop a ROW** (BY′ point 1). The cell
/// pattern `?obs ?p ?v` is REQUIRED without a projection — every
/// observation carries at least one predicate, so nothing is lost —
/// but with a `VALUES ?p { … }` in front of it, an observation that
/// carries none of the named predicates binds nothing and falls out of
/// the page: `returned` would then count survivors while `total` came
/// from the unprojected count, and no field would say so. Under a
/// projection the cells therefore hang OPTIONALLY on the row the inner
/// `SELECT` already bound, and a row that carries none of them comes
/// back with `cells: []` — which is what the register gives, and means
/// «does not carry it», not `0` and not `stated: false`.
///
/// A FUNCTION so the shape can be proven offline: the fixture backends
/// answer by semantic key and never read the query, so a test over
/// recorded answers cannot tell a projected query from an unprojected
/// one (`the_projection_binds_its_predicates_and_never_drops_a_row`).
/// The count over a cube's observations, with or without filter
/// clauses. One function, so the filtered count and the cube's own
/// count — which decides whether a filter miss is an empty cube —
/// are byte-identically the same query shape and share the fixture
/// key grammar `observations:count:<cube>:<filters>`.
fn count_query_of(cube: &str, clauses: &str) -> String {
    format!(
        "{PREFIXES}SELECT (COUNT(DISTINCT ?obs) AS ?total) WHERE {{\n\
         <{cube}> cube:observationSet ?set . ?set cube:observation ?obs .\n\
         {clauses}}}"
    )
}

pub fn observations_page_query(
    cube: &str,
    clauses: &str,
    projected: &[String],
    limit: usize,
    offset: usize,
) -> String {
    // The untagged arm (`LANG(?label) = ""`) is the audit's third-
    // ranked defect closed (01.09.2026): candidate names are UNTAGGED
    // literals in this holding, so the five-language filter dropped
    // every one of them and an election row arrived as a bare IRI —
    // which the model then read a «name» out of, a slug that actually
    // encodes canton and list codes. Same arm `resolve_label` has
    // always had; `choose_label` already serves such a label as «und».
    const LABELS: &str = "OPTIONAL { VALUES ?labelP { schema:name skos:prefLabel rdfs:label }\n\
                          ?v ?labelP ?label . FILTER(LANG(?label) = \"\" || LANG(?label) IN (\"de\", \"fr\", \"it\", \"en\", \"rm\"))\n\
                          BIND(LANG(?label) AS ?labelLang) }";
    let cells = if projected.is_empty() {
        format!("?obs ?p ?v .\n{LABELS}\n")
    } else {
        let list: Vec<String> = projected.iter().map(|d| format!("<{d}>")).collect();
        format!(
            "OPTIONAL {{ VALUES ?p {{ {} }}\n?obs ?p ?v .\n{LABELS} }}\n",
            list.join(" ")
        )
    };
    format!(
        "{PREFIXES}SELECT ?obs ?p ?v ?label ?labelLang WHERE {{\n\
         {{ SELECT DISTINCT ?obs WHERE {{\n\
         <{cube}> cube:observationSet ?set . ?set cube:observation ?obs .\n\
         {clauses}}} ORDER BY ?obs LIMIT {limit} OFFSET {offset} }}\n\
         {cells}}} ORDER BY ?obs ?p"
    )
}

pub fn observations(
    ctx: &Ctx,
    cube: &str,
    filters: &[(String, String)],
    projection: &[String],
    lang: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Value> {
    let lang = match language(lang) {
        Ok(l) => l,
        Err(refusal) => return Ok(refusal),
    };
    let cube = match served_cube(cube) {
        Ok(c) => c,
        Err(refusal) => return Ok(refusal),
    };
    let limit = capped(limit, MAX_OBSERVATIONS, 50);
    let offset = offset.unwrap_or(0) as usize;

    // BEFORE ANY REQUEST (BY point 0): everything about a filter or a
    // projection that is decidable without asking the store is decided
    // here. A short name never reaches the shape query, and never
    // costs the `ASK` that an unknown-but-well-formed IRI does.
    if filters.len() > MAX_FILTERS {
        return Ok(too_many(
            &format!("filters: {} were named", filters.len()),
            MAX_FILTERS,
            "filters",
            "every dimension the shape does not declare costs one bound ASK before the page is \
             read, and every filter adds a clause to the query",
            "lindas.describe_cube",
        ));
    }
    let mut filters_checked: Vec<(String, String)> = Vec::with_capacity(filters.len());
    for (dimension, value) in filters {
        let dimension = match iri_safe(dimension) {
            Ok(d) => normalise_value(d),
            Err(e) => return Ok(invalid_dimension(&format!("filter: {e:#}"))),
        };
        let value = normalise_value(value);
        if (value.starts_with("https://") || value.starts_with("http://"))
            && iri_safe(&value).is_err()
        {
            return Ok(invalid_dimension(&format!(
                "filter value: «{value}» begins like an IRI and is none"
            )));
        }
        filters_checked.push((dimension, value));
    }
    let mut projected: Vec<String> = Vec::with_capacity(projection.len());
    for dimension in projection {
        match iri_safe(dimension) {
            Ok(d) => projected.push(normalise_value(d)),
            Err(e) => return Ok(invalid_projection(&format!("dimensions: {e:#}"))),
        }
    }
    projected.sort();
    projected.dedup();
    if projected.len() > MAX_PROJECTED {
        return Ok(too_many(
            &format!("dimensions: {} were named", projected.len()),
            MAX_PROJECTED,
            "dimensions",
            "every dimension the shape does not declare costs one bound ASK before the page is \
             read, so the list is capped; name the cells the question needs",
            "lindas.describe_cube",
        ));
    }

    let (shape, _) = match cube_shape(ctx, &cube, lang) {
        Ok(s) => s,
        Err(refusal) => return Ok(refusal),
    };
    // P12, for every filter AND every projected dimension: declared →
    // free; undeclared → one ASK; absent → not-found naming the cube
    // and the dimension. A projection is a claim about the record
    // exactly as a filter is, so it is admitted the same way.
    let mut undeclared: Vec<String> = Vec::new();
    let mut clauses = String::new();
    let mut clause_index = 0usize;
    // ONE `ASK` per dimension, not one per mention (BY′ point 3): a
    // caller that filters AND projects the same undeclared dimension
    // asked the store the same question twice, where P12(b) names one.
    let mut gated: Vec<(String, Gate)> = Vec::new();
    let gate_of = |ctx: &Ctx, dimension: &str, gated: &mut Vec<(String, Gate)>| {
        if let Some((_, gate)) = gated.iter().find(|(d, _)| d == dimension) {
            return Ok(*gate);
        }
        let gate = dimension_gate(ctx, &cube, dimension, &shape)?;
        gated.push((dimension.to_string(), gate));
        Ok(gate)
    };
    for dimension in &projected {
        match gate_of(ctx, dimension, &mut gated) {
            Ok(Gate::Declared) => {}
            Ok(Gate::Undeclared) => undeclared.push(dimension.clone()),
            Ok(Gate::Absent) => return Ok(not_found_dimension(&cube, dimension)),
            Err(refusal) => return Ok(refusal),
        }
    }
    for (dimension, value) in &filters_checked {
        let dimension = dimension.clone();
        match gate_of(ctx, &dimension, &mut gated) {
            Ok(Gate::Declared) => {}
            Ok(Gate::Undeclared) => {
                if !undeclared.contains(&dimension) {
                    undeclared.push(dimension.clone())
                }
            }
            Ok(Gate::Absent) => return Ok(not_found_dimension(&cube, &dimension)),
            Err(refusal) => return Ok(refusal),
        }
        if value.starts_with("https://") || value.starts_with("http://") {
            // An IRI value matches exactly, and the object stays bound.
            clauses.push_str(&format!("?obs <{dimension}> <{value}> .\n"));
        } else {
            // A LITERAL is compared on its lexical form: this holding
            // writes the same dimension as xsd:date, xsd:string,
            // xsd:dateTime or xsd:decimal (C3.5), and a caller who
            // knows «1971-02-07» should not have to know which. The
            // predicate stays bound, which is what C12.3 asks for.
            let var = format!("?filterValue{}", clause_index);
            clauses.push_str(&format!(
                "?obs <{dimension}> {var} . FILTER(STR({var}) = \"{}\")\n",
                literal_safe(value)
            ));
            clause_index += 1;
        }
    }
    let filter_key: Vec<String> = filters.iter().map(|(d, v)| format!("{d}={v}")).collect();
    let filter_key = filter_key.join(";");
    // The projection changes the QUERY, so it belongs in the fixture
    // key — but only when there is one, so every answer recorded
    // before projections existed keeps its key (contract §3.5).
    let projection_key = if projected.is_empty() {
        String::new()
    } else {
        format!("dims={}:", projected.join(","))
    };

    let count_query = count_query_of(&cube, &clauses);
    let count = match ask(
        ctx,
        &format!("observations:count:{cube}:{filter_key}"),
        &count_query,
    ) {
        Ok(a) => a,
        Err(refusal) => return Ok(refusal),
    };
    let total: u64 = Backend::bindings(&count.value)
        .ok()
        .and_then(|b| b.first().and_then(|b| text(b, "total")))
        .and_then(|t| t.parse().ok())
        .unwrap_or(0);
    // **A filter that matches nothing is not an empty cube** — and the
    // first cut said it was: `placeholder` was computed from the
    // FILTERED total, so one wrong lexical form answered «this cube is
    // unpublished» about 9'008 published rows, and both the contract
    // (P3) and the skill told the reader to believe it (audit of
    // 01.09.2026, ranked first of the whole tool surface). When a
    // filter misses, the cube's OWN count decides which of the two
    // states this is; its key is the unfiltered count's, so the
    // fixture store already holds it for every recorded cube.
    let (placeholder, filters_matched_nothing) = if filters_checked.is_empty() {
        (total == 0, false)
    } else if total > 0 {
        (false, false)
    } else {
        let own_count = match ask(
            ctx,
            &format!("observations:count:{cube}:"),
            &count_query_of(&cube, ""),
        ) {
            Ok(a) => a,
            Err(refusal) => return Ok(refusal),
        };
        let own: u64 = Backend::bindings(&own_count.value)
            .ok()
            .and_then(|b| b.first().and_then(|b| text(b, "total")))
            .and_then(|t| t.parse().ok())
            .unwrap_or(0);
        (own == 0, own > 0)
    };
    let query = observations_page_query(&cube, &clauses, &projected, limit, offset);
    let answer = match ask(
        ctx,
        &format!("observations:{cube}:{filter_key}:{projection_key}{limit}:{offset}"),
        &query,
    ) {
        Ok(a) => a,
        Err(refusal) => return Ok(refusal),
    };
    let bindings = match Backend::bindings(&answer.value) {
        Ok(b) => b,
        Err(e) => return Ok(upstream(format!("{e:#}"))),
    };
    struct Row {
        observation: String,
        cells: Vec<LabelledCell>,
    }
    let mut rows: Vec<Row> = Vec::new();
    for binding in bindings {
        // The ROW exists because the inner SELECT bound it. Its cells
        // are what the register gives — under a projection they hang
        // OPTIONALLY, so a row that carries none of the named
        // predicates arrives here with `?p` and `?v` unbound and must
        // still be answered (BY′ point 1).
        let Some(obs) = text(binding, "obs") else {
            continue;
        };
        let row_index = match rows.iter().position(|r| r.observation == obs) {
            Some(i) => i,
            None => {
                rows.push(Row {
                    observation: obs.clone(),
                    cells: Vec::new(),
                });
                rows.len() - 1
            }
        };
        let (Some(p), Some(c)) = (text(binding, "p"), cell(binding, "v")) else {
            continue;
        };
        let row = &mut rows[row_index];
        let cell_index = match row
            .cells
            .iter()
            .position(|(dim, existing, _)| *dim == p && existing.value == c.value)
        {
            Some(i) => i,
            None => {
                row.cells.push((p.clone(), c.clone(), Vec::new()));
                row.cells.len() - 1
            }
        };
        if let Some(label) = text(binding, "label") {
            let lang = text(binding, "labelLang").unwrap_or_default();
            let labels = &mut row.cells[cell_index].2;
            if !labels.iter().any(|(l, t)| *l == lang && *t == label) {
                labels.push((lang, label));
            }
        }
    }
    let mut stated_cells = 0usize;
    let mut not_stated_cells = 0usize;
    // P15, rebuilt from the audit of 01.09.2026. The first cut read
    // «the row's region» off the FIRST cell whose dimension name ended
    // in `/region` — twice wrong at once: a 26-canton ballot page was
    // headed by whichever canton the endpoint returned first and
    // stated `region_state: "read"` about it, and the whole election
    // family — whose region dimension carries the canton in its NAME
    // rather than «region» (see `scope::is_region_dimension`) — was
    // answered «no row carries a region» by name-blindness. A region
    // is recognised by WHERE ITS VALUE LIVES (the canton and country
    // hosts, and the politics vocabulary that C1.4 missed), and one
    // value speaks for the page only when it is the page's ONLY one.
    let mut region_hits: Vec<(String, String, Option<String>, String)> = Vec::new();
    let observations: Vec<Value> = rows
        .iter()
        .map(|row| {
            let cells: Vec<Value> = row
                .cells
                .iter()
                .map(|(dimension, c, labels)| {
                    if c.stated {
                        stated_cells += 1;
                    } else {
                        not_stated_cells += 1;
                    }
                    let label = choose_label(labels, Some(lang));
                    if let Some(value) = c.value.as_deref() {
                        // A canton or country value is a region hit on
                        // ANY dimension (the election family carries
                        // its canton under its own name); a vocabulary
                        // value counts only where the dimension itself
                        // is the region one (§17.11).
                        if c.form == "iri"
                            && (scope::is_region_value(value)
                                || scope::is_region_dimension(dimension))
                        {
                            region_hits.push((
                                dimension.clone(),
                                value.to_string(),
                                label.as_ref().map(|(l, _)| l.clone()),
                                row.observation.clone(),
                            ));
                        }
                    }
                    cell_json(
                        dimension,
                        c,
                        label.as_ref().map(|(l, lg)| (l.as_str(), lg.as_str())),
                    )
                })
                .collect();
            json!({
                "observation": row.observation,
                "cells": cells,
                "citation_shape": citation_shape(&row.cells),
            })
        })
        .collect();
    // P15 under a projection (BY′ point 2): a region that was not
    // projected is NOT absent — it was not asked for. The shape says
    // whether the cube has such a dimension at all, so the three
    // states are distinguishable without another request.
    // P15 under a projection (BY′, all four arms decided at BY″): the
    // shape is a SUBSET of the record (C2.2), so «the shape declares
    // none» is a sentence about the claim and not about the rows —
    // true, and an answer to a question nobody asked. What decides
    // here is what was ASKED FOR (the projection) and what the store
    // ANSWERED about it (the P12 gate), and the shape only where
    // nothing else can speak.
    // Three states, and each one says what THIS page knows (BY‴). A
    // fourth arm for `Gate::Absent` was written and struck: both call
    // sites of the gate return `not-found` on it, so the answer never
    // reaches this line — a state that cannot happen is not a state.
    let mut distinct: Vec<&(String, String, Option<String>, String)> = Vec::new();
    for hit in &region_hits {
        if !distinct.iter().any(|seen| seen.1 == hit.1) {
            distinct.push(hit);
        }
    }
    // A business-level fact repeats on every region row (F6): when the
    // page spans regions, no single one may head it, and the national
    // row — the one right address for an unqualified question — is
    // named so the caller does not have to find it.
    let national_row = distinct
        .iter()
        .find(|(_, value, _, _)| value.starts_with("https://ld.admin.ch/country/"))
        .map(|(_, _, _, observation)| observation.clone());
    let (region, regions_on_page): (Option<Value>, usize) = match distinct.as_slice() {
        [] => (None, 0),
        [(dimension, value, label, _)] => (
            Some(json!({
                "value": value,
                "label": label,
                // which cell it was read from, so «region» is a
                // checkable claim and not a guess about the page
                "dimension": dimension,
            })),
            1,
        ),
        several => (None, several.len()),
    };
    let region_asked_for =
        projected.is_empty() || projected.iter().any(|d| scope::is_region_dimension(d));
    let region_admitted = gated.iter().any(|(d, _)| scope::is_region_dimension(d));
    let region_state = if region.is_some() {
        "read"
    } else if regions_on_page > 1 {
        "mixed — this page spans several regions, so no single one may head it; an \
         unqualified business fact is answered by the country row, which \
         «national_row» names when the page carries one"
    } else if !region_asked_for {
        "not projected — the projection named no region dimension, so this page cannot say"
    } else if region_admitted
        || shape
            .iter()
            .any(|d| scope::is_region_dimension(&d.dimension))
    {
        // Asked for and admitted by the P12 gate (declared, or
        // undeclared and found by the bound ASK), or declared by the
        // shape: the dimension is there and these rows carry no value
        // for it.
        "the rows of this page carry none"
    } else {
        // Nothing was projected, so every cell of every row came back —
        // and none of them is a region. The shape declares none either.
        "no row of this page carries a region cell, and the shape declares no region dimension"
    };
    let returned = observations.len();
    let cells_per_row = observations
        .iter()
        .filter_map(|row| row["cells"].as_array().map(Vec::len))
        .max()
        .unwrap_or(0);
    // BY point 0: «either the default projection is the declared
    // dimensions, or the answer says which projection would fit».
    //
    // The default CANNOT be the declared set, and the reason is this
    // holding's own first rule: the shape is a SUBSET of the record
    // (C2.2, 14 declared against 51 carried), and the Ständemehr — the
    // one figure P20 forbids deriving — is among the undeclared. A
    // default projection would drop it silently, which is the worse
    // failure. So the answer SAYS what a projection would cost, in the
    // one place a caller reads before it pages again.
    //
    // The key sorts before «observations» on purpose: a client that
    // cuts the payload at a byte count (the chat's cap is 24'000) then
    // still sees the advice that would have made the next call small.
    // BY′ point 7: the advice is worth its bytes ONCE. A caller that is
    // already paging has had it; repeating ~1.7 kB on every page bills
    // the remedy in the currency of the problem. The FIRST page carries
    // the whole advice — that is the one condition, and the only one:
    // `offset == 0`. Every later page carries a pointer to it.
    let advise_in_full = offset == 0;
    let fewer_cells = if projected.is_empty() && cells_per_row > shape.len() && returned > 0 {
        let declared: Vec<&str> = shape.iter().map(|d| d.dimension.as_str()).collect();
        let projected_rows: Vec<Value> = observations
            .iter()
            .map(|row| {
                let cells: Vec<&Value> = row["cells"]
                    .as_array()
                    .map(|cells| {
                        cells
                            .iter()
                            .filter(|cell| {
                                cell["dimension"]
                                    .as_str()
                                    .is_some_and(|d| declared.contains(&d))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                json!({"observation": row["observation"], "cells": cells})
            })
            .collect();
        let bytes_now = serde_json::to_string(&observations)
            .map(|t| t.len())
            .unwrap_or(0);
        let bytes_declared = serde_json::to_string(&projected_rows)
            .map(|t| t.len())
            .unwrap_or(0);
        if advise_in_full {
            json!({
                "cells_per_row": cells_per_row,
                "rows_bytes": bytes_now,
                "how": "pass «dimensions»: a list of dimension IRIs, and the rows come back with \
                        those cells only — one call instead of paging the same table twice at a \
                        smaller limit. The IRIs are the ones lindas.describe_cube serves; a \
                        dimension the shape does not declare is admitted the same way a filter \
                        is (a bound ASK), so the Ständemehr can be projected too",
                "declared_only": {
                    "dimensions": declared,
                    "cells_per_row": declared.len(),
                    "rows_bytes": bytes_declared,
                    "warning": "the declared set is NOT everything a row carries (C2.2: 14 \
                                declared against 51 carried in the vote cube) — the Ständemehr \
                                fields are undeclared, and projecting the declared set alone \
                                drops them. Project what the question needs, not what the shape \
                                happens to declare"
                }
            })
        } else {
            // The pointer: the same knob, none of the bulk — and it
            // must not turn the bytes it saves into a REQUEST (BY″).
            // A caller that opens the table at an offset never saw the
            // first page, so «it is at offset 0» would cost it a call;
            // the IRIs are what `describe_cube` serves, which such a
            // caller has almost always already read.
            json!({
                "cells_per_row": cells_per_row,
                "rows_bytes": bytes_now,
                "declared_dimensions": declared.len(),
                "how": "pass «dimensions»: the IRIs are the ones lindas.describe_cube serves under \
                        «dimensions» — no further call is needed for them. The full advice, with \
                        the list and what it would cost in bytes, is on the answer at offset 0"
            })
        }
    } else {
        Value::Null
    };
    Ok(json!({
        "cube": cube,
        "version": scope::version_of(&cube),
        "observations": observations,
        "dimensions": if projected.is_empty() { Value::Null } else { json!(projected) },
        "cells_per_row": cells_per_row,
        "fewer_cells": fewer_cells,
        "limit": limit,
        "offset": offset,
        "returned": returned,
        "total": total,
        "truncated": (offset + returned) < total as usize,
        "stated_cells": stated_cells,
        "not_stated_cells": not_stated_cells,
        "undeclared_dimensions": undeclared,
        // The pairs actually applied — the ONE argument the envelope
        // never echoed, which made a wrongly-scoped table
        // indistinguishable from a rightly-scoped one without parsing
        // SPARQL out of provenance.query (audit of 01.09.2026).
        "filters": if filters_checked.is_empty() {
            Value::Null
        } else {
            json!(filters_checked.iter().map(|(d, v)| format!("{d}={v}")).collect::<Vec<_>>())
        },
        "filters_matched_nothing": filters_matched_nothing,
        "placeholder": placeholder,
        "region": region,
        "regions_on_page": regions_on_page,
        "national_row": national_row,
        "region_state": region_state,
        "citation_shape_read_over": if projected.is_empty() {
            "every cell of the row"
        } else {
            "the projected cells only — a title in a cell that was not projected cannot be seen \
             here, and a null is «not seen», not «not there»"
        },
        "note": "values are served as the store holds them: a decimal is the lexical form (no \
                 float arithmetic — the Ständemehr is READ, never counted from the cantonal \
                 rows), a date carries its datatype, and «not stated» is answered for both forms \
                 of cube:Undefined",
        "kind": "norm",
        "provenance": provenance(ctx, &answer),
    }))
}

/// Does a row carry a label that has the SHAPE of a legal citation
/// (P35, C7.4)? The shape is all this says: the resolution is open.
fn citation_shape(cells: &[LabelledCell]) -> Value {
    let matched = cells
        .iter()
        .any(|(_, _, labels)| labels.iter().any(|(_, label)| looks_like_citation(label)));
    if matched {
        json!({
            "citation_shape": true,
            "resolution": "open",
            "note": "a title of this row is written like a Swiss legal citation. It is handed \
                     over verbatim: no server resolves a dated act title today (fedlex's citation \
                     parser reads the last capitalised word as an abbreviation and answers \
                     unresolved), so nothing here claims an act",
        })
    } else {
        Value::Null
    }
}

/// «Bundesbeschluss vom 26.09.1952 über …» — the form, and nothing
/// about what it resolves to.
pub fn looks_like_citation(label: &str) -> bool {
    const OPENERS: [&str; 4] = ["Bundesbeschluss", "Bundesgesetz", "Verordnung", "Änderung"];
    let Some(rest) = OPENERS.iter().find_map(|opener| label.strip_prefix(opener)) else {
        return false;
    };
    let rest = rest.trim_start();
    let Some(after) = rest.strip_prefix("vom ") else {
        return false;
    };
    after.chars().next().is_some_and(|c| c.is_ascii_digit())
}

// --- 3.6 list_versions ----------------------------------------------

/// `lindas.list_versions` — the versions of a cube family (P3, P26,
/// P27).
pub fn list_versions(ctx: &Ctx, cube: &str) -> Result<Value> {
    let cube = match served_cube(cube) {
        Ok(c) => c,
        Err(refusal) => return Ok(refusal),
    };
    let family = scope::version_family(&cube);
    let versions = scope::versions_of(&cube);
    let counts = match observation_counts(ctx, &versions, &format!("family:{family}")) {
        Ok(c) => c,
        Err(refusal) => return Ok(refusal),
    };
    let profiles = match cube_profiles(ctx, &versions, &format!("family:{family}"), "de") {
        Ok(p) => p,
        Err(refusal) => return Ok(refusal),
    };
    let rows: Vec<Value> = versions
        .iter()
        .map(|v| {
            let row = profiles.row(v, counts.get(v), "de");
            json!({
                "cube": v,
                "version": scope::version_of(v),
                "observations": row["observations"],
                "status": row["status"],
                "status_unset": row["status_unset"],
                "placeholder": row["placeholder"],
            })
        })
        .collect();
    let total = rows.len();
    Ok(json!({
        "family": family,
        "versions": rows,
        "versioned": scope::version_of(&cube).is_some(),
        "limit": total,
        "returned": total,
        "total": total,
        "truncated": false,
        "note": "the versions come from the served list, by the last IRI segment; nothing in the \
                 graph links an old version to a new one, so no answer says «newer» (P27)",
        "kind": "norm",
        "provenance": profiles.provenance.clone(),
    }))
}

// --- 3.7 describe ---------------------------------------------------

/// The count query of `describe`: how many STATEMENTS the subject
/// says, counted the way the pages page them.
///
/// `COUNT(*)` over a `DISTINCT ?p ?v` subselect, not over the pattern:
/// a subject can say the same thing in two of the ten named graphs
/// (C7.5), and counting the raw pattern would promise a total no
/// sequence of pages could reach.
pub fn describe_count_query(iri: &str) -> String {
    format!(
        "{PREFIXES}SELECT (COUNT(*) AS ?total) WHERE {{\n\
         {{ SELECT DISTINCT ?p ?v WHERE {{ <{iri}> ?p ?v }} }}\n\
         }}"
    )
}

/// One page of `describe`, as a page of STATEMENTS (BX′).
///
/// The `LIMIT`/`OFFSET` sits INSIDE a `DISTINCT ?p ?v` subselect and
/// the labels are joined OUTSIDE it — the idiom `observations` already
/// uses for its rows. With the bound on the outer pattern, one
/// statement with labels in five languages ate five of the fifty and
/// the next page began fifty BINDINGS later, so statements between the
/// two were served by no page at all.
///
/// It is a FUNCTION so that the shape can be proven offline: the
/// fixture backends answer by key and never look at the query, so a
/// test over recorded answers alone could not tell this query from the
/// one it replaced (`the_page_bound_sits_inside_the_distinct_subselect`).
pub fn describe_page_query(iri: &str, limit: usize, offset: usize) -> String {
    format!(
        "{PREFIXES}SELECT ?p ?v ?label ?labelLang WHERE {{\n\
         {{ SELECT DISTINCT ?p ?v WHERE {{ <{iri}> ?p ?v }}\n\
         ORDER BY ?p ?v LIMIT {limit} OFFSET {offset} }}\n\
         OPTIONAL {{ VALUES ?labelP {{ schema:name skos:prefLabel rdfs:label }}\n\
         ?v ?labelP ?label . FILTER(LANG(?label) = \"\" || LANG(?label) IN (\"de\", \"fr\", \"it\", \"en\", \"rm\"))\n\
         BIND(LANG(?label) AS ?labelLang) }}\n\
         }} ORDER BY ?p ?v"
    )
}

/// `lindas.describe` — one IRI, asked of the ONE endpoint (P17, §8).
pub fn describe(
    ctx: &Ctx,
    iri: &str,
    lang: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Value> {
    let lang = match language(lang) {
        Ok(l) => l,
        Err(refusal) => return Ok(refusal),
    };
    let iri = match iri_safe(iri) {
        Ok(i) => normalise_value(i),
        Err(e) => return Ok(invalid(&format!("{e:#}"))),
    };
    let limit = capped(limit, MAX_STATEMENTS, 100);
    let offset = offset.unwrap_or(0) as usize;
    // The count is one row and runs at the SELECT bound; the page
    // reads the wide body and runs at the DESCRIBE bound (§8: «30 s
    // for anything that reads a body»).
    let count = match ask(
        ctx,
        &format!("describe:count:{iri}"),
        &describe_count_query(&iri),
    ) {
        Ok(a) => a,
        Err(refusal) => return Ok(refusal),
    };
    let total: usize = Backend::bindings(&count.value)
        .ok()
        .and_then(|b| b.first().and_then(|b| text(b, "total")))
        .and_then(|t| t.parse().ok())
        .unwrap_or(0);
    if total == 0 {
        return Ok(not_found(&iri));
    }
    let query = describe_page_query(&iri, limit, offset);
    let answer = match ask_wide(ctx, &format!("describe:{iri}:{limit}:{offset}"), &query) {
        Ok(a) => a,
        Err(refusal) => return Ok(refusal),
    };
    let bindings = match Backend::bindings(&answer.value) {
        Ok(b) => b,
        Err(e) => return Ok(upstream(format!("{e:#}"))),
    };
    let mut statements: Vec<LabelledCell> = Vec::new();
    for binding in bindings {
        let (Some(p), Some(c)) = (text(binding, "p"), cell(binding, "v")) else {
            continue;
        };
        // The identity of a statement is the TERM, not its lexical
        // form (BX′): `"Uri"@de` and `"Uri"@fr` are two statements to
        // the `DISTINCT (?p ?v)` that pages them, and so are the two
        // written forms of `cube:Undefined` (C3.1), which both carry
        // `value: None`. A fold on the lexical form alone merged them,
        // and then `returned` was smaller than the page the endpoint
        // served and `truncated` promised more on a page that was
        // already complete.
        let index = match statements.iter().position(|(pred, existing, _)| {
            *pred == p
                && existing.value == c.value
                && existing.form == c.form
                && existing.datatype == c.datatype
                && existing.lang == c.lang
        }) {
            Some(i) => i,
            None => {
                statements.push((p.clone(), c.clone(), Vec::new()));
                statements.len() - 1
            }
        };
        if let Some(label) = text(binding, "label") {
            let lang = text(binding, "labelLang").unwrap_or_default();
            let labels = &mut statements[index].2;
            if !labels.iter().any(|(l, t)| *l == lang && *t == label) {
                labels.push((lang, label));
            }
        }
    }
    let rows: Vec<Value> = statements
        .iter()
        .map(|(predicate, c, labels)| {
            let label = choose_label(labels, Some(lang));
            cell_json(
                predicate,
                c,
                label.as_ref().map(|(l, lg)| (l.as_str(), lg.as_str())),
            )
        })
        .collect();
    let returned = rows.len();
    Ok(json!({
        "iri": iri,
        "statements": rows,
        "limit": limit,
        "offset": offset,
        "returned": returned,
        "total": total,
        "truncated": offset + returned < total,
        "via": "endpoint",
        "note": "asked of the one endpoint this server speaks to; the IRI's own host stays \
                 reader-side (§8)",
        "kind": "norm",
        "provenance": provenance(ctx, &answer),
    }))
}

// --- 3.8 resolve_label ----------------------------------------------

/// `lindas.resolve_label` — a label for an IRI of ANY host, asked of
/// the one endpoint (P28, P29, P30).
pub fn resolve_label(ctx: &Ctx, iri: &str, lang: Option<&str>) -> Result<Value> {
    let lang = match language(lang) {
        Ok(l) => l,
        Err(refusal) => return Ok(refusal),
    };
    let iri = match iri_safe(iri) {
        Ok(i) => normalise_value(i),
        Err(e) => return Ok(invalid(&format!("{e:#}"))),
    };
    // P29: never «all labels». The query asks for the five languages
    // this holding writes — one value of the recorded corpus carries
    // 45 (C4.4) — and the fallback of P28 chooses among them.
    let query = format!(
        "{PREFIXES}SELECT ?label ?labelLang WHERE {{\n\
         VALUES ?labelP {{ schema:name skos:prefLabel rdfs:label }}\n\
         <{iri}> ?labelP ?label .\n\
         FILTER(LANG(?label) = \"\" || LANG(?label) IN (\"de\", \"fr\", \"it\", \"en\", \"rm\"))\n\
         BIND(LANG(?label) AS ?labelLang)\n\
         }} LIMIT 30"
    );
    let answer = match ask(ctx, &format!("resolve_label:{iri}:{lang}"), &query) {
        Ok(a) => a,
        Err(refusal) => return Ok(refusal),
    };
    let bindings = match Backend::bindings(&answer.value) {
        Ok(b) => b,
        Err(e) => return Ok(upstream(format!("{e:#}"))),
    };
    let mut found: Vec<(String, String)> = Vec::new();
    for binding in bindings {
        if let Some(label) = text(binding, "label") {
            let lang = text(binding, "labelLang").unwrap_or_default();
            if !found.iter().any(|(l, t)| *l == lang && *t == label) {
                found.push((lang, label));
            }
        }
    }
    let in_store = !found.is_empty();
    let chosen = choose_label(&found, Some(lang));
    let mut languages: Vec<String> = found
        .iter()
        .map(|(l, _)| {
            if l.is_empty() {
                "und".to_string()
            } else {
                l.clone()
            }
        })
        .collect();
    languages.sort();
    languages.dedup();
    Ok(json!({
        "iri": iri,
        "label": chosen.as_ref().map(|(l, _)| l.clone()),
        "label_lang": chosen.as_ref().map(|(_, l)| l.clone()),
        "languages": languages,
        "in_store": in_store,
        "note": "asked of the one endpoint: a foreign IRI is a SUBJECT the store is asked about, \
                 never a URL this server fetches (§8). in_store: false means the store carries no \
                 label for it — an answer, not a fetch elsewhere; «languages» lists what the \
                 five-language filter found, never a census of the store (P29)",
        "kind": "hint",
        "provenance": provenance(ctx, &answer),
    }))
}

#[cfg(test)]
mod tests {

    /// BY′ point 4: the projection is IN the query, and it can never
    /// drop a row.
    ///
    /// Deleting the projection from the page query left all 52 tests
    /// green before this: the fixture backends answer by semantic key
    /// (which already carries `dims=`) and never read the SPARQL. This
    /// is the test that goes red — it reads the query itself.
    #[test]
    fn the_projection_binds_its_predicates_and_never_drops_a_row() {
        let cube = "https://politics.ld.admin.ch/political-rights/popular-vote/1";
        let dims = vec![
            "https://politics.ld.admin.ch/political-rights/popular-vote/date".to_string(),
            "https://politics.ld.admin.ch/political-rights/popular-vote/standesstimmenJa"
                .to_string(),
        ];

        // The filter clauses have a PLACE in this query, and passing
        // none left it unheld (BY‴): both calls carry one now.
        let clauses = "?obs <https://politics.ld.admin.ch/political-rights/popular-vote/date> \
                       ?filterValue0 .\n";

        let plain = observations_page_query(cube, clauses, &[], 5, 0);
        assert!(
            !plain.contains("VALUES ?p"),
            "without a projection nothing binds the predicate: {plain}"
        );
        assert!(
            plain.contains("?obs ?p ?v ."),
            "and the cells are required, because every observation carries some: {plain}"
        );
        let clause_at = plain
            .find("?filterValue0")
            .expect("the filter clauses are in the query");
        let inner_bound = plain
            .find("ORDER BY ?obs LIMIT")
            .expect("the inner select bounds the page");
        assert!(
            clause_at < inner_bound,
            "the filters belong INSIDE the inner SELECT, where they choose the rows the page is \
             cut from — outside it they would filter the cells instead: {plain}"
        );

        let projected = observations_page_query(cube, clauses, &dims, 5, 0);
        assert!(
            projected.contains("?filterValue0")
                && projected.find("?filterValue0") < Some(inner_bound + 200),
            "and they keep that place under a projection: {projected}"
        );
        // EVERY projected IRI is bound, and bound IN THE VALUES SET —
        // not merely present somewhere in the query (BY‴: the test
        // named only the alphabetically last IRI, so dropping the
        // first stayed green; and once the filter clauses joined this
        // test, a bare `contains` would have found the dropped IRI in
        // a clause instead).
        let values_from = projected
            .find("VALUES ?p {")
            .expect("the projection binds ?p");
        let values_to = values_from
            + projected[values_from..]
                .find('}')
                .expect("the VALUES set closes");
        let values = &projected[values_from..values_to];
        for dimension in &dims {
            assert!(
                values.contains(&format!("<{dimension}>")),
                "the VALUES set binds «{dimension}»: {values}"
            );
        }
        assert!(
            projected.contains("VALUES ?p {")
                && projected.contains(
                    "<https://politics.ld.admin.ch/political-rights/popular-vote/standesstimmenJa>"
                ),
            "the projection binds the PREDICATE, which is what makes it a projection: {projected}"
        );
        // The invariant: the cells hang OPTIONALLY on the row the inner
        // SELECT bound, so a row carrying none of them still comes back.
        let optional_at = projected
            .find("OPTIONAL { VALUES ?p")
            .expect("the projected cells are optional — a row is never dropped");
        let inner_select_end = projected
            .find("ORDER BY ?obs LIMIT")
            .expect("the inner select bounds the page");
        assert!(
            inner_select_end < optional_at,
            "the row is bound first and the cells hang off it: {projected}"
        );
        assert_eq!(
            projected.matches("?obs ?p ?v .").count(),
            1,
            "the cell pattern appears once, inside the OPTIONAL: {projected}"
        );
        // P31/C12.3: the page still binds its subject.
        assert!(projected.contains("cube:observationSet ?set"));

        // The label join sits INSIDE the projected OPTIONAL (BY′,
        // asserted properly at BY″). Outside it, a row that binds no
        // projected predicate leaves `?v` unbound and the label
        // pattern joins on nothing — a left join against every label
        // triple in the holding.
        //
        // The first version of this could not fail: it looked for a
        // closing brace with a pattern the generated text never
        // carries and fell back to the END of the query, so «inside»
        // meant «anywhere». What decides it is the STRUCTURE: from the
        // opening of the projected OPTIONAL, count braces to its close
        // and require the label join to lie before it.
        let labels_at = projected
            .find("VALUES ?labelP")
            .expect("the labels are joined");
        let mut depth = 0usize;
        let mut closes_at = None;
        for (index, byte) in projected.bytes().enumerate().skip(optional_at) {
            match byte {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        closes_at = Some(index);
                        break;
                    }
                }
                _ => {}
            }
        }
        let closes_at = closes_at.expect("the projected OPTIONAL closes");
        assert!(
            optional_at < labels_at && labels_at < closes_at,
            "the label join belongs inside the projected OPTIONAL, which closes at {closes_at}: \
             {projected}"
        );
    }

    /// BY′: `find_cube`'s query is held the same way — the fixture key
    /// (`find_cube:<query>:<lang>:<limit>`) is blind to the filter, so
    /// only this can tell the word-wise filter from the contiguous one.
    #[test]
    fn the_find_cube_query_binds_the_scope_and_asks_for_every_word() {
        let cubes = vec![
            "https://politics.ld.admin.ch/political-rights/popular-vote-stat/1".to_string(),
            "https://politics.ld.admin.ch/political-rights/popular-vote/1".to_string(),
        ];
        let words: Vec<String> = "kennzahlen volksabstimmungen"
            .split_whitespace()
            .map(str::to_string)
            .collect();
        let query = find_cube_query(&cubes, &words);
        assert!(
            query.contains("VALUES ?cube {") && query.contains("popular-vote-stat/1>"),
            "the scope is bound, never scanned (P1, P31): {query}"
        );
        assert_eq!(
            query.matches("CONTAINS(").count(),
            2,
            "two words, two tests: {query}"
        );
        assert!(
            query.contains(" && ") && !query.contains("kennzahlen volksabstimmungen"),
            "joined by AND, and the phrase is never sent as one string: {query}"
        );
        // One word: the filter this tool always had.
        let one = find_cube_query(&cubes, &["abstimmung".to_string()]);
        assert!(
            one.contains("FILTER(CONTAINS(LCASE(STR(?name)), \"abstimmung\"))"),
            "{one}"
        );
    }

    /// BY′ point 10: `find_cube` asks for every WORD in the same name,
    /// and for one word it is the filter it always was.
    #[test]
    fn the_name_filter_asks_for_every_word() {
        let one = vec!["abstimmung".to_string()];
        assert_eq!(
            name_filter(&one),
            "CONTAINS(LCASE(STR(?name)), \"abstimmung\")",
            "one word: unchanged"
        );
        let two: Vec<String> = "kennzahlen volksabstimmungen"
            .split_whitespace()
            .map(str::to_string)
            .collect();
        let filter = name_filter(&two);
        assert_eq!(filter.matches("CONTAINS(").count(), 2);
        assert!(filter.contains(" && "), "both words, same name: {filter}");
        assert!(
            !filter.contains("kennzahlen volksabstimmungen"),
            "never the phrase — «Kennzahlen zu Volksabstimmungen» carries a word between them: \
             {filter}"
        );
    }

    use super::*;

    fn binding(value: Value) -> Value {
        json!({"v": value})
    }

    /// P13/C3.1: «not stated» is recognised in BOTH forms, and the
    /// form is carried without being interpreted.
    /// BX′: the page's bound sits INSIDE the `DISTINCT ?p ?v`
    /// subselect, and nowhere else.
    ///
    /// This is the only offline proof of the paging rewrite that can
    /// fail: the fixture backends answer by KEY and never read the
    /// query, so a test over recorded answers alone cannot tell this
    /// query from the one it replaced (which bounded the OUTER pattern
    /// and served 43 statements where it promised 50).
    #[test]
    fn the_page_bound_sits_inside_the_distinct_subselect() {
        let query = describe_page_query("https://ld.admin.ch/canton/1", 50, 50);
        let subselect = query
            .find("SELECT DISTINCT ?p ?v")
            .expect("the page is a page of statements");
        let bound = query
            .find("LIMIT 50 OFFSET 50")
            .expect("the page is bounded");
        let optional = query.find("OPTIONAL").expect("the labels are joined");
        assert!(
            subselect < bound && bound < optional,
            "the bound belongs between the DISTINCT and the label join: {query}"
        );
        assert_eq!(query.matches("LIMIT").count(), 1, "one bound, one place");
        assert!(
            query.trim_end().ends_with("ORDER BY ?p ?v"),
            "no bound on the outer pattern — that is the defect: {query}"
        );
        // P31: bound subject, no scan.
        assert!(query.contains("<https://ld.admin.ch/canton/1> ?p ?v"));
        assert!(!query.contains("STRSTARTS") && !query.contains("GRAPH ?"));

        let count = describe_count_query("https://ld.admin.ch/canton/1");
        assert!(
            count.contains("COUNT(*)") && count.contains("SELECT DISTINCT ?p ?v"),
            "the count counts what the pages page (C7.5): {count}"
        );
    }

    #[test]
    fn both_forms_of_not_stated_are_recognised() {
        let iri_form =
            cell(&binding(json!({"type": "uri", "value": UNDEFINED})), "v").expect("a cell");
        assert!(!iri_form.stated);
        assert_eq!(iri_form.form, "iri");
        assert_eq!(iri_form.value, None);

        let literal_form = cell(
            &binding(json!({"type": "literal", "value": "", "datatype": UNDEFINED})),
            "v",
        )
        .expect("a cell");
        assert!(!literal_form.stated);
        assert_eq!(literal_form.form, "literal");
        assert_eq!(literal_form.value, None);

        let stated = cell(
            &binding(json!({"type": "literal", "value": "15.5",
                            "datatype": "http://www.w3.org/2001/XMLSchema#decimal"})),
            "v",
        )
        .expect("a cell");
        assert!(stated.stated);
        assert_eq!(
            stated.value.as_deref(),
            Some("15.5"),
            "the lexical form, P14"
        );
        assert_eq!(
            stated.datatype.as_deref(),
            Some("http://www.w3.org/2001/XMLSchema#decimal")
        );
    }

    /// P14: a decimal is served as it stands — no float, no rounding.
    #[test]
    fn a_decimal_is_served_as_the_store_holds_it() {
        for lexical in ["15.5", "0.5", "69.91", "6.5"] {
            let c = cell(
                &binding(json!({"type": "literal", "value": lexical,
                                "datatype": "http://www.w3.org/2001/XMLSchema#decimal"})),
                "v",
            )
            .expect("a cell");
            assert_eq!(c.value.as_deref(), Some(lexical));
        }
    }

    /// P28: the caller's language first, then de → fr → it → en → rm;
    /// an untagged literal is «und» and is never dropped (P2).
    #[test]
    fn a_label_is_chosen_in_the_language_the_store_has() {
        let l = |lang: &str, text: &str| (lang.to_string(), text.to_string());
        let five = [l("fr", "Code"), l("de", "Kodex"), l("it", "Codice")];
        assert_eq!(choose_label(&five, Some("de")).unwrap().1, "de");
        assert_eq!(choose_label(&five, Some("it")).unwrap().1, "it");
        assert_eq!(
            choose_label(&[l("fr", "Code")], Some("de")).unwrap().1,
            "fr",
            "the fallback answers rather than refusing"
        );
        assert_eq!(
            choose_label(&[l("", "BK-APG Personen")], Some("de")).unwrap(),
            ("BK-APG Personen".to_string(), "und".to_string()),
            "an untagged name is served, not dropped (P2)"
        );
        assert_eq!(choose_label(&[], Some("de")), None);
    }

    /// P31: every scope-wide query binds its subjects with VALUES —
    /// there is no prefix scan in this server.
    #[test]
    fn a_scope_wide_query_binds_its_subjects() {
        let block = values_block("cube", &scope::all());
        assert!(block.starts_with("VALUES ?cube {"));
        assert_eq!(block.matches("https://politics.ld.admin.ch/").count(), 44);
        assert!(!block.contains("STRSTARTS"), "no prefix filter, ever");
    }

    /// P35/C7.4: the SHAPE of a citation, and nothing about what it
    /// resolves to.
    #[test]
    fn a_citation_shape_is_a_shape_and_no_claim() {
        assert!(looks_like_citation(
            "Bundesbeschluss vom 26.09.1952 über die Brotgetreideversorgung des Landes"
        ));
        assert!(looks_like_citation("Bundesgesetz vom 01.10.2021 über …"));
        assert!(looks_like_citation("Änderung vom 03.10.2003 des …"));
        assert!(!looks_like_citation(
            "Volksinitiative «für sauberes Wasser»"
        ));
        assert!(!looks_like_citation("Bundesbeschluss über die Sache"));
        assert!(!looks_like_citation("Bundesbeschluss vom Sommer"));
    }

    /// §2: the language is decidable without a request.
    #[test]
    fn an_unsupported_language_is_refused_before_any_request() {
        assert_eq!(language(Some("de")), Ok("de"));
        assert_eq!(language(None), Ok("de"));
        assert_eq!(language(Some("rm")), Ok("rm"));
        let refusal = language(Some("es")).expect_err("es is not one of the five");
        assert_eq!(refusal["error"], "invalid-input");
        // Since BY‴ this refusal carries what its siblings carry: the
        // shape it accepts, and the sentence that says it cost nothing.
        assert_eq!(refusal["accepted"], "de | fr | it | en | rm");
        assert!(refusal["note"]
            .as_str()
            .unwrap()
            .contains("costs no request"));
        assert!(refusal["detail"].as_str().unwrap().contains("«es»"));
    }

    /// P1: a cube outside the served list is not-found — and it is
    /// decided without a request.
    #[test]
    fn a_cube_outside_the_list_is_not_found() {
        assert!(
            served_cube("https://politics.ld.admin.ch/political-rights/popular-vote/1").is_ok()
        );
        let refusal = served_cube("https://politics.ld.admin.ch/political-rights/popular-vote/2")
            .expect_err("not served");
        assert_eq!(refusal["error"], "not-found");
        let malformed = served_cube("nonsense").expect_err("malformed");
        assert_eq!(malformed["error"], "invalid-input");
    }

    /// P32/C12.2: a 406 is a permanent answer about the request.
    #[test]
    fn the_endpoints_refusals_are_typed_apart() {
        let busy = backend_refusal(&anyhow::anyhow!(
            "upstream-busy: retry_after_ms=1200: the polite brake …"
        ));
        assert_eq!(busy["error"], "upstream-busy");
        assert_eq!(busy["retry_after_ms"], 1200);
        let unacceptable = backend_refusal(&anyhow::anyhow!("bad-request: HTTP 406"));
        assert_eq!(unacceptable["error"], "invalid-input");
        let down = backend_refusal(&anyhow::anyhow!("upstream-unavailable: SPARQL select …"));
        assert_eq!(down["error"], "upstream-unavailable");
    }
}
