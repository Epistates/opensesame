//! Command building module.
//!
//! This module constructs editor-specific command-line arguments for opening
//! files at specific line and column positions.

use std::path::Path;
use std::process::Command;

use crate::detect::DetectedEditor;
use crate::editor::EditorKind;

/// Builds the command to open a file in an editor.
pub fn build_command(
    editor: &DetectedEditor,
    file: &Path,
    line: Option<u32>,
    column: Option<u32>,
    wait: bool,
) -> Command {
    let mut cmd = Command::new(&editor.binary);

    // Add any extra args from environment (e.g., "--wait" from "$EDITOR=code --wait")
    for arg in &editor.extra_args {
        cmd.arg(arg);
    }

    // Build editor-specific arguments
    let args = build_args(editor.kind, file, line, column, wait);
    for arg in args {
        cmd.arg(arg);
    }

    // Terminal editors need to inherit stdio
    if editor.is_terminal_editor() {
        cmd.stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());
    }

    cmd
}

/// Builds the argument list for an editor.
fn build_args(
    kind: EditorKind,
    file: &Path,
    line: Option<u32>,
    column: Option<u32>,
    wait: bool,
) -> Vec<String> {
    let file_str = file.display().to_string();

    match kind {
        // VS Code family: code -g file:line:column [--wait]
        EditorKind::VsCode
        | EditorKind::VsCodeInsiders
        | EditorKind::VSCodium
        | EditorKind::Cursor
        | EditorKind::Windsurf => {
            build_vscode_args(&file_str, line, column, wait)
        }

        // Vim family: vim +call\ cursor(line,col) file
        EditorKind::Vim | EditorKind::NeoVim | EditorKind::Vi | EditorKind::GVim => {
            build_vim_args(&file_str, line, column)
        }

        // Emacs: emacs +line:col file [--wait]
        EditorKind::Emacs | EditorKind::EmacsClient => {
            build_emacs_args(&file_str, line, column, wait)
        }

        // Sublime Text: subl file:line:column [--wait]
        EditorKind::Sublime => {
            build_sublime_args(&file_str, line, column, wait)
        }

        // Zed: zed file:line:column [--wait]
        EditorKind::Zed => {
            build_zed_args(&file_str, line, column, wait)
        }

        // Helix: hx file:line:column
        EditorKind::Helix => {
            build_helix_args(&file_str, line, column)
        }

        // Nano: nano +line,col file
        EditorKind::Nano => {
            build_nano_args(&file_str, line, column)
        }

        // TextMate: mate --line line file [--wait]
        EditorKind::TextMate => {
            build_textmate_args(&file_str, line, wait)
        }

        // Notepad++: notepad++ -nLINE -cCOL file
        EditorKind::NotepadPlusPlus => {
            build_notepadpp_args(&file_str, line, column)
        }

        // JetBrains IDEs: idea file:line [--wait]
        EditorKind::IntelliJ
        | EditorKind::WebStorm
        | EditorKind::PhpStorm
        | EditorKind::PyCharm
        | EditorKind::RubyMine
        | EditorKind::GoLand
        | EditorKind::CLion
        | EditorKind::Rider
        | EditorKind::DataGrip
        | EditorKind::AndroidStudio => {
            build_jetbrains_args(&file_str, line, wait)
        }

        // Xcode: xed --line LINE file
        EditorKind::Xcode => {
            build_xcode_args(&file_str, line, wait)
        }

        // Kate: kate --line LINE --column COL file
        EditorKind::Kate => {
            build_kate_args(&file_str, line, column)
        }

        // Atom (deprecated but still used): atom file:line:column [--wait]
        EditorKind::Atom => {
            build_atom_args(&file_str, line, column, wait)
        }

        // Notepad (Windows): no line/column support
        EditorKind::Notepad => {
            vec![file_str]
        }

        // Unknown editor: just pass the file
        EditorKind::Unknown => {
            vec![file_str]
        }
    }
}

/// VS Code family: `code -g file:line:column [--wait]`
fn build_vscode_args(file: &str, line: Option<u32>, column: Option<u32>, wait: bool) -> Vec<String> {
    let mut args = Vec::new();

    // Use --goto flag for line:column positioning
    args.push("--goto".to_string());

    let position = match (line, column) {
        (Some(l), Some(c)) => format!("{file}:{l}:{c}"),
        (Some(l), None) => format!("{file}:{l}"),
        _ => file.to_string(),
    };
    args.push(position);

    if wait {
        args.push("--wait".to_string());
    }

    args
}

/// Vim family: `vim +call\ cursor(line,col) file` or `vim +LINE file`
fn build_vim_args(file: &str, line: Option<u32>, column: Option<u32>) -> Vec<String> {
    match (line, column) {
        (Some(l), Some(c)) => {
            vec![format!("+call cursor({l},{c})"), file.to_string()]
        }
        (Some(l), None) => {
            vec![format!("+{l}"), file.to_string()]
        }
        _ => vec![file.to_string()],
    }
}

/// Emacs: `emacs +line:col file`
fn build_emacs_args(file: &str, line: Option<u32>, column: Option<u32>, wait: bool) -> Vec<String> {
    let mut args = Vec::new();

    match (line, column) {
        (Some(l), Some(c)) => args.push(format!("+{l}:{c}")),
        (Some(l), None) => args.push(format!("+{l}")),
        _ => {}
    }

    args.push(file.to_string());

    if wait {
        args.push("--eval".to_string());
        args.push("(while (get-buffer-window) (sit-for 1))".to_string());
    }

    args
}

/// Sublime Text: `subl file:line:column [--wait]`
fn build_sublime_args(file: &str, line: Option<u32>, column: Option<u32>, wait: bool) -> Vec<String> {
    let mut args = Vec::new();

    let position = match (line, column) {
        (Some(l), Some(c)) => format!("{file}:{l}:{c}"),
        (Some(l), None) => format!("{file}:{l}"),
        _ => file.to_string(),
    };
    args.push(position);

    if wait {
        args.push("--wait".to_string());
    }

    args
}

/// Zed: `zed file:line:column [--wait]`
fn build_zed_args(file: &str, line: Option<u32>, column: Option<u32>, wait: bool) -> Vec<String> {
    let mut args = Vec::new();

    let position = match (line, column) {
        (Some(l), Some(c)) => format!("{file}:{l}:{c}"),
        (Some(l), None) => format!("{file}:{l}"),
        _ => file.to_string(),
    };
    args.push(position);

    if wait {
        args.push("--wait".to_string());
    }

    args
}

/// Helix: `hx file:line:column`
fn build_helix_args(file: &str, line: Option<u32>, column: Option<u32>) -> Vec<String> {
    let position = match (line, column) {
        (Some(l), Some(c)) => format!("{file}:{l}:{c}"),
        (Some(l), None) => format!("{file}:{l}"),
        _ => file.to_string(),
    };
    vec![position]
}

/// Nano: `nano +line,col file`
fn build_nano_args(file: &str, line: Option<u32>, column: Option<u32>) -> Vec<String> {
    match (line, column) {
        (Some(l), Some(c)) => {
            vec![format!("+{l},{c}"), file.to_string()]
        }
        (Some(l), None) => {
            vec![format!("+{l}"), file.to_string()]
        }
        _ => vec![file.to_string()],
    }
}

/// TextMate: `mate --line line file [--wait]`
fn build_textmate_args(file: &str, line: Option<u32>, wait: bool) -> Vec<String> {
    let mut args = Vec::new();

    if let Some(l) = line {
        args.push("--line".to_string());
        args.push(l.to_string());
    }

    args.push(file.to_string());

    if wait {
        args.push("--wait".to_string());
    }

    args
}

/// Notepad++: `notepad++ -nLINE -cCOL file`
fn build_notepadpp_args(file: &str, line: Option<u32>, column: Option<u32>) -> Vec<String> {
    let mut args = Vec::new();

    if let Some(l) = line {
        args.push(format!("-n{l}"));
    }

    if let Some(c) = column {
        args.push(format!("-c{c}"));
    }

    args.push(file.to_string());
    args
}

/// JetBrains IDEs: `idea file:line [--wait]`
fn build_jetbrains_args(file: &str, line: Option<u32>, wait: bool) -> Vec<String> {
    let mut args = Vec::new();

    // JetBrains doesn't support column positioning
    let position = match line {
        Some(l) => format!("{file}:{l}"),
        None => file.to_string(),
    };
    args.push(position);

    if wait {
        args.push("--wait".to_string());
    }

    args
}

/// Xcode: `xed --line LINE file`
fn build_xcode_args(file: &str, line: Option<u32>, wait: bool) -> Vec<String> {
    let mut args = Vec::new();

    if let Some(l) = line {
        args.push("--line".to_string());
        args.push(l.to_string());
    }

    args.push(file.to_string());

    if wait {
        args.push("--wait".to_string());
    }

    args
}

/// Kate: `kate --line LINE --column COL file`
fn build_kate_args(file: &str, line: Option<u32>, column: Option<u32>) -> Vec<String> {
    let mut args = Vec::new();

    if let Some(l) = line {
        args.push("--line".to_string());
        args.push(l.to_string());
    }

    if let Some(c) = column {
        args.push("--column".to_string());
        args.push(c.to_string());
    }

    args.push(file.to_string());
    args
}

/// Atom: `atom file:line:column [--wait]`
fn build_atom_args(file: &str, line: Option<u32>, column: Option<u32>, wait: bool) -> Vec<String> {
    let mut args = Vec::new();

    let position = match (line, column) {
        (Some(l), Some(c)) => format!("{file}:{l}:{c}"),
        (Some(l), None) => format!("{file}:{l}"),
        _ => file.to_string(),
    };
    args.push(position);

    if wait {
        args.push("--wait".to_string());
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vscode_args() {
        let args = build_vscode_args("test.rs", Some(42), Some(10), false);
        assert_eq!(args, vec!["--goto", "test.rs:42:10"]);

        let args = build_vscode_args("test.rs", Some(42), None, false);
        assert_eq!(args, vec!["--goto", "test.rs:42"]);

        let args = build_vscode_args("test.rs", None, None, true);
        assert_eq!(args, vec!["--goto", "test.rs", "--wait"]);
    }

    #[test]
    fn test_vim_args() {
        let args = build_vim_args("test.rs", Some(42), Some(10));
        assert_eq!(args, vec!["+call cursor(42,10)", "test.rs"]);

        let args = build_vim_args("test.rs", Some(42), None);
        assert_eq!(args, vec!["+42", "test.rs"]);

        let args = build_vim_args("test.rs", None, None);
        assert_eq!(args, vec!["test.rs"]);
    }

    #[test]
    fn test_nano_args() {
        let args = build_nano_args("test.rs", Some(42), Some(10));
        assert_eq!(args, vec!["+42,10", "test.rs"]);

        let args = build_nano_args("test.rs", Some(42), None);
        assert_eq!(args, vec!["+42", "test.rs"]);
    }

    #[test]
    fn test_emacs_args() {
        let args = build_emacs_args("test.rs", Some(42), Some(10), false);
        assert_eq!(args, vec!["+42:10", "test.rs"]);
    }

    #[test]
    fn test_notepadpp_args() {
        let args = build_notepadpp_args("test.rs", Some(42), Some(10));
        assert_eq!(args, vec!["-n42", "-c10", "test.rs"]);
    }

    #[test]
    fn test_jetbrains_args() {
        let args = build_jetbrains_args("test.rs", Some(42), false);
        assert_eq!(args, vec!["test.rs:42"]);

        // JetBrains doesn't support column
        let args = build_jetbrains_args("test.rs", Some(42), true);
        assert_eq!(args, vec!["test.rs:42", "--wait"]);
    }

    #[test]
    fn test_helix_args() {
        let args = build_helix_args("test.rs", Some(42), Some(10));
        assert_eq!(args, vec!["test.rs:42:10"]);
    }

    #[test]
    fn test_kate_args() {
        let args = build_kate_args("test.rs", Some(42), Some(10));
        assert_eq!(args, vec!["--line", "42", "--column", "10", "test.rs"]);
    }
}
