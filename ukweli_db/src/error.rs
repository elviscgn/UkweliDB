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

    #[error("Clock error: {0}")]
    ClockError(String),

    #[error("No signers provided")]
    NoSigners,

    #[error("Duplicate record detected")]
    DuplicateRecord,

    #[error("Timestamp out of acceptable range")]
    InvalidTimestamp,
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

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not a valid Ukweli database file")]
    InvalidMagic,

    #[error("Unsupported database version: {0}.{1}")]
    UnsupportedVersion(u8, u8),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Checksum mismatch - database file may be corrupted")]
    ChecksumMismatch,

    #[error("Database validation failed: {0}")]
    ValidationFailed(String),
}
