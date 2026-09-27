---
status: draft
---

# Build setup

## Problem

Netplaycraft has no build. `main` holds documentation only. There is no Cargo workspace, no pinned toolchain, no formatter, and no CI.

Every later pipeline needs the same way to install tools and check its work, both locally and in CI. Without one, each pipeline invents its own setup, and "it passes" means something different on every machine.

[PR #1](https://github.com/lxcid/netplaycraft/pull/1) bundles this setup with a counter prototype that is larger than the smallest useful step. The setup should land first, on its own, so each later step can stay small.

## Proposed outcome

A fresh clone installs its pinned tools and verifies the whole repository with one command. CI runs that same command on every pull request and every push to `main`.

The outcome is true when:

- On a fresh clone on macOS, the documented setup installs every pinned tool, and the verify command passes.
- CI runs the same verify command on a pull request, and it passes.
- Verify fails on unformatted Rust, unformatted Markdown, a Clippy warning, and a failing test.
- The workspace holds one crate: the minimum counter game rules, with unit tests.
- The README and `AGENTS.md` name the setup and verify commands.

## Affected users and systems

- The operator, setting up and verifying on their own machine.
- Coding agents, which read `AGENTS.md` to learn how to check their work.
- GitHub Actions, which runs the check.
- PR #1, which later rebases onto this setup and drops its own tooling commits.
- Every later pipeline, which inherits the toolchain and the verify command.

## Constraints

- proto pins tool versions, Moon orchestrates tasks, and pnpm installs Node dev tools. This is how the operator sets up monorepos.
- The only crate holds the minimum counter game rules: two players alternate turns, and each turn increments or decrements a shared count.
- Out of scope: networking, transport, sessions, authority, simulation traits, ticks, snapshots, checksums, and shared core types.
- Out of scope: anything from a later milestone, including a browser app and WASM bindings.
- This pipeline does not change PR #1.

## Open questions

1. Does setup include making Claude Code load `AGENTS.md`?
2. Should verify check that the rules compile for the browser target, `wasm32-unknown-unknown`?
