use rmcp::model::{CallToolResult, Content, ErrorData};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Rfc7807 {
    pub status: Option<u16>,
    pub title: Option<String>,
    pub detail: Option<String>,
}

pub fn format_api_error(status_code: u16, body: &str) -> String {
    if let Ok(rfc) = serde_json::from_str::<Rfc7807>(body)
        && let Some(title) = &rfc.title
    {
        let code = rfc.status.unwrap_or(status_code);
        let detail = rfc.detail.as_deref().unwrap_or_default();
        return format!("[{code}] {title}: {detail}");
    }
    let truncated: String = body.chars().take(500).collect();
    format!("[{status_code}] {truncated}")
}

pub fn domain_error(msg: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(msg.into())])
}

pub fn protocol_error(msg: impl Into<String>) -> ErrorData {
    ErrorData::internal_error(msg.into(), None)
}
