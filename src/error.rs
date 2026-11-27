use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("scan directory not found: {0}")]
    RootNotFound(PathBuf),

    #[error("text arena exceeded u32 capacity")]
    ArenaOverflow,

    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
}
