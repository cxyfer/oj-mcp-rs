use rmcp::model::{CallToolResult, Content, ErrorData};
use rmcp::schemars;
use serde::Deserialize;

use crate::client::OjClient;
use crate::convert::{format_similar, truncate_output};
use crate::error::{domain_error, format_api_error, protocol_error};
use crate::models::SimilarResponse;

#[derive(Deserialize, schemars::JsonSchema)]
pub struct SimilarParams {
    #[serde(default)]
    #[schemars(description = "Problem source (required for ID-based search)")]
    pub source: Option<String>,

    #[serde(default)]
    #[schemars(description = "Problem ID (required for ID-based search)")]
    pub id: Option<String>,

    #[serde(default)]
    #[schemars(description = "Text query for semantic search (3-2000 chars, takes priority over source+id)")]
    pub query: Option<String>,

    #[serde(default)]
    #[schemars(description = "Maximum results to return (1-50, default: 10)")]
    pub limit: Option<u32>,

    #[serde(default)]
    #[schemars(description = "Minimum similarity threshold (0.0-1.0, default: 0.0)")]
    pub threshold: Option<f64>,

    #[serde(default)]
    #[schemars(description = "Comma-separated platform filter (e.g. 'leetcode,codeforces')")]
    pub source_filter: Option<String>,
}

pub async fn run(client: &OjClient, params: SimilarParams) -> Result<CallToolResult, ErrorData> {
    let limit = params.limit.unwrap_or(10);
    if !(1..=50).contains(&limit) {
        return Ok(domain_error("limit must be between 1 and 50"));
    }
    let threshold = params.threshold.unwrap_or(0.0);
    if !(0.0..=1.0).contains(&threshold) {
        return Ok(domain_error("threshold must be between 0.0 and 1.0"));
    }

    let mut qs = format!("limit={limit}&threshold={threshold}");
    if let Some(ref sf) = params.source_filter {
        qs.push_str(&format!("&source={}", urlencoding::encode(sf)));
    }

    let trimmed_query = params.query.as_deref().map(str::trim).unwrap_or("");

    let path = if !trimmed_query.is_empty() {
        let len = trimmed_query.chars().count();
        if len < 3 || len > 2000 {
            return Ok(domain_error(
                "query must be between 3 and 2000 characters",
            ));
        }
        format!(
            "/api/v1/similar?q={}&{qs}",
            urlencoding::encode(trimmed_query)
        )
    } else {
        let source = params
            .source
            .as_deref()
            .map(str::trim)
            .unwrap_or("");
        let id = params.id.as_deref().map(str::trim).unwrap_or("");
        if source.is_empty() || id.is_empty() {
            return Ok(domain_error(
                "either 'query' or both 'source' and 'id' must be provided",
            ));
        }
        format!(
            "/api/v1/similar/{}/{}?{qs}",
            urlencoding::encode(source),
            urlencoding::encode(id),
        )
    };

    let resp = client.get_raw(&path).await?;

    if resp.status != 200 {
        return Ok(domain_error(format_api_error(resp.status, &resp.body)));
    }
    if !resp.is_json {
        return Err(protocol_error("unexpected non-JSON response"));
    }

    let parsed: SimilarResponse = serde_json::from_str(&resp.body)
        .map_err(|e| protocol_error(format!("invalid JSON: {e}")))?;

    let md = format_similar(&parsed);
    Ok(CallToolResult::success(vec![Content::text(
        truncate_output(md),
    )]))
}
