# lunar-extract-rust

**LunarAST adapter for Rust (Axum framework).**

`lunar-extract-rust` is a language-specific adapter that statically analyzes Rust source code and extracts HTTP route definitions into the [LunarAST LDJSON format](https://github.com/LunarAST/RouteAST#31-line-delimited-json-ldjson-output-stream-format).

## Supported Frameworks

| Framework | Status |
|:---|:---|
| Axum 0.7 | ✅ Supported |

## Installation

```bash
cargo install lunar-extract-rust
```

Or build from source:

```bash
git clone https://github.com/LunarAST/lunar-extract-rust.git
cd lunar-extract-rust
cargo build --release
```

## Usage

### Standalone

```bash
lunar-extract-rust /path/to/rust/project
```

### With `lunar` CLI

The `lunar` CLI discovers this adapter automatically via `PATH`. Install both:

```bash
cargo install lunar
cargo install lunar-extract-rust
```

Then run:

```bash
cd /path/to/rust/project
lunar scan
```

## Output Format

The adapter outputs [Line-Delimited JSON (LDJSON)](https://jsonlines.org/) to stdout:

```json
{"method":"GET","segments":[{"type":"literal","value":"healthz"}],"source_file":"src/main.rs","line_number":0,"extraction_method":"ast"}
{"method":"POST","segments":[{"type":"literal","value":"api"},{"type":"literal","value":"v1"},{"type":"parameter","name":"userId","raw_constraint":"\\d+"}],"source_file":"src/main.rs","line_number":0,"extraction_method":"ast"}
{"_lunar":{"status":"success","count":2}}
```

## License

Apache-2.0# lunar-extract-rust

**LunarAST adapter for Rust (Axum framework).**

`lunar-extract-rust` is a language-specific adapter that statically analyzes Rust source code and extracts HTTP route definitions into the [LunarAST LDJSON format](https://github.com/LunarAST/RouteAST#31-line-delimited-json-ldjson-output-stream-format).

## Supported Frameworks

| Framework | Status |
|:---|:---|
| Axum 0.7 | ✅ Supported |

## Installation

```bash
cargo install lunar-extract-rust
