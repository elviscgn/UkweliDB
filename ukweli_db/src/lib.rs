#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::indexing_slicing)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

pub mod core;
pub mod entity;
pub mod error;
pub mod workflow;

pub use core::{Ledger, Record};
pub use entity::{EntityState, Tracker};
pub use error::LedgerError;
pub use workflow::{Workflow, WorkflowState};
