# mcp-lindas-politics-oh

> **Kurz auf Deutsch.** Dieser MCP-Server macht die 44 politischen Datenwürfel des Bundes im Linked Data Service LINDAS für KI-Systeme nutzbar: Volksabstimmungen mit Ständemehr, Referendumsvorlagen, Volksinitiativen, Nationalratswahlen, Bundesrat, Interessenbindungen. Acht Werkzeuge, jede Zahl mit ihrer Herkunft. Der Server spricht das Model Context Protocol (MCP) über stdin/stdout und lässt sich mit jedem MCP-Client verbinden. Alle Tests laufen offline gegen aufgezeichnete Antworten. Die Daten bleiben beim Bund; dieses Repository ist die Schnittstelle. Anleitung unten auf Englisch, Schritt für Schritt.

**mcp-lindas-politics-oh** is an MCP server by the association [OpenHelvetia](https://openhelvetia.swiss) over the Confederation's Linked Data Service [LINDAS](https://lindas.admin.ch). It gives an AI system — or any MCP client — eight tools over the 44 political data cubes published under `politics.ld.admin.ch`: popular votes with the cantonal majority, referendum bills, popular initiatives, the National Council elections of 2019, 2023 and 2027, the Federal Council, and the Federal Chancellery's register of vested interests.

**What it is not.** It is not a copy of the data and not a service you have to trust: every answer names the row, the IRI or the text it was read from, and the server derives no figure of its own. It keeps no state, needs no account, and stores nothing about you.

---

## Contents

1. [Before you start](#1-before-you-start)
2. [Get it running in five minutes](#2-get-it-running-in-five-minutes)
3. [Connect an MCP client](#3-connect-an-mcp-client)
4. [Your first call, by hand](#4-your-first-call-by-hand)
5. [The tools](#5-the-tools)
6. [Command-line flags](#6-command-line-flags)
7. [Where the data comes from, and how the server treats it](#7-where-the-data-comes-from-and-how-the-server-treats-it)
8. [What is in this repository](#8-what-is-in-this-repository)
9. [How it is verified](#9-how-it-is-verified)
10. [When something does not work](#10-when-something-does-not-work)
11. [Where this repository comes from](#11-where-this-repository-comes-from)
12. [Contributing, security, licence](#12-contributing-security-licence)

---

## 1. Before you start

You need three things. Nothing else.

| Need | Why | How to get it |
|---|---|---|
| **Rust, stable** (rustc and cargo) | the server is a Rust program and is built from source | <https://rustup.rs> — one command, then open a new terminal and run `cargo --version` |
| **Git** | to clone this repository | macOS: comes with Xcode command-line tools (`xcode-select --install`); Linux: your package manager; Windows: <https://git-scm.com> |
| **About 2 GB of disk and a few minutes** | the first build compiles the dependencies once; later builds take seconds | — |

Network: the **tests and the fixture mode need none**. Only the live mode talks to the Confederation's endpoint.

Operating systems: Linux and macOS are what the association builds on. Windows works in principle (Rust is portable) but is not tested here; use WSL if in doubt.

## 2. Get it running in five minutes

Copy each block into a terminal, one after the other. Every command runs from the folder you cloned into.

**Clone**

```bash
git clone https://github.com/OpenHelvetia/mcp-lindas-politics-oh.git
cd mcp-lindas-politics-oh
```

**Build and run the tests** (offline; the first build takes a few minutes)

```bash
cargo test --locked --manifest-path mcp/servers/lindas/Cargo.toml
```

What you should see at the end of each test binary: a line like `test result: ok. … passed; 0 failed` and no `FAILED`. Tests marked `ignored` are deliberate live recording runs that only the association runs.

**Start the server in fixture mode** (offline, answers from the recorded files)

```bash
cargo run --locked --manifest-path mcp/servers/lindas/Cargo.toml -- --fixtures mcp/servers/lindas/tests/fixtures
```

The server now waits for an MCP client on stdin/stdout. It prints nothing by itself — that is correct. Stop it with Ctrl+C.

**Start the server in live mode** (talks to the Confederation's public endpoint)

```bash
cargo run --locked --manifest-path mcp/servers/lindas/Cargo.toml
```

Live mode is polite by default: at most two upstream requests per second with a burst of four (see §6).

## 3. Connect an MCP client

The server speaks MCP over **stdio**: the client starts the process and talks to it through its input and output. Any MCP client that supports stdio servers works. Two examples.

**Claude Desktop** — add this to `claude_desktop_config.json` (macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`), replacing `/ABSOLUTE/PATH/TO/mcp-lindas-politics-oh` with the folder you cloned into, then restart Claude Desktop:

```json
{
  "mcpServers": {
    "mcp-lindas-politics-oh": {
      "command": "cargo",
      "args": ["run", "--quiet", "--locked", "--manifest-path", "/ABSOLUTE/PATH/TO/mcp-lindas-politics-oh/mcp/servers/lindas/Cargo.toml"]
    }
  }
}
```

For an offline demo add `"--", "--fixtures", "/ABSOLUTE/PATH/TO/mcp-lindas-politics-oh/mcp/servers/lindas/tests/fixtures"` to the `args` list.

**Any stdio-capable client** — the command is the same as in §2; the binary itself lives at `mcp/servers/lindas/target/debug/oh-mcp-lindas` after a build (`target/release/oh-mcp-lindas` after `cargo build --release`).

## 4. Your first call, by hand

You do not need a client to see the server answer. The block below sends three MCP messages over stdin — `initialize`, the `initialized` notification, and a `tools/list` — and prints what comes back. It works offline in fixture mode.

```bash
(printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"hand","version":"0"}}}'; sleep 1; printf '%s\n' '{"jsonrpc":"2.0","method":"notifications/initialized"}' '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'; sleep 2) | cargo run --quiet --locked --manifest-path mcp/servers/lindas/Cargo.toml -- --fixtures mcp/servers/lindas/tests/fixtures
```

You should see two JSON lines: the `initialize` result naming the server, and the `tools/list` result with 8 tools. Then call one:

```bash
(printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"hand","version":"0"}}}'; sleep 1; printf '%s\n' '{"jsonrpc":"2.0","method":"notifications/initialized"}' '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"lindas.list_cubes","arguments":{}}}'; sleep 10) | cargo run --quiet --locked --manifest-path mcp/servers/lindas/Cargo.toml -- --fixtures mcp/servers/lindas/tests/fixtures
```

The answer lists the 44 cubes with their states — including the four that are published but hold no observations yet (`placeholder: true`), which is a state the server reports, never hides.

If the second line does not appear, the server was still answering when the pipe closed: raise the `sleep 10` or use a real client (§3).

## 5. The tools

8 tools. Every name carries the domain prefix, so the same server can stand behind the association's gateway unchanged. Each tool's exact input and output shape is in the contract `mcp/servers/lindas/TOOLSET-v0.md`; the one-line purpose:

| Tool | What it does |
|---|---|
| `lindas.describe` | Show everything the holding says about one IRI (a cube, an observation, a Kanton): use to follow an address you were handed. norm. |
| `lindas.describe_cube` | Show a cube's declared dimensions and profile — the record may carry more: use to learn the filters a question needs. norm. |
| `lindas.dimension_values` | List the values one dimension takes (Kantone, Abstimmungstypen, Geschäftsstände): use to filter by IRI instead of by text. hint. |
| `lindas.find_cube` | Find the cube behind a question by a word of its name («Volksinitiative», «Petition», «Parteienregister»): use before reading rows. hint. |
| `lindas.list_cubes` | List the 44 political data cubes of the Confederation (Abstimmungen, Wahlen, Bundesrat, Interessenbindungen): use to see what data exists. norm. |
| `lindas.list_versions` | List the versions of a cube family (Nationalratswahl 2019/2023/2027): use before reading a year; nothing links old to new. norm. |
| `lindas.observations` | Read a cube's rows with filters (Abstimmung, Referendum, Volksinitiative, Ständemehr, Kanton, Datum): use for the figures themselves. norm. |
| `lindas.resolve_label` | Resolve an IRI to its label in one language with a fallback (Kanton, Partei, Gremium, Interessenbindung): use to name a value. hint. |

## 6. Command-line flags

| Flag | Meaning | Default |
|---|---|---|
| `--fixtures <dir>` | answer from recorded files in `<dir>` instead of the network; the tests use `mcp/servers/lindas/tests/fixtures` | off (live) |
| `--endpoint <url>` | the SPARQL endpoint to talk to in live mode | `https://lindas.admin.ch/query` |
| `--upstream-rate <n>` | polite brake: at most `n` upstream requests per second | `2` |
| `--upstream-burst <n>` | polite brake: how many requests may go out at once before the rate applies | `4` |

There is no `--help`: the binary starts serving the moment it runs, because an MCP client expects exactly that. This table is the reference.

## 7. Where the data comes from, and how the server treats it

The holding is the public SPARQL endpoint `https://lindas.admin.ch/query`, operated by the Swiss Federal Archives; the political cubes are published there by the Federal Chancellery under `politics.ld.admin.ch` in the `cube.link` shape. The server reads them; it hosts nothing.

Four rules the code enforces and the tests pin:

- **Every answer names its source** — the IRI, the row, the version, the element it was read from.
- **Nothing is derived.** The cantonal majority is read from the row (`standesstimmenJa`, `standesstimmenNein`), never counted from the cantons; the outcome is read from the `ergebnis` dimension. For the vote of 07.02.1971 the test reads 15.5 and 6.5 and «14 3/2», not the 17 that counting would give (contract points P19, P20).
- **States are answers, never faults.** A cube with zero observations is `placeholder: true`; a cube without a status is `status_unset: true`; both written forms of «not stated» come through as `stated: false` with the form that was found — never as `0`, never as a dropped row.
- **No state, no account, no memory.** The server keeps nothing between calls and writes nothing to disk.

Licence of the data: what the Confederation publishes under its own terms (<https://www.admin.ch/gov/en/start/terms-and-conditions.html>); the server passes it on and adds nothing.

## 8. What is in this repository

| Path | What |
|---|---|
| `mcp/servers/lindas/` | the server: sources, tests, 81 recorded fixture files, `TOOLSET-v0.md` (the contract, 38 numbered points), `ENGINE.md`, `engine.manifest.json` |
| `mcp/servers/common/` | what the association's domain MCP servers share: the polite brake and the semantic fixture store |
| `docs/reference/lindas-cube-rules.md` | the rulebook: 58 measured rules C0–C14 the contract was derived from |
| `testing/lindas-probe/cubes.txt` | the cube list one test reads |
| `LICENSE`, `NOTICE` | Apache-2.0 and the attributions |

The folder layout mirrors the corpus, so the relative paths inside the crates resolve unchanged.

## 9. How it is verified

The chain is rule → contract → test, and every link is gated. The holding was measured first (`testing/lindas-probe/`, fourteen probes); the measurements became the rulebook (58 rules C0–C14, each with its figure); the rulebook's consequences per tool became the contract `TOOLSET-v0.md` with 38 numbered points; `tests/contract_table.rs` walks those 38 points and fails when one is neither pinned by a running test nor deferred with a reason (37 pinned, 1 deferred). 64 tests run: 22 in the library, 7 in the contract gate, 35 end-to-end against the recorded fixtures; 5 more are the live recording runs, marked `#[ignore]`. The recording pass itself is counted (104 requests over 78 keys) and a test holds that figure.

Run everything yourself with the test command in §2. Nothing in the test suite reaches the network; the live recording runs are marked `#[ignore]` and are the association's job.

## 10. When something does not work

| You see | What it means | What to do |
|---|---|---|
| `error: package … requires rustc 1.xx` or an `edition` error | your Rust is too old | `rustup update stable`, open a new terminal |
| the build fails on the first run with a network error | cargo could not download dependencies | check your connection or proxy; the *tests* are offline, the *first build* is not |
| the server starts and prints nothing | correct — it waits for an MCP client on stdin | connect a client (§3) or use the hand-made call (§4) |
| a tool answers with a refusal naming `retry_after_ms` | the polite brake in live mode | wait that long, or start with a higher `--upstream-rate` if the endpoint's operator allows it |
| a tool answers `not-found` | the act, cube or IRI does not exist at the source | that is an answer, not an error; check the identifier |
| live mode: connection errors | the Confederation's endpoint is unreachable or slow | try again later; fixture mode keeps working offline |

Anything else: open an issue in this repository with the command you ran and the output.

## 11. Where this repository comes from

The association develops all its modules in one corpus, on its own GitLab, where every change runs through a gate (formatting, Clippy without warnings, all tests, seal and drift checks). This repository is **assembled from that corpus** by the publication lane (`tools/publish-module.sh` there): it takes the crate and exactly the files its build and tests need, runs the tests in the assembled tree, and pushes here. Each publication is one commit whose message names the corpus commit.

This copy was published from corpus commit `9a70151` on 2026-09-03.

On the association's website the module has a card with its state, evidence and dependencies — <https://openhelvetia.swiss/en/directory/building-blocks/political-data-engine/> — and a guide: <https://openhelvetia.swiss/en/docs/infrastructure/module-political-data-engine/>. The module is the association's own entry in its directory; the entry page names the endpoint and the probe.

## 12. Contributing, security, licence

- **Issues** here are welcome: a wrong answer, a missing tool, an unclear sentence in this README. Please include the tool call and what came back.
- **Changes** go through the corpus and arrive here with the next publication; a pull request here is read and carried over by hand.
- **Security reports**, in confidence: security@openhelvetia.swiss. The association answers within a working week.
- **Licence:** Apache-2.0 for the association's code (`LICENSE`, attribution in `NOTICE`).
