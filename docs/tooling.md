# Monorepo tooling

Netplaycraft is a crates-first monorepo. Libraries belong in `crates/`, reference
applications in `examples/`, and design documents in `docs/`. Add browser packages
or applications when a milestone actually needs them.

Proto manages tools; Moon orchestrates tasks; Cargo owns the Rust crate dependency
graph, shared lockfile, and incremental compilation. The initial Moon project,
`rust`, spans the Cargo workspace. This keeps dependencies in Cargo.toml and avoids
maintaining a second per-crate graph or running competing Cargo jobs.

```text
.prototools             Rust and Moon version pins
.moon/workspace.yml     Moon project registration
.moon/toolchains.yml    Proto bootstrap pin, Rust components and target
moon.yml                Cargo workspace tasks
Cargo.toml / Cargo.lock Rust workspace and dependencies
rust-toolchain.toml     Rustup/editor configuration; channel synced by Moon
crates/                 Reusable Rust libraries
examples/               Reference applications
```

## Setup and everyday commands

Install [proto](https://moonrepo.dev/docs/proto/install) and ensure its shims/bin
are on PATH as described by its installer. Then, from the repository root:

```sh
proto use
moon run rust:verify
```

Moon provisions the configured Rust components and `wasm32-unknown-unknown` target.
The GitHub workflow uses `moonrepo/setup-toolchain`, ensures the base Rust
components exist, and runs the same verify command. If an existing or partially
installed toolchain is missing Cargo, repair it with
`rustup component add cargo rustc rust-std` from the repository root before running
Moon. The version is selected by rust-toolchain.toml.

| Target | Purpose |
| --- | --- |
| rust:build | Build all workspace members |
| rust:check | Check all workspace targets |
| rust:format | Apply Rust formatting; excluded from automatic CI |
| rust:format-check | Check formatting |
| rust:lint | Clippy across all targets, warnings as errors |
| rust:test | All workspace tests and doctests |
| rust:wasm-check | Compile-check all libraries for browser WASM |
| rust:counter | Run the deterministic two-peer demo |
| rust:verify | Format check, lint, test, WASM check, counter, in sequence |

All dependency-resolving Cargo tasks use `--locked`. Task result caching is disabled
for these small checks so verification always executes; Cargo still reuses its
incremental artifacts. `target/` is not a Moon cached output. Moon-generated
cache/docker directories are ignored by Git.

Cargo remains usable for focused crate work, for example:

```sh
cargo test -p netplaycraft-transport-memory --locked
```

## Updating tools and growing the repo

Change Rust/Moon versions in `.prototools`, run `proto use`, then `moon sync` and
`moon run rust:verify`. Commit any synchronized `rust-toolchain.toml` change.
The proto bootstrap version lives in `.moon/toolchains.yml`, which is read by
both Moon and the CI setup action. Components/targets are configured there and
listed in rust-toolchain.toml so direct Cargo/editor usage also provisions them.
Keep those lists aligned when adding a component or target.

New Rust crates join the Cargo workspace under `crates/`; root Moon tasks already
cover them. Register future non-Rust projects in `.moon/workspace.yml` with their
own tasks and proto-managed tools. Split Rust into per-crate Moon projects only
when independent orchestration brings a concrete benefit.

This follows Moon's [Cargo workspace integration](https://moonrepo.dev/docs/guides/rust/handbook)
and [toolchain configuration](https://moonrepo.dev/docs/config/toolchain).
