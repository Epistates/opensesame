//! Editor types and builder API.
//!
//! This module provides the main `Editor` type and `EditorBuilder` for
//! opening files in text editors.

use std::path::{Path, PathBuf};

use crate::command::build_command;
use crate::detect::{detect_editor, find_editor, find_editor_by_kind, DetectedEditor};
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
#[derive(Debug, Default)]
pub struct EditorBuilder {
    file: Option<PathBuf>,
    line: Option<u32>,
    column: Option<u32>,
    wait: bool,
    editor: Option<EditorSpec>,
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
        match &self.editor {
            Some(EditorSpec::Kind(kind)) => find_editor_by_kind(*kind),
            Some(EditorSpec::Binary(binary)) => find_editor(binary),
            None => detect_editor(),
        }
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
}
