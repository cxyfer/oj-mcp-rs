use rmcp::model::{CallToolResult, Content, ErrorData};
use rmcp::schemars;
use serde::Deserialize;

use crate::client::OjClient;
use crate::convert::{format_problem, truncate_output};
use crate::error::{domain_error, format_api_error, protocol_error};
use crate::models::{DailyFetching, Problem};

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Domain {
    Com,
    Cn,
}

impl Default for Domain {
    fn default() -> Self {
        Self::Com
    }
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Com => write!(f, "com"),
            Self::Cn => write!(f, "cn"),
        }
    }
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

pub async fn run(client: &OjClient, params: DailyParams) -> Result<CallToolResult, ErrorData> {
    let domain = params.domain.unwrap_or_default();

    let date = match params.date {
        Some(d) => {
            if chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").is_err() {
                return Ok(domain_error("invalid date format, expected YYYY-MM-DD"));
            }
            d
        }
        None => chrono::Utc::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string(),
    };

    let path = format!("/api/v1/daily?domain={domain}&date={date}");
    let resp = client.get_raw(&path).await?;

    if resp.status == 202 {
        let msg = if let Ok(fetching) = serde_json::from_str::<DailyFetching>(&resp.body) {
            format!(
                "The daily challenge is currently being fetched. Please retry after {} seconds.",
                fetching.retry_after
            )
        } else {
            "The daily challenge is currently being fetched. Please retry later.".into()
        };
        return Ok(CallToolResult::success(vec![Content::text(msg)]));
    }

    if resp.status != 200 {
        return Ok(domain_error(format_api_error(resp.status, &resp.body)));
    }
    if !resp.is_json {
        return Err(protocol_error("unexpected non-JSON response"));
    }

    let mut problem: Problem = serde_json::from_str(&resp.body)
        .map_err(|e| protocol_error(format!("invalid JSON: {e}")))?;

    if problem.source.is_empty() {
        problem.source = "leetcode".into();
    }

    let md = format_problem(&problem);
    Ok(CallToolResult::success(vec![Content::text(
        truncate_output(md),
    )]))
}
