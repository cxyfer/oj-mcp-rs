use crate::models::{Problem, SimilarResponse, StatusResponse};

fn looks_like_html(s: &str) -> bool {
    let trimmed = s.trim();
    if !trimmed.contains('<') {
        return false;
    }
    const TAGS: &[&str] = &[
        "<p>", "<p ", "<div", "<ul", "<ol", "<li", "<table", "<br", "<h1", "<h2",
        "<h3", "<h4", "<h5", "<h6", "<pre>", "<pre ", "<code>", "<code ",
        "<strong", "<em>", "<em ", "<span", "<img", "<a ",
    ];
    let lower = trimmed.to_ascii_lowercase();
    TAGS.iter().any(|tag| lower.contains(tag))
}

pub fn html_to_markdown(content: &str) -> String {
    if content.trim().is_empty() {
        return "No description available.".into();
    }
    if !looks_like_html(content) {
        return content.to_owned();
    }
    match std::panic::catch_unwind(|| htmd::convert(content)) {
        Ok(Ok(md)) if !md.trim().is_empty() => md,
        _ => ammonia::clean_text(content),
    }
}

pub fn format_problem(p: &Problem) -> String {
    let difficulty = p.difficulty.as_deref().unwrap_or("N/A");
    let tags = match &p.tags {
        Some(v) if !v.is_empty() => v.join(", "),
        _ => "N/A".into(),
    };
    let link = p.link.as_deref().unwrap_or("N/A");
    let ac_rate = p
        .ac_rate
        .map(|v| format!("{v:.1}%"))
        .unwrap_or_else(|| "N/A".into());
    let content = html_to_markdown(p.content.as_deref().unwrap_or(""));

    format!(
        "\
# {title}

- Source: {source} | ID: {id} | Difficulty: {difficulty}
- Tags: {tags}
- Link: {link}
- AC Rate: {ac_rate}

---

{content}",
        title = p.title,
        source = p.source,
        id = p.id,
    )
}

pub fn format_similar(resp: &SimilarResponse) -> String {
    let mut out = format!(
        "\
# Similar Problems

Query: {}

| # | Source | ID | Title | Difficulty | Similarity | Link |
|---|--------|----|-------|------------|------------|------|
",
        resp.rewritten_query,
    );

    for (i, r) in resp.results.iter().enumerate() {
        let difficulty = r.difficulty.as_deref().unwrap_or("N/A");
        let link = r.link.as_deref().unwrap_or("N/A");
        let similarity = format!("{:.1}%", r.similarity * 100.0);
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            i + 1,
            r.source,
            r.id,
            r.title,
            difficulty,
            similarity,
            link,
        ));
    }

    out
}

pub fn format_status(resp: &StatusResponse) -> String {
    let mut out = format!(
        "\
# OJ Platform Status (v{version})

| Platform | Problems | Missing Content | Not Embedded |
|----------|----------|-----------------|--------------|
",
        version = resp.version,
    );

    for p in &resp.platforms {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            p.source,
            format_number(p.total),
            format_number(p.missing_content),
            format_number(p.not_embedded),
        ));
    }

    out
}

pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

pub fn truncate_output(s: String) -> String {
    if s.len() <= 102_400 {
        return s;
    }
    let boundary = s.floor_char_boundary(102_400);
    let mut truncated = s[..boundary].to_owned();
    truncated.push_str("\n\n... (truncated)");
    truncated
}
