# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.1] - 2026-09-21

### Changed
- mls release archives are extracted by Zed rather than by the extension, and
  are no longer checked against the release's `SHA256SUMS`, which is published
  alongside the asset and so only guarded against a corrupted download. This
  drops the `flate2`, `tar`, `zip`, and `sha2` dependencies (#57).

## [0.9.0] - 2026-09-18

### Added
- The extension installs the newest mach-lsp release whose compiler the
  project accepts, rather than always the latest (#53). It reads the
  `[project].mach` ranges of the root `mach.toml` and of the dependency
  closure, and chooses through the `RELEASES.json` map every mach-lsp release
  publishes. When no release satisfies the ranges, it installs the newest and
  mls reports why on `mach.toml`. A latest release without a valid map is
  reported as an error.
- The README documents mls's options under `lsp.mls.initialization_options`
  and how the compiler mls links decides which projects it loads, with the
  diagnostics it reports on `mach.toml` (#51).

### Changed
- Downloaded mls versions are kept until no project has used them for 30
  days, so projects that need different versions do not replace each other's
  copy. Offline, the extension chooses among the downloaded versions with the
  last map it fetched (#53).
- The README no longer suggests passing arguments to `mls`, which takes none
  (#51).
- The extension is attributed to Briar Systems LLC in `LICENSE` and in the
  `extension.toml` authors (#49).

## [0.8.0] - 2026-09-17

### Added
- The extension downloads a prebuilt `mls` from the latest mach-lsp release
  when neither the settings nor `$PATH` provide one (#39). The archive for the
  current platform is checked against the release's `SHA256SUMS` before it is
  unpacked, each version is kept in its own `mls-<version>/` directory, and
  older copies are removed after an upgrade. Without network access the newest
  downloaded copy is used. Platforms mach-lsp does not ship fail with a message
  that says how to provide `mls` instead.

### Changed
- `mls` on `$PATH` now takes precedence over a previously found binary, so
  replacing the one on `$PATH` takes effect on the next server start (#39).
- `lsp.mls.binary.arguments` applies without a `path`, to whichever `mls` is
  found (#39).
- CI runs clippy on the tests and runs `cargo test` (#39).

## [0.7.0] - 2026-09-16

### Added
- Mach 5.2 syntax highlights correctly (#44). A bodyless `def Name;` handle or
  abi type highlights its name as a type and appears in the outline, and the
  operands of `$is_integer` and `$is_float` highlight as types.
- A changelog, backfilled from the history of every release tag (#40).

### Changed
- The grammar is pinned to mach-tree-sitter v0.6.0 (#44).
- CI follows the family contract in briar-systems/.github (briar-systems/mach#3447).
  One `ci.yml` checks formatting, runs clippy, builds the extension for
  `wasm32-wasip2` (the target Zed builds extensions for) and compiles every
  query file against the pinned grammar. It ends in a `gate` job and runs on
  pull requests and dispatch only (#35).
- The README builds the extension for `wasm32-wasip2` instead of
  `wasm32-wasip1` (#35).

## [0.6.0] - 2026-09-13

### Added
- Mach 5.0 syntax support (#32). `tag` and `sel` highlight as keywords, tag
  cases highlight as constants in declarations and generic literals, and tag
  declarations appear in the outline with their cases.

### Changed
- `:>` highlights as the declassification operator, replacing the removed
  `:^` form (#32).
- The grammar is pinned to mach-tree-sitter v0.5.0 (#32).

## [0.5.0] - 2026-08-31

### Added
- Secret syntax highlighting (#29). The `^` marker in a `^T` secret type
  highlights as a type qualifier, and the operator in a `value:^T` strip
  highlights as an operator.

### Changed
- The grammar is pinned to mach-tree-sitter v0.4.0 (#29).
- Repository references point at the briar-systems organization after the
  transfer (#25, #27).

### Fixed
- Both `#[` and `]` of an annotation highlight as `punctuation.special`. The
  closing delimiter was overwritten by the broad bracket captures (#29).

## [0.4.2] - 2026-06-24

### Fixed
- Method and qualified call targets highlight as functions instead of
  properties. The call rules now follow the field rules, so they win under
  last-match-wins (#22).

## [0.4.1] - 2026-06-24

### Added
- Projection expressions (`v.[f]`) highlight their punctuation.
- `#[` ... `]` is a bracket-matching pair.

### Changed
- The decorator `#[` sigil highlights as `punctuation.special`, and the
  decorator name stays an attribute.

### Fixed
- Auto-indent no longer dedents on the closers of constructs that do not
  indent, such as `if` / `for` / `$if` conditions, generics, type arguments and
  indexing. Indent ends are scoped to their containers and the blanket outdent
  rule is gone.

## [0.4.0] - 2026-06-20

### Changed
- Decorators use the `#[attr]` form. The grammar is pinned to
  mach-tree-sitter v0.3.0 and the decorator query follows it.

## [0.3.0] - 2026-06-19

### Added
- Mach 2.0.0 query support (#16, #19). Decorators highlight as attributes
  while their arguments keep their own highlights. Comptime variadic packs
  highlight: `va: ...` pack parameters, the `$each a in va { ... }` unroll and
  the `va...` spread.

### Changed
- The grammar is pinned to mach-tree-sitter v0.2.0.
- `zed_extension_api` is 0.7.0.
- The README documents the `mach-lsp` to `mls` rename and builds mach-lsp with
  `mach dep pull` and `mach build`.

### Removed
- The C-style `varargs_expression` highlight, which Mach 2.0.0 dropped.

### Fixed
- The README license link points at `./LICENSE` (#11).

## [0.2.0] - 2026-06-08

### Added
- Initial Zed extension for Mach: syntax highlighting, auto-indentation,
  bracket matching, comment toggling and the document outline, backed by
  mach-tree-sitter.
- A Rust WASM component that starts `mls`. It resolves the binary from the
  `lsp.mls.binary.path` setting, then a cached path, then `$PATH`.
- An MIT license.
- Queries for the `:~` and `::` casts, `fwd` re-exports, `$` comptime
  parameters and fields, and the `asm <isa> { ... }` form.

### Fixed
- The bare `*` and `&` sigils highlight, so pointer types render.

[Unreleased]: https://github.com/briar-systems/mach-zed/compare/v0.9.1...dev
[0.9.1]: https://github.com/briar-systems/mach-zed/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/briar-systems/mach-zed/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/briar-systems/mach-zed/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/briar-systems/mach-zed/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/briar-systems/mach-zed/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/briar-systems/mach-zed/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/briar-systems/mach-zed/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/briar-systems/mach-zed/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/briar-systems/mach-zed/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/briar-systems/mach-zed/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/briar-systems/mach-zed/releases/tag/v0.2.0
