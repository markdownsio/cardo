use crate::dependency::DependencySource;
use anyhow::Result;
use reqwest::Client;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

pub struct GitHubClient {
    client: Client,
    token: Option<String>,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("cardo/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client, token }
    }

    pub async fn fetch_file(&self, source: &DependencySource) -> Result<String, GitHubError> {
        let url = source.to_raw_url();

        let mut request = self.client.get(&url);

        // 如果提供了 token，添加到请求头
        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let content = response.text().await?;
            Ok(content)
        } else if response.status() == reqwest::StatusCode::NOT_FOUND {
            Err(GitHubError::NotFound(url))
        } else {
            Err(GitHubError::NetworkError(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )))
        }
    }

    pub async fn fetch_file_with_retry(
        &self,
        source: &DependencySource,
        max_retries: u32,
    ) -> Result<String, GitHubError> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match self.fetch_file(source).await {
                Ok(content) => return Ok(content),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        // 指数退避：等待 2^attempt 秒
                        let delay = 1u64 << attempt;
                        tokio::time::sleep(Duration::from_secs(delay)).await;
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            GitHubError::NetworkError("Unknown error after retries".to_string())
        }))
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new(None)
    }
}
