# oj-mcp-rs

An [MCP](https://modelcontextprotocol.io/) server that wraps the [oj-api-rs](https://craboj.zeabur.app) REST API, exposing online judge problem data to LLM clients via stdio transport.

## Tools

| Tool | Description |
|---|---|
| `get_daily_challenge` | Get the LeetCode daily challenge (supports `com` / `cn` domains) |
| `get_problem` | Fetch a problem by source platform and ID |
| `find_similar_problems` | Semantic search by problem ID or free-text query |
| `resolve_problem` | Auto-detect a problem from URL, slug, or pattern |
| `get_platform_status` | Show backend platform support statistics (requires `--token`) |

## Install

### npm (recommended)

```sh
npx oj-mcp-rs --base-url https://craboj.zeabur.app
```

Pre-built binaries are published for:

| OS | Architecture |
|---|---|
| Linux | x64, arm64 |
| macOS | x64, arm64 |
| Windows | x64, arm64 |

### Build from source

Requires Rust 1.85+ (edition 2024).

```sh
cargo build --release
./target/release/oj-mcp-rs --base-url https://craboj.zeabur.app
```

## Usage

```
oj-mcp-rs --base-url <URL> [--token <TOKEN>]
```

| Flag | Required | Description |
|---|---|---|
| `--base-url` | Yes | oj-api-rs origin (e.g. `https://craboj.zeabur.app`) |
| `--token` | No | Bearer token for authenticated endpoints (`get_platform_status`) |
| `--version` | - | Print version and exit |

### Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "oj": {
      "command": "npx",
      "args": ["-y", "oj-mcp-rs", "--base-url", "https://craboj.zeabur.app"]
    }
  }
}
```

With authentication:

```json
{
  "mcpServers": {
    "oj": {
      "command": "npx",
      "args": ["-y", "oj-mcp-rs", "--base-url", "https://craboj.zeabur.app", "--token", "YOUR_TOKEN"]
    }
  }
}
```

## License

MIT
