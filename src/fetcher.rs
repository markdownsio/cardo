use crate::dependency::DependencySource;
use crate::github::GitHubClient;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct Fetcher {
    client: GitHubClient,
    output_dir: String,
}

#[derive(Debug)]
pub struct FetchResult {
    pub name: String,
    pub path: String,
    pub success: bool,
    pub error: Option<String>,
}

impl Fetcher {
    pub fn new(output_dir: String, github_token: Option<String>) -> Self {
        Self {
            client: GitHubClient::new(github_token),
            output_dir,
        }
    }

    pub async fn fetch_all(
        &self,
        dependencies: &HashMap<String, DependencySource>,
        force: bool,
    ) -> Result<Vec<FetchResult>> {
        let pb = ProgressBar::new(dependencies.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );

        let mut results = Vec::new();

        for (name, source) in dependencies {
            pb.set_message(format!("Downloading {}...", name));
            let result = self.fetch_one(name, source, force).await;
            results.push(result);
            pb.inc(1);
        }

        pb.finish_with_message("Done!");
        Ok(results)
    }

    async fn fetch_one(
        &self,
        name: &str,
        source: &DependencySource,
        force: bool,
    ) -> FetchResult {
        let output_path = format!("{}/{}", self.output_dir, source.output_path(name));

        // 检查文件是否已存在
        if !force && Path::new(&output_path).exists() {
            return FetchResult {
                name: name.to_string(),
                path: output_path,
                success: true,
                error: None,
            };
        }

        // 创建目录
        if let Some(parent) = Path::new(&output_path).parent() {
            if let Err(e) = fs::create_dir_all(parent).await {
                return FetchResult {
                    name: name.to_string(),
                    path: output_path,
                    success: false,
                    error: Some(format!("Failed to create directory: {}", e)),
                };
            }
        }

        // 下载文件
        match self.client.fetch_file_with_retry(source, 3).await {
            Ok(content) => {
                match fs::File::create(&output_path).await {
                    Ok(mut file) => {
                        if let Err(e) = file.write_all(content.as_bytes()).await {
                            FetchResult {
                                name: name.to_string(),
                                path: output_path,
                                success: false,
                                error: Some(format!("Failed to write file: {}", e)),
                            }
                        } else {
                            FetchResult {
                                name: name.to_string(),
                                path: output_path,
                                success: true,
                                error: None,
                            }
                        }
                    }
                    Err(e) => FetchResult {
                        name: name.to_string(),
                        path: output_path,
                        success: false,
                        error: Some(format!("Failed to create file: {}", e)),
                    },
                }
            }
            Err(e) => FetchResult {
                name: name.to_string(),
                path: output_path,
                success: false,
                error: Some(format!("{}", e)),
            },
        }
    }

    pub async fn clean(&self) -> Result<()> {
        if Path::new(&self.output_dir).exists() {
            fs::remove_dir_all(&self.output_dir).await?;
            fs::create_dir_all(&self.output_dir).await?;
        }
        Ok(())
    }
}
