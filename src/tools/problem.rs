use rmcp::model::{CallToolResult, Content, ErrorData};
use rmcp::schemars;
use serde::Deserialize;

use crate::client::OjClient;
use crate::convert::{format_problem, truncate_output};
use crate::error::{domain_error, format_api_error, protocol_error};
use crate::models::Problem;

#[derive(Deserialize, schemars::JsonSchema)]
pub struct GetProblemParams {
    #[schemars(description = "Problem source: leetcode, codeforces, atcoder, or luogu")]
    pub source: String,
    #[schemars(description = "Problem ID on the platform. Examples: '1' or 'two-sum' (leetcode), '1A' (codeforces), 'abc001_1' (atcoder), 'P1001' (luogu)")]
    pub id: String,
}

pub async fn run(client: &OjClient, params: GetProblemParams) -> Result<CallToolResult, ErrorData> {
    let source = params.source.trim();
    let id = params.id.trim();
    if source.is_empty() || id.is_empty() {
        return Ok(domain_error("source and id must be non-empty"));
    }

    let encoded_source = urlencoding::encode(source);
    let encoded_id = urlencoding::encode(id);
    let path = format!("/api/v1/problems/{encoded_source}/{encoded_id}");
    let resp = client.get_raw(&path).await?;

    if resp.status != 200 {
        return Ok(domain_error(format_api_error(resp.status, &resp.body)));
    }
    if !resp.is_json {
        return Err(protocol_error("unexpected non-JSON response"));
    }

    let problem: Problem = serde_json::from_str(&resp.body)
        .map_err(|e| protocol_error(format!("invalid JSON: {e}")))?;

    let md = format_problem(&problem);
    Ok(CallToolResult::success(vec![Content::text(
        truncate_output(md),
    )]))
}
