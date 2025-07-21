# Protocol Buffers Edition 2023 Language Server

A Language Server Protocol (LSP) implementation for Protocol Buffers with full support for edition 2023 syntax, written in Rust.

## Features

- **Edition 2023 Support**: Full support for the latest Protocol Buffers edition syntax
- **Real-time Diagnostics**: Instant syntax and semantic error checking as you type
- **Code Completion**: Context-aware completions for:
  - Field types (int32, string, bool, etc.)
  - Message and enum types
  - Keywords (message, enum, service, etc.)
- **Hover Documentation**: Tooltips for Protocol Buffers types and keywords
- **Traditional Syntax Support**: Compatible with proto2/proto3 syntax

## Installation

### VSCode Extension

The easiest way to use this language server is through the VSCode extension:

1. Build and package the extension:
   ```bash
   cd vscode-extension
   pnpm install
   pnpm run package
   ```

2. Install the generated `.vsix` file in VSCode:
   - Open VSCode
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS)
   - Run "Extensions: Install from VSIX..."
   - Select the `protobuf-edition-lsp-0.1.0.vsix` file

### Manual Installation

If you want to use the language server with other editors:

1. Build the language server:
   ```bash
   cargo build --release
   ```

2. The binary will be available at `target/release/protobuf-edition-lsp`

3. Configure your editor to use this binary as the language server for `.proto` files

## Development

### Prerequisites

- Rust 1.70+ (for building the language server)
- Node.js 18+ and pnpm (for the VSCode extension)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/your-username/protobuf-edition-lsp.git
cd protobuf-edition-lsp

# Build the language server
cargo build --release

# Run tests
cargo test

# Build the VSCode extension
cd vscode-extension
pnpm install
pnpm run compile
```

### Project Structure

```
├── src/
│   ├── main.rs              # Entry point
│   ├── lsp_server/          # LSP server implementation
│   │   ├── mod.rs           # Server trait implementation
│   │   ├── handlers.rs      # LSP feature handlers
│   │   └── document_store.rs # Document state management
│   └── parser/              # Protocol Buffers parser
│       ├── mod.rs           # AST definitions
│       ├── lexer.rs         # Tokenizer
│       ├── parser_impl.rs   # Recursive descent parser
│       └── validator.rs     # Semantic validation
├── tests/                   # Integration tests
└── vscode-extension/        # VSCode extension
```

### Testing

The project follows Test-Driven Development (TDD) principles:

```bash
# Run all tests
cargo test

# Run specific test module
cargo test --test parser_test
cargo test --test lsp_server_test

# Run with output
cargo test -- --nocapture
```

### Architecture

The language server is built with:
- **tower-lsp**: Provides the LSP framework
- **Tokio**: Async runtime
- **Functional Design**: Core logic implemented as pure functions

Key design principles:
- Parser continues on errors for better error recovery
- Validation is a separate pass after parsing
- LSP handlers are pure functions that don't modify state

## Supported Protocol Buffers Features

### Edition 2023
```protobuf
edition = "2023";

message Example {
  string name = 1;
  int32 id = 2;
  repeated string tags = 3;
}
```

### Traditional Syntax
```protobuf
syntax = "proto3";

message Example {
  string name = 1;
  int32 id = 2;
  repeated string tags = 3;
}
```

### Features
- Messages with nested types
- Enums with aliases
- Services and RPC methods
- Import statements
- Package declarations
- Options (file, message, field level)
- Comments and documentation

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

[MIT License](LICENSE)

## Acknowledgments

Built with [tower-lsp](https://github.com/tower-lsp/tower-lsp) and inspired by the official Protocol Buffers specification.