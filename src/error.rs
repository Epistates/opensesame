//! Error types for opensesame.
//!
//! This module provides a rich error type that covers all failure modes
//! when opening files in editors.

use std::path::PathBuf;

/// A specialized Result type for opensesame operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when opening files in editors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No editor could be found.
    ///
    /// This occurs when:
    /// - `$VISUAL` and `$EDITOR` environment variables are not set
    /// - No known editor binary could be found in `PATH`
    #[error("no editor found: set $VISUAL or $EDITOR, or install a supported editor")]
    NoEditorFound,

    /// The specified editor binary was not found in PATH.
    #[error("editor not found: '{binary}' is not installed or not in PATH")]
    EditorNotFound {
        /// The binary name that was searched for.
        binary: String,
    },

    /// The specified file does not exist.
    #[error("file not found: {}", path.display())]
    FileNotFound {
        /// Path to the file that was not found.
        path: PathBuf,
    },

    /// The editor process failed to start.
    #[error("failed to start editor '{binary}': {source}")]
    SpawnFailed {
        /// The editor binary that failed to start.
        binary: String,
        /// The underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// The editor process exited with a non-zero status.
    #[error("editor '{binary}' exited with status {status}")]
    EditorFailed {
        /// The editor binary that failed.
        binary: String,
        /// The exit status code.
        status: i32,
    },

    /// The editor process was terminated by a signal.
    #[error("editor '{binary}' was terminated by signal")]
    EditorTerminated {
        /// The editor binary that was terminated.
        binary: String,
    },

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// No file was specified to open.
    #[error("no file specified: use .file() to specify a file path")]
    NoFileSpecified,

    /// Invalid position specified (line or column is 0).
    #[error("invalid position: line and column numbers must be >= 1")]
    InvalidPosition,

    /// Invalid configuration was provided.
    #[error("invalid editor configuration: {message}")]
    InvalidConfig {
        /// Description of the configuration error.
        message: String,
    },
}

impl Error {
    /// Returns `true` if this error indicates the editor was not found.
    pub const fn is_editor_not_found(&self) -> bool {
        matches!(self, Self::NoEditorFound | Self::EditorNotFound { .. })
    }

    /// Returns `true` if this error indicates the file was not found.
    pub const fn is_file_not_found(&self) -> bool {
        matches!(self, Self::FileNotFound { .. })
    }

    /// Returns `true` if this error indicates the editor process failed.
    pub const fn is_editor_failed(&self) -> bool {
        matches!(
            self,
            Self::SpawnFailed { .. } | Self::EditorFailed { .. } | Self::EditorTerminated { .. }
        )
    }

    /// Returns `true` if this error indicates invalid configuration.
    pub const fn is_invalid_config(&self) -> bool {
        matches!(self, Self::InvalidConfig { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::NoEditorFound;
        assert!(err.to_string().contains("no editor found"));

        let err = Error::EditorNotFound {
            binary: "vim".to_string(),
        };
        assert!(err.to_string().contains("vim"));

        let err = Error::FileNotFound {
            path: PathBuf::from("/tmp/test.txt"),
        };
        assert!(err.to_string().contains("/tmp/test.txt"));
    }

    #[test]
    fn test_error_predicates() {
        assert!(Error::NoEditorFound.is_editor_not_found());
        assert!(Error::EditorNotFound {
            binary: "vim".to_string()
        }
        .is_editor_not_found());

        assert!(Error::FileNotFound {
            path: PathBuf::from("/tmp/test.txt")
        }
        .is_file_not_found());

        assert!(Error::InvalidConfig {
            message: "test error".to_string()
        }
        .is_invalid_config());
    }

    #[test]
    fn test_invalid_config_display() {
        let err = Error::InvalidConfig {
            message: "editor field is empty".to_string(),
        };
        assert!(err.to_string().contains("invalid editor configuration"));
        assert!(err.to_string().contains("editor field is empty"));
    }
}
