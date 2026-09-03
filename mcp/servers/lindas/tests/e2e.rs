//! The offline suite: every tool against the recorded answers, and one
//! recording pass that is `#[ignore]`d because it hits the live
//! endpoint.
//!
//! The recording pass is deliberate and braked: one request at a time,
//! the polite bucket in front of it, every answer written under its
//! semantic key with the day it was recorded (`tests/fixtures/INDEX.txt`).
//! What it costs is not trusted but MEASURED — the counting double
//! replays the same call sequence in `the_recording_pass_costs_what_the_report_says`.

use oh_mcp_lindas::backend::{Backend, LINDAS_ENDPOINT};
use oh_mcp_lindas::domain::{self, Ctx};
use oh_mcp_lindas::{scope, server};

const VOTE: &str = "https://politics.ld.admin.ch/political-rights/popular-vote/1";
const VOTE_DIM: &str = "https://politics.ld.admin.ch/political-rights/popular-vote";
const REFERENDUM: &str = "https://politics.ld.admin.ch/political-rights/referendum/1";
const REFERENDUM_STAT: &str = "https://politics.ld.admin.ch/political-rights/referendum-stat/1";
const INITIATIVE_STAT: &str =
    "https://politics.ld.admin.ch/political-rights/popular-initiative-stat/1";
const COUNCILLOR: &str = "https://politics.ld.admin.ch/fc/cube-councillor";
const VESTED: &str = "https://politics.ld.admin.ch/fch/apg/vested-interest/1";
const CANDIDATES_2023: &str =
    "https://politics.ld.admin.ch/national-council-election/candidates/2023";
const CANDIDATES_2027: &str =
    "https://politics.ld.admin.ch/national-council-election/candidates/2027";
const LIST_RESULTS_2023: &str =
    "https://politics.ld.admin.ch/national-council-election/list-results/2023";
const CANTON_ZH: &str = "https://ld.admin.ch/canton/1";
const COUNTRY_CH: &str = "https://ld.admin.ch/country/CHE";
const SEX_CONCEPT: &str = "https://register.ld.admin.ch/i14y/concept/sex/2";
/// The day the fixtures were recorded — every offline answer is «as of»
/// this, because the moment is injected and never read from a clock.
const TODAY: &str = "2026-08-30";

/// What ONE full recording pass costs, and over how many keys — the
/// figure `ENGINE.md`, `README.md` and `engine.manifest.json` state as
/// «<requests> requests over <keys> keys», and the one
/// `tools/check.sh` reads out of this file to hold them to it.
const PASS_REQUESTS: usize = 108;
/// What the advice costs, and what the pointer costs instead — the
/// figures `TOOLSET-v0.md` §3.5 states, read by `tools/check.sh` from
/// here so the contract cannot drift from them (BY‴).
const ADVICE_BYTES: usize = 1721;
const ADVICE_OF_ANSWER_BYTES: usize = 14375;
const POINTER_BYTES: usize = 294;
const PASS_KEYS: usize = 80;

fn fixtures_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn ctx() -> Ctx {
    Ctx {
        backend: Backend::Fixtures {
            dir: fixtures_dir(),
        },
        today: TODAY.into(),
    }
}

/// The recorded index: `<file> <key> <recorded>` lines, notes with `#`.
fn recorded_keys() -> Vec<(String, String, String)> {
    std::fs::read_to_string(fixtures_dir().join("INDEX.txt"))
        .expect("INDEX.txt")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            // `<file> <key> <recorded>` — and a KEY may carry spaces
            // (a search query is one), so it is what lies between the
            // first token and the last.
            let file = l.split(' ').next().unwrap_or_default().to_string();
            let recorded = l.rsplit(' ').next().unwrap_or_default().to_string();
            let key = oh_mcp_common::fixtures::key_of(l)
                .unwrap_or_default()
                .to_string();
            (file, key, recorded)
        })
        .collect()
}

/// The call sequence the recording pass makes, and the offline suite
/// replays. One function, so «what was recorded» and «what is tested»
/// cannot drift apart.
fn call_sequence(ctx: &Ctx) {
    // every tool's contract example …
    let _ = domain::list_cubes(ctx, None, None, Some(50), None);
    let _ = domain::list_cubes(ctx, Some("fch/apg"), None, None, None);
    let _ = domain::find_cube(ctx, "Abstimmung", None, None);
    let _ = domain::find_cube(ctx, "Interessenbindung", None, None);
    // BY′ point 10: two words, as a model asks — «Kennzahlen zu
    // Volksabstimmungen» carries a word between them, so the
    // contiguous filter found nothing.
    let _ = domain::find_cube(ctx, "Kennzahlen Volksabstimmungen", None, None);
    for cube in [
        VOTE,
        REFERENDUM,
        REFERENDUM_STAT,
        INITIATIVE_STAT,
        COUNCILLOR,
        VESTED,
        CANDIDATES_2023,
        CANDIDATES_2027,
        LIST_RESULTS_2023,
    ] {
        let _ = domain::describe_cube(ctx, cube, None, None, None);
    }
    // … the filter vocabularies the acceptance families need …
    let _ = domain::dimension_values(ctx, VOTE, &format!("{VOTE_DIM}/typologie"), None, None);
    let _ = domain::dimension_values(ctx, VOTE, &format!("{VOTE_DIM}/region"), None, None);
    let _ = domain::dimension_values(
        ctx,
        INITIATIVE_STAT,
        "https://politics.ld.admin.ch/political-rights/popular-initiative-stat/stand",
        None,
        None,
    );
    let _ = domain::dimension_values(
        ctx,
        LIST_RESULTS_2023,
        "https://politics.ld.admin.ch/national-council-election/list-results/listName",
        None,
        Some(200),
    );
    // … the six acceptance families of §7 …
    // 1 — votes per canton, with the Ständemehr (national row)
    let _ = domain::observations(
        ctx,
        VOTE,
        &[
            (format!("{VOTE_DIM}/date"), "1971-02-07".into()),
            (format!("{VOTE_DIM}/region"), COUNTRY_CH.into()),
        ],
        &[],
        None,
        Some(5),
        None,
    );
    // 1p — the same row, PROJECTED (BY point 0): two declared cells and
    // the undeclared Ständemehr, which is what the measurement's
    // question actually needed
    let _ = domain::observations(
        ctx,
        VOTE,
        &[
            (format!("{VOTE_DIM}/date"), "1971-02-07".into()),
            (format!("{VOTE_DIM}/region"), COUNTRY_CH.into()),
        ],
        &[
            format!("{VOTE_DIM}/date"),
            format!("{VOTE_DIM}/region"),
            format!("{VOTE_DIM}/standesstimmenJa"),
        ],
        None,
        Some(5),
        None,
    );
    // 1p2 — the same, projected to DECLARED dimensions only: P12's
    // first step, which costs no ASK at all
    let _ = domain::observations(
        ctx,
        VOTE,
        &[
            (format!("{VOTE_DIM}/date"), "1971-02-07".into()),
            (format!("{VOTE_DIM}/region"), COUNTRY_CH.into()),
        ],
        &[format!("{VOTE_DIM}/date"), format!("{VOTE_DIM}/region")],
        None,
        Some(5),
        None,
    );
    // 1b — the same vote in one canton
    let _ = domain::observations(
        ctx,
        VOTE,
        &[
            (format!("{VOTE_DIM}/date"), "1971-02-07".into()),
            (format!("{VOTE_DIM}/region"), CANTON_ZH.into()),
        ],
        &[],
        None,
        Some(5),
        None,
    );
    // 1c — a filter on the UNDECLARED numeric Ständemehr (P12 step b)
    let _ = domain::observations(
        ctx,
        VOTE,
        &[
            (format!("{VOTE_DIM}/date"), "1971-02-07".into()),
            (format!("{VOTE_DIM}/standesstimmenJa"), "15.5".into()),
        ],
        &[],
        None,
        Some(5),
        None,
    );
    // 1d — a dimension the cube does not carry at all (P12 step c)
    let _ = domain::observations(
        ctx,
        VOTE,
        &[(format!("{VOTE_DIM}/gibtEsNicht"), "x".into())],
        &[],
        None,
        Some(5),
        None,
    );
    // 2 — referendum bills of one year, and their states
    let _ = domain::observations(
        ctx,
        REFERENDUM,
        &[(
            "https://politics.ld.admin.ch/political-rights/referendum/beschlussdatumJahr".into(),
            "2023".into(),
        )],
        &[],
        None,
        Some(5),
        None,
    );
    let _ = domain::observations(ctx, REFERENDUM_STAT, &[], &[], None, Some(5), None);
    // 2m — a filter that matches NOTHING on a full cube: the state the
    // audit ranked first (01.09.2026). The answer must say «your
    // filter matched nothing», never «this cube is empty» — the
    // cube's own count decides which, and its key is the unfiltered
    // count the pass records anyway.
    let _ = domain::observations(
        ctx,
        INITIATIVE_STAT,
        &[(
            "https://politics.ld.admin.ch/political-rights/popular-initiative-stat/stand".into(),
            "kein-solcher-stand".into(),
        )],
        &[],
        None,
        Some(5),
        None,
    );
    // 3 — Volksinitiativen: the states cube
    let _ = domain::observations(ctx, INITIATIVE_STAT, &[], &[], None, Some(5), None);
    // 4 — Bundesrat by canton
    let _ = domain::observations(
        ctx,
        COUNCILLOR,
        &[(
            "http://schema.org/addressRegion".into(),
            "https://ld.admin.ch/canton/2".into(),
        )],
        &[],
        None,
        Some(5),
        None,
    );
    // 5 — Interessenbindungen
    let _ = domain::observations(ctx, VESTED, &[], &[], None, Some(3), None);
    // 6a — seats of a list in a canton, READ from list-results
    let _ = domain::observations(
        ctx,
        LIST_RESULTS_2023,
        &[(
            "https://politics.ld.admin.ch/national-council-election/list-results/hasCanton".into(),
            CANTON_ZH.into(),
        )],
        &[],
        None,
        Some(5),
        None,
    );
    // 6b — who was elected on a list
    let _ = domain::observations(
        ctx,
        CANDIDATES_2023,
        &[
            (
                "https://politics.ld.admin.ch/national-council-election/candidates/hasCanton"
                    .into(),
                CANTON_ZH.into(),
            ),
            (
                "https://politics.ld.admin.ch/national-council-election/candidates/elected".into(),
                "true".into(),
            ),
        ],
        &[],
        None,
        Some(5),
        None,
    );
    // … a placeholder cube answers the state, not a not-found …
    let _ = domain::observations(ctx, CANDIDATES_2027, &[], &[], None, Some(5), None);
    // … the versions of two families …
    let _ = domain::list_versions(ctx, CANDIDATES_2019);
    let _ = domain::list_versions(ctx, COUNCILLOR);
    // … one IRI described — page one, page two and the two of them in
    // one page, which is what proves the paging pages STATEMENTS
    // (BX') — one label resolved on three hosts …
    let _ = domain::describe(ctx, CANTON_ZH, None, Some(50), None);
    let _ = domain::describe(ctx, CANTON_ZH, None, Some(50), Some(50));
    let _ = domain::describe(ctx, CANTON_ZH, None, Some(100), None);
    let _ = domain::describe(
        ctx,
        "https://politics.ld.admin.ch/fc/cube-chancellor/observation/corina-casanova",
        None,
        Some(50),
        None,
    );
    let _ = domain::describe(
        ctx,
        "https://politics.ld.admin.ch/nothing/at/all",
        None,
        None,
        None,
    );
    let _ = domain::resolve_label(ctx, CANTON_ZH, None);
    let _ = domain::resolve_label(ctx, SEX_CONCEPT, None);
    let _ = domain::resolve_label(
        ctx,
        "https://ld.admin.ch/vocabulary/CreativeWorkStatus/Published",
        None,
    );
}

const CANDIDATES_2019: &str =
    "https://politics.ld.admin.ch/national-council-election/candidates/2019";

/// A targeted re-record: only the three `resolve_label` keys, after
/// P29's language filter was added to that query. Three requests, not
/// a whole pass — a corrected query should not cost the endpoint the
/// other 88.
#[test]
#[ignore = "hits the live endpoint once per key; run deliberately"]
fn record_fixtures_labels() {
    let ctx = Ctx {
        backend: Backend::recording(LINDAS_ENDPOINT, fixtures_dir()),
        today: TODAY.into(),
    };
    for iri in [
        CANTON_ZH,
        SEX_CONCEPT,
        "https://ld.admin.ch/vocabulary/CreativeWorkStatus/Published",
    ] {
        let out = domain::resolve_label(&ctx, iri, None).expect("record");
        println!(
            "resolve_label {iri}: label={} in_store={} languages={}",
            out["label"], out["in_store"], out["languages"]
        );
    }
}

/// The one two-word `find_cube` window of BY′ point 10.
///
/// Budget named in advance: **at most 4 requests** — the name query is
/// one, the observation counts of the served scope are already
/// recorded, and the margin is for a retry.
/// `cargo test --test e2e record_fixtures_find_cube -- --ignored --nocapture --test-threads 1`
#[test]
#[ignore = "hits the live endpoint; run deliberately"]
fn record_fixtures_find_cube() {
    let ctx = Ctx {
        backend: Backend::recording(LINDAS_ENDPOINT, fixtures_dir()),
        today: TODAY.into(),
    };
    let out = domain::find_cube(&ctx, "Kennzahlen Volksabstimmungen", None, None).expect("record");
    println!(
        "find_cube «Kennzahlen Volksabstimmungen»: {} hits, first {}",
        out["returned"], out["hits"][0]["cube"]
    );
}

/// What that recording costs, measured before it ran.
#[test]
fn the_find_cube_recording_costs_what_the_report_says() {
    use std::sync::atomic::Ordering;
    let (backend, selects) = Backend::counting(fixtures_dir());
    let ctx = Ctx {
        backend,
        today: TODAY.into(),
    };
    let _ = domain::find_cube(&ctx, "Kennzahlen Volksabstimmungen", None, None);
    let requests = selects.load(Ordering::SeqCst);
    println!("find_cube recording: {requests} requests");
    assert_eq!(
        requests, 2,
        "the name query over the fixed 44-cube scope and the observation counts beside it — two \
         per call, so the two recorded runs of this window cost FOUR (BY″ corrects «one request \
         each»), inside the four named in advance"
    );
    // …and a search that finds NOTHING costs one request, not two: the
    // count query is one call over the fixed 44-cube scope, and with
    // no hits there is nothing to attach its answer to.
    //
    // The empty answer is BUILT (BY‴): asserting this over a key with
    // no fixture measured an upstream failure, not an empty search —
    // the call died at the first request and the guard walked past it.
    {
        use oh_mcp_lindas::backend::key_file;
        let dir = std::env::temp_dir().join(format!("oh-lindas-empty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a fixture directory of our own");
        std::fs::write(
            key_file(&dir, "find_cube:Quantencomputer:de:10"),
            r#"{"results": {"bindings": []}}"#,
        )
        .expect("the empty answer");
        let (backend, selects) = Backend::counting(dir.clone());
        let ctx = Ctx {
            backend,
            today: TODAY.into(),
        };
        let empty = domain::find_cube(&ctx, "Quantencomputer", None, None).expect("runs");
        assert_eq!(
            empty["kind"], "hint",
            "an empty search is an ANSWER: {empty}"
        );
        assert_eq!(empty["returned"], 0);
        assert_eq!(empty["total"], 0);
        assert!(empty.get("error").is_none(), "and not a failure: {empty}");
        assert_eq!(
            selects.load(Ordering::SeqCst),
            1,
            "one request: the name query. The counts are not asked for, because there is nothing \
             to attach them to"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// The ONE projected call, recorded at BY point 0: `dimensions` makes
/// a new query, so it makes a new fixture key.
///
/// Budget named in advance: **at most 10 requests**; what it costs is
/// measured by `the_projected_recording_costs_what_the_report_says`
/// before this runs.
/// `cargo test --test e2e record_fixtures_projection -- --ignored --nocapture --test-threads 1`
#[test]
#[ignore = "hits the live endpoint; run deliberately"]
fn record_fixtures_projection() {
    let ctx = Ctx {
        backend: Backend::recording(LINDAS_ENDPOINT, fixtures_dir()),
        today: TODAY.into(),
    };
    projection_sequence(&ctx);
}

/// The two projected calls, as the recorder and the counter both run
/// them: one that names an undeclared dimension (P12 step b, one ASK)
/// and one that names only declared ones (step a, no ASK at all).
fn projection_sequence(ctx: &Ctx) {
    let filters = [
        (format!("{VOTE_DIM}/date"), "1971-02-07".to_string()),
        (format!("{VOTE_DIM}/region"), COUNTRY_CH.to_string()),
    ];
    for projection in [
        vec![
            format!("{VOTE_DIM}/date"),
            format!("{VOTE_DIM}/region"),
            format!("{VOTE_DIM}/standesstimmenJa"),
        ],
        vec![format!("{VOTE_DIM}/date"), format!("{VOTE_DIM}/region")],
    ] {
        let out = domain::observations(ctx, VOTE, &filters, &projection, None, Some(5), None)
            .expect("runs");
        println!(
            "projected {}: returned={} cells_per_row={} undeclared={}",
            projection.len(),
            out["returned"],
            out["cells_per_row"],
            out["undeclared_dimensions"]
        );
    }
}

/// What the projected recording costs — measured before it ran.
#[test]
fn the_projected_recording_costs_what_the_report_says() {
    use std::sync::atomic::Ordering;
    let (backend, selects) = Backend::counting(fixtures_dir());
    let ctx = Ctx {
        backend,
        today: TODAY.into(),
    };
    projection_sequence(&ctx);
    let requests = selects.load(Ordering::SeqCst);
    println!("projected recording: {requests} requests");
    assert_eq!(
        requests, 7,
        "the wide projection: shape, one ASK for the undeclared Ständemehr, count, page — four; \
         the declared-only one: shape, count, page — three. Seven, inside the ten named in \
         advance"
    );
}

/// The describe keys, re-recorded at BX' because the QUERY changed:
/// the page's LIMIT/OFFSET moved into a `DISTINCT (?p ?v)` subselect
/// and the count became a count of statements, so the answers recorded
/// against the old query would have been a recording of a query this
/// crate no longer sends.
///
/// Budget named in advance: **at most 15 requests**. The sequence is
/// five `describe` calls; each sends a count and (where the subject
/// exists) a page, which is nine — the margin is for one retry.
/// `cargo test --test e2e record_fixtures_describe -- --ignored --nocapture --test-threads 1`
#[test]
#[ignore = "hits the live endpoint; run deliberately"]
fn record_fixtures_describe() {
    let ctx = Ctx {
        backend: Backend::recording(LINDAS_ENDPOINT, fixtures_dir()),
        today: TODAY.into(),
    };
    describe_sequence(&ctx);
}

/// The five calls the describe recorder makes — the ONE sequence, so
/// the counting double below measures the pass that actually runs.
fn describe_sequence(ctx: &Ctx) {
    for (iri, limit, offset) in [
        (CANTON_ZH, Some(50), None),
        (CANTON_ZH, Some(50), Some(50)),
        (CANTON_ZH, Some(100), None),
        (
            "https://politics.ld.admin.ch/fc/cube-chancellor/observation/corina-casanova",
            Some(50),
            None,
        ),
        ("https://politics.ld.admin.ch/nothing/at/all", None, None),
    ] {
        let out = domain::describe(ctx, iri, None, limit, offset).expect("runs");
        println!(
            "describe {iri} limit={limit:?} offset={offset:?}: returned={} total={} error={}",
            out["returned"], out["total"], out["error"]
        );
    }
}

/// What the describe re-recording cost, measured BEFORE it ran and
/// kept as the record afterwards — the lesson of BX, where two of
/// three passes went uncounted and the total could only be bounded.
#[test]
fn the_describe_recording_costs_what_the_report_says() {
    use std::sync::atomic::Ordering;
    let (backend, selects) = Backend::counting(fixtures_dir());
    let ctx = Ctx {
        backend,
        today: TODAY.into(),
    };
    describe_sequence(&ctx);
    let requests = selects.load(Ordering::SeqCst);
    println!("describe re-recording: {requests} requests");
    assert_eq!(
        requests, 9,
        "four subjects with a page (two requests each) and one that does not exist (a count \
         only): nine, inside the fifteen named in advance"
    );
}

/// The recording pass — deliberate, braked, one request at a time:
/// `cargo test --test e2e record_fixtures -- --ignored --nocapture --test-threads 1`
///
/// Budget named in advance: **at most 120 requests** for the whole
/// pass. What it actually cost is measured by the counting double in
/// `the_recording_pass_costs_what_the_report_says`.
#[test]
#[ignore = "hits the live endpoint once per key; run deliberately"]
fn record_fixtures() {
    let ctx = Ctx {
        backend: Backend::recording(LINDAS_ENDPOINT, fixtures_dir()),
        today: TODAY.into(),
    };
    call_sequence(&ctx);
    let keys = recorded_keys();
    println!("recorded {} keys", keys.len());
}

/// What the recording pass cost, replayed over the fixtures: the
/// budget is measured, not trusted (the fedlex discipline).
#[test]
fn the_recording_pass_costs_what_the_report_says() {
    use std::sync::atomic::Ordering;
    let (backend, selects) = Backend::counting(fixtures_dir());
    let ctx = Ctx {
        backend,
        today: TODAY.into(),
    };
    call_sequence(&ctx);
    let requests = selects.load(Ordering::SeqCst);
    let distinct: std::collections::BTreeSet<String> =
        ctx.backend.seen_keys().into_iter().collect();
    println!(
        "recording pass: {requests} requests over {} keys",
        distinct.len()
    );
    assert!(
        requests <= 120,
        "the budget named in advance was 120 requests; the pass makes {requests}"
    );
    // And the figure three documents state is the figure, EXACTLY —
    // both halves of it (BY′: the key count was only asserted
    // relatively, and the gate in tools/check.sh read the number out
    // of this message rather than out of an assertion). The two
    // constants are what the gate reads; the message is built from
    // them, so message and assertion cannot part company.
    assert_eq!(
        requests, PASS_REQUESTS,
        "the recording pass costs what the engine documents say: {PASS_REQUESTS} requests over \
         {PASS_KEYS} keys — change them in the same commit as the sequence"
    );
    assert_eq!(
        distinct.len(),
        PASS_KEYS,
        "and over that many keys: {PASS_REQUESTS} requests over {PASS_KEYS} keys"
    );
    assert_eq!(
        distinct.len(),
        recorded_keys().len(),
        "every key the sequence asks for is recorded, and nothing else is"
    );
}

/// Every fixture is indexed with the day it was recorded, and nothing
/// lies in the directory that the index does not name.
#[test]
fn every_fixture_is_indexed_with_the_day_it_was_recorded() {
    let keys = recorded_keys();
    assert!(keys.len() > 40, "the index carries the recorded keys");
    let dir = fixtures_dir();
    for (file, key, recorded) in &keys {
        assert!(!key.is_empty(), "a line without a key: {file}");
        assert_eq!(recorded.len(), 10, "«{recorded}» is no date ({key})");
        assert!(
            dir.join(file).exists(),
            "{file} is named for «{key}» and is not there"
        );
    }
    let indexed: std::collections::BTreeSet<&str> =
        keys.iter().map(|(f, _, _)| f.as_str()).collect();
    for entry in std::fs::read_dir(&dir).expect("fixtures dir") {
        let name = entry
            .expect("dir entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        if name == "INDEX.txt" {
            continue;
        }
        assert!(
            indexed.contains(name.as_str()),
            "{name} is a fixture nobody indexes"
        );
    }
}

// --- P1, P2, P3, P5, P7: the served scope and its states -------------

/// P1: the scope is the list of 44; P3: status as the vocabulary
/// answers it, with `placeholder` orthogonal to it.
#[test]
fn list_cubes_serves_the_scope_with_its_states() {
    let ctx = ctx();
    let out = domain::list_cubes(&ctx, None, None, Some(50), None).expect("runs");
    assert_eq!(out["kind"], "norm", "{out}");
    assert_eq!(out["total"], 44, "the served scope is a list of 44 (P1)");
    assert_eq!(
        out["returned"], 44,
        "the whole scope fits under a limit of 50"
    );
    assert_eq!(out["truncated"], false);
    assert_eq!(out["provenance"]["licence"], "not stated at the source");
    assert_eq!(out["provenance"]["access"], "public (I14Y)");
    assert_eq!(out["provenance"]["as_of"], TODAY);
    let cubes = out["cubes"].as_array().expect("cubes");
    // P3: 14 of 44 carry no status, four hold nothing — and the two
    // facts are independent fields, not one enum.
    let status_unset = cubes.iter().filter(|c| c["status_unset"] == true).count();
    let placeholders = cubes.iter().filter(|c| c["placeholder"] == true).count();
    assert_eq!(status_unset, 14, "C5.1");
    assert_eq!(placeholders, 4, "C5.4 — the 2027 election cubes");
    let both = cubes
        .iter()
        .filter(|c| c["status_unset"] == true && c["placeholder"] == true)
        .count();
    assert_eq!(
        both, 2,
        "two cubes are BOTH a placeholder and status-less — which one state enum could not say"
    );
    // P3: where a status is carried, it is the IRI AND its decoded
    // label, in the language that answered.
    let published = cubes
        .iter()
        .find(|c| c["status_unset"] == false)
        .expect("a published cube");
    assert!(published["status"]
        .as_str()
        .unwrap()
        .contains("CreativeWorkStatus/Published"));
    assert_eq!(published["status_label"], "Publiziert");
    assert_eq!(published["status_label_lang"], "de");
    // P5: a description is optional, carries its own language, and is
    // never served in place of a missing name.
    let described = cubes
        .iter()
        .find(|c| c["description"].is_string())
        .expect("some cube carries a description");
    assert!(described["description_lang"].is_string());
    assert!(
        cubes.iter().all(|c| c["name"].is_string()),
        "every cube has a name of its own (C4.1)"
    );
}

/// P2/P7: nine cubes carry a name with NO language tag — served as
/// «und», never dropped.
#[test]
fn the_untagged_names_of_the_fch_apg_family_are_served() {
    let ctx = ctx();
    let out = domain::list_cubes(&ctx, Some("fch/apg"), None, None, None).expect("runs");
    let cubes = out["cubes"].as_array().expect("cubes");
    assert_eq!(cubes.len(), 9, "the register is nine cubes (C0.1)");
    assert!(
        cubes.iter().all(|c| c["name_lang"] == "und"),
        "every fch/apg name carries no language tag (C4.1): {out}"
    );
    assert!(cubes.iter().any(|c| c["name"]
        .as_str()
        .unwrap_or_default()
        .contains("Interessenbindungen")));
    // P7: and they are findable despite it.
    let found = domain::find_cube(&ctx, "Interessenbindung", None, None).expect("runs");
    assert_eq!(found["kind"], "hint");
    let hits = found["hits"].as_array().expect("hits");
    assert!(
        hits.iter().any(|h| h["cube"] == VESTED),
        "a LANG(de) filter would lose this cube: {found}"
    );
}

// --- P8, P9, P10, P11: the declared shape ---------------------------

/// P37: attribution is named ONCE and never repeated per row.
///
/// The cube profile carries the publisher — that is where a caller
/// looks for it — and the engine manifest carries the authority for
/// the holding as a whole. An observation row carries neither: all 44
/// cubes name the same publisher (C7.1), so repeating it on every one
/// of 92'688 rows would be bytes spent to say nothing new.
///
/// Written at BX commit 2, when the manifest this point waited for
/// came into being; until then the row said «deferred» and why.
#[test]
fn attribution_is_named_once_and_never_repeated_per_row() {
    let ctx = ctx();
    // Once, in the profile.
    let profile = domain::describe_cube(&ctx, VOTE, None, None, None).expect("runs");
    assert!(
        profile["publisher"]
            .as_str()
            .is_some_and(|p| p.starts_with("https://")),
        "the cube profile names its publisher: {}",
        profile["publisher"]
    );
    // Never on a row.
    let rows = domain::observations(
        &ctx,
        VOTE,
        &[
            (format!("{VOTE_DIM}/date"), "1971-02-07".into()),
            (format!("{VOTE_DIM}/region"), COUNTRY_CH.into()),
        ],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    for row in rows["observations"].as_array().expect("rows") {
        for field in ["publisher", "creator", "contributor"] {
            assert!(
                row.get(field).is_none(),
                "an observation row repeats «{field}»: {row}"
            );
        }
    }
    // And once for the holding, in the engine manifest beside this crate.
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("engine.manifest.json"),
        )
        .expect("engine.manifest.json is committed beside the crate"),
    )
    .expect("the manifest is JSON");
    let authority = manifest["holding"]["authority"]
        .as_str()
        .expect("the manifest names the authority");
    assert!(
        authority.contains("Federal Chancellery"),
        "the publisher of the political cubes is named in the manifest, once: {authority}"
    );
}

/// P9: what `describe_cube` answers is the DECLARED shape, and it says
/// so — the record carries more, and the sample shows how much.
#[test]
fn describe_cube_serves_the_declared_shape_and_says_it_is_only_that() {
    let ctx = ctx();
    let out = domain::describe_cube(&ctx, VOTE, None, None, None).expect("runs");
    assert_eq!(out["kind"], "norm", "{out}");
    assert_eq!(out["declared_only"], true);
    assert_eq!(out["sampled"], true);
    assert_eq!(out["dimensions_total"], 14, "the shape declares 14 (C2.2)");
    let sample = out["carried_predicates_sample"].as_array().expect("sample");
    assert_eq!(sample.len(), 51, "one observation carries 51 (C2.2)");
    let undeclared = out["undeclared_in_sample"].as_array().expect("undeclared");
    assert_eq!(undeclared.len(), 37, "37 of them are undeclared");
    assert!(
        undeclared
            .iter()
            .any(|d| d.as_str().unwrap().ends_with("/standesstimmenJa")),
        "the numeric Ständemehr is among them: {undeclared:?}"
    );
    // P6: the three dates, each with the granularity it was written in.
    assert!(out["dates"]["created"].is_string());
    assert!(out["dates"]["published"].is_string());
    assert!(["date", "dateTime"].contains(&out["dates"]["granularity"].as_str().unwrap_or("")));
    // P8: the shape answer is capped and paged, with the original size.
    let capped = domain::describe_cube(&ctx, VOTE, None, Some(5), None).expect("runs");
    assert_eq!(capped["returned"], 5);
    assert_eq!(capped["dimensions_total"], 14);
    assert_eq!(capped["truncated"], true, "the cap says so: {capped}");
    // P36: where the cube carries a viewer, the profile serves it.
    assert!(
        out["viewer"].is_string() || out["viewer"].is_null(),
        "a viewer is served where it exists and absent where it does not"
    );
    // P10: a dimension whose class is not declared says «unknown».
    let kinds: Vec<&str> = out["dimensions"]
        .as_array()
        .expect("dimensions")
        .iter()
        .map(|d| d["dimension_kind"].as_str().unwrap_or(""))
        .collect();
    assert!(kinds
        .iter()
        .all(|k| ["key", "measure", "attribute", "unknown"].contains(k)));
}

/// P3/C5.4: a published cube that holds nothing answers the state —
/// never a not-found.
#[test]
fn a_placeholder_cube_answers_that_it_is_one() {
    let ctx = ctx();
    let out = domain::describe_cube(&ctx, CANDIDATES_2027, None, None, None).expect("runs");
    assert!(
        out.get("error").is_none(),
        "an answer, not a refusal: {out}"
    );
    assert_eq!(out["observations"], 0);
    assert_eq!(out["placeholder"], true);
    assert_eq!(
        out["dimensions_total"], 0,
        "a cube with no rows has no shape"
    );
    let rows =
        domain::observations(&ctx, CANDIDATES_2027, &[], &[], None, Some(5), None).expect("runs");
    assert!(rows.get("error").is_none(), "{rows}");
    assert_eq!(rows["total"], 0);
    assert_eq!(rows["placeholder"], true);
    assert_eq!(rows["observations"].as_array().expect("rows").len(), 0);
}

// --- P12: the three-step dimension rule ------------------------------

/// P12 in all three steps, on one cube: declared → no request;
/// undeclared but carried → accepted and NAMED; absent → not-found
/// echoing the pair, never invalid-input.
#[test]
fn the_three_step_dimension_rule_decides_every_filter() {
    let ctx = ctx();
    // (a) declared by the shape
    let declared = domain::observations(
        &ctx,
        VOTE,
        &[
            (format!("{VOTE_DIM}/date"), "1971-02-07".into()),
            (format!("{VOTE_DIM}/region"), COUNTRY_CH.into()),
        ],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    assert!(declared.get("error").is_none(), "{declared}");
    assert_eq!(
        declared["undeclared_dimensions"]
            .as_array()
            .expect("array")
            .len(),
        0,
        "a declared dimension costs no ASK and is not reported as undeclared"
    );
    // (b) undeclared, and the bound ASK finds it
    let undeclared = domain::observations(
        &ctx,
        VOTE,
        &[
            (format!("{VOTE_DIM}/date"), "1971-02-07".into()),
            (format!("{VOTE_DIM}/standesstimmenJa"), "15.5".into()),
        ],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    assert!(undeclared.get("error").is_none(), "{undeclared}");
    assert_eq!(
        undeclared["undeclared_dimensions"],
        serde_json::json!([format!("{VOTE_DIM}/standesstimmenJa")]),
        "the answer NAMES what the shape did not declare (P9/P12)"
    );
    assert_eq!(undeclared["total"], 1, "the national row of 07.02.1971");
    // (c) absent: a request was made, so it is not-found — not
    // invalid-input.
    let absent = domain::observations(
        &ctx,
        VOTE,
        &[(format!("{VOTE_DIM}/gibtEsNicht"), "x".into())],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    assert_eq!(absent["error"], "not-found", "{absent}");
    assert_eq!(absent["subject"]["cube"], VOTE);
    assert_eq!(
        absent["subject"]["dimension"],
        format!("{VOTE_DIM}/gibtEsNicht")
    );
    // …and what IS decidable without a request stays invalid-input.
    let malformed = domain::observations(
        &ctx,
        VOTE,
        &[("not-an-iri".into(), "x".into())],
        &[],
        None,
        None,
        None,
    )
    .expect("runs");
    assert_eq!(malformed["error"], "invalid-input", "{malformed}");
    let unknown_cube = domain::observations(
        &ctx,
        "https://politics.ld.admin.ch/political-rights/popular-vote/2",
        &[],
        &[],
        None,
        None,
        None,
    )
    .expect("runs");
    assert_eq!(
        unknown_cube["error"], "not-found",
        "a cube outside the list (P1)"
    );
    let bad_lang =
        domain::observations(&ctx, VOTE, &[], &[], Some("es"), None, None).expect("runs");
    assert_eq!(bad_lang["error"], "invalid-input", "{bad_lang}");
}

// --- P13, P14, P15, P20: the rows themselves -------------------------

/// P20/C8.3: the Ständemehr is READ — 15.5 : 6.5 for 07.02.1971, where
/// counting the accepting cantons would give 17. P14: the decimals are
/// the lexical forms the store holds.
#[test]
fn the_staendemehr_is_read_and_never_derived() {
    let ctx = ctx();
    let out = domain::observations(
        &ctx,
        VOTE,
        &[
            (format!("{VOTE_DIM}/date"), "1971-02-07".into()),
            (format!("{VOTE_DIM}/region"), COUNTRY_CH.into()),
        ],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    let rows = out["observations"].as_array().expect("rows");
    assert_eq!(rows.len(), 1, "one vote × one region (P15/C8.1)");
    let cells = rows[0]["cells"].as_array().expect("cells");
    let value_of = |suffix: &str| -> Option<String> {
        cells
            .iter()
            .find(|c| c["dimension"].as_str().unwrap_or("").ends_with(suffix))
            .and_then(|c| c["value"].as_str().map(str::to_string))
    };
    assert_eq!(value_of("/standesstimmenJa").as_deref(), Some("15.5"));
    assert_eq!(value_of("/standesstimmenNein").as_deref(), Some("6.5"));
    assert_eq!(
        value_of("/kantoneJaGanzeStandesstimme").as_deref(),
        Some("14")
    );
    assert_eq!(
        value_of("/kantoneJaHalbeStandesstimme").as_deref(),
        Some("3")
    );
    assert_eq!(value_of("/staendeJaText").as_deref(), Some("14 3/2"));
    assert_eq!(value_of("/jaAnteil").as_deref(), Some("65.73"));
    // P15: the answer says which region it read.
    assert_eq!(out["region"]["value"], COUNTRY_CH);
    // P19/C8.2: the outcome is an IRI with a label, not a derived verdict.
    let outcome = cells
        .iter()
        .find(|c| c["dimension"].as_str().unwrap_or("").ends_with("/ergebnis"))
        .expect("the outcome cell");
    assert!(outcome["value"].as_str().unwrap().contains("/ergebnis/"));
    assert!(
        outcome["label"].as_str().unwrap().contains("angenommen"),
        "{outcome}"
    );
}

/// P13, C3.1 as corrected by §17.2: «not stated» has FOUR shapes and
/// the answer refuses to serve any of them as a value. The register's
/// silence must never arrive as `stated: true, value: ""` — that is a
/// blank rendered as a fact, and it made the honesty counters wrong by
/// exactly the cells that matter (audit of 01.09.2026).
#[test]
fn no_stated_cell_ever_carries_an_empty_value() {
    // Directly on the reader, over every shape the holding writes: the
    // two self-declaring cube:Undefined forms, the empty plain
    // literal (mmDate on the national vote rows), and the empty
    // literal typed plain xsd:string (pr:fax in the party register).
    let read = |node: serde_json::Value| {
        domain::cell(&serde_json::json!({ "v": node }), "v").expect("a cell")
    };
    let empty_plain = read(serde_json::json!({ "type": "literal", "value": "" }));
    assert!(!empty_plain.stated, "an empty plain literal is silence");
    assert_eq!(empty_plain.value, None);
    let empty_typed = read(serde_json::json!({
        "type": "literal", "value": "",
        "datatype": "http://www.w3.org/2001/XMLSchema#string"
    }));
    assert!(!empty_typed.stated, "an empty xsd:string is silence too");
    assert_eq!(
        empty_typed.datatype.as_deref(),
        Some("http://www.w3.org/2001/XMLSchema#string"),
        "WHICH shape of empty a cell used is itself a fact of the store"
    );
    // And a STATED lexical zero survives as a number — the three
    // states «0.0», «empty», «Undefined» must never collapse (§17.2).
    let zero = read(serde_json::json!({
        "type": "literal", "value": "0.0",
        "datatype": "http://www.w3.org/2001/XMLSchema#decimal"
    }));
    assert!(zero.stated, "a stated zero is a fact, not an absence");
    assert_eq!(zero.value.as_deref(), Some("0.0"));
}

/// P13/C3.1: «not stated» is answered for BOTH forms, and the answer
/// counts stated against not-stated.
#[test]
fn both_forms_of_not_stated_are_answered_as_such() {
    let ctx = ctx();
    let out = domain::observations(
        &ctx,
        VOTE,
        &[
            (format!("{VOTE_DIM}/date"), "1971-02-07".into()),
            (format!("{VOTE_DIM}/region"), COUNTRY_CH.into()),
        ],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    let cells = out["observations"][0]["cells"].as_array().expect("cells");
    let not_stated: Vec<&serde_json::Value> =
        cells.iter().filter(|c| c["stated"] == false).collect();
    assert!(!not_stated.is_empty(), "this row carries not-stated cells");
    assert!(
        not_stated.iter().all(|c| c.get("value").is_none()),
        "a not-stated cell carries no value — never 0, never «»"
    );
    let forms: std::collections::BTreeSet<&str> = not_stated
        .iter()
        .map(|c| c["form"].as_str().unwrap_or(""))
        .collect();
    assert!(forms.contains("iri"), "the IRI form: {forms:?}");
    assert!(
        forms.contains("literal"),
        "the empty typed literal: {forms:?}"
    );
    assert_eq!(
        out["not_stated_cells"].as_u64().expect("count"),
        not_stated.len() as u64
    );
    assert!(out["stated_cells"].as_u64().expect("count") > 0);
}

/// P16/C7.5: this endpoint can answer the same group key twice — the
/// theme census did, 12 + 9 for one identical pair. The server folds
/// before it serves, so a caller never sees a cube twice or a count
/// doubled. The case does not occur in the recorded answers, so the
/// test BUILDS it: the recorded answer with one binding repeated.
#[test]
fn an_aggregate_key_that_repeats_is_folded() {
    let dir = std::env::temp_dir().join(format!("oh-lindas-fold-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    // Every fixture the call needs, with the counts answer doctored.
    for key in ["list_cubes:all:de", "observation_counts:all"] {
        let name = oh_mcp_lindas::backend::fixture_file_name(key);
        let raw = std::fs::read_to_string(fixtures_dir().join(&name)).expect("a recorded answer");
        let mut value: serde_json::Value = serde_json::from_str(&raw).expect("json");
        if key == "observation_counts:all" {
            let bindings = value["results"]["bindings"]
                .as_array_mut()
                .expect("bindings");
            let first = bindings[0].clone();
            // the same key, a second time, with a smaller count — what
            // an aggregate over the default union may hand back
            let mut repeated = first.clone();
            repeated["observations"]["value"] = serde_json::json!("1");
            bindings.insert(1, repeated);
        }
        std::fs::write(
            dir.join(&name),
            serde_json::to_string(&value).expect("json"),
        )
        .expect("write");
    }
    let ctx = Ctx {
        backend: Backend::Fixtures { dir: dir.clone() },
        today: TODAY.into(),
    };
    let out = domain::list_cubes(&ctx, None, None, Some(50), None).expect("runs");
    let cubes = out["cubes"].as_array().expect("cubes");
    assert_eq!(cubes.len(), 44, "the repeated key does not add a cube");
    let iris: std::collections::BTreeSet<&str> =
        cubes.iter().filter_map(|c| c["cube"].as_str()).collect();
    assert_eq!(iris.len(), 44, "and it does not duplicate one either");
    let doubled = cubes
        .iter()
        .find(|c| c["cube"] == "https://politics.ld.admin.ch/fc/cube-chancellor")
        .expect("the first cube of the answer");
    assert!(
        doubled["observations"].as_u64().expect("count") > 1,
        "the count is folded to what the store really says, not to the repeat: {doubled}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// --- P26, P27: versions ---------------------------------------------

/// P26/P27: the versions come from the served list, and no answer says
/// «newer».
#[test]
fn list_versions_filters_the_list_and_promises_nothing() {
    let ctx = ctx();
    let out = domain::list_versions(&ctx, CANDIDATES_2019).expect("runs");
    assert_eq!(out["kind"], "norm", "{out}");
    assert_eq!(out["total"], 3, "2019, 2023, 2027");
    assert_eq!(out["versioned"], true);
    let versions = out["versions"].as_array().expect("versions");
    let placeholder = versions
        .iter()
        .find(|v| v["version"] == "2027")
        .expect("the 2027 cube");
    assert_eq!(placeholder["observations"], 0);
    assert_eq!(placeholder["placeholder"], true);
    assert!(out["note"].as_str().unwrap().contains("newer"));
    // The five unversioned cubes are their own family of one.
    let alone = domain::list_versions(&ctx, COUNCILLOR).expect("runs");
    assert_eq!(alone["total"], 1);
    assert_eq!(
        alone["versioned"], false,
        "C1.1: five cubes carry no version"
    );
}

// --- P17, P28, P29, P30: describe and labels -------------------------

/// P30/§8: a label of a FOREIGN host is asked of the one endpoint —
/// and a value the store has no label for answers `in_store: false`
/// rather than a fetch elsewhere.
#[test]
fn a_label_is_asked_of_the_one_endpoint_whatever_host_the_iri_has() {
    let ctx = ctx();
    let canton = domain::resolve_label(&ctx, CANTON_ZH, None).expect("runs");
    assert_eq!(canton["kind"], "hint", "{canton}");
    assert_eq!(canton["label"], "Zürich");
    assert_eq!(canton["label_lang"], "de");
    assert_eq!(canton["in_store"], true);
    assert!(canton["languages"].as_array().expect("languages").len() >= 4);
    // A concept of the i14y register — another host, the same endpoint.
    let sex = domain::resolve_label(&ctx, SEX_CONCEPT, None).expect("runs");
    assert!(
        sex.get("error").is_none(),
        "a foreign IRI is a subject: {sex}"
    );
    assert!(sex["in_store"].is_boolean());
    if sex["in_store"] == false {
        assert_eq!(
            sex["label"],
            serde_json::Value::Null,
            "an answer, not a fetch"
        );
    }
    let malformed = domain::resolve_label(&ctx, "canton/1", None).expect("runs");
    assert_eq!(malformed["error"], "invalid-input");
}

/// P17/§8: `describe` answers `via: "endpoint"` and caps its rows.
#[test]
fn describe_answers_from_the_one_endpoint_and_caps_its_rows() {
    let ctx = ctx();
    let out = domain::describe(&ctx, CANTON_ZH, None, Some(50), None).expect("runs");
    assert_eq!(out["kind"], "norm", "{out}");
    assert_eq!(out["via"], "endpoint", "there is no other branch (§8)");
    assert!(out["total"].as_u64().expect("total") > 0);
    assert_eq!(
        out["returned"].as_u64().expect("returned"),
        out["statements"].as_array().expect("statements").len() as u64
    );
    assert!(out["limit"].as_u64().expect("limit") <= 400);
    let unknown = domain::describe(
        &ctx,
        "https://politics.ld.admin.ch/nothing/at/all",
        None,
        None,
        None,
    )
    .expect("runs");
    assert_eq!(unknown["error"], "not-found", "{unknown}");
}

/// `describe` pages by STATEMENT, not by binding (BX').
///
/// **The defect this pins.** The page's `LIMIT`/`OFFSET` used to sit
/// on the outer pattern, where the label join multiplies a statement
/// into up to five rows. Measured on the recorded answer of page one
/// for `canton/1`: **50 bindings folding to 43 statements** — so
/// `returned` said 43 where `limit` said 50, and the next page began
/// fifty BINDINGS in, which is seven statements past where page one
/// ended. Those seven were served by no page at all.
///
/// The proof is arithmetic and needs no trust: page one plus page two
/// must BE the hundred-statement page, in order, statement for
/// statement.
#[test]
fn describe_pages_by_statement_so_no_statement_falls_between_two_pages() {
    let ctx = ctx();
    let page_one = domain::describe(&ctx, CANTON_ZH, None, Some(50), None).expect("runs");
    let page_two = domain::describe(&ctx, CANTON_ZH, None, Some(50), Some(50)).expect("runs");
    let both = domain::describe(&ctx, CANTON_ZH, None, Some(100), None).expect("runs");

    // returned counts STATEMENTS, and a full page is as long as it says.
    for (page, expected) in [(&page_one, 50), (&page_two, 50), (&both, 100)] {
        let statements = page["statements"].as_array().expect("statements");
        // `returned` is `statements.len()` by construction; what
        // proves it counts STATEMENTS and not bindings is that a full
        // page is exactly as long as the limit asked for — the old
        // query answered 43 here, because the label rows ate the
        // bound. (The fold's identity is pinned separately: a term
        // differing only in language tag or datatype is its own
        // statement, `the_page_bound_sits_inside_the_distinct_subselect`
        // pins the query, and `src/domain.rs`'s unit tests pin the fold.)
        assert_eq!(
            page["returned"].as_u64().expect("returned"),
            statements.len() as u64,
            "the count in the answer is the list in the answer"
        );
        assert_eq!(
            statements.len(),
            expected,
            "a page of {expected} is {expected} STATEMENTS long — the bindings behind it are more"
        );
        assert_eq!(page["total"], 228, "the count counts the same statements");
        assert_eq!(
            page["truncated"], true,
            "228 is more than any of these pages"
        );
    }

    // The identity of a statement is (predicate, value) — the pair the
    // subselect makes DISTINCT.
    let key = |row: &serde_json::Value| {
        format!(
            "{} => {}",
            row["dimension"].as_str().unwrap_or_default(),
            row["value"]
        )
    };
    let ones: Vec<String> = page_one["statements"]
        .as_array()
        .expect("statements")
        .iter()
        .map(key)
        .collect();
    let twos: Vec<String> = page_two["statements"]
        .as_array()
        .expect("statements")
        .iter()
        .map(key)
        .collect();
    let hundred: Vec<String> = both["statements"]
        .as_array()
        .expect("statements")
        .iter()
        .map(key)
        .collect();

    let unique: std::collections::BTreeSet<&String> = ones.iter().chain(twos.iter()).collect();
    assert_eq!(
        unique.len(),
        100,
        "no statement is served twice across the two pages"
    );
    let mut walked = ones.clone();
    walked.extend(twos.clone());
    assert_eq!(
        walked, hundred,
        "page one followed by page two IS the hundred-statement page, in order: offset 50 \
         continues exactly where page one ended and nothing falls between them"
    );

    // And the labels are still joined — the thing that made the old
    // paging wrong is still being done, just outside the bound.
    assert!(
        page_one["statements"]
            .as_array()
            .expect("statements")
            .iter()
            .any(|row| row["label"].is_string()),
        "a value the store labels still comes back labelled"
    );
}

/// A statement is a TERM, and two terms that differ only in their
/// language tag or their datatype are two statements (BX′).
///
/// The pages are made `DISTINCT` over `(?p ?v)`, which is identity by
/// RDF term; the Rust fold that turns bindings into rows used to key
/// on the predicate and the LEXICAL form alone, so `"Uri"@de` and
/// `"Uri"@fr` became one row. `returned` was then smaller than the page
/// the endpoint served, statements vanished from an answer whose
/// stage-one line promises «everything the holding says about one
/// IRI», and `truncated` promised another page after the last one.
///
/// The recorded corpus cannot show this — no `?v` in any recorded
/// describe page carries a language tag — so the case is BUILT here,
/// as a fixture directory of its own, in the way the crate's other
/// «what if the store said this» tests do.
#[test]
fn two_terms_that_differ_only_in_their_tag_are_two_statements() {
    use oh_mcp_lindas::backend::key_file;

    const IRI: &str = "https://ld.admin.ch/canton/26";
    let dir = std::env::temp_dir().join(format!("oh-lindas-terms-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a fixture directory of our own");

    let literal = |value: &str, tag: &str| serde_json::json!({"type": "literal", "value": value, "xml:lang": tag});
    let bindings = serde_json::json!([
        {"p": {"type": "uri", "value": "http://schema.org/name"}, "v": literal("Uri", "de")},
        {"p": {"type": "uri", "value": "http://schema.org/name"}, "v": literal("Uri", "fr")},
        {"p": {"type": "uri", "value": "http://schema.org/name"}, "v": literal("Uri", "it")},
        {"p": {"type": "uri", "value": "http://schema.org/name"}, "v": literal("Uri", "rm")},
        {"p": {"type": "uri", "value": "http://schema.org/alternateName"},
         "v": {"type": "literal", "value": "UR"}},
        {"p": {"type": "uri", "value": "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"},
         "v": {"type": "uri", "value": "https://schema.ld.admin.ch/Canton"}},
    ]);
    let write = |key: &str, value: serde_json::Value| {
        std::fs::write(
            key_file(&dir, key),
            serde_json::to_string_pretty(&value).expect("json"),
        )
        .expect("fixture written");
    };
    write(
        &format!("describe:count:{IRI}"),
        serde_json::json!({"results": {"bindings": [
            {"total": {"type": "literal", "value": "6"}}
        ]}}),
    );
    write(
        &format!("describe:{IRI}:100:0"),
        serde_json::json!({"results": {"bindings": bindings}}),
    );

    let ctx = Ctx {
        backend: Backend::Fixtures { dir: dir.clone() },
        today: TODAY.into(),
    };
    let out = domain::describe(&ctx, IRI, None, Some(100), None).expect("runs");
    let statements = out["statements"].as_array().expect("statements");

    assert_eq!(
        out["returned"], 6,
        "six terms were served and six statements are answered: {out}"
    );
    assert_eq!(statements.len(), 6);
    assert_eq!(out["total"], 6);
    assert_eq!(
        out["truncated"], false,
        "the page is complete, so nothing may promise another: {out}"
    );
    let tags: std::collections::BTreeSet<&str> = statements
        .iter()
        .filter(|row| row["value"] == "Uri")
        .filter_map(|row| row["lang"].as_str().or_else(|| row["label_lang"].as_str()))
        .collect();
    assert!(
        statements
            .iter()
            .filter(|row| row["value"] == "Uri")
            .count()
            == 4,
        "the four language versions of one name are four statements: {statements:?}"
    );
    let _ = tags;
    let _ = std::fs::remove_dir_all(&dir);
}

/// BY point 0: `dimensions` is a projection, and it is real.
///
/// §3.5 listed it as an input from the day the contract was written;
/// the crate took `cube, filters, lang, limit, offset` and no
/// projection, so every canton row came back with all 51 cells, the
/// chat's 24'000-byte cap cut the answer and the model paged the same
/// table thirteen times at `limit: 2` (the first live measurement,
/// §3.2 and the addendum). Three cells now, asked for by name.
#[test]
fn a_projection_asks_for_the_cells_the_question_needs() {
    let ctx = ctx();
    let filters = [
        (format!("{VOTE_DIM}/date"), "1971-02-07".to_string()),
        (format!("{VOTE_DIM}/region"), COUNTRY_CH.to_string()),
    ];
    let projection = [
        format!("{VOTE_DIM}/date"),
        format!("{VOTE_DIM}/region"),
        format!("{VOTE_DIM}/standesstimmenJa"),
    ];
    let out =
        domain::observations(&ctx, VOTE, &filters, &projection, None, Some(5), None).expect("runs");
    assert_eq!(out["kind"], "norm", "{out}");
    assert_eq!(out["returned"], 1, "the national row of that ballot");
    assert_eq!(out["cells_per_row"], 3, "three cells, not fifty-one");
    assert_eq!(
        out["dimensions"].as_array().map(Vec::len),
        Some(3),
        "the answer echoes the projection it served"
    );
    // P20: the Ständemehr is READ — and it is UNDECLARED, so the
    // projection had to pass P12's bound ASK to get it.
    let cells = out["observations"][0]["cells"].as_array().expect("cells");
    assert_eq!(cells.len(), 3);
    assert!(
        cells.iter().any(|c| c["dimension"]
            .as_str()
            .is_some_and(|d| d.ends_with("/standesstimmenJa"))
            && c["value"] == "15.5"),
        "the projected Ständemehr, read from the row: {cells:?}"
    );
    assert!(
        out["undeclared_dimensions"].as_array().is_some_and(|u| u
            .iter()
            .any(|d| d.as_str().is_some_and(|d| d.ends_with("/standesstimmenJa")))),
        "and the answer says the shape does not declare it (P9/P12): {out}"
    );
    assert_eq!(
        out["fewer_cells"],
        serde_json::Value::Null,
        "a caller that projected needs no advice about projecting"
    );
}

/// P12's three steps apply to a PROJECTED dimension exactly as they do
/// to a filtered one — and the cost of each step is measured, not
/// assumed.
#[test]
fn p12s_three_steps_apply_to_a_projection_too() {
    use std::sync::atomic::Ordering;
    let requests_of = |projection: &[String]| {
        let (backend, selects) = Backend::counting(fixtures_dir());
        let ctx = Ctx {
            backend,
            today: TODAY.into(),
        };
        let filters = [
            (format!("{VOTE_DIM}/date"), "1971-02-07".to_string()),
            (format!("{VOTE_DIM}/region"), COUNTRY_CH.to_string()),
        ];
        let out = domain::observations(&ctx, VOTE, &filters, projection, None, Some(5), None)
            .expect("runs");
        (out, selects.load(Ordering::SeqCst))
    };

    // (a) DECLARED: no request beyond the shape, the count and the page.
    let (declared, declared_cost) =
        requests_of(&[format!("{VOTE_DIM}/date"), format!("{VOTE_DIM}/region")]);
    assert_eq!(declared["kind"], "norm", "{declared}");
    assert_eq!(
        declared_cost, 3,
        "a declared projection asks nothing extra: shape, count, page"
    );
    assert_eq!(declared["cells_per_row"], 2);

    // (b) UNDECLARED: one bound ASK, and the answer says so.
    let (undeclared, undeclared_cost) = requests_of(&[
        format!("{VOTE_DIM}/date"),
        format!("{VOTE_DIM}/region"),
        format!("{VOTE_DIM}/standesstimmenJa"),
    ]);
    assert_eq!(
        undeclared_cost, 4,
        "one ASK for the one dimension the shape does not declare"
    );
    assert_eq!(
        undeclared["undeclared_dimensions"].as_array().map(Vec::len),
        Some(1)
    );

    // (c) ABSENT: the ASK says false, and the answer is not-found —
    // never an empty success, and never invalid-input, because a
    // request WAS made.
    let (absent, absent_cost) = requests_of(&[format!("{VOTE_DIM}/gibtEsNicht")]);
    assert_eq!(absent["error"], "not-found", "{absent}");
    assert_eq!(absent["subject"]["cube"], VOTE);
    assert!(absent["subject"]["dimension"]
        .as_str()
        .is_some_and(|d| d.ends_with("/gibtEsNicht")));
    assert_eq!(
        absent_cost, 2,
        "the shape and the false ASK — and then the refusal, BEFORE the count and the page \
         (BY′ point 5: the report's cost table said four, and no test held the number)"
    );

    // (d) ONE ASK PER DIMENSION, however often it is named (BY′ point
    // 3): filtering AND projecting the same undeclared dimension asked
    // the store the same question twice.
    //
    // The measurement is a COMPARISON, so it needs no new recording:
    // the same call with and without the projection must cost the
    // same, because the dimension is the same one. (The projected page
    // has no recorded answer — this case is about the counter, and the
    // counting double counts a request before it reads a fixture.)
    let cost_of = |projection: &[String]| {
        let (backend, selects) = Backend::counting(fixtures_dir());
        let ctx = Ctx {
            backend,
            today: TODAY.into(),
        };
        let out = domain::observations(
            &ctx,
            VOTE,
            &[
                (format!("{VOTE_DIM}/date"), "1971-02-07".to_string()),
                (format!("{VOTE_DIM}/standesstimmenJa"), "15.5".to_string()),
            ],
            projection,
            None,
            Some(5),
            None,
        )
        .expect("runs");
        (out, selects.load(Ordering::SeqCst))
    };
    let (filtered, filtered_cost) = cost_of(&[]);
    assert_eq!(filtered["kind"], "norm", "the recorded call: {filtered}");
    assert_eq!(
        filtered_cost, 4,
        "shape, the ASK for the undeclared Ständemehr, count, page"
    );
    let (_, both_cost) = cost_of(&[format!("{VOTE_DIM}/standesstimmenJa")]);
    assert_eq!(
        both_cost, filtered_cost,
        "naming the same dimension twice — once as a filter, once as a projection — must not \
         ask the store twice (BY′ point 3)"
    );
}

/// BY point 0: what is decidable without a request costs no request,
/// and the refusal says what WOULD have been right.
///
/// The measured failure: a model tried short names and quoted literals,
/// and every guess cost a refusal — and for a well-formed but unknown
/// IRI an `ASK` as well — because the refusal said only what was
/// wrong. A short name is no IRI, which is decidable here, before the
/// shape query, before any ASK, before anything.
#[test]
fn a_filter_or_a_projection_on_a_short_name_costs_no_request() {
    use std::sync::atomic::Ordering;
    for (what, filters, projection) in [
        (
            "a filter",
            vec![("datum".to_string(), "1971-02-07".to_string())],
            vec![],
        ),
        ("a projection", vec![], vec!["standesstimmenJa".to_string()]),
        (
            "a prefixed name",
            vec![("vote:date".to_string(), "1971-02-07".to_string())],
            vec![],
        ),
        // The third KIND the report names — a VALUE that begins like
        // an IRI and is none. It had no test until BY″: deleting the
        // branch left all 62 tests green while the addendum said «it
        // has one now».
        ("a language that is not one of the five", vec![], vec![]),
        (
            "a list over its cap",
            (0..30)
                .map(|n| (format!("{VOTE_DIM}/dimension{n}"), "x".to_string()))
                .collect::<Vec<_>>(),
            vec![],
        ),
        (
            "a value that begins like an IRI and is none",
            vec![(
                format!("{VOTE_DIM}/region"),
                "https://ld.admin.ch/canton /1".to_string(),
            )],
            vec![],
        ),
    ] {
        let (backend, selects) = Backend::counting(fixtures_dir());
        let ctx = Ctx {
            backend,
            today: TODAY.into(),
        };
        // The language case is the same class — decidable without a
        // request — and this is the loop that COUNTS requests, so the
        // cost table's «0» row is true of it too (BY‴).
        let lang = what.contains("language").then_some("es");
        let out = domain::observations(&ctx, VOTE, &filters, &projection, lang, Some(5), None)
            .expect("runs");
        assert_eq!(out["error"], "invalid-input", "{what}: {out}");
        assert_eq!(
            selects.load(Ordering::SeqCst),
            0,
            "{what}: not one request — not even the shape"
        );
        // Each site says what IT accepts (BY′): a filter is a pair, a
        // projection is a list of bare IRIs. A refusal that advertises
        // another input's shape is a refusal that teaches the wrong
        // lesson.
        let accepted = out["accepted"].as_str().unwrap_or_default();
        if what.contains("language") {
            assert_eq!(accepted, "de | fr | it | en | rm", "{what}: {accepted}");
        } else if what.contains("cap") {
            // A cap rejects a COUNT, and says only that (BY‴).
            assert!(
                accepted.starts_with("at most 24") && accepted.contains("a COUNT"),
                "{what}: {accepted}"
            );
        } else if what.contains("projection") {
            assert!(
                accepted.starts_with("[<full IRI") && accepted.contains("bare dimension IRIs"),
                "{what}: {accepted}"
            );
        } else {
            assert_eq!(
                accepted,
                "{dimension: <full IRI as describe_cube served it>, value: <IRI or plain literal>}",
                "{what}: the refusal states the shape it accepts, verbatim"
            );
        }
        assert!(
            out["note"].as_str().is_some_and(|n| n.contains("lindas.")),
            "{what}: and names the tool that would have served it: {out}"
        );
        assert!(
            out["note"]
                .as_str()
                .is_some_and(|n| n.contains("costs no request")),
            "{what}: and says that saying so cost nothing: {out}"
        );
    }
}

/// BY point 0: the answer says which projection would fit — and says
/// what that projection would cost the caller in truth.
///
/// The other option the measurement named — making the DECLARED
/// dimensions the default projection — is refused here on the
/// holding's own first rule: the shape is a subset of the record
/// (C2.2), and the Ständemehr is undeclared. A default projection
/// would drop it silently, which is worse than a wide answer.
#[test]
fn the_answer_says_which_projection_would_fit() {
    let ctx = ctx();
    let filters = [
        (format!("{VOTE_DIM}/date"), "1971-02-07".to_string()),
        (format!("{VOTE_DIM}/region"), COUNTRY_CH.to_string()),
    ];
    let out = domain::observations(&ctx, VOTE, &filters, &[], None, Some(5), None).expect("runs");
    assert_eq!(out["cells_per_row"], 51, "the row as the record carries it");
    let advice = &out["fewer_cells"];
    assert!(
        advice.is_object(),
        "an unprojected wide answer advises: {out}"
    );
    assert_eq!(advice["cells_per_row"], 51);
    assert_eq!(
        advice["declared_only"]["cells_per_row"], 14,
        "the shape declares fourteen of them (C2.2)"
    );
    let now = advice["rows_bytes"].as_u64().expect("rows_bytes");
    let projected = advice["declared_only"]["rows_bytes"]
        .as_u64()
        .expect("rows_bytes");
    assert!(
        projected * 2 < now,
        "the advice is measured on THIS answer: {now} bytes against {projected}"
    );
    assert!(
        advice["declared_only"]["warning"]
            .as_str()
            .is_some_and(|w| w.contains("Ständemehr")),
        "and it warns that the declared set drops the figure P20 is about: {advice}"
    );
    // What the advice costs on the widest row of this holding, measured
    // rather than remembered (BY′ point 7 cites the figure).
    let advice_bytes = serde_json::to_string(advice).expect("json").len();
    let answer_bytes = serde_json::to_string(&out).expect("json").len();
    println!(
        "advice on the 51-cell row: {advice_bytes} bytes of {answer_bytes} \
         ({}%)",
        advice_bytes * 100 / answer_bytes
    );
    assert!(
        advice_bytes < answer_bytes / 4,
        "the advice stays a small part of the answer it advises on"
    );
    // The contract states these two figures (§3.5) and the report
    // repeats them; nothing held them until BY″.
    assert_eq!(
        (advice_bytes, answer_bytes),
        (ADVICE_BYTES, ADVICE_OF_ANSWER_BYTES),
        "«1'721 bytes of a 13'754-byte answer» is what TOOLSET-v0.md §3.5 says — change it in \
         the same commit as the advice"
    );

    // And it does NOT fire where there is nothing to save: the
    // `vested-interest` rows carry seven cells and the shape declares
    // at least seven, so no advice is given — advice on an answer that
    // is already narrow is noise, and noise is bytes too.
    let narrow = domain::observations(&ctx, VESTED, &[], &[], None, Some(3), None).expect("runs");
    assert_eq!(narrow["cells_per_row"], 7);
    assert_eq!(
        narrow["fewer_cells"],
        serde_json::Value::Null,
        "nothing to save, nothing said: {narrow}"
    );

    // The advice sorts BEFORE the rows: a client that cuts the payload
    // at a byte count still reads what would have made the next call
    // small. (serde_json orders keys; this test is what makes that an
    // assurance rather than an accident.)
    let text = serde_json::to_string(&out).expect("serialises");
    assert!(
        text.find("\"fewer_cells\"").expect("present")
            < text.find("\"observations\"").expect("present"),
        "the advice must survive a truncated payload"
    );
}

/// BY′ point 1: a projection may take CELLS away, never ROWS.
///
/// The projection bound the predicate in front of the required
/// `?obs ?p ?v .`, so an observation carrying none of the named
/// predicates yielded no binding and vanished: `returned` counted
/// survivors while `total` came from the unprojected count, and no
/// field said so — the silent drop this server refuses the default
/// projection for, one level down.
///
/// The case is BUILT, because no recorded page has a row without the
/// projected cell: three observations, of which the middle one does
/// not carry the projected dimension.
#[test]
fn a_projection_takes_cells_away_never_rows() {
    use oh_mcp_lindas::backend::key_file;
    const CUBE: &str = "https://politics.ld.admin.ch/political-rights/popular-vote/1";
    const DIM: &str = "https://politics.ld.admin.ch/political-rights/popular-vote/standesstimmenJa";

    let dir = std::env::temp_dir().join(format!("oh-lindas-rows-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a fixture directory of our own");
    let write = |key: &str, value: serde_json::Value| {
        std::fs::write(
            key_file(&dir, key),
            serde_json::to_string_pretty(&value).expect("json"),
        )
        .expect("fixture written");
    };
    let obs = |n: u32| format!("{CUBE}/observations/CHE-{n}");
    // The shape declares nothing here, so the projected dimension goes
    // through P12's ASK — which answers true.
    write(
        &format!("describe_cube:shape:{CUBE}:de"),
        serde_json::json!({"results": {"bindings": []}}),
    );
    write(
        &format!("ask_dimension:{CUBE}:{DIM}"),
        serde_json::json!({"boolean": true}),
    );
    write(
        &format!("observations:count:{CUBE}:"),
        serde_json::json!({"results": {"bindings": [{"total": {"type": "literal", "value": "3"}}]}}),
    );
    // The page: three rows, and only the first and the third carry the
    // projected cell. The middle row arrives with ?p and ?v UNBOUND —
    // which is what an OPTIONAL gives for a row that has none.
    write(
        &format!("observations:{CUBE}::dims={DIM}:5:0"),
        serde_json::json!({"results": {"bindings": [
            {"obs": {"type": "uri", "value": obs(1)},
             "p": {"type": "uri", "value": DIM},
             "v": {"type": "literal", "value": "15.5"}},
            {"obs": {"type": "uri", "value": obs(2)}},
            {"obs": {"type": "uri", "value": obs(3)},
             "p": {"type": "uri", "value": DIM},
             "v": {"type": "literal", "value": "6.5"}},
        ]}}),
    );

    let ctx = Ctx {
        backend: Backend::Fixtures { dir: dir.clone() },
        today: TODAY.into(),
    };
    let out = domain::observations(&ctx, CUBE, &[], &[DIM.to_string()], None, Some(5), None)
        .expect("runs");
    let rows = out["observations"].as_array().expect("observations");
    assert_eq!(
        rows.len(),
        3,
        "all three rows of the page come back, not the two that carry the cell: {out}"
    );
    assert_eq!(out["returned"], 3);
    assert_eq!(
        out["total"], 3,
        "and returned counts what total counts — the two used to disagree"
    );
    assert_eq!(out["truncated"], false);
    // The row that does not carry it says so by carrying nothing —
    // not 0, not stated: false.
    let middle = &rows[1];
    assert_eq!(middle["observation"], obs(2));
    assert_eq!(
        middle["cells"].as_array().map(Vec::len),
        Some(0),
        "empty means «does not carry it»: {middle}"
    );
    assert_eq!(rows[0]["cells"].as_array().map(Vec::len), Some(1));
    assert_eq!(rows[2]["cells"][0]["value"], "6.5");
    let _ = std::fs::remove_dir_all(&dir);
}

/// BY′ point 2: a projection may not silently lose P15's region or
/// P35's citation shape — both are read FROM the cells.
#[test]
fn a_projection_says_whether_the_region_and_the_citation_shape_could_be_read() {
    let ctx = ctx();
    let filters = [
        (format!("{VOTE_DIM}/date"), "1971-02-07".to_string()),
        (format!("{VOTE_DIM}/region"), COUNTRY_CH.to_string()),
    ];

    // Unprojected: the region is READ, and the citation shape was
    // looked for over every cell of the row.
    let all = domain::observations(&ctx, VOTE, &filters, &[], None, Some(5), None).expect("runs");
    assert!(all["region"].is_object(), "the region is read: {all}");
    assert_eq!(all["region_state"], "read");
    assert_eq!(all["citation_shape_read_over"], "every cell of the row");

    // Projected WITHOUT the region dimension: the region is not
    // absent, it was not asked for — and the answer says which.
    let narrow = domain::observations(
        &ctx,
        VOTE,
        &filters,
        &[
            format!("{VOTE_DIM}/date"),
            format!("{VOTE_DIM}/region"),
            format!("{VOTE_DIM}/standesstimmenJa"),
        ],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    assert_eq!(
        narrow["region_state"], "read",
        "this projection kept the region dimension: {narrow}"
    );
    let without_region = domain::observations(
        &ctx,
        VOTE,
        &filters,
        &[format!("{VOTE_DIM}/date"), format!("{VOTE_DIM}/region")],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    assert_eq!(without_region["region_state"], "read");

    // A projection that leaves the region dimension OUT: the answer
    // must say «not projected» — the state this whole field exists for
    // (BY′: no test reached it, and the branch could be deleted with
    // the suite green). Built as its own fixture directory, because
    // every recorded projection keeps the region.
    {
        use oh_mcp_lindas::backend::key_file;
        let dir = std::env::temp_dir().join(format!("oh-lindas-region-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a fixture directory of our own");
        let write = |key: &str, value: serde_json::Value| {
            std::fs::write(
                key_file(&dir, key),
                serde_json::to_string_pretty(&value).expect("json"),
            )
            .expect("fixture written");
        };
        let date = format!("{VOTE_DIM}/date");
        let region = format!("{VOTE_DIM}/region");
        write(
            &format!("describe_cube:shape:{VOTE}:de"),
            serde_json::json!({"results": {"bindings": [
                {"dim": {"type": "uri", "value": date}},
                {"dim": {"type": "uri", "value": region}},
            ]}}),
        );
        write(
            &format!("observations:count:{VOTE}:"),
            serde_json::json!({"results": {"bindings": [
                {"total": {"type": "literal", "value": "1"}}
            ]}}),
        );
        write(
            &format!("observations:{VOTE}::dims={date}:5:0"),
            serde_json::json!({"results": {"bindings": [
                {"obs": {"type": "uri", "value": format!("{VOTE}/observations/CHE-1")},
                 "p": {"type": "uri", "value": date},
                 "v": {"type": "literal", "value": "1971-02-07"}},
            ]}}),
        );
        let ctx = Ctx {
            backend: Backend::Fixtures { dir: dir.clone() },
            today: TODAY.into(),
        };
        let out = domain::observations(
            &ctx,
            VOTE,
            &[],
            std::slice::from_ref(&date),
            None,
            Some(5),
            None,
        )
        .expect("runs");
        assert_eq!(out["region"], serde_json::Value::Null);
        assert!(
            out["region_state"]
                .as_str()
                .is_some_and(|s| s.starts_with("not projected")),
            "the region was not asked for, and the answer says so rather than «none»: {out}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // THE THIRD STATE, on a page built for it (BY‴): a cube whose rows
    // CARRY a region dimension the shape does not DECLARE — C2.2's
    // ordinary case — projected WITH that dimension, and the one row
    // of the page carrying no value for it. «the rows of this page
    // carry none» is what holds; «the shape declares none» would be a
    // sentence about the claim.
    {
        use oh_mcp_lindas::backend::key_file;
        let dir = std::env::temp_dir().join(format!("oh-lindas-carried-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a fixture directory of our own");
        let write = |key: &str, value: serde_json::Value| {
            std::fs::write(
                key_file(&dir, key),
                serde_json::to_string_pretty(&value).expect("json"),
            )
            .expect("fixture written");
        };
        let date = format!("{VOTE_DIM}/date");
        let region = format!("{VOTE_DIM}/region");
        // The shape declares the date and NOTHING else.
        write(
            &format!("describe_cube:shape:{VOTE}:de"),
            serde_json::json!({"results": {"bindings": [
                {"dim": {"type": "uri", "value": date}},
            ]}}),
        );
        // …but the observations carry the region: the bound ASK of P12
        // finds it, which is how an undeclared dimension is admitted.
        write(
            &format!("ask_dimension:{VOTE}:{region}"),
            serde_json::json!({"boolean": true}),
        );
        write(
            &format!("observations:count:{VOTE}:"),
            serde_json::json!({"results": {"bindings": [
                {"total": {"type": "literal", "value": "1"}}
            ]}}),
        );
        // The page: the row carries the date, and no region VALUE.
        write(
            &format!("observations:{VOTE}::dims={date},{region}:5:0"),
            serde_json::json!({"results": {"bindings": [
                {"obs": {"type": "uri", "value": format!("{VOTE}/observations/CHE-1")},
                 "p": {"type": "uri", "value": date},
                 "v": {"type": "literal", "value": "1971-02-07"}},
            ]}}),
        );
        let ctx = Ctx {
            backend: Backend::Fixtures { dir: dir.clone() },
            today: TODAY.into(),
        };
        let out = domain::observations(
            &ctx,
            VOTE,
            &[],
            &[date.clone(), region.clone()],
            None,
            Some(5),
            None,
        )
        .expect("runs");
        assert_eq!(
            out["undeclared_dimensions"].as_array().map(Vec::len),
            Some(1),
            "the region is carried and undeclared — the ASK admitted it: {out}"
        );
        assert_eq!(
            out["region_state"], "the rows of this page carry none",
            "asked for, admitted, and no value on this page: {out}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // THE FOURTH STATE, on recorded data: nothing was projected, so
    // every cell of every row came back — and none is a region, and
    // the shape declares none either. That is what the answer says,
    // and it says it about the ROWS first (BY‴: the sentence used to
    // read «neither declared nor asked», which is not what an
    // unprojected page knows).
    let no_region =
        domain::observations(&ctx, VESTED, &[], &[], None, Some(3), None).expect("runs");
    assert_eq!(no_region["region"], serde_json::Value::Null);
    assert_eq!(
        no_region["region_state"],
        "no row of this page carries a region cell, and the shape declares no region dimension",
        "{no_region}"
    );

    // And the citation shape says what it was read over, so a null is
    // «not seen in what you asked for», not «not there».
    assert!(
        narrow["citation_shape_read_over"]
            .as_str()
            .is_some_and(|s| s.contains("projected cells only")),
        "{narrow}"
    );
}

/// BY′ point 6: every place a guessing model first lands says what it
/// accepts — and costs no request to say it.
#[test]
fn the_refusals_a_guessing_model_meets_all_state_what_they_accept() {
    use std::sync::atomic::Ordering;
    const ACCEPTED_DIMENSION: &str =
        "{dimension: <full IRI as describe_cube served it>, value: <IRI or plain literal>}";

    // (a) a filter string with no «=» — the MCP boundary, which is
    // where the guessing starts. The refusal is what `split_filters`
    // ITSELF returns, so this test cannot pass by building one
    // (BY′: it used to build the refusal it then asserted on).
    let refusal = oh_mcp_lindas::server::split_filters(&["datum 1971-02-07".to_string()])
        .expect_err("no «=» is no filter");
    assert_eq!(refusal["error"], "invalid-input");
    assert_eq!(refusal["accepted"], ACCEPTED_DIMENSION);
    assert!(
        refusal["detail"]
            .as_str()
            .is_some_and(|d| d.contains("«datum 1971-02-07»")),
        "and it quotes what was given: {refusal}"
    );

    // (b) dimension_values with a short name, and (c) any tool with a
    // cube that is no IRI: both refuse before a request.
    for (what, call) in [
        (
            "dimension_values",
            Box::new(|ctx: &Ctx| domain::dimension_values(ctx, VOTE, "typologie", None, None))
                as Box<dyn Fn(&Ctx) -> anyhow::Result<serde_json::Value>>,
        ),
        (
            "a cube that is no IRI",
            Box::new(|ctx: &Ctx| domain::describe_cube(ctx, "popular-vote", None, None, None)),
        ),
    ] {
        let (backend, selects) = Backend::counting(fixtures_dir());
        let ctx = Ctx {
            backend,
            today: TODAY.into(),
        };
        let out = call(&ctx).expect("runs");
        assert_eq!(out["error"], "invalid-input", "{what}: {out}");
        assert_eq!(
            selects.load(Ordering::SeqCst),
            0,
            "{what}: not one request — a name that is no IRI is decidable here"
        );
        assert!(
            out["accepted"].is_string(),
            "{what}: the refusal states what it accepts: {out}"
        );
        assert!(
            out["note"].as_str().is_some_and(|n| n.contains("lindas.")),
            "{what}: and names the tool that would have served it: {out}"
        );
    }

    // The cube refusal names the tools that serve a cube IRI.
    let (backend, _) = Backend::counting(fixtures_dir());
    let ctx = Ctx {
        backend,
        today: TODAY.into(),
    };
    let cube = domain::describe_cube(&ctx, "popular-vote", None, None, None).expect("runs");
    assert!(
        cube["note"]
            .as_str()
            .is_some_and(|n| n.contains("lindas.list_cubes") && n.contains("lindas.find_cube")),
        "{cube}"
    );
}

/// BY′ point 7: the advice is worth its bytes ONCE.
///
/// A caller that is already paging has had it; ~1.7 kB on every page
/// bills the remedy in the currency of the problem. The first page
/// carries the whole advice, a later page a pointer to it — and the
/// bytes are MEASURED here, on both.
///
/// Built as its own fixture directory, because the recorded corpus has
/// no wide cube whose second page was recorded.
#[test]
fn the_advice_is_charged_on_the_first_page_and_pointed_at_afterwards() {
    use oh_mcp_lindas::backend::key_file;
    const CUBE: &str = "https://politics.ld.admin.ch/political-rights/popular-vote/1";
    const DIM: &str = "https://politics.ld.admin.ch/political-rights/popular-vote";

    let dir = std::env::temp_dir().join(format!("oh-lindas-advice-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a fixture directory of our own");
    let write = |key: &str, value: serde_json::Value| {
        std::fs::write(
            key_file(&dir, key),
            serde_json::to_string_pretty(&value).expect("json"),
        )
        .expect("fixture written");
    };
    // A shape that declares two dimensions, and rows that carry six:
    // the record is wider than the claim (C2.2), which is when the
    // advice has something to say.
    write(
        &format!("describe_cube:shape:{CUBE}:de"),
        serde_json::json!({"results": {"bindings": [
            {"dim": {"type": "uri", "value": format!("{DIM}/date")}},
            {"dim": {"type": "uri", "value": format!("{DIM}/region")}},
        ]}}),
    );
    write(
        &format!("observations:count:{CUBE}:"),
        serde_json::json!({"results": {"bindings": [{"total": {"type": "literal", "value": "20"}}]}}),
    );
    let page = |first: u32| {
        let mut bindings = Vec::new();
        for row in first..first + 5 {
            for cell in 0..6 {
                bindings.push(serde_json::json!({
                    "obs": {"type": "uri", "value": format!("{CUBE}/observations/CHE-{row}")},
                    "p": {"type": "uri", "value": format!("{DIM}/dimension{cell}")},
                    "v": {"type": "literal", "value": format!("value {row}-{cell}")},
                }));
            }
        }
        serde_json::json!({"results": {"bindings": bindings}})
    };
    write(&format!("observations:{CUBE}::5:0"), page(0));
    write(&format!("observations:{CUBE}::5:5"), page(5));

    let ctx = Ctx {
        backend: Backend::Fixtures { dir: dir.clone() },
        today: TODAY.into(),
    };
    let first = domain::observations(&ctx, CUBE, &[], &[], None, Some(5), None).expect("runs");
    let later = domain::observations(&ctx, CUBE, &[], &[], None, Some(5), Some(5)).expect("runs");

    assert_eq!(first["cells_per_row"], 6);
    let full = &first["fewer_cells"];
    let pointer = &later["fewer_cells"];
    assert!(full.is_object(), "the first page carries the whole advice");
    assert!(
        pointer.is_object(),
        "a later page still says the knob exists"
    );
    assert!(
        full["declared_only"]["dimensions"].is_array(),
        "with the IRIs to copy: {full}"
    );
    assert!(
        pointer.get("declared_only").is_none(),
        "and the later page does not repeat them: {pointer}"
    );
    assert!(
        pointer["how"]
            .as_str()
            .is_some_and(|h| h.contains("offset 0")),
        "it points at where the whole advice is: {pointer}"
    );
    // …and it does not turn the bytes it saves into a REQUEST (BY″): a
    // caller that OPENS at an offset never saw the first page, so the
    // pointer names where the IRIs come from and how many there are.
    assert_eq!(
        pointer["declared_dimensions"], 2,
        "the pointer says how many the declared set has: {pointer}"
    );
    assert!(
        pointer["how"]
            .as_str()
            .is_some_and(|h| h.contains("lindas.describe_cube")),
        "and where to get them without another page of this table: {pointer}"
    );

    let full_bytes = serde_json::to_string(full).expect("json").len();
    let pointer_bytes = serde_json::to_string(pointer).expect("json").len();
    println!("advice: {full_bytes} bytes in full, {pointer_bytes} as a pointer");
    // The pointer's size is stated in the contract too (BY″: on the
    // built page, which is what «measured on a page built for the
    // purpose» means there).
    assert_eq!(
        pointer_bytes, POINTER_BYTES,
        "«a pointer of {POINTER_BYTES} bytes» is what TOOLSET-v0.md §3.5 says"
    );
    assert!(
        pointer_bytes * 2 < full_bytes,
        "the pointer is a fraction of the advice: {pointer_bytes} against {full_bytes}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// BY′ point 10: `find_cube` finds a cube by TWO words of its name.
///
/// The entrance of two-stage discovery carried the same contiguous
/// `CONTAINS` that `search_law` shed at BY point 0, and it fails the
/// same way: «Kennzahlen zu Volksabstimmungen» has a word between the
/// two a caller would type. Measured live before and after, one
/// request each: **0 hits with the contiguous filter, 1 with the
/// word-wise one** — the cube the question is about.
#[test]
fn find_cube_finds_a_cube_by_two_words_of_its_name() {
    let ctx = ctx();
    let out = domain::find_cube(&ctx, "Kennzahlen Volksabstimmungen", None, None).expect("runs");
    assert_eq!(out["kind"], "hint", "a search answers a hint: {out}");
    assert_eq!(out["returned"], 1, "{out}");
    assert_eq!(
        out["hits"][0]["cube"], "https://politics.ld.admin.ch/political-rights/popular-vote-stat/1",
        "«Kennzahlen zu Volksabstimmungen» — the word between them is «zu»: {out}"
    );
}

/// All THREE caps of this crate, each held by a test that counts
/// requests — and each refusal saying what it rejects: a COUNT, and
/// nothing else.
///
/// The history is why they are in one test. BY′ capped the projection
/// and left the filter list open; BY″ capped the filters and left
/// `MAX_QUERY_WORDS` unheld (set to a million, all 62 tests stayed
/// green); BY‴ found that the refusal claimed «every one of the names
/// given was well formed» on a path that had not looked at the names
/// yet. A cap is a class, and a class is tested once, together.
#[test]
fn every_cap_is_held_and_each_refusal_names_only_the_count() {
    use std::sync::atomic::Ordering;
    let names: Vec<String> = (0..30)
        .map(|n| format!("{VOTE_DIM}/dimension{n}"))
        .collect();
    let filters: Vec<(String, String)> =
        names.iter().map(|d| (d.clone(), "x".to_string())).collect();
    // A SHORT NAME among the thirty: the count is decided before any
    // name is looked at, so the refusal must be the cap's — and must
    // not claim the names were well formed (BY‴).
    let mut mixed = filters.clone();
    mixed[7] = ("datum".to_string(), "1971-02-07".to_string());

    let observations = |filters: &[(String, String)], projection: &[String]| {
        let (backend, selects) = Backend::counting(fixtures_dir());
        let ctx = Ctx {
            backend,
            today: TODAY.into(),
        };
        let out = domain::observations(&ctx, VOTE, filters, projection, None, Some(5), None)
            .expect("runs");
        (out, selects.load(Ordering::SeqCst))
    };
    // Thirteen words inside the 100-character length limit, so it is
    // the WORD cap that answers and not the length check.
    let long_query = (1..=13)
        .map(|n| format!("w{n:02}"))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(long_query.chars().count() <= 100);
    let find = || {
        let (backend, selects) = Backend::counting(fixtures_dir());
        let ctx = Ctx {
            backend,
            today: TODAY.into(),
        };
        let out = domain::find_cube(&ctx, &long_query, None, None).expect("runs");
        (out, selects.load(Ordering::SeqCst))
    };

    for (what, cap, (out, requests)) in [
        ("dimensions", 24, observations(&[], &names)),
        ("filters", 24, observations(&filters, &[])),
        ("filters", 24, observations(&mixed, &[])),
        ("words", 12, find()),
    ] {
        assert_eq!(out["error"], "invalid-input", "{what}: {out}");
        assert_eq!(
            requests, 0,
            "{what}: not one request — a count is decidable before the store is asked anything"
        );
        assert!(
            out["detail"]
                .as_str()
                .is_some_and(|d| d.contains(what) && d.contains(&format!("at most {cap}"))),
            "{what}: the refusal names how many were given and how many may be: {out}"
        );
        // A cap rejects a NUMBER — and reports no check it did not make.
        let accepted = out["accepted"].as_str().unwrap_or_default();
        assert!(
            accepted.starts_with(&format!("at most {cap}")) && accepted.contains("a COUNT"),
            "{what}: a cap accepts a count: {accepted}"
        );
        assert!(
            !accepted.contains("well formed"),
            "{what}: and claims no check it did not make: {accepted}"
        );
    }
}

/// THE CLAMPS, held by their ceilings (BY‴, the class sweep).
///
/// Six of this crate's caps do not refuse — they CLAMP a caller's
/// `limit` (`capped`), and the contract states each ceiling («int ≤
/// 200, default 50»). No test named one: raising `MAX_OBSERVATIONS` to
/// a million would have left the suite green and the contract's
/// sentence false. A ceiling is a promise, and a promise is a test.
///
/// Where the fixture KEY carries the limit, the ceiling is read off
/// the key the call asked for — which is the same thing the endpoint
/// would have been asked.
#[test]
fn every_clamp_answers_at_its_ceiling_and_says_so() {
    let ctx = ctx();
    let over = 100_000;

    // Keys without a limit: the answer says what it served.
    let cubes = domain::list_cubes(&ctx, None, None, Some(over), None).expect("runs");
    assert_eq!(cubes["limit"], 100, "MAX_CUBES: §3.1 says int ≤ 100");
    let shape = domain::describe_cube(&ctx, VOTE, None, Some(over), None).expect("runs");
    assert_eq!(shape["limit"], 100, "MAX_DIMENSIONS: §3.3 says int ≤ 100");
    let values = domain::dimension_values(
        &ctx,
        LIST_RESULTS_2023,
        "https://politics.ld.admin.ch/national-council-election/list-results/listName",
        None,
        Some(over),
    )
    .expect("runs");
    assert_eq!(values["limit"], 200, "MAX_VALUES: §3.4 says int ≤ 200");

    // Keys that carry the limit: the key itself is the evidence.
    let asked_for = |call: &dyn Fn(&Ctx)| {
        let (backend, _) = Backend::counting(fixtures_dir());
        let ctx = Ctx {
            backend,
            today: TODAY.into(),
        };
        call(&ctx);
        ctx.backend.seen_keys()
    };
    let hits = asked_for(&|ctx| {
        let _ = domain::find_cube(ctx, "Abstimmung", None, Some(over));
    });
    assert!(
        hits.iter().any(|k| k.ends_with(":de:50")),
        "MAX_HITS: §3.2 says int ≤ 50, and the key says so: {hits:?}"
    );
    // Filters whose COUNT is recorded, so the call reaches the page
    // key — which is the one that carries the limit.
    let rows = asked_for(&|ctx| {
        let _ = domain::observations(
            ctx,
            VOTE,
            &[
                (format!("{VOTE_DIM}/date"), "1971-02-07".to_string()),
                (format!("{VOTE_DIM}/region"), COUNTRY_CH.to_string()),
            ],
            &[],
            None,
            Some(over),
            None,
        );
    });
    assert!(
        rows.iter().any(|k| k.ends_with(":200:0")),
        "MAX_OBSERVATIONS: §3.5 says int ≤ 200: {rows:?}"
    );
    let statements = asked_for(&|ctx| {
        let _ = domain::describe(ctx, CANTON_ZH, None, Some(over), None);
    });
    assert!(
        statements.iter().any(|k| k.ends_with(":400:0")),
        "MAX_STATEMENTS: §3.7 says int ≤ 400: {statements:?}"
    );
}

// --- §7: the acceptance families -------------------------------------

/// §7 rows 2 and 3: a Vorlage year and the states cubes — future dates
/// are data (P25), and the `stand` vocabulary is read per cube (P22).
#[test]
fn the_referendum_and_initiative_families_answer_from_their_cubes() {
    let ctx = ctx();
    let bills = domain::observations(
        &ctx,
        REFERENDUM,
        &[(
            "https://politics.ld.admin.ch/political-rights/referendum/beschlussdatumJahr".into(),
            "2023".into(),
        )],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    assert!(bills.get("error").is_none(), "{bills}");
    assert!(bills["total"].as_u64().expect("total") > 0);
    // P11/P22: the «Stand» vocabulary is READ per cube — and where the
    // shape closes a dimension with sh:in, the values come from that
    // list instead of a scan of the observations.
    let stand = domain::dimension_values(
        &ctx,
        INITIATIVE_STAT,
        "https://politics.ld.admin.ch/political-rights/popular-initiative-stat/stand",
        None,
        None,
    )
    .expect("runs");
    assert_eq!(stand["kind"], "hint");
    assert_eq!(stand["source"], "enumeration", "P11: {stand}");
    let values = stand["values"].as_array().expect("values");
    assert!(
        values.len() >= 20,
        "the Stand vocabulary is read, not hard-coded"
    );
    assert!(
        values
            .iter()
            .any(|v| v["label"].as_str().unwrap_or("").contains("Sammelbeginn")),
        "a state answer carries the IRI AND its label (C9.2): {values:?}"
    );
    // P25: a date after today is data — the states cube reaches 2029.
    let states =
        domain::observations(&ctx, REFERENDUM_STAT, &[], &[], None, Some(5), None).expect("runs");
    assert!(states["total"].as_u64().expect("total") > 1000);
    let dates: Vec<&str> = states["observations"]
        .as_array()
        .expect("rows")
        .iter()
        .flat_map(|o| o["cells"].as_array().expect("cells"))
        .filter(|c| c["dimension"].as_str().unwrap_or("").ends_with("/datum"))
        .filter_map(|c| c["value"].as_str())
        .collect();
    assert!(!dates.is_empty(), "the states carry their dates");
    // P21: a ballot is filtered by the typology IRI, never by a word
    // in a title — and the typology is a vocabulary the tool reads.
    let typology =
        domain::dimension_values(&ctx, VOTE, &format!("{VOTE_DIM}/typologie"), None, None)
            .expect("runs");
    let types = typology["values"].as_array().expect("values");
    assert!(
        types
            .iter()
            .all(|v| v["value"].as_str().unwrap_or("").starts_with("https://")),
        "the filter values are IRIs: {types:?}"
    );
    // The contract's example for this tool, run: seven typologies, and
    // the shape's `sh:in` is what answers (§3.4, «Built»).
    assert_eq!(types.len(), 7, "seven typologies: {types:?}");
    assert_eq!(typology["source"], "enumeration", "{typology}");
    assert!(
        types.iter().any(|v| v["label"]
            .as_str()
            .unwrap_or("")
            .contains("Volksinitiative")),
        "{types:?}"
    );
}

/// §7 rows 4 and 5: the Bundesrat by canton, and the register of
/// interest ties — both with labels from hosts outside the scope.
#[test]
fn the_councillor_and_interest_families_answer_with_foreign_labels() {
    let ctx = ctx();
    let councillors = domain::observations(
        &ctx,
        COUNCILLOR,
        &[(
            "http://schema.org/addressRegion".into(),
            "https://ld.admin.ch/canton/2".into(),
        )],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    assert!(councillors.get("error").is_none(), "{councillors}");
    assert!(
        councillors["total"].as_u64().expect("total") > 0,
        "Bern has had councillors"
    );
    let ties = domain::observations(&ctx, VESTED, &[], &[], None, Some(3), None).expect("runs");
    assert!(
        ties["total"].as_u64().expect("total") > 4000,
        "C0.2: 4'954 rows"
    );
    let cells = ties["observations"][0]["cells"].as_array().expect("cells");
    assert!(
        cells.iter().any(|c| c["dimension"]
            .as_str()
            .unwrap_or("")
            .ends_with("/hasPerson")),
        "an interest tie names its person"
    );
}

/// §7 row 6a/6b: the seats are READ from `list-results`, and «who was
/// elected» is one call on `candidates` — no join anywhere (P23).
#[test]
fn the_election_family_answers_without_a_join() {
    let ctx = ctx();
    let seats = domain::observations(
        &ctx,
        LIST_RESULTS_2023,
        &[(
            "https://politics.ld.admin.ch/national-council-election/list-results/hasCanton".into(),
            CANTON_ZH.into(),
        )],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    assert!(seats.get("error").is_none(), "{seats}");
    let cells = seats["observations"][0]["cells"].as_array().expect("cells");
    let has = |suffix: &str| {
        cells
            .iter()
            .any(|c| c["dimension"].as_str().unwrap_or("").ends_with(suffix))
    };
    assert!(
        has("/seats"),
        "the seats are IN the row — nothing is counted"
    );
    assert!(has("/listName"));
    assert!(has("/listNr"));
    let elected = domain::observations(
        &ctx,
        CANDIDATES_2023,
        &[
            (
                "https://politics.ld.admin.ch/national-council-election/candidates/hasCanton"
                    .into(),
                CANTON_ZH.into(),
            ),
            (
                "https://politics.ld.admin.ch/national-council-election/candidates/elected".into(),
                "true".into(),
            ),
        ],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    assert!(elected["total"].as_u64().expect("total") > 0, "{elected}");
}

// --- §5: the catalogue ----------------------------------------------

/// The stage-one lines are the contract's §5, and they obey the house
/// Every stage-one line follows the house rule — and IS the line the
/// contract promises.
///
/// The lines have THREE copies, and this is where two of
/// them are held equal: the router's (what `tools/list` serves) and
/// §5 of `TOOLSET-v0.md` (what the contract promises). The third — the
/// gateway's `meta.tools` — is held against the router by
/// `mcp/gateway/tests/inventory.rs`, so the chain contract ↔ router ↔
/// gateway is closed and no copy can drift alone.
///
/// The lines were hard-coded here until BX'; a fourth copy inside the
/// gate is a copy the gate cannot catch.
#[test]
fn every_stage_one_line_follows_the_house_rule_and_is_the_contracts() {
    let router: std::collections::BTreeMap<String, String> = server::LindasServer::tool_router()
        .list_all()
        .into_iter()
        .map(|tool| {
            (
                tool.name.to_string(),
                tool.description
                    .as_deref()
                    .unwrap_or_default()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        })
        .collect();
    assert_eq!(router.len(), 8, "eight tools (contract §3)");

    // §5's table: `| `lindas.x` | 2 | <line> |`
    let contract = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("TOOLSET-v0.md"),
    )
    .expect("the contract is beside the crate");
    let promised: std::collections::BTreeMap<String, String> = contract
        .lines()
        .filter(|line| line.starts_with("| `lindas."))
        .filter_map(|line| {
            // A cell may carry an escaped pipe (`a\|b`), which is no
            // column border — the contract gate's parser handles it and
            // this one must too, or a legitimate line would silently
            // leave the table and take its check with it.
            let guarded = line.replace("\\|", "\u{1}");
            let cells: Vec<String> = guarded
                .trim_matches('|')
                .split('|')
                .map(|cell| cell.trim().replace('\u{1}', "|"))
                .collect();
            (cells.len() == 3).then(|| {
                (
                    cells[0].trim_matches('`').to_string(),
                    cells[2].split_whitespace().collect::<Vec<_>>().join(" "),
                )
            })
        })
        .collect();
    assert_eq!(
        promised.len(),
        8,
        "§5 of the contract lists the eight lines; found {}",
        promised.len()
    );
    assert_eq!(
        router, promised,
        "the router's stage-one lines and §5 of TOOLSET-v0.md have parted company — they are the \
         same sentence, and the contract is where it is written down"
    );

    for (id, summary) in &router {
        server::summary_conforms(summary).unwrap_or_else(|e| panic!("{id}: {e}"));
        assert!(id.starts_with("lindas."), "one domain, one prefix");
    }
    // The German triggers are IN the lines: the inventory has no other
    // field for them (contract §5).
    let all: String = router.values().cloned().collect::<Vec<_>>().join(" ");
    for trigger in [
        "Abstimmung",
        "Kanton",
        "Ständemehr",
        "Referendum",
        "Volksinitiative",
        "Bundesrat",
        "Nationalratswahl",
        "Interessenbindung",
    ] {
        assert!(
            all.contains(trigger),
            "the trigger «{trigger}» is in no line"
        );
    }
}

/// P1: the served scope of the crate IS the list in `cubes.txt` — the
/// contract's §9, byte for byte.
#[test]
fn the_served_scope_is_the_list_the_contract_names() {
    let listed = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../testing/lindas-probe/cubes.txt"),
    )
    .expect("cubes.txt");
    let from_file: Vec<String> = listed
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    let mut from_code = scope::all();
    let mut from_file_sorted = from_file.clone();
    from_code.sort();
    from_file_sorted.sort();
    assert_eq!(
        from_code, from_file_sorted,
        "the scope is the list, not a pattern"
    );
}

/// **A stray argument key is a refusal, never a silent drop.** The
/// audit of 01.09.2026 measured the open shape: `{"cube": …, "filter":
/// "hasCanton=…"}` — singular, a plausible near-miss of `filters` —
/// was accepted, the key dropped, and the caller handed the whole of
/// Switzerland while believing it had filtered one canton (18'366 rows
/// where 26 were asked for, no complaint anywhere in the payload). The
/// closed shape turns the same call into a typed error that names the
/// stray key and the accepted ones.
#[test]
fn a_near_miss_argument_name_is_refused_not_dropped() {
    let stray = serde_json::from_value::<server::ObservationsParams>(serde_json::json!({
        "cube": "https://politics.ld.admin.ch/political-rights/popular-vote/1",
        "filter": "hasCanton=https://ld.admin.ch/canton/19"
    }));
    let message = match stray {
        Err(e) => e.to_string(),
        Ok(_) => panic!("the stray key must not be dropped"),
    };
    assert!(
        message.contains("filter") && message.contains("filters"),
        "the error names the near-miss and the accepted field: {message}"
    );

    // And the eight shapes are all closed — a stray key on any of them
    // is an error, so the next tool added open would fail this list.
    let stray_key = serde_json::json!({ "no_such_key": true });
    assert!(serde_json::from_value::<server::ListCubesParams>(stray_key.clone()).is_err());
    assert!(serde_json::from_value::<server::FindCubeParams>(stray_key.clone()).is_err());
    assert!(serde_json::from_value::<server::CubeParams>(stray_key.clone()).is_err());
    assert!(serde_json::from_value::<server::DimensionValuesParams>(stray_key.clone()).is_err());
    assert!(serde_json::from_value::<server::ObservationsParams>(stray_key.clone()).is_err());
    assert!(serde_json::from_value::<server::VersionsParams>(stray_key.clone()).is_err());
    assert!(serde_json::from_value::<server::IriParams>(stray_key.clone()).is_err());
    assert!(serde_json::from_value::<server::LabelParams>(stray_key).is_err());
}

/// **A filter that matches nothing is not an empty cube.** The audit
/// of 01.09.2026 ranked this first on the whole tool surface: the
/// first cut computed `placeholder` from the FILTERED total, so one
/// wrong lexical form answered «this cube is unpublished» about 9'008
/// published rows — and P3 and the skill both told the reader to
/// believe it. The cube's own count decides now, and the miss is its
/// own named state.
#[test]
fn a_filter_that_matches_nothing_is_a_miss_and_never_a_placeholder() {
    let ctx = ctx();
    let out = domain::observations(
        &ctx,
        INITIATIVE_STAT,
        &[(
            "https://politics.ld.admin.ch/political-rights/popular-initiative-stat/stand".into(),
            "kein-solcher-stand".into(),
        )],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    assert_eq!(out["total"], 0, "the filter really matches nothing");
    assert_eq!(
        out["placeholder"], false,
        "the cube is full — a miss must not call it unpublished: {out}"
    );
    assert_eq!(out["filters_matched_nothing"], true);
    // …and the pairs actually applied are echoed, so a surface can
    // tell a wrongly-scoped table from a rightly-scoped one without
    // parsing SPARQL out of provenance.query.
    let echoed = out["filters"].as_array().expect("the filters are echoed");
    assert_eq!(echoed.len(), 1);
    assert!(
        echoed[0]
            .as_str()
            .unwrap_or_default()
            .ends_with("=kein-solcher-stand"),
        "{echoed:?}"
    );

    // The unfiltered page of the same cube: no filters, no miss.
    let full =
        domain::observations(&ctx, INITIATIVE_STAT, &[], &[], None, Some(5), None).expect("runs");
    assert_eq!(full["filters"], serde_json::Value::Null);
    assert_eq!(full["filters_matched_nothing"], false);
    assert_eq!(full["placeholder"], false);
}

/// **An untagged label is a label.** Candidate names are untagged
/// literals in this holding, and the five-language filter dropped
/// every one of them — an election row arrived as a bare IRI, and the
/// model read a «name» out of a slug that actually encodes canton and
/// list codes (audit of 01.09.2026, rank 3). The join admits the
/// untagged arm now, exactly as `resolve_label` always has, and
/// `choose_label` serves it as «und».
#[test]
fn an_untagged_candidate_name_reaches_the_row_as_its_label() {
    let ctx = ctx();
    let out = domain::observations(
        &ctx,
        CANDIDATES_2023,
        &[
            (
                "https://politics.ld.admin.ch/national-council-election/candidates/hasCanton"
                    .into(),
                CANTON_ZH.into(),
            ),
            (
                "https://politics.ld.admin.ch/national-council-election/candidates/elected".into(),
                "true".into(),
            ),
        ],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    let labelled: Vec<&serde_json::Value> = out["observations"]
        .as_array()
        .expect("rows")
        .iter()
        .flat_map(|row| row["cells"].as_array().expect("cells"))
        .filter(|cell| {
            cell["dimension"]
                .as_str()
                .unwrap_or_default()
                .ends_with("/hasCandidate")
        })
        .collect();
    assert!(!labelled.is_empty(), "the elected rows carry candidates");
    for cell in &labelled {
        assert!(
            cell["label"].as_str().is_some_and(|l| !l.is_empty()),
            "a candidate cell carries the person's name, not a bare IRI: {cell}"
        );
        assert_eq!(
            cell["label_lang"], "und",
            "and says honestly that the store tagged no language: {cell}"
        );
    }
}

/// **One region speaks for a page only when it is the page's only
/// one** — and the election family's canton counts as one at all. The
/// first cut read the FIRST cell whose dimension name ended in
/// `/region`: a mixed page was headed by whichever region came back
/// first, and every election page was answered «no row carries a
/// region» by name-blindness (audit rank 5; F6/T6/T7/T18).
#[test]
fn a_page_of_one_region_names_it_and_a_mixed_page_refuses_to_choose() {
    let ctx = ctx();
    // 6a: every row of this page is the same canton, read from a
    // dimension the old name test was blind to.
    let one = domain::observations(
        &ctx,
        LIST_RESULTS_2023,
        &[(
            "https://politics.ld.admin.ch/national-council-election/list-results/hasCanton".into(),
            CANTON_ZH.into(),
        )],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    assert_eq!(one["region"]["value"], CANTON_ZH, "{}", one["region"]);
    assert!(
        one["region"]["dimension"]
            .as_str()
            .is_some_and(|d| d.ends_with("/hasCanton")),
        "the claim names the cell it was read from: {}",
        one["region"]
    );
    assert_eq!(one["region_state"], "read");
    assert_eq!(one["regions_on_page"], 1);

    // The unfiltered states cube spans regions: no single one may head
    // the page, and the national row — the one right address for an
    // unqualified business fact (F6) — is named when present.
    let mixed =
        domain::observations(&ctx, INITIATIVE_STAT, &[], &[], None, Some(5), None).expect("runs");
    if mixed["regions_on_page"].as_u64().unwrap_or(0) > 1 {
        assert_eq!(
            mixed["region"],
            serde_json::Value::Null,
            "{}",
            mixed["region"]
        );
        assert!(
            mixed["region_state"]
                .as_str()
                .unwrap_or_default()
                .starts_with("mixed"),
            "{}",
            mixed["region_state"]
        );
    } else {
        // The recorded page happens to carry one region or none: then
        // the claim must be exactly that, never a guess about the
        // cube. (The states cube keys its rows by business id, not by
        // region, so «none» is its honest answer.)
        match mixed["regions_on_page"].as_u64() {
            Some(1) => assert_eq!(mixed["region_state"], "read"),
            Some(0) => assert_eq!(mixed["region"], serde_json::Value::Null),
            other => panic!("regions_on_page: {other:?}"),
        }
    }
}

/// **C15.1 — the lexical filter is immune to typed numerics, and that
/// is pinned rather than accidental.** The election numerics are
/// `xsd:int`; a SPARQL filter written as a bare number reads as
/// `xsd:integer` and matches nothing — an honest-looking empty answer
/// (§17.4 measured 0 rows against 34). The server never binds a typed
/// literal: every literal filter compares on `STR(…)`, which cannot
/// know the datatype and therefore cannot miss by it. Proven here on
/// two typed dimensions the fixtures carry — a decimal (the
/// Ständemehr) and a boolean (`elected`) — both filtered as plain
/// text, both answering rows.
#[test]
fn a_lexical_filter_is_immune_to_typed_numerics() {
    let ctx = ctx();
    let decimal = domain::observations(
        &ctx,
        VOTE,
        &[
            (format!("{VOTE_DIM}/date"), "1971-02-07".into()),
            (format!("{VOTE_DIM}/standesstimmenJa"), "15.5".into()),
        ],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    assert!(
        decimal["total"].as_u64().unwrap_or(0) > 0,
        "a decimal-typed cell answers a plain-text filter: {}",
        decimal["total"]
    );
    let boolean = domain::observations(
        &ctx,
        CANDIDATES_2023,
        &[
            (
                "https://politics.ld.admin.ch/national-council-election/candidates/hasCanton"
                    .into(),
                CANTON_ZH.into(),
            ),
            (
                "https://politics.ld.admin.ch/national-council-election/candidates/elected".into(),
                "true".into(),
            ),
        ],
        &[],
        None,
        Some(5),
        None,
    )
    .expect("runs");
    assert!(
        boolean["total"].as_u64().unwrap_or(0) > 0,
        "a boolean-typed cell answers a plain-text filter: {}",
        boolean["total"]
    );
    // And the mechanism is visible in the query the answer carries:
    // the comparison is lexical, never a typed literal in the pattern.
    let query = boolean["provenance"]["query"].as_str().unwrap_or_default();
    assert!(
        query.contains("FILTER(STR("),
        "the filter compares on the lexical form: {query}"
    );
    assert!(
        !query.contains("^^"),
        "and no typed literal stands in the pattern: {query}"
    );
}
