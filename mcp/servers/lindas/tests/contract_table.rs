//! The contract gate: every point of `TOOLSET-v0.md` §1 is pinned by a
//! test this crate RUNS, or its row says «deferred» and why.
//!
//! The LINDAS counterpart of the fedlex crate's `rules_table.rs`, and
//! the fourth link of the chain: rule → §16 → contract → **test**. A
//! contract that claims what the suite does not run is a wish list;
//! this file is what keeps it from becoming one.
//!
//! Three of the points are properties of the SOURCE rather than of an
//! answer — nothing is hard-coded (P4), no tool joins (P23), no query
//! is unbound (P31) — so they are pinned here, over the crate's own
//! text, in the way the fedlex gate pins its own refusals.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn crate_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn contract() -> String {
    std::fs::read_to_string(crate_dir().join("TOOLSET-v0.md")).expect("TOOLSET-v0.md")
}

fn source(file: &str) -> String {
    std::fs::read_to_string(crate_dir().join(file)).unwrap_or_default()
}

/// A source file WITHOUT its test module: the three source-level
/// points are about what the server does, and a test that names a real
/// cube is not the server hard-coding one.
fn production_source(file: &str) -> String {
    let text = source(file);
    match text.find("#[cfg(test)]") {
        Some(at) => text[..at].to_string(),
        None => text,
    }
}

/// One row of the contract's §1 table.
#[derive(Debug)]
struct Point {
    id: String,
    pinned_by: Vec<String>,
    deferred: Option<String>,
}

/// Reads §1's table: `| **P7** | … | … | … | test, test |`.
fn points(page: &str) -> Vec<Point> {
    page.lines()
        .filter(|line| line.starts_with("| **P"))
        .filter_map(|line| {
            // A cell may carry an escaped pipe (`a\\|b`); it is no column border.
            let guarded = line.replace("\\|", "\u{1}");
            let cells: Vec<String> = guarded
                .trim_matches('|')
                .split('|')
                .map(|c| c.trim().replace('\u{1}', "|"))
                .collect();
            if cells.len() < 5 {
                return None;
            }
            let id = cells[0]
                .as_str()
                .trim_matches('*')
                .trim_end_matches(" ⊘")
                .trim()
                .trim_matches('*')
                .to_string();
            let last = cells[4].as_str();
            let deferred = last
                .strip_prefix("deferred:")
                .map(|reason| reason.trim().to_string());
            let pinned_by = if deferred.is_some() {
                Vec::new()
            } else {
                last.split(';')
                    .map(|t| t.trim().trim_matches('`').to_string())
                    .filter(|t| !t.is_empty() && t != "—")
                    .collect()
            };
            Some(Point {
                id,
                pinned_by,
                deferred,
            })
        })
        .collect()
}

/// Is `function` a test this crate RUNS? `#[test]` immediately above
/// the definition, and `#[ignore]` disqualifies it — a recording pass
/// proves nothing offline.
fn is_a_test(source: &str, function: &str) -> Result<(), String> {
    let lines: Vec<&str> = source.lines().collect();
    let needle = format!("fn {function}(");
    let Some(at) = lines
        .iter()
        .position(|line| line.trim_start().starts_with(&needle))
    else {
        return Err(format!("no «fn {function}»"));
    };
    let mut attributes = Vec::new();
    for line in lines[..at].iter().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[") {
            attributes.push(trimmed);
            continue;
        }
        if trimmed.starts_with("///") || trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }
        break;
    }
    if attributes.iter().any(|a| a.starts_with("#[ignore")) {
        return Err(format!(
            "«fn {function}» carries #[ignore] — a recording pass proves nothing offline"
        ));
    }
    if attributes.contains(&"#[test]") {
        Ok(())
    } else {
        Err(format!(
            "«fn {function}» carries no #[test] — a helper is not a proof"
        ))
    }
}

/// The findings of the gate over a page and a way of reading sources.
fn problems_of(page: &str, source_of: &dyn Fn(&str) -> Option<String>) -> Vec<String> {
    let points = points(page);
    let mut findings = Vec::new();
    if points.is_empty() {
        return vec!["TOOLSET-v0.md: no point rows («| **P1** | …») found in §1".into()];
    }
    for point in &points {
        if let Some(reason) = &point.deferred {
            if reason.len() < 10 {
                findings.push(format!(
                    "{}: «deferred» without a reason — a point may wait, but it must say why",
                    point.id
                ));
            }
            continue;
        }
        if point.pinned_by.is_empty() {
            findings.push(format!(
                "{}: no test named — a contract point is pinned or it is deferred with a reason",
                point.id
            ));
            continue;
        }
        for reference in &point.pinned_by {
            let Some((file, function)) = reference.split_once("::") else {
                findings.push(format!(
                    "{}: «{reference}» is no «file::function»",
                    point.id
                ));
                continue;
            };
            let Some(text) = source_of(file) else {
                findings.push(format!(
                    "{}: «{file}» is not a file of this crate",
                    point.id
                ));
                continue;
            };
            if let Err(why) = is_a_test(&text, function) {
                findings.push(format!("{}: {why} (in {file})", point.id));
            }
        }
    }
    findings
}

fn source_from_disk(file: &str) -> Option<String> {
    let text = source(file);
    (!text.is_empty()).then_some(text)
}

/// THE GATE: every point of the contract is pinned by a test this
/// crate runs, or deferred with a reason.
#[test]
fn the_contract_claims_only_what_the_suite_runs() {
    let page = contract();
    let findings = problems_of(&page, &source_from_disk);
    assert!(
        findings.is_empty(),
        "the contract table and the suite disagree:\n  {}",
        findings.join("\n  ")
    );
    let points = points(&page);
    let pinned = points.iter().filter(|p| p.deferred.is_none()).count();
    let deferred = points.len() - pinned;
    println!("contract points: {} pinned, {deferred} deferred", pinned);
    assert_eq!(points.len(), 38, "the contract carries 38 points");
}

/// The gate itself, proven to bite over synthetic text: a point with
/// no test, a point naming a function nobody wrote, a point naming a
/// helper, a point naming an `#[ignore]`d recorder, and a «deferred»
/// with no reason.
#[test]
fn the_gate_refuses_a_point_the_suite_does_not_pin() {
    const SOURCE: &str = "\
/// A real test.
#[test]
fn a_real_test() { assert!(true); }

/// A helper, not a test.
fn a_helper() {}

/// A recorder.
#[test]
#[ignore = \"hits the endpoint\"]
fn a_recorder() {}
";
    let page = |cell: &str| {
        format!(
            "| # | Contract point | Rules | Owner | Pinned by |\n\
             |---|---|---|---|---|\n\
             | **P1** | something | C0.1 | list_cubes | {cell} |\n"
        )
    };
    let sources = |file: &str| (file == "tests/x.rs").then(|| SOURCE.to_string());

    assert!(
        problems_of(&page("tests/x.rs::a_real_test"), &sources).is_empty(),
        "a point pinned by a real test passes"
    );
    let missing = problems_of(&page("tests/x.rs::nobody_wrote_this"), &sources);
    assert!(
        missing
            .iter()
            .any(|f| f.contains("no «fn nobody_wrote_this»")),
        "{missing:?}"
    );
    let helper = problems_of(&page("tests/x.rs::a_helper"), &sources);
    assert!(
        helper.iter().any(|f| f.contains("carries no #[test]")),
        "{helper:?}"
    );
    let recorder = problems_of(&page("tests/x.rs::a_recorder"), &sources);
    assert!(
        recorder.iter().any(|f| f.contains("#[ignore]")),
        "{recorder:?}"
    );
    let empty = problems_of(&page("—"), &sources);
    assert!(
        empty.iter().any(|f| f.contains("no test named")),
        "{empty:?}"
    );
    let unreasoned = problems_of(&page("deferred: x"), &sources);
    assert!(
        unreasoned.iter().any(|f| f.contains("without a reason")),
        "{unreasoned:?}"
    );
    let reasoned = problems_of(
        &page("deferred: the manifest is written in commit 2, with the entry"),
        &sources,
    );
    assert!(
        reasoned.is_empty(),
        "a reasoned deferral passes: {reasoned:?}"
    );
}

/// EVERY test this crate's documents name exists and runs (BY‴, the
/// class sweep).
///
/// The contract's 38-point table is held by the gate above; its
/// «Built» lines, `ENGINE.md` and `README.md` name tests in prose, and
/// nothing read those. A named test that does not exist is the same
/// claim as an unpinned point, one paragraph away from the table that
/// forbids it. A reference into ANOTHER crate is written with its full
/// path and skipped here — this gate speaks for this crate only.
#[test]
fn every_test_the_documents_name_exists_and_runs() {
    let mut checked = 0usize;
    let mut problems = Vec::new();
    for document in ["TOOLSET-v0.md", "ENGINE.md", "README.md"] {
        let page = std::fs::read_to_string(crate_dir().join(document)).unwrap_or_default();
        for reference in page.split('`') {
            let Some((file, function)) = reference.split_once("::") else {
                continue;
            };
            if !(file.ends_with(".rs")) || !(file.starts_with("tests/") || file.starts_with("src/"))
            {
                continue;
            }
            let function = function.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
            checked += 1;
            let source = source(file);
            if source.is_empty() {
                problems.push(format!("{document}: «{file}» is no file of this crate"));
            } else if !source.contains(&format!("fn {function}(")) {
                problems.push(format!(
                    "{document}: «{file}::{function}» — no such function"
                ));
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n  "));
    assert!(
        checked >= 10,
        "the documents name tests, and this gate reads them: {checked} found"
    );
    println!("documents name {checked} tests of this crate, all of them real");
}

// --- the three points that are properties of the source --------------

/// P4: no dimension name, no vocabulary and no family is hard-coded.
/// The served scope is the ONE place that names IRIs of the holding,
/// and `domain.rs` names none.
#[test]
fn nothing_of_the_holding_is_hard_coded_outside_the_scope() {
    let domain = production_source("src/domain.rs");
    for forbidden in [
        "politics.ld.admin.ch",
        "popular-vote",
        "standesstimmen",
        "hasCanton",
        "creativeWorkStatus/",
    ] {
        assert!(
            !domain.contains(forbidden),
            "«{forbidden}» is hard-coded in domain.rs — everything a tool needs about a cube is \
             read from that cube (P4)"
        );
    }
    // The scope module names the 44 — that is what a LIST is (P1).
    let scope = production_source("src/scope.rs");
    assert_eq!(
        scope.matches("politics.ld.admin.ch").count(),
        1,
        "the prefix is written once; the 44 are relative paths"
    );
}

/// P23: v0 offers no join tool — the eight ids are the contract's
/// eight, and none of them joins two cubes.
#[test]
fn the_eight_tools_are_the_contracts_eight_and_none_joins() {
    let server = production_source("src/server.rs");
    let ids: BTreeSet<&str> = [
        "lindas.list_cubes",
        "lindas.find_cube",
        "lindas.describe_cube",
        "lindas.dimension_values",
        "lindas.observations",
        "lindas.list_versions",
        "lindas.describe",
        "lindas.resolve_label",
    ]
    .into_iter()
    .collect();
    for id in &ids {
        assert!(server.contains(id), "{id} is not mounted");
    }
    let mounted = server.matches("name = \"lindas.").count();
    assert_eq!(mounted, 8, "eight tools, no more");
    assert!(
        !server.contains("join"),
        "no join tool in v0 (P23): the seat rows carry their seats, and a join would answer a \
         question none of the six families asks"
    );
}

/// §8's two timeout classes are REACHED, not merely declared (BX′).
///
/// The offline backends answer by fixture key and never look at the
/// query or the timeout, so no test over recorded answers can tell a
/// describe that ran at 30 s from one that ran at 15 — a reviewer
/// demonstrated exactly that by swapping the call back and watching
/// the whole suite stay green. This gate reads the crate's own source,
/// the way P4, P23 and P31 are read: the description PAGE asks through
/// the wide class, the count keeps the narrow one, and both classes
/// have a call half and a body half.
#[test]
fn the_description_page_asks_through_the_wide_class() {
    let domain = production_source("src/domain.rs");
    assert!(
        domain.contains("ask_wide(ctx, &format!(\"describe:{iri}:{limit}:{offset}\"), &query)"),
        "the description PAGE — the one that reads a wide body — asks at DESCRIBE_TIMEOUT (§8)"
    );
    assert!(
        domain.contains("ask(\n        ctx,\n        &format!(\"describe:count:{iri}\"),"),
        "the count is one row and keeps the select bound"
    );
    assert!(
        domain.contains("fn ask_wide(") && domain.contains("describe_within(key, query)"),
        "ask_wide is the wide class's one door"
    );

    let backend = production_source("src/backend.rs");
    for class in [
        "fn select_class()",
        "fn select_body_class()",
        "fn describe_class()",
        "fn describe_body_class()",
        "fn classes_of(",
    ] {
        assert!(
            backend.contains(class),
            "both bounds carry a call half and a body half: «{class}» is missing"
        );
    }
    assert!(
        backend.contains("let (class, body_class) = classes_of(timeout);"),
        "one place decides the class pair — deriving one half at the call site is how a \
         describe came to report «select, timeout 15 s» for a body it read at 30"
    );
}

/// P31: every query binds a subject or a predicate — the crate carries
/// no unbound scan, and no graph enumeration at all.
#[test]
fn no_query_of_this_crate_is_unbound() {
    let domain = production_source("src/domain.rs");
    for forbidden in ["STRSTARTS", "GRAPH ?", "DISTINCT ?g", "?s ?p ?o"] {
        assert!(
            !domain.contains(forbidden),
            "«{forbidden}» would be an unbound scan (P31, C12.3); graph enumeration timed out at \
             90 s and is not offered at all"
        );
    }
    // Every SELECT of this crate names its subject or its predicate.
    let selects = domain.matches("SELECT").count();
    assert!(selects >= 8, "the tools ask real queries: {selects}");
    assert!(
        domain.matches("VALUES ?cube").count() >= 1,
        "the scope-wide queries bind their subjects with VALUES"
    );
}
