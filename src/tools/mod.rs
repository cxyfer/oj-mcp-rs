mod daily;
mod problem;
mod resolve;
mod similar;
mod status;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};

use crate::client::OjClient;

#[derive(Clone)]
pub struct OjServer {
    client: OjClient,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl OjServer {
    pub fn new(client: OjClient) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Auto-detect and resolve a problem from URL, slug, or pattern")]
    async fn resolve_problem(
        &self,
        params: Parameters<resolve::ResolveParams>,
    ) -> Result<CallToolResult, ErrorData> {
        resolve::run(&self.client, params.0).await
    }

    #[tool(description = "Get the support status of each platform on the backend")]
    async fn get_platform_status(&self) -> Result<CallToolResult, ErrorData> {
        status::run(&self.client).await
    }

    #[tool(description = "Get a specific problem by source and ID")]
    async fn get_problem(
        &self,
        params: Parameters<problem::GetProblemParams>,
    ) -> Result<CallToolResult, ErrorData> {
        problem::run(&self.client, params.0).await
    }

    #[tool(description = "Get LeetCode daily challenge problem")]
    async fn get_daily_challenge(
        &self,
        params: Parameters<daily::DailyParams>,
    ) -> Result<CallToolResult, ErrorData> {
        daily::run(&self.client, params.0).await
    }

    #[tool(description = "Find similar problems by problem ID or text query")]
    async fn find_similar_problems(
        &self,
        params: Parameters<similar::SimilarParams>,
    ) -> Result<CallToolResult, ErrorData> {
        similar::run(&self.client, params.0).await
    }
}

#[tool_handler]
impl ServerHandler for OjServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability { list_changed: Some(false) }),
                ..Default::default()
            },
            server_info: Implementation {
                name: "oj-mcp-rs".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
