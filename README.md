# oh-mcp-lindas

An MCP server by the association [OpenHelvetia](https://openhelvetia.swiss) over the Confederation's Linked Data Service [LINDAS](https://lindas.admin.ch): the 44 political data cubes under `politics.ld.admin.ch` — popular votes with the cantonal majority, referendum bills, popular initiatives, National Council elections, the Federal Council, the Federal Chancellery's register of vested interests. Eight tools. The server derives nothing: every figure is read from a row and carries its source. Stateless, no account.

The data stays with the Confederation. This repository is the interface.

## Run it

```bash
cargo run --locked --manifest-path mcp/servers/lindas/Cargo.toml -- --help
```

## Test it

Every test answers from recorded fixtures; nothing reaches the network. The few live recording runs are marked `#[ignore]`.

```bash
cargo test --locked --manifest-path mcp/servers/lindas/Cargo.toml
```

## What is in here

| Path | What |
|---|---|
| `mcp/servers/lindas/` | the server: `TOOLSET-v0.md` (the contract, 38 numbered points), `ENGINE.md`, `engine.manifest.json`, sources, tests, fixtures |
| `mcp/servers/common/` | what the platform's domain MCP servers share: the polite brake and the semantic fixture store |
| `testing/lindas-probe/cubes.txt` | the cube list one test reads |

## Where it comes from

Published by the association's publication lane from its corpus at commit `45dad0c` (2026-09-03). The module's card with state, evidence and dependencies: <https://openhelvetia.swiss/en/directory/building-blocks/political-data-engine/>. Its guide: <https://openhelvetia.swiss/en/docs/infrastructure/module-political-data-engine/>.

Issues here are welcome; changes go through the corpus and arrive with the next publication. Security reports, in confidence: security@openhelvetia.swiss.

## Licence

Apache-2.0 (see `LICENSE`, attribution in `NOTICE`).
