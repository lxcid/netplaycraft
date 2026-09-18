# Netplaycraft

A Rust-first, browser-first portable multiplayer state-machine runtime.

Formerly planned as Synctide. The project and crate prefix are now **netplaycraft**.

```text
Game = Simulation + Synchronization Strategy + Authority
     + Transport + Persistence + Runtime
```

These parts must remain independently swappable. Topology determines how peers
connect; it does not determine who has authority. Game logic must not depend on
browser APIs, an engine, a transport, a cloud provider, or durable storage.

## Project plan

- [Mission and staged roadmap](docs/plan.md)
- Architecture and the first executable prototype are being developed next.

The first scope is architecture and a deterministic two-player counter over an
in-memory network. WebRTC, rollback, browser persistence, and Mahjong follow only
after this establishes useful boundaries.
