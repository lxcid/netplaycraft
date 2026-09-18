//! Seeded, single-threaded two-endpoint network driven exclusively by virtual time.
use netplaycraft_core::{ChannelId, PeerId};
use netplaycraft_transport::{Delivery, Transport, TransportEvent};
use std::{cell::RefCell, collections::VecDeque, rc::Rc, time::Duration};

pub const GAMEPLAY: ChannelId = ChannelId(0);
pub const MAX_PAYLOAD: usize = 1200;
const MAX_QUEUED: usize = 4096;

/// Fault percentages use integer basis points (0..=10_000).
/// Jitter and reordering add delay; loss is not repaired by this adapter.
#[derive(Clone, Copy, Debug, Default)]
pub struct NetworkConditions {
    pub latency_ms: u32,
    pub jitter_ms: u32,
    pub loss_bps: u16,
    pub duplicate_bps: u16,
    pub reorder_bps: u16,
    pub reorder_delay_ms: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkError {
    InvalidConditions,
    DuplicatePeer,
    UnknownPeer,
    UnknownChannel,
    Disconnected,
    PayloadTooLarge,
    Backpressure,
    TimeOverflow,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NetworkStats {
    pub sent: u64,
    pub dropped: u64,
    pub duplicated: u64,
    pub delivered: u64,
}

struct Packet {
    due: Duration,
    destination: usize,
    event: TransportEvent,
}

struct State {
    now: Duration,
    rng: u64,
    peers: [PeerId; 2],
    connected: bool,
    conditions: NetworkConditions,
    packets: Vec<Packet>,
    inboxes: [VecDeque<TransportEvent>; 2],
    stats: NetworkStats,
}

impl State {
    // Explicit wrapping arithmetic makes the generator platform independent.
    fn random(&mut self) -> u64 {
        self.rng = self
            .rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.rng >> 16
    }
    fn chance(&mut self, bps: u16) -> bool {
        self.random() % 10_000 < u64::from(bps)
    }
}

/// The controller is kept outside endpoints and outside game logic.
pub struct SimulatedNetwork(Rc<RefCell<State>>);
pub struct MemoryTransport {
    state: Rc<RefCell<State>>,
    local: usize,
}

impl SimulatedNetwork {
    pub fn pair(
        peers: [PeerId; 2],
        conditions: NetworkConditions,
        seed: u64,
    ) -> Result<(Self, MemoryTransport, MemoryTransport), NetworkError> {
        if peers[0] == peers[1] {
            return Err(NetworkError::DuplicatePeer);
        }
        if [
            conditions.loss_bps,
            conditions.duplicate_bps,
            conditions.reorder_bps,
        ]
        .iter()
        .any(|&value| value > 10_000)
        {
            return Err(NetworkError::InvalidConditions);
        }
        let state = Rc::new(RefCell::new(State {
            now: Duration::ZERO,
            rng: seed,
            peers,
            connected: true,
            conditions,
            packets: Vec::new(),
            stats: NetworkStats::default(),
            inboxes: [
                VecDeque::from([TransportEvent::Connected(peers[1])]),
                VecDeque::from([TransportEvent::Connected(peers[0])]),
            ],
        }));
        Ok((
            Self(state.clone()),
            MemoryTransport {
                state: state.clone(),
                local: 0,
            },
            MemoryTransport { state, local: 1 },
        ))
    }

    pub fn advance(&mut self, elapsed: Duration) -> Result<(), NetworkError> {
        let mut state = self.0.borrow_mut();
        state.now = state
            .now
            .checked_add(elapsed)
            .ok_or(NetworkError::TimeOverflow)?;
        // Stable sorting preserves send order when delivery times tie.
        state.packets.sort_by_key(|packet| packet.due);
        let ready = state
            .packets
            .partition_point(|packet| packet.due <= state.now);
        let packets: Vec<_> = state.packets.drain(..ready).collect();
        for packet in packets {
            state.inboxes[packet.destination].push_back(packet.event);
            state.stats.delivered += 1;
        }
        Ok(())
    }

    pub fn stats(&self) -> NetworkStats {
        self.0.borrow().stats
    }

    /// Drops in-flight/queued messages and emits one disconnect event per endpoint.
    /// Reconnection is intentionally deferred; construct a fresh pair for a new link.
    pub fn disconnect(&mut self) {
        let mut state = self.0.borrow_mut();
        if !state.connected {
            return;
        }
        state.connected = false;
        state.packets.clear();
        for local in 0..2 {
            state.inboxes[local].clear();
            let remote = state.peers[1 - local];
            state.inboxes[local].push_back(TransportEvent::Disconnected(remote));
        }
    }
}

impl Transport for MemoryTransport {
    type Error = NetworkError;
    fn delivery(&self, channel: ChannelId) -> Option<Delivery> {
        (channel == GAMEPLAY).then_some(Delivery::Unreliable)
    }
    fn send(&mut self, peer: PeerId, channel: ChannelId, bytes: &[u8]) -> Result<(), Self::Error> {
        if self.delivery(channel).is_none() {
            return Err(NetworkError::UnknownChannel);
        }
        if bytes.len() > MAX_PAYLOAD {
            return Err(NetworkError::PayloadTooLarge);
        }
        let mut state = self.state.borrow_mut();
        if peer != state.peers[1 - self.local] {
            return Err(NetworkError::UnknownPeer);
        }
        if !state.connected {
            return Err(NetworkError::Disconnected);
        }
        // Reserve space for a possible duplicate before accepting the message.
        let queued = state.packets.len() + state.inboxes.iter().map(VecDeque::len).sum::<usize>();
        if queued + 2 > MAX_QUEUED {
            return Err(NetworkError::Backpressure);
        }
        let conditions = state.conditions;
        // Validate the worst-case time before changing queue state or diagnostics.
        let max_delay = u64::from(conditions.latency_ms)
            + u64::from(conditions.jitter_ms)
            + u64::from(conditions.reorder_delay_ms);
        state
            .now
            .checked_add(Duration::from_millis(max_delay))
            .ok_or(NetworkError::TimeOverflow)?;
        state.stats.sent += 1;
        if state.chance(conditions.loss_bps) {
            state.stats.dropped += 1;
            return Ok(());
        }
        let duplicate = state.chance(conditions.duplicate_bps);
        state.stats.duplicated += u64::from(duplicate);
        for _ in 0..(if duplicate { 2 } else { 1 }) {
            let jitter = state.random() % (u64::from(conditions.jitter_ms) + 1);
            let reorder = if state.chance(conditions.reorder_bps) {
                u64::from(conditions.reorder_delay_ms)
            } else {
                0
            };
            let due = state.now
                + Duration::from_millis(u64::from(conditions.latency_ms) + jitter + reorder);
            let sender = state.peers[self.local];
            state.packets.push(Packet {
                due,
                destination: 1 - self.local,
                event: TransportEvent::Message {
                    peer: sender,
                    channel,
                    payload: bytes.to_vec(),
                },
            });
        }
        Ok(())
    }
    fn poll(&mut self) -> Result<Option<TransportEvent>, Self::Error> {
        Ok(self.state.borrow_mut().inboxes[self.local].pop_front())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pair(c: NetworkConditions) -> (SimulatedNetwork, MemoryTransport, MemoryTransport) {
        let (n, mut a, mut b) = SimulatedNetwork::pair([PeerId(10), PeerId(20)], c, 42).unwrap();
        assert_eq!(a.poll(), Ok(Some(TransportEvent::Connected(PeerId(20)))));
        b.poll().unwrap();
        (n, a, b)
    }
    fn messages(endpoint: &mut MemoryTransport) -> Vec<Vec<u8>> {
        let mut result = Vec::new();
        while let Some(event) = endpoint.poll().unwrap() {
            if let TransportEvent::Message { payload, .. } = event {
                result.push(payload);
            }
        }
        result
    }
    #[test]
    fn latency_and_poll_do_not_advance_time() {
        let (mut n, mut a, mut b) = pair(NetworkConditions {
            latency_ms: 80,
            ..Default::default()
        });
        a.send(PeerId(20), GAMEPLAY, &[1]).unwrap();
        n.advance(Duration::from_millis(79)).unwrap();
        assert!(messages(&mut b).is_empty());
        assert!(messages(&mut b).is_empty());
        n.advance(Duration::from_millis(1)).unwrap();
        assert_eq!(messages(&mut b), vec![vec![1]]);
    }
    #[test]
    fn total_loss_and_duplication_are_real() {
        let (mut n, mut a, mut b) = pair(NetworkConditions {
            loss_bps: 10_000,
            ..Default::default()
        });
        a.send(PeerId(20), GAMEPLAY, &[1]).unwrap();
        n.advance(Duration::from_secs(1)).unwrap();
        assert!(messages(&mut b).is_empty());
        assert_eq!(n.stats().dropped, 1);
        let (mut n, mut a, mut b) = pair(NetworkConditions {
            duplicate_bps: 10_000,
            ..Default::default()
        });
        a.send(PeerId(20), GAMEPLAY, &[2]).unwrap();
        n.advance(Duration::ZERO).unwrap();
        assert_eq!(messages(&mut b), vec![vec![2], vec![2]]);
    }
    #[test]
    fn seeded_faults_repeat_and_reorder() {
        fn run() -> (Vec<Vec<u8>>, NetworkStats) {
            let (mut n, mut a, mut b) = pair(NetworkConditions {
                latency_ms: 20,
                jitter_ms: 10,
                loss_bps: 2000,
                duplicate_bps: 2000,
                reorder_bps: 5000,
                reorder_delay_ms: 100,
            });
            for byte in 0..100 {
                a.send(PeerId(20), GAMEPLAY, &[byte]).unwrap();
            }
            n.advance(Duration::from_secs(1)).unwrap();
            (messages(&mut b), n.stats())
        }
        let (packets, stats) = run();
        assert_eq!((packets.clone(), stats), run());
        assert!(stats.dropped > 0 && stats.duplicated > 0);
        assert!(packets.windows(2).any(|p| p[0] > p[1]));
    }
    #[test]
    fn disconnect_clears_traffic_and_rejects_sends() {
        let (mut n, mut a, mut b) = pair(Default::default());
        a.send(PeerId(20), GAMEPLAY, &[1]).unwrap();
        n.disconnect();
        n.disconnect();
        n.advance(Duration::from_secs(1)).unwrap();
        assert_eq!(b.poll(), Ok(Some(TransportEvent::Disconnected(PeerId(10)))));
        assert_eq!(b.poll(), Ok(None));
        assert_eq!(
            a.send(PeerId(20), GAMEPLAY, &[1]),
            Err(NetworkError::Disconnected)
        );
    }
    #[test]
    fn invalid_configuration_and_send_limits_are_errors() {
        assert!(matches!(
            SimulatedNetwork::pair(
                [PeerId(1), PeerId(2)],
                NetworkConditions {
                    loss_bps: 10_001,
                    ..Default::default()
                },
                1
            ),
            Err(NetworkError::InvalidConditions)
        ));
        let (_, mut a, _) = pair(Default::default());
        assert_eq!(
            a.send(PeerId(99), GAMEPLAY, &[]),
            Err(NetworkError::UnknownPeer)
        );
        assert_eq!(
            a.send(PeerId(20), ChannelId(9), &[]),
            Err(NetworkError::UnknownChannel)
        );
        assert_eq!(
            a.send(PeerId(20), GAMEPLAY, &[0; MAX_PAYLOAD + 1]),
            Err(NetworkError::PayloadTooLarge)
        );
        let mut bounded = false;
        for _ in 0..MAX_QUEUED {
            if a.send(PeerId(20), GAMEPLAY, &[0]) == Err(NetworkError::Backpressure) {
                bounded = true;
                break;
            }
        }
        assert!(bounded);
    }
}
