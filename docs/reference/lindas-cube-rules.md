---
title: LINDAS cube.link data rules — the rulebook the contract will be derived from
type: Reference
status: informative (reference); every figure is a measurement of 2026-08-29/30
language: English
updated: 2026-08-30
owner: Verein OpenHelvetia
review-by: 2026-12-31
maintenance: a rule changes only when the data changes; the holding is actively maintained, so the figures age
---

# LINDAS `cube.link` data rules — rule, figure, consequence

<!-- language: English -->

**Provenance.** The method is the author's — script → findings →
rulebook → verification — derived from the Fedlex data-understanding
analyses, 2026-06, adopted by consent (see
[`fedlex-data-rules.md`](fedlex-data-rules.md)). The DATA here is not
the author's: every figure below was measured against
`https://lindas.admin.ch/query` by the scripts in
[`testing/lindas-probe/`](../../testing/lindas-probe/README.md) on
2026-08-29 and 2026-08-30, one polite request at a time, and the
findings they produced are recorded in
[`research-lindas-cube.md` §2c](../explanation/research-lindas-cube.md).
Where a figure was already measured in §2 or §2b it is cited, not
re-measured.

**What this page is for.** The first domain server was built and its
hardest defects were DATA defects — the rules that would have predicted
them existed, and nobody had held the server against them. The second
domain server is not built yet. This page is the rulebook BEFORE the
contract: every rule carries a consequence for a tool, and §16 collects
those consequences, grouped by the tool that will own them. The
contract assignment consumes §16; it does not have to read the rest.

**Scope.** The 44 cubes of `politics.ld.admin.ch` (§2.4 of the research
document chose them and says why). Nothing here describes `cube.link`
in general: it describes what THIS holding does, which is what a served
scope has to survive.

## 1. How to read an entry

Every entry carries three things: the **statement** (what the data
does), the **figure** it rests on (measured, with the script that
measured it), and the **consequence for a tool** (what a tool must
therefore do). A rule with no consequence is not a rule and is not
here.

---

## 2. C0 — scope and families

### C0.1 — Forty-four cubes in four families

The served scope holds 44 cubes in four families: `fc` (5 — who held
which federal office), `political-rights` (12 — votes, initiatives,
referendums, petitions, the party register), `national-council-election`
(18 — six families × three election years) and `fch/apg` (9 — the
Federal Chancellery's register of interest ties, committees and
persons).

**Figure:** 44 cubes; 92'688 observations over the 40 cubes that hold
any (`c1-identifiers.sh`, 2026-08-29). §2.4 estimated «roughly
110'000» from an earlier run; the typed count is the figure to use.

**Consequence for a tool:** the served scope is a LIST of 44 IRIs, not
a pattern. Widening it later changes a list, not code — and a tool must
never answer «all cubes» from a prefix match it did not verify.

### C0.2 — The sizes are two orders of magnitude apart

The largest cube holds 18'340 observations (`popular-vote`), the
smallest with content 1 (`committee-type-statistic`), and four hold
none at all.

**Figure:** 18'340 / 16'312 / 10'336 / 9'008 for the four largest;
92'688 in total; 4 cubes at 0 (`c1-identifiers.sh`).

**Consequence for a tool:** every answer that lists observations is
capped and says the original size. «Read the cube» is not an operation
this tier offers.

### C0.3 — The four families are four subject vocabularies

`fc` describes people in office, `political-rights` describes ballots,
`national-council-election` describes an election, `fch/apg` describes
interest ties. They share the canton and country vocabularies and
nothing else.

**Figure:** observations by family — `political-rights` 65'865,
`national-council-election` 15'371, `fch/apg` 9'814, `fc` 1'638.

**Consequence for a tool:** one generic tool family over four
vocabularies. A tool that hard-codes a dimension name serves one family
and breaks on the next.

---

## 3. C1 — identifiers

### C1.1 — The version is the last segment of the cube IRI, when there is one

A cube IRI ends in its version: `…/popular-vote/1`,
`…/candidates/2023`. Five cubes carry no version segment at all
(`fc/cube-chancellor` and its four siblings).

**Figure:** 21 cubes end in `/1`, 6 in `/2019`, 6 in `/2023`, 6 in
`/2027`, 5 in nothing (`c1-identifiers.sh`).

**Consequence for a tool:** version discovery is string work on the
IRI — `list_versions` is nearly free (§2.6) — but a tool must not
assume a version segment exists.

### C1.2 — A dimension IRI is NOT built by appending to the cube IRI

The cube is `…/political-rights/popular-vote/1`; its dimensions are
`…/political-rights/popular-vote/region`, `…/date`,
`…/abstimmungstitel` — the version segment is dropped.

**Figure:** measured the hard way: three queries built as
`<cube>/<dimension>` returned 0 rows; the same queries against
`<cube-without-version>/<dimension>` returned 28, 60 and 5 rows
(`c8-popular-votes.sh`, first and second run).

**Consequence for a tool:** a dimension IRI is READ from the shape or
from an observation, never constructed. A tool that builds it answers
an honest-looking empty list.

### C1.3 — An observation IRI is a slug, and it dereferences

Observation IRIs are `<cube>/observation/<slug>` with human-readable
slugs: `corina-casanova`,
`IV_2015_ueli-maurer_johann-n-schneider-ammann`, `2023-1030-UR-0`,
`831-25`.

**Figure:** 40 of 40 cubes with observations answer one
(`c1-identifiers.sh`); 10 of 10 sampled IRIs dereference as
`text/turtle` on their own host (`c13-completeness.sh`).

**Consequence for a tool:** an observation IRI is a stable address a
caller may keep and a tool may hand out. Its slug is NOT parseable —
nothing may be derived from it.

### C1.4 — Dimensions are cube-local, values are shared

74 dimension IRIs occur in more than one cube, and every one of them
stays inside its own family; no dimension IRI comes from another host
(beyond `cube:observedBy` and a handful of `schema:` predicates). The
VALUES are the shared part: `ld.admin.ch/canton` (26),
`ld.admin.ch/country`, `ld.admin.ch/FCh`,
`ld.admin.ch/ech/97/legalforms`,
`register.ld.admin.ch/i14y/concept/sex`.

**Figure:** 74 shared dimension IRIs, `cube:observedBy` in 17 cubes,
`rdf:type` in 5; value hosts measured in `popular-vote`:
`politics.ld.admin.ch` 144'553 cells, `ld.admin.ch/canton` 17'621 over
26 values, `ld.admin.ch/country` 714 over 1
(`c1-identifiers.sh`, `c3-values.sh`).

**Consequence for a tool:** joins happen on VALUES, never on dimension
names. «The same thing in two cubes» means the same value IRI — the
canton is the one key that spans families.

---

## 4. C2 — schema

### C2.1 — The SHACL shape is one row per dimension, and it can be enormous

Every cube with observations carries a `cube:observationConstraint`
shape whose `sh:property` nodes name the dimensions.

**Figure:** 656 property rows over 40 cubes; median 8 dimensions,
minimum 1, maximum **288** (`fch/apg/committee-type-statistic`)
(`c2-schema.sh`).

**Consequence for a tool:** a shape answer is capped and paged like any
other list. 288 dimensions in one answer is a real case, not a
pathology.

### C2.2 — The shape is a SUBSET of what an observation carries

Every cube's observations carry more predicates than its shape
declares — always at least `rdf:type`, and in the political-rights
cubes far more.

**Figure:** shape dimensions against observation predicates, for the
eight largest cubes: `popular-vote` **14 → 51**, `popular-initiative`
25 → 31, `referendum` 20 → 24, `candidates/2023` 11 → 13,
`popular-vote-stat` 12 → 13, `vested-interest` 8 → 10,
`referendum-stat` 6 → 7 (`c6-quantities.sh`).

What the popular-vote shape KEEPS: `abstimmungsNummerBk`,
`abstimmungstitel`, `date`, `ergebnis`, `ergebnisDetail`, `id`,
`jaAnteil`, `neinAnteil`, `observedBy`, `region`, **`staendeJaText`**,
**`staendeNeinText`**, `stimmbeteiligung`, `typologie`. What it LOSES,
among the 37: the NUMERIC Ständemehr — `standesstimmenJa`,
`standesstimmenNein`, `standesstimmenInitiative`,
`standesstimmenGegenentwurf` and the eight canton counters
(`kantone{Ja,Nein,Initiative,Gegenentwurf}{Ganze,Halbe}Standesstimme`)
— together with the raw vote counts (`ja`, `nein`, `gueltig`,
`ungueltig`, `leer`, `stimmberechtigte`, `eingereicht`) and the two
further Stände texts (`staendeInitiativeText`,
`staendeGegenentwurfText`).

**Consequence for a tool:** a read must NOT filter by the shape. The
shape keeps the Ständemehr's TEXT form («14 3/2») and drops every
numeric one, so a shape-filtered answer can show the notation and not
the 15.5 it stands for. A «structure» answer says it is the DECLARED
shape, and that the record carries more.

### C2.3 — The shape does not type its values

`sh:datatype` is the exception, not the rule; the node kind is the
reliable signal.

**Figure:** of 656 properties, `sh:path` 656, `sh:nodeKind` 506
(Literal 406, IRI 100), `qudt:scaleType` 489, a `cube.link` dimension
class 484, `sh:datatype` **40** (`c2-schema.sh`).

**Consequence for a tool:** type a value from what it IS (IRI or
literal, and the literal's own datatype), never from the shape.

### C2.4 — A quarter of the properties declare neither scale nor class

A property node may declare its scale type (`qudt:scaleType`) and its
`cube.link` dimension class — key, measure or attribute. Many declare
neither, and nothing in the data marks that as incomplete.

**Figure:** 158 of 656 property nodes carry neither `qudt:scaleType`
nor a `cube.link` dimension class, spread over 21 cubes; the declared
combinations are RatioScale+Measure 346, NominalScale+Attribute 50,
NominalScale+Key 48, IntervalScale+Key 9, OrdinalScale+Key 8
(`c2-schema.sh`).

**Consequence for a tool:** «kind unknown» is a state the shape answer
carries, exactly like «no status» — it is never guessed from the
datatype.

### C2.5 — Cardinalities and enumerations are declared where they matter

`sh:minCount`, `sh:maxCount`, `sh:in` and `qudt:hasUnit` appear where
the modeller had something to say — an optional dimension, a closed
value list, a unit — and are silent elsewhere.

**Figure:** `sh:minCount` on 178 properties (163 × 1, 15 × 0),
`sh:maxCount` on 178 (all 1), `sh:in` enumerations on 69 properties in
16 cubes, `qudt:hasUnit` on 27 (`c2-schema.sh`).

**Consequence for a tool:** an enumerated dimension can be offered as a
filter list without a query. A dimension with `minCount 0` is optional
and its absence is not a fault.

---

## 5. C3 — values

### C3.1 — «Not stated» is written in TWO shapes

A missing value is explicit, never empty — but it is explicit in two
different ways: as the IRI `https://cube.link/Undefined`, and as an
EMPTY literal whose datatype is `https://cube.link/Undefined`.

**Figure:** in `popular-vote` alone, 75'532 cells carry the IRI and
246'820 carry the typed empty literal, across 31 dimensions
(`standesstimmenInitiative`, `staendeJaText`, `ohneAntwort`, `mmDate`
…) (`c3-values.sh`).

**Consequence for a tool:** BOTH shapes are recognised and answered as
«not stated». Rendering either as `0`, as `""` or as a missing key is
the one-line mistake with a substantive effect (§2.6 finding 2).

### C3.2 — Undefined is the normal case, not the exception

The explicit «not stated» is not reserved for rare gaps: whole
dimensions of the political-rights cubes carry it in every single
observation.

**Figure:** 8 cubes carry Undefined values; in `popular-vote` all
18'340 observations do, in `popular-initiative` all 10'336, in
`referendum` all 9'008 (`c3-values.sh`).

**Consequence for a tool:** an answer that silently drops undefined
cells drops most of the record. The count of stated and not-stated
cells belongs in the answer.

### C3.3 — Numbers are integers and decimals, and the decimal matters

Counts are integers, shares and Standesstimmen are decimals, and the
decimal IS the value — half a canton vote is 0.5, not a rounding
artefact.

**Figure:** literal datatypes in `popular-vote`: `xsd:integer` 254'383,
`cube:Undefined` 246'820, `xsd:decimal` 79'119, `xsd:string` 61'455,
`xsd:date` 18'340, `xsd:dateTime` 123 (`c3-values.sh`).

**Consequence for a tool:** a decimal is answered as it stands
(`69.91`, `15.5`). No float arithmetic, no rounding, no locale — the
Ständemehr is a half-integer and it must survive the answer.

### C3.4 — Units are declared, sparsely

Where a measure has a unit, the shape declares it with
`qudt:hasUnit`; most measures carry none.

**Figure:** `qudt:hasUnit` on 27 properties in 9 cubes: `unit/COUNT`
16, `unit/PERCENT` 9, `unit/UNITLESS` 2 (`c3-values.sh`).

**Consequence for a tool:** where a unit is declared it travels with
the number. A percentage without its unit is a number that invites the
wrong sentence.

### C3.5 — One date dimension can carry three datatypes

A dimension is not typed by its name. The same dimension carries
values of two datatypes in the same cube, and a «date» may be a date, a
dateTime, a year or an empty string.

**Figure:** in `popular-vote`, `date` is `xsd:date` (18'340 cells)
while `mmDate` is `xsd:string` (18'217, empty) AND `xsd:dateTime`
(123). Years are `xsd:gYear` (`sammelbeginnJahr`,
`beschlussdatumJahr`). The date range across the political-rights
cubes runs 1873-07-07 … 2029-01-01 (`c3-values.sh`,
`c9-referendums.sh`).

**Consequence for a tool:** a date is answered with its lexical form
AND its datatype; a tool must not parse a dimension into one type
because its name looks like a date. A date in the future is data.

---

## 6. C4 — labels and languages

### C4.1 — Nine cubes carry a name WITHOUT a language tag

The `fch/apg` family's cubes are named — «BK-APG Interessenbindungen»,
«BK-APG Personen», «BK-APG Gremien» — but the literals carry no
language tag at all.

**Figure:** 35 of 44 cubes carry `schema:name@de`; the remaining 9
carry an untagged `schema:name`; 0 cubes carry no name
(`c4-labels.sh`). §2.6 recorded this family as «no German name», which
is literally true and reads as «no name» — the measurement is finer.

**Consequence for a tool:** a label lookup that filters
`LANG(?name) = "de"` loses nine cubes. Untagged literals are accepted,
and the answer says the label came without a language.

### C4.2 — A cube name comes in one of five language sets

Not every cube is named in every language, and the sets differ from
cube to cube.

**Figure:** de+fr+it 18 cubes, de+en+fr+it 12, untagged 9, de+en 4,
de+en+fr+it+rm 1 (`c4-labels.sh`).

**Consequence for a tool:** the answer names the language it served,
and falls back rather than refusing.

### C4.3 — Dimension labels are trilingual, English partial, Romansh sparse

Dimension labels are the best-covered layer of the holding: German,
French and Italian are complete across the cubes that carry a shape,
English and Romansh are not.

**Figure:** dimension `schema:name` by language — de 634, fr 634, it
634 (38 cubes each), en 501 (24 cubes), rm 162 (17 cubes)
(`c4-labels.sh`).

**Consequence for a tool:** the label fallback is de → fr → it → en →
rm, and the served language is named — the same discipline the first
server needed for its status labels.

### C4.4 — Value labels are complete, and one value is labelled in forty languages

A value that is an IRI carries its own labels — and a value shared
with a wider vocabulary carries the languages THAT vocabulary has,
which can be far more than this holding needs.

**Figure:** in `popular-vote`, 2'152 IRI values carry de/fr/it labels,
2'151 en, 2'125 rm — and one value (`ld.admin.ch/country/CHE`) carries
a further 40 languages, from `ar` to `zh` (`c4-labels.sh`).

**Consequence for a tool:** a label resolver asks for ONE language with
a fallback. «Fetch all labels» is 181'228 rows for one cube.

### C4.5 — Descriptions are thinner than names, and nine are untagged

A cube may carry `schema:description` beside its name; the coverage is
thinner and shows the same untagged pattern as C4.1.

**Figure:** `schema:description` by language — de 35, fr 25, it 25, en
17, untagged 9, rm 1 (`c4-labels.sh`).

**Consequence for a tool:** a description is optional in the answer and
never a fallback for a missing name.

---

## 7. C5 — status and lifecycle

### C5.1 — In this scope the status vocabulary has ONE value, and «none» is the other state

`schema:creativeWorkStatus` is optional, and where it appears in this
scope it takes exactly one value.

**Figure:** `schema:creativeWorkStatus` =
`CreativeWorkStatus/Published` on 30 cubes; **14 cubes carry none** —
`fch/apg/membership`, `fch/apg/vested-interest` and all twelve
`national-council-election` statistics cubes (`c5-status.sh`).

**Consequence for a tool:** «no status» is a visible state of its own,
exactly like `NeverChecked` in the checker's gradient (§2.6 finding 4)
— never rendered as Draft, never as Published.

### C5.2 — Every cube is dated, in two granularities

Three dates describe a cube's life — created, published, modified —
and the last of them is written in two different granularities.

**Figure:** all 44 carry `schema:dateCreated` and
`schema:datePublished`; `schema:dateModified` is a plain date on 28
cubes and a `dateTime` with milliseconds on 16 (`c5-status.sh`).

**Consequence for a tool:** the answer carries the value as it stands
and says which granularity it is. A comparison between the two forms is
the tool's job, not the caller's.

### C5.3 — Nothing links an old version to a new one

A new version is a new cube IRI; the graph carries no predicate
pointing from the old one to it.

**Figure:** the only version-ish predicate on any of the 44 cubes is
`schema:version` (12 cubes, always the literal «1»); no
`supersededBy`, `isReplacedBy`, `previousVersion` or `expires`
(`c5-status.sh`).

**Consequence for a tool:** `list_versions` reads the IRI path and
nothing else. A tool must not promise «the newer version of this cube»
— it can only offer «the other cubes whose IRI differs in the last
segment».

### C5.4 — Four cubes are published and hold nothing

A cube can be published, dated and complete in its metadata while
holding no observations at all — the election cubes of the next
election year are exactly that.

**Figure:** `candidates/2027`, `list-results/2027`,
`seats-in-connected-lists/2027`, `seats-per-list/2027` — status
Published, 0 observations, and no SHACL shape either (§2.6 finding 1;
verified again in C14 and in §2b.2).

**Consequence for a tool:** the answer is «the cube exists, it is
published, it holds no observations yet, as of <date>» — an ANSWER,
not a not-found and not an error.

---

## 8. C6 — quantities

### C6.1 — The scope is 92'688 observations, unevenly spread

The observations are spread unevenly: four cubes hold more than half
of everything in the scope.

**Figure:** 92'688 over 40 cubes; the top four hold 59 % of them
(`c1-identifiers.sh`).

**Consequence for a tool:** a per-answer cap with the original size
beside it, on every list.

### C6.2 — The canton dimension has 26 values — or 20

The canton dimension resolves to the shared canton vocabulary in every
family — but not every family knows all twenty-six cantons.

**Figure:** `hasCanton` resolves to `ld.admin.ch/canton` in all six
election families, with **26 distinct values in three of them**
(candidates, canton-candidate-statistics, canton-election-statistics)
and **20 in the other three** (list-results, seats-per-list,
seats-in-connected-lists) — the list-based cantons only. The
`popular-vote` cube uses the same 26, each with 689 observations, plus
`country/CHE` with 714 (`c10-elections.sh`, `c6-quantities.sh`).

**Consequence for a tool:** never assume 26. The cardinality of a key
dimension is read per cube, and a cantonal comparison across families
must say which cantons the smaller family knows.

### C6.3 — One observation can carry 304 predicates

The width of ONE observation is as unbounded as the width of a cube:
the statistics cubes write one predicate per counted category.

**Figure:** the widest observation of the scope belongs to
`fch/apg/committee-type-statistic` and carries 304 predicates; the
narrowest carry 5 (`c14-verification.sh`).

**Consequence for a tool:** even a single-observation answer needs a
cap and a «truncated» flag.

### C6.4 — The endpoint has no row cap; the cap must be the tool's

A query without a `LIMIT` is answered in full, however large the
answer is.

**Figure:** `LIMIT 100000` over the largest cube returned all 18'340
rows — 1'514'434 bytes as CSV, 3'201'807 as JSON, in about one second
(`c12-endpoint.sh`).

**Consequence for a tool:** the endpoint will hand over everything it
has. Every query a tool sends carries its own `LIMIT`, and the answer
says what was cut.

---

## 9. C7 — relations

### C7.1 — One publisher for all 44

Publisher, creator and contributor are metadata of the HOLDING rather
than of the individual cube: they are the same on all of them.

**Figure:** `schema:publisher`, `schema:creator` and
`schema:contributor` are `ld.admin.ch/FCh` (the Federal Chancellery) on
all 44 cubes; `schema:contactPoint` is FCh on 32 and a per-cube contact
node on 12 (`c7-relations.sh`).

**Consequence for a tool:** attribution is constant and belongs in the
manifest once, not in every answer. The per-cube contact point is the
exception worth carrying.

### C7.2 — No cube points at another cube

The cubes are islands. No predicate leads from one cube of the scope
to another.

**Figure:** 0 rows for any predicate between two cubes of the scope
(`c7-relations.sh`).

**Consequence for a tool:** there is no navigable graph BETWEEN cubes.
Everything a caller can follow is a shared value (C1.4) or a literal
key (C9.1).

### C7.3 — Half the cubes carry a theme and a viewer link

Beside the metadata every cube must carry, some carry a subject theme
and a link to the LINDAS visualisation application.

**Figure:** `dcat:theme` → `opendataswiss/category/politics` on 21
cubes, `schema:workExample` → `ld.admin.ch/application/visualize` on 29
(`c7-relations.sh`, §2b.1).

**Consequence for a tool:** a «see also» exists for 29 cubes and is
worth answering; the theme is one value and carries no information a
served scope does not already imply.

### C7.4 — A third of the vote titles carry the SHAPE of a citation

A vote title is usually the title of the act the ballot was about, and
it is written the way Swiss legal citation writes one: «Bundesbeschluss
vom 26.09.1952 über die Brotgetreideversorgung des Landes». What is
measured here is that SHAPE — that the title matches the form — and
nothing about whether a tool can resolve it.

**Figure:** 711 distinct vote titles carry a German name; **232 of them
(32.6 %)** match `(Bundesbeschluss|Bundesgesetz|Verordnung|Änderung)
vom <date>` (`c7-relations.sh`, a conditional aggregate over the
titles).

**Consequence for a tool:** the bridge to the first server is a
CANDIDATE, not a capability. `fedlex.parse_reference` has no grammar
for a dated act title: fed three representatives of these 232 it reads
the last capitalised word as an abbreviation — «Landes», «Wahlrechts»,
«NFA» — finds no act for it and answers `kind: unknown`, `act: null`,
`unresolved: true`. That is measured, offline, in the fedlex crate
(`tests/e2e.rs::a_dated_act_title_is_not_a_citation_this_parser_resolves`),
and it is an honest miss rather than a wrong answer. So: the tool hands
the title over verbatim and says the resolution is OPEN. The bridge is
counted when the grammar exists — resolution by title and date against
JOLux, a ranked item for the fedlex server's next wave — and not
before.

### C7.5 — The scope lives in ten named graphs, and an aggregate can repeat a key

The holding is partitioned into named graphs by subject family, and a
plain query reads their union.

**Figure:** the 44 cubes sit in 10 named graphs under
`https://lindas.admin.ch/fch/…`, one per subject family; a
`GROUP BY ?predicate ?value` over the default union answered the
identical `dcat:theme` pair TWICE, with 12 and 9 cubes
(`c7-relations.sh`).

**Consequence for a tool:** a group key from this endpoint is not
guaranteed unique. Every aggregate answer is folded client-side before
it is served, or the caller sees the same row twice.

---

## 10. C8 — popular votes in depth

### C8.1 — An observation is one vote × one region

The Volksabstimmungen cube is not one row per ballot: it is one row
per ballot AND region, with the national row beside the cantonal ones.

**Figure:** `popular-vote` holds 714 national rows
(`region = ld.admin.ch/country/CHE`) and 689 rows for each of the 26
cantons — 18'340 in total (`c8-popular-votes.sh`).

**Consequence for a tool:** every answer says which region it read. A
national figure and a cantonal figure are different observations, never
an aggregate the tool computes.

### C8.2 — The outcome is a labelled IRI, not a derived verdict

The outcome of a ballot is a value of a small vocabulary with its own
labels — not a number a reader has to interpret.

**Figure:** `ergebnis` has five values with German labels — «Die
Vorlage wurde angenommen» (8'661 cells), «Die Vorlage wurde abgelehnt»
(9'584), «Der Abstimmungstermin wurde festgelegt» (6), «Erwahrung in
Vorbereitung» (7), `cube:Undefined` (82) — beside `ergebnisBinary` and
`ergebnisDetail` (`c8-popular-votes.sh`).

**Consequence for a tool:** the outcome is READ and its label served.
Deriving «accepted» from the yes share would be wrong for the 95 rows
that are neither accepted nor rejected.

### C8.3 — The Ständemehr is stated, in three forms, and must never be derived

The Ständemehr is not something a reader computes from the cantonal
rows: the holding states it, in three forms at once — as a decimal, as
whole and half canton counts, and as the Chancellery's own notation.

For the women's-suffrage vote of 07.02.1971 the national row carries
`standesstimmenJa` **15.5** and `standesstimmenNein` **6.5**;
`kantoneJaGanzeStandesstimme` 14 and `kantoneJaHalbeStandesstimme` 3;
`staendeJaText` «14 3/2» and `staendeNeinText` «5 3/2» — the
Chancellery's own notation. Each cantonal row carries its own
contribution (1 or 0 whole votes, 0.5 for a half-canton).

**Figure:** the row measured verbatim (`c8-popular-votes.sh`); a tool
that counted accepting cantons would answer 17 instead of 15.5.

**Consequence for a tool:** read the Stände fields. A tool that
computes a Ständemehr from the cantonal rows is wrong by construction,
because the half-cantons weigh half.

### C8.4 — The typology is a seven-value vocabulary

Which kind of ballot an observation describes is a value of a
seven-entry vocabulary, not a word in its title.

**Figure:** `typologie` — Fakultative Referendumsvorlage 5'751,
Volksinitiative 6'124, Obligatorische Referendumsvorlage 5'054,
Gegenentwurf 1'117, Verfahrensleitender Entscheid 130, Doppeltes Ja 82,
Stichfrage 82 (`c8-popular-votes.sh`).

**Consequence for a tool:** filter by the IRI, never by a word in the
title. `status` is a single value («Aktiv») and carries nothing.

### C8.5 — A vote title is an entity with names in four languages

The title of a ballot is not a string on the observation: it is an
entity the observation points at, carrying its own names.

**Figure:** `abstimmungstitel/101` carries `schema:name` in de, fr, it
and en — plus one untagged empty literal (`c8-popular-votes.sh`).

**Consequence for a tool:** search over titles is a LABEL lookup, not a
text scan (§2.5) — and the empty untagged literal must not become an
answer.

---

## 11. C9 — referendums and initiatives

### C9.1 — The Vorlagen cubes and the Geschäftsstände cubes join on a literal id

The Vorlagen cubes carry the business, the Geschäftsstände cubes its
states over time, and both carry the same business id.

**Figure:** `referendum/id` holds 4'265 distinct ids,
`referendum-stat/id` 4'266, and **4'265 occur in both**
(`c9-referendums.sh`).

**Consequence for a tool:** the join is a literal id, offered as a
capability («the states of this Vorlage»), and the answer says that one
id in the states cube has no Vorlage.

### C9.2 — «Stand» is a per-cube vocabulary with German labels

A «Stand» is a value of a per-cube vocabulary — «Sammelbeginn»,
«Zustandegekommen», «Abgestimmt» — never a free-text status.

**Figure:** `popular-initiative-stat/stand` has 23 values —
Sammelbeginn 554, Vorprüfung 423, Ablauf Sammelfrist 417, Eingereicht
386, Zustandegekommen 377, Botschaft des Bundesrats 356, Beschluss des
Parlaments 316, Abgestimmt 242, Im Sammelstadium gescheitert 149,
«Zurückgezogen, …» in three variants, … (`c9-referendums.sh`).

**Consequence for a tool:** a state answer carries the IRI AND its
label; the vocabulary is per cube, so it is read, never hard-coded.

### C9.3 — The dates reach into the future

The date dimensions do not stop at today: scheduled ballots and
running deadlines are part of the data.

**Figure:** `referendum-stat/datum` runs 1873-07-07 … **2029-01-01**;
`popular-initiative-stat/datum` 1892-05-10 … 2027-12-23;
`referendum/beschlussdatum` 1874-06-17 … 2026-06-19
(`c9-referendums.sh`).

**Consequence for a tool:** a date after today is DATA — a deadline, a
scheduled ballot — and is answered as such. The first server learned
the same lesson with its future consolidations.

---

## 12. C10 — elections 2019/2023/2027

### C10.1 — Six families, three years, four placeholders

The election families exist once per election year, and a year that
has not happened yet is already present with its cubes.

**Figure:** 18 cubes = 6 families × 3 years; the 2027 cubes of
`candidates`, `list-results`, `seats-in-connected-lists` and
`seats-per-list` hold 0 observations and no shape, while
`canton-candidate-statistics/2027` (546) and
`canton-election-statistics/2027` (26) are filled. §2b.2 measured that
the dimension IRIs are version-independent and five of six families are
identical between 2019 and 2023.

**Consequence for a tool:** «the 2027 election» is a partly filled
year, not an empty one. A tool answers per cube, never per year.

### C10.2 — The two families name a list differently, and the join is by NUMBER

The seat cube writes a list as a NUMBER — a literal, and often several
numbers comma-joined («1, 28, 31, 32»), because one row can describe a
Listenverbindung. The candidate cube writes an IRI whose last segment
carries that number:
`…/candidates/2019/electoral-list/2019-ZH-1001-01`. Nothing in the data
links the two forms.

**Figure:** the two sides share no identifier, because only one side
has IRIs at all. Joined on (canton, election year, list number):
**219 distinct seat rows name 1'129 list numbers, and all 1'129 match a
candidate list — 219 of 219 rows join fully, none partly, none not at
all**; 161 of the 219 rows name more than one number
(`c10-elections.sh`: both sides fetched, the join computed locally).

**Consequence for a tool:** the join a caller needs is (canton, year,
list number) — split the seat cube's literal on commas, compare against
the last segment of the candidate list IRI. A tool may offer it, since
it is complete on the recorded holding, but it must say that this is a
NUMBER match the tool computes and not a link the data carries, and it
must answer the Listenverbindung case as what it is: one seat row for
several lists.

### C10.3 — The canton is the one key that spans the families

Every election family names its canton with the same shared
vocabulary, whatever else it calls differently.

**Figure:** all six families use `ld.admin.ch/canton` for `hasCanton`
(C6.2 for the cardinalities) (`c10-elections.sh`).

**Consequence for a tool:** the canton is the join a tool may offer,
and the only one.

### C10.4 — A candidate is identified per year, not per person

A candidate is an IRI of the election year's own cube; the same person
in another year is another IRI.

**Figure:** `hasCandidate` →
`…/candidates/2023/candidate/2023-1030-UR-0` — the year is inside the
IRI (`c10-elections.sh`).

**Consequence for a tool:** «the same candidate in 2019 and 2023» is
not answerable from these cubes. A tool must not offer a cross-year
person view.

---

## 13. C11 — the `fch/apg` family, read

### C11.1 — What the four content cubes actually say

`vested-interest` (4'954 observations) carries an interest tie:
`schema:name` («Zentralamt für Edelmetallkontrolle»),
`schema:legalName` → an eCH legal-form IRI, `hasCommittee` →
`fch/apg/vocabulary/interest-committee/<n>`, `hasFunction` →
`…/interest-function/<n>`, `hasPerson` → `fch/apg/person/<n>`.
`person` (1'515) carries family and given name, `honorificPrefix`,
`birthDate` as a year, `gender` → the i14y register, and
`occupation` in German, French and Italian. `membership` (1'609) and
`committee` (143) tie the two together.

**Figure:** the observations read verbatim (`c11-fch-apg.sh`); §2.6
counted this family and named it a gap («counted, not read»).

**Consequence for a tool:** the family is READABLE and belongs in the
served scope — it is the register of who sits in which federal
committee, which is a political-transparency question, not an
administrative leftover.

### C11.2 — The content is German; only the cube name lacks a tag

The `fch/apg` cubes' CONTENT is labelled like the rest of the holding.
It is only the cubes' own names that carry no language tag.

**Figure:** in `vested-interest`, the IRI values carry 24'464 German
labels over 1'252 distinct values, 19'510 French over 46, 14'862
Italian over 34, 4'954 English over 1 (`c11-fch-apg.sh`).

**Consequence for a tool:** the answer is German-first with the same
fallback as everywhere else — and C4.1's untagged cube name is a
labelling gap, not an emptiness.

### C11.3 — Its values come from three further vocabularies

The family's values reach into vocabularies outside
`politics.ld.admin.ch`: its own `fch/apg/vocabulary/*`, the eCH legal
forms and the i14y interoperability register.

**Figure:** `fch/apg/vocabulary/*` (interest-committee,
interest-function), `ld.admin.ch/ech/97/legalforms`,
`register.ld.admin.ch/i14y/concept/sex` (`c11-fch-apg.sh`).

**Consequence for a tool:** a label resolver follows ANY host it is
handed, not only `politics.ld.admin.ch`. A resolver restricted to the
served prefix answers `null` for the gender of every person.

---

## 14. C12 — endpoint behaviour

### C12.1 — No server-side row cap

The endpoint answers what the query asks for; it does not cap a result
set of its own accord.

**Figure:** `LIMIT 100000` returned 18'340 rows / 1.5 MB CSV / 3.2 MB
JSON in ≈ 1 s (`c12-endpoint.sh`; C6.4).

**Consequence for a tool:** the tool caps, always.

### C12.2 — An unsupported content type is a clean 406

Content negotiation is honoured, and a serialisation that cannot carry
a result set is refused rather than approximated.

**Figure:** a `SELECT` asked with `accept: text/turtle` answers
**HTTP 406**, body «No acceptable file format found.»; CSV, JSON and
`ASK`-JSON answer 200 (`c12-endpoint.sh`).

**Consequence for a tool:** the backend maps 406 to its own typed
refusal instead of reporting «upstream unavailable». `DESCRIBE`
answers turtle (2'693 B) and n-triples (4'388 B) for a cube.

### C12.3 — Two query shapes do not answer inside two minutes

Some query shapes are affordable here and some are not, and the
difference is whether a predicate or a subject is bound.

**Figure:** §2.7 measured graph enumeration at a 90 s timeout. Two more
were measured here: a `GROUP BY ?cube` that counts shape dimensions AND
observation predicates for all 44 cubes at once (no answer in 120 s),
and a `REGEX` over an UNBOUND predicate (`?obs ?dimension ?canton` with
a filter on `?dimension`, no answer in 120 s). Both answer at once when
the predicate is named: 0.31 s for the six named canton dimensions,
2–75 s per cube for the width count (`c6-quantities.sh`,
`c10-elections.sh`).

**Consequence for a tool:** every query a tool sends binds a subject or
a predicate. No tool may offer a shape whose predicate is a variable,
and none may fan a whole-scope aggregate into one query.

### C12.4 — The endpoint answers CSV with CRLF

The endpoint's CSV answers are CRLF-terminated, so a value read out of
one carries a carriage return with it.

**Figure:** an IRI read out of a CSV answer carries a trailing `\r`;
pasted into the next query it produces HTTP 400 («`<Q_IRI_REF>`
expected»). Measured the expensive way: 44 verification queries lost to
it (`c14-verification.sh`, first run).

**Consequence for a tool:** a value that came out of one answer is
normalised before it goes into the next request. This is the second
server's version of the first server's eId normalisation.

---

## 15. C13/C14 — completeness and the verification pass

### C13.1 — Every cube is reachable through the typed query

The cube TYPE is the discovery path of this holding: everything in the
scope is reachable through it.

**Figure:** `?cube a cube:Cube` plus the IRI prefix returns all 44,
in 0.14 s (`c1-identifiers.sh`). Graph enumeration, the alternative,
times out (§2.7).

**Consequence for a tool:** discovery goes over `cube.link` types, and
the served scope is verified against that list at build time.

### C13.2 — Observation IRIs dereference

The IRIs of the holding resolve on their own host, not only inside the
SPARQL endpoint.

**Figure:** 10 of 10 sampled observation IRIs answered
`content-type: text/turtle` with their own triples, 543 B … 43'205 B
(`c13-completeness.sh`).

**Consequence for a tool:** an observation IRI may be handed to a
caller as a citable address; the tool does not have to promise to
resolve it itself.

### C13.3 — Every version is visible in the IRI list

Every version that exists is visible as its own cube IRI in the list —
there is nothing to enumerate beyond it.

**Figure:** 21 × `/1`, 6 × `/2019`, 6 × `/2023`, 6 × `/2027`, 5
without a version segment (`c1-identifiers.sh`; C1.1).

**Consequence for a tool:** `list_versions` is a filter over the cube
list — no query per version.

### C14.1 — The verification pass: 44/44 answered, 40 with an observation

Every cube of the scope answers the same query shape, and the only ones
that answer nothing are the ones that hold nothing.

For every one of the 44 cubes, one query returned one observation with
all of its predicates.

**Figure:** 44 of 44 queries answered HTTP 200; **40 returned an
observation** (5 … 304 predicates, median 11) and **4 returned the
empty state** — exactly the 2027 placeholders of C5.4
(`c14-verification.sh`, 2026-08-30).

**Consequence for a tool:** the scope is fully readable through one
query shape, and the only «empty» is a state the tool already has to
answer. A fixture set for this server can be complete rather than
sampled.

---

## 15b. C15 — what the evaluation's ground truth measured (promoted from §17)

These eight rules began as findings F1–F8 of the phase-5 ground truth
(§17, 2026-09-01). A finding becomes a rule when it gains what every
rule here has — a consequence a tool must carry, a line in §16 and,
where the consequence is code, a point in the contract — and that
promotion happened on 01.09.2026, together with the server changes the
audit of the same day ranked. The measurements themselves stay in §17,
dated; what stands here is the rule each one leaves behind.

### C15.1 — The election numerics are `xsd:int`, and a filter must not care

`electionYear`, `candidateNumber`, `seats` and `votes` are typed
`xsd:int`, not `xsd:integer`. A SPARQL filter written as a bare number
is read as `xsd:integer` and matches nothing — an honest-looking empty
answer, C1.2's failure mode in a datatype.

**Figure:** `?o lr:electionYear 2023` → 0 rows; the same filter as
`"2023"^^xsd:int` → 34 rows (§17.4, measured 2026-09-01).

**Consequence for a tool:** never bind a numeric filter as a typed
number. The served tool compares on the LEXICAL form
(`FILTER(STR(?v) = …)`), which is immune to the datatype by
construction — a property the suite pins so it stays a property
(`tests/e2e.rs::a_lexical_filter_is_immune_to_typed_numerics`).

### C15.2 — `seats-per-list` does not hold a list's seats

The cube whose name reads like the answer to «how many seats did this
list win» is the apportionment-round cube: one row per
Hagenbach-Bischoff round for a whole Listenverbindung, `list` a
comma-joined untyped literal, `seats` the seats of the GROUP. Per-list
seats live only in `list-results`.

**Figure:** a Zürich 2023 row of `seats-per-list` carries `seats` = 14
for a seven-list group; `list-results/2023/ZH/02` carries 8 (§17.5).

**Consequence for a tool:** a caller who reaches `seats-per-list` for
a seats question is answering a different question with a plausible
number. The research skill names `list-results` as the cube that holds
a list's seats and `seats-per-list` as the apportionment trace; a
seats answer must name the LIST row it read.

### C15.3 — A party is not a list

One party fields several lists, and only rows have seats. An answer
about a party is already a statement about several rows.

**Figure:** the SVP fields six lists in Zug 2023 (`listNr` 28–33);
only the Stammliste (28) carries `seats` = 1 (§17.7). In Zug the
sub-lists sum to the Stammliste's figure, so the substitution would be
invisible exactly where a reader checks.

**Consequence for a tool:** a seats answer names the list it read
(`listNr` and `listName`); a party figure is a computed sum the answer
must declare as its own step.

### C15.4 — «Vereinzelte» is recorded as a candidate

The scattered-votes row carries `schema:name` «Vereinzelte»,
`candidateNumber` `"0"^^xsd:int` and real votes, once per canton.

**Figure:** `…/candidates/2023/candidate/2023-1030-UR-0`, votes 305
(§17.8).

**Consequence for a tool:** a candidate listing or a «most votes»
answer must not present this row as a person.

### C15.5 — A title is not a key

Distinct businesses carry byte-identical titles; only the literal `id`
joins the Vorlagen and Geschäftsstände cubes (C9.1).

**Figure:** two referendum businesses titled «Kernenergiegesetz
(KEG)» — id 2562 (beschlussdatum 2003-03-21, erledigt) and id 4115
(2026-06-19, running) (§17.9).

**Consequence for a tool:** resolving a citizen's words through
`schema:name` may land on the wrong business silently. Where more than
one matches, hand over the candidates with their distinguishing
`beschlussdatum` and choose none.

### C15.6 — Business-level facts repeat on every region row

A Vorlage has 27 observations — the country and the 26 cantons — and
its business cells hold the SAME value on all 27. The country row is
the one right address for an unqualified question.

**Figure:** title 498 (Gletscher-Initiative): 27 rows, all
`zurueckgezogen/ja` (§17.10).

**Consequence for a tool:** a page that spans regions is headed by no
single one; the answer says how many regions it spans and names the
country row where it carries one, so a citation of a national fact
lands on `…/CHE-…` and never on a canton
(`tests/e2e.rs::a_page_of_one_region_names_it_and_a_mixed_page_refuses_to_choose`).

### C15.7 — `region` is not always a canton or the country

At least one observation carries a `region` from the holding's own
vocabulary rather than the shared hosts C1.4 names.

**Figure:** `…/popular-initiative/1/observations/MilitarySchool-16`,
`region` = `…/political-rights/vocabulary/MilitarySchool` (§17.11).

**Consequence for a tool:** typing `region` as «canton or Switzerland»
meets a row it cannot place. A vocabulary value on a region dimension
is served as the row's region like any other; only the shared hosts
count as regions on OTHER dimensions.

### C15.8 — The referendum deadline lives in the statistics cube

`referendum/1` carries the flag that a deadline is running and the
parliament's decision date — never the deadline. The date is only in
`referendum-stat/1`, under the `stand` «Ablauf der Referendumsfrist
am», joined on the business `id`.

**Figure:** business 4115: `laufendeReferendumsfrist: ja` in the
Vorlage cube, the date 2026-10-08 only in the stat cube (§17.12) — and
phase 5 measured an exchange that never reached the data at all on
exactly this question.

**Consequence for a tool:** «until when can the referendum be called?»
is a two-cube question. The research skill states the join
(`referendum-stat/1`, the deadline `stand`, joined on `id`); this is
the one error on the platform a citizen can act on and lose a right
by, so the answer must name the dated row it read.

---

## 16. What the contract must carry

Every consequence above, once, grouped by the tool that will own it.
This is the input for the contract assignment; the ids point back at
the rule that measured it.

### The cube list and search

- the served scope is a list of 44 IRIs, verified against the typed
  query at build time (C0.1, C13.1)
- a name may be untagged — a `LANG(?name) = "de"` filter loses nine
  cubes (C4.1); the answer names the language it served (C4.2)
- «no status» is a state of its own beside Published (C5.1); a
  published cube with no observations is an ANSWER, not a not-found
  (C5.4, C14.1)
- families differ in vocabulary; nothing may be hard-coded per
  dimension name (C0.3)
- a cube's description is optional and never a stand-in for its name;
  where it exists the answer says which language it is in (C4.5)
- the three dates of a cube belong in its profile, each with the
  granularity it was written in (C5.2)
- the `fch/apg` cubes belong in the served scope: they are the register
  of who sits in which federal committee, and they are readable (C11.1)
  — their content is labelled like the rest, only their own names are
  untagged (C11.2)

### The shape tool

- cap and page: 288 dimensions in one cube is real (C2.1, C6.3)
- the shape is the DECLARED shape, and the record may carry more —
  say so in the answer (C2.2)
- do not type values from the shape: `sh:datatype` is on 6 % of the
  properties (C2.3); «kind unknown» is a state (C2.4)
- offer `sh:in` enumerations as filter lists and `minCount 0` as
  optional (C2.5)
- never construct a dimension IRI from the cube IRI (C1.2)

### The observation tool

- compare a literal filter on its LEXICAL form, never as a typed
  number — the election numerics are `xsd:int` and a bare number
  matches nothing (C15.1)
- a page that spans regions is headed by no single one: say how many,
  and name the country row where the page carries one (C15.6); a
  vocabulary value on a region dimension is a region like any other
  (C15.7)
- recognise BOTH shapes of «not stated» and answer them as such
  (C3.1); count stated against not-stated (C3.2)
- decimals verbatim, no float arithmetic (C3.3); units travel with the
  number (C3.4); dates carry their lexical form and datatype (C3.5)
- every answer says which region/canton it read (C8.1) and caps its
  rows with the original size (C0.2, C6.1, C6.4)
- fold aggregate group keys client-side — the endpoint may repeat one
  (C7.5)
- an observation IRI is a citable address and its slug means nothing
  (C1.3, C13.2)
- there is no navigable link between cubes: a «related cube» answer may
  only be built from shared values, and must say so (C7.2)

### The domain answers (votes, referendums, elections)

- read the outcome IRI and its label; never derive a verdict (C8.2)
- read the Ständemehr fields; never count cantons (C8.3)
- filter by the typology IRI, not by title text (C8.4); a title is an
  entity with names (C8.5)
- offer the Vorlage ↔ Geschäftsstände join on the literal id, and name
  the one id that has no Vorlage (C9.1); read the Stand vocabulary
  (C9.2)
- the candidate and seat cubes join by (canton, year, list NUMBER) —
  the tool computes that match, says it computed it, and answers the
  Listenverbindung case as one row for several lists (C10.2); the
  canton is the shared key (C10.3, C6.2); no cross-year person view
  (C10.4)
- an election year that has not happened yet is present and partly
  filled — answer per cube, never per year (C10.1)
- future dates are data (C9.3)
- «seats of a list» is `list-results` and never the apportionment-round
  cube whose name suggests it (C15.2); a seats answer names the LIST
  row it read, and a party figure is a declared sum over lists (C15.3)
- the scattered-votes row is not a person (C15.4)
- resolving a citizen's words through a title may land on the wrong
  business: hand over the colliding candidates with their
  `beschlussdatum` and choose none (C15.5)
- the referendum deadline is a two-cube question — the flag in the
  Vorlage cube, the date only in the statistics cube, joined on the
  literal id (C15.8)

### Versions

- `list_versions` is a filter over the cube list, by the last IRI
  segment — and five cubes carry no version segment at all (C1.1,
  C13.3)
- nothing links an old version to a new one (C5.3): the tool must not
  promise «newer»

### Labels

- fallback de → fr → it → en → rm, the served language named (C4.3)
- ask for ONE language: «all labels» is 181'228 rows for one cube
  (C4.4)
- follow any host a value comes from (C11.3, C1.4)

### The backend

- every query binds a subject or a predicate; no whole-scope aggregate
  in one request (C12.3)
- map HTTP 406 to a typed refusal (C12.2)
- normalise a value that came out of an answer before it goes into the
  next request — the CSV is CRLF (C12.4)
- the endpoint has no row cap; the tool's `LIMIT` is the only one
  (C12.1)

### The gateway

- a vote title carries the SHAPE of a citation in 232 of 711 cases —
  hand the title over verbatim and say the resolution is OPEN: today
  `fedlex.parse_reference` reads its last capitalised word as an
  abbreviation and answers `unresolved` (C7.4). The bridge becomes a
  capability when the fedlex server grows a grammar for a dated act
  title; until then no answer may claim it
- a cube that carries `schema:workExample` has a viewer a caller can be
  pointed at, and 29 of 44 do (C7.3)

### The manifest

- one publisher for all 44 cubes (`ld.admin.ch/FCh`) — attribution
  belongs there, not in every answer (C7.1)
- no licence is stated anywhere on these cubes (§2b.1): the manifest
  must say what the operator answers, and until then the answer says
  «licence not stated at the source»

---

## 17. Addendum, 2026-09-01 — what the evaluation's ground truth corrected

**These findings are not yet rules, and are numbered F1–F8 to say so.**
*(Promotion, 2026-09-01, same day, later: all eight WERE promoted — they
stand as C15.1–C15.8 in §15b, each with its §16 line, and the
consequences that are code were built and pinned in the same commit.
The measurements below stay as they were taken; the F-numbers remain
the names the audit and the phase-5 record used.)*
A rule of this book carries a statement, a measured figure, a
consequence for a tool, a line in §16 and a point in the contract that
answers it. What follows has the first three and not the last two:
promoting a finding to a rule means writing its §16 line and its
contract point, which is a decision about
[`TOOLSET-v0.md`](../../mcp/servers/lindas/TOOLSET-v0.md) and not
something an addendum may help itself to. Corrections to EXISTING
rules are in §17.13 and change no numbering.

**Why this is appended and not merged.** Every figure above is a
measurement of 2026-08-29/30 and stays one. Rewriting a measured line
with a later reading destroys the only thing that made it worth
writing: that it can be checked against the day it was taken. So the
corrections live here, dated, naming the rule they touch — the same
discipline `research-lindas-cube.md` follows for its phases.

**Where these came from.** Building CRISP-DM phase 5
([`testing/answer-eval`](../../testing/answer-eval/README.md)) needed
ground truth the platform had no part in producing: twelve answers read
out of `https://lindas.admin.ch/query` by direct SPARQL, across the four
families, one polite request at a time — 50 endpoint requests and 5
dereferences on 2026-09-01, none through any tool of this corpus. The
measurement was asked to stay inside this rulebook and report whatever
contradicted it. It reported a great deal, which is the useful outcome:
the rules were written from a survey of the holding, and this was the
first time they were held against the holding one answer at a time.

### 17.1 C1.3 is wrong about the shape, and right about the prohibition

**Statement.** C1.3 writes an observation IRI as
`<cube>/observation/<slug>` with a human-readable slug. **No family
does this.** Four distinct shapes were measured, in four families:

| family | measured shape | example |
|---|---|---|
| popular votes | `<cube>/observations/<slug>` — **plural** | `…/popular-vote/1/observations/CHE-2240` |
| elections | family segment **doubled**, no `observation` segment, version in the middle | `…/list-results/list-results/2023/ZG/28` |
| party register | `<cube-with-version>/observations/<n>`, slug a bare integer | `…/political-party-register/1/observations/1` |
| `fch/apg` | `<cube-without-version>/<n>`, **no observation segment at all** | `…/fch/apg/vested-interest/13051` |

**Figure.** All four dereference: GET with `accept: text/turtle`
returns 200 on `politics.ld.admin.ch`. So C1.3's *dereference* half
holds everywhere and its *shape* half holds nowhere.

**Consequence for a tool.** Unchanged in force and stronger in reason:
an observation IRI is **read** from `cube:observation`, never built. A
tool that composed the rulebook's pattern would hand out a 404 on every
family. One measurement spent a request proving it — a hand-built
`…/candidates/2023/candidate-result/2023-ZG-28-01` returned nothing.
And the slugs are regular enough to tempt construction (`CHE-2240` is
the national row of vote 2240, `25-2240` its Geneva row), which is
exactly why the prohibition has to be stated as a prohibition rather
than left to look unnecessary.

### 17.2 C3.1 names two shapes of «not stated»; there are four states

**Statement.** C3.1 says a missing value is explicit, never empty, in
two shapes — the IRI `cube:Undefined`, or an empty literal **typed**
`cube:Undefined`. Two more were measured, and a fourth state collapses
with them in any renderer that tests for truthiness:

1. the `cube:Undefined` IRI (C3.1, holds);
2. an empty literal typed `cube:Undefined` (C3.1, holds);
3. an empty **plain** literal, no datatype — `mmDate` and `mmId` on
   both national vote rows measured;
4. an empty literal typed plain `xsd:string` — `pr:fax` on the party
   register's EVP and GRÜNE rows, indistinguishable **by type** from a
   stated value.

And beside them, not a missing value at all: a **stated lexical zero**,
`0.0`, measured on `unterschriftenUngueltigAnteil` of CHE-5/6/7.

**Consequence for a tool.** `stated` must be computed per cell from the
datatype AND the lexical form, and a stated `0.0` must survive as a
number. Only two of the four empty shapes are self-declaring; an
emptiness check written against C3.1 alone reads the other two as
stated-and-empty, and a truthiness check turns a real zero into
«nothing».

### 17.3 Which «not stated» shape appears is a property of neither the row nor the dimension

**Statement.** C3.1's two shapes appear **on the same observation at
once**: on CHE-2240 the IRI shape carries `abstimmungsfrage`,
`titelVolksmund` and `stichfrage` while the typed-empty-literal shape
carries `standesstimmenInitiative`, `ohneAntwort`,
`staendeGegenentwurfText` and four `kantone*` counters. And one
dimension switches between them across rows: `staendeJaText` is the
plain literal `"14 3/2"` on CHE-2240 and `""^^cube:Undefined` on
CHE-6500.

**Consequence for a tool.** Test every cell for every shape. A schema
inferred from one sample row mistypes the next, and C2.3's «type a
value from what it IS» has to be applied cell by cell, not once per
dimension.

### 17.4 Finding F1 — the numerics are `xsd:int`, not `xsd:integer`

**Statement.** In the election families `electionYear`,
`candidateNumber`, `seats` and `votes` are typed `xsd:int`. A filter
written `?o lr:electionYear 2023` — which SPARQL reads as
`xsd:integer` — matches nothing.

**Figure.** 0 rows against 34 for the same query written
`"2023"^^xsd:int`, measured 2026-09-01.

**Consequence for a tool.** Every numeric bind carries the datatype
read from the data. This is C1.2's failure mode one layer down: a
plausible query, a 200, an empty answer, and nothing anywhere saying
the filter never had a chance. C3.3 covers integer-versus-decimal and
does not cover this.

### 17.5 Finding F2 — `seats-per-list` does not hold a list's seats

**Statement.** The cube whose name reads like the answer to «how many
seats did this list win» is the **apportionment-round** cube: one row
per Hagenbach-Bischoff distribution round for a whole
*Listenverbindung*, with `distributionRound`, `divider`, `quotient`,
`list` as a comma-joined untyped literal (`"1, 5, 14, 15, 18, 22, 25"`)
and `seats` the seats of the **group**. Per-list seats live only in
`list-results`.

**Figure.** A Zürich 2023 row of `seats-per-list` carries `seats` = 14
for a seven-list group; `list-results/2023/ZH/02` carries 8.

**Consequence for a tool.** This belongs beside C10.2 as a named trap:
a tool that picks a cube by its name answers a different question with
a number that looks right, and no reader can see the substitution. It
is the most likely proximate cause of the failure this family was
measured after.

### 17.6 C10.2 refined — the join key is rougher than stated

**Statement.** C10.2 gives the join key as (canton, year, list number),
split on commas. Measured further: `list-results/listNr` is an untyped
string **with a leading zero** (`"01"`, `"28"`) and is not always
numeric at all (Aargau carries `"01a"` and `"01b"`); and the two
families shape their electoral-list IRIs differently —
`…/list-results/2023/electoral-list/2023-ZG-28` against
`…/candidates/2023/electoral-list/2023-ZG-1035-28`, with an extra code
before the number. The last segment is therefore not the list number in
both families. `hlv` and `ulv` are untyped strings and are group
identifiers, not counts.

### 17.7 Finding F3 — a party is not a list

**Statement.** Zug 2023 holds 34 list rows for roughly nine parties;
the SVP alone fields `listNr` 28–33, of which only the *Stammliste*
(28) carries `seats` = 1 and five carry 0.

**Consequence for a tool.** A citizen question naming a **party** is
already a question about several rows. The answer must name the list it
read, or say plainly that the party's figure is a sum the reader is
being handed — even where the sub-lists happen to add up to the same
number, which in Zug they do.

### 17.8 Finding F4 — «Vereinzelte» is recorded as a candidate

**Statement.** `…/candidates/2023/candidate/2023-1030-UR-0` carries
`schema:name` «Vereinzelte» with `candidateNumber` `"0"^^xsd:int` and
`votes` `"305"^^xsd:int`, and the pattern repeats per canton.

**Consequence for a tool.** A list of candidates, or a «who got the
most votes» answer, must not present this observation as a person.

### 17.9 Finding F5 — a title is not a key

**Statement.** Two distinct referendum businesses carry the
byte-identical German title «Kernenergiegesetz (KEG)»: id 2562
(`beschlussdatum` 2003-03-21, status *erledigt*) and id 4115
(2026-06-19, *haengig*).

**Consequence for a tool.** C9.1 names the literal id as the join key
and says nothing about collisions. A lookup that resolves a citizen's
words to a business through `schema:name` can land on the wrong one
silently; where more than one matches, the tool hands over the
candidates and their distinguishing dates rather than choosing.

### 17.10 Finding F6 — business-level facts are repeated on every region row

**Statement.** In `popular-initiative/1` a Vorlage has 27 observations
— `ld.admin.ch/country/CHE` plus the 26 cantons — and business-level
cells (`zurueckgezogen`, `status`, `angenommen`, `sammelbeginn`) hold
the SAME value in all 27. Measured for title 498: 27 rows, all
`zurueckgezogen/ja`.

**Consequence for a tool.** An unqualified question has exactly one
right address, the `country/CHE` row. A tool that reports «27 hits», or
counts rows, or lets 26 cantonal rows outvote the national one, is
wrong on a question whose answer it was holding.

### 17.11 Finding F7 — `region` is not always a canton or the country

**Statement.**
`…/popular-initiative/1/observations/MilitarySchool-16` carries
`region` = `…/political-rights/vocabulary/MilitarySchool`, a value from
a `politics.ld.admin.ch` vocabulary rather than from
`ld.admin.ch/canton` or `ld.admin.ch/country`. Found while sampling;
not counted exhaustively.

**Consequence for a tool.** C1.4's two shared value hosts do not
exhaust this dimension. A tool that types `region` as «canton or
Switzerland» will meet a row it cannot place, and must say so rather
than drop it.

### 17.12 Finding F8 — the deadline date is in a different cube from the deadline flag

**Statement.** `referendum/1` carries `laufendeReferendumsfrist` (`ja`
or `cube:Undefined`) and `phase`, but **no date** for the deadline;
`beschlussdatum` is the parliamentary decision. The date lives only in
`referendum-stat/1`, under the `stand` «Ablauf der Referendumsfrist
am».

**Consequence for a tool.** «Until when can the referendum be called?»
cannot be answered from the Vorlage cube. This is the practical shape
of C9.1's join and belongs in the contract as a capability rather than
being rediscovered per question.

### 17.13 Corrections to figures stated elsewhere in this rulebook

- **C8.1** — «689 rows for each of the 26 cantons» is an aggregate over
  the cube, **not a per-vote guarantee**. The 1971 women's-suffrage
  ballot has 25 cantonal rows plus the national one: canton 26 (Jura)
  did not exist. A tool that pads a ballot to 26 regions, or reports «1
  canton missing», is wrong for every ballot before 1979.
- **C8.5** — the empty label it names as untagged is **tagged** here:
  `abstimmungstitel/229` and `/682` carry `schema:name` in de/fr/it/en
  and an empty literal tagged `@rm`. Worse in the vocabularies:
  `ergebnis/1` carries empty names in **both** en and rm, so an English
  label lookup for the outcome of an accepted ballot returns `""`. A
  guard written against «no language tag» misses both. The honest
  answer is «no English label» — never the empty string, and never a
  fallback presented as the label.
- **C4.3** — «dimension labels are the best-covered layer of the
  holding» does not hold for `fch/apg`. Every predicate other than
  `sh:path` on every `sh:property` node of `vested-interest` was asked
  for: **zero rows**. All eight property nodes carry a path and nothing
  else — no name in any language, no `sh:nodeKind`, no
  `qudt:scaleType`. A tool that reaches for a dimension label there
  must have an answer for its absence.
- **C4.1** — confirmed again: `…/fch/apg/vested-interest/1` carries
  `schema:name` «BK-APG Interessenbindungen» with an **empty language
  tag**, while `…/political-party-register/1` carries four tagged
  names. Both are in the same family, so one question family spans both
  naming regimes — a trap for any shared label helper.
- **C2.2** — the shape is not a subset here; it **overlaps**.
  `…/vested-interest/13051` carries 8 predicates, two of which
  (`rdf:type`, `cube:observedBy`) the shape does not declare, while two
  the shape does declare (`schema:validFrom`, `schema:validTo`) are
  absent from the row. C2.2's consequence — never filter a read by the
  shape — survives and is strengthened; its statement should say
  «overlap», not «subset».
- **C3.4** — «units are sparse» is true and is not the whole story.
  Where the unit IS declared it can be the entire answer: the shape of
  `popular-initiative/1` declares for `unterschriftenUngueltigAnteil`
  `qudt:hasUnit unit/PERCENT`, `RatioScale`, `sh:min 0.0`, `sh:max
  100.0` — so `0.52` is 0.52 %, and «52 %» is a hundredfold error one
  keystroke away. In the popular-vote shape exactly three of fourteen
  property rows carry a unit (`jaAnteil`, `neinAnteil`,
  `stimmbeteiligung`, all PERCENT) and `standesstimmenJa` is not in the
  shape at all — so 15.5 Standesstimmen must never be rendered
  «15.5 %». The asymmetry is correct and easy to flatten.
- **C12** — confirmed, since it cost the earlier probe two 400s:
  `DATATYPE()` in a SELECT projection is rejected by this endpoint;
  `BIND(DATATYPE(?v) AS ?dt)` is accepted. Also new: a prefixed name
  with a slash in its local part (`pr:name/1`) inside a `VALUES` block
  is malformed and must be written as a full IRI.

### 17.14 A hazard for the evaluation itself, not for a tool

**Ground truth ages at different rates by question kind.** The twelve
values were read on 2026-09-01. Dates and historical decimals will not
move; **status values will** — 26 referendum businesses stand at
`status/haengig` today and every one of them will read `erledigt` in
time. Any question built on `status`, `phase` or a running `stand` must
be re-measured at grading time, never frozen. The set avoids them for
that reason, and this note is here so the next person building on it
knows the avoidance was deliberate.

Related: `pr:nationalratsmandate` carries 62 for the SVP with **nothing
in the row saying which legislature it counts** — the party register
has no date dimension for it. An answer quoting it must attach the
cube's own `schema:dateModified` rather than a year the reader will
assume, and must not be graded against an election result: the
`national-council-election` family is a different vocabulary (C0.3).
