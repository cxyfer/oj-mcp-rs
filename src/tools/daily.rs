use rmcp::model::{CallToolResult, ErrorData};
use rmcp::schemars;
use serde::Deserialize;

use crate::client::OjClient;

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Domain {
    Com,
    Cn,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct DailyParams {
    #[serde(default)]
    #[schemars(description = "LeetCode domain: 'com' (default) or 'cn'")]
    pub domain: Option<Domain>,

    #[serde(default)]
    #[schemars(description = "Date in YYYY-MM-DD format (default: today UTC)")]
    pub date: Option<String>,
}

pub async fn run(_client: &OjClient, _params: DailyParams) -> Result<CallToolResult, ErrorData> {
    todo!()
}
