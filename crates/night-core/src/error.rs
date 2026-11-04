//! 🌙 Project Night Strongly-Typed Domain Errors
//! Powered by `thiserror` for comprehensive, structured library error management.

use std::path::PathBuf;
use thiserror::Error;

/// Root error type for all Night desktop operations
#[derive(Error, Debug)]
pub enum NightError {
    #[error("IPC communication error: {0}")]
    Ipc(#[from] IpcError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Hardware probe failure on subsystem '{subsystem}': {message}")]
    Hardware {
        subsystem: &'static str,
        message: String,
    },

    #[error("Desktop session lifecycle error: {0}")]
    Session(String),

    #[error("Filesystem IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors occurring during IPC socket requests, responses, or status streaming
#[derive(Error, Debug)]
pub enum IpcError {
    #[error("Failed to connect to IPC socket at '{path}': {source}")]
    ConnectionFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Binary payload serialization error: {0}")]
    Encode(#[from] postcard::Error),

    #[error("Binary payload deserialization error: {0}")]
    Decode(postcard::Error),

    #[error("IPC server returned error response: {0}")]
    ServerError(String),

    #[error("IPC socket stream closed unexpectedly")]
    UnexpectedEof,

    #[error("IPC socket path unavailable or runtime directory unset")]
    SocketUnavailable,

    #[error("IPC operation timed out after {0:?}")]
    Timeout(std::time::Duration),
}

/// Errors occurring during configuration loading, parsing, or persistence
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file at '{path}': {source}")]
    ReadFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to write config file to '{path}': {source}")]
    WriteFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("TOML syntax error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("RON syntax error: {0}")]
    Ron(#[from] ron::error::SpannedError),
}

pub type Result<T, E = NightError> = std::result::Result<T, E>;

