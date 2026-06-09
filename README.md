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

The adapter outputs [Line-Delimited JSON (LDJSON)](https://jsonlines.org/) to stdout. Every route is serialized as a single JSON object, followed by an end-of-stream marker:

```json
{"method":"GET","segments":[{"type":"literal","value":"healthz"}],"source_file":"src/main.rs","line_number":10,"extraction_method":"ast"}
{"method":"POST","segments":[{"type":"literal","value":"api"},{"type":"literal","value":"v1"},{"type":"parameter","name":"userId","raw_constraint":"\\d+"}],"source_file":"src/main.rs","line_number":20,"extraction_method":"ast"}
{"_lunar":{"status":"success","count":2}}
```

The end marker contains a `count` field that must match the number of route lines emitted. The `lunar` CLI verifies this count before accepting the output as valid.

## Diagnostics

- If the adapter fails to parse a source file, it prints a warning to `stderr` and continues scanning other files.
- If no routes are found, it still outputs a valid end marker with `count: 0`.
- On unrecoverable errors (e.g., invalid project path), it outputs `{"_lunar":{"status":"error","message":"..."}}` and exits with a non-zero exit code.

## Limitations

- Only Axum 0.7's `Router::route()` pattern is supported. Custom route macros or other router builders may not be detected.
- Route parameters with complex regex constraints (e.g., `/:id(\\d+)`) are preserved as descriptive metadata (`rawConstraint`) but not validated across projects.
- gRPC routes are not yet supported.

## License

Apache-2.0
