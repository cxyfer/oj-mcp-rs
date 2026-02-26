use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use rmcp::model::ErrorData;

use crate::error::protocol_error;

pub struct RawResponse {
    pub status: u16,
    pub body: String,
    pub is_json: bool,
}

#[derive(Clone)]
pub struct OjClient {
    http: reqwest::Client,
    base_url: String,
}

impl OjClient {
    pub fn new(base_url: String, token: Option<String>) -> Result<Self, String> {
        let mut builder = reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .use_rustls_tls();

        if let Some(t) = token {
            let mut headers = HeaderMap::new();
            let mut val = HeaderValue::from_str(&format!("Bearer {t}"))
                .map_err(|_| "token contains invalid header characters")?;
            val.set_sensitive(true);
            headers.insert(AUTHORIZATION, val);
            builder = builder.default_headers(headers);
        }

        let http = builder.build().expect("failed to build HTTP client");
        Ok(Self { http, base_url })
    }

    pub async fn get_raw(&self, path: &str) -> Result<RawResponse, ErrorData> {
        let url = format!("{}{path}", self.base_url);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| protocol_error(format!("request failed: {e}")))?;

        let status = resp.status().as_u16();
        let is_json = resp
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|ct| {
                let ct = ct.to_ascii_lowercase();
                ct.starts_with("application/json")
                    || ct.starts_with("application/problem+json")
            });

        const MAX_BODY: usize = 1_048_576;
        let mut buf = Vec::with_capacity(8192);
        let mut stream = resp;
        while let Some(chunk) = stream
            .chunk()
            .await
            .map_err(|e| protocol_error(format!("read body failed: {e}")))?
        {
            let remaining = MAX_BODY.saturating_sub(buf.len());
            if remaining == 0 {
                break;
            }
            let take = chunk.len().min(remaining);
            buf.extend_from_slice(&chunk[..take]);
            if buf.len() >= MAX_BODY {
                break;
            }
        }

        let body = String::from_utf8_lossy(&buf).into_owned();
        Ok(RawResponse { status, body, is_json })
    }
}
