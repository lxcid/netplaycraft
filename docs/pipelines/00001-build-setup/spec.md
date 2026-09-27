# Build setup: spec

## Decisions

### D1 — proto and pnpm install every project tool

Binding: every later pipeline that adds a project tool. The browser P2P pipeline for milestone 2 is the first, because it needs WASM build tools.

`.prototools` pins the versions of Moon, Rust, Node, and pnpm. `.moon/toolchains.yml` pins proto itself, for Moon and for CI. Node dev tools are pnpm dev dependencies in the root `package.json`, installed from `pnpm-lock.yaml`. `moon sync` generates `rust-toolchain.toml` from `.prototools`, so the Rust version has one source.

proto installs Rust through rustup, and installs rustup first when it is missing. Moon adds the Rust components and targets that `.moon/toolchains.yml` lists.

proto is the bootstrap, so it cannot install itself. Its own installer installs it locally, and CI's setup action installs it in CI. Moon and that action both read proto's version from `.moon/toolchains.yml`.

A later pipeline is in violation if it needs any other project tool installed another way, such as `cargo install`, `npm install -g`, Homebrew, or proto's npm backend. It is also in violation if it pins a tool version outside `.prototools`, `.moon/toolchains.yml`, `package.json`, and the lockfiles.

What would overturn this: a tool a pipeline needs that neither proto nor pnpm can install at a pinned version.

### D2 — One verify task defines passing

Binding: every later pipeline that adds code or a language project. The browser P2P pipeline for milestone 2 is the first to add a second language.

One Moon task, `verify`, runs every check. Locally and in CI, passing means this task passes. It runs:

- the Rust formatting check (rustfmt)
- the formatting check for Markdown, YAML, TOML, and JSON (oxfmt)
- Clippy on all targets, with warnings denied
- the tests
- the WASM compile check (D5)

GitHub Actions installs the pinned tools and runs `verify` on every pull request and every push to `main`.

A later pipeline is in violation if verify does not check code the pipeline adds, or if CI runs a check that verify does not.

What would overturn this: a required check too slow to run on every change, such as a browser test suite. The split between the fast and slow commands is then recorded as a decision.

### D3 — One Moon project at the repository root

Proposed: awaiting operator approval.

Moon manages the repository as a single project at the root. That project holds every task, including formatting for files that are not Rust.

Cargo already owns the crate graph. A Moon project per crate would duplicate it. Split into one project per language when a second language arrives.

### D4 — The rules crate is `netplaycraft-counter` at `examples/counter`

Proposed: awaiting operator approval.

The crate is a library with no dependencies. It holds the rules from the intent and nothing else:

- Two players alternate turns.
- Each turn increments or decrements a shared `i32` count.
- A move by the wrong player is rejected.
- A move that would overflow the count is rejected. It never wraps or panics.
- A rejected move leaves the state unchanged.

The path matches PR #1's counter. The prototype pipeline then extends this crate instead of moving it. Ticks, checksums, and traits each arrive with the pipeline that needs them.

### D5 — Verify checks that the rules compile for `wasm32-unknown-unknown`

Answers open question 2: yes. Setup installs the target, and verify runs `cargo check` for it on the workspace's libraries.

The project is browser-first, and game rules must not depend on a platform. The check catches a platform-only dependency on the day it is added, not when milestone 2 first builds for the browser. It costs one target install and one check.

The check proves that the rules compile for the browser. It does not prove that they behave identically there.

### D6 — oxfmt formats files that are not Rust, without re-wrapping prose

oxfmt formats Markdown, YAML, TOML, and JSON. Rust uses its own tools: rustfmt formats it, and Clippy lints it. Markdown keeps `proseWrap: "preserve"`, so oxfmt never re-wraps prose. This follows what the operator asked when oxfmt was first set up in PR #1.

Files that devloop manages are formatted like every other file. Formatting pads the table in `docs/pipelines/README.md`, so its devloop marker must record the hash of the formatted reference body. That is the baseline devloop's adopt skill writes when a project has a formatter. Otherwise the first format run makes the section look locally edited, and the next devloop update asks before replacing it.

### D7 — The README and `AGENTS.md` carry the commands, and this spec carries the reasons

Proposed: awaiting operator approval.

The README's development section gives humans the setup and verify commands. `AGENTS.md` gets a repository section, outside the devloop markers, that tells agents which commands to run and when.

There is no separate tooling guide. It would be a third copy of the same commands.

### D8 — `CLAUDE.md` is a symlink to `AGENTS.md`

Answers open question 1: yes.

Claude Code reads `CLAUDE.md`, not `AGENTS.md`. A symlink gives it the same guide, with no second copy to keep in sync. Devloop's adopt skill deliberately leaves this link to the repository.
