# lindas engine — E15 contract + strategy description

<!-- language: English -->

**The E15 clause-1 artifacts for the LINDAS `cube.link` engine:** the
strategy description (this file) and the engine manifest
(`engine.manifest.json`). E15 normalizes for every engine «den
Kontrakt (Eingabe-Typen des Bestands, erzeugtes Manifest inkl.
Stufen-Deklaration, capabilities) und die Strategie-Beschreibung
(Chunking, Graphmodell, Retrieval, Eval — lernbar und auf eigene
Bestände übertragbar), keine Binärform» (docs/decisions/E15-engine-format-governance.md, Ziff. 1).
The shape follows the first-instance extraction of the fedlex family
(`../fedlex/ENGINE.md`), which is what E15 calls for until the
`engines/standard` 0.x specification exists.

**What this file is NOT.** It invents nothing. Every figure below is
either measured (`testing/lindas-probe/`, dated in
`docs/explanation/research-lindas-cube.md`), a rule of
`docs/reference/lindas-cube-rules.md` (58 rules `C0`–`C14`), a point of
this directory's `TOOLSET-v0.md` (38 points, `P1`–`P38`), or a test of
this crate. Where a claim carries no such anchor it is not made.

**The order this server was built in is the argument.** Data →
rulebook → §16 → contract → crate. The fedlex server was built first
and its hardest defects were DATA defects; here the measurement came
first, the rulebook second, the contract third, and the crate last —
and `tests/contract_table.rs` holds the crate against the contract the
way `rules_table.rs` holds the fedlex crate against its rules.

## 1. The tier ensemble (Stufen-Deklaration)

**Base is built** (this directory, `oh-mcp-lindas` 0.1.0): eight tools
over the PUBLIC LINDAS SPARQL endpoint, stateless, no index of its
own, every answer offline-provable on recorded fixtures. **Semantic
and generative are not built** and are declared as planned tiers, never
as served capability. There is no vendored upstream: unlike the fedlex
engine, this one has no reference implementation — the reference is the
measured holding.

**Two-stage discovery is part of the contract (E16 Ziff. 1).** Stage
one is the one-line inventory — `tools/list` here, `meta.tools` at the
gateway — where every line is ≤ 160 characters, begins with the verb,
says WHEN to use the tool and whether it answers a `hint` or a `norm`,
and carries the German trigger words a question would contain
(Abstimmung, Volksinitiative, Ständemehr, Kanton, Wahl, Sitz,
Interessenbindung, Geschäftsstand). Stage two loads the input schemas
of the tools a model intends to call. The eight lines are fixed in
`TOOLSET-v0.md` §5 and pinned verbatim by the suite (P36).

## 2. Strategy: chunking

A cube store has no text to chunk; the unit of retrieval is given by
the data model, and picking it wrongly is the whole risk.

- **The chunk is the observation row** — one vote × one region (C8.1),
  one candidate × one year (C10.4). A row is never split and never
  merged: `lindas.observations` returns rows with their dimensions as
  cells, and a cell carries `{value, label?, datatype?, stated}`.
- **The addressable unit above it is the cube VERSION**, not the cube
  family: the version is the last segment of the IRI where there is
  one (C1.1), a new version is a new cube IRI, and nothing links an
  old version to a new one (C5.3). `lindas.list_versions` therefore
  lists what exists and orders it; it never claims a succession the
  store does not state (P16).
- **The cap is the tool's, because the endpoint has none** (C6.4,
  C12.1): every list answers with `limit`, `offset`, `total`,
  `returned` and `truncated`. Nothing is truncated silently, and the
  original size is always in the answer (P32).
- **Descriptions are bounded by breadth, not by length**: one
  observation can carry 304 predicates (C6.3), so `lindas.describe`
  caps statements and says so, and gets the wider 30 s timeout.

## 3. Strategy: graph model

Harvested from `docs/reference/lindas-cube-rules.md`; the crate reads
these facts, it never assumes them.

- **The vocabulary is `cube.link`**, one endpoint carrying 2'028 cubes;
  the served scope is the 44 cubes under `politics.ld.admin.ch` in four
  families (C0.1, C0.3). The scope is a LIST in `src/scope.rs` — the
  ONE place in the crate that names IRIs of the holding (P1, P4,
  test-pinned).
- **The schema is a SHACL shape, and it is a SUBSET of the record**
  (C2.2): 14 declared dimensions against 51 carried in the largest
  cube. Hence the three-step dimension rule (P12): a declared dimension
  is used without a request; an undeclared one is ADMITTED by a bound
  `ASK` against the cube and then reported in
  `undeclared_dimensions`; a dimension the ASK denies is a typed
  `not-found` naming cube and dimension. The server never pretends the
  shape is the truth, and never scans to find out.
- **Dimensions are cube-local, values are shared** (C1.4, C1.2): a
  dimension IRI is not built by appending to the cube IRI, so it is
  read from the shape; a value IRI (`ld.admin.ch/canton/1`) is shared
  across cubes and across hosts, which is why `lindas.resolve_label`
  accepts an IRI of ANY host and still asks the ONE endpoint (P30).
- **«Not stated» is written in two shapes** (C3.1): the IRI
  `cube.link/Undefined` and an empty typed literal. Both answer
  `stated: false` with the `form` that was found — never `0`, never
  `""`, never a dropped row (P8).
- **Status is a vocabulary with one occupied value and an unset third
  state** (C5.1): the answer carries `status`, `status_label`,
  `status_label_lang` and `status_unset` — «unset» is a state, not a
  synonym for Draft. Orthogonally, a published cube may hold zero
  observations (C5.4): `placeholder: true` with `observation_count: 0`
  is an ANSWER, never a fault (P6, P7).
- **No cube points at another cube** (C7.2). There is no join tool in
  v0 (P23): the seat rows carry their seats, and the one real join in
  the holding is by a literal id (C9.1) or by a list NUMBER (C10.2) —
  both are the caller's, done over two answers, and both are named in
  the contract instead of guessed by the server.
- **The scope lives in ten named graphs** (C7.5) — which the server
  never enumerates: graph enumeration does not answer inside 90 s
  (C12.3), so it is not offered at all, at any price (P31).

## 4. Strategy: retrieval

- **Two structurally distinct answer kinds.** Every answer is a `norm`
  (a figure or a statement of the store, quotable, with provenance) or
  a `hint` (a candidate — a search result, a label, a value list —
  never quotable). `find_cube` and `resolve_label` and
  `dimension_values` are hints; the five others are norms (P34). The
  loop is «hint → norm → cite», as in the fedlex engine, and the split
  makes recording a search hit as substantiated data structurally
  impossible.
- **Every query binds a subject or a predicate** (P31, C12.3). The
  scope-wide queries bind their 44 subjects with `VALUES ?cube`; the
  per-cube queries bind the cube; the label query binds the IRI. There
  is no `STRSTARTS` prefix filter, no unbound `?s ?p ?o`, no
  `GRAPH ?g`. `tests/contract_table.rs` proves this over the crate's
  own source.
- **Query templates are fixed; only the bindings vary.** A caller's
  string never becomes SPARQL syntax: IRIs are shape-checked and
  literals are escaped, and a literal filter is expressed as
  `?obs <dim> ?v . FILTER(STR(?v) = "…")` so that the typed literals of
  the store (C3.5: one date dimension carries three datatypes) match
  regardless of their datatype.
- **Labels have a language and a fallback chain, and «all languages» is
  never fetched** (C4.4, P29): de → fr → it → en → rm, an untagged
  label answers as language `und` (C4.1: nine cubes carry an untagged
  German name), and the language filter is IN the query — one value of
  the recorded corpus carries labels in 45 languages, and one cube's
  values carry 181'228 label rows.
- **Four typed refusals, and nothing else.** `not-found` (the store
  answers, and the answer is empty for a bound subject),
  `invalid-input` (decidable without a request — a malformed IRI, a
  limit out of range, a cube outside the served scope; a clean HTTP 406
  from the endpoint is mapped here too, C12.2), `upstream-unavailable`
  (the class and its bound are named: «SPARQL select (timeout 15 s)»,
  «SPARQL result body (select, timeout 15 s)», «SPARQL describe
  (timeout 30 s)», «SPARQL result body (describe, timeout 30 s)») and
  `upstream-busy` with `retry_after_ms` from the brake. Every one of
  those strings is pinned by a test, and both halves of a bound belong
  to the same class — until BX′ a describe that failed while READING
  its answer reported the select class's 15 s.
- **Provenance on every answer**: `source` (the one endpoint; the
  field is named `source`, which is what the answers carry), `as_of`
  (injected — the library never reads a clock), `served` (`live` | `fixture`),
  `licence: "not stated at the source"` and
  `access: "public (I14Y)"`. The licence field says what is true: none
  of the 44 cubes states a licence in the graph at any distance the
  probe looked, and the I14Y schema for a data SERVICE carries no
  licence field at all — while access is registered PUBLIC
  (`dataservice/c9cf11b6-d165-4498-92fc-d51167def66c`).

## 5. Strategy: eval

- **The contract gate.** `tests/contract_table.rs` walks the 38 points
  of `TOOLSET-v0.md` §1 and fails when a point names no test, names a
  function nobody wrote, names a helper, names an `#[ignore]`d
  recorder, or says «deferred» without a reason. The gate is itself
  proven to bite over synthetic text. **37 pinned, 1 deferred** since
  the mount (P37 waited for the manifest, which now exists; P18 answers
  no related cubes at all, and says why).
- **Offline by default.** 80 fixture keys under `tests/fixtures/` with
  SEMANTIC keys — `<tool>:<cube version IRI>:<arguments>`, hashed to a
  file name, listed with their recording date in `INDEX.txt`. The
  offline e2e suite (34 tests) covers every tool's example of the
  contract, the six acceptance families, and the states (placeholder,
  status-unset, both forms of Undefined, the untagged name, the
  undeclared dimension, the empty answer).
- **The recording pass is measured, not trusted.** `Backend::Counting`
  replays the exact call sequence the recorder uses and counts the
  requests it would send; the test asserts the count stays under the
  declared budget. Measured: **108 requests over 80 keys** (104 over 78 before the audit fixes of 01.09.2026, 91 over 73 at BX; the
  describe pages of BX′ added two keys; the two projected calls of BY
  point 0 added two; the two-word `find_cube` window of BY′ added one —
  and every one of those passes was measured against the fixtures
  BEFORE it was spent live).
- **The probe scripts are the ground truth.** `testing/lindas-probe/`
  (fourteen Rust probes, `c1`…`c14`) produced every figure in the
  rulebook; the rulebook's §16 collects the consequences per tool, and
  `tools/consistency` fails when a rule reaches no §16 line, when §16
  names an id that is no rule, or when a rule §16 cites appears nowhere
  in the contract. Rule → §16 → contract → test is one unbroken chain,
  and every link is gated.

## 6. Manifest, capabilities, egress

- **ONE host, enforced in code.** `backend::ENDPOINT_HOST =
  "https://lindas.admin.ch/"`; a connect target that does not carry the
  prefix is refused as `invalid-input` before a socket is opened —
  including a `--endpoint` a deployer passes. The nine `*.ld.admin.ch`
  hosts that appear IN the data stay reader-side identifiers: the
  endpoint's DESCRIBE was measured to be a superset of the host's
  dereference, so the server never follows them.
- **The polite brake.** Every live request takes a token from one
  bucket: 2 a second sustained, burst 4 (`--upstream-rate`,
  `--upstream-burst`); without a token a request waits up to 5 s and is
  then refused at once as `upstream-busy` with `retry_after_ms`.
  Fixtures are never braked. The mechanism is the shared
  `oh-mcp-common::throttle` — the same one the fedlex server uses; only
  the refusal's wording is this server's, and it names
  `lindas.admin.ch`.
- **The agent identifies itself:** `oh-mcp-lindas/0.1
  (+https://openhelvetia.swiss; base-tier domain server)`.
- **Timeouts are the caller's patience, not the endpoint's — per
  REQUEST:** 15 s for a select or ASK (which leaves room for the
  brake's 5 s reservation) and 30 s for the description PAGE, the one
  thing here that reads a wide body; a description's count is one row
  and keeps the 15 s bound. One `describe` call therefore sends two
  braked requests, and its worst case before a refusal is
  (5 + 15) + (5 + 30) = **50 s** — the honest per-CALL figure, which no
  document may shorten to «30 s».
- **Weight 2 at the gateway**, as for the other live SPARQL surface.
- **The manifest** (`engine.manifest.json`, 0.1.0) declares the tier,
  the eight capabilities, the one egress host and the provenance
  fields; it is written together with the gateway mount and the parked
  registry entry, and is egress-conformance-tested there.

## 7. What this engine does NOT serve

Named, with the ground, because an engine description that lists only
what exists is a sales page:

- **No graph enumeration, at any price** (C12.3): 90 s without an
  answer where the typed query answers in 223 ms.
- **No join tool** (P23): the two real joins of the holding are by a
  literal id and by a list number; the contract names them and the
  caller does them over two answers.
- **No related-cube discovery** (P18, deferred): no cube points at
  another (C7.2), so v0 answers no relation it would have to invent.
- **No full-text search of the store**: `find_cube` searches the NAMES
  of the 44 served cubes, which is what a 44-element list allows, and
  says `hint`.
- **No derived verdict**: the Ständemehr is stated in three forms and
  must never be counted from the canton rows (C8.3) — 15.5 : 6.5 where
  counting cantons gives 17.
- **No policy**: auth, rate limiting and budget live at the L2.3
  gateway boundary (E11/E16). The brake in here is a courtesy to a
  shared public endpoint, not a policy.

## Citations

- `docs/explanation/research-lindas-cube.md` — the CRISP-DM assessment,
  §2b (the four «no» conditions measured) and §2c (the findings).
- `docs/reference/lindas-cube-rules.md` — 58 rules `C0`–`C14`, each
  with its figure; §16 collects every consequence per tool.
- `mcp/servers/lindas/TOOLSET-v0.md` — the contract: 38 points, eight
  tools, the acceptance families, the served scope.
- `testing/lindas-probe/` — the fourteen probe scripts and their
  request record.
- `docs/decisions/E15-engine-format-governance.md`,
  `E16-machine-view.md`, `E11-widget-cost-model.md` — the sealed
  decisions this file answers to.
