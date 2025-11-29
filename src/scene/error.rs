use thiserror::Error;

#[derive(Debug, Error)]
pub enum SceneError {
    #[error("unknown entity: {0}")]
    UnknownEntity(u64),
    #[error("resource missing: {0}")]
    ResourceMissing(&'static str),
    #[error("invalid state: {0}")]
    InvalidState(&'static str),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("backend error: {0}")]
    Backend(&'static str),
}

pub type SceneResult<T> = Result<T, SceneError>;
