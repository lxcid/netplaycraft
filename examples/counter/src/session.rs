//! Example-specific host authority and ordered command replay. No transport or clock API.
use crate::{
    game::{Action, Counter, GameError},
    protocol::Frame,
};
use netplaycraft_core::{Command, PeerId, PlayerId, Sequence, Tick};
use netplaycraft_sim::{Checksummed, Simulation};
use std::collections::{BTreeMap, btree_map::Entry};

/// This demo retains a short complete history. Durable storage/pruning is future work.
pub const MAX_HISTORY: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionError {
    InvalidPeers,
    Unauthorized,
    StaleTurn,
    PendingCommand,
    HistoryFull,
    OutsideWindow,
    ConflictingFrame,
    Desync,
    InvalidAck,
    Game(GameError),
}
impl From<GameError> for SessionError {
    fn from(error: GameError) -> Self {
        Self::Game(error)
    }
}

pub struct Host {
    game: Counter,
    players: [PeerId; 2],
    pending: Option<Command<Action>>,
    frames: Vec<Frame>,
    acknowledged: Tick,
}
impl Host {
    pub fn new(players: [PeerId; 2]) -> Result<Self, SessionError> {
        if players[0] == players[1] {
            return Err(SessionError::InvalidPeers);
        }
        Ok(Self {
            game: Counter::default(),
            players,
            pending: None,
            frames: Vec::new(),
            acknowledged: Tick(0),
        })
    }
    pub fn game(&self) -> &Counter {
        &self.game
    }
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }
    pub fn unacknowledged(&self) -> &[Frame] {
        &self.frames[self.acknowledged.0 as usize..]
    }

    /// Bind a proposal to its actual sender. Payloads cannot claim a player ID.
    /// One proposal per turn; retries are rejected once that turn is accepted.
    pub fn propose(
        &mut self,
        sender: PeerId,
        turn: Sequence,
        action: Action,
    ) -> Result<(), SessionError> {
        let player = self
            .players
            .iter()
            .position(|&peer| peer == sender)
            .ok_or(SessionError::Unauthorized)?;
        if PlayerId(player as u8) != self.game.next_player() {
            return Err(SessionError::Unauthorized);
        }
        if turn != self.game.turn() {
            return Err(SessionError::StaleTurn);
        }
        if self.pending.is_some() {
            return Err(SessionError::PendingCommand);
        }
        self.pending = Some(Command {
            player: PlayerId(player as u8),
            payload: action,
        });
        Ok(())
    }
    /// Runtime calls this once per fixed step. Authority chooses the accepted tick.
    pub fn advance(&mut self) -> Result<Frame, SessionError> {
        if self.frames.len() >= MAX_HISTORY {
            return Err(SessionError::HistoryFull);
        }
        let tick = self.game.next_tick();
        self.game.tick(tick, self.pending.as_slice())?;
        let frame = Frame {
            tick,
            command: self.pending.take(),
            checksum: self.game.checksum(),
        };
        self.frames.push(frame);
        Ok(frame)
    }
    pub fn acknowledge(&mut self, sender: PeerId, next_tick: Tick) -> Result<(), SessionError> {
        if sender != self.players[1] {
            return Err(SessionError::Unauthorized);
        }
        if next_tick > self.game.next_tick() {
            return Err(SessionError::InvalidAck);
        }
        self.acknowledged = self.acknowledged.max(next_tick);
        Ok(())
    }
}

pub struct Replica {
    game: Counter,
    authority: PeerId,
    pending: BTreeMap<Tick, Frame>,
}
impl Replica {
    pub fn new(authority: PeerId) -> Self {
        Self {
            game: Counter::default(),
            authority,
            pending: BTreeMap::new(),
        }
    }
    pub fn game(&self) -> &Counter {
        &self.game
    }

    /// Queue reordered frames; apply only a contiguous verified prefix.
    /// Every applied tick is checked, not just the final state.
    pub fn receive(&mut self, sender: PeerId, frame: Frame) -> Result<(), SessionError> {
        if sender != self.authority {
            return Err(SessionError::Unauthorized);
        }
        if frame.tick < self.game.next_tick() {
            return Ok(());
        } // retransmission
        if frame.tick.0 - self.game.next_tick().0 >= MAX_HISTORY as u64 {
            return Err(SessionError::OutsideWindow);
        }
        match self.pending.entry(frame.tick) {
            Entry::Occupied(entry) if *entry.get() != frame => {
                return Err(SessionError::ConflictingFrame);
            }
            Entry::Occupied(_) => {}
            Entry::Vacant(entry) => {
                entry.insert(frame);
            }
        }
        while let Some(frame) = self.pending.remove(&self.game.next_tick()) {
            let mut candidate = self.game.clone();
            candidate.tick(frame.tick, frame.command.as_slice())?;
            if candidate.checksum() != frame.checksum {
                return Err(SessionError::Desync);
            }
            self.game = candidate;
        }
        Ok(())
    }
}
