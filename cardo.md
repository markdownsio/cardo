# Cardo 技术方案文档

## 1. 项目概述

Cardo 是一个基于 Rust 开发的命令行工具，用于管理 Markdown 文件的依赖加载。Cardo 借鉴了 Cargo（Rust 包管理器）的设计理念，但专注于管理 GitHub 上的 Markdown 文件资源，而非 Rust Crate。

### 1.1 核心概念

- **Cardo**: CLI 工具名称，灵感来自 "Cargo of document" 的含义和 "Cargo"（Rust 包管理器）
- **markdown.toml**: 配置文件，用于声明 Markdown 文件的依赖关系
- **markdowns/**: 输出目录，用于存储下载的 Markdown 文件

### 1.2 设计目标

- 类似 Cargo.toml 的配置体验
- 支持从 GitHub 仓库加载单个 .md 文件
- 自动管理依赖和版本控制
- 简洁高效的命令行接口

## 2. 核心功能

### 2.1 依赖管理

Cardo 通过 `markdown.toml` 文件声明依赖，支持的依赖源：

- **GitHub 仓库文件**: 通过 GitHub API 或直接 URL 下载特定文件的特定版本（tag/branch/commit）
- **本地文件**: 支持引用本地 Markdown 文件（可选功能）

### 2.2 主要命令

```bash
# 初始化项目，创建 markdown.toml 文件
cardo init

# 下载所有依赖的 Markdown 文件到 markdowns/ 目录
cardo fetch

# 更新依赖（如果有新版本）
cardo update

# 列出所有依赖
cardo list

# 清理 markdowns/ 目录
cardo clean
```

## 3. markdown.toml 格式规范

### 3.1 基本结构

`markdown.toml` 文件格式类似 `Cargo.toml`，采用 TOML 格式：

```toml
[package]
name = "my-project"
version = "0.1.0"

[dependencies]
# 依赖声明
```

### 3.2 依赖声明格式

#### GitHub 文件依赖

```toml
[dependencies]
# 基本格式：name = "github:owner/repo/path/to/file.md"
example-doc = "github:username/repo/docs/README.md"

# 指定版本（tag）
api-docs = { git = "github:owner/repo/docs/api.md", tag = "v1.0.0" }

# 指定分支
guide = { git = "github:owner/repo/docs/guide.md", branch = "main" }

# 指定提交 SHA
changelog = { git = "github:owner/repo/CHANGELOG.md", rev = "abc1234" }

# 完整 URL 格式（备用方案）
config = "https://raw.githubusercontent.com/owner/repo/main/docs/config.md"
```

### 3.3 配置示例

详见 `markdown.toml.example` 文件。

## 4. 技术架构

### 4.1 技术栈

- **语言**: Rust
- **命令行解析**: `clap` 或 `structopt`
- **TOML 解析**: `toml` crate
- **HTTP 客户端**: `reqwest`（支持异步）
- **文件系统操作**: `std::fs` + `tokio::fs`（异步）
- **错误处理**: `anyhow` + `thiserror`
- **日志**: `tracing` 或 `env_logger`

### 4.2 项目结构

```
cardo/
├── Cargo.toml
├── src/
│   ├── main.rs              # 入口点
│   ├── cli.rs               # CLI 命令定义
│   ├── config.rs            # markdown.toml 解析
│   ├── dependency.rs        # 依赖模型定义
│   ├── fetcher.rs           # Markdown 文件下载逻辑
│   ├── github.rs            # GitHub API 交互
│   └── utils.rs             # 工具函数
├── tests/                   # 测试文件
└── README.md
```

### 4.3 核心模块设计

#### 4.3.1 配置解析 (`config.rs`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MarkdownConfig {
    pub package: Package,
    pub dependencies: Option<Dependencies>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dependencies {
    // 依赖表，key 为依赖名称，value 为依赖源
}
```

#### 4.3.2 依赖模型 (`dependency.rs`)

```rust
pub enum DependencySource {
    GitHub {
        owner: String,
        repo: String,
        path: String,
        version: Option<Version>,
    },
    Url(String),
}

pub enum Version {
    Tag(String),
    Branch(String),
    Commit(String),
}
```

#### 4.3.3 GitHub 交互 (`github.rs`)

- 使用 GitHub Raw Content API 下载文件
- 支持通过 GitHub API 获取特定版本的文件
- 处理 GitHub 的认证（可选，支持 token）

API 端点示例：
- `https://raw.githubusercontent.com/{owner}/{repo}/{ref}/{path}`
- 或使用 GitHub GraphQL/REST API

#### 4.3.4 文件下载 (`fetcher.rs`)

- 异步下载 Markdown 文件
- 保持目录结构（如果需要）
- 错误处理和重试机制
- 缓存管理（避免重复下载）

### 4.4 数据流

```
用户执行命令
    ↓
解析 markdown.toml
    ↓
解析依赖声明
    ↓
构建下载任务列表
    ↓
并发/异步下载文件
    ↓
保存到 markdowns/ 目录
    ↓
生成锁定文件（可选，类似 Cargo.lock）
```

## 5. 实现细节

### 5.1 文件下载策略

1. **并发下载**: 使用 `tokio::spawn` 或 `futures::future::join_all` 并发下载多个文件
2. **错误处理**: 对每个文件下载进行独立错误处理，不影响其他文件
3. **重试机制**: 实现指数退避重试，提高下载成功率
4. **进度显示**: 使用 `indicatif` 显示下载进度

### 5.2 目录结构处理

- 根据 GitHub 路径保持相对目录结构
- 例如：`github:owner/repo/docs/api.md` → `markdowns/owner-repo/docs/api.md`
- 或使用扁平结构：`markdowns/api.md`（简化方案）

### 5.3 版本管理

- 可选：生成 `markdown.lock` 文件，记录实际下载的版本
- 支持版本锁定，确保可复现性
- 版本比较和更新检测

### 5.4 缓存机制

- 检查文件是否已存在且版本匹配
- 支持 `--force` 标志强制重新下载
- 可选：本地缓存目录，避免重复下载

## 6. 使用示例

### 6.1 初始化项目

```bash
$ cardo init
Created markdown.toml
```

### 6.2 添加依赖

编辑 `markdown.toml`，添加依赖：

```toml
[dependencies]
rust-guide = "github:rust-lang/book/src/ch01-00-getting-started.md"
```

### 6.3 下载依赖

```bash
$ cardo fetch
Downloading rust-guide...
  ✓ Downloaded to markdowns/rust-lang-book/src/ch01-00-getting-started.md
```

### 6.4 更新依赖

```bash
$ cardo update
Updating dependencies...
  ✓ rust-guide: v1.0.0 -> v1.1.0
```

## 7. 错误处理

常见错误场景：

1. **网络错误**: GitHub API 不可达或超时
2. **文件不存在**: 指定的 .md 文件在仓库中不存在
3. **权限问题**: 私有仓库需要认证
4. **格式错误**: markdown.toml 格式不正确
5. **文件系统错误**: 权限不足或磁盘空间不足

## 8. 扩展功能（可选）

### 8.1 高级特性

- **依赖解析**: 支持嵌套依赖（Markdown 文件引用其他 Markdown 文件）
- **插件系统**: 支持预处理和后处理钩子
- **Git 集成**: 支持将 markdowns/ 目录加入 .gitignore，或可选地提交
- **验证**: 验证下载的 Markdown 文件格式
- **搜索**: 在下载的 Markdown 文件中搜索内容

### 8.2 性能优化

- 并行下载多个文件
- 增量更新（仅下载变更的文件）
- 本地缓存池

## 9. 开发计划

### Phase 1: 核心功能
- [ ] CLI 框架搭建
- [ ] markdown.toml 解析
- [ ] GitHub 文件下载
- [ ] 基本命令实现（init, fetch, list）

### Phase 2: 增强功能
- [ ] 版本管理
- [ ] 错误处理和重试
- [ ] 进度显示
- [ ] 锁定文件支持

### Phase 3: 优化和扩展
- [ ] 性能优化
- [ ] 扩展功能
- [ ] 文档完善
- [ ] 测试覆盖

## 10. 参考资料

- [Cargo 文档](https://doc.rust-lang.org/cargo/)
- [GitHub API 文档](https://docs.github.com/en/rest)
- [TOML 规范](https://toml.io/)
- [Rust CLI 最佳实践](https://rust-cli.github.io/book/)
