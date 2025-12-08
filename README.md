# opensesame

A cross-platform Rust library for opening files in text editors with line:column positioning support.

## Features

- **Cross-platform**: Works on macOS, Linux, and Windows
- **Smart editor detection**: Finds editors via `$VISUAL`, `$EDITOR`, or PATH search
- **Line:column positioning**: Opens files at specific locations when supported
- **Comprehensive editor support**: 25+ editors including VS Code, Vim, NeoVim, Emacs, Sublime Text, Zed, Helix, Cursor, Windsurf, JetBrains IDEs, and more
- **Ergonomic API**: Simple functions and builder pattern for flexibility
- **Type-safe errors**: Rich error types for proper error handling

## Installation

```toml
[dependencies]
opensesame = "0.1"
```

## Quick Start

```rust
use opensesame::Editor;

// Open a file in the default editor
Editor::open("src/main.rs")?;

// Open at a specific line (1-indexed)
Editor::open_at("src/main.rs", 42)?;

// Open at a specific line and column
Editor::open_at_position("src/main.rs", 42, 10)?;
```

## Builder API

For more control, use the builder pattern:

```rust
use opensesame::{Editor, EditorKind};

Editor::builder()
    .file("src/main.rs")
    .line(42)
    .column(10)
    .wait(true)  // Wait for editor to close
    .open()?;

// Specify a particular editor
Editor::builder()
    .file("src/main.rs")
    .editor(EditorKind::VsCode)
    .line(42)
    .open()?;

// Or by binary name
Editor::builder()
    .file("src/main.rs")
    .editor_binary("nvim")
    .line(42)
    .open()?;
```

## Supported Editors

| Editor | Binary | Line | Column | Wait |
|--------|--------|:----:|:------:|:----:|
| VS Code | `code` | ✓ | ✓ | ✓ |
| VS Code Insiders | `code-insiders` | ✓ | ✓ | ✓ |
| VSCodium | `codium` | ✓ | ✓ | ✓ |
| Cursor | `cursor` | ✓ | ✓ | ✓ |
| Windsurf | `windsurf` | ✓ | ✓ | ✓ |
| Vim | `vim` | ✓ | ✓ | - |
| NeoVim | `nvim` | ✓ | ✓ | - |
| Emacs | `emacs` | ✓ | ✓ | ✓ |
| Sublime Text | `subl` | ✓ | ✓ | ✓ |
| Zed | `zed` | ✓ | ✓ | ✓ |
| Helix | `hx` | ✓ | ✓ | - |
| Nano | `nano` | ✓ | ✓ | - |
| TextMate | `mate` | ✓ | - | ✓ |
| Notepad++ | `notepad++` | ✓ | ✓ | - |
| Kate | `kate` | ✓ | ✓ | - |
| Atom | `atom` | ✓ | ✓ | ✓ |
| IntelliJ IDEA | `idea` | ✓ | - | ✓ |
| WebStorm | `webstorm` | ✓ | - | ✓ |
| PyCharm | `pycharm` | ✓ | - | ✓ |
| GoLand | `goland` | ✓ | - | ✓ |
| CLion | `clion` | ✓ | - | ✓ |
| Xcode | `xed` | ✓ | - | ✓ |

## Editor Detection

opensesame detects your preferred editor in this order:

1. `$VISUAL` environment variable (preferred for GUI editors)
2. `$EDITOR` environment variable (traditional editor variable)
3. Search PATH for known editors (VS Code, Cursor, Zed, nvim, vim, etc.)

The environment variables can include arguments:

```bash
export VISUAL="code --wait"
export EDITOR="nvim"
```

## Error Handling

opensesame provides rich error types:

```rust
use opensesame::{Editor, Error};

match Editor::open("src/main.rs") {
    Ok(()) => println!("Opened successfully"),
    Err(Error::NoEditorFound) => println!("No editor configured"),
    Err(Error::EditorNotFound { binary }) => println!("{binary} not found"),
    Err(Error::FileNotFound { path }) => println!("{} not found", path.display()),
    Err(e) => println!("Error: {e}"),
}
```

## License

MIT
