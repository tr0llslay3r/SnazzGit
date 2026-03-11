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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_error_display() {
        let err = GitError::General("something went wrong".to_string());
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn test_repo_not_found_display() {
        let err = GitError::RepoNotFound("/some/path".to_string());
        assert!(err.to_string().contains("/some/path"));
    }

    #[test]
    fn test_no_repo_open_display() {
        let err = GitError::NoRepoOpen;
        assert_eq!(err.to_string(), "No repository open");
    }

    #[test]
    fn test_invalid_path_display() {
        let err = GitError::InvalidPath("bad/path".to_string());
        assert!(err.to_string().contains("bad/path"));
    }

    #[test]
    fn test_error_serializes_to_message_string() {
        let err = GitError::General("test error".to_string());
        let serialized = serde_json::to_string(&err).unwrap();
        assert_eq!(serialized, "\"test error\"");
    }

    #[test]
    fn test_repo_not_found_serializes() {
        let err = GitError::RepoNotFound("/tmp/foo".to_string());
        let serialized = serde_json::to_string(&err).unwrap();
        assert!(serialized.contains("/tmp/foo"));
    }
}
