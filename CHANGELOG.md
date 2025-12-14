# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2024-12-14

### Added

- **Configurable editor resolution** - Applications can now control how editors are detected
  - New `EditorConfig` struct for passing editor preferences from application config files
  - New `ResolveFrom` enum to specify resolution sources (Config, Visual, Editor, PathSearch)
  - New `EditorBuilder::with_config()` method to add configs to the resolution chain
  - New `EditorBuilder::resolve_order()` method to customize detection priority
  - Predefined resolution orders: `DEFAULT_RESOLVE_ORDER` and `ENV_ONLY_RESOLVE_ORDER`

- **Serde support** (optional) - Enable the `serde` feature for config file deserialization
  - `EditorConfig` supports serde Deserialize/Serialize when feature is enabled
  - `EditorKindConfig` wrapper for deserializing editor kinds from strings
  - Works with YAML, TOML, JSON, or any serde-supported format

- **EditorKind parsing**
  - New `EditorKind::from_name()` method to parse editor kinds from string names
  - New `EditorKind::as_str()` method to get canonical string representation
  - Case-insensitive matching with support for common aliases

- **New error variant**
  - `Error::InvalidConfig` for configuration-related errors
  - `Error::is_invalid_config()` predicate method

### Changed

- `EditorSource` enum now includes a `Config { index }` variant for config-sourced editors
- Detection logic refactored to support configurable resolution order

## [0.1.0] - 2024-12-14

### Added

- Initial release
- Cross-platform support for macOS, Linux, and Windows
- Smart editor detection via `$VISUAL`, `$EDITOR`, and PATH search
- Line and column positioning support
- Support for 30+ editors including VS Code, Vim, NeoVim, Emacs, Sublime Text, Zed, Helix, and JetBrains IDEs
- Builder pattern API for flexible configuration
- Type-safe error handling with rich error types
