# Protobuf Edition 2023 LSP Extension

This extension provides language support for Protocol Buffers with edition 2023 syntax.

## Features

- Syntax highlighting
- Real-time diagnostics
- Code completion
- Hover information

## Installation

1. Build the LSP server:
   ```bash
   cd ..
   cargo build --release
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Compile the extension:
   ```bash
   npm run compile
   ```

4. Package the extension:
   ```bash
   npm run package
   ```

5. Install the generated `.vsix` file in VSCode.

## Development

- Run `npm run watch` for development
- Press F5 in VSCode to launch a new Extension Development Host window