# Proposal: oj-mcp-rs Initial Implementation

## Context

Build a Rust MCP (Model Context Protocol) server that wraps the `oj-api-rs` REST API, exposing Online Judge problem data as MCP tools. The server acts as a thin HTTP client layer — it does NOT embed `oj-api-rs` as a library, but calls its deployed REST endpoints.

Target backend: `https://craboj.zeabur.app/api/v1/*` (configurable via CLI args).

## Requirements

### R1: CLI Configuration

The binary accepts two CLI arguments:
- `--base-url <URL>` (required) — base URL of the oj-api-rs instance (e.g. `https://craboj.zeabur.app`)
- `--token <TOKEN>` (optional) — Bearer token for authenticated endpoints

These are passed via MCP client config (e.g. Claude Desktop `claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "oj": {
      "command": "oj-mcp-rs",
      "args": ["--base-url", "https://craboj.zeabur.app", "--token", "xxx"]
    }
  }
}
```

### R2: Transport — stdio only

Server communicates via stdin/stdout using `rmcp`'s stdio transport. No HTTP/SSE server mode.

### R3: MCP Tools (5 tools)

#### T1: `get_daily_challenge`
- Description: Get LeetCode daily challenge problem
- Parameters:
  - `domain` (optional, enum: `com` | `cn`, default: `com`)
  - `date` (optional, string `YYYY-MM-DD`, default: today in UTC)
- Maps to: `GET /api/v1/daily?domain={domain}&date={date}`
- When `date` is omitted, use `chrono::Utc::now().date_naive()` to derive the date string
- On HTTP 202 (fetching): return a message indicating the problem is being fetched, suggest retry
- Response: formatted problem content (Markdown with metadata header)

#### T2: `get_problem`
- Description: Get a specific problem by source and ID
- Parameters:
  - `source` (required, string — e.g. `leetcode`, `atcoder`, `codeforces`)
  - `id` (required, string — problem ID)
- Path segments (`source`, `id`) must be percent-encoded via `urlencoding::encode()`
- Maps to: `GET /api/v1/problems/{source}/{id}`
- Response: formatted problem content (Markdown with metadata header)

#### T3: `find_similar_problems`
- Description: Find similar problems by problem ID or text query
- Parameters (mutually exclusive modes):
  - Mode A — by ID: `source` (required) + `id` (required)
  - Mode B — by text: `query` (required, 3–2000 chars)
  - `limit` (optional, integer, default: 10, range: 1–50)
  - `threshold` (optional, float, default: 0.0, range: 0.0–1.0)
  - `source_filter` (optional, comma-separated source filter)
- Validation rules:
  - If `query` is provided, use Mode B (ignore `source`/`id` even if present)
  - If `query` is absent, both `source` and `id` must be present (Mode A)
  - If neither mode is satisfied, return `CallToolResult` with `is_error: true` and a descriptive message
- Maps to:
  - Mode A: `GET /api/v1/similar/{source}/{id}?limit=&threshold=&source=`
  - Mode B: `GET /api/v1/similar?q={query}&limit=&threshold=&source=`
- Path segments in Mode A must be percent-encoded
- Response: list of similar problems with similarity scores

#### T4: `resolve_problem`
- Description: Auto-detect and resolve a problem from URL, slug, or pattern
- Parameters:
  - `query` (required, string — URL, slug, prefixed ID, or bare pattern)
- The `query` value must be percent-encoded before interpolation into the path segment
- Maps to: `GET /api/v1/resolve/{query}`
- Response: resolved problem content (Markdown with metadata header)

#### T5: `get_platform_status`
- Description: Get the support status of each platform on the backend
- Parameters: none
- Maps to: `GET /status` (root path, not under `/api/v1/`)
- Requires `Authorization: Bearer <token>` header (same token as other tools)
- Response: Markdown table of platform support status
- Output format:
```markdown
# OJ Platform Status (v{version})

| Platform   | Problems | Missing Content | Not Embedded |
|------------|----------|-----------------|--------------|
| atcoder    | 8,356    | 320             | 339          |
| codeforces | 12,984   | 106             | 127          |
| leetcode   | 3,760    | 729             | 729          |
| luogu      | 15,393   | 2               | 13,358       |
```
- Backend status response schema:
```json
{
  "version": "0.1.4",
  "platforms": [
    { "source": "string", "total": 0, "missing_content": 0, "not_embedded": 0 }
  ]
}
```
- On auth failure (no token / 401): return `CallToolResult` with `is_error: true`

### R4: HTML-to-Markdown Conversion

API returns problem content as HTML. The MCP server must convert it to Markdown before returning to the LLM client. Use a lightweight HTML-to-Markdown crate (e.g. `htmd` or `html2md`).

### R5: Error Handling

- API errors follow RFC 7807 format (`{ type, title, status, detail }`)
- Error classification strategy:
  - **Protocol-level errors** (`McpError` / `ErrorData`): network failures (timeout, connection refused), malformed responses (non-JSON body, invalid JSON)
  - **Domain-level errors** (`CallToolResult` with `is_error: true`): HTTP 4xx/5xx from API (404 not found, 401/403 auth failure, 429 rate limit, 500 server error)
- For RFC 7807 responses, format message as: `[{status}] {title}: {detail}`
- For non-RFC7807 error bodies, fallback to: `[{status_code}] {response_body_text_truncated_to_500_chars}`
- Network errors (timeout, connection refused) must produce clear error text, not panics
- HTTP 202 on daily endpoint: return informational text, not an error

### R6: HTTP Client

Use `reqwest` with:
- Configurable base URL
- Optional `Authorization: Bearer <token>` header
- Reasonable timeout (30s)
- Connection pooling (default reqwest behavior)

## Constraints

### Hard Constraints
- **Rust edition 2024**, MSRV aligned with latest stable
- **rmcp** crate with features `macros`, `server`, `transport-io`
- **stdio transport only** — no HTTP server mode
- **All logs/diagnostics must go to stderr only** — no `println!` or stdout writes outside MCP transport, as they corrupt the protocol stream
- **No embedded oj-api-rs** — pure HTTP client against deployed instance
- **Single binary** — no workspace, no sub-crates
- All tool responses must be `CallToolResult` with `Content::text()`
- Error type: `rmcp::ErrorData` (aliased as `McpError` in project code)

### Soft Constraints
- Prefer `clap` for CLI argument parsing
- Prefer `serde` + `serde_json` for JSON deserialization
- Prefer `reqwest` with `rustls-tls` (no OpenSSL dependency)
- Minimize dependencies — no unnecessary crates

### R7: npx Distribution

The binary must be publishable to npm and runnable via `npx`:

```json
{
  "command": "npx",
  "args": ["-y", "oj-mcp-rs@latest", "--base-url", "https://craboj.zeabur.app", "--token", "xxx"]
}
```

#### npm Package Structure

```
npm/
├── package.json.tmpl          # Template for platform-specific packages
└── oj-mcp-rs/
    ├── package.json           # Root package (shim only, no binary)
    └── src/
        └── index.ts           # JS shim that resolves and spawns platform binary
```

#### Root package (`npm/oj-mcp-rs/package.json`)
```json
{
  "name": "oj-mcp-rs",
  "version": "${VERSION}",
  "bin": "lib/index.js",
  "optionalDependencies": {
    "oj-mcp-rs-linux-x64":    "${VERSION}",
    "oj-mcp-rs-linux-arm64":  "${VERSION}",
    "oj-mcp-rs-darwin-x64":   "${VERSION}",
    "oj-mcp-rs-darwin-arm64": "${VERSION}",
    "oj-mcp-rs-windows-x64":  "${VERSION}",
    "oj-mcp-rs-windows-arm64":"${VERSION}"
  }
}
```

#### Platform package template (`npm/package.json.tmpl`)
```json
{
  "name": "${node_pkg}",
  "version": "${node_version}",
  "os": ["${node_os}"],
  "cpu": ["${node_arch}"]
}
```

#### JS shim logic (`npm/oj-mcp-rs/src/index.ts`)
- Uses `process.platform` + `process.arch` to construct package name
- Maps `win32` / `cygwin` → `windows`, appends `.exe` extension
- Resolves binary path via `require.resolve(`oj-mcp-rs-${os}-${arch}/bin/oj-mcp-rs${ext}`)`
- Spawns binary via `spawnSync(exePath, process.argv.slice(2), { stdio: "inherit" })`
- Exits with `processResult.status ?? 0`

#### Target platforms (6 total)

| npm package name        | Rust target                     | Runner OS        |
|-------------------------|---------------------------------|------------------|
| oj-mcp-rs-linux-x64     | x86_64-unknown-linux-gnu        | ubuntu-22.04     |
| oj-mcp-rs-linux-arm64   | aarch64-unknown-linux-gnu       | ubuntu-22.04 + cross |
| oj-mcp-rs-darwin-x64    | x86_64-apple-darwin             | macos-14         |
| oj-mcp-rs-darwin-arm64  | aarch64-apple-darwin            | macos-14         |
| oj-mcp-rs-windows-x64   | x86_64-pc-windows-msvc          | windows-2022     |
| oj-mcp-rs-windows-arm64 | aarch64-pc-windows-msvc         | windows-2022     |

**Note**: Package names use `windows` (not `win32`) to avoid npm spam detection.

#### GitHub Actions workflow (`.github/workflows/release.yml`)

Triggered on `push --tags v*`. Two jobs:
1. **`publish-npm-binaries`**: Build matrix × 6 platforms → cross-compile → publish platform-specific npm package
2. **`publish-npm-base`**: `needs: publish-npm-binaries` → build TS shim → `npm publish`

Linux arm64 cross-compilation uses `cross` crate (avoids QEMU overhead on native runners).

## Project Structure

```
oj-mcp-rs/
├── Cargo.toml
├── src/
│   ├── main.rs        # CLI parsing, server bootstrap
│   ├── models.rs      # API response DTOs (Problem, Similar, Resolve, Status, RFC7807)
│   ├── client.rs      # HTTP client wrapper for oj-api-rs
│   ├── tools.rs       # MCP tool definitions (#[tool_router] + #[tool_handler])
│   └── convert.rs     # HTML-to-Markdown conversion + metadata header formatting
├── npm/
│   ├── package.json.tmpl              # Platform-specific package template
│   └── oj-mcp-rs/
│       ├── package.json               # Root npm package
│       ├── tsconfig.json
│       └── src/
│           └── index.ts               # JS shim script
├── .github/
│   └── workflows/
│       └── release.yml                # Cross-compile + npm publish workflow
└── .gitignore
```

## Dependencies (Cargo.toml)

```toml
[dependencies]
rmcp = { version = "=0.16.0", features = ["macros", "server", "transport-io"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
clap = { version = "4", features = ["derive"] }
htmd = "0.5"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
chrono = { version = "0.4", features = ["clock"], default-features = false }
urlencoding = "2"
```

> rmcp 0.16.0 default features 包含 `base64`, `macros`, `server`。此處額外啟用 `transport-io` 以支援 stdio transport。

### rmcp Implementation Pattern

```rust
use rmcp::{ErrorData as McpError, model::*, tool, tool_router, tool_handler,
    handler::server::tool::ToolRouter, service::ServiceExt};

#[derive(Clone)]
pub struct OjServer {
    client: OjClient,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl OjServer {
    pub fn new(client: OjClient) -> Self {
        Self { client, tool_router: Self::tool_router() }
    }

    #[tool(description = "Get LeetCode daily challenge problem")]
    async fn get_daily_challenge(&self, /* params */) -> Result<CallToolResult, McpError> {
        // ...
        Ok(CallToolResult::success(vec![Content::text(markdown)]))
    }
}

#[tool_handler]
impl ServerHandler for OjServer {}
```

### Response Format Templates

Problem tools (T1, T2, T4) return Markdown with a metadata header:
```markdown
# {title}

- Source: {source} | ID: {id} | Difficulty: {difficulty}
- Tags: {tags_comma_separated}
- Link: {link}
- AC Rate: {ac_rate}%

---

{content_as_markdown}
```

Similar problems (T3) return:
```markdown
# Similar Problems

Query: {rewritten_query}

| # | Source | ID | Title | Difficulty | Similarity | Link |
|---|--------|----|-------|------------|------------|------|
| 1 | leetcode | 167 | ... | Medium | 79.0% | ... |
```

## Success Criteria

1. `cargo build --release` 成功編譯為單一二進位檔
2. 透過 stdio 與 MCP client 正常通訊（可用 `rmcp` inspector 或 Claude Desktop 驗證）
3. `get_daily_challenge` 回傳 LeetCode 每日一題的 Markdown 格式內容
4. `get_problem` 以 source + id 查詢回傳正確題目（支援 `leetcode`, `codeforces`, `atcoder`, `luogu`）
5. `find_similar_problems` 以 ID 或文字查詢回傳相似題目列表
6. `resolve_problem` 以 URL/slug/pattern 自動解析並回傳題目
7. `get_platform_status` 回傳後端 `/status` 資料的 Markdown 表格
8. 所有 HTML content 正確轉換為 Markdown
9. API 錯誤和網路錯誤產生有意義的錯誤訊息，不會 panic
10. `--base-url` 和 `--token` CLI 參數正常運作
11. `npx -y oj-mcp-rs@latest --base-url <url> --token <tok>` 可在 6 個平台上正確執行
12. GitHub Actions release workflow 在 tag push 後完成交叉編譯並發布至 npm

## API Response Reference

### Problem Object
```json
{
  "id": "1",
  "source": "leetcode",
  "slug": "two-sum",
  "title": "Two Sum",
  "difficulty": "Easy",
  "ac_rate": 57.08,
  "rating": 0.0,
  "tags": ["Array", "Hash Table"],
  "link": "https://leetcode.com/problems/two-sum/",
  "content": "<p>HTML content...</p>",
  "similar_questions": []
}
```

### Daily Response (HTTP 200)
Returns a Problem object directly.

### Daily Response (HTTP 202 — fetching)
```json
{ "retry_after": 30, "status": "fetching" }
```

### Similar Response
```json
{
  "rewritten_query": "...",
  "results": [
    { "source": "leetcode", "id": "167", "title": "...", "difficulty": "Medium", "link": "...", "similarity": 0.79 }
  ]
}
```

### Resolve Response
```json
{
  "source": "leetcode",
  "id": "1",
  "problem": { /* Problem object */ }
}
```

### Status Response
```json
{
  "version": "0.1.4",
  "platforms": [
    { "source": "atcoder", "total": 8356, "missing_content": 320, "not_embedded": 339 },
    { "source": "codeforces", "total": 12984, "missing_content": 106, "not_embedded": 127 },
    { "source": "leetcode", "total": 3760, "missing_content": 729, "not_embedded": 729 },
    { "source": "luogu", "total": 15393, "missing_content": 2, "not_embedded": 13358 }
  ]
}
```

```json
{ "type": "about:blank", "title": "Not Found", "status": 404, "detail": "problem not found" }
```
