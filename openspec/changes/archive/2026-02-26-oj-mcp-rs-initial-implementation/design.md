## Context

A new Rust MCP server (`oj-mcp-rs`) wrapping the deployed `oj-api-rs` REST API (`https://craboj.zeabur.app/api/v1/*`). The server is a thin HTTP client layer communicating via stdio transport, exposing 5 tools for Online Judge problem data. No existing codebase — greenfield implementation.

Key constraints from proposal: Rust edition 2024, single binary, `rmcp` 0.16 with macros, `reqwest` with `rustls-tls`, `htmd` for HTML-to-Markdown, `clap` for CLI, npm distribution via JS shim for 6 platforms.

## Goals / Non-Goals

**Goals:**
- Expose 5 MCP tools (get_daily_challenge, get_problem, find_similar_problems, resolve_problem, get_platform_status) via stdio transport
- Clean error separation: protocol-level (`McpError`) vs domain-level (`CallToolResult` with `is_error: true`)
- Safe HTML-to-Markdown conversion with fallback on failure
- Cross-platform npm distribution via JS shim + platform-specific binary packages
- Forward-compatible API response deserialization

**Non-Goals:**
- HTTP/SSE server mode
- Embedding `oj-api-rs` as a library
- Retry logic or rate-limit backoff (relay errors to LLM client)
- Caching of API responses
- Workspace or sub-crate structure

## Decisions

### D1: Module Structure — Sub-modularized tools/

```
src/
  main.rs          # CLI parsing (clap), server bootstrap, tracing init
  client.rs        # reqwest HTTP client wrapper
  models.rs        # API response DTOs (serde, forward-compatible)
  error.rs         # RFC 7807 parsing, error classification, formatting helpers
  convert.rs       # HTML-to-Markdown (htmd) + fallback (ammonia strip tags)
  tools/
    mod.rs         # OjServer struct, #[tool_router], #[tool_handler] impl
    daily.rs       # T1: get_daily_challenge
    problem.rs     # T2: get_problem
    similar.rs     # T3: find_similar_problems
    resolve.rs     # T4: resolve_problem
    status.rs      # T5: get_platform_status
```

**Rationale:** 5 tools with distinct parameter validation and response formatting logic. Sub-modules prevent `tools.rs` from growing unwieldy and reduce merge conflicts. Each tool file contains its parameter struct (`#[derive(Deserialize, JsonSchema)]`) and handler function. `tools/mod.rs` aggregates via `#[tool_router]` on `OjServer`.

**Alternative considered:** Single `tools.rs` — simpler but harder to maintain as tools grow. Rejected for extensibility.

### D2: rmcp Integration Pattern — `#[tool(aggr)]` with `schemars::JsonSchema`

```rust
use rmcp::{ServerHandler, tool, tool_router, tool_handler,
    handler::server::tool::ToolRouter, model::*, schemars};
use serde::Deserialize;

#[derive(Deserialize, schemars::JsonSchema)]
pub struct GetProblemParams {
    #[schemars(description = "Problem source (e.g. leetcode, codeforces, atcoder)")]
    pub source: String,
    #[schemars(description = "Problem ID")]
    pub id: String,
}

#[tool_router]
impl OjServer {
    #[tool(description = "Get a specific problem by source and ID")]
    async fn get_problem(&self, #[tool(aggr)] params: GetProblemParams) -> Result<CallToolResult, McpError> { ... }
}

#[tool_handler]
impl ServerHandler for OjServer { ... }
```

**Rationale:** `rmcp` re-exports `schemars`, no extra dependency needed. `#[tool(aggr)]` maps JSON parameters directly to struct fields, providing automatic schema generation with `#[schemars(description)]` for per-field documentation visible to LLM clients.

### D3: Error Classification — Dual-layer with `error.rs`

| Error Source | Classification | Return Type |
|---|---|---|
| Network failure (timeout, connection refused) | Protocol-level | `Err(McpError)` |
| Malformed response (non-JSON body, invalid JSON) | Protocol-level | `Err(McpError)` |
| HTTP 4xx/5xx with RFC 7807 body | Domain-level | `Ok(CallToolResult { is_error: true })` |
| HTTP 4xx/5xx with non-RFC body | Domain-level | `Ok(CallToolResult { is_error: true })` |
| HTTP 202 (daily fetching) | Informational | `Ok(CallToolResult { is_error: false })` |
| Missing token for authenticated endpoint | Domain-level | `Ok(CallToolResult { is_error: true })` |
| Parameter validation failure (e.g. T3 mode) | Domain-level | `Ok(CallToolResult { is_error: true })` |

RFC 7807 format: `[{status}] {title}: {detail}`
Non-RFC fallback: `[{status_code}] {body_truncated_to_500_chars}`

**Rationale:** Protocol errors cause reconnection; domain errors are informational for the LLM. Keeping them strictly separated prevents MCP session drops on expected API failures.

### D4: HTML-to-Markdown — `htmd` with `catch_unwind` + `ammonia` fallback

```rust
pub fn html_to_markdown(html: &str) -> String {
    if html.trim().is_empty() { return "No description available.".into(); }
    match std::panic::catch_unwind(|| htmd::convert(html)) {
        Ok(Ok(md)) if !md.trim().is_empty() => md,
        _ => ammonia::clean_text(html), // strip all tags, return plain text
    }
}
```

**Rationale:** `htmd` may panic on malformed HTML. `catch_unwind` prevents MCP session crash. `ammonia` (already a mature, audited crate) provides safe tag stripping as fallback. Plain text is always preferable to no output.

### D5: Nullable Fields — `Option<T>` with "N/A" display

All potentially-null API fields use `Option<T>`:
- `tags: Option<Vec<String>>` → display as comma-separated or "N/A"
- `ac_rate: Option<f64>` → display as `{:.1}%` or "N/A"
- `difficulty: Option<String>` → display as-is or "N/A"
- `rating: Option<f64>` → display as `{:.1}` or "N/A"

**Rationale:** The API may return null for fields on certain platforms (e.g. Luogu problems lack ac_rate). Using `Option` with `#[serde(default)]` ensures forward compatibility.

### D6: HTTP Client Configuration

- Timeout: **30 seconds** (unified for all endpoints)
- `reqwest::Client` shared via `Arc` (single instance, connection pooling by default)
- `rustls-tls` (no OpenSSL dependency, better cross-compilation)
- Optional `Authorization: Bearer <token>` header (applied to all requests when token is present)
- Redirect policy: `reqwest` default (follow up to 10 redirects)

### D7: Serde Forward Compatibility

All API DTOs use default serde behavior (no `deny_unknown_fields`). Unknown fields from future API versions are silently ignored. This prevents deserialization failures when the backend adds new fields.

### D8: Logging & Diagnostics

- `tracing-subscriber` with `env-filter`, default level: `info`
- Override via `RUST_LOG` environment variable
- All output to **stderr only** — stdout is reserved for MCP protocol
- Token values are never logged

### D9: Server Metadata

- Server name: `"oj-mcp-rs"`
- Server version: `env!("CARGO_PKG_VERSION")` (from Cargo.toml)
- CLI supports `--version` flag via `clap`
- MCP capabilities: tools only (no resources, prompts, or logging capabilities)

### D10: Output Format — `Content::text()` with Markdown

All tools return `CallToolResult::success(vec![Content::text(markdown_string)])`. The text content is Markdown-formatted but uses the `text` content type as specified in the proposal. LLM clients parse Markdown from text content natively.

### D11: Additional Dependency — `ammonia`

```toml
ammonia = "4"  # HTML sanitizer for strip-tags fallback
```

Added to the dependency list from the proposal. Used only in the `catch_unwind` fallback path in `convert.rs`.

### D12: Number Formatting

- `ac_rate`: `{:.1}%` (one decimal place, e.g. "57.1%") or "N/A"
- `similarity`: `{:.1}%` (percentage form, e.g. "79.0%")
- `total`, `missing_content`, `not_embedded`: comma-separated thousands (e.g. "12,984")

## Risks / Trade-offs

### [Risk] rmcp 0.16 macro instability → Mitigation: Pin exact version, add compile-time smoke test

`#[tool_router]` and `#[tool_handler]` are proc macros whose generated code may change between minor versions. Use an exact version pin `rmcp = "=0.16.0"` (not `"0.16"` which is a semver range `^0.16`). Add a basic integration test that lists tools to catch registration failures.

### [Risk] htmd conversion quality for complex HTML → Mitigation: ammonia fallback + content truncation

Math formulas, deeply nested tables, and interactive elements may not convert cleanly. The `ammonia` fallback ensures the LLM always receives usable text. Output is capped at reasonable size.

### [Risk] Cross-compilation for 6 platforms → Mitigation: CI smoke test per platform

`aarch64-unknown-linux-gnu` requires `cross` crate. `aarch64-pc-windows-msvc` has limited ecosystem support. Each platform binary should be smoke-tested with `--version` in CI.

### [Risk] npm shim path resolution on Windows → Mitigation: Explicit `.exe` extension handling

Windows requires `.exe` suffix. The JS shim maps `win32`/`cygwin` → `windows` and appends `.exe`. Error message includes available platforms when binary not found.

### [Risk] API returning unexpected Content-Type → Mitigation: Check status + content-type before JSON parse

If response is not `application/json`, read as text and use non-RFC7807 fallback format. This prevents deserialization panics on HTML error pages or plain text responses.

### [Risk] Large API responses consuming excessive memory → Mitigation: Response body size limit

Cap response body reading at 1MB. Truncate output Markdown at 100KB with `... (truncated)` suffix.
