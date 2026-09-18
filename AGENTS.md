# Netplaycraft agent guide

Read docs/architecture.md for implemented boundaries and docs/plan.md for product
intent. Roadmap items are not authorization to expand the current milestone.

- Prefer the smallest design that meets the current need. Do not create placeholder
  crates, universal configuration objects, or speculative adapter machinery.
- Weigh edge cases by likelihood, impact, recoverability, and implementation plus
  maintenance cost. Document tolerated limitations instead of hiding complexity.
- Keep game rules independent of networking, authority placement, durable storage,
  browsers, engines, and runtimes. Adapter dependencies point inward.
- Portable core/netcode must not import WebRTC, WebTransport, Cloudflare, S3, Bevy,
  browser APIs, or Tokio without a demonstrated need and an explicit decision.
- Separate player identity, peer identity, simulation ticks, and journal ordering.
  Never use a diagnostic checksum as authentication or integrity protection.
- Keep time and fault injection deterministic in tests. No sleeps or ambient RNG.
- At a library boundary, inspect its source before guessing about behavior. A
  failed fix indicates an incomplete hypothesis; narrow it before redesigning UX
  or replacing libraries.
- Prefer stable safe Rust, small composable traits, and minimal dependencies.
  Add tracing when runtime integration needs structured events; current diagnostic
  counters should stay directly testable.
- Run cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings,
  cargo test --workspace, and cargo check --workspace --lib --target wasm32-unknown-unknown
  for cross-cutting Rust changes. Update protocol/architecture docs when contracts change.
