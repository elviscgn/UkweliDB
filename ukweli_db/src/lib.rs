#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::indexing_slicing)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

pub mod core;
pub mod error;
pub mod storage;
pub mod workflow;

pub use core::{Ledger, Record};
pub use error::LedgerError;
pub use storage::persitence;
pub use workflow::{Workflow, WorkflowState};
