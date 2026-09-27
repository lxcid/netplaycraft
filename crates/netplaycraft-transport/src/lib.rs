//! Nonblocking message transport contract; adapters own connection establishment.
use netplaycraft_core::{ChannelId, PeerId};

/// Only the semantics exercised by milestone 1. Extend with reliable channels
/// when an adapter actually supplies their guarantees.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delivery {
    /// Messages may be lost, duplicated, or reordered.
    Unreliable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransportEvent {
    Connected(PeerId),
    Disconnected(PeerId),
    Message {
        peer: PeerId,
        channel: ChannelId,
        payload: Vec<u8>,
    },
}

pub trait Transport {
    type Error;
    fn delivery(&self, channel: ChannelId) -> Option<Delivery>;
    /// Ok means accepted for sending, not delivered. Must not block.
    fn send(&mut self, peer: PeerId, channel: ChannelId, bytes: &[u8]) -> Result<(), Self::Error>;
    /// Return the next available event, if any; this does not advance simulation time.
    fn poll(&mut self) -> Result<Option<TransportEvent>, Self::Error>;
}
