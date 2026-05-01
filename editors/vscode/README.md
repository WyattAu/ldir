# ldir — VS Code Extension

Language Server and tooling for [ldir](../../) — the LLVM of documents.

## Features

- Syntax highlighting for LaTeX (`.tex`) and Typst (`.typ`) files
- LSP integration for diagnostics, hover, and completions (requires `ldir-lsp`)
- Commands for compiling documents and viewing output

## Requirements

- `ldir-lsp` — the ldir language server (optional; extension works without it)
- `ldc` — the ldir compiler (for compile/preview commands)

## Installation

### From source

```bash
cd editors/vscode
npm install
npm run compile
```

Then open this folder in VS Code and press `F5` to launch the Extension Development Host.

### From VSIX

```bash
npx vsce package
code --install-extension ldir-0.1.0.vsix
```

## Commands

| Command | Description |
|---------|-------------|
| `ldir: Compile Document` | Compile the current file to PDF |
| `ldir: Show PDF Preview` | Open the compiled PDF |
| `ldir: Show S-IR` | Show the intermediate representation |

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `ldir.serverPath` | `ldir-lsp` | Path to the ldir-lsp binary |
| `ldir.compilerPath` | `ldc` | Path to the ldc compiler binary |

## License

See the [root repository](../../) for license information.
