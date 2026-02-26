use serde::Deserialize;

#[derive(Deserialize)]
pub struct Problem {
    pub id: String,
    pub source: String,
    #[serde(default)]
    pub slug: Option<String>,
    pub title: String,
    #[serde(default)]
    pub difficulty: Option<String>,
    #[serde(default)]
    pub ac_rate: Option<f64>,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Deserialize)]
pub struct DailyFetching {
    pub retry_after: u64,
}

#[derive(Deserialize)]
pub struct SimilarResponse {
    pub rewritten_query: String,
    pub results: Vec<SimilarResult>,
}

#[derive(Deserialize)]
pub struct SimilarResult {
    pub source: String,
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub difficulty: Option<String>,
    #[serde(default)]
    pub link: Option<String>,
    pub similarity: f64,
}

#[derive(Deserialize)]
pub struct ResolveResponse {
    pub problem: Problem,
}

#[derive(Deserialize)]
pub struct StatusResponse {
    pub version: String,
    pub platforms: Vec<PlatformStatus>,
}

#[derive(Deserialize)]
pub struct PlatformStatus {
    pub source: String,
    pub total: u64,
    pub missing_content: u64,
    pub not_embedded: u64,
}
