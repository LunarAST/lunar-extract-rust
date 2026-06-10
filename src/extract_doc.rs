use anyhow::Result;
use serde_json::Value;
use std::path::Path;

/// Extract routes from a rustdoc JSON file.
/// This is a minimal implementation that extracts function signatures.
/// Full RouteAST extraction from rustdoc JSON will be completed in a future iteration.
pub fn extract_from_rustdoc(json_path: &Path) -> Result<Vec<crate::RouteEntry>> {
    let content = std::fs::read_to_string(json_path)?;
    let doc: Value = serde_json::from_str(&content)?;

    let mut routes = Vec::new();

    // Navigate the rustdoc JSON structure: index -> paths
    if let Some(index) = doc.get("index").and_then(|v| v.as_object()) {
        for (_id, item) in index {
            if let Some(inner) = item.get("inner") {
                if let Some(function) = inner.get("function") {
                    if let Some(decl) = function.get("decl") {
                        // Extract function name as a basic route hint
                        let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                        if let Some(output) = decl.get("output") {
                            // This is a simplified extraction; full axum route detection
                            // from rustdoc JSON requires mapping the expanded macro outputs.
                            if let Some(attrs) = item.get("attrs").and_then(|a| a.as_array()) {
                                for attr in attrs {
                                    let attr_str = attr.as_str().unwrap_or("");
                                    if attr_str.contains("route") || attr_str.contains("get") || attr_str.contains("post") {
                                        // Found a potential route handler
                                        // For now, record it as a placeholder entry
                                        routes.push(crate::RouteEntry {
                                            method: "GET".to_string(),
                                            segments: vec![],
                                            source_file: json_path.to_string_lossy().to_string(),
                                            line_number: 0,
                                            extraction_method: "rustdoc".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if routes.is_empty() {
        eprintln!("  [!] rustdoc JSON parsed successfully but no routes extracted.");
        eprintln!("  [!] This mode is under active development and will improve.");
        eprintln!("  [!] For now, please use the default syn-based extraction:");
        eprintln!("  [!]   lunar-extract-rust /path/to/project");
    }

    Ok(routes)
}
