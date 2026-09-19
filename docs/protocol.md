# Counter protocol v1

This is an example-local binary format, not a stable library protocol. One
transport message contains exactly one packet; channel 0 is unreliable. All u64
fields are little-endian. The version byte is 1. There is no padding, framing
length, compression, encryption, or authentication in the payload.

| Tag              | Payload after version and tag                                  | Total bytes |
| ---------------- | -------------------------------------------------------------- | ----------- |
| 0 Proposal       | turn:u64, action:u8                                            | 11          |
| 1 Frame, empty   | tick:u64, has_command=0:u8, checksum:u64                       | 19          |
| 1 Frame, command | tick:u64, has_command=1:u8, player:u8, action:u8, checksum:u64 | 21          |
| 2 Ack            | next_tick:u64                                                  | 10          |

Action 0 means Increment; 1 means Decrement. Players are 0 or 1. Unknown versions,
message tags, actions, players, flags, lengths, truncation, and trailing bytes are
rejected before simulation. Decode performs no allocation. Transport source
identity is passed separately; a Proposal never supplies a player identity.

A Proposal requests the current command turn, not a wall-clock or simulation tick.
The host binds the sender to a player and commits at its next simulation step.
It rejects duplicate pending proposals, stale turns, unknown senders, and wrong
players. A stale turn is reported before the player check, so a late retry is
never reported as unauthorized. Game validation still applies. Every step emits
a Frame, including empty steps, so fixed tick progression is replayable. Frame
checksums describe state _after_ applying that tick. Sequence is the count of
accepted game commands in this example, not the frame count or a future
persistence journal sequence.

Replica queues up to a 256-tick forward window and applies only contiguous frames.
Already-applied frames are ignored. Conflicting buffered frames, an excessive
future tick, invalid game inputs, a wrong authority, and checksum mismatch are
errors. A rejected transition leaves the last verified state intact; earlier
valid transitions from the same receive call may already have advanced it.

Ack.next_tick identifies the first missing frame. The host accepts ACKs only from
the replica, no farther than its current progress; older ACKs are harmless.
The driver repeats proposals, resends the unacknowledged frame suffix, and sends
cumulative ACKs each network pump. Lost ACKs therefore lead to harmless duplicate
frames. A complete outage ends with a bounded demo timeout, never false success.

Limitations: one short session, two peers, fixed authority, no handshaking,
reconnect epochs, private state, fragmentation, bandwidth control, or replay attack
protection. A checksum detects accidental divergence; it does not prevent cheating
or authorize a peer. The sample driver treats decode errors as a returned failure;
a production room can choose a peer-local rejection/disconnect policy. Decoder
and bounded session paths handle malformed data with errors rather than panics.
