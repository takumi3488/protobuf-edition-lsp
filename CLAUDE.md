# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Language Server Protocol (LSP) implementation for Protocol Buffers with edition 2023 support, written in Rust using the `tower-lsp` framework. The project follows Test-Driven Development (TDD) principles and functional programming style.

## Build and Test Commands

```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Run all tests
cargo test

# Run specific test module
cargo test --test parser_test
cargo test --test lsp_server_test

# Run tests with output
cargo test -- --nocapture

# Run the LSP server
cargo run --release
```

## Architecture

The codebase is organized into two main modules:

### Parser Module (`src/parser/`)
- **`mod.rs`**: Type definitions for the Protocol Buffers AST (Abstract Syntax Tree)
- **`lexer.rs`**: Tokenizes Protocol Buffers source code into tokens
- **`parser_impl.rs`**: Recursive descent parser that converts tokens into AST
- **`validator.rs`**: Validates the parsed AST for semantic errors (e.g., duplicate field numbers)

The parser follows a functional approach with pure functions that return `Result<T, ParseError>`. It supports:
- Protocol Buffers edition 2023 syntax
- Traditional proto2/proto3 syntax
- Messages, enums, services, imports, and options

### LSP Server Module (`src/lsp_server/`)
- **`mod.rs`**: Main LSP server implementation using `tower-lsp::LanguageServer` trait
- **`document_store.rs`**: In-memory storage for opened documents
- **`handlers.rs`**: Pure functions that implement LSP features:
  - `compute_diagnostics`: Parses and validates protobuf files, returns errors
  - `compute_completions`: Provides context-aware completions (types, keywords)
  - `compute_hover`: Shows documentation for types and keywords

The server maintains document state in `DocumentStore` and delegates actual processing to pure functions in `handlers.rs`.

## Key Design Principles

1. **Functional Style**: Core logic (parsing, validation, LSP features) is implemented as pure functions that don't modify state
2. **TDD Approach**: Tests are in `tests/` directory and were written before implementation
3. **Error Handling**: Uses `anyhow::Result` for internal errors and custom error types for parsing
4. **Async Runtime**: Uses Tokio for the LSP server runtime

## LSP Features Implemented

- **Diagnostics**: Real-time syntax and semantic error checking
- **Completion**: Context-aware completions for field types and keywords
- **Hover**: Documentation tooltips for Protocol Buffers types and keywords
- **Document Synchroni zation**: Full text synchronization mode

## Development Notes

- The parser is designed to be resilient and continue parsing even when encountering errors
- Validation is a separate pass after parsing, allowing for better error recovery
- The LSP server logs to stderr (can be viewed in VSCode's "Output" panel)
- All tests must pass before committing changes