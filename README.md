# Cardo

A CLI tool for managing Markdown file dependencies, inspired by Cargo.

## Installation

```bash
cargo build --release
```

## Quick Start

```
>> following the SKILL to create a canvas about this cardo tool
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

## License

MIT
