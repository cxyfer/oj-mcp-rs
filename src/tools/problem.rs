use rmcp::model::{CallToolResult, ErrorData};
use rmcp::schemars;
use serde::Deserialize;

use crate::client::OjClient;

#[derive(Deserialize, schemars::JsonSchema)]
pub struct GetProblemParams {
    #[schemars(description = "Problem source (e.g. leetcode, codeforces, atcoder)")]
    pub source: String,
    #[schemars(description = "Problem ID")]
    pub id: String,
}

pub async fn run(_client: &OjClient, _params: GetProblemParams) -> Result<CallToolResult, ErrorData> {
    todo!()
}
