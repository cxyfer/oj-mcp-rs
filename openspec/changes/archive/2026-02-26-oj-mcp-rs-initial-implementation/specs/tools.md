# Tools Specification

Covers all 5 MCP tools: parameter schemas, endpoint mapping, validation rules, response formatting, and edge cases.

All tools return `Ok(CallToolResult)` with `Content::text(markdown)`. Error classification follows [core.md S4](core.md#s4-error-classification).

---

## Common Conventions

- All parameter structs derive `Deserialize` + `schemars::JsonSchema`
- Optional parameters use `Option<T>` with `#[serde(default)]`
- Path segments are percent-encoded via `urlencoding::encode()`
- Tool descriptions use `#[schemars(description = "...")]` on each field
- Each tool function lives in its own sub-module under `src/tools/`

---

## T1: `get_daily_challenge`

### Endpoint

`GET {base_url}/api/v1/daily?domain={domain}&date={date}`

### Parameters

| Name | Type | Required | Default | Constraints |
|---|---|---|---|---|
| `domain` | enum: `com`, `cn` | No | `com` | Case-sensitive, exactly `"com"` or `"cn"` |
| `date` | string | No | UTC today | Format: `YYYY-MM-DD` |

### Date Handling

When `date` is `None`, compute: `chrono::Utc::now().date_naive().format("%Y-%m-%d")`.

When `date` is `Some(value)`, validate by parsing with `chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")`. If parsing fails, return `CallToolResult { is_error: true }` with a message indicating the expected format. This prevents query-string injection via malformed date values.

**UTC only.** Never use `Local` or any system timezone. The date reflects the UTC calendar day at the moment of the call.

### HTTP 202 Handling

When the backend returns HTTP 202 with body `{"retry_after": N, "status": "fetching"}`:
- Return `CallToolResult { is_error: false }` (this is NOT an error)
- Text: informational message indicating the problem is being fetched
- Include `retry_after` value in the message to suggest when to retry

### Response Format (HTTP 200)

```markdown
# {title}

- Source: {source} | ID: {id} | Difficulty: {difficulty}
- Tags: {tags_comma_separated}
- Link: {link}
- AC Rate: {ac_rate}%

---

{content_as_markdown}
```

Fields use nullable display rules (see [convert.md S4](convert.md#s4-nullable-field-display)).

### PBT Properties

```
[INVARIANT] When date is None, the query parameter equals
  chrono::Utc::now().date_naive().format("%Y-%m-%d").
  It never uses the local system timezone.
[FALSIFICATION] Set system TZ to UTC+14 at 23:30 UTC →
               date param must reflect UTC date, not local date.

[INVARIANT] domain defaults to "com" when not provided.
[FALSIFICATION] Call with domain=None → outbound URL contains "domain=com".

[INVARIANT] HTTP 202 response produces is_error=false.
[FALSIFICATION] Mock 202 → assert is_error == false.

[INVARIANT] HTTP 200 response produces is_error=false with Markdown matching
  the metadata header template (starts with "# ", contains "Source:", etc.).
[FALSIFICATION] Mock 200 with valid Problem JSON → parse output lines →
               line 0 starts with "# ", line 2 contains "Source:".

[INVARIANT] For any Problem response, the content field is passed through
  html_to_markdown() before inclusion in output.
[FALSIFICATION] Mock Problem with content "<b>bold</b>" →
               output contains "**bold**" or "bold" (not "<b>").
```

---

## T2: `get_problem`

### Endpoint

`GET {base_url}/api/v1/problems/{source}/{id}`

### Parameters

| Name | Type | Required | Constraints |
|---|---|---|---|
| `source` | string | Yes | Non-empty after trim |
| `id` | string | Yes | Non-empty after trim |

### Percent-Encoding

Both `source` and `id` are percent-encoded via `urlencoding::encode()` before URL interpolation. This handles special characters in problem IDs (e.g. Codeforces `"1920/A"` or IDs with unicode).

### Input Validation

Both `source` and `id` must be non-empty after trimming. If either is empty/whitespace-only, return `CallToolResult { is_error: true }` with a descriptive message. This validation is performed at the handler level (not relying on schema-level enforcement).

### Response Format (HTTP 200)

Same metadata header template as T1.

### PBT Properties

```
[INVARIANT] For any (source, id) pair containing URL-unsafe characters,
  the outbound request path is correctly percent-encoded.
[FALSIFICATION] source="a/b", id="c d" →
               request path contains "a%2Fb/c%20d" (or equivalent encoding).

[INVARIANT] The response Markdown starts with "# {title}" where title matches
  the Problem.title field from the API response.
[FALSIFICATION] Mock Problem with title "Two Sum" → output starts with "# Two Sum".

[INVARIANT] Empty source or id after trimming produces is_error=true.
  Validation is performed at the handler level, not relying on schema enforcement.
[FALSIFICATION] Call with source="" → tool returns is_error=true with descriptive message.
               Call with source="  " → tool returns is_error=true.
               Call with id="" → tool returns is_error=true.
```

---

## T3: `find_similar_problems`

### Endpoints

- **Mode A (by ID):** `GET {base_url}/api/v1/similar/{source}/{id}?limit={limit}&threshold={threshold}&source={source_filter}`
- **Mode B (by query):** `GET {base_url}/api/v1/similar?q={query}&limit={limit}&threshold={threshold}&source={source_filter}`

### Parameters

| Name | Type | Required | Default | Constraints |
|---|---|---|---|---|
| `source` | string | Conditional | - | Required for Mode A |
| `id` | string | Conditional | - | Required for Mode A |
| `query` | string | Conditional | - | Required for Mode B; 3..=2000 Unicode scalar chars |
| `limit` | integer | No | 10 | Range: 1..=50 |
| `threshold` | float | No | 0.0 | Range: 0.0..=1.0 |
| `source_filter` | string | No | - | Comma-separated platform filter |

### Validation Rules (Strict Order)

1. **Clamp/validate numeric params first:**
   - `limit`: if provided, must be in range 1..=50. If out of range, return `CallToolResult { is_error: true }`.
   - `threshold`: if provided, must be in range 0.0..=1.0. If out of range, return `CallToolResult { is_error: true }`.

2. **Trim** `query`. If the trimmed value is non-empty, select **Mode B**.
   - Ignore `source` and `id` entirely (even if provided).
   - Validate trimmed query length: 3..=2000 Unicode scalar chars.
   - If length is outside range, return `CallToolResult { is_error: true }` with a descriptive message. Do **NOT** fall back to Mode A.

2. If `query` is absent or empty after trimming, select **Mode A**.
   - Both `source` and `id` must be present and non-empty after trimming.
   - If either is missing/empty, return `CallToolResult { is_error: true }`.

3. Path segments in Mode A are percent-encoded.

4. `source_filter` must be percent-encoded via `urlencoding::encode()` when building query parameters.

### Response Format

```markdown
# Similar Problems

Query: {rewritten_query}

| # | Source | ID | Title | Difficulty | Similarity | Link |
|---|--------|----|-------|------------|------------|------|
| 1 | leetcode | 167 | Two Sum II | Medium | 79.0% | https://... |
```

- `similarity` is displayed as percentage with one decimal: `{:.1}%` (value * 100)
- Row numbering starts at 1
- `rewritten_query` comes from the API response field

### PBT Properties

```
[INVARIANT] If query (after trim) is non-empty, Mode B is selected regardless of
  whether source and id are also provided.
[FALSIFICATION] Provide query="arrays", source="leetcode", id="1" →
               outbound request hits /api/v1/similar?q=arrays... (Mode B).

[INVARIANT] If trimmed query length < 3, the tool returns is_error=true
  without making an HTTP request.
[FALSIFICATION] query="ab" → immediate error, no outbound request.

[INVARIANT] If trimmed query length > 2000, the tool returns is_error=true
  without making an HTTP request.
[FALSIFICATION] query = "x".repeat(2001) → immediate error.

[INVARIANT] If trimmed query length is exactly 3 or exactly 2000,
  the tool proceeds with Mode B.
[FALSIFICATION] query = "abc" (len 3) → HTTP request made.
               query = "x".repeat(2000) → HTTP request made.

[INVARIANT] If query is empty/absent and source or id is missing/empty,
  the tool returns is_error=true.
[FALSIFICATION] query=None, source="leetcode", id="" → error.
               query=None, source="", id="1" → error.
               query=None, source=None, id=None → error.

[INVARIANT] When query is invalid (too short/long), the tool never falls back
  to Mode A even if source and id are valid.
[FALSIFICATION] query="ab", source="leetcode", id="1" → error (not Mode A).

[INVARIANT] similarity values are formatted as "{:.1}%" (e.g. 0.79 → "79.0%").
[FALSIFICATION] Mock result with similarity=0.791 → output shows "79.1%".
               Mock result with similarity=1.0 → output shows "100.0%".

[INVARIANT] Mode A path segments are percent-encoded.
[FALSIFICATION] source="a b", id="c/d" → path contains "a%20b/c%2Fd".
```

---

## T4: `resolve_problem`

### Endpoint

`GET {base_url}/api/v1/resolve/{query}`

### Parameters

| Name | Type | Required | Constraints |
|---|---|---|---|
| `query` | string | Yes | Non-empty; URL, slug, prefixed ID, or bare pattern |

### Percent-Encoding

The `query` value is percent-encoded via `urlencoding::encode()` before path interpolation. This correctly handles URLs containing `/`, `?`, `#`, etc.

### Response Format (HTTP 200)

Same metadata header template as T1/T2, using the `problem` field from the resolve response.

### PBT Properties

```
[INVARIANT] For any query string, the outbound request path-encodes the value.
[FALSIFICATION] query="https://leetcode.com/problems/two-sum/" →
               path segment is fully percent-encoded.

[INVARIANT] The response uses the nested problem object for formatting,
  not the top-level source/id fields.
[FALSIFICATION] Mock resolve response where top-level source differs from
               problem.source → output uses problem.source.

[INVARIANT] For any valid resolve response, the output matches the metadata
  header template.
[FALSIFICATION] Mock valid response → verify "# {title}" header present.
```

---

## T5: `get_platform_status`

### Endpoint

`GET {base_url}/status`

Note: This is at the **root** path, not under `/api/v1/`.

### Parameters

None.

### Authentication

This endpoint **requires** the Bearer token. If `--token` was not provided at startup, the tool must return `CallToolResult { is_error: true }` with a message indicating authentication is required. This check happens **before** making the HTTP request.

### Response Format

```markdown
# OJ Platform Status (v{version})

| Platform | Problems | Missing Content | Not Embedded |
|----------|----------|-----------------|--------------|
| atcoder  | 8,356    | 320             | 339          |
```

- `version` from the status response `version` field
- `total`, `missing_content`, `not_embedded` are formatted with ASCII comma grouping (see [convert.md S3](convert.md#s3-number-formatting))
- Platforms are listed in the order returned by the API

### PBT Properties

```
[INVARIANT] When no token is configured, the tool returns is_error=true
  without making an HTTP request.
[FALSIFICATION] Construct OjClient with token=None → call get_platform_status →
               assert is_error==true, assert no outbound HTTP request.

[INVARIANT] The output header matches "# OJ Platform Status (v{version})"
  where {version} is the API response version field.
[FALSIFICATION] Mock status response with version="1.2.3" →
               output starts with "# OJ Platform Status (v1.2.3)".

[INVARIANT] Numeric fields use comma-separated thousands grouping.
[FALSIFICATION] Mock total=12984 → table cell contains "12,984".
               Mock total=100 → table cell contains "100" (no comma).
               Mock total=1000 → table cell contains "1,000".

[INVARIANT] The Markdown table has exactly 4 columns:
  Platform, Problems, Missing Content, Not Embedded.
[FALSIFICATION] Parse output as Markdown table → assert 4 header columns.

[INVARIANT] Row count matches the platforms array length from the API response.
[FALSIFICATION] Mock 3 platforms → table has 3 data rows.
```
