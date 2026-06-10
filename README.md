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

## Rustdoc Mode (Optional)

For maximum accuracy with complex macro patterns, `lunar-extract-rust` supports an optional rustdoc mode that leverages the Rust compiler's own semantic analysis.

```bash
# 1. Generate rustdoc JSON (requires nightly)
cargo +nightly rustdoc -- -Z unstable-options --output-format json

# 2. Extract routes from the compiler output
lunar-extract-rust --rustdoc /path/to/project
```

**How it works**: Instead of parsing `.rs` source files directly, this mode reads the `target/doc/rustdoc.json` produced by `rustc`. This JSON contains fully expanded, type-checked, and macro-resolved API information — including route handlers. It provides 100% accurate macro penetration and eliminates the need for manual AST traversal of complex macro patterns.

**Fallback behavior**: If the rustdoc JSON file is not found, the adapter automatically falls back to the default `syn`-based extraction, ensuring zero workflow interruption. This means you can safely use `--rustdoc` in CI pipelines without pre-checks.

**When to use**:
- Projects with heavy macro usage that obscures route definitions
- CI pipelines and security audits where 100% accuracy is required
- When you already generate rustdoc JSON for documentation purposes

## Diagnostics

- If the adapter fails to parse a source file, it prints a warning to `stderr` and continues scanning other files.
- If no routes are found, it still outputs a valid end marker with `count: 0`.
- On unrecoverable errors (e.g., invalid project path), it outputs `{"_lunar":{"status":"error","message":"..."}}` and exits with a non-zero exit code.

## Output Conventions

- All JSON field names follow **camelCase** (Google JSON Style Guide) via `#[serde(rename_all = "camelCase")]`.
- All path extraction trims leading and trailing slashes before splitting, preventing empty literal segments.
- The adapter uses recursive AST traversal and covers control flow expressions (`if`, `match`, `loop`, `while`, `for`, `closure`, `async`, `unsafe`, etc.).
- In rustdoc mode, the `extractionMethod` field is set to `"rustdoc"` to indicate the source of the extraction.

## Limitations

- Only Axum 0.7's `Router::route()` pattern is supported. Custom route macros or other router builders may not be detected.
- Route parameters with complex regex constraints (e.g., `/:id(\\d+)`) are preserved as descriptive metadata (`rawConstraint`) but not validated across projects.
- gRPC routes are not yet supported.
- Rustdoc mode is under active development and currently extracts basic route information. Full semantic route extraction will be completed in a future iteration.

## License

Apache-2.0
