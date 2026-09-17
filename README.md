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

The download needs no setup. The extension fetches the archive for your platform, checks it against the release's `SHA256SUMS`, and keeps it in the extension's work directory as `mls-<version>/`. A newer mach-lsp release is picked up the next time Zed loads the extension, and older copies are removed. When GitHub cannot be reached, the newest copy already downloaded is used. Prebuilt binaries exist for x86_64 and aarch64 Linux, x86_64 and aarch64 macOS, and x86_64 Windows. On any other platform, put `mls` on your `$PATH` or set its path in settings.

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

To use a specific build of `mls`, set its path in your Zed `settings.json`. `arguments` are passed to `mls` whichever way it was found:

```json
{
    "lsp": {
        "mls": {
            "binary": {
                "path": "/absolute/path/to/mls",
                "arguments": []
            }
        }
    }
}
```

## Project Structure

```
mach-zed/
├── .github/workflows/ci.yml    # CI (fmt, clippy, wasm build, query checks)
├── extension.toml              # Extension manifest (grammars, LSP, metadata)
├── Cargo.toml                  # Rust WASM extension build configuration
├── src/
│   ├── lib.rs                  # WASM extension entry point (language_server_command)
│   └── install.rs              # mls release asset selection, verification, extraction
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
3. **Release download**: the asset for the current platform from the latest [mach-lsp release](https://github.com/briar-systems/mach-lsp/releases), named by mach-lsp's release asset contract. It is verified against `SHA256SUMS`, extracted in the extension, and installed atomically into `mls-<version>/` (`src/install.rs`)

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
