# lindas domain server — v0 base-tier tool contract (PROPOSAL, text only)

<!-- language: English -->

**Derived, not invented.** Every point below comes from a line of
[`docs/reference/lindas-cube-rules.md`](../../../docs/reference/lindas-cube-rules.md)
§16 «What the contract must carry», and every point names the rule id
it comes from. §16 in turn comes from 58 rules `C0`–`C14`, each with a
figure measured against `https://lindas.admin.ch/query` by the scripts
in [`testing/lindas-probe/`](../../../testing/lindas-probe/README.md).
The chain is rule → §16 → contract, and all three links are gated:
`tools/consistency` fails when a rule reaches no §16 line, when §16
names an id that is no rule, and when a rule cited in §16 appears
nowhere in this file.

**This is text.** No crate, no `Cargo.toml`, no `src/`, no manifest, no
mount, no inventory row, no fixture, and not one request to LINDAS was
made to write it. The server is built when the licence question is
answered (§11).

**Names** follow the platform's capability-id style
(`<domain>.<verb_object>`, the gateway convention: tool names ARE
capability ids). The tools are generic to `cube.link`; the SERVED SCOPE
is a list in this contract (§9), never a pattern in code.

---

## 1. Derivation: §16 → contract points

Every consequence of §16, once, as a numbered point with the rule it
comes from and the tool that owns it. Points marked **⊘** are demanded
by a rule but deliberately NOT built in v0; the reason is with the
point.

| # | Contract point | Rules | Owner | Pinned by |
|---|---|---|---|---|
| **P1** | The served scope is the LIST of 44 cube IRIs in §9, verified against the typed query (`?cube a cube:Cube` + the `politics.ld.admin.ch` prefix) at build time. A cube outside the list is `not-found`, never fetched. | C0.1, C13.1 | `list_cubes` | src/scope.rs::the_served_scope_is_a_list_of_forty_four; tests/e2e.rs::the_served_scope_is_the_list_the_contract_names |
| **P2** | A cube name may carry no language tag. Every label lookup accepts an untagged literal, and every answer names the language it served (`name_lang`, `"und"` when the literal had no tag). | C4.1, C4.2 | `list_cubes`, `find_cube` | tests/e2e.rs::the_untagged_names_of_the_fch_apg_family_are_served |
| **P3** | Status is answered as the VOCABULARY answers it, in the fedlex server's own pattern: `status` (the IRI, or `null`), `status_label` and `status_label_lang` (decoded through the endpoint with P28's fallback; `null` where the vocabulary carries no label — an answer), and `status_unset: true` where the cube carries no status at all (14 of 44). Orthogonally: `placeholder: true` where the CUBE holds 0 observations — decided from the cube's own count, never from a filtered total: a filter that matches nothing is `filters_matched_nothing: true` on a full cube, and reading that miss as an empty cube was the audit's highest-ranked defect (01.09.2026). An ANSWER, never a `not-found` and never an error; none of these is a `state` field. | C5.1, C5.4, C14.1 | `list_cubes`, `describe_cube`, `observations` | tests/e2e.rs::list_cubes_serves_the_scope_with_its_states; tests/e2e.rs::a_placeholder_cube_answers_that_it_is_one; tests/e2e.rs::a_filter_that_matches_nothing_is_a_miss_and_never_a_placeholder |
| **P4** | No dimension name, no vocabulary and no family is hard-coded anywhere in the server. Everything a tool needs about a cube is read from that cube. | C0.3 | all tools | tests/contract_table.rs::nothing_of_the_holding_is_hard_coded_outside_the_scope |
| **P5** | `description` is optional, is never served as a substitute for a missing name, and carries its own language tag (`description_lang`, `"und"` where untagged). | C4.5 | `list_cubes`, `describe_cube` | tests/e2e.rs::list_cubes_serves_the_scope_with_its_states |
| **P6** | A cube profile carries `date_created`, `date_published` and `date_modified` as they are written, each with a `granularity` of `date` or `dateTime`. The server never truncates one into the other. | C5.2 | `describe_cube` | tests/e2e.rs::describe_cube_serves_the_declared_shape_and_says_it_is_only_that |
| **P7** | The nine `fch/apg` cubes are IN the served scope — the register of who sits in which federal committee — and are searchable despite their untagged names. | C11.1, C11.2 | `list_cubes`, `find_cube` | tests/e2e.rs::the_untagged_names_of_the_fch_apg_family_are_served |
| **P8** | A shape answer is capped and paged (`limit` ≤ 100, default 50) with `total` and `truncated`: 288 dimensions in one cube is a real case. | C2.1, C6.3 | `describe_cube` | tests/e2e.rs::describe_cube_serves_the_declared_shape_and_says_it_is_only_that |
| **P9** | What `describe_cube` answers is the DECLARED shape. Every answer carries `declared_only: true` and a note that the observations may carry more predicates — measured: 14 declared against 51 carried in `popular-vote`. | C2.2 | `describe_cube` | tests/e2e.rs::describe_cube_serves_the_declared_shape_and_says_it_is_only_that |
| **P10** | A value is typed from what it IS (IRI or literal, and the literal's own datatype), never from `sh:datatype`, which only 40 of 656 properties carry. Where neither `qudt:scaleType` nor a `cube.link` dimension class is declared, the answer says `kind: "unknown"`. | C2.3, C2.4 | `describe_cube`, `observations` | tests/e2e.rs::describe_cube_serves_the_declared_shape_and_says_it_is_only_that |
| **P11** | Where a property declares `sh:in`, the shape answer carries the enumeration as a ready filter list; `sh:minCount 0` is answered as `optional: true` and its absence in an observation is not a fault. | C2.5 | `describe_cube`, `dimension_values` | tests/e2e.rs::the_referendum_and_initiative_families_answer_from_their_cubes |
| **P12** | A dimension IRI is never constructed from a cube IRI: every tool takes it as `describe_cube` served it. A dimension the shape does not declare is decided in THREE steps, the same for a filter, a projection and `dimension_values`: **(a)** declared by the shape → accepted, no request; **(b)** undeclared → ONE bound `ASK` (`<cube> cube:observationSet/cube:observation ?o . ?o <dim> ?v`, the bound shape C12.3 permits) answering `true` → accepted, and the answer carries `undeclared_dimensions: [<iri>]` — the numeric Ständemehr is reachable no other way (C2.2); **(c)** the `ASK` answers `false` → `not-found` echoing `{cube, dimension}`, never `invalid-input`, because a request WAS made. | C1.2, C2.2, C12.3 | `describe_cube`, `dimension_values`, `observations` | tests/e2e.rs::the_three_step_dimension_rule_decides_every_filter |
| **P13** | «Not stated» is recognised in all FOUR shapes — the IRI `https://cube.link/Undefined`, an empty literal typed so, an empty PLAIN literal, and an empty literal typed plain `xsd:string` (C3.1 as corrected by §17.2 of the rulebook: only the first two are self-declaring) — and answered as `{"stated": false}`, never as `0`, `""` or a missing key; a STATED lexical zero (`0.0`) survives as a number, because the three states «0.0», «empty» and «Undefined» must never collapse. Every observation answer carries `stated_cells` and `not_stated_cells`. | C3.1, C3.2, §17.2 | `observations` | tests/e2e.rs::both_forms_of_not_stated_are_answered_as_such; tests/e2e.rs::no_stated_cell_ever_carries_an_empty_value; src/domain.rs::both_forms_of_not_stated_are_recognised |
| **P14** | Numbers are served as the lexical form the store holds (`69.91`, `15.5`, `0.5`): no float arithmetic, no rounding, no locale. A declared `qudt:hasUnit` travels with the number. A date carries its lexical form AND its datatype, because one dimension can hold three. | C3.3, C3.4, C3.5 | `observations` | src/domain.rs::a_decimal_is_served_as_the_store_holds_it; tests/e2e.rs::the_staendemehr_is_read_and_never_derived |
| **P15** | Every observation answer says which region it read (`region` with its IRI and label, `country/CHE` for the national row) and caps its rows with the original size (`limit`, `returned`, `total`, `truncated`). | C8.1, C0.2, C6.1, C6.4 | `observations` | tests/e2e.rs::the_staendemehr_is_read_and_never_derived |
| **P16** | Aggregate group keys are folded in the server before they are served: this endpoint can answer the same key twice (measured on `dcat:theme`, 12 + 9 for one identical pair). | C7.5 | backend (§8) | tests/e2e.rs::an_aggregate_key_that_repeats_is_folded |
| **P17** | An observation IRI is served as a citable address that dereferences on its own host. Nothing is ever derived from its slug. | C1.3, C13.2 | `observations`, `describe` | tests/e2e.rs::describe_answers_from_the_one_endpoint_and_caps_its_rows |
| **P18** | There is no link between cubes. `describe_cube` may name related cubes ONLY as «cubes that share a value vocabulary», with `basis: "shared values"` in the answer; it never claims a link the data does not carry. | C7.2 | `describe_cube` | deferred: v0 answers no related cubes at all, which is the honest form of «never claim a link the data does not carry»; the field is written when a caller asks for one |
| **P19** | A ballot outcome is READ from `ergebnis` with its label, never derived from the yes share — 95 of the recorded rows are neither accepted nor rejected. | C8.2 | `observations` (domain point) | tests/e2e.rs::the_staendemehr_is_read_and_never_derived |
| **P20** | The Ständemehr is READ: `standesstimmenJa`/`standesstimmenNein` (decimals), the eight `kantone…Ganze/Halbe` counters and `staendeJaText`/`staendeNeinText`. No tool counts cantons to obtain it, and no tool sums the cantonal rows: counting gives 17 where the Chancellery states 15.5. | C8.3 | `observations` (domain point) | tests/e2e.rs::the_staendemehr_is_read_and_never_derived |
| **P21** | A ballot is filtered by the `typologie` IRI, never by a word in its title; a title is an ENTITY, so a title search is a label lookup over `abstimmungstitel` values and says so. | C8.4, C8.5 | `observations`, `dimension_values` | tests/e2e.rs::the_referendum_and_initiative_families_answer_from_their_cubes |
| **P22** | The Vorlage ↔ Geschäftsstände relation is offered as the literal `id` join it is, and the answer says that one `id` of the states cube has no Vorlage (4'265 of 4'266). The `stand` vocabulary is read per cube, never hard-coded. Two consequences ride on this join: a TITLE never stands in for the `id` — distinct businesses carry byte-identical titles, so a name lookup that matches several hands over the candidates with their `beschlussdatum` and chooses none (C15.5) — and the referendum DEADLINE is a two-cube question, the running flag in the Vorlage cube and the date only in the statistics cube under the deadline `stand` (C15.8): the one error a citizen can act on and lose a right by. | C9.1, C9.2, C15.5, C15.8 | `observations` (+ §7, and the research skill states the join) | tests/e2e.rs::the_referendum_and_initiative_families_answer_from_their_cubes |
| **P23** ⊘ | No join tool, and for a better reason than «it is computed»: **it answers no question of the six families**. `list-results` already carries the list's name, its number and its seats, so «how many seats has list X in canton Y» is one `observations` call (§7 row 6); `seats-per-list` is the APPORTIONMENT TRACE — distribution round, divider, quotient — and 161 of its 219 rows name several lists at once, so joining it to candidates answers «how was the seat computed», which nobody asked. The measured join stays on record for a later tool: 219 of 219 seat rows and 1'129 of 1'129 numbers match a candidate list, and a `lindas.join_lists` would have to answer ONE ROW PER SEAT ROW with `computed: true`, `join_key: "(canton, year, list number)"` and `lists` as an ARRAY — never one row per list. The canton is the shared key; no cross-year person view exists, because a candidate IRI carries its year. | C10.2, C10.3, C6.2, C10.4 | caller sequence, §7 | tests/contract_table.rs::the_eight_tools_are_the_contracts_eight_and_none_joins |
| **P24** | An election year is answered PER CUBE, never per year: 2027 has four empty cubes and two filled ones. | C10.1 | `list_cubes`, `observations` | tests/e2e.rs::list_versions_filters_the_list_and_promises_nothing |
| **P25** | A date after today is data, not a fault: the states cube reaches 2029-01-01. No tool filters the future away. | C9.3 | `observations` | tests/e2e.rs::the_referendum_and_initiative_families_answer_from_their_cubes |
| **P26** | `list_versions` is a filter over the cube list by the last IRI segment, and answers `versioned: false` for the five cubes that carry no version segment. | C1.1, C13.3 | `list_versions` | tests/e2e.rs::list_versions_filters_the_list_and_promises_nothing; src/scope.rs::the_version_is_the_last_segment_when_there_is_one |
| **P27** | No answer ever says «newer»: nothing in the graph links an old version to a new one, and `schema:version` is decoration (always «1», on 12 cubes). | C5.3 | `list_versions` | tests/e2e.rs::list_versions_filters_the_list_and_promises_nothing |
| **P28** | A label is asked for in ONE language with the fallback de → fr → it → en → rm, and the answer names the language it served. | C4.3 | `resolve_label`, all tools | src/domain.rs::a_label_is_chosen_in_the_language_the_store_has |
| **P29** | «All labels» is never fetched: one value of the recorded corpus carries labels in 45 languages, and one cube's values carry 181'228 label rows. | C4.4 | `resolve_label` | tests/e2e.rs::a_label_is_asked_of_the_one_endpoint_whatever_host_the_iri_has |
| **P30** | A label lookup ACCEPTS an IRI of any host a value comes from (`ld.admin.ch/canton`, `…/country`, `…/ech/97/legalforms`, `register.ld.admin.ch/i14y/…`, `fch/apg/vocabulary/…`) and asks the ONE endpoint for it — `<iri> schema:name\|skos:prefLabel\|rdfs:label ?l` over `lindas.admin.ch/query`, subject bound. The server never dereferences a value's host. A value whose labels are not in the store answers `label: null, in_store: false` — an answer, never a fetch elsewhere. Measured: 24'464 German labels over 1'252 foreign value IRIs are readable through the endpoint. | C11.3, C11.2, C1.4 | `resolve_label` | tests/e2e.rs::a_label_is_asked_of_the_one_endpoint_whatever_host_the_iri_has; src/backend.rs::an_iri_is_checked_for_shape_and_not_for_host |
| **P31** | Every query the server sends binds a subject or a predicate. No whole-scope aggregate in one request, no unbound-predicate scan — both were measured not answering inside 120 s. | C12.3 | backend (§8) | tests/contract_table.rs::no_query_of_this_crate_is_unbound; src/domain.rs::a_scope_wide_query_binds_its_subjects |
| **P32** | HTTP 406 from the endpoint maps to a typed refusal of this server, never to «upstream unavailable»: it is a permanent answer about the request. | C12.2 | backend (§8) | src/domain.rs::the_endpoints_refusals_are_typed_apart |
| **P33** | A value that came out of one answer is normalised before it goes into the next request (the endpoint's CSV is CRLF, and a pasted IRI with a trailing `\r` answers 400). | C12.4 | backend (§8) | src/backend.rs::a_value_from_an_answer_is_normalised_before_it_is_reused |
| **P34** | Every query carries the server's own `LIMIT`: the endpoint has no row cap and returned 18'340 rows / 1.5 MB for one unbounded ask. | C12.1 | backend (§8) | tests/e2e.rs::describe_answers_from_the_one_endpoint_and_caps_its_rows |
| **P35** | A vote title that carries the SHAPE of a citation is handed over verbatim with `citation_shape: true` and `resolution: "open"`. No answer claims a resolved act until the fedlex server carries a grammar for a dated act title (§6). | C7.4 | `observations`, gateway | src/domain.rs::a_citation_shape_is_a_shape_and_no_claim |
| **P36** | Where a cube carries `schema:workExample`, the profile serves it as `viewer` (29 of 44); where it does not, the field is absent rather than invented. | C7.3 | `describe_cube` | tests/e2e.rs::describe_cube_serves_the_declared_shape_and_says_it_is_only_that |
| **P37** | Attribution belongs in the manifest, once: publisher, creator and contributor are `ld.admin.ch/FCh` on all 44 cubes. No answer repeats it per row. | C7.1 | manifest (§11) | tests/e2e.rs::attribution_is_named_once_and_never_repeated_per_row |
| **P38** | No licence is stated anywhere on these cubes — measured at every distance the probe looked, and the finding that made the licence a «no» condition of its own. Until the operator answers, the manifest says so and every answer carries `licence: "not stated at the source"`. | C0.1 (the served scope), §2b.1, §2b.5 | manifest, all tools | tests/e2e.rs::list_cubes_serves_the_scope_with_its_states |

**Coverage.** 38 §16 consequences → 38 contract points; one (P23) is
deliberately not a v0 tool and says so. Points without a rule behind
them are in §10.

---

## 2. Cross-cutting contract (all tools)

- **Provenance mandatory.** Every answer carries
  `provenance: {source: "lindas.admin.ch/query", retrieved_at,
  served: "live"|"fixture"|"cache", as_of, licence: "not stated at the
  source", access, endpoint, query}` (P38). A content answer
  additionally carries the cube IRI it read and — where it read one —
  the observation IRI (P17).
- **The answer carries its own question** (31.08.2026). `endpoint` and
  `query` are what make a figure REPRODUCIBLE rather than merely
  attributed: the store holds millions of statements and an answer
  holds twelve, so «source: lindas.admin.ch» names no evidence a
  reader can check. With the pair, one form-encoded POST returns the
  same rows — measured on 31.08.2026 against the live endpoint:
  `find_cube {query: "Volksabstimmung", limit: 2}` answered two cubes,
  and the command built from its own provenance returned the same two
  from a shell with no credentials and no involvement of this
  platform. A fixture carries the query too: what it names is what a
  live run sends, which is what makes a recorded answer auditable
  rather than merely convenient.
- **`kind`** is `"norm"` for an answer read out of an observation or a
  cube's own metadata, `"hint"` for a label hit, a search result or a
  «related cube» suggestion (P18). Discovery is never served as fact.
- **`as_of`** is an optional ISO date on every tool, echoed back
  resolved; absent means «today». The holding is actively maintained
  (`dateModified` mostly within days), so an answer without a moment is
  not an answer. **Built: only the second half** (BY′ point 11) — every
  answer carries `provenance.as_of`, the moment the server was given,
  and NO tool takes the input. §10 point 1 writes the deviation out:
  what it would take to build it, and why building it on an invented
  rule would be worse than not having it.
- **States are answers, not errors** (§4): a published cube with no
  observations, a cube without a status, a name without a language tag
  and a dimension the shape does not declare (but the observations do
  carry) are all ANSWERS, each with the field that names it.
- **Typed refusals**, exactly four, all of them the fedlex server's own
  shapes: `not-found` (echoing its subject — including a dimension the
  cube's observations do not carry, P12(c), because a request was made
  to find that out), `invalid-input` (only for what is decidable WITHOUT
  a request: a malformed IRI, a cube outside §9, an unsupported
  language), `upstream-unavailable` (the endpoint failed) and
  `upstream-busy` (the brake, with `retry_after_ms`). HTTP 406 maps to
  `invalid-input` (P32), never to `upstream-unavailable`. **An
  `invalid-input` about a filter or a projection states the shape it
  WOULD have accepted** (BY point 0): a model that only learns what was
  wrong guesses again, and the first live measurement counted what that
  costs — a refusal per guess, and an `ASK` for every guess that was a
  well-formed IRI.
- **Caps everywhere**: every list answer — `list_cubes`, `find_cube`,
  `describe_cube`, `dimension_values`, `observations`, `list_versions`
  and `describe` — carries `limit`, `returned`, `total` and `truncated`
  with the ORIGINAL size (P8, P15, P34). `describe` is capped too: one
  observation of the widest cube carries 304 predicates (C6.3).
- **Weight** (E11, the gateway's budget): **`2` for all eight tools.**
  Six reach the endpoint for their content; `list_cubes` and
  `list_versions` take the IRIs and the families from §9 but read the
  status and the observation counts live, and a list without those
  answers no question a caller has. The cheaper shape (weight 1, §9
  alone) was rejected because it moves the same cost into 44
  `describe_cube` calls. **For the mount:** the gateway's `call_weight`
  currently knows only the `fedlex.` prefix — mounting this server means
  teaching it `lindas.`, or every call is weighed at 1.
- **Fixture key** (C5, §2b.2): `<tool>:<cube version IRI>:<arguments>`.
  The cube IRI carries its version (C1.1), so a key is stable exactly
  as long as the cube is — a new version is a new key, which is what a
  versioned holding should cost.
- **Language**: `lang` is optional on every tool that serves a label,
  `de` by default, and the answer names the language it actually served
  (P2, P28).

---

## 3. The tools

Eight tools. The proposal that entered this assignment was
`list_cubes, find_cube, describe_cube, dimension_values, observations,
list_versions, describe, probe`; `probe` was struck and `resolve_label`
added — with grounds, at the end of this section.

### 3.1 `lindas.list_cubes` — the served scope, and what each cube says about itself

- **Purpose:** the 44 cubes this server serves, with the names, the
  status and the size a caller needs before asking for anything.
- **Inputs:** `family?` (string, one of `fc`, `political-rights`,
  `national-council-election`, `fch/apg`), `lang?` (string, default
  `de`), `limit?` (int ≤ 100, default 50), `offset?` (int).
- **Outputs:** `cubes: [{cube, name, name_lang, description?,
  description_lang?, family, version?, versioned, status,
  status_label?, status_label_lang?, status_unset, observations,
  placeholder}]`, `limit`, `returned`, `total`, `truncated`,
  `kind: "norm"`, `provenance`.
- **States:** `status_unset: true` for the 14 cubes that carry none,
  `status_label: null` where the vocabulary carries no label in any
  language, `placeholder: true` for the four cubes that hold nothing,
  `name_lang: "und"` for the nine untagged names (P2, P3, P7).
- **Weight:** **2 — it touches the endpoint.** The names, the status
  and the observation counts are read live; only the IRIs and the
  families come from §9. The alternative (weight 1, §9 alone, with
  `describe_cube` carrying the rest) was rejected: a caller that has to
  ask 44 times to learn which cubes hold anything pays 44 × 2 instead
  of one × 2, and E11's cost lens is about what a QUESTION costs, not
  what a call looks like. **For the mount:** the gateway's
  `call_weight` knows only the `fedlex.` prefix today; mounting this
  server means teaching it `lindas.` or it will weigh every call at 1.
- **Fixture key:** `list_cubes:<family|all>:<lang>` — the 44 IRIs are
  static, their metadata is not; the build-time verification against
  the typed query is `list_cubes:verify:<date>`.
- **Example:** `{family: "fch/apg"}` → nine cubes, every one
  `name_lang: "und"`, two with `status_unset: true`, none with
  `placeholder: true`.

- **Built** (BX, `src/domain.rs::list_cubes`): as specified, in TWO
  bound queries per call — the profiles (`list_cubes:<family|all>:<lang>`)
  and the counts (`observation_counts:<family|all>`), because a count is
  an aggregate the metadata query cannot carry without multiplying its
  rows. **Deviation:** the second fixture key `list_cubes:verify:<date>`
  was NOT built — the 44/44 verification pass is the probe's (C14.1), and
  a server that re-verifies its own list would spend requests on what the
  scope list already states; `tests/e2e.rs::the_served_scope_is_the_list_the_contract_names`
  pins the list instead.

### 3.2 `lindas.find_cube` — a cube by a word in its label

- **Purpose:** find the cube behind a question («Abstimmung»,
  «Interessenbindung») without knowing an IRI.
- **Inputs:** `query` (string, required, 2–100 chars), `lang?`
  (default `de`), `limit?` (int ≤ 50, default 10).
- **Outputs:** `hits: [{cube, name, name_lang, family, observations,
  placeholder, score?}]`, `limit`, `returned`, `total`, `truncated`,
  `kind: "hint"`, `provenance`.
- **States:** a hit whose name carried no language tag is served with
  `name_lang: "und"` and never dropped (P2, P7) — the whole `fch/apg`
  register would disappear from a `LANG() = "de"` search.
- **Weight:** 2. **Fixture key:** `find_cube:<query>:<lang>:<limit>`.
- **Example:** `{query: "Interessenbindung"}` → `fch/apg/vested-interest/1`
  («BK-APG Interessenbindungen», `name_lang: "und"`).

- **Built** (`src/domain.rs::find_cube`): as specified — one bound query
  over the served 44 (`VALUES ?cube`), scored in the server, so a hit
  carries the name and the count from the same answer.
- **Built, corrected at BY′ — every WORD, in the same name.** The
  filter was the whole query as ONE contiguous substring, the
  construction `fedlex.search_law` shed at BY point 0, at the ENTRANCE
  of two-stage discovery where a model asks in its own words. Measured
  live — **two requests per call**, the name query and the observation
  counts, so the two runs cost four (BY″): «Kennzahlen
  Volksabstimmungen» found **0** cubes contiguously and **1**
  word-wise — «Kennzahlen zu Volksabstimmungen», the cube the question
  is about. The window is the fixed 44-cube scope either way, so
  word-wise costs nothing upstream; more than twelve words is refused
  before any request, and a search that finds NOTHING costs one
  request, because the counts have nothing to attach to. The query is a
  pure function whose shape is asserted
  (`src/domain.rs::the_find_cube_query_binds_the_scope_and_asks_for_every_word`),
  because the fixture key is blind to the filter — a recorded answer
  alone could not tell the two filters apart.

### 3.3 `lindas.describe_cube` — the declared shape and the profile

- **Purpose:** what a cube says about itself: its metadata and the
  dimensions its SHACL shape declares.
- **Inputs:** `cube` (IRI, required, from the served list), `lang?`,
  `limit?` (int ≤ 100, default 50, over dimensions), `offset?`.
- **Outputs:** `cube`, `name`, `name_lang`, `description?`,
  `description_lang?`, `publisher`, `dates: {created, published,
  modified, granularity}`, `status`, `status_label?`, `viewer?`,
  `theme?`, `observations`, `dimensions: [{dimension, name?, name_lang?,
  node_kind, datatype?, scale?, dimension_kind, optional, max_count?,
  enumeration?, unit?}]`, `dimensions_total`, `truncated`,
  `declared_only: true`, `carried_predicates_sample?: [iri]`,
  `sampled: true`, `note`, `related_cubes?: [{cube, basis:
  "shared values", values: [...]}]`, `limit`, `returned`,
  `kind: "norm"`, `provenance`.
- **States:** `dimension_kind: "unknown"` where neither scale nor class
  is declared (158 of 656 properties, P10); `carried_predicates_sample`
  from ONE observation of the cube with `sampled: true` — a HINT that
  shows what the shape does not declare (C14.1 read one observation per
  cube for exactly this), never a claim about the whole cube; the gate
  for using such a dimension is P12's bound `ASK`, not this sample;
  `declared_only` with the note «the observations may carry predicates this shape does not
  declare — measured 14 declared against 51 carried in `popular-vote`»
  (P9); a cube with no shape (the four placeholders) answers
  `dimensions: []`, `placeholder: true` (P3).
- **Weight:** 2. **Fixture key:** `describe_cube:<cube>:<lang>`.
- **Example:** `{cube: ".../political-rights/popular-vote/1"}` → 14
  dimensions, `declared_only: true`, `observations: 18340`.

- **Built** (`src/domain.rs::describe_cube`): in THREE bound queries —
  metadata, shape, and one observation for `carried_predicates_sample`.
  **Deviation 1:** the shape query does not read `sh:in` members; it only
  flags `enumerated: true` per dimension, because one `id` dimension
  enumerates 2'880 members and the members would have made a single
  cube's shape 2'900 rows. The members are `dimension_values`'s answer.
  **Deviation 2:** `related_cubes` is not emitted at all (P18, deferred
  with its reason).

### 3.4 `lindas.dimension_values` — the values a dimension takes

- **Purpose:** the filter vocabulary of one dimension, so a caller can
  ask for «Volksinitiative» or «Kanton Zug» by IRI instead of by text.
- **Inputs:** `cube` (IRI, required), `dimension` (IRI, required, as
  `describe_cube` served it — P12), `lang?`, `limit?` (int ≤ 200,
  default 50), `offset?`.
- **Outputs:** `values: [{value, label?, label_lang?, observations}]`,
  `limit`, `returned`, `total`, `truncated`,
  `source: "enumeration"|"observations"`, `undeclared_dimensions: [iri]`,
  `kind: "hint"`, `provenance`.
- **States:** where the shape declares `sh:in`, the answer is served
  from the enumeration without a query (`source: "enumeration"`, P11);
  a value that is `cube:Undefined` is answered as `{"stated": false}`
  (P13); an UNDECLARED dimension goes through P12's three steps — the
  bound `ASK` decides, `undeclared_dimensions: [<iri>]` when it answers
  `true`, `not-found` echoing `{cube, dimension}` when it answers
  `false`.
- **Weight:** 2. **Fixture key:**
  `dimension_values:<cube>:<dimension>:<lang>:<limit>`.
- **Example:** `{cube: ".../popular-vote/1", dimension: ".../typologie"}`
  → seven values with German labels, `source: "enumeration"` (BX: the
  shape declares `sh:in` for this dimension, so the enumeration answers;
  the line said `observations` before the crate ran it — see **Built**).

- **Built** (`src/domain.rs::dimension_values`): **deviation** — an
  enumerated dimension is not served «without a query». The shape flags
  it, and the members are fetched WITH their labels in one bound query
  (`dimension_values:enum:<cube>:<dimension>:<lang>`), answered as
  `source: "enumeration"`. The alternative would have put 2'880 IRIs into
  every `describe_cube` answer to save one request here.

### 3.5 `lindas.observations` — the rows, filtered, capped, honest

- **Purpose:** the observations of one cube, filtered by dimension
  values, with every value served as the store holds it.
- **Inputs:** `cube` (IRI, required), `filters?` (array of
  `{dimension, value}` — value an IRI or a literal, dimensions as
  `describe_cube` served them, at most **24** for the same reason the
  projection is capped), `dimensions?` (array of IRIs to
  project, at most **24** — every one the shape does not declare costs
  a bound `ASK`, so the list is capped and a longer one is refused
  before any request; default all), `lang?`, `limit?` (int ≤ 200,
  default 50), `offset?`, `as_of?` (**not built** — §2, §10 point 1).
- **Outputs:** `cube`, `version`, `observations: [{observation, cells:
  [{dimension, value, label?, label_lang?, datatype?, stated, form}],
  citation_shape}]` — a cell carries no `unit`: the unit is a property
  of the DIMENSION and `describe_cube` serves it there (BY‴), `dimensions` (the projection served, or `null`),
  `cells_per_row`, `fewer_cells` (the advice, or `null` — see the
  States), `limit`, **`offset`** (the field the advice's own rule keys
  on), `returned`, `total`, `truncated`, `stated_cells`,
  `not_stated_cells`, `undeclared_dimensions: [iri]`, `filters` (the
  pairs actually applied, or `null` — the one argument the envelope
  never used to echo, which made a wrongly-scoped table
  indistinguishable from a rightly-scoped one), `filters_matched_nothing`,
  `placeholder`, `region?` (with the `dimension` it was read from),
  `regions_on_page`, `national_row?` (the country row's observation
  IRI, where a mixed page carries one), `region_state`,
  `citation_shape_read_over`, `note`, `kind: "norm"`, `provenance`.
- **States:** a cell whose value is «not stated» in any of its four
  shapes is `stated: false` with no `value` (P13); a cube that holds
  nothing answers `observations: []`, `placeholder: true` (P3); a
  filter that matches nothing on a cube that holds rows answers
  `filters_matched_nothing: true` with `placeholder: false` — the
  cube's own count decides which of the two states it is, and «your
  filter matched nothing» must never be read as «this cube is empty»
  (audit of 01.09.2026, its highest-ranked defect). One region heads
  the page only when it is the page's ONLY one; a page that spans
  several answers `region: null`, `regions_on_page: n` and a `mixed`
  `region_state`, naming the national row where it carries one
  (C15.6 — business-level facts repeat on all 27 region rows and the
  country row is the one right address). A value from the holding's
  own vocabulary on a region dimension is served as the row's region
  like any other (C15.7). A
  filter or a
  projection on a dimension the shape does not declare goes through
  P12's three steps: declared → no request; undeclared and the bound
  `ASK` says `true` → accepted, and the answer carries
  `undeclared_dimensions: [<iri>]` (the record is the truth, the shape
  is the claim, P9); the `ASK` says `false` → `not-found` echoing
  `{cube, dimension}`.
- **States, continued (BY point 0):** an answer that was NOT projected
  and whose rows carry more cells than the shape declares says what a
  projection would cost — `cells_per_row`, the bytes these rows take,
  the declared dimensions with the bytes THEY would take, and how to
  ask. The advice carries its own warning: the declared set is not
  everything a row holds (C2.2), and the Ständemehr is undeclared, so a
  caller that projects the declared set alone loses the figure P20 is
  about. The key sorts before `observations` on purpose, so a client
  that cuts the payload at a byte count still reads it. **The whole
  advice is carried at `offset 0` and nowhere else** (BY′): measured at
  1'721 bytes of a 14'375-byte answer on the 51-cell vote row, against
  a pointer of 294 bytes measured on a page built for the purpose. The
  three figures are ASSERTED in `tests/e2e.rs` (as `ADVICE_BYTES`,
  `ADVICE_OF_ANSWER_BYTES`, `POINTER_BYTES`) and `tools/check.sh` holds
  this line to them — the sentence they stand in is a copy, and the
  gate is what keeps a copy true. It
  appears where the row carries more cells than the shape DECLARES — a
  count, not a subset test: on `vested-interest` (7 cells carried, 8
  dimensions declared) there is nothing to advise and nothing is said.
- **States, continued (BY′):** a projection takes CELLS away, never
  ROWS. Every observation the page bound is answered; one that carries
  none of the projected dimensions comes back with `cells: []` — what
  the register gives, meaning «does not carry it», not `0` and not
  `stated: false` — so `returned` keeps counting what `total` counts.
  Because P15's region and P35's citation shape are read FROM the
  cells, a projection could have lost them silently: `region_state`
  says which of **four** holds — «read»; «not projected» (a projection
  was given and named no region dimension); «the rows of this page
  carry none» (it was asked for and the P12 gate admitted it, or the
  shape declares it, and no row carries a value); and «no row of this
  page carries a region cell, and the shape declares no region
  dimension» (nothing was projected, so every cell came back). Decided
  from the PROJECTION and the gate, not from the shape, because the
  shape is a subset of the record (C2.2) and «the shape declares none»
  would be a true sentence answering the wrong question. Each arm has
  its test; a fifth for a gate that answers `Absent` was written and
  struck, because both call sites return `not-found` before this line.
  And `citation_shape_read_over`
  says whether the shape was looked for over every cell or only over
  the projected ones, so a `null` is «not seen», never «not there». A
  dimension named twice (as a filter AND as a projection) costs ONE
  `ASK`, not two.
- **What a call costs** (BY″: the report's cost table, now where a
  reader of the contract finds it — each row held by a test that counts
  requests):

  | The call | Requests |
  |---|---:|
  | filters and projections the shape declares | 3 — the shape, the count, the page |
  | one dimension the shape does not declare, admitted by the bound `ASK` | 4 |
  | the same dimension named twice (filtered AND projected) | 4 — it is asked once |
  | a dimension the `ASK` denies | **2** — the shape and the `ASK`, then the refusal, before the count and the page |
  | a name that is no IRI, a list over its cap, a language that is not one of the five | **0** — decidable without asking the store anything; all three are in the loop that counts |

- **Domain points that live here:** P19 (read `ergebnis`), P20 (read
  the Ständemehr — the server has no arithmetic across observations),
  P21 (filter by `typologie` IRI), P22 (the `id` join and the
  `stand` vocabulary), P25 (future dates), P35 (a title with the
  citation shape).
- **Weight:** 2. **Fixture key:**
  `observations:<cube>:<filters, sorted, joined by ;>:<limit>:<offset>`
  — and, when a projection is given (it changes the query),
  `observations:<cube>:<filters>:dims=<projection, sorted, joined by
  ,>:<limit>:<offset>`. The segment appears only when there IS a
  projection, so every answer recorded before projections existed keeps
  its key.
- **Example:** `{cube: ".../popular-vote/1", filters: [{dimension:
  ".../date", value: "1971-02-07"}, {dimension: ".../region", value:
  "https://ld.admin.ch/country/CHE"}]}` → one observation, 51 cells,
  `standesstimmenJa: 15.5` READ (P20), `not_stated_cells: 14`.

- **Built, corrected at BY′ — a projection may not drop a ROW.** The
  `VALUES ?p { … }` stood in front of the REQUIRED `?obs ?p ?v .`, so
  an observation carrying none of the named predicates bound nothing
  and vanished from the page: `returned` counted survivors while
  `total` came from the unprojected count, and no field said so. Under
  a projection the cells now hang OPTIONALLY on the row the inner
  `SELECT` bound, with the label join INSIDE that OPTIONAL. Pinned
  twice: a built page whose middle row carries none of the projected
  cells comes back with all three rows
  (`tests/e2e.rs::a_projection_takes_cells_away_never_rows`), and the
  query is built by a pure function whose shape is asserted
  (`src/domain.rs::the_projection_binds_its_predicates_and_never_drops_a_row`)
  — deleting the projection from the query left all 52 tests green
  before that, because the fixture backends answer by key and never
  read the SPARQL.
- **Built, BY′ — one `ASK` per dimension, and two caps.** The
  projection and the filter loops gated independently, so a dimension
  named in both cost two identical `ASK`s where P12(b) names one; the
  decision is memoised per dimension now. At most **24** dimensions may
  be projected and **24** filters given — every undeclared one costs a
  request, so an uncapped list is a request multiplier the caller
  controls — and each refusal names the shape ITS input has: a filter
  is a `{dimension, value}` pair, a projection a list of bare IRIs,
  `dimension_values` one dimension IRI and no value at all. All pinned
  by tests that count requests: the caps refuse with the number, before
  any request.
- **Built, BY′ — the advice is charged once, and the states a
  projection could have hidden are answered.** `fewer_cells` is the
  whole object at `offset 0` and a pointer on every later page;
  `region_state` and `citation_shape_read_over` are the two states
  above, each with a test — and the «not projected» one is reached by a
  page built for it, because no recorded projection leaves the region
  out.
- **Built, BY point 0 — the projection §3.5 promised.** `dimensions`
  was an input of this contract from the day it was written, and the
  crate took `cube, filters, lang, limit, offset` and no projection: no
  «Built» line said so, and the first live measurement is what found it
  — a model asked for fewer cells, got all 51, and paged the same table
  thirteen times at `limit: 2` against the chat's 24'000-byte cap. It
  is built now: the projection binds the PREDICATE as
  `VALUES ?p { <d1> <d2> … }` inside the page query — the endpoint
  serves the cells the caller named, and C12.3's rule (a bound
  predicate) is kept. **P12's three steps apply to a projected
  dimension exactly as to a filtered one:** declared → no request;
  undeclared and the bound `ASK` says true → served, and the answer
  lists it under `undeclared_dimensions` (so the Ständemehr can be
  projected); the `ASK` says false → `not-found` echoing
  `{cube, dimension}`. Measured on the recorded call: three cells where
  the row carries 51.
- **Built, BY point 0 — the refusal states the shape, and costs
  nothing.** Every check that is decidable without a request now
  happens BEFORE the shape query: a dimension that is no IRI (a short
  name, a prefixed name) and a value that begins like an IRI and is
  none are refused as `invalid-input` carrying
  `accepted: "{dimension: <full IRI as describe_cube served it>, value:
  <IRI or plain literal>}"` and a note naming `dimension_values`. A
  short name costs **no request at all** — not the shape query, and
  never the `ASK` that a well-formed unknown IRI costs. Pinned by a
  test that counts the requests: zero.
- **Built, BY point 0 — the answer says which projection would fit, and
  the default did NOT change.** The measurement offered two ways out:
  make the declared dimensions the default projection, or say which
  projection would fit. The first is refused on this holding's own
  first rule — the shape is a SUBSET of the record (C2.2: 14 declared
  against 51 carried) and the Ständemehr is undeclared, so a default
  projection would silently drop the one figure P20 exists to protect.
  So the answer advises instead: `fewer_cells` carries the row width,
  the bytes these rows take, the declared dimensions with the bytes
  they would take, and the warning that the declared set is not the
  record. What is NOT solved here: the client's byte cap cuts a payload
  the server cannot shape — that is the loop's part (BP part 4), and
  the advice is placed where a cut payload still shows it.
- **Built** (`src/domain.rs::observations`): in TWO bound queries — the
  count and the page — plus P12's `ask_dimension:<cube>:<dimension>` when
  a filter names an undeclared dimension. A LITERAL filter
  is expressed as `?obs <dim> ?vN . FILTER(STR(?vN) = "…")` instead of a
  typed literal in the pattern — once a deviation, now the rule C15.1:
  one date dimension carries three datatypes (C3.5), the election
  numerics are `xsd:int` where a bare number reads as `xsd:integer` and
  matches nothing (§17.4), and the lexical comparison is immune to both
  by construction
  (tests/e2e.rs::a_lexical_filter_is_immune_to_typed_numerics).

### 3.6 `lindas.list_versions` — the versions of a cube family

- **Purpose:** which versions of a cube exist, from the IRI path.
- **Inputs:** `cube` (IRI, required — any version of the family).
- **Outputs:** `family`, `versions: [{cube, version, observations,
  status, status_unset, placeholder}]`, `versioned`, `limit`,
  `returned`, `total`, `truncated`, `note`, `kind: "norm"`,
  `provenance`.
- **States:** `versioned: false` for the five cubes with no version
  segment (P26); no answer claims a «newer» version, and the note says
  why: nothing in the graph links one to another (P27).
- **Weight:** **2 — the same decision as `list_cubes`:** the family and
  the version segments come from §9 without a request, but the counts
  and the status do not, and a version list without them cannot tell a
  caller which year holds anything. A weight-1 variant that answers the
  IRIs alone is possible and was rejected for the same reason: it moves
  the cost to `describe_cube`, once per version.
- **Fixture key:** `list_versions:<family>` (the family is the cube IRI
  without its last segment).
- **Example:** `{cube: ".../national-council-election/candidates/2019"}`
  → three versions, 2027 with `observations: 0`,
  `placeholder: true`.

- **Built** (`src/domain.rs::list_versions`): with NO fixture key of its
  own — the versions come from the served list by the last IRI segment,
  and their states from the same two queries `list_cubes` uses, bound to
  the family (`list_cubes:family:<family>:de`,
  `observation_counts:family:<family>`). No deviation.

### 3.7 `lindas.describe` — one IRI, dereferenced

- **Purpose:** what the holding says about ONE IRI — a cube, an
  observation, a value — for a caller that has an address and wants its
  triples.
- **Inputs:** `iri` (IRI, required — any host: what decides is whether
  the STORE knows it), `lang?`, `limit?` (int ≤ 400, default 100),
  `offset?`.
- **Outputs:** `iri`, `statements: [{dimension, value, datatype?,
  label?, stated, form}]` — the key is `dimension` and not `predicate`
  because a statement is served through the SAME cell shape as an
  observation's cell (BX′: the contract said `predicate`, the code has
  always emitted `dimension`; one shape, one name), `limit`,
  `returned`, `total`, `truncated`,
  `via: "endpoint"`, `kind: "norm"`, `provenance`.
- **States:** `via` is ALWAYS `"endpoint"` — this server has one host
  and never dereferences another (§8); an IRI the store knows nothing
  about is `not-found` echoing it; only a malformed IRI is
  `invalid-input`. The cap is real: one observation of the widest cube
  carries 304 predicates (C6.3).
- **Weight:** 2. **Fixture key:** `describe:<iri>:<limit>:<offset>`.
- **Example:** `{iri: ".../fc/cube-chancellor/observation/corina-casanova"}`
  → 8 statements, `via: "endpoint"` (§2b.4 measured the endpoint's
  `DESCRIBE` as a superset of dereferencing the IRI on its own host;
  the host stays reader-side for the CALLER, P17).

- **Built** (`src/domain.rs::describe`): as a BOUND SELECT `?p ?o` over
  the subject, counted and paged (`describe:count:<iri>`,
  `describe:<iri>:<limit>:<offset>`). **Deviation from the tool's name:**
  SPARQL `DESCRIBE` answers RDF, not SPARQL-JSON bindings — it could
  neither be capped, counted nor paged, and C6.3's 304 predicates make a
  cap the point.
- **Built, corrected at BX′ — the page is a page of STATEMENTS.** The
  `LIMIT`/`OFFSET` sat on the outer pattern, where the label join
  multiplies one statement into up to five rows; measured on the
  recorded page one of `ld.admin.ch/canton/1`, **50 bindings folded to
  43 statements**, so `returned` said 43 where `limit` said 50 and the
  next page began fifty BINDINGS later — the seven statements in
  between were served by no page at all. The bound now sits in a
  `SELECT DISTINCT ?p ?v … ORDER BY ?p ?v LIMIT n OFFSET m` subselect
  with the labels joined outside it, the idiom `observations` already
  used, and the count counts the same DISTINCT statements (C7.5: a
  subject can say the same thing in two of the ten named graphs).
  Pinned by `tests/e2e.rs::describe_pages_by_statement_so_no_statement_falls_between_two_pages`,
  which proves page one followed by page two IS the hundred-statement
  page, in order.
- **Built, decided at BX′ — the 30 s bound is real now.** §8 declares
  two timeout classes and the crate carried both constants, both
  refusal strings and a test asserting all four, but every query ran at
  `SELECT_TIMEOUT`: `describe_class()` was a sentence no code could
  produce, and three documents stated a bound nothing enforced. The
  code is made to match the declaration rather than the declaration
  trimmed to the code — the description PAGE, which reads the wide
  body, now asks through `Backend::describe_within` at
  **`DESCRIBE_TIMEOUT` (30 s)**, while the count (one row) keeps the
  select bound; §8's rule is «30 s for anything that reads a body», and
  splitting them that way is what makes the rule true rather than
  approximately true. The class has a BODY half now
  (`describe_body_class`), because a describe that failed while reading
  its answer used to report «select, timeout 15 s» for a request that
  ran at 30. The measurement is on that side: a description is the
  class where one subject carries 304 predicates (C6.3) and where the
  count is a `DISTINCT` aggregate over everything a subject says.

### 3.8 `lindas.resolve_label` — a label for an IRI, in one language

- **Purpose:** the label of any value the answers hand out — canton,
  country, legal form, gender, committee — whatever host its IRI
  belongs to.
- **Inputs:** `iri` (IRI, required, ANY host), `lang?` (default `de`).
- **Outputs:** `iri`, `label`, `label_lang`, `languages: [tags]`,
  `in_store`, `kind: "hint"`, `provenance`.
- **How:** ONE bound query against the ONE endpoint —
  `<iri> schema:name|skos:prefLabel|rdfs:label ?l` over
  `lindas.admin.ch/query`, subject bound (C12.3). The server does not
  dereference the value's host: a foreign IRI is an IRI the STORE is
  asked about. Measured: 24'464 German labels over 1'252 foreign value
  IRIs are readable that way (C11.2).
- **States:** a value whose labels are not in the store answers
  `label: null`, `label_lang: null`, `in_store: false` — an answer,
  never a fetch elsewhere and never a not-found. A value the store
  knows without a label in any of the five languages answers
  `in_store: true`, `label: null`. The fallback order is
  de → fr → it → en → rm and `label_lang` names the language that
  answered (P28, P29, P30).
- **Weight:** 2. **Fixture key:** `resolve_label:<iri>:<lang>`.
- **Example:** `{iri: "https://ld.admin.ch/canton/1"}` → «Zürich»,
  `label_lang: "de"`, `in_store: true`, `languages: [de, en, fr, it]`
  (BX: the canton was `12`/«Basel-Stadt» before the crate ran it — the
  recorded example is canton 1, and its four languages show that
  `languages` reports what the filter FOUND, not a fixed set).

- **Built** (`src/domain.rs::resolve_label`): as specified, and the
  language filter is IN the query. **Deviation from the draft's shape:**
  «fetch the labels, then choose one» is a request this server must never
  send — one value of the corpus carries labels in 45 languages (C4.4,
  P29) — so the fallback chain de → fr → it → en → rm plus the untagged
  case is expressed in the query itself and the answer names the language
  it found. `languages` therefore lists what that filter found — the
  answer's note says so — not a census of the store.

### What was struck, and what was added

- **⊘ `probe` — struck.** «Is the endpoint reachable» is not a domain
  question and would be a capability id for something the platform
  already has: the L0.8 probe is a MANIFEST field
  (`oh:probe {kind: "sparql-ask", target, expect: "boolean"}`) that the
  registry checker executes. Keeping it as a tool would put the same
  fact in two places and cost a capability id for it (E16 Ziff. 1).
  The manifest declaration is specified in §11 instead.
- **+ `resolve_label` — added.** §16's «Labels» group (P28, P29, P30)
  owns three consequences and had no tool: C11.3 in particular —
  «follow ANY host a value comes from» — is answerable by no other
  tool, and without it the gender of every person in the `fch/apg`
  register answers `null`. The fedlex server carries the same
  capability for the same reason (`fedlex.resolve_vocabulary_label`).
- **Kept as proposed:** `list_cubes`, `find_cube`, `describe_cube`,
  `dimension_values`, `observations`, `list_versions`, `describe`.

---

## 4. States and provenance

Three answer classes, and the states each may carry. A state is never
an error and never an empty success: it is named in the answer.

| Class | When | Carries |
|---|---|---|
| **observation** | a row was read | `cube`, `version`, `observation` (IRI), `date_modified` of the cube, `served: live\|fixture\|cache`, `as_of`, `licence: "not stated at the source"` |
| **hint** | a label hit, a search result, a related-cube suggestion | the same provenance, `kind: "hint"`, and the basis of the suggestion (`basis: "shared values"` for a related cube — P18) |
| **state** | the answer is about the holding rather than about a row | the FIELD that names it (there is no `state` enum — each state is its own field), the rule it comes from, and the figure that makes it real |

The states, each a numbered point. None of them is a value of a `state`
field: a state is a named field, so two of them can be true at once —
a cube may be a placeholder AND carry no status, and four of them are.

1. **`placeholder: true`** — a cube is published, dated and complete in
   its metadata and holds 0 observations (four 2027 election cubes).
   The answer says «published, no observations yet, as of <date>» (P3,
   C5.4, C14.1).
2. **`status_unset: true`** — 14 of 44 cubes carry no
   `creativeWorkStatus` at all: neither Draft nor Published. Where a
   status IS carried, `status` holds the IRI and `status_label` its
   decoded label — `null` where the vocabulary carries none, which is
   an answer too (P3, C5.1, P28).
3. **`name_lang: "und"`** — nine cubes carry a name with no language
   tag; they are served, not dropped (P2, C4.1).
4. **`stated: false`, IRI form** — the value is the IRI
   `https://cube.link/Undefined` (75'532 cells in `popular-vote`)
   (P13, C3.1).
5. **`stated: false`, literal form** — the value is an EMPTY literal
   typed `https://cube.link/Undefined` (246'820 cells over 31
   dimensions). The two forms are carried as ONE state with a
   `form: "iri"|"literal"` field, because no source says whether the
   difference means anything (P13, C3.1, §2c.15).
6. **`undeclared_dimensions: [<iri>]`** — the observations carry a
   predicate the shape does not declare (37 of 51 in `popular-vote`).
   The bound `ASK` of P12(b) decided it exists; the rows are served and
   the answer lists which dimensions were undeclared (P9, P12, C2.2).
7. **`dimension_kind: "unknown"`** — neither scale type nor dimension
   class is declared (158 of 656 properties) (P10, C2.4).
8. **`versioned: false`** — the cube IRI carries no version segment
   (five cubes) (P26, C1.1).
9. **`resolution: "open"`** — a vote title carries the citation shape
   and no server resolves it yet (P35, C7.4, §6).

---

## 5. The catalogue — what the chat model reads to choose

The stage-one lines, in the `tool-inventory.json` form (≤ 160
characters, verb-first, «use for …», the answer class last). The
GERMAN triggers are in the lines on purpose: they are what makes the
model pick this domain over the legal one.

The inventory carries `id`, `domain`, `summary` and `weight` and
nothing else, so the German triggers live IN the lines — there is no
other field for them, and a list beside the lines would be a field the
mount cannot serve.

| id | weight | stage-one line |
|---|---:|---|
| `lindas.list_cubes` | 2 | List the 44 political data cubes of the Confederation (Abstimmungen, Wahlen, Bundesrat, Interessenbindungen): use to see what data exists. norm. |
| `lindas.find_cube` | 2 | Find the cube behind a question by a word of its name («Volksinitiative», «Petition», «Parteienregister»): use before reading rows. hint. |
| `lindas.describe_cube` | 2 | Show a cube's declared dimensions and profile — the record may carry more: use to learn the filters a question needs. norm. |
| `lindas.dimension_values` | 2 | List the values one dimension takes (Kantone, Abstimmungstypen, Geschäftsstände): use to filter by IRI instead of by text. hint. |
| `lindas.observations` | 2 | Read a cube's rows with filters (Abstimmung, Referendum, Volksinitiative, Ständemehr, Kanton, Datum): use for the figures themselves. norm. |
| `lindas.list_versions` | 2 | List the versions of a cube family (Nationalratswahl 2019/2023/2027): use before reading a year; nothing links old to new. norm. |
| `lindas.describe` | 2 | Show everything the holding says about one IRI (a cube, an observation, a Kanton): use to follow an address you were handed. norm. |
| `lindas.resolve_label` | 2 | Resolve an IRI to its label in one language with a fallback (Kanton, Partei, Gremium, Interessenbindung): use to name a value. hint. |

The words that make the chat model reach for THIS domain rather than
the legal one are therefore in the lines above: Abstimmung, Referendum,
Volksinitiative, Ständemehr, Kanton, Bundesrat, Nationalratswahl,
Interessenbindung, Gremium, Parteienregister, Petition,
Geschäftsstände.

---

## 6. The bridge to the fedlex server, honestly

232 of 711 vote titles carry the SHAPE of a legal citation
(«Bundesbeschluss vom 26.09.1952 über …», C7.4). This contract says
exactly this much:

- an observation answer marks such a title `citation_shape: true` and
  serves it **verbatim**, with `resolution: "open"`;
- no answer of this server claims a resolved act, an ELI or an SR
  number for it;
- the condition under which the bridge closes is named: the fedlex
  server needs a grammar for a DATED ACT TITLE (resolution by title +
  date against JOLux). Today `fedlex.parse_reference` reads the last
  capitalised word of such a title as an abbreviation («Landes»,
  «Wahlrechts», «NFA») and answers `unresolved: true` — measured
  offline in that crate
  (`mcp/servers/fedlex/tests/e2e.rs::a_dated_act_title_is_not_a_citation_this_parser_resolves`
  — that server's suite, not this one's);
- when that grammar exists, the bridge becomes a GATEWAY capability
  (L2.3): the chat hands the title to the legal domain. It does not
  become a lindas tool, and this server never calls the other one.

---

## 7. Acceptance table — six question families

Jonathan's six families, each as: the cube it lives in, the tool
sequence, the states that can occur on the way, and the rules that
govern the step. This is what «the contract answers the questions» has
to mean. Family 6 is two questions, because the holding answers them in
two different cubes — and neither needs a join: the seats are READ from
the `list-results` row, never counted from the elected candidates.

| # | Question family | Cube(s) | Tool sequence | States on the way | Rules |
|---|---|---|---|---|---|
| 1 | **Votes per canton, with the Ständemehr** — «Wie haben die Kantone 1971 zum Frauenstimmrecht gestimmt?» | `political-rights/popular-vote/1` | `find_cube` («Abstimmung») → `describe_cube` → `dimension_values`(`region`) → `observations`(filters: `date`, `region`) | `undeclared_dimension` for the Ständemehr fields (the shape declares 14 of 51), `stated: false` for the Initiative/Gegenentwurf columns, `region` national vs cantonal | P9, P13, P15, P19, P20 |
| 2 | **Referendum bills and their outcome** — «Welche Referendumsvorlagen sind 2023 zustande gekommen?» | `political-rights/referendum/1` + `referendum-stat/1` | `find_cube` → `describe_cube` → `observations`(`beschlussdatumJahr`) → `observations`(states cube, filter `id`) | future dates in the states cube (to 2029), one `id` without a Vorlage, `stand` read per cube | P21, P22, P25 |
| 3 | **Volksinitiativen** — «Welche Initiativen sind im Sammelstadium gescheitert?» | `political-rights/popular-initiative/1` + `popular-initiative-stat/1` | `find_cube` → `dimension_values`(`stand`) → `observations`(filter `stand`) → `observations`(Vorlage by `id`) | 23-value `stand` vocabulary read live, `stated: false` on the optional columns | P22, P11, P13 |
| 4 | **Bundesrat by canton and party** — «Welche Bundesrätinnen kamen aus dem Kanton Bern?» | `fc/cube-councillor` | `find_cube` («Bundesrat») → `describe_cube` → `observations`(filter `addressRegion`) → `resolve_label`(`memberOf`, `actor`) | `versioned: false` (the `fc` cubes carry no version segment), labels from a host outside the served prefix | P26, P30, P17 |
| 5 | **Interessenbindungen** — «In welchen Gremien sitzt Person X?» | `fch/apg/vested-interest/1` (+ `person/1`, `committee/1`, `membership/1`) | `find_cube` («Interessenbindung») → `observations`(filter `hasPerson`) → `resolve_label`(`hasCommittee`, `hasFunction`, `legalName`) | `name_lang: "und"` on the cube itself, `no-status`, labels from `fch/apg/vocabulary/*`, eCH and i14y | P2, P3, P7, P30 |
| 6a | **Nationalratswahlen — seats** «Wie viele Sitze hat Liste X im Kanton Y geholt?» — the answer names the LIST row it read; `seats-per-list` is the apportionment trace and never holds a list's seats (C15.2), and a PARTY figure is a declared sum over its lists (C15.3) | `national-council-election/list-results/{2019,2023}` | `list_versions` → `describe_cube` → `dimension_values`(`listName`) → `observations`(`list-results`, filters `hasCanton`, `listName`) | 2027 `placeholder: true`; 20 cantons in the list families against 26 in `candidates` (C6.2) — the six single-seat cantons elect by majority and have no lists | P15, P24, P26 |
| 6b | **Nationalratswahlen — who was elected** «Wer wurde auf Liste X gewählt?» | `national-council-election/candidates/{2019,2023}` | `find_cube` → `observations`(`candidates`, filters `hasCanton`, `hasElectoralList`, `elected = true`) | a candidate IRI carries its year, so no cross-year view (C10.4); `elected` is a literal boolean the row states; the scattered-votes row («Vereinzelte», candidateNumber 0) is not a person and is never listed as one (C15.4) | P15, P24 |

---

## 8. Egress, politeness, timeouts

- **One host for queries:** `https://lindas.admin.ch/query`. The
  manifest declares it as the only egress target; §2b.4 measured the
  endpoint's `DESCRIBE` as a superset of dereferencing the IRI on its
  own host, so one host suffices.
- **IRIs stay reader-side dereferenceable.** The nine `*.ld.admin.ch`
  hosts an answer may hand out are identifiers a READER can follow;
  this server follows only the query endpoint (P17, §2b.4). That holds
  for LABELS too: `resolve_label` accepts an IRI of any host and asks
  the ONE endpoint about it (P30) — a foreign IRI is a subject the
  store is asked about, never a URL this server fetches. `describe`
  answers `via: "endpoint"` and nothing else.
- **The brake** is the fedlex server's: a token bucket at 2 requests
  per second with a burst of 4, a wait of at most 5 s, then a typed
  `upstream-busy` with `retry_after_ms`. It stays until the operator
  answers what rate is acceptable (§2b.3, §11).
- **Timeouts by class**, in the fedlex server's form (J17.5): 15 s for
  a SPARQL select — the caller's budget, not the endpoint's patience —
  and 30 s for anything that reads a body; the refusal names the class
  AND the bound. **Both halves of a bound belong to the same class**
  (BX′): a describe that fails while READING its answer says «SPARQL
  result body (describe, timeout 30 s)», not the select class's
  wording. **The bound is per REQUEST, and one tool call may send
  two:** `describe` sends a count (select class — one row) and a page
  (describe class — the wide body), and the brake may reserve up to 5 s
  before each, so its worst case is (5 + 15) + (5 + 30) = **50 s**
  before a refusal is produced. That is the honest figure; no document
  may call 30 s the budget of the whole call.
- **Every query binds a subject or a predicate** (P31). Two shapes are
  forbidden by measurement: a whole-scope aggregate over all 44 cubes
  and a filter over an unbound predicate — neither answered inside
  120 s. Graph enumeration is forbidden outright (§2.7: 90 s timeout).
- **No file is fetched.** The server reads the endpoint; it does not
  download distributions.

---

## 9. The served scope — a list, not a pattern

The 44 cube IRIs are
[`testing/lindas-probe/cubes.txt`](../../../testing/lindas-probe/cubes.txt),
the answer of the typed listing query on 2026-08-29, by family:

- **`fc`** (5, no version segment): `cube-chancellor`,
  `cube-councillor`, `cube-declined-election`, `cube-department`,
  `cube-president`
- **`fch/apg`** (9, `/1`): `committee`, `committee-canton-statistic`,
  `committee-function-statistic`,
  `committee-gender-language-statistic`,
  `committee-type-department-statistic`, `committee-type-statistic`,
  `membership`, `person`, `vested-interest`
- **`political-rights`** (12, `/1`): `petition`,
  `political-party-register`, `political-party-register-persons`,
  `popular-initiative`, `popular-initiative-keyfigures-stat`,
  `popular-initiative-stat`, `popular-vote`,
  `popular-vote/voting_dates`, `popular-vote-stat`, `referendum`,
  `referendum-keyfigures-stat`, `referendum-stat`
- **`national-council-election`** (18 = 6 × `{2019, 2023, 2027}`):
  `candidates`, `canton-candidate-statistics`,
  `canton-election-statistics`, `list-results`,
  `seats-in-connected-lists`, `seats-per-list`

A build verifies this list against the typed query and fails when a
cube has appeared or vanished (P1). Widening the scope changes this
list — never code (E15).

---

## 10. Points the rules did not demand

Four points below have no rule behind them. Each says why it is here
anyway; nothing else in this contract is without a rule id.

1. **`as_of` on every tool.** No LINDAS rule demands it; the platform's
   bitemporal discipline does (E14, and the fedlex server's own
   contract). Ground: an answer without a moment cannot be compared
   with a later one, and this holding is maintained daily.
   **Built: the stamp, not the input** (BY′ point 11). Every answer
   carries `provenance.as_of` — the moment injected into the server,
   never read from a clock inside it — and no `Params` struct takes an
   `as_of`. The input is NOT built, and the ground is this holding's:
   fedlex can resolve a date because JOLux dates every consolidation
   and says which one governed when; `cube.link` does not. A cube
   version is the temporal unit here, `dateCreated`/`dateModified` are
   the only dates a cube carries (C5.2), **nothing links an old version
   to a new one** (C5.3), and no observation carries a validity span.
   An `as_of` would therefore have to INVENT a rule — «the version
   whose `dateModified` is the latest before your date» is an
   interpretation this holding never states, and a wrong answer with a
   date on it is worse than an honest answer without one. What would
   make it real: a statement from the publisher about which version
   governed when, or a `schema:validFrom`-shaped field in the cubes.
   Until then this line is a deviation, not a plan.
2. **The four typed refusals** in the fedlex shapes. The rules demand
   only that 406 maps to a refusal (P32); the SET of refusals is the
   platform's, so two domain servers answer alike.
3. **`find_cube` as its own tool.** The rules demand that untagged
   names stay findable (P2, P7) but not that search be a separate
   capability. Ground: E16 Ziff. 1 — one capability id per question,
   and «which cube answers this» is a question of its own.
4. **The weights (1 and 2).** E11's cost lens, not a data rule: the
   two tools that answer from this contract's list cost less than the
   six that reach the endpoint.

---

## 11. Open inputs — what must be answered before code

1. **The licence.** No licence is stated on any of the 44 cubes at any
   distance the probe looked (§2b.1), and the LINDAS fair-use page could
   not be read by a plain fetch. **No manifest may be written until the
   operator and the publisher answer** — a manifest states a licence,
   and this one cannot. Until then every answer carries `licence: "not
   stated at the source"` (P38). This is Jonathan's question to send,
   and it is the one thing that keeps this contract from becoming a
   crate.
2. **The acceptable rate.** The brake is the platform's own guess at
   politeness (§2b.3). What the operator accepts is a question to the
   LINDAS Service Desk.
3. **The shape hash's second run** (§2b.2) — a day apart, not a
   request: it tells whether a cube's shape moves under its own IRI.
4. **The manifest fields**, once the licence is answered: publisher
   `ld.admin.ch/FCh` (P37), egress `lindas.admin.ch`, probe
   `{kind: "sparql-ask", target: "…/query", expect: "boolean"}` (the
   struck `probe` tool's job), tier `base`, and the served scope of §9.
5. **Whether the foreign vocabularies carry their labels IN THE STORE.**
   `resolve_label` asks the one endpoint for any IRI (P30), which is
   measured for the 1'252 foreign value IRIs of `vested-interest`
   (C11.2) — but NOT for
   `register.ld.admin.ch/i14y/concept/sex/*` nor for the
   `CreativeWorkStatus` vocabulary that `status_label` decodes (P3).
   One bound `SELECT` each settles it; neither has been run. If a
   vocabulary's labels are not in the store, those answers carry
   `label: null, in_store: false` — which is honest, and thin.
6. **Whether the two `cube:Undefined` forms mean different things.**
   They are carried as one state with a `form` field (§4.5) until a
   source says otherwise.

### Answered since (BX, 2026-08-30 — append, the items above stand as written)

- **Item 1, the licence: no longer a gate.** Jonathan decided at the
  close of BW′, on three polite I14Y requests: the LINDAS data service
  is registered (`dataservice/c9cf11b6-d165-4498-92fc-d51167def66c`,
  publisher Swiss Federal Archives) with `accessRights: PUBLIC`, and the
  I14Y schema for a data SERVICE carries no licence field at all. Access
  is settled; the licence stays «not stated at the source» as a field
  value every answer carries (P38). Crate, manifest and mount proceed.
- **Item 5, the foreign vocabularies: MEASURED, and they do carry their
  labels in the store.** Two bound `SELECT`s, recorded as fixtures:
  `register.ld.admin.ch/i14y/concept/sex/2` answers «Weiblich»
  (de/fr/it/en), and `ld.admin.ch/vocabulary/CreativeWorkStatus/Published`
  answers «Publiziert» in German plus an untagged «Published». So
  `resolve_label` and `status_label` are not thin: the one endpoint
  answers for both vocabularies.
- **Item 4, the manifest fields:** written with the mount (BX commit 2);
  P37 is the contract table's second «deferred» until then.
- **Items 2, 3 and 6 stand open** — the acceptable rate is Jonathan's
  question to the Service Desk (a courtesy, not a gate), the shape
  hash's second run is still owed, and the two `cube:Undefined` forms
  are still carried as one state with a `form` field.
