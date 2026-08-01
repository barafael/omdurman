# traceability-lsp

An LSP server that maps rulebook requirements (`§N`) to their implementation
sites across this repo. It is a thin client over the same `checks` that
`cargo test -p omdurman-rules --test traceability` runs, so the editor and the
test suite always agree.

It provides, over **`.rs`** files, **`docs/traceability.toml`**, and the OCR
manual markdown:

- **Diagnostics** — the bijectivity failures the rules test enforces (impl
  anchor drift, missing symbols, unanchored symbols, orphan citations, test
  list mismatches) as errors on the exact TOML line / Rust line; implemented
  mappings without an annotated test appear as *warnings*.
- **Hover** — requirement card (`§N — title`, status, impl sites, tests),
  impl-site summary, and annotated-test coverage.
- **Definition / References / Implementation** — jump between a `§N` in the
  manual, its `[[mapping]]` block, its source impl sites, and its `#[rulebook]`
  tests.
- **CodeLens** — `§6.53 — Artillery Fire ... [implemented · 2 tests]` over each
  impl symbol and `covers §6.22` over each annotated test.

## Build

```shell
cargo build -p traceability-lsp
```

The binary speaks LSP over stdio: `./target/debug/traceability-lsp`.

## VS Code

The extension in `vscode-extension/` is a dependency-free client (no
`npm install`). To run it:

1. Build the server first: `cargo build -p traceability-lsp`.
2. Open the extension folder in VS Code: `code tools/traceability-lsp/vscode-extension`.
3. Press **F5** (there is a `launch.json`, so this launches an Extension
   Development Host window — not the file debugger).
4. In the new window, open the repo root: **File > Open Folder… → the
   omdurman repo root** (or open individual `.rs` / `.toml` / manual `.md`
   files). The server locates `docs/traceability.toml` itself, so it works
   even if that folder is opened elsewhere.

The server binary is auto-detected from `target/debug` / `target/release`
next to the repo; override with the `traceabilityLsp.serverPath` setting.

To use the extension without F5, install it locally and reload:

```shell
cd tools/traceability-lsp/vscode-extension
code --install-extension . --force
```

(Dependencies are bundled; there is no `npm install` step.)

## Neovim

Point `vim.lsp.start` at the built binary (adjust the path):

```lua
vim.lsp.start({ name = 'traceability',
  cmd = { vim.fn.getcwd() .. '/target/debug/traceability-lsp' },
  root_dir = vim.fn.getcwd(),
  capabilities = vim.lsp.protocol.make_client_capabilities() })
```

## How the checks map to the test suite

`omdurman-rules/tests/traceability.rs` is a thin runner over the same
`traceability_lsp::checks` the server uses, so a fix that clears an editor
diagnostic also fixes the corresponding `cargo test` failure and vice versa.
