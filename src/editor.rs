//! Editor types and builder API.
//!
//! This module provides the main `Editor` type and `EditorBuilder` for
//! opening files in text editors.

use std::path::{Path, PathBuf};

use crate::command::build_command;
use crate::config::{EditorConfig, ResolveFrom, DEFAULT_RESOLVE_ORDER, ENV_ONLY_RESOLVE_ORDER};
use crate::detect::{
    detect_editor, find_editor, find_editor_by_kind, resolve_editor_with_order, DetectedEditor,
};
use crate::error::{Error, Result};

/// Known text editor types.
///
/// This enum represents all the text editors that opensesame knows how to
/// invoke with proper line:column positioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EditorKind {
    // VS Code family
    /// Visual Studio Code
    VsCode,
    /// VS Code Insiders
    VsCodeInsiders,
    /// VSCodium (open source VS Code)
    VSCodium,
    /// Cursor (AI-powered VS Code fork)
    Cursor,
    /// Windsurf (Codeium's editor)
    Windsurf,

    // Vim family
    /// Vim
    Vim,
    /// NeoVim
    NeoVim,
    /// Vi
    Vi,
    /// GVim (graphical Vim)
    GVim,

    // Emacs family
    /// GNU Emacs
    Emacs,
    /// Emacs Client
    EmacsClient,

    // Modern GUI editors
    /// Sublime Text
    Sublime,
    /// Zed
    Zed,
    /// Helix
    Helix,
    /// Atom (deprecated but still used)
    Atom,
    /// Kate (KDE)
    Kate,

    // Terminal editors
    /// GNU Nano
    Nano,

    // macOS editors
    /// TextMate
    TextMate,
    /// Xcode
    Xcode,

    // Windows editors
    /// Notepad++ (Windows)
    NotepadPlusPlus,
    /// Notepad (Windows, no line support)
    Notepad,

    // JetBrains family
    /// IntelliJ IDEA
    IntelliJ,
    /// WebStorm
    WebStorm,
    /// PhpStorm
    PhpStorm,
    /// PyCharm
    PyCharm,
    /// RubyMine
    RubyMine,
    /// GoLand
    GoLand,
    /// CLion
    CLion,
    /// Rider
    Rider,
    /// DataGrip
    DataGrip,
    /// Android Studio
    AndroidStudio,

    /// Unknown editor (will just pass file path)
    Unknown,
}

impl EditorKind {
    /// Parses an `EditorKind` from its string name.
    ///
    /// Accepts names like "VsCode", "NeoVim", "Vim", etc. The matching
    /// is case-insensitive and supports common variations.
    ///
    /// Returns `None` for unrecognized names.
    ///
    /// # Example
    ///
    /// ```rust
    /// use opensesame::EditorKind;
    ///
    /// assert_eq!(EditorKind::from_name("NeoVim"), Some(EditorKind::NeoVim));
    /// assert_eq!(EditorKind::from_name("vscode"), Some(EditorKind::VsCode));
    /// assert_eq!(EditorKind::from_name("unknown"), None);
    /// ```
    pub fn from_name(name: &str) -> Option<Self> {
        // Normalize: lowercase and remove common separators
        let normalized = name.to_lowercase().replace(['-', '_'], "");

        match normalized.as_str() {
            // VS Code family
            "vscode" | "visualstudiocode" | "code" => Some(Self::VsCode),
            "vscodeinsiders" | "codeinsiders" => Some(Self::VsCodeInsiders),
            "vscodium" | "codium" => Some(Self::VSCodium),
            "cursor" => Some(Self::Cursor),
            "windsurf" => Some(Self::Windsurf),

            // Vim family
            "vim" => Some(Self::Vim),
            "neovim" | "nvim" => Some(Self::NeoVim),
            "vi" => Some(Self::Vi),
            "gvim" | "mvim" => Some(Self::GVim),

            // Emacs family
            "emacs" | "gnuemacs" | "xemacs" => Some(Self::Emacs),
            "emacsclient" => Some(Self::EmacsClient),

            // Modern GUI editors
            "sublime" | "sublimetext" | "subl" => Some(Self::Sublime),
            "zed" => Some(Self::Zed),
            "helix" | "hx" => Some(Self::Helix),
            "atom" => Some(Self::Atom),
            "kate" => Some(Self::Kate),

            // Terminal editors
            "nano" => Some(Self::Nano),

            // macOS editors
            "textmate" | "mate" => Some(Self::TextMate),
            "xcode" | "xed" => Some(Self::Xcode),

            // Windows editors
            "notepadplusplus" | "notepad++" | "npp" => Some(Self::NotepadPlusPlus),
            "notepad" => Some(Self::Notepad),

            // JetBrains family
            "intellij" | "intellijidea" | "idea" => Some(Self::IntelliJ),
            "webstorm" => Some(Self::WebStorm),
            "phpstorm" | "pstorm" => Some(Self::PhpStorm),
            "pycharm" | "charm" => Some(Self::PyCharm),
            "rubymine" | "mine" => Some(Self::RubyMine),
            "goland" => Some(Self::GoLand),
            "clion" => Some(Self::CLion),
            "rider" => Some(Self::Rider),
            "datagrip" => Some(Self::DataGrip),
            "androidstudio" | "studio" => Some(Self::AndroidStudio),

            _ => None,
        }
    }

    /// Returns the canonical string name for this editor kind.
    ///
    /// This is the preferred name for display and serialization.
    ///
    /// # Example
    ///
    /// ```rust
    /// use opensesame::EditorKind;
    ///
    /// assert_eq!(EditorKind::NeoVim.as_str(), "NeoVim");
    /// assert_eq!(EditorKind::VsCode.as_str(), "VsCode");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::VsCode => "VsCode",
            Self::VsCodeInsiders => "VsCodeInsiders",
            Self::VSCodium => "VSCodium",
            Self::Cursor => "Cursor",
            Self::Windsurf => "Windsurf",
            Self::Vim => "Vim",
            Self::NeoVim => "NeoVim",
            Self::Vi => "Vi",
            Self::GVim => "GVim",
            Self::Emacs => "Emacs",
            Self::EmacsClient => "EmacsClient",
            Self::Sublime => "Sublime",
            Self::Zed => "Zed",
            Self::Helix => "Helix",
            Self::Atom => "Atom",
            Self::Kate => "Kate",
            Self::Nano => "Nano",
            Self::TextMate => "TextMate",
            Self::Xcode => "Xcode",
            Self::NotepadPlusPlus => "NotepadPlusPlus",
            Self::Notepad => "Notepad",
            Self::IntelliJ => "IntelliJ",
            Self::WebStorm => "WebStorm",
            Self::PhpStorm => "PhpStorm",
            Self::PyCharm => "PyCharm",
            Self::RubyMine => "RubyMine",
            Self::GoLand => "GoLand",
            Self::CLion => "CLion",
            Self::Rider => "Rider",
            Self::DataGrip => "DataGrip",
            Self::AndroidStudio => "AndroidStudio",
            Self::Unknown => "Unknown",
        }
    }

    /// Detects the editor kind from a binary name.
    ///
    /// This handles both bare binary names (`vim`) and full paths
    /// (`/usr/bin/vim`), extracting just the filename for comparison.
    pub fn from_binary(binary: &str) -> Self {
        let name = Path::new(binary)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(binary)
            .to_lowercase();

        // Strip common extensions on Windows
        let name = name
            .strip_suffix(".exe")
            .or_else(|| name.strip_suffix(".cmd"))
            .or_else(|| name.strip_suffix(".bat"))
            .unwrap_or(&name);

        match name {
            // VS Code family
            "code" | "vscode" => Self::VsCode,
            "code-insiders" => Self::VsCodeInsiders,
            "codium" | "vscodium" | "code-oss" => Self::VSCodium,
            "cursor" => Self::Cursor,
            "windsurf" => Self::Windsurf,

            // Vim family
            "vim" => Self::Vim,
            "nvim" | "neovim" => Self::NeoVim,
            "vi" => Self::Vi,
            "gvim" | "mvim" => Self::GVim,

            // Emacs family
            "emacs" | "xemacs" => Self::Emacs,
            "emacsclient" => Self::EmacsClient,

            // Modern GUI editors
            "subl" | "sublime" | "sublime_text" => Self::Sublime,
            "zed" => Self::Zed,
            "hx" | "helix" => Self::Helix,
            "atom" => Self::Atom,
            "kate" => Self::Kate,

            // Terminal editors
            "nano" => Self::Nano,

            // macOS editors
            "mate" | "textmate" => Self::TextMate,
            "xed" | "xcode" => Self::Xcode,

            // Windows editors
            "notepad++" => Self::NotepadPlusPlus,
            "notepad" => Self::Notepad,

            // JetBrains family
            "idea" | "intellij" | "idea64" => Self::IntelliJ,
            "webstorm" | "webstorm64" => Self::WebStorm,
            "pstorm" | "phpstorm" | "phpstorm64" => Self::PhpStorm,
            "pycharm" | "pycharm64" | "charm" => Self::PyCharm,
            "rubymine" | "mine" => Self::RubyMine,
            "goland" | "goland64" => Self::GoLand,
            "clion" | "clion64" => Self::CLion,
            "rider" | "rider64" => Self::Rider,
            "datagrip" | "datagrip64" => Self::DataGrip,
            "studio" | "studio64" | "android-studio" => Self::AndroidStudio,

            _ => Self::Unknown,
        }
    }

    /// Returns the default binary name for this editor kind.
    pub const fn default_binary(&self) -> &'static str {
        match self {
            Self::VsCode => "code",
            Self::VsCodeInsiders => "code-insiders",
            Self::VSCodium => "codium",
            Self::Cursor => "cursor",
            Self::Windsurf => "windsurf",
            Self::Vim => "vim",
            Self::NeoVim => "nvim",
            Self::Vi => "vi",
            Self::GVim => "gvim",
            Self::Emacs => "emacs",
            Self::EmacsClient => "emacsclient",
            Self::Sublime => "subl",
            Self::Zed => "zed",
            Self::Helix => "hx",
            Self::Atom => "atom",
            Self::Kate => "kate",
            Self::Nano => "nano",
            Self::TextMate => "mate",
            Self::Xcode => "xed",
            Self::NotepadPlusPlus => "notepad++",
            Self::Notepad => "notepad",
            Self::IntelliJ => "idea",
            Self::WebStorm => "webstorm",
            Self::PhpStorm => "pstorm",
            Self::PyCharm => "pycharm",
            Self::RubyMine => "rubymine",
            Self::GoLand => "goland",
            Self::CLion => "clion",
            Self::Rider => "rider",
            Self::DataGrip => "datagrip",
            Self::AndroidStudio => "studio",
            Self::Unknown => "unknown",
        }
    }

    /// Returns `true` if this editor runs in the terminal (requires TTY).
    pub const fn is_terminal_editor(&self) -> bool {
        matches!(self, Self::Vim | Self::NeoVim | Self::Vi | Self::Nano | Self::Emacs | Self::Helix)
    }

    /// Returns `true` if this editor supports column positioning.
    pub const fn supports_column(&self) -> bool {
        matches!(
            self,
            Self::VsCode
                | Self::VsCodeInsiders
                | Self::VSCodium
                | Self::Cursor
                | Self::Windsurf
                | Self::Vim
                | Self::NeoVim
                | Self::Vi
                | Self::GVim
                | Self::Emacs
                | Self::EmacsClient
                | Self::Sublime
                | Self::Zed
                | Self::Helix
                | Self::Atom
                | Self::Kate
                | Self::Nano
                | Self::NotepadPlusPlus
        )
    }

    /// Returns `true` if this editor supports the `--wait` flag.
    pub const fn supports_wait(&self) -> bool {
        matches!(
            self,
            Self::VsCode
                | Self::VsCodeInsiders
                | Self::VSCodium
                | Self::Cursor
                | Self::Windsurf
                | Self::Sublime
                | Self::Zed
                | Self::Atom
                | Self::TextMate
                | Self::Xcode
                | Self::IntelliJ
                | Self::WebStorm
                | Self::PhpStorm
                | Self::PyCharm
                | Self::RubyMine
                | Self::GoLand
                | Self::CLion
                | Self::Rider
                | Self::DataGrip
                | Self::AndroidStudio
        )
    }
}

impl std::fmt::Display for EditorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::VsCode => "VS Code",
            Self::VsCodeInsiders => "VS Code Insiders",
            Self::VSCodium => "VSCodium",
            Self::Cursor => "Cursor",
            Self::Windsurf => "Windsurf",
            Self::Vim => "Vim",
            Self::NeoVim => "NeoVim",
            Self::Vi => "Vi",
            Self::GVim => "GVim",
            Self::Emacs => "Emacs",
            Self::EmacsClient => "Emacs Client",
            Self::Sublime => "Sublime Text",
            Self::Zed => "Zed",
            Self::Helix => "Helix",
            Self::Atom => "Atom",
            Self::Kate => "Kate",
            Self::Nano => "Nano",
            Self::TextMate => "TextMate",
            Self::Xcode => "Xcode",
            Self::NotepadPlusPlus => "Notepad++",
            Self::Notepad => "Notepad",
            Self::IntelliJ => "IntelliJ IDEA",
            Self::WebStorm => "WebStorm",
            Self::PhpStorm => "PhpStorm",
            Self::PyCharm => "PyCharm",
            Self::RubyMine => "RubyMine",
            Self::GoLand => "GoLand",
            Self::CLion => "CLion",
            Self::Rider => "Rider",
            Self::DataGrip => "DataGrip",
            Self::AndroidStudio => "Android Studio",
            Self::Unknown => "Unknown Editor",
        };
        write!(f, "{name}")
    }
}

/// Main entry point for opening files in editors.
///
/// Provides both simple functions and a builder pattern for more control.
///
/// # Examples
///
/// ```rust,no_run
/// use opensesame::Editor;
///
/// // Simple API
/// Editor::open("src/main.rs")?;
/// Editor::open_at("src/main.rs", 42)?;
/// Editor::open_at_position("src/main.rs", 42, 10)?;
///
/// // Builder API
/// Editor::builder()
///     .file("src/main.rs")
///     .line(42)
///     .column(10)
///     .wait(true)
///     .open()?;
/// # Ok::<(), opensesame::Error>(())
/// ```
pub struct Editor;

impl Editor {
    /// Creates a new editor builder.
    ///
    /// Use this for fine-grained control over how files are opened.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use opensesame::Editor;
    ///
    /// Editor::builder()
    ///     .file("src/main.rs")
    ///     .line(42)
    ///     .open()?;
    /// # Ok::<(), opensesame::Error>(())
    /// ```
    pub fn builder() -> EditorBuilder {
        EditorBuilder::new()
    }

    /// Opens a file in the default editor.
    ///
    /// This is the simplest way to open a file. The editor is detected
    /// automatically from `$VISUAL`, `$EDITOR`, or by searching PATH.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use opensesame::Editor;
    ///
    /// Editor::open("src/main.rs")?;
    /// # Ok::<(), opensesame::Error>(())
    /// ```
    pub fn open(file: impl AsRef<Path>) -> Result<()> {
        Self::builder().file(file).open()
    }

    /// Opens a file at a specific line number.
    ///
    /// The line number is 1-indexed (first line is 1).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use opensesame::Editor;
    ///
    /// // Open at line 42
    /// Editor::open_at("src/main.rs", 42)?;
    /// # Ok::<(), opensesame::Error>(())
    /// ```
    pub fn open_at(file: impl AsRef<Path>, line: u32) -> Result<()> {
        Self::builder().file(file).line(line).open()
    }

    /// Opens a file at a specific line and column.
    ///
    /// Both line and column numbers are 1-indexed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use opensesame::Editor;
    ///
    /// // Open at line 42, column 10
    /// Editor::open_at_position("src/main.rs", 42, 10)?;
    /// # Ok::<(), opensesame::Error>(())
    /// ```
    pub fn open_at_position(file: impl AsRef<Path>, line: u32, column: u32) -> Result<()> {
        Self::builder().file(file).line(line).column(column).open()
    }

    /// Detects the default editor without opening anything.
    ///
    /// Useful for checking which editor would be used.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use opensesame::Editor;
    ///
    /// let editor = Editor::detect()?;
    /// println!("Default editor: {}", editor);
    /// # Ok::<(), opensesame::Error>(())
    /// ```
    pub fn detect() -> Result<EditorKind> {
        let detected = detect_editor()?;
        Ok(detected.kind)
    }
}

/// Builder for opening files in editors with fine-grained control.
///
/// # Example
///
/// ```rust,no_run
/// use opensesame::{Editor, EditorKind};
///
/// Editor::builder()
///     .file("src/main.rs")
///     .editor(EditorKind::VsCode)
///     .line(42)
///     .column(10)
///     .wait(true)
///     .open()?;
/// # Ok::<(), opensesame::Error>(())
/// ```
///
/// # Configuration
///
/// Use [`with_config()`](Self::with_config) to provide editor preferences from your application's config:
///
/// ```rust,no_run
/// use opensesame::{Editor, EditorConfig};
///
/// let config = EditorConfig::with_editor("nvim");
///
/// Editor::builder()
///     .file("src/main.rs")
///     .with_config(config)
///     .open()?;
/// # Ok::<(), opensesame::Error>(())
/// ```
#[derive(Debug, Default)]
pub struct EditorBuilder {
    file: Option<PathBuf>,
    line: Option<u32>,
    column: Option<u32>,
    wait: bool,
    editor: Option<EditorSpec>,
    /// Configs in priority order (first = highest priority).
    configs: Vec<EditorConfig>,
    /// Custom resolution order.
    resolve_order: Option<Vec<ResolveFrom>>,
}

/// Specification for which editor to use.
#[derive(Debug)]
enum EditorSpec {
    Kind(EditorKind),
    Binary(String),
}

impl EditorBuilder {
    /// Creates a new editor builder with default settings.
    fn new() -> Self {
        Self::default()
    }

    /// Sets the file to open.
    ///
    /// This is required before calling `open()`.
    pub fn file(mut self, path: impl AsRef<Path>) -> Self {
        self.file = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the line number to open at (1-indexed).
    ///
    /// If the editor doesn't support line positioning, this is ignored.
    pub const fn line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    /// Sets the column number to open at (1-indexed).
    ///
    /// If the editor doesn't support column positioning, this is ignored.
    /// Requires `line()` to also be set.
    pub const fn column(mut self, column: u32) -> Self {
        self.column = Some(column);
        self
    }

    /// Sets whether to wait for the editor to close before returning.
    ///
    /// Not all editors support this. For editors that don't, this is ignored.
    pub const fn wait(mut self, wait: bool) -> Self {
        self.wait = wait;
        self
    }

    /// Specifies which editor to use by kind.
    ///
    /// If not specified, the editor is detected automatically.
    pub fn editor(mut self, kind: EditorKind) -> Self {
        self.editor = Some(EditorSpec::Kind(kind));
        self
    }

    /// Specifies which editor to use by binary name.
    ///
    /// This is useful for editors not in the `EditorKind` enum.
    pub fn editor_binary(mut self, binary: impl Into<String>) -> Self {
        self.editor = Some(EditorSpec::Binary(binary.into()));
        self
    }

    /// Adds a configuration to be checked during editor resolution.
    ///
    /// Multiple configs can be added. They are checked in the order added,
    /// with earlier configs taking priority.
    ///
    /// The resolution order when configs are present (and no explicit editor is set):
    /// 1. Configs (in order added)
    /// 2. `$VISUAL` environment variable
    /// 3. `$EDITOR` environment variable
    /// 4. PATH search
    ///
    /// Use [`resolve_order()`](Self::resolve_order) to customize this behavior.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use opensesame::{Editor, EditorConfig};
    ///
    /// let user_config = EditorConfig::with_editor("nvim");
    /// let app_defaults = EditorConfig::with_editor("code");
    ///
    /// Editor::builder()
    ///     .file("test.rs")
    ///     .with_config(user_config)    // Checked first
    ///     .with_config(app_defaults)   // Checked second (fallback)
    ///     .open()?;
    /// # Ok::<(), opensesame::Error>(())
    /// ```
    pub fn with_config(mut self, config: EditorConfig) -> Self {
        self.configs.push(config);
        self
    }

    /// Sets the order in which editor sources are checked.
    ///
    /// By default, when configs are provided, the order is:
    /// `[Config, Visual, Editor, PathSearch]`
    ///
    /// Without configs, the legacy order is used:
    /// `[Visual, Editor, PathSearch]`
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use opensesame::{Editor, ResolveFrom};
    ///
    /// // Ignore environment variables, only use PATH search
    /// Editor::builder()
    ///     .file("test.rs")
    ///     .resolve_order(&[ResolveFrom::PathSearch])
    ///     .open()?;
    /// # Ok::<(), opensesame::Error>(())
    /// ```
    ///
    /// # Predefined Orders
    ///
    /// - [`DEFAULT_RESOLVE_ORDER`](crate::DEFAULT_RESOLVE_ORDER): `[Config, Visual, Editor, PathSearch]`
    /// - [`ENV_ONLY_RESOLVE_ORDER`](crate::ENV_ONLY_RESOLVE_ORDER): `[Visual, Editor, PathSearch]`
    pub fn resolve_order(mut self, order: &[ResolveFrom]) -> Self {
        self.resolve_order = Some(order.to_vec());
        self
    }

    /// Opens the file in the editor.
    ///
    /// This method spawns the editor process. For GUI editors, it returns
    /// immediately. For terminal editors, it blocks until the editor closes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No file was specified
    /// - The file doesn't exist
    /// - No editor could be found
    /// - The editor failed to start
    pub fn open(self) -> Result<()> {
        // Validate file is specified
        let file = self.file.clone().ok_or(Error::NoFileSpecified)?;

        // Validate position (must be >= 1)
        if let Some(line) = self.line {
            if line == 0 {
                return Err(Error::InvalidPosition);
            }
        }
        if let Some(column) = self.column {
            if column == 0 {
                return Err(Error::InvalidPosition);
            }
        }

        // Resolve the editor
        let editor = self.resolve_editor()?;

        // Build and execute the command
        let mut cmd = build_command(&editor, &file, self.line, self.column, self.wait);

        // Execute
        let status = cmd.status().map_err(|e| Error::SpawnFailed {
            binary: editor.binary.clone(),
            source: e,
        })?;

        // Check exit status
        if !status.success() {
            if let Some(code) = status.code() {
                return Err(Error::EditorFailed {
                    binary: editor.binary,
                    status: code,
                });
            }
            return Err(Error::EditorTerminated {
                binary: editor.binary,
            });
        }

        Ok(())
    }

    /// Resolves which editor to use.
    fn resolve_editor(&self) -> Result<DetectedEditor> {
        // If an explicit editor was set via .editor() or .editor_binary(), use it
        // This always takes highest priority and bypasses all resolution logic
        if let Some(ref spec) = self.editor {
            return match spec {
                EditorSpec::Kind(kind) => find_editor_by_kind(*kind),
                EditorSpec::Binary(binary) => find_editor(binary),
            };
        }

        // Determine the resolution order
        let order = if let Some(ref custom_order) = self.resolve_order {
            // Use custom order if explicitly set
            custom_order.as_slice()
        } else if !self.configs.is_empty() {
            // With configs, use default order (includes Config)
            DEFAULT_RESOLVE_ORDER
        } else {
            // Without configs, use legacy behavior (env vars + PATH)
            ENV_ONLY_RESOLVE_ORDER
        };

        resolve_editor_with_order(order, &self.configs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_kind_from_binary() {
        assert_eq!(EditorKind::from_binary("code"), EditorKind::VsCode);
        assert_eq!(EditorKind::from_binary("code.exe"), EditorKind::VsCode);
        assert_eq!(EditorKind::from_binary("code.cmd"), EditorKind::VsCode);
        assert_eq!(EditorKind::from_binary("/usr/bin/code"), EditorKind::VsCode);
        assert_eq!(EditorKind::from_binary("vim"), EditorKind::Vim);
        assert_eq!(EditorKind::from_binary("nvim"), EditorKind::NeoVim);
        assert_eq!(EditorKind::from_binary("emacs"), EditorKind::Emacs);
        assert_eq!(EditorKind::from_binary("subl"), EditorKind::Sublime);
        assert_eq!(EditorKind::from_binary("zed"), EditorKind::Zed);
        assert_eq!(EditorKind::from_binary("hx"), EditorKind::Helix);
        assert_eq!(EditorKind::from_binary("nano"), EditorKind::Nano);
        assert_eq!(EditorKind::from_binary("cursor"), EditorKind::Cursor);
        assert_eq!(EditorKind::from_binary("windsurf"), EditorKind::Windsurf);
        assert_eq!(EditorKind::from_binary("notepad++"), EditorKind::NotepadPlusPlus);
        assert_eq!(EditorKind::from_binary("idea"), EditorKind::IntelliJ);
        assert_eq!(EditorKind::from_binary("unknown-editor"), EditorKind::Unknown);
    }

    #[test]
    fn test_editor_kind_display() {
        assert_eq!(EditorKind::VsCode.to_string(), "VS Code");
        assert_eq!(EditorKind::NeoVim.to_string(), "NeoVim");
        assert_eq!(EditorKind::Helix.to_string(), "Helix");
    }

    #[test]
    fn test_editor_kind_properties() {
        assert!(EditorKind::Vim.is_terminal_editor());
        assert!(EditorKind::NeoVim.is_terminal_editor());
        assert!(EditorKind::Nano.is_terminal_editor());
        assert!(!EditorKind::VsCode.is_terminal_editor());

        assert!(EditorKind::VsCode.supports_column());
        assert!(EditorKind::Vim.supports_column());
        assert!(!EditorKind::TextMate.supports_column());
        assert!(!EditorKind::IntelliJ.supports_column());

        assert!(EditorKind::VsCode.supports_wait());
        assert!(!EditorKind::Vim.supports_wait());
    }

    #[test]
    fn test_builder_no_file_error() {
        let result = Editor::builder().open();
        assert!(matches!(result, Err(Error::NoFileSpecified)));
    }

    #[test]
    fn test_builder_invalid_position() {
        let result = Editor::builder()
            .file("test.rs")
            .line(0)
            .open();
        assert!(matches!(result, Err(Error::InvalidPosition)));

        let result = Editor::builder()
            .file("test.rs")
            .line(1)
            .column(0)
            .open();
        assert!(matches!(result, Err(Error::InvalidPosition)));
    }

    #[test]
    fn test_editor_kind_from_name() {
        // Case insensitive
        assert_eq!(EditorKind::from_name("NeoVim"), Some(EditorKind::NeoVim));
        assert_eq!(EditorKind::from_name("neovim"), Some(EditorKind::NeoVim));
        assert_eq!(EditorKind::from_name("NEOVIM"), Some(EditorKind::NeoVim));

        // With separators
        assert_eq!(EditorKind::from_name("vs-code"), Some(EditorKind::VsCode));
        assert_eq!(EditorKind::from_name("vs_code"), Some(EditorKind::VsCode));

        // Aliases
        assert_eq!(EditorKind::from_name("nvim"), Some(EditorKind::NeoVim));
        assert_eq!(EditorKind::from_name("code"), Some(EditorKind::VsCode));
        assert_eq!(EditorKind::from_name("subl"), Some(EditorKind::Sublime));
        assert_eq!(EditorKind::from_name("hx"), Some(EditorKind::Helix));
        assert_eq!(EditorKind::from_name("idea"), Some(EditorKind::IntelliJ));

        // Unknown returns None
        assert_eq!(EditorKind::from_name("unknown"), None);
        assert_eq!(EditorKind::from_name(""), None);
    }

    #[test]
    fn test_editor_kind_as_str() {
        assert_eq!(EditorKind::VsCode.as_str(), "VsCode");
        assert_eq!(EditorKind::NeoVim.as_str(), "NeoVim");
        assert_eq!(EditorKind::Helix.as_str(), "Helix");
        assert_eq!(EditorKind::IntelliJ.as_str(), "IntelliJ");
        assert_eq!(EditorKind::Unknown.as_str(), "Unknown");
    }

    #[test]
    fn test_editor_kind_roundtrip() {
        // Test that from_name(as_str()) returns the same kind
        let kinds = [
            EditorKind::VsCode,
            EditorKind::NeoVim,
            EditorKind::Vim,
            EditorKind::Emacs,
            EditorKind::Sublime,
            EditorKind::Zed,
            EditorKind::Helix,
            EditorKind::Cursor,
            EditorKind::Windsurf,
            EditorKind::IntelliJ,
        ];

        for kind in kinds {
            let name = kind.as_str();
            let parsed = EditorKind::from_name(name);
            assert_eq!(parsed, Some(kind), "roundtrip failed for {kind:?}");
        }
    }

    #[test]
    fn test_builder_with_config_stores_config() {
        let config = EditorConfig::with_editor("nvim");
        let builder = Editor::builder()
            .file("test.rs")
            .with_config(config);

        assert_eq!(builder.configs.len(), 1);
        assert_eq!(builder.configs[0].editor.as_deref(), Some("nvim"));
    }

    #[test]
    fn test_builder_with_multiple_configs() {
        let config1 = EditorConfig::with_editor("nvim");
        let config2 = EditorConfig::with_editor("code");
        let builder = Editor::builder()
            .file("test.rs")
            .with_config(config1)
            .with_config(config2);

        assert_eq!(builder.configs.len(), 2);
        assert_eq!(builder.configs[0].editor.as_deref(), Some("nvim"));
        assert_eq!(builder.configs[1].editor.as_deref(), Some("code"));
    }

    #[test]
    fn test_builder_resolve_order_stores_order() {
        let builder = Editor::builder()
            .file("test.rs")
            .resolve_order(&[ResolveFrom::PathSearch, ResolveFrom::Visual]);

        assert!(builder.resolve_order.is_some());
        let order = builder.resolve_order.unwrap();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], ResolveFrom::PathSearch);
        assert_eq!(order[1], ResolveFrom::Visual);
    }

    #[test]
    fn test_builder_default_has_empty_configs() {
        let builder = Editor::builder();
        assert!(builder.configs.is_empty());
        assert!(builder.resolve_order.is_none());
    }
}
