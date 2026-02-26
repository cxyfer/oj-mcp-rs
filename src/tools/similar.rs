use rmcp::model::{CallToolResult, ErrorData};
use rmcp::schemars;
use serde::Deserialize;

use crate::client::OjClient;

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

pub async fn run(_client: &OjClient, _params: SimilarParams) -> Result<CallToolResult, ErrorData> {
    todo!()
}
