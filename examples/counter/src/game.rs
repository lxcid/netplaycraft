//! Counter rules know only players, commands, and ticks.
use netplaycraft_core::{Checksum, Command, PlayerId, Sequence, Tick};
use netplaycraft_sim::{Checksummed, Simulation, Snapshotable};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Increment,
    Decrement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameError {
    WrongTick,
    WrongPlayer,
    TooManyCommands,
    Overflow,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Counter {
    count: i32,
    next_tick: Tick,
    turn: Sequence,
}

impl Counter {
    pub fn count(&self) -> i32 {
        self.count
    }
    pub fn next_tick(&self) -> Tick {
        self.next_tick
    }
    pub fn turn(&self) -> Sequence {
        self.turn
    }
    pub fn next_player(&self) -> PlayerId {
        PlayerId((self.turn.0 % 2) as u8)
    }
}

impl Simulation for Counter {
    type Input = Command<Action>;
    type Error = GameError;
    fn tick(&mut self, tick: Tick, inputs: &[Self::Input]) -> Result<(), Self::Error> {
        if tick != self.next_tick {
            return Err(GameError::WrongTick);
        }
        if inputs.len() > 1 {
            return Err(GameError::TooManyCommands);
        }
        let next_tick = Tick(tick.0.checked_add(1).ok_or(GameError::Overflow)?);
        let mut count = self.count;
        let mut turn = self.turn;
        if let Some(command) = inputs.first() {
            if command.player != self.next_player() {
                return Err(GameError::WrongPlayer);
            }
            let change = match command.payload {
                Action::Increment => 1,
                Action::Decrement => -1,
            };
            count = count.checked_add(change).ok_or(GameError::Overflow)?;
            turn = Sequence(turn.0.checked_add(1).ok_or(GameError::Overflow)?);
        }
        self.count = count;
        self.turn = turn;
        self.next_tick = next_tick;
        Ok(())
    }
}

impl Snapshotable for Counter {
    type Snapshot = Self;
    type Error = std::convert::Infallible;
    fn snapshot(&self) -> Self {
        self.clone()
    }
    fn restore(&mut self, snapshot: &Self) -> Result<(), Self::Error> {
        *self = snapshot.clone();
        Ok(())
    }
}

impl Checksummed for Counter {
    fn checksum(&self) -> Checksum {
        // FNV-1a over canonical little-endian fields, never Rust memory layout.
        let mut hash = 0xcbf29ce484222325_u64;
        for byte in self
            .count
            .to_le_bytes()
            .into_iter()
            .chain(self.next_tick.0.to_le_bytes())
            .chain(self.turn.0.to_le_bytes())
        {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
        Checksum(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state(game: &Counter) -> (i32, Tick, Sequence, PlayerId) {
        (
            game.count(),
            game.next_tick(),
            game.turn(),
            game.next_player(),
        )
    }
    #[test]
    fn rules_alternate_players_and_empty_ticks_only_advance_time() {
        let mut game = Counter::default();
        game.tick(Tick(0), &[]).unwrap();
        assert_eq!(state(&game), (0, Tick(1), Sequence(0), PlayerId(0)));
        let command = |player: u8, payload| Command {
            player: PlayerId(player),
            payload,
        };
        game.tick(Tick(1), &[command(0, Action::Increment)])
            .unwrap();
        assert_eq!(state(&game), (1, Tick(2), Sequence(1), PlayerId(1)));
        game.tick(Tick(2), &[command(1, Action::Decrement)])
            .unwrap();
        assert_eq!(state(&game), (0, Tick(3), Sequence(2), PlayerId(0)));
        assert_eq!(game.tick(Tick(2), &[]), Err(GameError::WrongTick));
        assert_eq!(
            game.tick(
                Tick(3),
                &[command(0, Action::Increment), command(1, Action::Increment)]
            ),
            Err(GameError::TooManyCommands)
        );
        assert_eq!(state(&game), (0, Tick(3), Sequence(2), PlayerId(0)));
    }
    #[test]
    fn rejected_commands_are_atomic() {
        let mut game = Counter::default();
        let before = game.clone();
        assert_eq!(
            game.tick(
                Tick(0),
                &[Command {
                    player: PlayerId(1),
                    payload: Action::Increment
                }]
            ),
            Err(GameError::WrongPlayer)
        );
        assert_eq!(game, before);
        game.count = i32::MAX;
        let before = game.clone();
        assert_eq!(
            game.tick(
                Tick(0),
                &[Command {
                    player: PlayerId(0),
                    payload: Action::Increment
                }]
            ),
            Err(GameError::Overflow)
        );
        assert_eq!(game, before);
    }
}
