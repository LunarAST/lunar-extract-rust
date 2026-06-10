use serde::Serialize;
use std::fs;
use std::path::Path;

mod extract_doc;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RouteSegment {
    #[serde(rename = "type")]
    segment_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_constraint: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RouteEntry {
    method: String,
    segments: Vec<RouteSegment>,
    source_file: String,
    line_number: usize,
    extraction_method: String,
}

fn parse_path_segments(path: &str) -> Vec<RouteSegment> {
    let mut segments = Vec::new();
    for part in path.trim_matches('/').split('/') {
        if part.is_empty() { continue; }
        if part == "*" {
            segments.push(RouteSegment { segment_type: "wildcard".to_string(), value: None, name: None, raw_constraint: None });
        } else if part.starts_with(':') {
            let name = part[1..].to_string();
            segments.push(RouteSegment { segment_type: "parameter".to_string(), value: None, name: Some(name), raw_constraint: None });
        } else if part.starts_with('{') && part.ends_with('}') {
            let inner = &part[1..part.len()-1];
            if let Some(colon_pos) = inner.find(':') {
                let name = inner[..colon_pos].to_string();
                let constraint = inner[colon_pos+1..].to_string();
                segments.push(RouteSegment { segment_type: "parameter".to_string(), value: None, name: Some(name), raw_constraint: Some(constraint) });
            } else {
                segments.push(RouteSegment { segment_type: "parameter".to_string(), value: None, name: Some(inner.to_string()), raw_constraint: None });
            }
        } else {
            segments.push(RouteSegment { segment_type: "literal".to_string(), value: Some(part.to_string()), name: None, raw_constraint: None });
        }
    }
    segments
}

fn scan_file(file_path: &Path) -> anyhow::Result<Vec<RouteEntry>> {
    let content = fs::read_to_string(file_path)?;
    let syntax = syn::parse_file(&content)?;
    let mut routes = Vec::new();
    for item in &syntax.items {
        extract_from_item(item, file_path, &mut routes);
    }
    Ok(routes)
}

fn extract_from_item(item: &syn::Item, file_path: &Path, routes: &mut Vec<RouteEntry>) {
    match item {
        syn::Item::Fn(item_fn) => {
            for stmt in &item_fn.block.stmts {
                extract_from_stmt(stmt, file_path, routes);
            }
        }
        _ => {}
    }
}

fn extract_from_stmt(stmt: &syn::Stmt, file_path: &Path, routes: &mut Vec<RouteEntry>) {
    match stmt {
        syn::Stmt::Expr(expr, _) => extract_from_expr(expr, file_path, routes),
        syn::Stmt::Local(local) => {
            if let Some(init) = &local.init {
                extract_from_expr(&init.expr, file_path, routes);
            }
        }
        _ => {}
    }
}

fn extract_from_expr(expr: &syn::Expr, file_path: &Path, routes: &mut Vec<RouteEntry>) {
    match expr {
        syn::Expr::MethodCall(method_call) => {
            if method_call.method == "route" {
                if let Some((path, methods)) = extract_route_methods(&method_call.args) {
                    let segments = parse_path_segments(&path);
                    let line = method_call.method.span().start().line as usize;
                    for method in methods {
                        routes.push(RouteEntry {
                            method,
                            segments: segments.clone(),
                            source_file: file_path.to_string_lossy().to_string(),
                            line_number: line,
                            extraction_method: "ast".to_string(),
                        });
                    }
                }
            }
            extract_from_expr(&method_call.receiver, file_path, routes);
            for arg in &method_call.args {
                extract_from_expr(arg, file_path, routes);
            }
        }
        syn::Expr::Call(call) => {
            extract_from_expr(&call.func, file_path, routes);
            for arg in &call.args {
                extract_from_expr(arg, file_path, routes);
            }
        }
        syn::Expr::If(expr_if) => {
            extract_from_expr(&expr_if.cond, file_path, routes);
            for stmt in &expr_if.then_branch.stmts { extract_from_stmt(stmt, file_path, routes); }
            if let Some((_, else_branch)) = &expr_if.else_branch { extract_from_expr(else_branch, file_path, routes); }
        }
        syn::Expr::Match(expr_match) => {
            extract_from_expr(&expr_match.expr, file_path, routes);
            for arm in &expr_match.arms {
                if let Some((_, guard)) = &arm.guard { extract_from_expr(guard, file_path, routes); }
                extract_from_expr(&arm.body, file_path, routes);
            }
        }
        syn::Expr::Loop(expr_loop) => {
            for stmt in &expr_loop.body.stmts { extract_from_stmt(stmt, file_path, routes); }
        }
        syn::Expr::While(expr_while) => {
            extract_from_expr(&expr_while.cond, file_path, routes);
            for stmt in &expr_while.body.stmts { extract_from_stmt(stmt, file_path, routes); }
        }
        syn::Expr::ForLoop(expr_for) => {
            extract_from_expr(&expr_for.expr, file_path, routes);
            for stmt in &expr_for.body.stmts { extract_from_stmt(stmt, file_path, routes); }
        }
        syn::Expr::Block(expr_block) => {
            for stmt in &expr_block.block.stmts { extract_from_stmt(stmt, file_path, routes); }
        }
        syn::Expr::Closure(expr_closure) => {
            extract_from_expr(&expr_closure.body, file_path, routes);
        }
        syn::Expr::Async(expr_async) => {
            for stmt in &expr_async.block.stmts { extract_from_stmt(stmt, file_path, routes); }
        }
        syn::Expr::Unsafe(expr_unsafe) => {
            for stmt in &expr_unsafe.block.stmts { extract_from_stmt(stmt, file_path, routes); }
        }
        syn::Expr::Try(expr_try) => {
            extract_from_expr(&expr_try.expr, file_path, routes);
        }
        syn::Expr::Await(expr_await) => {
            extract_from_expr(&expr_await.base, file_path, routes);
        }
        syn::Expr::Assign(expr_assign) => {
            extract_from_expr(&expr_assign.right, file_path, routes);
        }
        syn::Expr::Group(expr_group) => {
            extract_from_expr(&expr_group.expr, file_path, routes);
        }
        syn::Expr::Let(expr_let) => {
            extract_from_expr(&expr_let.expr, file_path, routes);
        }
        _ => {}
    }
}

fn recurse_method_names(expr: &syn::Expr, names: &mut Vec<String>) {
    match expr {
        syn::Expr::Call(call) => {
            if let syn::Expr::Path(expr_path) = &*call.func {
                if let Some(ident) = expr_path.path.get_ident() {
                    let method = ident.to_string().to_uppercase();
                    if is_http_method(&method) { names.push(method); }
                }
            }
        }
        syn::Expr::MethodCall(method_call) => {
            let method = method_call.method.to_string().to_uppercase();
            if is_http_method(&method) { names.push(method); }
            recurse_method_names(&method_call.receiver, names);
        }
        _ => {}
    }
}

fn is_http_method(s: &str) -> bool {
    matches!(s, "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS")
}

fn extract_route_methods(args: &syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>) -> Option<(String, Vec<String>)> {
    let mut iter = args.iter();
    let path_expr = iter.next()?;
    let handler_expr = iter.next()?;
    let path = extract_string_literal(path_expr)?;
    let mut methods = Vec::new();
    recurse_method_names(handler_expr, &mut methods);
    if methods.is_empty() { None } else { Some((path, methods)) }
}

fn extract_string_literal(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Lit(lit) => {
            if let syn::Lit::Str(s) = &lit.lit { Some(s.value()) } else { None }
        }
        _ => None,
    }
}

pub fn scan_project(dir: &Path) -> anyhow::Result<Vec<RouteEntry>> {
    let mut all_routes = Vec::new();
    scan_dir(dir, &mut all_routes)?;
    Ok(all_routes)
}

fn scan_dir(dir: &Path, routes: &mut Vec<RouteEntry>) -> anyhow::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    let name = name.to_string_lossy();
                    if name == "target" || name.starts_with('.') { continue; }
                }
                scan_dir(&path, routes)?;
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                match scan_file(&path) {
                    Ok(file_routes) => routes.extend(file_routes),
                    Err(e) => eprintln!("Warning: failed to parse {}: {}", path.display(), e),
                }
            }
        }
    }
    Ok(())
}

pub fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut use_rustdoc = false;
    let mut project_dir: Option<&str> = None;

    for arg in &args[1..] {
        if arg == "--rustdoc" {
            use_rustdoc = true;
        } else {
            project_dir = Some(arg);
        }
    }

    let project_dir = project_dir.ok_or_else(|| anyhow::anyhow!("Usage: lunar-extract-rust [--rustdoc] <project_directory>"))?;
    let dir = Path::new(project_dir);

    let routes = if use_rustdoc {
        // Rustdoc mode: look for rustdoc JSON output in target/doc/
        let doc_path = dir.join("target").join("doc").join("rustdoc.json");
        if doc_path.exists() {
            extract_doc::extract_from_rustdoc(&doc_path)?
        } else {
            eprintln!("  [!] rustdoc JSON not found at {}", doc_path.display());
            eprintln!("  [!] Generate it first:");
            eprintln!("  [!]   cargo +nightly rustdoc -- -Z unstable-options --output-format json");
            eprintln!("  [!] Falling back to syn-based extraction...");
            scan_project(dir)?
        }
    } else {
        scan_project(dir)?
    };

    for route in &routes {
        println!("{}", serde_json::to_string(route)?);
    }
    let marker = serde_json::json!({"_lunar": {"status": "success", "count": routes.len()}});
    println!("{}", serde_json::to_string(&marker)?);
    Ok(())
}
