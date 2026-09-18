# Netplaycraft: mission and implementation plan

This document preserves the founding brief under the project's new name,
**Netplaycraft** (formerly Synctide). It describes product intent, not a promise
that all listed features exist. The first delivery is milestones 0–1 only.

## Mission

Build a Rust-first, browser-first multiplayer state-machine runtime that makes
game simulation portable across networking strategies, hosting topologies,
transports, persistence backends, and execution runtimes.

```text
Game = Simulation + Synchronization Strategy + Authority
     + Transport + Persistence + Runtime
```

Every axis must be independently swappable where its semantic requirements are
satisfied. The same game implementation should eventually run between browsers
over WebRTC, with a browser or phone host, on an authoritative cloud server, in
a stateful edge actor such as Cloudflare Durable Objects, or on native Rust.
Storage should range from ephemeral RAM through browser-local persistence and
SQLite to S3/R2 checkpoints and journals.

Optimize first for modern browsers and Rust/WASM; keep the core independent of
browsers and engines. Netplaycraft is not primarily a socket library.

## Product philosophy and invariant

Do not assume a server exists, that server and authority are synonymous, that
transport means WebSocket, that multiplayer means state replication, that every
game rolls back, that persistence means SQL, that Bevy/ECS owns simulation, or
that cloud hosting is mandatory. Expose small composable primitives.

Target configurations, without rewriting the game rules:

| Game | Synchronization | Authority | Transport | Persistence | Runtime |
| --- | --- | --- | --- | --- | --- |
| Mahjong | Commands | Player host | WebRTC | OPFS | Browser/WASM |
| Mahjong | Commands | Server | WebTransport | S3 checkpoints | Native Rust |
| Fighter | Rollback | Peers | WebRTC | RAM | Browser/WASM |
| FPS | Prediction, reconciliation, snapshots | Dedicated | WebTransport | RAM + S3 | Native Rust |

Continuously test this broader portability target: browser host/WebRTC/OPFS,
native host/WebRTC/RAM, dedicated server/WebTransport/S3, and edge
actor/WebSocket or WebTransport/SQLite or R2.

## Networking families

### Fighting games

Deterministic simulation, local and remote input prediction, input and state
history, rollback, resimulation, configurable input delay, and checksum/desync
detection. Study GGPO/GGRS and Fortress Rollback. Rollback operates on abstract
peer-input messages and must not depend on a concrete transport.

### RTS / MOBA / command-driven games

Deterministic ordered command streams, fixed ticks, delayed scheduling, optional
lockstep barriers, optional prediction/rollback, batching, checksums, desync
detection, and deterministic replay. Think in commands rather than snapshots:

```text
Command { player: PlayerId, tick: Tick, payload: ... }
Checkpoint + ordered commands = reconstructed authoritative state
```

### FPS games (later)

Authoritative simulation, client prediction and input history, reconciliation,
remote snapshot interpolation, configured extrapolation, server rewind/lag
compensation, entity replication, interest management, and bandwidth priority.
Keep authoritative_tick, predicted_tick, and render_tick explicit. There is no
universal notion of now.

## Boundaries and dependency direction

Conceptual layers: game → simulation → netcode → authority → topology →
transport → persistence → runtime. These describe concerns, not a requirement
to make each module depend on the next. Upper-level algorithms use practical
trait boundaries; adapters depend inward on portable contracts.

Netcode core must not depend on WebRTC, WebTransport, Cloudflare, S3, Bevy,
browser APIs, or Tokio without a demonstrated necessity. Runtime integration
owns time and I/O. Authority is independent of deployment.

Start with only useful crates. Candidate initial layout:

```text
crates/netplaycraft-core
crates/netplaycraft-sim
crates/netplaycraft-transport
crates/netplaycraft-transport-memory
examples/counter
docs/architecture.md
docs/protocol.md
docs/determinism.md
```

Add netplaycraft-transport-webrtc, netplaycraft-wasm, netplaycraft-rollback,
netplaycraft-store, netplaycraft-store-memory, netplaycraft-store-opfs,
netplaycraft-lockstep, and examples/mahjong-minimal when the corresponding
milestone needs them. Do not create empty future crates.

Possible later crates: snapshot, transport-webtransport, transport-websocket,
store-s3, store-sqlite, runtime-native, runtime-cloudflare, bevy (all using the
netplaycraft prefix). Names and packaging are provisional.

## Minimal APIs to investigate

Use strong domain types: Tick, PeerId, PlayerId, Sequence, ChannelId, and
SessionId when required. Distinguish peer identity from player identity.
Clock views may distinguish authoritative, local/predicted, and render ticks;
do not force meaningless fields on strategies that do not use them.

Simulation should expose an engine-independent tick operation with associated
input and error types. Evaluate a separate Snapshotable trait for snapshot and
restore; command synchronization should not require rollback capabilities.
Snapshot and checksum capabilities should be composable, not a giant game trait.

Transport describes delivery semantics: reliable ordered, reliable unordered,
unreliable, and unreliable sequenced. Candidate operations: send(peer, channel,
bytes), poll() → connected/disconnected/message events. Protocol-specific SDP,
ICE candidates, QUIC stream identifiers, and JS objects belong in adapters.
Applications request semantic channels, not SCTP identifiers.

## WebRTC and WebTransport

Browser P2P is first class. Browser WASM must use the browser's native WebRTC
stack, not compile a whole WebRTC stack into WASM. Eventually support
browser↔browser and browser↔native hosts. Adapter configuration should cover
ICE, STUN, TURN, NAT traversal, ordered reliable and unordered/partially reliable
DataChannels, connection state, RTT, and candidate/path information where practical.

Keep signaling replaceable and separate from gameplay transport. Exchange
opaque offers/answers/candidates; a small WebSocket or HTTP service is acceptable
initially. The game protocol must not depend on that signaling service.

WebTransport follows WebRTC and primarily targets browser↔authoritative server:
datagrams for unreliable gameplay, streams for reliable traffic. Both adapters
implement common semantics where practical; unsupported semantics must be clear.

## Persistence

Separate hot state (RAM each tick), recovery checkpoints, journals of commands/
inputs/events, and archives in object storage. Never write the whole world to S3
each tick. Explore separate SnapshotStore and JournalStore contracts:

```text
load_latest(session) → optional checkpoint
write(session, checkpoint)
append(session, records)
read_from(session, sequence) → records
checkpoint at N + journal N+1..current = recovered state
```

Choose async trait technology only after evaluating native and browser/WASM
constraints. Begin with MemoryStore, then BrowserStore using OPFS where practical;
IndexedDB can hold metadata or be a fallback. Do not build around localStorage.

Later S3Store should accommodate AWS S3, Cloudflare R2, MinIO, Backblaze B2's S3
API, and other compatible stores. Use object storage for checkpoints, command
logs, deterministic replays, spectator history, and match archives.

Derive persistence policy after primitives work: ephemeral, periodic checkpoints
(for example every 600 ticks), journaled, and durable. Policy should trade cost,
latency, durability, and recovery precision independently of storage backend.
Do not freeze the policy API now.

## Authority, topology, and migration

Possible authority placements: local, peer host, dedicated server, edge actor.
Authority receives proposals, validates, applies, orders, and broadcasts accepted
results. Deterministic games may need authoritative command ordering, not world
property replication. Do not implement distributed consensus.

No initial host migration. Keep authority identity out of game state so a future
host can load a checkpoint, replay the journal, and take over when the previous
host disappears. Recovery does not by itself solve election, split brain, or
access to another device's storage.

## Runtimes and language bridges

Rust is canonical; browsers use WASM with wasm-bindgen and eventually a native-
feeling JS/TS API such as hosting a peer-host room with WebRTC and OPFS. That API
is an ergonomic target, not an initial contract. Do not force future native
consumers through WASM: leave room for C ABI, UniFFI-style bindings, Unity/C#,
Godot, and C++.

Avoid embedding an async runtime in the core. Target browser/WASM, native Rust,
edge actors, and dedicated servers. Durable Objects are one adapter for a
stateful authoritative actor, never a core architectural assumption.

## Reference games

First build a two-player alternating-turn counter: i32 count; Increment and
Decrement commands. Across successive milestones it should demonstrate game
simulation, serialization, host authority, WebRTC, reconnect/recovery primitives,
and browser persistence without distracting game rules.

Only then begin a minimal Mahjong state machine. Ultimately it should cover
four peers, private and public information, validated commands, deterministic
replay, device-hosted and server-hosted modes, reconnect, spectators, local
persistence, S3 checkpoints/journals, and host migration. “Host on this device”
and “Host in cloud” must share the game implementation. Private-state projection
requires explicit design before broadcasting a Mahjong journal to every peer.

## Verification and observability

Networking must be testable without real sockets. Seeded in-memory faults should
cover latency, jitter, packet loss, duplication, reordering, and disconnects.
Most netcode tests should use deterministic virtual time.

Run identical initial state and inputs through two simulations and continuously
compare stable checksums. Add properties for rollback→replay equivalence,
checkpoint+journal reconstruction, and ordered commands despite packet reorder.
Consider serialization fuzzing; malformed/untrusted packets must not panic.

Plan metrics for peer, RTT, loss, transport, direct/relay path, authoritative /
predicted / render ticks, rollback count/distance, prediction error, snapshot
buffer depth, journal sequence, and checkpoint age. A later visual timeline
debugger should consume these diagnostics. Do not build the full UI now.

## Research before deeper architecture

Inspect Matchbox, GGRS, Fortress Rollback, Lightyear, Renet, and CrystalOrb.
Study channel semantics and signaling leakage; the smallest rollback interface;
frame/input/state history and replay; engine-specific versus reusable prediction
and replication boundaries; and network-independent reconciliation. Record
architectural lessons with source references, not copied APIs.

## Milestones

| Stage | Deliverable and acceptance |
| --- | --- |
| 0 — Architecture | architecture.md, dependency direction, ADRs for major unresolved questions; only traits needed by the prototype |
| 1 — Local deterministic session | core, sim, in-memory transport, counter; two peers in one process, fixed ticks, commands, encoded messages, latency/loss, checksums; no browser required |
| 2 — Browser P2P | WebRTC and WASM adapters; two tabs run counter; signaling, ICE/STUN, configurable TURN, reliable/unreliable channels, connect/disconnect; replaceable signaling |
| 3 — Rollback | input prediction, state history, rollback/resimulation, input delay, desync checksums; two moving squares under injected latency |
| 4 — Browser persistence | OPFS SnapshotStore + JournalStore; page refresh restores checkpoint+journal and verifies deterministic reconstruction; memory reference backend first |
| 5 — Command/lockstep | scheduling, input delay, barriers where needed, checksums/desync, replay; begin Mahjong state machine |
| Later | WebTransport, native authority, snapshot replication/interpolation, prediction/reconciliation, lag compensation, S3/R2, Durable Objects, host migration, interest management, timeline debugger |

## First implementation checklist

1. Create the Cargo workspace under ~/workspaces/lxcid/netplaycraft.
2. Write architecture.md and document dependency direction.
3. Define only the types required for deterministic command simulation.
4. Implement an in-memory simulated transport.
5. Run the two-player counter over encoded messages.
6. Test deterministic latency, loss, reordering, and checksum equivalence.
7. Report APIs, dependency graph, unresolved choices, and recommendations before
   expanding into WebRTC.

## Engineering constraints and explicit non-goals

Prefer stable safe Rust, minimal dependencies, serde only where useful, tracing
for future runtime observability, strong types, deterministic tests, and small
composable components. Correctness and boundaries precede throughput. Avoid
unnecessary macros, a universal game trait, and giant configuration objects.

Do not begin with FPS replication, physics synchronization, interest management,
lag compensation, matchmaking, authentication, a TURN server, custom STUN,
consensus, MMO scaling, complete Mahjong, Cloudflare deployment, S3 persistence,
Unity/Godot bindings, or Bevy integration.

Use the smallest design that meets the present need. Weigh edge cases by
likelihood, impact, recoverability, and implementation/maintenance cost. Defer
abstractions until an actual second use requires them. The objective is to prove
the seams, not maximize feature count.
