# ADR 002: semantic message transport with external progress

Status: nonblocking contract accepted; browser adapter decisions open.

Context: browser callbacks, native executors, and deterministic tests have
incompatible scheduling needs. SDP, ICE, and QUIC identifiers are adapter details.

Decision: byte messages, logical peers/channels, connection events, associated
errors, and nonblocking send/poll. Runtime or adapter machinery owns progress;
the in-memory controller advances virtual time. No Send/Sync bound, async executor,
wall clock, or concrete connection-establishment operation in the core contract.
Only unreliable delivery is currently exercised and advertised.

Consequences: the simulated transport really drops packets. The counter's retries
are visible application policy. The memory implementation is single-threaded and
pair-only; neither constraint is imposed on other adapters by Transport.

Open before milestone 2:

- How are channels/capabilities agreed before traffic, and how are unsupported
  reliable/unordered/sequenced combinations reported?
- What progress handle and send-buffer/backpressure policy fits native WebRTC
  and browser callbacks? Avoid an implicit always-running Tokio task.
- Wrap Matchbox behind the contract, or build a small web-sys adapter? Inspect
  candidate dependencies and browser failure paths before choosing.
- How does a replaceable signaling connector exchange offers/answers/candidates,
  while gameplay stays independent of signaling room URLs?
- Define disconnect, reconnect identity/epoch, and retained history semantics.

First browser proof: two tabs, native RTCPeerConnection, ICE/STUN, configurable
TURN, reliable ordered and unreliable channels, connect/disconnect, and unchanged
counter rules. Native-host interoperability should follow without a WASM WebRTC
stack. Keep WebTransport for the later authoritative-server milestone.
