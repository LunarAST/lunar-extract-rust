# lunar-extract-rust

**LunarAST 适配 Rust（Axum 框架）的路由解析适配器**

`lunar-extract-rust` 是面向 Rust 语言的专用适配器，用于对 Rust 源代码执行静态分析，提取其中定义的 HTTP 路由，并输出为 [LunarAST 按行分隔 JSON（LDJSON）标准格式](https://github.com/LunarAST/RouteAST#31-line-delimited-json-ldjson-output-stream-format)。

## 支持框架

| 框架 | 适配状态 |
|:---|:---|
| Axum 0.7 | ✅ 已支持 |

## 安装

```bash
cargo install lunar-extract-rust
```

也可通过源码编译安装：

```bash
git clone https://github.com/LunarAST/lunar-extract-rust.git
cd lunar-extract-rust
cargo build --release
```

## 使用方法

### 独立运行

```bash
lunar-extract-rust /path/to/rust/project
```

### 搭配 lunar 命令行工具使用
`lunar` 命令行会通过系统环境变量 `PATH` 自动发现本适配器。请先完成两者安装：

```bash
cargo install lunar
cargo install lunar-extract-rust
```

随后执行扫描：

```bash
cd /path/to/rust/project
lunar scan
```

## 输出格式
本适配器向**标准输出（stdout）**输出**按行分隔 JSON（LDJSON）**格式内容。每一条路由对应一个独立 JSON 对象，末尾附带一条流结束标记：

```json
{"method":"GET","segments":[{"type":"literal","value":"healthz"}],"source_file":"src/main.rs","line_number":10,"extraction_method":"ast"}
{"method":"POST","segments":[{"type":"literal","value":"api"},{"type":"literal","value":"v1"},{"type":"parameter","name":"userId","raw_constraint":"\\d+"}],"source_file":"src/main.rs","line_number":20,"extraction_method":"ast"}
{"_lunar":{"status":"success","count":2}}
```

流结束标记中包含 `count` 字段，该数值必须与前面路由条目总行数保持一致。`lunar` 命令行会校验该数值，校验通过才会判定输出结果合法。

## Rustdoc 模式（可选）
针对复杂宏语法场景，为保证解析准确率，`lunar-extract-rust` 提供可选的 Rustdoc 模式，该模式依托 Rust 编译器原生语义分析能力完成解析。

```bash
# 1. 生成 Rustdoc 结构化 JSON（需使用 Rust 夜间版工具链）
cargo +nightly rustdoc -- -Z unstable-options --output-format json

# 2. 从编译器输出结果中提取路由信息
lunar-extract-rust --rustdoc /path/to/project
```

**工作原理**：
该模式不会直接解析 `.rs` 源码文件，而是读取 `rustc` 生成的 `target/doc/rustdoc.json` 文件。这份 JSON 包含经过宏展开、类型校验、语法解析后的完整接口信息（包含路由处理器）。借助该能力可 100% 穿透各类复杂宏语法，无需手动遍历抽象语法树（AST）做兼容处理。

**降级逻辑**：
如果未找到 Rustdoc 生成的 JSON 文件，适配器会**自动降级**为默认基于 `syn` 的解析方案，全程不会中断执行流程。因此你可以在 CI 流水线中直接使用 `--rustdoc` 参数，无需提前做环境检测。

**适用场景**：
- 项目大量使用宏语法、导致路由定义被隐藏
- 要求解析结果 100% 准确的 CI 流水线、安全审计场景
- 项目本身已在生成 Rustdoc 文档的场景

## 诊断规则
- 若解析单个源码文件失败，适配器会向**标准错误输出（stderr）**打印警告信息，并继续扫描其他文件。
- 若项目中未扫描到任何路由，仍会输出合法的流结束标记，且 `count` 值为 0。
- 遇到**不可恢复错误**（例如项目路径无效）时，会输出 `{"_lunar":{"status":"error","message":"..."}}`，并以非零退出码终止程序。

## 输出规范
- 所有 JSON 字段统一采用**小驼峰命名（camelCase）**，遵循 Google JSON 编码规范（通过 `#[serde(rename_all = "camelCase")]` 实现）。
- 解析路由路径时，会先剔除路径首尾斜杠再做分割，从根源避免产生空文本路由段。
- 适配器采用**递归 AST 遍历**，可覆盖 `if`、`match`、`loop`、`while`、`for`、闭包、异步代码、`unsafe` 等各类代码逻辑。
- 启用 Rustdoc 模式时，`extractionMethod` 字段会标记为 `"rustdoc"`，用于区分数据来源。

## 功能局限
- 目前仅支持 Axum 0.7 标准 `Router::route()` 写法，自定义路由宏、其他路由构建器暂无法识别。
- 路由参数中复杂正则约束（例如 `/:id(\\d+)`）会原样保留在 `rawConstraint` 元数据中，但不会跨项目做正则校验。
- 暂不支持 gRPC 路由解析。
- Rustdoc 模式仍在持续迭代，目前仅能提取基础路由信息，完整语义化路由解析将在后续版本完成。

## 开源许可证
Apache-2.0
