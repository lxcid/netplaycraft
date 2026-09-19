use netplaycraft_core::{Checksum, Command, PeerId, PlayerId, Sequence, Tick};
use netplaycraft_counter::{
    DemoError,
    game::{Action, Counter},
    protocol::Message,
    run_demo,
    session::{Frame, Host, MAX_HISTORY, Replica, SessionError},
};
use netplaycraft_sim::{Checksummed, Simulation, Snapshotable};
use netplaycraft_transport_memory::NetworkConditions;

const PEERS: [PeerId; 2] = [PeerId(10), PeerId(20)];

/// Seeded test-input generator; not a statistical RNG.
fn lcg(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    *state
}

fn history(seed: u64, steps: usize) -> (Vec<Frame>, Counter) {
    let mut host = Host::new(PEERS).unwrap();
    let mut rng = seed;
    for _ in 0..steps {
        let sample = lcg(&mut rng);
        // Include empty ticks as well as both actions; every command alternates player.
        if !sample.is_multiple_of(4) {
            let action = if sample & 256 == 0 {
                Action::Increment
            } else {
                Action::Decrement
            };
            host.propose(
                PEERS[host.game().next_player().0 as usize],
                host.game().turn(),
                action,
            )
            .unwrap();
        }
        host.advance().unwrap();
    }
    (host.frames().to_vec(), host.game().clone())
}

#[test]
fn two_peers_converge_under_seeded_faults() {
    let conditions = NetworkConditions {
        latency_ms: 80,
        jitter_ms: 30,
        loss_bps: 3000,
        duplicate_bps: 2000,
        reorder_bps: 5000,
        reorder_delay_ms: 100,
    };
    for seed in 0..32 {
        let report = run_demo(conditions, seed).unwrap();
        assert_eq!(report.host, report.replica, "seed {seed}");
        assert!(report.host.turn().0 >= 2, "both players must participate");
        assert!(report.stats.dropped > 0 && report.stats.duplicated > 0);
        assert_eq!(
            report,
            run_demo(conditions, seed).unwrap(),
            "seed {seed} must repeat exactly"
        );
    }
}

#[test]
fn zero_faults_work_and_total_loss_times_out() {
    let report = run_demo(NetworkConditions::default(), 7).unwrap();
    assert_eq!(report.host, report.replica);
    assert_eq!(report.stats.dropped, 0);
    assert_eq!(
        run_demo(
            NetworkConditions {
                loss_bps: 10_000,
                ..Default::default()
            },
            7
        ),
        Err(DemoError::Timeout)
    );
}

#[test]
fn reversed_duplicated_encoded_frames_replay_identically() {
    for seed in 0..32 {
        let (frames, expected) = history(seed, 100);
        let mut replica = Replica::new(PEERS[0]);
        for frame in frames.iter().rev() {
            let Message::Frame(decoded) =
                Message::decode(&Message::Frame(*frame).encode()).unwrap()
            else {
                unreachable!()
            };
            replica.receive(PEERS[0], decoded).unwrap();
            replica.receive(PEERS[0], decoded).unwrap();
        }
        assert_eq!(replica.game(), &expected);
    }
}

#[test]
fn checkpoint_plus_suffix_and_restore_plus_replay_match_at_every_tick() {
    for seed in 0..16 {
        let (frames, expected) = history(seed, 80);
        for cut in 0..=frames.len() {
            let mut original = Counter::default();
            for frame in &frames[..cut] {
                original.tick(frame.tick, frame.command.as_slice()).unwrap();
                assert_eq!(original.checksum(), frame.checksum);
            }
            let checkpoint = original.snapshot();
            let mut recovered = Counter::default();
            recovered.restore(&checkpoint).unwrap();
            for frame in &frames[cut..] {
                recovered
                    .tick(frame.tick, frame.command.as_slice())
                    .unwrap();
                assert_eq!(recovered.checksum(), frame.checksum);
            }
            assert_eq!(recovered, expected);
            recovered.restore(&checkpoint).unwrap();
            for frame in &frames[cut..] {
                recovered
                    .tick(frame.tick, frame.command.as_slice())
                    .unwrap();
            }
            assert_eq!(recovered, expected);
        }
    }
}

#[test]
fn authority_rejects_spoofing_stale_turns_and_future_acks() {
    assert!(matches!(
        Host::new([PEERS[0], PEERS[0]]),
        Err(SessionError::InvalidPeers)
    ));
    let mut host = Host::new(PEERS).unwrap();
    assert_eq!(
        host.propose(PeerId(99), Sequence(0), Action::Increment),
        Err(SessionError::Unauthorized)
    );
    assert_eq!(
        host.propose(PEERS[1], Sequence(0), Action::Increment),
        Err(SessionError::Unauthorized)
    );
    host.propose(PEERS[0], Sequence(0), Action::Increment)
        .unwrap();
    assert_eq!(
        host.propose(PEERS[0], Sequence(0), Action::Increment),
        Err(SessionError::PendingCommand)
    );
    host.advance().unwrap();
    assert_eq!(
        host.propose(PEERS[1], Sequence(0), Action::Increment),
        Err(SessionError::StaleTurn)
    );
    // Turn staleness is reported before the player check.
    assert_eq!(
        host.propose(PEERS[0], Sequence(0), Action::Increment),
        Err(SessionError::StaleTurn)
    );
    assert_eq!(
        host.acknowledge(PEERS[1], Tick(u64::MAX)),
        Err(SessionError::InvalidAck)
    );
    assert_eq!(
        host.acknowledge(PEERS[0], Tick(1)),
        Err(SessionError::Unauthorized)
    );
    host.acknowledge(PEERS[1], Tick(1)).unwrap();
    host.acknowledge(PEERS[1], Tick(0)).unwrap(); // old ACK cannot move backwards
    assert!(host.unacknowledged().is_empty());
}

#[test]
fn desync_and_invalid_frames_do_not_corrupt_state() {
    let (frames, _) = history(3, 2);
    let mut replica = Replica::new(PEERS[0]);
    assert_eq!(
        replica.receive(PEERS[1], frames[0]),
        Err(SessionError::Unauthorized)
    );
    let mut bad = frames[0];
    bad.checksum.0 ^= 1;
    assert_eq!(replica.receive(PEERS[0], bad), Err(SessionError::Desync));
    assert_eq!(replica.game(), &Counter::default());
    replica.receive(PEERS[0], frames[0]).unwrap();
    bad.tick = Tick(u64::MAX);
    assert_eq!(
        replica.receive(PEERS[0], bad),
        Err(SessionError::OutsideWindow)
    );
}

#[test]
fn history_window_and_conflicts_are_bounded() {
    let mut host = Host::new(PEERS).unwrap();
    for _ in 0..MAX_HISTORY {
        host.advance().unwrap();
    }
    assert_eq!(host.advance(), Err(SessionError::HistoryFull));
    let mut replica = Replica::new(PEERS[0]);
    let future = host.frames()[1];
    replica.receive(PEERS[0], future).unwrap();
    let mut conflict = future;
    conflict.checksum.0 ^= 1;
    assert_eq!(
        replica.receive(PEERS[0], conflict),
        Err(SessionError::ConflictingFrame)
    );
    let mut outside = future;
    outside.tick = Tick(MAX_HISTORY as u64);
    assert_eq!(
        replica.receive(PEERS[0], outside),
        Err(SessionError::OutsideWindow)
    );
}

#[test]
fn protocol_is_canonical_bounded_and_rejects_truncation() {
    let messages = [
        Message::Proposal {
            turn: Sequence(u64::MAX),
            action: Action::Decrement,
        },
        Message::Frame(Frame {
            tick: Tick(0),
            command: None,
            checksum: Checksum(0),
        }),
        Message::Frame(Frame {
            tick: Tick(2),
            command: Some(Command {
                player: PlayerId(1),
                payload: Action::Increment,
            }),
            checksum: Checksum(u64::MAX),
        }),
        Message::Ack {
            next_tick: Tick(u64::MAX),
        },
    ];
    for message in messages {
        let mut bytes = message.encode();
        assert_eq!(Message::decode(&bytes), Ok(message));
        for len in 0..bytes.len() {
            assert!(Message::decode(&bytes[..len]).is_err());
        }
        bytes.push(0);
        assert!(Message::decode(&bytes).is_err());
    }
    assert_eq!(
        Message::Ack {
            next_tick: Tick(0x0102)
        }
        .encode(),
        vec![1, 2, 2, 1, 0, 0, 0, 0, 0, 0]
    );
    for version in 2..=255 {
        assert!(Message::decode(&[version, 2, 0, 0, 0, 0, 0, 0, 0, 0]).is_err());
    }
    let mut bytes = messages[2].encode();
    bytes[11] = 2; // invalid player
    assert!(Message::decode(&bytes).is_err());
    bytes[11] = 1;
    bytes[12] = 2; // invalid action
    assert!(Message::decode(&bytes).is_err());
    assert!(Message::decode(&vec![0; 65_536]).is_err());
}

#[test]
fn malformed_packet_corpus_never_panics_or_normalizes_invalid_bytes() {
    let mut rng = 1_u64;
    for len in 0..128 {
        for _ in 0..128 {
            let bytes: Vec<u8> = (0..len).map(|_| (lcg(&mut rng) >> 32) as u8).collect();
            if let Ok(message) = Message::decode(&bytes) {
                assert_eq!(message.encode(), bytes);
            }
        }
    }
}
