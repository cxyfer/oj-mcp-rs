# Conversion Specification

Covers HTML-to-Markdown conversion, metadata header formatting, number formatting, nullable field display, and output truncation.

---

## S1: HTML-to-Markdown Pipeline

### S1.1: Primary Path — `htmd::convert`

Use `htmd::convert(html)` wrapped in `std::panic::catch_unwind` to guard against panics on malformed HTML.

### S1.2: Fallback Path — `ammonia::clean_text`

If `htmd::convert` panics or returns `Err`, fall back to `ammonia::clean_text(html)` which strips all HTML tags and returns plain text.

### S1.3: Empty Input

If the input HTML is empty (after trimming), return the literal string `"No description available."`.

### S1.4: Decision Flow

```
html_to_markdown(html):
  if html.trim().is_empty() → "No description available."
  catch_unwind(|| htmd::convert(html)):
    Ok(Ok(md)) if !md.trim().is_empty() → md
    _ → ammonia::clean_text(html)
```

### PBT Properties

```
[INVARIANT] For any input string html, html_to_markdown(html) returns valid UTF-8
  and never panics.
[FALSIFICATION] Fuzz with arbitrary byte sequences decoded as UTF-8 →
               function returns without panic.

[INVARIANT] Empty or whitespace-only input always returns "No description available."
[FALSIFICATION] html="" → "No description available."
               html="   " → "No description available."
               html="\n\t" → "No description available."

[INVARIANT] If htmd::convert succeeds and produces non-empty output,
  that output is returned (no fallback).
[FALSIFICATION] html="<b>bold</b>" → output contains "**bold**".

[INVARIANT] If htmd::convert panics, the function still returns a string
  (the ammonia fallback), never propagates the panic.
[FALSIFICATION] Mock htmd::convert to panic → function returns ammonia output.

[INVARIANT] The ammonia fallback produces output with zero HTML tags.
[FALSIFICATION] html="<script>alert(1)</script><p>text</p>" →
               fallback output contains no '<' or '>' characters.
```

---

## S2: Metadata Header Template

### S2.1: Problem Header (T1, T2, T4)

```
# {title}

- Source: {source} | ID: {id} | Difficulty: {difficulty}
- Tags: {tags}
- Link: {link}
- AC Rate: {ac_rate}

---

{content}
```

Field formatting:
- `title`: as-is from API
- `source`: as-is
- `id`: as-is
- `difficulty`: value or `"N/A"`
- `tags`: comma + space separated list, or `"N/A"` if null/empty
- `link`: as-is, or `"N/A"`
- `ac_rate`: `format!("{:.1}%", value)` or `"N/A"`
- `content`: output of `html_to_markdown(problem.content)`

### S2.2: Similar Problems Table (T3)

```
# Similar Problems

Query: {rewritten_query}

| # | Source | ID | Title | Difficulty | Similarity | Link |
|---|--------|----|-------|------------|------------|------|
| {i} | {source} | {id} | {title} | {difficulty} | {similarity}% | {link} |
```

Field formatting:
- `i`: 1-based row index
- `similarity`: `format!("{:.1}", value * 100.0)` + `%`
- `difficulty`: value or `"N/A"`

### S2.3: Platform Status Table (T5)

```
# OJ Platform Status (v{version})

| Platform | Problems | Missing Content | Not Embedded |
|----------|----------|-----------------|--------------|
| {source} | {total} | {missing_content} | {not_embedded} |
```

Numeric fields use comma grouping (S3).

### PBT Properties

```
[INVARIANT] The problem header always starts with "# " followed by the title.
[FALSIFICATION] Any Problem input → first line matches regex "^# .+".

[INVARIANT] The metadata line contains exactly the fields:
  Source, ID, Difficulty separated by " | ".
[FALSIFICATION] Parse second non-empty line → contains "Source:" and "ID:" and "Difficulty:".

[INVARIANT] Tags line shows comma-separated values or "N/A".
[FALSIFICATION] tags=Some(vec!["A","B"]) → "Tags: A, B".
               tags=None → "Tags: N/A".
               tags=Some(vec![]) → "Tags: N/A".

[INVARIANT] "---" separator appears between metadata and content.
[FALSIFICATION] Any Problem → output contains "\n---\n".
```

---

## S3: Number Formatting

### S3.1: Comma Grouping

Format non-negative integers with ASCII comma separator every 3 digits from the right. This is a pure function with no OS locale dependency.

```
format_number(0)      → "0"
format_number(999)    → "999"
format_number(1000)   → "1,000"
format_number(12984)  → "12,984"
format_number(1000000)→ "1,000,000"
```

### S3.2: Applicable Fields

- `PlatformStatus.total`
- `PlatformStatus.missing_content`
- `PlatformStatus.not_embedded`

### PBT Properties

```
[INVARIANT] For any non-negative integer n, format_number(n):
  - Contains only ASCII digits and commas
  - Starts with a digit (never a comma)
  - Ends with a digit (never a comma)
  - Has commas only at positions separating groups of exactly 3 digits from the right
  - Parsing by removing commas yields the original number
[FALSIFICATION] Exhaustive for 0..=9999:
               format_number(0) == "0"
               format_number(1) == "1"
               format_number(10) == "10"
               format_number(100) == "100"
               format_number(999) == "999"
               format_number(1000) == "1,000"
               format_number(9999) == "9,999"

[INVARIANT] The function is locale-independent: output is identical regardless of
  LC_ALL, LC_NUMERIC, or LANG environment variables.
[FALSIFICATION] Set LC_ALL=de_DE.UTF-8 → format_number(1000) == "1,000" (not "1.000").

[INVARIANT] For any n, format_number(n).replace(",", "").parse::<u64>() == Ok(n).
[FALSIFICATION] Property-based: generate random u64 → round-trip succeeds.
```

---

## S4: Nullable Field Display

### S4.1: Rule

Any `Option<T>` field from API responses displays as:
- `Some(value)` → formatted value (type-specific formatting below)
- `None` → literal string `"N/A"`

### S4.2: Type-Specific Formatting

| Field | Type | Format when Some |
|---|---|---|
| `difficulty` | `Option<String>` | As-is |
| `ac_rate` | `Option<f64>` | `format!("{:.1}%", value)` |
| `rating` | `Option<f64>` | `format!("{:.1}", value)` |
| `tags` | `Option<Vec<String>>` | `join(", ")` ; empty vec → `"N/A"` |
| `link` | `Option<String>` | As-is |

### PBT Properties

```
[INVARIANT] For any Option<T> field, None always displays as exactly "N/A".
[FALSIFICATION] difficulty=None → "N/A".
               ac_rate=None → "N/A".
               tags=None → "N/A".

[INVARIANT] ac_rate formatting always has exactly one decimal place.
[FALSIFICATION] ac_rate=Some(57.0) → "57.0%".
               ac_rate=Some(0.1) → "0.1%".
               ac_rate=Some(100.0) → "100.0%".

[INVARIANT] An empty tags vector is treated identically to None.
[FALSIFICATION] tags=Some(vec![]) → "N/A".
```

---

## S5: Output Truncation

### S5.1: Strategy

After assembling the complete Markdown output string:

1. Encode as UTF-8 bytes
2. If byte length <= 102,400 → return as-is
3. If byte length > 102,400 → find the largest valid UTF-8 boundary at or before byte 102,400, truncate there, append `"\n\n... (truncated)"`

### S5.2: UTF-8 Safety

Truncation must never split a multi-byte UTF-8 sequence. The implementation must scan backward from the byte boundary to find a valid char boundary.

### S5.3: Suffix

The truncation suffix is exactly: `"\n\n... (truncated)"` (17 bytes). This is appended **after** the truncated content, so the total output may be up to 102,417 bytes.

### PBT Properties

```
[INVARIANT] For any input string, the truncated output is valid UTF-8.
[FALSIFICATION] Generate strings with multi-byte chars (CJK, emoji) at various
               positions near the 102,400 boundary → output is valid UTF-8.

[INVARIANT] If input byte length <= 102,400, output equals input exactly
  (no truncation applied).
[FALSIFICATION] 102,400-byte ASCII string → output == input.

[INVARIANT] If truncation occurs, output ends with "... (truncated)".
[FALSIFICATION] 200,000-byte string → output.ends_with("... (truncated)").

[INVARIANT] The content portion (before suffix) is at most 102,400 bytes.
[FALSIFICATION] Truncated output minus suffix → byte length <= 102,400.

[INVARIANT] Truncation is idempotent: truncating an already-truncated string
  that is within limits produces the same string.
[FALSIFICATION] Let s = truncate(large_input); truncate(s) == s.
```
