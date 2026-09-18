# Architectural research

Inspected upstream source snapshots on 2026-09-19 before implementing the portable
contracts. These are architectural observations, not performance, security, or
maintenance audits. No upstream implementation was copied or added as a dependency.
Immutable links identify the actual revisions inspected.

## Matchbox

[Source 1](https://github.com/johanhelsing/matchbox/blob/234740535683ecd4d59215eb54c29df7a8fcbcbd/matchbox_socket/src/webrtc_socket/socket.rs) · [Source 2](https://github.com/johanhelsing/matchbox/blob/234740535683ecd4d59215eb54c29df7a8fcbcbd/matchbox_socket/src/webrtc_socket/wasm.rs)

The socket API uses configurable channels with ordering and retransmit controls;
its WASM backend imports web-sys RTCPeerConnection. Its builder includes signaling
room configuration and a replaceable SignallerBuilder; callers separately drive a
message-loop future. Learn from the native/browser adapter split and progress
ownership. Keep room URLs and WebRTC-specific channel configuration outside the
Netplaycraft game/session contract. Evaluate wrapping this implementation before
writing another browser transport; no dependency choice has been made.

## Ggrs

[Source 1](https://github.com/gschup/ggrs/blob/e97e3d2416cc68af2d2876d41180d950c2939b6e/src/lib.rs) · [Source 2](https://github.com/gschup/ggrs/blob/e97e3d2416cc68af2d2876d41180d950c2939b6e/src/input_queue.rs)

GgrsRequest asks the caller to save state, load state, or advance with player
inputs. NonBlockingSocket abstracts packet exchange. InputQueue maintains a ring
history, prediction, first incorrect frame, and frame-delay state. This suggests
rollback can be an algorithm over inputs and state operations without owning an
engine or transport. Avoid copying its complete Config bounds, numeric sentinel
frames, and history sizing into a universal Netplaycraft simulation trait. Make
history limits and prediction policy explicit when rollback is actually built.

## Fortress

[Source 1](https://github.com/wallstop/fortress-rollback/blob/46e4e22ab3582a4204cf9cf0ea6a9b4bc00f7217/src/lib.rs) · [Source 2](https://github.com/wallstop/fortress-rollback/blob/46e4e22ab3582a4204cf9cf0ea6a9b4bc00f7217/src/input_queue/mod.rs)

FortressRequest retains save/load/advance operations. Its input queue and
configuration expand the history/prediction controls, and its documentation
emphasizes validated error paths, observability, and verification. Learn from
explicit limits and error handling, but do not import an entire configuration and
verification framework into the first prototype. Claims about zero panics in its
README are project claims, not an audit performed here.

## Lightyear

[Source 1](https://github.com/cBournhonesque/lightyear/blob/1759ae99f3cedd41c73d5c081869f1c95be69843/README.md) · [Source 2](https://github.com/cBournhonesque/lightyear/tree/1759ae99f3cedd41c73d5c081869f1c95be69843/crates/replication)

The repository separates IO, connection, core, and replication/prediction/
interpolation concerns. It supports input-driven deterministic replication as well
as world replication. Public integration centers on Bevy plugins, ECS entities,
components, and scheduling. Reuse the conceptual separation of input streams,
replicated state, predicted state, and rendered/interpolated state. Do not expose
Bevy's world/component ownership model in portable Netplaycraft traits. A later
Bevy adapter should own those integration details.

## Renet

[Source 1](https://github.com/lucaspoffo/renet/blob/2a5080d78d9ea4c3868c3efc80487573906a0eb1/renet/src/channel/mod.rs) · [Source 2](https://github.com/lucaspoffo/renet/blob/2a5080d78d9ea4c3868c3efc80487573906a0eb1/renet/src/lib.rs)

SendType distinguishes unreliable, reliable ordered, and reliable unordered
channels. ChannelConfig carries channel identity, memory budget, and resend policy;
its documented full-buffer behavior differs by delivery mode. Adopt clear channel
semantics and bounded memory. Keep resend intervals as implementation/policy
choices rather than pretending every adapter implements the same packet-level
reliability. Do not inherit client/server placement as the definition of authority.

## Crystalorb

[Source 1](https://github.com/ErnWong/crystalorb/blob/b49cf551f41a8363c1aa0ca4a2486ac2dd433673/src/world/world_trait.rs) · [Source 2](https://github.com/ErnWong/crystalorb/blob/b49cf551f41a8363c1aa0ca4a2486ac2dd433673/src/network_resource.rs)

World separates commands, snapshots, and display state conceptually, while
NetworkResource/Connection let an external network supply message exchange.
Prediction/reconciliation and display interpolation are separate concerns. The
World contract nevertheless bundles snapshot, rendering, serialization, and
Send/Sync requirements. Learn from its network boundary but keep these game
capabilities independent so a command-only counter needs no render state or
interpolation API. Client/server timeline policy should belong to its strategy.

## Resulting choices

The prototype uses independent simulation/snapshot/checksum capabilities,
nonblocking byte transport, explicit virtual time, and example-local authority
and replay. There is no universal netcode session, complete channel registry,
async store contract, or engine integration. The next browser experiment should
resolve adapter progress ownership and semantic channel negotiation; the rollback
experiment should resolve trait-driven versus request-driven simulation control.
