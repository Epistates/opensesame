//! # opensesame
//!
//! A cross-platform Rust library for opening files in text editors with support
//! for line and column positioning.
//!
//! ## Features
//!
//! - **Cross-platform**: Works on macOS, Linux, and Windows
//! - **Smart editor detection**: Finds editors via `$VISUAL`, `$EDITOR`, or PATH
//! - **Line:column positioning**: Opens files at specific locations when supported
//! - **Comprehensive editor support**: VS Code, Vim, NeoVim, Emacs, Sublime Text,
//!   Zed, Helix, Nano, Cursor, Windsurf, JetBrains IDEs, Notepad++, and more
//! - **Ergonomic API**: Builder pattern for flexible configuration
//! - **Type-safe errors**: Rich error types for proper error handling
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use opensesame::Editor;
//!
//! // Open a file in the default editor
//! Editor::open("src/main.rs")?;
//!
//! // Open at a specific line
//! Editor::open_at("src/main.rs", 42)?;
//!
//! // Open at a specific line and column
//! Editor::open_at_position("src/main.rs", 42, 10)?;
//! # Ok::<(), opensesame::Error>(())
//! ```
//!
//! ## Builder API
//!
//! For more control, use the builder pattern:
//!
//! ```rust,no_run
//! use opensesame::Editor;
//!
//! Editor::builder()
//!     .file("src/main.rs")
//!     .line(42)
//!     .column(10)
//!     .wait(true)  // Wait for editor to close
//!     .open()?;
//! # Ok::<(), opensesame::Error>(())
//! ```
//!
//! ## Specifying an Editor
//!
//! You can specify which editor to use:
//!
//! ```rust,no_run
//! use opensesame::{Editor, EditorKind};
//!
//! // Use a specific editor
//! Editor::builder()
//!     .file("src/main.rs")
//!     .editor(EditorKind::VsCode)
//!     .line(42)
//!     .open()?;
//!
//! // Or by binary name
//! Editor::builder()
//!     .file("src/main.rs")
//!     .editor_binary("nvim")
//!     .line(42)
//!     .open()?;
//! # Ok::<(), opensesame::Error>(())
//! ```
//!
//! ## Supported Editors
//!
//! | Editor | Binary | Line:Column Support |
//! |--------|--------|---------------------|
//! | VS Code | `code` | ✓ |
//! | VS Code Insiders | `code-insiders` | ✓ |
//! | VSCodium | `codium` | ✓ |
//! | Cursor | `cursor` | ✓ |
//! | Windsurf | `windsurf` | ✓ |
//! | Vim | `vim` | ✓ |
//! | NeoVim | `nvim` | ✓ |
//! | Emacs | `emacs` | ✓ |
//! | Sublime Text | `subl` | ✓ |
//! | Zed | `zed` | ✓ |
//! | Helix | `hx` | ✓ |
//! | Nano | `nano` | ✓ |
//! | TextMate | `mate` | Line only |
//! | Notepad++ | `notepad++` | ✓ |
//! | JetBrains IDEs | `idea`, `webstorm`, etc. | Line only |
//! | Xcode | `xed` | Line only |
//!
//! ## Configuration
//!
//! Applications can pass editor configuration to opensesame:
//!
//! ```rust,no_run
//! use opensesame::{Editor, EditorConfig};
//!
//! let config = EditorConfig {
//!     editor: Some("nvim".to_string()),
//!     args: vec!["--noplugin".to_string()],
//!     ..Default::default()
//! };
//!
//! Editor::builder()
//!     .file("src/main.rs")
//!     .with_config(config)
//!     .open()?;
//! # Ok::<(), opensesame::Error>(())
//! ```
//!
//! ### Custom Resolution Order
//!
//! Control how editors are detected:
//!
//! ```rust,no_run
//! use opensesame::{Editor, ResolveFrom};
//!
//! // Ignore config, only use environment variables
//! Editor::builder()
//!     .file("src/main.rs")
//!     .resolve_order(&[ResolveFrom::Visual, ResolveFrom::Editor])
//!     .open()?;
//! # Ok::<(), opensesame::Error>(())
//! ```
//!
//! ### Serde Support
//!
//! Enable the `serde` feature for config file deserialization:
//!
//! ```toml
//! [dependencies]
//! opensesame = { version = "0.1", features = ["serde"] }
//! ```

mod command;
mod config;
mod detect;
mod editor;
mod error;

pub use config::{
    EditorConfig, EditorKindConfig, ResolveFrom, DEFAULT_RESOLVE_ORDER, ENV_ONLY_RESOLVE_ORDER,
};
pub use editor::{Editor, EditorBuilder, EditorKind};
pub use error::{Error, Result};
