//! Configuration types for editor resolution.
//!
//! This module provides types for configuring editor selection, allowing
//! applications to pass editor preferences from their own config files.
//!
//! # Example
//!
//! ```rust,no_run
//! use opensesame::{Editor, EditorConfig, ResolveFrom};
//!
//! // Create config programmatically
//! let config = EditorConfig {
//!     editor: Some("nvim".to_string()),
//!     args: vec!["--noplugin".to_string()],
//!     ..Default::default()
//! };
//!
//! // Use config with custom resolution order
//! Editor::builder()
//!     .file("src/main.rs")
//!     .with_config(config)
//!     .resolve_order(&[ResolveFrom::Config, ResolveFrom::Visual, ResolveFrom::PathSearch])
//!     .open()?;
//! # Ok::<(), opensesame::Error>(())
//! ```
//!
//! # Serde Support
//!
//! Enable the `serde` feature for config file deserialization:
//!
//! ```toml
//! [dependencies]
//! opensesame = { version = "0.1", features = ["serde"] }
//! ```
//!
//! Then in your application config:
//!
//! ```yaml
//! # Your app's config.yaml
//! opensesame:
//!   editor: nvim
//!   args: ["--noplugin"]
//! ```

use crate::editor::EditorKind;

/// Sources from which an editor can be resolved.
///
/// Used with [`EditorBuilder::resolve_order()`](crate::EditorBuilder::resolve_order)
/// to control the priority of editor detection.
///
/// # Example
///
/// ```rust,no_run
/// use opensesame::{Editor, ResolveFrom};
///
/// // Only use PATH search, ignore env vars and config
/// Editor::builder()
///     .file("src/main.rs")
///     .resolve_order(&[ResolveFrom::PathSearch])
///     .open()?;
/// # Ok::<(), opensesame::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ResolveFrom {
    /// Check configs passed via `.with_config()` (in order they were added).
    Config,
    /// Check `$VISUAL` environment variable.
    Visual,
    /// Check `$EDITOR` environment variable.
    Editor,
    /// Search PATH for known editors.
    PathSearch,
}

/// Default resolution order when configs are provided.
///
/// Order: Config, Visual, Editor, PathSearch
pub const DEFAULT_RESOLVE_ORDER: &[ResolveFrom] = &[
    ResolveFrom::Config,
    ResolveFrom::Visual,
    ResolveFrom::Editor,
    ResolveFrom::PathSearch,
];

/// Resolution order that ignores config (matches legacy behavior).
///
/// Order: Visual, Editor, PathSearch
pub const ENV_ONLY_RESOLVE_ORDER: &[ResolveFrom] = &[
    ResolveFrom::Visual,
    ResolveFrom::Editor,
    ResolveFrom::PathSearch,
];

/// Configuration for editor selection.
///
/// This struct is typically loaded from an application's config file and
/// passed to [`EditorBuilder::with_config()`](crate::EditorBuilder::with_config).
///
/// When the `serde` feature is enabled, this struct can be deserialized from
/// YAML, TOML, JSON, or any other format supported by serde.
///
/// # Fields
///
/// - `editor`: Binary name or path (e.g., "nvim", "/usr/local/bin/code")
/// - `editor_kind`: Alternative to `editor`, uses [`EditorKind`] string names
/// - `args`: Extra arguments to pass to the editor
///
/// # Example
///
/// ```rust
/// use opensesame::EditorConfig;
///
/// let config = EditorConfig {
///     editor: Some("nvim".to_string()),
///     args: vec!["--noplugin".to_string()],
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct EditorConfig {
    /// Editor binary name or path.
    ///
    /// This can be a simple binary name like "nvim" or a full path like
    /// "/usr/local/bin/code". The binary must exist in PATH (or at the
    /// specified path) for resolution to succeed.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub editor: Option<String>,

    /// Editor kind (alternative to `editor` field).
    ///
    /// Accepts string names like "VsCode", "NeoVim", "Helix", etc.
    /// The corresponding binary must be in PATH for resolution to succeed.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub editor_kind: Option<EditorKindConfig>,

    /// Extra arguments to pass to the editor.
    ///
    /// These are appended to the command after opensesame's positioning arguments.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub args: Vec<String>,
}

impl EditorConfig {
    /// Creates a new empty config.
    pub const fn new() -> Self {
        Self {
            editor: None,
            editor_kind: None,
            args: Vec::new(),
        }
    }

    /// Creates a config with the specified editor binary.
    pub fn with_editor(editor: impl Into<String>) -> Self {
        Self {
            editor: Some(editor.into()),
            editor_kind: None,
            args: Vec::new(),
        }
    }

    /// Creates a config with the specified editor kind.
    pub fn with_editor_kind(kind: EditorKind) -> Self {
        Self {
            editor: None,
            editor_kind: Some(EditorKindConfig(kind)),
            args: Vec::new(),
        }
    }

    /// Returns true if this config has no editor specified.
    pub const fn is_empty(&self) -> bool {
        self.editor.is_none() && self.editor_kind.is_none()
    }
}

/// Wrapper for [`EditorKind`] that supports serde string deserialization.
///
/// This allows config files to specify editors by name:
///
/// ```yaml
/// editor_kind: NeoVim
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorKindConfig(pub EditorKind);

impl From<EditorKind> for EditorKindConfig {
    fn from(kind: EditorKind) -> Self {
        Self(kind)
    }
}

impl From<EditorKindConfig> for EditorKind {
    fn from(config: EditorKindConfig) -> Self {
        config.0
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for EditorKindConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        EditorKind::from_name(&s)
            .map(EditorKindConfig)
            .ok_or_else(|| {
                serde::de::Error::unknown_variant(
                    &s,
                    &[
                        "VsCode",
                        "VsCodeInsiders",
                        "VSCodium",
                        "Cursor",
                        "Windsurf",
                        "Vim",
                        "NeoVim",
                        "Vi",
                        "GVim",
                        "Emacs",
                        "EmacsClient",
                        "Sublime",
                        "Zed",
                        "Helix",
                        "Atom",
                        "Kate",
                        "Nano",
                        "TextMate",
                        "Xcode",
                        "NotepadPlusPlus",
                        "Notepad",
                        "IntelliJ",
                        "WebStorm",
                        "PhpStorm",
                        "PyCharm",
                        "RubyMine",
                        "GoLand",
                        "CLion",
                        "Rider",
                        "DataGrip",
                        "AndroidStudio",
                    ],
                )
            })
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for EditorKindConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_config_default() {
        let config = EditorConfig::default();
        assert!(config.editor.is_none());
        assert!(config.editor_kind.is_none());
        assert!(config.args.is_empty());
        assert!(config.is_empty());
    }

    #[test]
    fn test_editor_config_with_editor() {
        let config = EditorConfig::with_editor("nvim");
        assert_eq!(config.editor.as_deref(), Some("nvim"));
        assert!(config.editor_kind.is_none());
        assert!(!config.is_empty());
    }

    #[test]
    fn test_editor_config_with_editor_kind() {
        let config = EditorConfig::with_editor_kind(EditorKind::NeoVim);
        assert!(config.editor.is_none());
        assert_eq!(config.editor_kind.unwrap().0, EditorKind::NeoVim);
        assert!(!config.is_empty());
    }

    #[test]
    fn test_editor_kind_config_conversion() {
        let kind = EditorKind::VsCode;
        let config: EditorKindConfig = kind.into();
        let back: EditorKind = config.into();
        assert_eq!(kind, back);
    }

    #[test]
    fn test_resolve_from_equality() {
        assert_eq!(ResolveFrom::Config, ResolveFrom::Config);
        assert_ne!(ResolveFrom::Config, ResolveFrom::Visual);
    }

    #[test]
    fn test_default_resolve_order() {
        assert_eq!(DEFAULT_RESOLVE_ORDER.len(), 4);
        assert_eq!(DEFAULT_RESOLVE_ORDER[0], ResolveFrom::Config);
        assert_eq!(DEFAULT_RESOLVE_ORDER[1], ResolveFrom::Visual);
        assert_eq!(DEFAULT_RESOLVE_ORDER[2], ResolveFrom::Editor);
        assert_eq!(DEFAULT_RESOLVE_ORDER[3], ResolveFrom::PathSearch);
    }

    #[test]
    fn test_env_only_resolve_order() {
        assert_eq!(ENV_ONLY_RESOLVE_ORDER.len(), 3);
        assert_eq!(ENV_ONLY_RESOLVE_ORDER[0], ResolveFrom::Visual);
        assert!(!ENV_ONLY_RESOLVE_ORDER.contains(&ResolveFrom::Config));
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;

    #[test]
    fn test_editor_config_json_roundtrip() {
        let config = EditorConfig {
            editor: Some("nvim".to_string()),
            args: vec!["--noplugin".to_string()],
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: EditorConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.editor, config.editor);
        assert_eq!(parsed.args, config.args);
    }

    #[test]
    fn test_editor_config_deserialize_minimal() {
        let json = r#"{"editor": "code"}"#;
        let config: EditorConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.editor.as_deref(), Some("code"));
        assert!(config.editor_kind.is_none());
        assert!(config.args.is_empty());
    }

    #[test]
    fn test_editor_config_deserialize_with_kind() {
        let json = r#"{"editor_kind": "NeoVim"}"#;
        let config: EditorConfig = serde_json::from_str(json).unwrap();

        assert!(config.editor.is_none());
        assert_eq!(config.editor_kind.unwrap().0, EditorKind::NeoVim);
    }

    #[test]
    fn test_editor_config_deserialize_case_insensitive_kind() {
        // Test various case formats
        let cases = [
            (r#"{"editor_kind": "neovim"}"#, EditorKind::NeoVim),
            (r#"{"editor_kind": "VSCODE"}"#, EditorKind::VsCode),
            (r#"{"editor_kind": "Helix"}"#, EditorKind::Helix),
        ];

        for (json, expected) in cases {
            let config: EditorConfig = serde_json::from_str(json).unwrap();
            assert_eq!(config.editor_kind.unwrap().0, expected);
        }
    }

    #[test]
    fn test_editor_kind_config_serialize() {
        let config = EditorConfig {
            editor_kind: Some(EditorKindConfig(EditorKind::VsCode)),
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("VsCode"));
    }

    #[test]
    fn test_editor_config_skip_empty_fields() {
        let config = EditorConfig::default();
        let json = serde_json::to_string(&config).unwrap();

        // Empty config should serialize to "{}"
        assert_eq!(json, "{}");
    }
}
