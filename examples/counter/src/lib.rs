//! Milestone 1 reference application; protocol/session policy remains example-local.
pub mod game;
pub mod protocol;
pub mod session;

use game::{Action, Counter};
use netplaycraft_core::{PeerId, PlayerId, Tick};
use netplaycraft_sim::{Checksummed, Snapshotable};
use netplaycraft_transport::{Transport, TransportEvent};
use netplaycraft_transport_memory::{
    GAMEPLAY, NetworkConditions, NetworkError, NetworkStats, SimulatedNetwork,
};
use protocol::{DecodeError, Message};
use session::{Host, Replica, SessionError};
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub enum DemoError {
    Network(NetworkError),
    Decode(DecodeError),
    Session(SessionError),
    Timeout,
}
impl From<NetworkError> for DemoError {
    fn from(e: NetworkError) -> Self {
        Self::Network(e)
    }
}
impl From<DecodeError> for DemoError {
    fn from(e: DecodeError) -> Self {
        Self::Decode(e)
    }
}
impl From<SessionError> for DemoError {
    fn from(e: SessionError) -> Self {
        Self::Session(e)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DemoReport {
    pub host: Counter,
    pub replica: Counter,
    pub stats: NetworkStats,
    pub network_steps: usize,
}

/// Two endpoints, host-selected command order, a 50 Hz virtual simulation,
/// and application-level retries over an actually lossy channel.
pub fn run_demo(conditions: NetworkConditions, seed: u64) -> Result<DemoReport, DemoError> {
    let peers = [PeerId(10), PeerId(20)];
    let (mut network, mut host_io, mut guest_io) = SimulatedNetwork::pair(peers, conditions, seed)?;
    let mut host = Host::new(peers)?;
    let mut guest = Replica::new(peers[0]);
    const SIMULATION_STEPS: u64 = 120;
    const MAX_NETWORK_STEPS: usize = 2000;
    for step in 0..MAX_NETWORK_STEPS {
        network.advance(Duration::from_millis(20));
        while let Some(event) = host_io.poll()? {
            if let TransportEvent::Message {
                peer,
                channel: GAMEPLAY,
                payload,
            } = event
            {
                match Message::decode(&payload)? {
                    Message::Proposal { turn, action }
                        if host.game().next_tick().0 < SIMULATION_STEPS =>
                    {
                        // Late retries are StaleTurn; a proposal already in flight
                        // is PendingCommand. Anything else is a real fault.
                        match host.propose(peer, turn, action) {
                            Ok(())
                            | Err(SessionError::StaleTurn | SessionError::PendingCommand) => {}
                            Err(error) => return Err(error.into()),
                        }
                    }
                    Message::Ack { next_tick } => host.acknowledge(peer, next_tick)?,
                    _ => {}
                }
            }
        }
        while let Some(event) = guest_io.poll()? {
            if let TransportEvent::Message {
                peer,
                channel: GAMEPLAY,
                payload,
            } = event
                && let Message::Frame(frame) = Message::decode(&payload)?
            {
                guest.receive(peer, frame)?;
            }
        }
        // Inputs are synthetic UI actions; this policy is outside the game rules.
        let choose_action = |turn: u64| {
            if turn % 3 == 2 {
                Action::Decrement
            } else {
                Action::Increment
            }
        };
        if guest.game().next_player() == PlayerId(1) {
            guest_io.send(
                peers[0],
                GAMEPLAY,
                &Message::Proposal {
                    turn: guest.game().turn(),
                    action: choose_action(guest.game().turn().0),
                }
                .encode(),
            )?;
        }
        guest_io.send(
            peers[0],
            GAMEPLAY,
            &Message::Ack {
                next_tick: guest.game().next_tick(),
            }
            .encode(),
        )?;
        if host.game().next_tick().0 < SIMULATION_STEPS {
            if host.game().next_player() == PlayerId(0) {
                host.propose(
                    peers[0],
                    host.game().turn(),
                    choose_action(host.game().turn().0),
                )?;
            }
            host.advance()?;
        }
        // Intentionally simple: resend the unacknowledged suffix on each pump.
        // This is a small example policy, not a general reliable transport.
        for frame in host.unacknowledged() {
            host_io.send(peers[1], GAMEPLAY, &Message::Frame(*frame).encode())?;
        }
        if host.game().next_tick() == Tick(SIMULATION_STEPS) && host.unacknowledged().is_empty() {
            if host.game().checksum() != guest.game().checksum() {
                return Err(SessionError::Desync.into());
            }
            return Ok(DemoReport {
                host: host.game().snapshot(),
                replica: guest.game().snapshot(),
                stats: network.stats(),
                network_steps: step + 1,
            });
        }
    }
    Err(DemoError::Timeout)
}
