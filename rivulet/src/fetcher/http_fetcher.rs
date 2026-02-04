use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, IF_MODIFIED_SINCE, IF_NONE_MATCH};
use reqwest::{Client, StatusCode};

use crate::app::Result;
use crate::fetcher::{FetchResult, Fetcher};

pub struct HttpFetcher {
    client: Client,
}

impl HttpFetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .gzip(true)
            .brotli(true)
            .user_agent("rivulet/0.1.0")
            .build()
            .expect("Failed to build HTTP client");

        Self { client }
    }
}

impl Default for HttpFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Fetcher for HttpFetcher {
    async fn fetch(
        &self,
        url: &str,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<FetchResult> {
        let mut headers = HeaderMap::new();

        if let Some(etag) = etag {
            if let Ok(value) = HeaderValue::from_str(etag) {
                headers.insert(IF_NONE_MATCH, value);
            }
        }

        if let Some(last_modified) = last_modified {
            if let Ok(value) = HeaderValue::from_str(last_modified) {
                headers.insert(IF_MODIFIED_SINCE, value);
            }
        }

        let response = self.client.get(url).headers(headers).send().await?;

        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(FetchResult::NotModified);
        }

        response.error_for_status_ref()?;

        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let last_modified = response
            .headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let body = response.bytes().await?.to_vec();

        Ok(FetchResult::Content {
            body,
            etag,
            last_modified,
        })
    }
}
