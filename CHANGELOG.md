# Changelog

## [0.1.0] - 2026-02-26

### ✨ Features

- MCP server with stdio transport for Online Judge problem data
- Tool handlers: `get_problem`, `search_problems`, `get_contest`, `search_contests`, `get_user`
- HTML-to-markdown conversion for problem statements
- HTTP client wrapper around oj-api-rs backend
- Typed DTO models with serde serialization

### 📦 Build & Distribution

- npm distribution packages (linux/darwin/windows × x64/arm64)
- GitHub Actions release workflow (cargo build + npm publish)

### 📝 Documentation

- README with installation and usage guide
- LLM-friendly MCP tool and parameter descriptions
