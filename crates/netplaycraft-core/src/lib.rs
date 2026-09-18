//! Portable identity and ordering types. No runtime, I/O, or serialization policy.
#![no_std]

/// Simulation step, starting at zero. A state at `Tick(n)` is before step n.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tick(pub u64);

/// Connection identity; it is not a game player or an authority role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PeerId(pub u64);

/// Game participant identity, independent of connections.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerId(pub u8);

/// Ordered application record identifier; interpretation belongs to the protocol.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sequence(pub u64);

/// Logical channel identifier, not an adapter's stream or DataChannel ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelId(pub u8);

/// Diagnostic state fingerprint, not a cryptographic integrity check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Checksum(pub u64);

/// A game command contains no transport identity or authority placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command<T> {
    pub player: PlayerId,
    pub payload: T,
}
