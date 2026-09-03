# oh-mcp-lindas — LINDAS `cube.link` domain server, base tier (L2.7)

<!-- language: English -->

The platform's SECOND domain server, and the first one built from a
contract that existed before it: eight MCP tools over the PUBLIC
LINDAS SPARQL endpoint of the federal administration
(`lindas.admin.ch/query`), serving the 44 political data cubes under
`politics.ld.admin.ch` — Abstimmungen, Wahlen, Bundesrat,
Interessenbindungen.

**The order is the point.** The holding was measured first
(`testing/lindas-probe/`, fourteen probes), the measurements became
`docs/reference/lindas-cube-rules.md` (58 rules `C0`–`C14`, each with
its figure), the rulebook's §16 collected every consequence per tool,
[`TOOLSET-v0.md`](TOOLSET-v0.md) derived 38 numbered contract points
from §16 — and only then this crate. `tests/contract_table.rs` walks
those 38 points and fails when one is neither pinned by a test this
crate runs nor deferred with a reason (**37 pinned, 1 deferred**).
Rule → §16 → contract → test is one chain, and every link is gated.

- **The eight tools:** `list_cubes` (the served scope with its states),
  `find_cube` (a word of the name → candidates, `hint`),
  `describe_cube` (the declared dimensions, and the honest note that
  the record carries more), `dimension_values` (the values one
  dimension takes, labelled, `hint`), `observations` (the rows
  themselves, filtered, with cells that say whether a value is
  stated), `list_versions` (the versions of a family — and nothing
  links old to new, so nothing claims it), `describe` (everything the
  store says about one IRI), `resolve_label` (an IRI of ANY host,
  answered by the ONE endpoint, in one language with a fallback,
  `hint`).
- **States are answers, never faults:** a published cube holding zero
  observations is `placeholder: true` (four of the 44 are), a cube
  without `creativeWorkStatus` is `status_unset: true` (that is a third
  state, not Draft), and both written forms of «not stated» — the
  `cube.link/Undefined` IRI and an empty typed literal — carry through
  as `stated: false` with the form that was found, never as `0` and
  never as a dropped row.
- **The shape is a subset of the record** (14 declared dimensions
  against 51 carried in the largest cube), so a filter on an
  undeclared dimension is not refused blindly: a bound `ASK` admits
  it and the answer reports it in `undeclared_dimensions`; only a
  dimension the store denies is a typed `not-found`.
- **Two-stage discovery (E16):** every tool's description IS its
  stage-one line — ≤ 160 characters, verb-first, saying when to use it
  and whether it answers a `hint` or a `norm`, with the German trigger
  words; the gateway's `meta.tools` carries the same lines verbatim.
- **Egress:** ONE host, `lindas.admin.ch`, enforced in
  `backend::ENDPOINT_HOST` before a socket is opened — a `--endpoint`
  that is not it is refused as `invalid-input`. The nine
  `*.ld.admin.ch` hosts that appear in the DATA stay reader-side
  identifiers; the server never dereferences them. Timeouts, per
  request: 15 s for a select, 30 s for the description page that reads
  the wide body (its count keeps the 15 s bound), so one `describe`
  call is bounded at (5 + 15) + (5 + 30) = 50 s with the brake's
  reservations; a timeout refusal names its class
  and its bound.
- **The polite brake:** one token bucket over every live request — 2 a
  second sustained, burst 4 (`--upstream-rate <n/s>`,
  `--upstream-burst <n>`); without a token a request waits up to 5 s,
  beyond that it is refused as `upstream-busy` with `retry_after_ms`.
  The mechanism is the shared `oh-mcp-common` crate — the same brake
  the fedlex server uses; only the wording is this server's. Fixtures
  are never braked.
- **No graph enumeration, at any price:** it does not answer inside
  90 s where the typed query answers in 223 ms — so no tool offers it,
  and every query this crate sends binds a subject or a predicate
  (proven over the crate's own source).
- **Offline by default in tests:** 80 recorded fixtures with SEMANTIC
  keys (`tests/fixtures/` + `INDEX.txt`, keys are
  `<tool>:<cube version IRI>:<arguments>`). Re-record deliberately:
  `cargo test --test e2e record_fixtures -- --ignored --test-threads 1`
  — one recording pass at a time, sequential, polite. The pass is
  MEASURED before it runs: `Backend::Counting` replays the same call
  sequence and counts the requests (108 requests over 80 keys), and a test holds
  that count under the declared budget.
- **Provenance on every answer:** `source` (the one endpoint; the field
  is named `source`, which is what the answers carry), `as_of`
  (injected — the library reads no clock), `served: live|fixture`, `licence: "not
  stated at the source"` (none of the 44 states one in the graph) and
  `access: "public (I14Y)"` (the data service is registered
  `accessRights: PUBLIC`).
- Run: `oh-mcp-lindas` (live, polite UA) or `--fixtures <dir>` (fully
  offline). Policy — auth, rate, budget — lives at the L2.3 gateway
  boundary (E11/E16); this crate is domain logic only.

[`ENGINE.md`](ENGINE.md) is the E15 strategy description (chunking,
graph model, retrieval, eval, egress); `engine.manifest.json` is its
companion contract artifact.
