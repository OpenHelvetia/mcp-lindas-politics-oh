//! The served scope: a LIST of 44 cube IRIs, not a pattern (contract
//! P1, §9 — E15's «widening the scope changes a list, not code»).
//!
//! The list is the answer of the typed listing query on 2026-08-29,
//! kept verbatim in `testing/lindas-probe/cubes.txt`; a cube that is
//! not in it is `not-found` and is never fetched. The build-time
//! verification against `?cube a cube:Cube` lives in the recording
//! pass, not here: this module is what the server serves, and it must
//! answer without a request.

/// The prefix every served cube carries. It is NOT a membership test —
/// membership is the list below (P1).
pub const SCOPE_PREFIX: &str = "https://politics.ld.admin.ch/";

/// The 44 cubes, as relative paths under [`SCOPE_PREFIX`].
pub const CUBES: [&str; 44] = [
    "fc/cube-chancellor",
    "fc/cube-councillor",
    "fc/cube-declined-election",
    "fc/cube-department",
    "fc/cube-president",
    "fch/apg/committee-canton-statistic/1",
    "fch/apg/committee-function-statistic/1",
    "fch/apg/committee-gender-language-statistic/1",
    "fch/apg/committee-type-department-statistic/1",
    "fch/apg/committee-type-statistic/1",
    "fch/apg/committee/1",
    "fch/apg/membership/1",
    "fch/apg/person/1",
    "fch/apg/vested-interest/1",
    "national-council-election/candidates/2019",
    "national-council-election/candidates/2023",
    "national-council-election/candidates/2027",
    "national-council-election/canton-candidate-statistics/2019",
    "national-council-election/canton-candidate-statistics/2023",
    "national-council-election/canton-candidate-statistics/2027",
    "national-council-election/canton-election-statistics/2019",
    "national-council-election/canton-election-statistics/2023",
    "national-council-election/canton-election-statistics/2027",
    "national-council-election/list-results/2019",
    "national-council-election/list-results/2023",
    "national-council-election/list-results/2027",
    "national-council-election/seats-in-connected-lists/2019",
    "national-council-election/seats-in-connected-lists/2023",
    "national-council-election/seats-in-connected-lists/2027",
    "national-council-election/seats-per-list/2019",
    "national-council-election/seats-per-list/2023",
    "national-council-election/seats-per-list/2027",
    "political-rights/petition/1",
    "political-rights/political-party-register-persons/1",
    "political-rights/political-party-register/1",
    "political-rights/popular-initiative-keyfigures-stat/1",
    "political-rights/popular-initiative-stat/1",
    "political-rights/popular-initiative/1",
    "political-rights/popular-vote-stat/1",
    "political-rights/popular-vote/1",
    "political-rights/popular-vote/voting_dates/1",
    "political-rights/referendum-keyfigures-stat/1",
    "political-rights/referendum-stat/1",
    "political-rights/referendum/1",
];

/// The four families the scope divides into (C0.1, C0.3).
pub const FAMILIES: [&str; 4] = [
    "fc",
    "fch/apg",
    "national-council-election",
    "political-rights",
];

/// The full IRI of a served cube, or `None` when the path is not in
/// the list.
pub fn iri_of(path: &str) -> Option<String> {
    CUBES
        .iter()
        .find(|c| **c == path)
        .map(|c| format!("{SCOPE_PREFIX}{c}"))
}

/// Is this IRI one of the 44? (P1 — the only membership test there is.)
pub fn is_served(iri: &str) -> bool {
    iri.strip_prefix(SCOPE_PREFIX)
        .is_some_and(|rest| CUBES.contains(&rest))
}

/// Every served cube as a full IRI, in the list's order.
pub fn all() -> Vec<String> {
    CUBES.iter().map(|c| format!("{SCOPE_PREFIX}{c}")).collect()
}

/// The cubes of one family, or all of them when no family is named.
/// An unknown family name answers `None` — the caller turns that into
/// `invalid-input`, decidable without a request (contract §2).
pub fn of_family(family: Option<&str>) -> Option<Vec<String>> {
    let Some(family) = family else {
        return Some(all());
    };
    if !FAMILIES.contains(&family) {
        return None;
    }
    Some(
        CUBES
            .iter()
            .filter(|c| family_of_path(c) == family)
            .map(|c| format!("{SCOPE_PREFIX}{c}"))
            .collect(),
    )
}

/// The family a served path belongs to.
pub fn family_of_path(path: &str) -> &str {
    if path.starts_with("fch/apg/") {
        "fch/apg"
    } else {
        path.split('/').next().unwrap_or(path)
    }
}

/// The family of a full IRI (`None` when it is not served).
pub fn family_of(iri: &str) -> Option<&'static str> {
    let rest = iri.strip_prefix(SCOPE_PREFIX)?;
    let path = CUBES.iter().find(|c| **c == rest)?;
    Some(match family_of_path(path) {
        "fch/apg" => "fch/apg",
        "fc" => "fc",
        "national-council-election" => "national-council-election",
        _ => "political-rights",
    })
}

/// The version segment of a cube IRI — its LAST path segment, when it
/// is one (C1.1: `/1`, `/2019`; five cubes carry none).
pub fn version_of(iri: &str) -> Option<String> {
    let rest = iri.strip_prefix(SCOPE_PREFIX)?;
    let last = rest.rsplit('/').next()?;
    if !last.is_empty() && last.chars().all(|c| c.is_ascii_digit()) {
        Some(last.to_string())
    } else {
        None
    }
}

/// The family stem of a versioned cube: its IRI without the version
/// segment. For an unversioned cube the IRI itself, because that IS
/// its family (C1.1, P26).
pub fn version_family(iri: &str) -> String {
    match version_of(iri) {
        Some(version) => iri
            .strip_suffix(&format!("/{version}"))
            .unwrap_or(iri)
            .to_string(),
        None => iri.to_string(),
    }
}

/// The dimension base of a cube: the IRI WITHOUT its version segment,
/// which is where this holding writes its dimension IRIs (C1.2 — a
/// dimension IRI is never built by appending to the cube IRI; this
/// function exists so no other code is tempted to try).
pub fn dimension_base(iri: &str) -> String {
    version_family(iri)
}

/// Every served cube of the same family as `iri`, version by version
/// (P26): the version list is a filter over the scope, not a query.
pub fn versions_of(iri: &str) -> Vec<String> {
    let stem = version_family(iri);
    all()
        .into_iter()
        .filter(|c| version_family(c) == stem)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// P1: the scope is a list of 44, and membership is the list.
    #[test]
    fn the_served_scope_is_a_list_of_forty_four() {
        assert_eq!(CUBES.len(), 44);
        assert_eq!(all().len(), 44);
        assert!(is_served(
            "https://politics.ld.admin.ch/political-rights/popular-vote/1"
        ));
        assert!(
            !is_served("https://politics.ld.admin.ch/political-rights/popular-vote/2"),
            "a cube that only LOOKS served is not served (P1)"
        );
        assert!(!is_served("https://politics.ld.admin.ch/some/other/cube"));
        assert!(!is_served("https://example.org/cube"));
    }

    /// C0.1: four families, and the counts the rulebook measured.
    #[test]
    fn the_four_families_carry_the_counts_the_rulebook_measured() {
        let count = |f: &str| of_family(Some(f)).expect("a family").len();
        assert_eq!(count("fc"), 5);
        assert_eq!(count("fch/apg"), 9);
        assert_eq!(count("political-rights"), 12);
        assert_eq!(count("national-council-election"), 18);
        assert_eq!(of_family(None).expect("all").len(), 44);
        assert!(
            of_family(Some("nonsense")).is_none(),
            "decidable without a request"
        );
    }

    /// C1.1: the version is the last segment when there is one, and
    /// five cubes carry none.
    #[test]
    fn the_version_is_the_last_segment_when_there_is_one() {
        assert_eq!(
            version_of("https://politics.ld.admin.ch/national-council-election/candidates/2023")
                .as_deref(),
            Some("2023")
        );
        assert_eq!(
            version_of("https://politics.ld.admin.ch/political-rights/popular-vote/1").as_deref(),
            Some("1")
        );
        assert_eq!(
            version_of("https://politics.ld.admin.ch/fc/cube-councillor"),
            None,
            "five cubes carry no version segment"
        );
        assert_eq!(all().iter().filter(|c| version_of(c).is_none()).count(), 5);
    }

    /// P26: the version list is a filter over the scope — and the
    /// unversioned cubes are their own family of one.
    #[test]
    fn the_versions_of_a_family_come_from_the_list() {
        let years =
            versions_of("https://politics.ld.admin.ch/national-council-election/candidates/2019");
        assert_eq!(years.len(), 3, "{years:?}");
        assert!(years.iter().all(|c| c.contains("/candidates/")));
        let alone = versions_of("https://politics.ld.admin.ch/fc/cube-councillor");
        assert_eq!(alone.len(), 1);
    }

    /// C1.2: the dimension base drops the version segment — the thing
    /// a tool must never rebuild by hand.
    #[test]
    fn the_dimension_base_drops_the_version() {
        assert_eq!(
            dimension_base("https://politics.ld.admin.ch/political-rights/popular-vote/1"),
            "https://politics.ld.admin.ch/political-rights/popular-vote"
        );
        assert_eq!(
            dimension_base("https://politics.ld.admin.ch/fc/cube-councillor"),
            "https://politics.ld.admin.ch/fc/cube-councillor"
        );
    }
}

/// Whether an IRI names a region by its HOST: the two shared value
/// hosts C1.4 records, cantons and countries. Lives HERE because P4
/// allows exactly one module to name IRIs of the holding, and this is
/// it. A value from the holding's own vocabulary (§17.11 measured
/// `…/political-rights/vocabulary/MilitarySchool` on a
/// popular-initiative row) is NOT matched by host — it counts as a
/// region only when it stands on a region dimension, which the caller
/// decides with [`is_region_dimension`]; matching the vocabulary host
/// unconditionally would let any vocabulary-valued cell make a page
/// «mixed».
pub fn is_region_value(iri: &str) -> bool {
    iri.starts_with("https://ld.admin.ch/canton/")
        || iri.starts_with("https://ld.admin.ch/country/")
}

/// Whether a dimension IRI is one that carries a row's region: the
/// political-rights family calls it `region`, the election families
/// call it `hasCanton` (T7 of the audit, 01.09.2026 — a test on the
/// first name alone answered «no row carries a region» about every
/// election page). Used only for the honest arms of `region_state`
/// («not projected», «the rows carry none»); when rows arrived, the
/// VALUE test above is authoritative.
pub fn is_region_dimension(dimension: &str) -> bool {
    dimension.ends_with("/region") || dimension.ends_with("/hasCanton")
}
