# mach-zed

[Zed](https://zed.dev) extension for the [Mach](https://github.com/briar-systems/mach) programming language.

## Features

- **Syntax highlighting** via [mach-tree-sitter](https://github.com/briar-systems/mach-tree-sitter)
- **Auto-indentation** for blocks, records, unions, parameter lists, and initializer lists
- **Bracket matching** and auto-closing for `{}`, `[]`, `()`, `""`, `''`
- **Comment toggling** with `#`
- **LSP integration** with [mach-lsp](https://github.com/briar-systems/mach-lsp) (when available on `$PATH`)

## Installation

### From the Zed Extension Registry

Once published, search for **Mach** in Zed's extension panel (`zed: extensions`) and click **Install**.

### Manual / Development

Clone this repository and symlink or copy it into your Zed extensions directory:

```bash
git clone https://github.com/briar-systems/mach-zed.git
ln -s "$(pwd)/mach-zed" ~/.local/share/zed/extensions/installed/mach
```

Restart Zed to pick up the extension.

## Language Server

The extension starts `mls`, the [mach-lsp](https://github.com/briar-systems/mach-lsp) server. When you open a `.mach` file, Zed resolves it in this order:

1. The binary path in your Zed settings (see [Configuration](#configuration) below).
2. `mls` on your `$PATH`.
3. A prebuilt `mls` downloaded from the latest mach-lsp release.

The download needs no setup. The extension picks the newest mach-lsp release whose compiler your project accepts (see [Compiler Compatibility](#compiler-compatibility)). It fetches that release's archive for your platform and keeps it in the extension's work directory as `mls-<version>/`. A newer mach-lsp release is picked up the next time Zed loads the extension. A copy no project has used for 30 days is removed. When GitHub cannot be reached, the extension chooses among the copies already downloaded. Prebuilt binaries exist for x86_64 and aarch64 Linux, x86_64 and aarch64 macOS, and x86_64 Windows. On any other platform, put `mls` on your `$PATH` or set its path in settings.

### Building mach-lsp

To use your own build instead, build `mach-lsp` from source:

```bash
git clone https://github.com/briar-systems/mach-lsp.git
cd mach-lsp
mach dep pull
mach build .
```

The binary will be at `out/{target}/{profile}/bin/mls` (e.g. `out/linux-x86_64/debug/bin/mls` for a default debug build on Linux). Put it on your `$PATH` or point the settings at it:

```bash
install -Dm755 out/linux-x86_64/debug/bin/mls ~/.local/bin/mls
```

### Building the WASM Extension

If you're developing the extension locally and need to rebuild the WASM component:

```bash
cd mach-zed
cargo build --release --target wasm32-wasip2
```

> **Note:** You need the `wasm32-wasip2` target installed:
> ```bash
> rustup target add wasm32-wasip2
> ```

The compiled `.wasm` file will be at `target/wasm32-wasip2/release/mach_zed.wasm`. Zed handles building and loading the WASM automatically when installing from the extension registry or from a local dev extension directory.

## Configuration

### Editor Settings

You can customize Mach-specific editor settings in your Zed `settings.json`:

```json
{
    "languages": {
        "Mach": {
            "tab_size": 4,
            "hard_tabs": false,
            "format_on_save": "off"
        }
    }
}
```

### Language Server Binary

To use a specific build of `mls`, set its path in your Zed `settings.json`:

```json
{
    "lsp": {
        "mls": {
            "binary": {
                "path": "/absolute/path/to/mls"
            }
        }
    }
}
```

`mls` takes no arguments, so leave `binary.arguments` unset.

### Language Server Options

`mls` reads its options once, when it starts. Set them under `lsp.mls.initialization_options`. Zed passes them to the server unchanged and restarts it when they change:

```json
{
    "lsp": {
        "mls": {
            "initialization_options": {
                "trace": "messages",
                "traceFile": "/home/me/mls.log"
            }
        }
    }
}
```

| key | value | environment fallback |
| --- | --- | --- |
| `trace` | `"off"`, `"messages"` (what each message is, no contents) or `"bodies"` (also message contents, which include your source code) | `MLS_TRACE` |
| `traceFile` | an absolute path the trace is appended to. Without it or `MLS_TRACE_FILE`, the trace goes to `/tmp/mach-lsp.log` | `MLS_TRACE_FILE` |
| `requestDeadlineMs` | how long a request may wait on analysis before the server gives up on it, at least `1000` | `MLS_REQUEST_DEADLINE_MS` |

An option takes precedence over its environment variable, which takes precedence over the server's default. Nothing is traced unless `trace` or `MLS_TRACE` turns it on. An unknown key or an unusable value is ignored, noted in the trace, and never stops the server from starting.

Leave `requestDeadlineMs` unset unless you have a reason to change it. In mach-lsp 0.20.0, a deadline shorter than the time your project takes to load stops the language server altogether.

The server does not read `lsp.mls.settings`, so options placed there have no effect.

### Compiler Compatibility

`mls` contains the Mach compiler, linked from one mach release. `mls --version` names it, for example `mls 0.20.0 (mach 5.4.0)`.

A project states the compilers it builds with as `[project].mach` in its `mach.toml`:

```toml
[project]
mach = "^5.4"
```

When the linked compiler is outside that range, or outside a range one of the project's dependencies states, `mls` does not load the project. It reports why as an error on `mach.toml`, naming the dependency chain, and shows it as a notification. A `mach.toml` without the key loads, with a warning that gives the line to add. Both are reported on `mach.toml` itself, so they appear in the project diagnostics panel and when you open that file, and they clear once you save the fix.

The extension reads these ranges before it installs `mls`. It collects `[project].mach` from the `mach.toml` at the root of your Zed project and from every dependency it can reach (`dep/<id>/mach.toml` for a git dependency, the declared directory for a path dependency). Then it installs the newest mach-lsp release whose compiler satisfies all of them. Each release publishes `RELEASES.json`, which maps every mach-lsp version to the compiler it links, so no other release needs downloading to decide. When no release satisfies the ranges, or a range does not parse, the extension installs the newest release, and `mls` reports the problem on `mach.toml`. The choice is made once per project when its language server starts, so restart the server (`editor: restart language server`) after changing a range. An `mls` from your settings or `$PATH` is used as it is, whatever it links.

## Project Structure

```
mach-zed/
├── .github/workflows/ci.yml    # CI (fmt, clippy, wasm build, query checks)
├── extension.toml              # Extension manifest (grammars, LSP, metadata)
├── Cargo.toml                  # Rust WASM extension build configuration
├── src/
│   ├── lib.rs                  # WASM extension entry point (language_server_command)
│   ├── compat.rs               # dependency closure ranges and mls release selection
│   ├── install.rs              # mls release asset naming and install layout
│   └── semver.rs               # mach version and range grammar
├── languages/
│   └── mach/
│       ├── config.toml         # Language configuration (brackets, comments, etc.)
│       ├── brackets.scm        # Bracket matching queries
│       ├── highlights.scm      # Syntax highlighting queries
│       ├── indents.scm         # Auto-indentation queries
│       └── outline.scm         # Document outline queries
├── CHANGELOG.md
└── README.md
```

## How It Works

Zed extensions with language server support require a Rust WASM component that implements the `Extension` trait from `zed_extension_api`. The key method is `language_server_command`, which returns the command Zed should execute to start the LSP.

The extension resolves the `mls` binary in this order:

1. **User settings**: `lsp.mls.binary.path` in Zed's `settings.json`
2. **System PATH**: `worktree.which("mls")` searches `$PATH`
3. **Release download**: the newest [mach-lsp release](https://github.com/briar-systems/mach-lsp/releases) whose compiler satisfies the project's dependency closure, found through the latest release's `RELEASES.json` (`src/compat.rs`, with ranges parsed by `src/semver.rs` to the grammar in mach's `doc/language/manifest.md`). Its asset for the current platform, named by mach-lsp's release asset contract, is extracted by Zed and installed atomically into `mls-<version>/` (`src/install.rs`). A latest release without a valid `RELEASES.json` that lists itself is reported as an error, not worked around

If none of these succeed, Zed shows the reason in the language server status.

## Contributing

1. Clone this repo alongside [mach-tree-sitter](https://github.com/briar-systems/mach-tree-sitter).
2. Edit queries in `languages/mach/` and reload the Zed extension to test.
3. For grammar changes, update `mach-tree-sitter` and bump the `rev` in `[grammars.mach]` of `extension.toml`.
4. For LSP integration changes, edit `src/lib.rs` and rebuild the WASM component.

> **Highlight queries are coupled to the grammar.** A query may only reference
> node types and tokens that the pinned `mach-tree-sitter` revision actually
> produces — referencing a node or token the grammar does not emit makes Zed
> drop the entire query file. So query updates for new syntax must land
> *together with* a grammar bump, never ahead of one. Validate every query file
> against the pinned grammar before committing (`tree-sitter query <file>.scm
> sample.mach` must exit cleanly for each).
>
> The grammar now matches the authoritative `doc/language/grammar.md`, and the
> queries cover the full surface, including:
>
> - `:~` bit-reinterpret and `::` value casts — `(cast_expression operator: _
>   @operator)`.
> - `fwd` re-export declarations — `"fwd" @keyword`, plus a
>   `forward_declaration` outline item and alias highlight.
> - `$`-prefixed comptime value parameters / fields (`fun f($x: T)`, `$tag: T;`)
>   — `(parameter comptime: "$" ...)` and `(field_declaration comptime: "$"
>   ...)`.
> - The `asm <isa> { ... }` ISA-tag form — `(asm_statement isa: (identifier))`
>   and `(asm_statement body: (asm_body))`; the raw body is one `asm_body` node.
> - `#[attr]` decorators (`#[symbol("...")]`, `#[inline]`, `#[align(expr)]`,
>   `#[section(...)]`, `#[library(...)]`) — `(decorator "#[" name: (identifier)
>   @attribute)`; argument expressions keep their own highlights.
> - Comptime variadic packs — `(pack_parameter name: (identifier))` for `va: ...`,
>   the `$each a in va { ... }` unroll (`comptime_each_statement`, `$`/`each`/`in`
>   keywords), and the `va...` `pack_spread_expression` ellipsis.
> - Secret values: `^T` is a `secret_type` with its marker highlighted as a type
>   qualifier, while `value:^T` is a `secret_strip_expression` with its operator
>   highlighted as an operator.
> - `#[attr]` annotations: both `#[` and `]` are `punctuation.special`. The
>   decorator query follows the broad bracket captures so they do not overwrite
>   the closing delimiter's annotation highlight.
>
> Note `pub` / `ext` are repeatable `modifiers` children of each declaration
> (there is no `public_declaration` or `extern_declaration` node), and there is
> no method-receiver syntax. `&` is the bitwise-AND operator, so it highlights as
> an operator rather than a pointer sigil.

## License

[MIT](./LICENSE)
