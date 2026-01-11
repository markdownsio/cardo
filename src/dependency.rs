use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencySource {
    GitHub {
        owner: String,
        repo: String,
        path: String,
        version: Option<Version>,
    },
    Url(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Version {
    Tag(String),
    Branch(String),
    Commit(String),
}

#[derive(Debug, Error)]
pub enum DependencyError {
    #[error("Invalid dependency format: {0}")]
    InvalidFormat(String),
    #[error("Invalid GitHub URL: {0}")]
    InvalidGitHubUrl(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

impl DependencySource {
    pub fn parse(value: &toml::Value) -> Result<Self, DependencyError> {
        match value {
            toml::Value::String(s) => {
                // 尝试解析 github: 格式
                if let Some(github_str) = s.strip_prefix("github:") {
                    Self::parse_github_simple(github_str)
                }
                // 尝试解析 URL 格式
                else if s.starts_with("http://") || s.starts_with("https://") {
                    Ok(DependencySource::Url(s.clone()))
                } else {
                    Err(DependencyError::InvalidFormat(format!(
                        "Unknown dependency format: {}",
                        s
                    )))
                }
            }
            toml::Value::Table(table) => {
                // 解析完整格式：{ git = "github:...", tag/branch/rev = "..." }
                if let Some(git_value) = table.get("git") {
                    if let Some(git_str) = git_value.as_str() {
                        // 去除 "github:" 前缀（如果存在）
                        let github_path = git_str.strip_prefix("github:").unwrap_or(git_str);
                        let (owner, repo, path) = Self::parse_github_path(github_path)?;
                        
                        let version = if let Some(tag) = table.get("tag").and_then(|v| v.as_str()) {
                            Some(Version::Tag(tag.to_string()))
                        } else if let Some(branch) = table.get("branch").and_then(|v| v.as_str()) {
                            Some(Version::Branch(branch.to_string()))
                        } else if let Some(rev) = table.get("rev").and_then(|v| v.as_str()) {
                            Some(Version::Commit(rev.to_string()))
                        } else {
                            None
                        };

                        Ok(DependencySource::GitHub {
                            owner,
                            repo,
                            path,
                            version,
                        })
                    } else {
                        Err(DependencyError::InvalidFormat(
                            "git field must be a string".to_string(),
                        ))
                    }
                } else {
                    Err(DependencyError::MissingField("git".to_string()))
                }
            }
            _ => Err(DependencyError::InvalidFormat(
                "Dependency must be a string or table".to_string(),
            )),
        }
    }

    fn parse_github_simple(s: &str) -> Result<Self, DependencyError> {
        let (owner, repo, path) = Self::parse_github_path(s)?;
        Ok(DependencySource::GitHub {
            owner,
            repo,
            path,
            version: None, // 默认使用 main/master 分支
        })
    }

    fn parse_github_path(s: &str) -> Result<(String, String, String), DependencyError> {
        // 格式: owner/repo/path/to/file.md
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() < 3 {
            return Err(DependencyError::InvalidGitHubUrl(format!(
                "Expected format: owner/repo/path/to/file.md, got: {}",
                s
            )));
        }

        let owner = parts[0].to_string();
        let repo = parts[1].to_string();
        let path = parts[2..].join("/");

        Ok((owner, repo, path))
    }

    pub fn to_raw_url(&self) -> String {
        match self {
            DependencySource::GitHub {
                owner,
                repo,
                path,
                version,
            } => {
                let ref_part = match version {
                    Some(Version::Tag(t)) => t,
                    Some(Version::Branch(b)) => b,
                    Some(Version::Commit(c)) => c,
                    None => "main", // 默认分支
                };
                format!(
                    "https://raw.githubusercontent.com/{}/{}/{}/{}",
                    owner, repo, ref_part, path
                )
            }
            DependencySource::Url(url) => url.clone(),
        }
    }

    pub fn file_name(&self) -> String {
        match self {
            DependencySource::GitHub { path, .. } => {
                path.split('/').last().unwrap_or("file.md").to_string()
            }
            DependencySource::Url(url) => {
                url.split('/').last().unwrap_or("file.md").to_string()
            }
        }
    }

    pub fn output_path(&self, _name: &str) -> String {
        match self {
            DependencySource::GitHub {
                owner,
                repo,
                path,
                ..
            } => {
                // 创建类似 owner-repo/path/to/file.md 的路径
                let dir_part = if path.contains('/') {
                    let dir = path.rsplitn(2, '/').nth(1).unwrap_or("");
                    format!("{}-{}/{}", owner, repo, dir)
                } else {
                    format!("{}-{}", owner, repo)
                };
                format!("{}/{}", dir_part, self.file_name())
            }
            DependencySource::Url(_) => {
                self.file_name()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_simple() {
        let value = toml::Value::String("github:rust-lang/book/src/ch01.md".to_string());
        let dep = DependencySource::parse(&value).unwrap();
        
        if let DependencySource::GitHub { owner, repo, path, .. } = dep {
            assert_eq!(owner, "rust-lang");
            assert_eq!(repo, "book");
            assert_eq!(path, "src/ch01.md");
        } else {
            panic!("Expected GitHub source");
        }
    }

    #[test]
    fn test_parse_github_with_branch() {
        let mut table = toml::value::Table::new();
        table.insert(
            "git".to_string(),
            toml::Value::String("github:owner/repo/docs/api.md".to_string()),
        );
        table.insert(
            "branch".to_string(),
            toml::Value::String("main".to_string()),
        );

        let value = toml::Value::Table(table);
        let dep = DependencySource::parse(&value).unwrap();

        if let DependencySource::GitHub {
            owner,
            repo,
            path,
            version,
            ..
        } = dep
        {
            assert_eq!(owner, "owner");
            assert_eq!(repo, "repo");
            assert_eq!(path, "docs/api.md");
            assert_eq!(version, Some(Version::Branch("main".to_string())));
        } else {
            panic!("Expected GitHub source");
        }
    }
}
