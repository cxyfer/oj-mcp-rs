use rmcp::model::{CallToolResult, ErrorData};
use rmcp::schemars;
use serde::Deserialize;

use crate::client::OjClient;
use crate::convert::{format_problem, truncate_output};
use crate::error::{domain_error, format_api_error, protocol_error};
use crate::models::ResolveResponse;

#[derive(Deserialize, schemars::JsonSchema)]
pub struct ResolveParams {
    #[schemars(description = "URL, slug, prefixed ID, or bare pattern to resolve")]
    pub query: String,
}

pub async fn run(client: &OjClient, params: ResolveParams) -> Result<CallToolResult, ErrorData> {
    let encoded = urlencoding::encode(&params.query);
    let path = format!("/api/v1/resolve/{encoded}");
    let resp = client.get_raw(&path).await?;

    if resp.status != 200 {
        return Ok(domain_error(format_api_error(resp.status, &resp.body)));
    }
    if !resp.is_json {
        return Err(protocol_error("unexpected non-JSON response"));
    }

    let parsed: ResolveResponse = serde_json::from_str(&resp.body)
        .map_err(|e| protocol_error(format!("invalid JSON: {e}")))?;

    let md = format_problem(&parsed.problem);
    Ok(CallToolResult::success(vec![
        rmcp::model::Content::text(truncate_output(md)),
    ]))
}
