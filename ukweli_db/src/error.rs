use thiserror::Error;

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("User not registered on Ledger")]
    UnregistedUser,

    #[error("System error: Could not access previous record")]
    RecordAccessFailed,

    #[error("Payload cannot be empty")]
    EmptyPayload,

    #[error("{0}")]
    ChainValidation(String),
}

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("{0}")]
    Definition(String),

    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    Parsing(String),
}

#[derive(Error, Debug)]
pub enum EntityError {
    #[error("{0}")]
    Update(String),
}
