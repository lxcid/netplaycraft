# ADR 003: checkpoint and journal semantics before store technology

Status: architectural intent accepted; store interfaces and async shape deferred.

Context: durable recovery is an independent axis. OPFS, SQLite, and S3 do not have
identical execution, atomicity, or durability models. Rust browser futures may be
non-Send. A game snapshot is not automatically a committed durable checkpoint.

Decision: keep Snapshotable independent of storage. Plan separate SnapshotStore
and JournalStore contracts, first exercised with a memory backend. Preserve the
recovery invariant: checkpoint after committed record N plus records N+1 onward.
The prototype validates only in-memory snapshot plus ordered frame suffix.

Open before store implementation:

- Native async trait methods versus explicit futures; object safety, borrowing,
  and optional Send without imposing native bounds on browser futures.
- Checkpoint session/schema identity and committed journal watermark.
- Append ordering, retry/idempotency semantics, partial writes, and the exact
  durability point at which authority may acknowledge a command.
- Crash-consistent checkpoint publication and journal retention/pruning.
- OPFS worker/access model and IndexedDB metadata or fallback policy.
- Separate full authoritative recovery state from player-visible projections.

Derive configurable checkpoint/journal durability policy from working primitives.
Do not add store traits, retry infrastructure, transactions, or a policy builder
just to fill out the architecture. Later S3-compatible adapters should support
multiple providers and batch journals/checkpoints, never write the world each tick.
Host migration also needs checkpoint access, authority transfer, and split-brain
policy; persistence alone cannot supply it.
