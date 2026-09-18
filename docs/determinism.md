# Determinism and verification

Deterministic means the same initial state and ordered tick/input stream produce
the same state. It does not mean different network schedules select the same
accepted tick. Authority determines a canonical stream; peers replay that stream.

The counter uses checked integer arithmetic, explicit player turns, fixed logical
ticks, and no floats, clocks, randomness, iteration-dependent maps, or I/O inside
game rules. Its diagnostic checksum is FNV-1a over count:i32, next_tick:u64, and
turn:u64 in little-endian order, with wrapping checksum arithmetic. Never hash
Rust memory layout or DefaultHasher for a cross-platform state contract.
Checksums are not cryptographic proofs.

Simulation::tick and Snapshotable::restore must leave state unchanged on error.
Replica checks a candidate state before committing it. A snapshot is an in-memory
Counter clone with private fields, not a versioned durable or wire representation.
A checkpoint at next_tick N requires frames N onward. Empty frames matter for
reconstructing tick progression even when no command changed count.

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p netplaycraft-counter
rustup target add wasm32-unknown-unknown
cargo check --workspace --lib --target wasm32-unknown-unknown
```

The workflow runs these checks on stable Rust. No external Rust crates or services
are required. Network access is needed only to provision a Rust toolchain/target.
The project currently follows stable rather than declaring an independently
verified minimum supported Rust version.

## Coverage

- Virtual latency deadlines and proof that poll does not move time.
- Actual 100% loss, actual duplication, seeded repeatability and reordering.
- Disconnect queue cleanup and rejected post-disconnect sends.
- Invalid network rates, unsupported channels/peers, bounded packets and queues.
- Two peers converge across 32 seeds with 30% loss, 20% duplication, 50% extra
  reordering delay, latency, and jitter; repeating a seed repeats the whole report.
- Zero-fault delivery and bounded failure on total packet loss.
- Reversed and duplicated encoded histories yield the same final state.
- Every split point of generated histories restores from checkpoint plus suffix;
  restore/replay repeats the result and each applied tick matches its checksum.
- Sender binding, turn ordering, ACK bounds, duplicate/stale proposals, conflicting
  frames, future-window limits, history limits, and explicit desync errors.
- Exact wire round trips, golden ACK bytes, truncation/trailing-data rejection,
  invalid discriminants, oversized packets, and a seeded malformed-byte corpus.

These are deterministic property-style loops, not a coverage-guided fuzzer or a
formal proof. Restore/replay tests exercise the capability required by rollback;
there is no rollback prediction/history algorithm yet. Recovery is in memory,
not tested against browser crashes or durable storage failures.

Native tests plus WASM compilation do not establish cross-runtime equivalence.
Before browser claims, run the same canonical input fixtures in native Rust and
actual browser WASM and compare all per-tick checksums. Before adding rollback,
add late-input correction and resimulation properties. Before adding persistence,
test partial journal writes, checkpoint publication, and page-refresh recovery.
