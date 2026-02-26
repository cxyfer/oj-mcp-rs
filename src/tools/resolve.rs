use rmcp::model::{CallToolResult, ErrorData};
use rmcp::schemars;
use serde::Deserialize;

use crate::client::OjClient;

#[derive(Deserialize, schemars::JsonSchema)]
pub struct ResolveParams {
    #[schemars(description = "URL, slug, prefixed ID, or bare pattern to resolve")]
    pub query: String,
}

pub async fn run(_client: &OjClient, _params: ResolveParams) -> Result<CallToolResult, ErrorData> {
    todo!()
}
