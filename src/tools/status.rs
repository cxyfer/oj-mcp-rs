use rmcp::model::{CallToolResult, Content, ErrorData};

use crate::client::OjClient;
use crate::convert::{format_status, truncate_output};
use crate::error::{domain_error, format_api_error, protocol_error};
use crate::models::StatusResponse;

pub async fn run(client: &OjClient) -> Result<CallToolResult, ErrorData> {
    let resp = client.get_raw("/status").await?;

    if resp.status != 200 {
        return Ok(domain_error(format_api_error(resp.status, &resp.body)));
    }
    if !resp.is_json {
        return Err(protocol_error("unexpected non-JSON response"));
    }

    let parsed: StatusResponse = serde_json::from_str(&resp.body)
        .map_err(|e| protocol_error(format!("invalid JSON: {e}")))?;

    let md = format_status(&parsed);
    Ok(CallToolResult::success(vec![Content::text(
        truncate_output(md),
    )]))
}
