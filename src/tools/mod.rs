mod daily;
mod problem;
mod resolve;
mod similar;
mod status;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{ServerHandler, tool, tool_handler, tool_router};

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

    #[tool(
        description = "Preferred way to look up a problem. Accepts a URL, problem slug, or prefixed ID and returns the full problem (title, difficulty, tags, and description). Supports LeetCode, Codeforces, AtCoder, and Luogu. Use this when the input format is uncertain; use get_problem when source and ID are already known."
    )]
    async fn resolve_problem(
        &self,
        params: Parameters<resolve::ResolveParams>,
    ) -> Result<CallToolResult, ErrorData> {
        resolve::run(&self.client, params.0).await
    }

    #[tool(
        description = "Get problem counts and indexing coverage for each platform (LeetCode, Codeforces, AtCoder, Luogu). Returns total problems, missing content count, and un-embedded count per platform."
    )]
    async fn get_platform_status(&self) -> Result<CallToolResult, ErrorData> {
        status::run(&self.client).await
    }

    #[tool(
        description = "Get a specific problem by source and ID. Returns the full problem including title, difficulty, tags, and description. Supports LeetCode, Codeforces, AtCoder, and Luogu. Use resolve_problem instead when the input is a URL or the ID format is uncertain."
    )]
    async fn get_problem(
        &self,
        params: Parameters<problem::GetProblemParams>,
    ) -> Result<CallToolResult, ErrorData> {
        problem::run(&self.client, params.0).await
    }

    #[tool(
        description = "Get the LeetCode daily challenge problem. Returns the full problem including title, difficulty, tags, and description. Defaults to today (UTC) on leetcode.com; optionally specify a date or the 'cn' domain for leetcode.cn."
    )]
    async fn get_daily_challenge(
        &self,
        params: Parameters<daily::DailyParams>,
    ) -> Result<CallToolResult, ErrorData> {
        daily::run(&self.client, params.0).await
    }

    #[tool(
        description = "Find similar problems by problem ID or free-text query across LeetCode, Codeforces, AtCoder, and Luogu. Returns a ranked list with similarity scores. Provide either a text query, or a source + ID pair."
    )]
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
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
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
