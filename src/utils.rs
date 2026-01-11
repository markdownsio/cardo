use std::path::Path;

pub fn find_config_file() -> Option<String> {
    let current_dir = std::env::current_dir().ok()?;
    
    // 检查当前目录
    let config_path = current_dir.join("markdown.toml");
    if config_path.exists() {
        return Some("markdown.toml".to_string());
    }

    // 也可以检查父目录（类似 Cargo 的行为）
    let parent_config = current_dir.parent()?.join("markdown.toml");
    if parent_config.exists() {
        return Some(parent_config.display().to_string());
    }

    None
}

pub fn ensure_output_dir(dir: &str) -> std::io::Result<()> {
    if !Path::new(dir).exists() {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_config_file() {
        // 这个测试需要在实际环境中运行
        let _ = find_config_file();
    }
}
