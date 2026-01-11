# Cardo

A CLI tool for managing Markdown file dependencies, inspired by Cargo.

## Installation

```bash
cargo build --release
```


## Usage

### Fetch dependencies

```bash
./target/release/cardo fetch
```

This downloads all dependencies to the `markdowns/` directory.

### List dependencies

```bash
./target/release/cardo list
```

### Update dependencies

```bash
./target/release/cardo update
```

### Clean output directory

```bash
./target/release/cardo clean
```

## Configuration Format

See `markdown.toml.example` for examples.

### Simple format

```toml
[dependencies]
doc = "github:owner/repo/path/to/file.md"
```

### Full format with version

```toml
[dependencies]
doc = { git = "github:owner/repo/path/to/file.md", branch = "main" }
doc = { git = "github:owner/repo/path/to/file.md", tag = "v1.0.0" }
doc = { git = "github:owner/repo/path/to/file.md", rev = "abc123" }
```

### Direct URL

```toml
[dependencies]
doc = "https://raw.githubusercontent.com/owner/repo/main/file.md"
```

## Environment Variables

- `GITHUB_TOKEN`: Optional GitHub token for accessing private repositories or increasing rate limits

## License

MIT OR Apache-2.0
