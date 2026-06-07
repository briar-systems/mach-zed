# mach-zed

[Zed](https://zed.dev) extension for the [Mach](https://github.com/octalide/mach) programming language.

## Features

- **Syntax highlighting** via [mach-tree-sitter](https://github.com/octalide/mach-tree-sitter)
- **Auto-indentation** for blocks, records, unions, parameter lists, and initializer lists
- **Bracket matching** and auto-closing for `{}`, `[]`, `()`, `""`, `''`
- **Comment toggling** with `#`
- **LSP integration** with [mach-lsp](https://github.com/octalide/mach-lsp) (when available on `$PATH`)

## Installation

### From the Zed Extension Registry

Once published, search for **Mach** in Zed's extension panel (`zed: extensions`) and click **Install**.

### Manual / Development

Clone this repository and symlink or copy it into your Zed extensions directory:

```bash
git clone https://github.com/octalide/mach-zed.git
ln -s "$(pwd)/mach-zed" ~/.local/share/zed/extensions/installed/mach
```

Restart Zed to pick up the extension.

## Language Server

This extension ships a Rust WASM extension that tells Zed how to locate and start `mach-lsp`. When you open a `.mach` file, Zed will:

1. Check your Zed settings for a user-configured binary path (see [Configuration](#configuration) below).
2. Look for `mach-lsp` on your system `$PATH`.
3. If neither is found, display an error with instructions.

### Building mach-lsp

Build `mach-lsp` from source:

```bash
git clone https://github.com/octalide/mach-lsp.git
cd mach-lsp
cmach build .
```

The binary will be at `out/linux/bin/mach-lsp`. Add it to your `$PATH`:

```bash
cp out/linux/bin/mach-lsp ~/.local/bin/
```

### Building the WASM Extension

If you're developing the extension locally and need to rebuild the WASM component:

```bash
cd mach-zed
cargo build --release --target wasm32-wasip1
```

> **Note:** You need the `wasm32-wasip1` target installed:
> ```bash
> rustup target add wasm32-wasip1
> ```

The compiled `.wasm` file will be at `target/wasm32-wasip1/release/mach_zed.wasm`. Zed handles building and loading the WASM automatically when installing from the extension registry or from a local dev extension directory.

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

If `mach-lsp` is not on your `$PATH`, or you want to use a specific build, configure the binary path in your Zed `settings.json`:

```json
{
    "lsp": {
        "mach-lsp": {
            "binary": {
                "path": "/absolute/path/to/mach-lsp",
                "arguments": []
            }
        }
    }
}
```

## Project Structure

```
mach-zed/
├── extension.toml              # Extension manifest (grammars, LSP, metadata)
├── Cargo.toml                  # Rust WASM extension build configuration
├── src/
│   └── lib.rs                  # WASM extension entry point (language_server_command)
├── languages/
│   └── mach/
│       ├── config.toml         # Language configuration (brackets, comments, etc.)
│       ├── brackets.scm        # Bracket matching queries
│       ├── highlights.scm      # Syntax highlighting queries
│       ├── indents.scm         # Auto-indentation queries
│       └── outline.scm         # Document outline queries
└── README.md
```

## How It Works

Zed extensions with language server support require a Rust WASM component that implements the `Extension` trait from `zed_extension_api`. The key method is `language_server_command`, which returns the command Zed should execute to start the LSP.

The extension resolves the `mach-lsp` binary in this order:

1. **User settings** — `lsp.mach-lsp.binary.path` in Zed's `settings.json`
2. **Cached path** — a previously resolved path that still exists on disk
3. **System PATH** — `worktree.which("mach-lsp")` searches `$PATH`

If none of these succeed, Zed shows an error message guiding the user to install `mach-lsp`.

## Contributing

1. Clone this repo alongside [mach-tree-sitter](https://github.com/octalide/mach-tree-sitter).
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
>
> Note `pub` / `ext` are repeatable `modifiers` children of each declaration
> (there is no `public_declaration` or `extern_declaration` node), and there is
> no `&T` readonly-pointer or method-receiver syntax.

## License

[MIT](../mach/LICENSE)