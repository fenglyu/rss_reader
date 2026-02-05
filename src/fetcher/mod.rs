pub mod http_fetcher;
pub mod parallel;

use async_trait::async_trait;

use crate::app::Result;

#[derive(Debug)]
pub enum FetchResult {
    /// New content fetched successfully
    Content {
        body: Vec<u8>,
        etag: Option<String>,
        last_modified: Option<String>,
    },
    /// Content not modified (HTTP 304)
    NotModified,
}

#[async_trait]
pub trait Fetcher {
    async fn fetch(
        &self,
        url: &str,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<FetchResult>;
}
