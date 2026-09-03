//! `oh-mcp-lindas` — the LINDAS `cube.link` domain MCP server, base
//! tier (masterplan L2.7).
//!
//! Built from a contract that was written BEFORE it: every point of
//! `TOOLSET-v0.md` §1 carries the rule id it comes from, every rule
//! carries the figure that measured it, and the figures come from the
//! probe scripts in `testing/lindas-probe/`. The order — data →
//! rulebook → §16 → contract → crate — is the point of this server,
//! and `tests/contract_table.rs` holds the crate against the contract
//! the way `rules_table.rs` holds the fedlex server against its rules.
//!
//! Base tier: stateless queries over the PUBLIC LINDAS SPARQL endpoint
//! of the federal administration — one host, a polite brake, no
//! campaigns. Policy (auth, rate, budget) lives at the L2.3 gateway
//! boundary per E11/E16; this server is pure domain logic. Access to
//! the holding is public per I14Y; no licence is stated at the source,
//! which every answer says.

pub mod backend;
pub mod domain;
pub mod scope;
pub mod server;
