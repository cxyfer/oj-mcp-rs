<div align="center">

# oj-mcp-rs

[![NPM Version](https://img.shields.io/npm/v/oj-mcp-rs?style=flat-square&color=cb3837&logo=npm)](https://www.npmjs.com/package/oj-mcp-rs)
[![License](https://img.shields.io/badge/License-GPL%20v3-blue.svg?style=flat-square)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![CI](https://github.com/cxyfer/oj-mcp-rs/actions/workflows/release.yml/badge.svg?branch=main)](https://github.com/cxyfer/oj-mcp-rs/actions)

*An [MCP](https://modelcontextprotocol.io/) server that wraps the [oj-api-rs](https://github.com/cxyfer/oj-api-rs) REST API, exposing online judge problem data to LLM clients via stdio transport.*

</div>

## Overview

oj-mcp-rs provides LLM clients with access to online judge problem data from various competitive programming platforms. It enables natural language interaction with problem descriptions, examples, constraints, and related problems.

## Features

- **Multi-platform Support** - Fetch problems from LeetCode (com/cn), Codeforces, AtCoder, Luogu, and more
- **Daily Challenge** - Get today's LeetCode daily challenge with a single command
- **Problem Retrieval** - Fetch complete problem data including description, examples, constraints, and hints
- **Semantic Search** - Find related problems by ID or free-text query using AI-powered similarity
- **Auto-detection** - Resolve problems from URLs, slugs, or patterns automatically
- **Platform Status** - Query backend platform support statistics (requires authentication)

## Installation

> [!NOTE]
> Replace `YOUR_BASE_URL` with your oj-api-rs instance URL. You can use `https://oj-api.zeabur.app` as a public demo instance.

Run via npx (no installation required):

```bash
npx oj-mcp-rs --base-url YOUR_BASE_URL
```

**Supported platforms:** Linux (x64, arm64) • macOS (x64, arm64) • Windows (x64, arm64)

| Argument | Required | Description |
|----------|----------|-------------|
| `--base-url` | Yes | oj-api-rs origin (e.g., `https://oj-api.zeabur.app`) |
| `--token` | No | Bearer token for authenticated endpoints |
| `--version` | - | Print version and exit |

**Environment Variables:**
- `RUST_LOG` - Set log level (e.g., `info`, `debug`, `warn`)

### Client Configuration

<details>
<summary><b>Claude Desktop</b></summary>

Add to your config file:
- **macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "oj": {
      "command": "npx",
      "args": ["-y", "oj-mcp-rs", "--base-url", "YOUR_BASE_URL"]
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
      "args": ["-y", "oj-mcp-rs", "--base-url", "YOUR_BASE_URL", "--token", "YOUR_TOKEN"]
    }
  }
}
```
</details>

<details>
<summary><b>Cursor</b></summary>

Add to `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "oj": {
      "command": "npx",
      "args": ["-y", "oj-mcp-rs", "--base-url", "YOUR_BASE_URL"]
    }
  }
}
```
</details>

<details>
<summary><b>VS Code</b></summary>

Add to `.vscode/mcp.json` in your workspace:

```json
{
  "servers": {
    "oj": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "oj-mcp-rs", "--base-url", "YOUR_BASE_URL"]
    }
  }
}
```
</details>

<details>
<summary><b>Claude Code</b></summary>

```bash
claude mcp add --transport stdio oj-mcp-rs -- npx -y oj-mcp-rs --base-url YOUR_BASE_URL
```
</details>

<details>
<summary><b>Codex</b></summary>

```bash
codex mcp add oj-mcp-rs -- npx -y oj-mcp-rs --base-url YOUR_BASE_URL
```
</details>

<details>
<summary><b>Build from Source</b></summary>

Requires Rust 1.85+.

```bash
git clone https://github.com/cxyfer/oj-mcp-rs.git
cd oj-mcp-rs
cargo build --release
```

The binary will be at `target/release/oj-mcp-rs`.
</details>

## Available Tools

<details>
<summary><code>get_daily_challenge</code> — Get the LeetCode daily challenge</summary>

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `domain` | string | No | Domain to use: `"com"` (default) or `"cn"` |

**Example:**
```
What is today's LeetCode daily challenge?
```
</details>

<details>
<summary><code>get_problem</code> — Fetch a problem by source platform and ID</summary>

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `source` | string | Yes | Platform: `"leetcode"`, `"codeforces"`, `"atcoder"`, `"luogu"`, etc. |
| `id` | string | Yes | Problem ID (e.g., `"1"`, `"1A"`, `"awc0001_a"`) |

**Example:**
```
Get LeetCode problem 1. Two Sum
```
</details>

<details>
<summary><code>find_similar_problems</code> — Semantic search by problem ID or free-text query</summary>

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `source` | string | No | Platform to search (default: `"leetcode"`) |
| `query` | string | Yes | Problem ID or free-text query |
| `limit` | number | No | Maximum results to return (default: 5) |

**Example:**
```
Find problems similar to LeetCode 146 LRU Cache
```
</details>

<details>
<summary><code>resolve_problem</code> — Auto-detect a problem from URL, slug, or pattern</summary>

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | URL, slug, or pattern (e.g., `"https://leetcode.com/problems/two-sum/"`) |

**Example:**
```
What is this problem? https://leetcode.com/problems/median-of-two-sorted-arrays/
```
</details>

<details>
<summary><code>get_platform_status</code> — Show backend platform support statistics</summary>

**Note:** Requires authentication token (`--token`).

**Example:**
```
Show backend platform support statistics
```
</details>

## Examples

Once connected, you can use natural language to interact with online judge data:

| Query | Description |
|-------|-------------|
| "What is today's LeetCode daily challenge?" | Get today's daily challenge |
| "Get LeetCode problem 1. Two Sum" | Fetch a specific problem |
| "Find problems similar to LeetCode 146 LRU Cache" | Semantic similarity search |
| "What is this problem? https://leetcode.com/problems/median-of-two-sorted-arrays/" | URL resolution |
| "Show backend platform support statistics" | Query platform status |

## Supported Platforms

- **LeetCode** (leetcode.com, leetcode.cn)
- **Codeforces** (codeforces.com)
- **AtCoder** (atcoder.jp)
- **Luogu** (luogu.com.cn)
- And more via oj-api-rs backend

## License

[GPL v3](https://www.gnu.org/licenses/gpl-3.0)
