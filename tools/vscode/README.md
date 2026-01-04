# Bend-PVM VS Code Extension

A comprehensive VS Code extension for the Bend-PVM smart contract language.

## Features

### Language Support
- ✅ Syntax highlighting for `.bend` files
- ✅ Code completion
- ✅ Hover information
- ✅ Go to definition
- ✅ Find references
- ✅ Document symbols
- ✅ Workspace symbols
- ✅ Signature help
- ✅ Semantic tokens (syntax highlighting)
- ✅ Inlay hints
- ✅ Code actions (quick fixes)

### IDE Integration
- ✅ VS Code extension (this package)
- ✅ LSP server integration
- ✅ Multi-file project support
- ✅ Incremental updates
- ✅ Fast response times

## Installation

### From VSIX (Recommended)
1. Download the latest `.vsix` file from [releases](https://github.com/developerfred/Bend-PVM/releases)
2. Open VS Code
3. Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on Mac)
4. Type "Extensions: Install from VSIX"
5. Select the downloaded `.vsix` file

### From Source
```bash
cd tools/vscode
npm install
npm run compile
code --install-extension out/bend-pvm-*.vsix
```

### From Marketplace (Coming Soon)
The extension will be published to the VS Code marketplace once stable.

## Configuration

### bend-pvm.server.path
Path to the Bend-PVM LSP server executable.

```json
{
  "bend-pvm.server.path": "/path/to/bend-pvm-lsp"
}
```

### bend-pvm.trace.server
Traces the communication between VS Code and the language server.

```json
{
  "bend-pvm.trace.server": "off" | "messages" | "verbose"
}
```

### bend-pvm.autoUpdate
Automatically check for updates to the Bend-PVM language server.

```json
{
  "bend-pvm.autoUpdate": true
}
```

## Commands

- `bend-pvm.restart` - Restart the language server
- `bend-pvm.showOutput` - Show the Bend-PVM output channel

## Project Structure

```
tools/vscode/
├── package.json              # Extension manifest
├── language-configuration.json # Language settings
├── syntaxes/
│   └── bend.tmLanguage.json  # Syntax highlighting
├── src/
│   └── extension.ts          # Extension implementation
├── README.md                 # This file
└── CHANGELOG.md             # Version history
```

## Development

### Prerequisites
- Node.js 18+
- VS Code 1.74+
- Rust toolchain

### Setup
```bash
# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Run tests
npm test

# Package extension
vsce package
```

### Testing
1. Press `F5` in VS Code to launch the extension in debug mode
2. Open a `.bend` file to test features
3. Check the output channel for logs

## Supported Editors

This LSP server also supports:
- **Vim/Neovim** - Using `vim-lsp` or `coc.nvim`
- **Emacs** - Using `lsp-mode`
- **IntelliJ** - Coming soon

## Performance

The LSP server includes:
- **Document caching** - 5-minute TTL for parsed documents
- **Parse time tracking** - Monitored for optimization
- **Incremental updates** - Only reparse changed documents
- **Timeout protection** - 5-second parse timeout

## License

MIT

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](https://github.com/developerfred/Bend-PVM/blob/main/CONTRIBUTING.md) for details.

## Support

- 📧 Issues: [GitHub Issues](https://github.com/developerfred/Bend-PVM/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/developerfred/Bend-PVM/discussions)
- 📖 Documentation: [Wiki](https://github.com/developerfred/Bend-PVM/wiki)
