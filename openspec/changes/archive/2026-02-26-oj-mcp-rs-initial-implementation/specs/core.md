# Core Specification

Covers CLI configuration, HTTP client, MCP server initialization, error classification, response size limits, and logging.

---

## S1: CLI Configuration

### S1.1: `--base-url` (required)

The binary accepts `--base-url <URL>` via `clap`. The value must be a valid **origin** (scheme + host + optional port). No path, query, or fragment components are permitted.

**Normalization:** Parse once via `url::Url`, then strip any trailing `/` from the string representation. Store as `String` for subsequent path appending.

**Rejection criteria** (fail at startup with non-zero exit):
- Missing `--base-url` argument
- Value fails `Url::parse`
- Parsed URL has a non-empty path (anything other than `/`)
- Parsed URL has query or fragment components
- Scheme is not `http` or `https`

**Request construction:** Append literal absolute paths to the normalized origin.

```
base_url = "https://craboj.zeabur.app"
request  = format!("{base_url}/api/v1/daily?domain=com&date=2026-01-01")
```

### S1.2: `--token` (optional)

Optional Bearer token string. When present, attached as `Authorization: Bearer {token}` header to every outbound request. When absent, requests omit the header entirely.

### S1.3: `--version`

Prints `oj-mcp-rs {version}` from `env!("CARGO_PKG_VERSION")` and exits.

### PBT Properties

```
[INVARIANT] For any string S passed as --base-url:
  If S is a valid URL with path "/" and no query/fragment, startup succeeds.
  Otherwise, startup fails with a config error before MCP handshake.
[FALSIFICATION] Provide "https://example.com/api/v1" (has path) → must reject.
                Provide "https://example.com?foo=1" (has query) → must reject.
                Provide "ftp://example.com" (non-http scheme) → must reject.
                Provide "not a url" (invalid) → must reject.
                Provide "https://example.com" → must accept.
                Provide "https://example.com/" → must accept (trailing slash stripped).

[INVARIANT] The stored base_url never ends with '/'.
[FALSIFICATION] Input "https://host:8080/" → stored as "https://host:8080".

[INVARIANT] --token value is never logged to stderr at any log level.
[FALSIFICATION] Set RUST_LOG=trace, provide --token secret123, capture stderr →
               "secret123" must not appear.
```

---

## S2: HTTP Client

### S2.1: Construction

A single `reqwest::Client` instance constructed at startup:
- Timeout: 30 seconds
- TLS: `rustls-tls` (no OpenSSL)
- Redirect: default (follow up to 10)
- Shared via `Arc<reqwest::Client>` within `OjClient`

When `--token` is provided, set a default `Authorization: Bearer {token}` header on the client builder.

### S2.2: Response Body Size Limit

Read at most **1,048,576 bytes** (1 MiB) from any HTTP response body. Use streaming reads (`response.chunk()` in a loop) and stop after accumulating the cap. This prevents unbounded memory allocation from malicious or buggy upstream responses. Do NOT use `response.bytes()` which buffers the entire body first.

### S2.3: Content-Type Validation

Before JSON deserialization, verify the response `Content-Type` header starts with `application/json` or `application/problem+json`. If not, treat the entire response body as a non-RFC 7807 error and format with the fallback template.

### PBT Properties

```
[INVARIANT] All outbound requests have timeout <= 30s.
[FALSIFICATION] Inject a server that never responds → client returns error
               within 30s +/- 1s tolerance.

[INVARIANT] Response body bytes stored in memory never exceed 1,048,576.
[FALSIFICATION] Serve a 2 MiB body → client reads exactly 1,048,576 bytes.

[INVARIANT] If Content-Type is not application/json or application/problem+json, JSON parse is never attempted.
[FALSIFICATION] Serve Content-Type: text/html with body "<html>...</html>" →
               error formatted as non-RFC fallback, no serde error.

[INVARIANT] When token is Some(t), every request carries "Authorization: Bearer {t}".
            When token is None, no Authorization header is present.
[FALSIFICATION] Capture outbound headers for both cases → verify presence/absence.
```

---

## S3: Server Initialization

### S3.1: ServerInfo

```rust
ServerInfo {
    name: "oj-mcp-rs".into(),
    version: env!("CARGO_PKG_VERSION").into(),
}
```

### S3.2: Capabilities

Tools-only server:
- `tools` capability enabled with `listChanged: false`
- No `resources`, `prompts`, or `logging` capabilities advertised

### S3.3: Transport

stdio only. The server reads JSON-RPC from stdin and writes to stdout. No HTTP/SSE listener.

### S3.4: Bootstrap Sequence

1. Parse CLI args (`clap`)
2. Validate `--base-url` (reject non-origin)
3. Initialize `tracing-subscriber` (stderr, env-filter)
4. Build `reqwest::Client` with optional Bearer token
5. Construct `OjServer` with `OjClient` and `ToolRouter`
6. Start rmcp stdio transport, serve until stdin closes

### PBT Properties

```
[INVARIANT] The initialize response always contains:
  server_info.name == "oj-mcp-rs"
  server_info.version == env!("CARGO_PKG_VERSION")
  capabilities.tools is present
  capabilities.resources is absent
  capabilities.prompts is absent
  capabilities.logging is absent
[FALSIFICATION] Send initialize request → parse response JSON →
               assert exact field values and absence of non-tool capabilities.

[INVARIANT] tools/list returns exactly 5 tools:
  ["get_daily_challenge", "get_problem", "find_similar_problems",
   "resolve_problem", "get_platform_status"]
[FALSIFICATION] Send tools/list → assert result length == 5 and names match.
```

---

## S4: Error Classification

### S4.1: Protocol-Level Errors → `Err(McpError)`

Returned as `Err(McpError)` (rmcp `ErrorData`), causing the MCP client to see a JSON-RPC error response.

| Trigger | Example |
|---|---|
| Network failure | Connection refused, DNS resolution failure, timeout |
| Malformed response | Non-UTF-8 body, invalid JSON structure |
| Internal bug | Unexpected panic caught at tool boundary |

### S4.2: Domain-Level Errors → `Ok(CallToolResult { is_error: true })`

Returned as successful tool results with `is_error: true`.

| Trigger | Example |
|---|---|
| HTTP 4xx | 404 Not Found, 401 Unauthorized, 429 Too Many Requests |
| HTTP 5xx | 500 Internal Server Error |
| Parameter validation | T3 missing both query and source+id |
| Missing token | T5 called without `--token` |

### S4.3: RFC 7807 Formatting

When the response body deserializes as RFC 7807:
```
[{status}] {title}: {detail}
```

Fields `title` and `detail` are taken as-is from the response. `status` is the integer status code from the RFC 7807 body (not the HTTP status code, though they typically match).

### S4.4: Non-RFC Fallback Formatting

When the response body does NOT match RFC 7807:
```
[{http_status_code}] {body}
```

The body is truncated to **500 Unicode scalar characters**. Truncation appends no suffix (the body itself is the message).

### S4.5: HTTP 202 on Daily Endpoint

HTTP 202 is NOT an error. Return `Ok(CallToolResult { is_error: false })` with an informational message indicating the problem is being fetched and suggesting a retry.

### PBT Properties

```
[INVARIANT] For any HTTP response with status 4xx or 5xx,
  the tool returns Ok(CallToolResult { is_error: true }).
  It never returns Err(McpError) for HTTP error statuses.
[FALSIFICATION] Mock HTTP 404 with RFC 7807 body → assert Ok + is_error.
               Mock HTTP 500 with plain text body → assert Ok + is_error.

[INVARIANT] For any network-level failure (timeout, connection refused, DNS),
  the tool returns Err(McpError).
[FALSIFICATION] Point base-url to unreachable host → assert Err variant.

[INVARIANT] RFC 7807 formatted message matches "[{status}] {title}: {detail}".
[FALSIFICATION] Body: {"status":404,"title":"Not Found","detail":"problem not found"}
               → output: "[404] Not Found: problem not found"

[INVARIANT] Non-RFC fallback body is truncated to at most 500 Unicode scalar chars.
[FALSIFICATION] Serve 1000-char plain text body → assert output body length <= 500.

[INVARIANT] Fallback truncation preserves valid Unicode (never splits a multi-byte char).
[FALSIFICATION] Serve body of 499 ASCII chars + 1 four-byte emoji →
               truncation at char boundary (either 499 or 500 chars).

[INVARIANT] HTTP 202 on daily endpoint produces is_error: false.
[FALSIFICATION] Mock 202 with {"retry_after":30,"status":"fetching"} →
               assert is_error == false and text mentions retry.
```

---

## S5: Response Size Limits

| Limit | Value | Scope |
|---|---|---|
| HTTP body read cap | 1,048,576 bytes (1 MiB) | Raw bytes from `reqwest` response |
| Tool text output cap | 102,400 bytes (100 KiB) | Final `Content::text()` string |
| Non-RFC error body | 500 Unicode scalar chars | Fallback error message body |

### S5.1: HTTP Body Truncation

Use `response.bytes()` with a size check or chunked reading. If body exceeds 1 MiB, stop reading and use what was read.

### S5.2: Tool Output Truncation

After formatting the final Markdown string, if it exceeds 102,400 bytes:
1. Truncate at a valid UTF-8 boundary at or before byte 102,400
2. Append the literal string `"\n\n... (truncated)"`

The suffix is appended **outside** the byte limit (total output may be slightly over 102,400 bytes by the suffix length).

### PBT Properties

```
[INVARIANT] For any API response, the final Content::text() string is at most
  102,400 bytes + len("\n\n... (truncated)").
[FALSIFICATION] Mock a response that produces 200 KiB of Markdown →
               assert output bytes <= 102_400 + 17.

[INVARIANT] Truncation never produces invalid UTF-8.
[FALSIFICATION] Generate a string with multi-byte chars near the 102,400 boundary →
               assert String::from_utf8(output).is_ok().

[INVARIANT] The truncation suffix is exactly "\n\n... (truncated)" (17 chars).
[FALSIFICATION] Trigger truncation → assert output ends with "... (truncated)".
```

---

## S6: Logging

### S6.1: Output Target

All log output goes to **stderr**. No `println!`, `print!`, `dbg!`, or any stdout write outside the MCP transport layer. Stdout is exclusively for MCP JSON-RPC messages.

### S6.2: Configuration

- Crate: `tracing-subscriber` with `env-filter` feature
- Default level: `info`
- Override: `RUST_LOG` environment variable (standard `env_logger` syntax)
- Format: default `tracing_subscriber::fmt` (compact, with timestamps)

### S6.3: Sensitive Data

The `--token` value must never appear in log output at any level. Log the **presence** of a token (`"token: configured"` / `"token: not configured"`), never its value.

### PBT Properties

```
[INVARIANT] No byte is written to stdout except by the MCP transport layer.
[FALSIFICATION] Run server with RUST_LOG=trace, capture stdout separately →
               all stdout bytes are valid JSON-RPC messages.

[INVARIANT] The literal token string never appears in stderr output.
[FALSIFICATION] Set token to a unique UUID, capture stderr at trace level →
               UUID does not appear in stderr.

[INVARIANT] RUST_LOG=off suppresses all log output.
[FALSIFICATION] Set RUST_LOG=off → stderr is empty (modulo transport init).
```
