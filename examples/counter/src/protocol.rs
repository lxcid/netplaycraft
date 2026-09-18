//! Deliberately small, bounded example protocol, not the future library wire format.
use crate::game::Action;
use netplaycraft_core::{Checksum, Command, PlayerId, Sequence, Tick};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    pub tick: Tick,
    pub command: Option<Command<Action>>,
    pub checksum: Checksum,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Message {
    Proposal { turn: Sequence, action: Action },
    Frame(Frame),
    Ack { next_tick: Tick },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodeError;

fn action_byte(action: Action) -> u8 {
    match action {
        Action::Increment => 0,
        Action::Decrement => 1,
    }
}
fn action(byte: u8) -> Result<Action, DecodeError> {
    match byte {
        0 => Ok(Action::Increment),
        1 => Ok(Action::Decrement),
        _ => Err(DecodeError),
    }
}
fn number(bytes: &[u8]) -> Result<u64, DecodeError> {
    Ok(u64::from_le_bytes(
        bytes.try_into().map_err(|_| DecodeError)?,
    ))
}

impl Message {
    pub fn encode(self) -> Vec<u8> {
        let mut bytes = vec![1]; // protocol version
        match self {
            Self::Proposal { turn, action } => {
                bytes.push(0);
                bytes.extend(turn.0.to_le_bytes());
                bytes.push(action_byte(action));
            }
            Self::Frame(frame) => {
                bytes.push(1);
                bytes.extend(frame.tick.0.to_le_bytes());
                bytes.push(u8::from(frame.command.is_some()));
                if let Some(command) = frame.command {
                    bytes.push(command.player.0);
                    bytes.push(action_byte(command.payload));
                }
                bytes.extend(frame.checksum.0.to_le_bytes());
            }
            Self::Ack { next_tick } => {
                bytes.push(2);
                bytes.extend(next_tick.0.to_le_bytes());
            }
        }
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        // Check exact sizes before indexing. Reject unknown tags, flags and trailing data.
        match bytes {
            [1, 0, rest @ ..] if rest.len() == 9 => Ok(Self::Proposal {
                turn: Sequence(number(&rest[..8])?),
                action: action(rest[8])?,
            }),
            [1, 1, rest @ ..] if rest.len() == 17 && rest[8] == 0 => Ok(Self::Frame(Frame {
                tick: Tick(number(&rest[..8])?),
                command: None,
                checksum: Checksum(number(&rest[9..])?),
            })),
            [1, 1, rest @ ..] if rest.len() == 19 && rest[8] == 1 && rest[9] < 2 => {
                Ok(Self::Frame(Frame {
                    tick: Tick(number(&rest[..8])?),
                    command: Some(Command {
                        player: PlayerId(rest[9]),
                        payload: action(rest[10])?,
                    }),
                    checksum: Checksum(number(&rest[11..])?),
                }))
            }
            [1, 2, rest @ ..] if rest.len() == 8 => Ok(Self::Ack {
                next_tick: Tick(number(rest)?),
            }),
            _ => Err(DecodeError),
        }
    }
}
