//! Editor detection module.
//!
//! This module handles finding the user's preferred editor through various
//! mechanisms: configuration, environment variables, and PATH search.

use crate::config::{EditorConfig, ResolveFrom};
use crate::editor::EditorKind;
use crate::error::{Error, Result};

/// Common editor binaries to search for, in order of preference.
///
/// This list is ordered by:
/// 1. Modern, feature-rich editors (VS Code, Cursor, Windsurf, Zed)
/// 2. Traditional terminal editors (nvim, vim, emacs)
/// 3. Simple editors (nano)
const FALLBACK_EDITORS: &[&str] = &[
    "code",      // VS Code
    "cursor",    // Cursor
    "windsurf",  // Windsurf
    "zed",       // Zed
    "nvim",      // NeoVim
    "vim",       // Vim
    "hx",        // Helix
    "emacs",     // Emacs
    "subl",      // Sublime Text
    "nano",      // Nano
    "vi",        // Vi (last resort)
];

/// Windows-specific fallback editors.
#[cfg(windows)]
const WINDOWS_FALLBACK_EDITORS: &[&str] = &[
    "code.cmd",
    "cursor.cmd",
    "notepad++",
    "notepad",
];

/// Detects the user's preferred editor.
///
/// Detection order:
/// 1. `$VISUAL` environment variable (preferred for GUI editors)
/// 2. `$EDITOR` environment variable (traditional editor variable)
/// 3. Search PATH for known editors
///
/// # Errors
///
/// Returns `Error::NoEditorFound` if no editor could be detected.
pub fn detect_editor() -> Result<DetectedEditor> {
    // Try $VISUAL first (preferred for visual/GUI editors)
    if let Some(editor) = try_env_var("VISUAL") {
        return Ok(editor);
    }

    // Try $EDITOR
    if let Some(editor) = try_env_var("EDITOR") {
        return Ok(editor);
    }

    // Search PATH for known editors
    if let Some(editor) = search_path_for_editor() {
        return Ok(editor);
    }

    Err(Error::NoEditorFound)
}

/// Resolves an editor using the specified resolution order.
///
/// This function checks each source in the given order and returns the first
/// editor that can be found. Use this when you need custom resolution behavior.
///
/// # Arguments
///
/// * `order` - The order in which to check sources
/// * `configs` - Configurations passed via [`EditorBuilder::with_config()`](crate::EditorBuilder::with_config)
///
/// # Errors
///
/// Returns `Error::NoEditorFound` if no editor could be detected from any source.
///
/// # Example
///
/// ```rust,ignore
/// use opensesame::detect::{resolve_editor_with_order, ResolveFrom};
///
/// let order = &[ResolveFrom::Config, ResolveFrom::PathSearch];
/// let configs = vec![EditorConfig::with_editor("nvim")];
/// let editor = resolve_editor_with_order(order, &configs)?;
/// ```
pub fn resolve_editor_with_order(
    order: &[ResolveFrom],
    configs: &[EditorConfig],
) -> Result<DetectedEditor> {
    for source in order {
        match source {
            ResolveFrom::Config => {
                for (index, config) in configs.iter().enumerate() {
                    if let Some(editor) = try_config(config, index) {
                        return Ok(editor);
                    }
                }
            }
            ResolveFrom::Visual => {
                if let Some(editor) = try_env_var("VISUAL") {
                    return Ok(editor);
                }
            }
            ResolveFrom::Editor => {
                if let Some(editor) = try_env_var("EDITOR") {
                    return Ok(editor);
                }
            }
            ResolveFrom::PathSearch => {
                if let Some(editor) = search_path_for_editor() {
                    return Ok(editor);
                }
            }
        }
    }

    Err(Error::NoEditorFound)
}

/// Attempts to create a `DetectedEditor` from an `EditorConfig`.
///
/// Returns `None` if the config doesn't specify an editor or the editor
/// isn't available.
fn try_config(config: &EditorConfig, index: usize) -> Option<DetectedEditor> {
    // Try editor binary first (more specific)
    if let Some(ref binary) = config.editor {
        if which::which(binary).is_ok() {
            return Some(DetectedEditor {
                binary: binary.clone(),
                kind: EditorKind::from_binary(binary),
                extra_args: config.args.clone(),
                source: EditorSource::Config { index },
            });
        }
    }

    // Try editor_kind (fallback to default binary)
    if let Some(ref kind_config) = config.editor_kind {
        let binary = kind_config.0.default_binary();
        if which::which(binary).is_ok() {
            return Some(DetectedEditor {
                binary: binary.to_string(),
                kind: kind_config.0,
                extra_args: config.args.clone(),
                source: EditorSource::Config { index },
            });
        }
    }

    None
}

/// Attempts to get an editor from an environment variable.
fn try_env_var(var: &str) -> Option<DetectedEditor> {
    let value = std::env::var(var).ok()?;
    let value = value.trim();

    if value.is_empty() {
        return None;
    }

    // Parse the editor command (may include arguments like "code --wait")
    let parts: Vec<&str> = value.split_whitespace().collect();
    let binary = (*parts.first()?).to_string();
    let args: Vec<String> = parts[1..].iter().map(|s| (*s).to_string()).collect();

    // Extract just the binary name for kind detection
    let binary_name = std::path::Path::new(&binary)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&binary);
    let kind = EditorKind::from_binary(binary_name);

    Some(DetectedEditor {
        binary,
        kind,
        extra_args: args,
        source: EditorSource::Environment(var.to_string()),
    })
}

/// Searches PATH for known editor binaries.
fn search_path_for_editor() -> Option<DetectedEditor> {
    for &binary in FALLBACK_EDITORS {
        if which::which(binary).is_ok() {
            return Some(DetectedEditor {
                binary: binary.to_string(),
                kind: EditorKind::from_binary(binary),
                extra_args: Vec::new(),
                source: EditorSource::PathSearch,
            });
        }
    }

    // Windows-specific fallbacks
    #[cfg(windows)]
    for &binary in WINDOWS_FALLBACK_EDITORS {
        if which::which(binary).is_ok() {
            return Some(DetectedEditor {
                binary: binary.to_string(),
                kind: EditorKind::from_binary(binary),
                extra_args: Vec::new(),
                source: EditorSource::PathSearch,
            });
        }
    }

    None
}

/// Finds a specific editor binary.
///
/// # Errors
///
/// Returns `Error::EditorNotFound` if the binary is not in PATH.
pub fn find_editor(binary: &str) -> Result<DetectedEditor> {
    // Check if it's in PATH
    if which::which(binary).is_err() {
        return Err(Error::EditorNotFound {
            binary: binary.to_string(),
        });
    }

    let binary_name = std::path::Path::new(binary)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(binary);

    Ok(DetectedEditor {
        binary: binary.to_string(),
        kind: EditorKind::from_binary(binary_name),
        extra_args: Vec::new(),
        source: EditorSource::Explicit,
    })
}

/// Creates a detected editor from an `EditorKind`.
///
/// # Errors
///
/// Returns `Error::EditorNotFound` if the editor binary is not in PATH.
pub fn find_editor_by_kind(kind: EditorKind) -> Result<DetectedEditor> {
    let binary = kind.default_binary();

    // Check if it's in PATH
    if which::which(binary).is_err() {
        return Err(Error::EditorNotFound {
            binary: binary.to_string(),
        });
    }

    Ok(DetectedEditor {
        binary: binary.to_string(),
        kind,
        extra_args: Vec::new(),
        source: EditorSource::Explicit,
    })
}

/// A detected editor with its metadata.
#[derive(Debug, Clone)]
pub struct DetectedEditor {
    /// The binary name or path.
    pub binary: String,
    /// The detected editor kind.
    pub kind: EditorKind,
    /// Extra arguments from environment variable (e.g., "--wait" from "$EDITOR=code --wait").
    pub extra_args: Vec<String>,
    /// How the editor was detected (useful for debugging/introspection).
    #[allow(dead_code)]
    pub source: EditorSource,
}

impl DetectedEditor {
    /// Returns `true` if this is a terminal-based editor (requires TTY).
    pub const fn is_terminal_editor(&self) -> bool {
        self.kind.is_terminal_editor()
    }
}

/// How an editor was detected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorSource {
    /// Detected from an environment variable.
    Environment(String),
    /// Found by searching PATH.
    PathSearch,
    /// Explicitly specified by the user.
    Explicit,
    /// From application configuration.
    Config {
        /// Index of the config in the resolution chain (0 = highest priority).
        index: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DEFAULT_RESOLVE_ORDER, ENV_ONLY_RESOLVE_ORDER};

    #[test]
    fn test_editor_source_equality() {
        assert_eq!(EditorSource::PathSearch, EditorSource::PathSearch);
        assert_eq!(
            EditorSource::Environment("VISUAL".to_string()),
            EditorSource::Environment("VISUAL".to_string())
        );
        assert_ne!(
            EditorSource::Environment("VISUAL".to_string()),
            EditorSource::Environment("EDITOR".to_string())
        );
    }

    #[test]
    fn test_editor_source_config_equality() {
        assert_eq!(
            EditorSource::Config { index: 0 },
            EditorSource::Config { index: 0 }
        );
        assert_ne!(
            EditorSource::Config { index: 0 },
            EditorSource::Config { index: 1 }
        );
        assert_ne!(EditorSource::Config { index: 0 }, EditorSource::PathSearch);
    }

    #[test]
    fn test_fallback_order() {
        // Verify our fallback list has the expected order
        assert_eq!(FALLBACK_EDITORS[0], "code");
        assert!(FALLBACK_EDITORS.contains(&"vim"));
        assert!(FALLBACK_EDITORS.contains(&"nano"));
    }

    #[test]
    fn test_resolve_with_empty_order_fails() {
        let result = resolve_editor_with_order(&[], &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_with_empty_configs_and_config_source() {
        // When Config is in order but no configs provided, should fall through
        let result = resolve_editor_with_order(&[ResolveFrom::Config], &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_config_with_empty_config() {
        let config = EditorConfig::default();
        let result = try_config(&config, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_default_resolve_order_has_config_first() {
        assert_eq!(DEFAULT_RESOLVE_ORDER[0], ResolveFrom::Config);
    }

    #[test]
    fn test_env_only_resolve_order_excludes_config() {
        assert!(!ENV_ONLY_RESOLVE_ORDER.contains(&ResolveFrom::Config));
    }
}
