# Netplaycraft

A Rust-first, browser-first portable multiplayer state-machine runtime.
Formerly planned as Synctide; the project and crate prefix are **netplaycraft**.

```text
Game = Simulation + Synchronization Strategy + Authority
     + Transport + Persistence + Runtime
```

These parts must remain independently swappable. Topology determines connections,
not authority. Game rules must not depend on a browser, engine, transport, cloud
provider, or storage backend.

## Current milestone

The first prototype runs an alternating-turn counter on two peers in one process.
It has fixed ticks, host validation, encoded command frames, ordered replay,
checksums, and a seeded network with latency, jitter, loss, duplication, reordering,
and disconnects. Four small library crates plus the counter have **zero external
Rust dependencies**. Core and simulation contracts are `no_std`.

```sh
proto use
moon run rust:counter
moon run rust:test
```

The default run uses 80 ms latency, up to 30 ms extra jitter, 3% loss, 2% duplication,
and 2% extra reordering delay. Host and replica reach identical state at tick 120.
The virtual network runs without sleeping. Change conditions through the example's
`run_demo(conditions, seed)` function.

## Workspace

This is a **crates-first monorepo**. Reusable Rust libraries live in `crates/`,
reference applications in `examples/`, and architecture/roadmap material in `docs/`.
Proto manages tool versions; Moon orchestrates workspace tasks; Cargo owns crate
resolution and compilation. Future browser packages and applications join this
repository when their milestones need them.

| Package                       | Responsibility                                                        |
| ----------------------------- | --------------------------------------------------------------------- |
| netplaycraft-core             | Typed ticks, peers, players, sequences, channels, commands, checksums |
| netplaycraft-sim              | Independent Simulation, Snapshotable, Checksummed traits              |
| netplaycraft-transport        | Nonblocking semantic message transport contract                       |
| netplaycraft-transport-memory | Seeded network and two endpoints, explicit virtual time               |
| netplaycraft-counter          | Game rules, example-local authority/replay/protocol, CLI composition  |

The example depends on the four libraries. Simulation and transport depend on core;
the memory adapter depends on transport and core. Portable libraries never depend
on adapters. Authority and netcode stay example-local until another use justifies
extracting a shared strategy.

## Design and roadmap

- [Monorepo tooling and contributor commands](docs/tooling.md)
- [Mission and complete staged plan](docs/plan.md)
- [Architecture, API, and dependency graph](docs/architecture.md)
- [Counter wire protocol](docs/protocol.md)
- [Determinism and verification](docs/determinism.md)
- [Source research: six existing projects](docs/research.md)
- [Simulation capabilities ADR](docs/adr/001-capabilities.md)
- [Transport/runtime ADR](docs/adr/002-transport-runtime.md)
- [Persistence ADR](docs/adr/003-persistence.md)

This is an architectural prototype, not a production room runtime. Its history is
limited to 256 frames and its transport currently supports only unreliable delivery.
Checkpoint/replay tests use memory; they do not establish durable recovery.
WASM libraries compile, but browser execution and cross-runtime determinism have
not been verified. WebRTC, rollback, store backends, lockstep, and Mahjong remain
future milestones. The next proof is the unchanged counter in two browser tabs.

## Development

Install [proto](https://moonrepo.dev/docs/proto/install), then run:

```sh
proto use
moon run rust:verify
```

`rust:verify` runs formatting checks, Clippy, all tests, WASM compilation, and the
counter. CI uses this same task. Tool versions are pinned in `.prototools`; Moon
installs Rust components and the WASM target, and pnpm installs oxfmt for
Markdown, YAML, TOML, and JSON. See [tooling](docs/tooling.md) for individual
tasks and the version update workflow.

All workspace packages currently have publishing disabled. No browser, service,
cloud account, or signaling server is required for this milestone.
