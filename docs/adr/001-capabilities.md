# ADR 001: independent simulation capabilities

Status: accepted for the prototype; public APIs remain provisional.

Context: command replication can advance a game without rollback snapshots;
rollback needs restoration, and diagnostics need deterministic fingerprints.
A universal game trait would burden each strategy with unrelated requirements.

Decision: separate Simulation, Snapshotable, and Checksummed. Keep serialization
in the example protocol, authority in example Host, and replay in example Replica.
Require atomic failure from tick/restore. Use small typed identities and counters.

Consequences: strategies choose their required capabilities. No engine, runtime,
serializer, authority identity, or transport appears in Counter state. The sample
replica clones Counter to validate a frame; this is not a mandatory core Clone
bound or the chosen storage model for future large worlds.

Open: should future strategies return advance/save/load requests (as GGRS does),
or drive capability traits directly? How should fallible restoration and partial
resimulation report failures? Validate with the moving-squares rollback milestone
before abstracting a universal session or state-history container.
