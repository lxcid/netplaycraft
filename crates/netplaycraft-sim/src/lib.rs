//! Engine-independent capabilities. Algorithms require only what they use.
#![no_std]
use netplaycraft_core::{Checksum, Tick};

pub trait Simulation {
    type Input;
    type Error;

    /// Apply one fixed logical step, including empty-input steps.
    /// Input ordering is supplied by the synchronization strategy.
    /// On error, implementations must leave their state unchanged.
    fn tick(&mut self, tick: Tick, inputs: &[Self::Input]) -> Result<(), Self::Error>;
}

/// Optional recovery/rollback capability, not required by Simulation.
pub trait Snapshotable {
    type Snapshot;
    type Error;

    fn snapshot(&self) -> Self::Snapshot;
    /// On error, leave state unchanged.
    fn restore(&mut self, snapshot: &Self::Snapshot) -> Result<(), Self::Error>;
}

/// Optional deterministic diagnostic capability; encoding is game-defined.
pub trait Checksummed {
    fn checksum(&self) -> Checksum;
}
