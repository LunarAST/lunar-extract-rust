use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
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

#[derive(Serialize)]
struct RouteEntry {
    method: String,
    segments: Vec<RouteSegment>,
    source_file: String,
    line_number: usize,
    extraction_method: String,
}

fn parse_path_segments(path: &str) -> Vec<RouteSegment> {
    let mut segments = Vec::new();
    for part in path.split('/') {
        if part.is_empty() {
            continue;
        }
        if part == "*" {
            segments.push(RouteSegment {
                segment_type: "wildcard".to_string(),
                value: None,
                name: None,
                raw_constraint: None,
            });
        } else if part.starts_with(':') {
            let name = part[1..].to_string();
            segments.push(RouteSegment {
                segment_type: "parameter".to_string(),
                value: None,
                name: Some(name),
                raw_constraint: None,
            });
        } else if part.starts_with('{') && part.ends_with('}') {
            let inner = &part[1..part.len() - 1];
            if let Some(colon_pos) = inner.find(':') {
                let name = inner[..colon_pos].to_string();
                let constraint = inner[colon_pos + 1..].to_string();
                segments.push(RouteSegment {
                    segment_type: "parameter".to_string(),
                    value: None,
                    name: Some(name),
                    raw_constraint: Some(constraint),
                });
            } else {
                segments.push(RouteSegment {
                    segment_type: "parameter".to_string(),
                    value: None,
                    name: Some(inner.to_string()),
                    raw_constraint: None,
                });
            }
        } else {
            segments.push(RouteSegment {
                segment_type: "literal".to_string(),
                value: Some(part.to_string()),
                name: None,
                raw_constraint: None,
            });
        }
    }
    segments
}

fn scan_file(file_path: &Path) -> anyhow::Result<Vec<RouteEntry>> {
    let content = fs::read_to_string(file_path)?;
    let syntax = syn::parse_file(&content)?;
    let mut routes = Vec::new();

    for item in &syntax.items {
        if let syn::Item::Fn(item_fn) = item {
            for stmt in &item_fn.block.stmts {
                extract_routes_from_stmt(stmt, file_path, &mut routes);
            }
        }
    }
    Ok(routes)
}

fn extract_routes_from_stmt(stmt: &syn::Stmt, file_path: &Path, routes: &mut Vec<RouteEntry>) {
    match stmt {
        syn::Stmt::Expr(expr, _) => {
            extract_routes_from_expr(expr, file_path, routes);
        }
        syn::Stmt::Local(local) => {
            if let Some(init) = &local.init {
                extract_routes_from_expr(&init.expr, file_path, routes);
            }
        }
        _ => {}
    }
}

fn extract_routes_from_expr(expr: &syn::Expr, file_path: &Path, routes: &mut Vec<RouteEntry>) {
    match expr {
        syn::Expr::MethodCall(method_call) => {
            if method_call.method == "route" {
                if let Some((path, method)) = extract_route_args(&method_call.args) {
                    let segments = parse_path_segments(&path);
                    let line = 0; // span line number unavailable in current proc_macro2 version
                    routes.push(RouteEntry {
                        method,
                        segments,
                        source_file: file_path.to_string_lossy().to_string(),
                        line_number: line,
                        extraction_method: "ast".to_string(),
                    });
                }
            }
            extract_routes_from_expr(&method_call.receiver, file_path, routes);
            for arg in &method_call.args {
                extract_routes_from_expr(arg, file_path, routes);
            }
        }
        syn::Expr::Call(call) => {
            for arg in &call.args {
                extract_routes_from_expr(arg, file_path, routes);
            }
        }
        _ => {}
    }
}

fn extract_route_args(
    args: &syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
) -> Option<(String, String)> {
    let mut iter = args.iter();
    let path_expr = iter.next()?;
    let method_expr = iter.next()?;
    let path = extract_string_literal(path_expr)?;
    let method = match method_expr {
        syn::Expr::Path(expr_path) => {
            let ident = expr_path.path.get_ident()?;
            ident.to_string().to_uppercase()
        }
        syn::Expr::Call(call) => {
            if let syn::Expr::Path(expr_path) = &*call.func {
                let ident = expr_path.path.get_ident()?;
                ident.to_string().to_uppercase()
            } else {
                return None;
            }
        }
        _ => return None,
    };
    match method.as_str() {
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" => {
            Some((path, method))
        }
        _ => None,
    }
}

fn extract_string_literal(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Lit(lit) => {
            if let syn::Lit::Str(s) = &lit.lit {
                Some(s.value())
            } else {
                None
            }
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
                    if name == "target" || name.starts_with('.') {
                        continue;
                    }
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
    if args.len() < 2 {
        anyhow::bail!("Usage: lunar-extract-rust <project_directory>");
    }
    let project_dir = Path::new(&args[1]);
    let routes = scan_project(project_dir)?;

    for route in &routes {
        let line = serde_json::to_string(route)?;
        println!("{}", line);
    }

    let marker = serde_json::json!({
        "_lunar": {
            "status": "success",
            "count": routes.len()
        }
    });
    println!("{}", serde_json::to_string(&marker)?);

    Ok(())
}
