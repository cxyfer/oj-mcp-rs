# Contributing to oj-mcp-rs

Thank you for your interest in contributing to oj-mcp-rs!

## Reporting Issues

If you find a bug or have a feature request, please search existing issues first to avoid duplicates. If no related issue exists, feel free to open a new one.

## Submitting Pull Requests

1. Fork this repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin feature/my-feature`)
5. Open a Pull Request

## Development Setup

### Prerequisites

- Rust 1.85+

### Build

```bash
git clone https://github.com/cxyfer/oj-mcp-rs.git
cd oj-mcp-rs
cargo build --release
```

The compiled binary will be at `target/release/oj-mcp-rs`.

## Testing and Debugging

### MCP Inspector

Use [@modelcontextprotocol/inspector](https://github.com/modelcontextprotocol/inspector) to test and debug the MCP server:

```bash
npx @modelcontextprotocol/inspector ./target/release/oj-mcp-rs --base-url https://craboj.zeabur.app
```

For testing authenticated endpoints:

```bash
npx @modelcontextprotocol/inspector ./target/release/oj-mcp-rs --base-url https://craboj.zeabur.app --token YOUR_TOKEN
```

The inspector will start a web interface (usually at `http://localhost:5173`) where you can:
- Browse available tools
- Test tool invocations
- View request/response logs
- Inspect server capabilities

## Code Quality

### Formatter

Use **rustfmt** (`cargo fmt`) for code formatting:

```bash
# Check formatting
cargo fmt -- --check

# Apply formatting
cargo fmt
```

### Linter

Use **Clippy** for static analysis:

```bash
# Run clippy
cargo clippy --all-features -- -D warnings

# Auto-fix (some issues)
cargo clippy --fix --allow-dirty
```

## Pre-submit Checklist

- [ ] Run `cargo fmt` to format code
- [ ] Run `cargo clippy --all-features -- -D warnings` to ensure no warnings
- [ ] Run `cargo test` to pass all tests
- [ ] Verify functionality with MCP Inspector

## Project Structure

```
src/
├── main.rs      # Entry point
├── client.rs    # HTTP client
├── models.rs    # Data models
├── error.rs     # Error handling
├── convert.rs   # HTML to Markdown conversion
└── tools/       # MCP tool implementations
    ├── mod.rs
    ├── daily.rs
    ├── problem.rs
    ├── resolve.rs
    ├── similar.rs
    └── status.rs
```

## License

Contributions will be licensed under the same [GPL v3](LICENSE) license as the project.
