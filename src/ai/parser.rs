use std::collections::HashMap;

/// Parse AI response into a map of filename -> content.
///
/// Supports multiple patterns:
/// 1. ```lang:filename ... ```
/// 2. === filename ===
/// 3. // filename comments
pub fn parse_ai_response(response: &str) -> HashMap<String, String> {
    let mut files = HashMap::new();

    // Pattern 1: ```lang:filename or ```filename
    // Matches: ```tsx:app/root.tsx ... ``` or ```js:worker.js ... ```
    let mut in_block = false;
    let mut current_file: Option<String> = None;
    let mut current_content = String::new();

    for line in response.lines() {
        let trimmed = line.trim();

        if !in_block {
            // Check for code block start with filename
            if let Some(rest) = trimmed.strip_prefix("```") {
                // rest could be "tsx:app/root.tsx" or "json:package.json" or just "tsx"
                let after_lang = if let Some(colon_pos) = rest.find(':') {
                    let after = &rest[colon_pos + 1..];
                    // Check if what's after the colon looks like a filename
                    if after.contains('/') || after.contains('.') || after.contains('\\') {
                        Some(after.to_string())
                    } else {
                        // The colon might be part of the lang (e.g., "typescript:app.tsx")
                        if let Some(colon2_pos) = after.find(':') {
                            let maybe_file = &after[colon2_pos + 1..];
                            if maybe_file.contains('/') || maybe_file.contains('.') {
                                Some(maybe_file.to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                } else {
                    None
                };

                if let Some(filename) = after_lang {
                    in_block = true;
                    current_file = Some(filename);
                    current_content.clear();
                } else {
                    // Code block without filename - try to detect from lang line
                    // Skip unnamed blocks
                    in_block = true;
                    current_file = None;
                    current_content.clear();
                }
            }
        } else {
            // Check for code block end
            if trimmed == "```" {
                in_block = false;
                if let Some(ref filename) = current_file {
                    let content = current_content.trim_end_matches('\n').to_string();
                    if !content.is_empty() {
                        files.insert(filename.clone(), content);
                    }
                }
                current_file = None;
                current_content.clear();
            } else {
                if !current_content.is_empty() {
                    current_content.push('\n');
                }
                current_content.push_str(line);
            }
        }
    }

    // If we ended with an unclosed block, still capture it
    if in_block {
        if let Some(ref filename) = current_file {
            let content = current_content.trim_end_matches('\n').to_string();
            if !content.is_empty() {
                files.insert(filename.clone(), content);
            }
        }
    }

    // Pattern 2: === filename === (some models use this)
    if files.is_empty() {
        let mut sections: Vec<(&str, usize)> = Vec::new();
        for (i, line) in response.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("===") && trimmed.ends_with("===") && trimmed.len() > 6 {
                let filename = trimmed[3..trimmed.len() - 3].trim();
                if !filename.is_empty() {
                    sections.push((filename, i));
                }
            }
        }

        for (idx, (filename, start_line)) in sections.iter().enumerate() {
            let end_line = if idx + 1 < sections.len() {
                sections[idx + 1].1
            } else {
                response.lines().count()
            };

            let content: String = response
                .lines()
                .skip(start_line + 1)
                .take(end_line - start_line - 1)
                .collect::<Vec<_>>()
                .join("\n");

            let content = content.trim().to_string();
            if !content.is_empty() {
                files.insert(filename.to_string(), content);
            }
        }
    }

    files
}

/// Validate that the generated files contain minimum required files for the framework.
pub fn validate_project_structure(
    files: &HashMap<String, String>,
    framework: &str,
) -> Result<(), String> {
    match framework {
        "react-router" => {
            let required = [
                "package.json",
                "vite.config.ts",
                "wrangler.jsonc",
                "app/root.tsx",
            ];
            for f in &required {
                if !files.contains_key(*f) {
                    return Err(format!("Missing required file for react-router: {}", f));
                }
            }
        }
        "nuxtjs" => {
            let required = ["package.json", "app/app.vue"];
            for f in &required {
                if !files.contains_key(*f) {
                    return Err(format!("Missing required file for nuxtjs: {}", f));
                }
            }
        }
        "worker" => {
            if !files.contains_key("worker.js") && !files.contains_key("index.js") {
                return Err("Missing required file for worker: worker.js or index.js".to_string());
            }
        }
        _ => {}
    }
    Ok(())
}
