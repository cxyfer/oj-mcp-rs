# Changelog

## [0.1.2] - 2026-02-27

### 📝 Documentation

- Add CONTRIBUTING.md with contribution guidelines
- Add GPL v3 license file
- Refactor README structure with badges
- Improve `get_daily_challenge` timezone descriptions

### 🔧 Maintenance

- Fix all cargo clippy warnings
- Apply cargo fmt formatting
- Sync Cargo.lock version

## [0.1.1] - 2026-02-26

### 🐛 Bug Fixes

- Fix npm package shebang and optionalDependencies version placeholders
- Fix CI workflow for publish-npm-base job triggers
- Add @types/node and workflow_dispatch for base package

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
