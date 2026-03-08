use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum GitError {
    #[error("Git error: {0}")]
    Git2(#[from] git2::Error),

    #[error("Repository not found at: {0}")]
    RepoNotFound(String),

    #[error("No repository open")]
    NoRepoOpen,

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Syntect error: {0}")]
    Syntect(#[from] syntect::Error),

    #[error("{0}")]
    General(String),
}

impl serde::Serialize for GitError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
