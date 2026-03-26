/// Document quality scorer for Metis initiative markdown files.
///
/// Reads generated initiative docs and checks whether AI actually filled in
/// the template vs left placeholder text. Used to compare cadre vs
/// original metis template quality impact on AI output.
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct DocQualityScore {
    /// Number of `{...}` placeholder patterns still present
    pub placeholder_count: u32,
    /// Section headings that were filled with real content
    pub filled_sections: Vec<String>,
    /// Section headings that are empty or still contain placeholders
    pub empty_sections: Vec<String>,
    /// Non-empty, non-placeholder, non-comment lines of content
    pub content_lines: u32,
    /// 0-100% based on filled vs total tracked sections
    pub completeness_percent: f32,
}

impl DocQualityScore {
    /// True if document looks substantively complete.
    pub fn is_complete(&self) -> bool {
        self.completeness_percent >= 70.0 && self.placeholder_count == 0
    }
}

/// Score a single markdown document file.
pub fn score_document(path: &Path) -> anyhow::Result<DocQualityScore> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read {:?}: {}", path, e))?;
    Ok(score_content(&content))
}

/// Score markdown content (exposed for unit tests).
pub fn score_content(content: &str) -> DocQualityScore {
    // Strip YAML frontmatter
    let body = strip_frontmatter(content);

    let placeholder_count = count_placeholders(body);
    let sections = extract_sections(body);

    let mut filled_sections = vec![];
    let mut empty_sections = vec![];

    for (name, section_body) in &sections {
        // Track sections that AI should fill
        let is_tracked = is_tracked_section(name);
        if !is_tracked {
            continue;
        }

        if section_is_filled(section_body) {
            filled_sections.push(name.clone());
        } else {
            empty_sections.push(name.clone());
        }
    }

    let total_tracked = (filled_sections.len() + empty_sections.len()).max(1);
    let completeness = (filled_sections.len() as f32 / total_tracked as f32) * 100.0;

    let content_lines = body
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty()
                && !t.starts_with('#')
                && !t.starts_with("---")
                && !t.starts_with("<!--")
                && !t.starts_with("//")
                && !t.starts_with('*')
                && !t.starts_with('{')
                && !t.ends_with('}')
                && !t.contains("{placeholder")
        })
        .count() as u32;

    DocQualityScore {
        placeholder_count,
        filled_sections,
        empty_sections,
        content_lines,
        completeness_percent: completeness,
    }
}

fn strip_frontmatter(content: &str) -> &str {
    if !content.starts_with("---") {
        return content;
    }
    // Find the closing ---
    let after_first = &content[3..];
    if let Some(pos) = after_first.find("\n---") {
        &after_first[pos + 4..]
    } else {
        content
    }
}

/// Count `{...}` patterns (un-filled template placeholders).
fn count_placeholders(text: &str) -> u32 {
    let mut count = 0u32;
    let mut depth = 0u32;
    for ch in text.chars() {
        match ch {
            '{' => depth += 1,
            '}' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    count += 1;
                }
            }
            _ => {}
        }
    }
    count
}

/// Extract `(heading, body_text)` pairs from markdown `##` sections.
fn extract_sections(body: &str) -> Vec<(String, String)> {
    let mut sections: Vec<(String, String)> = vec![];
    let mut current: Option<(String, String)> = None;

    for line in body.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            if let Some(sec) = current.take() {
                sections.push(sec);
            }
            current = Some((heading.trim().to_string(), String::new()));
        } else if line.starts_with("### ") {
            // Sub-section: append to current body but don't create new tracked section
            if let Some((_, ref mut body)) = current {
                body.push_str(line);
                body.push('\n');
            }
        } else if let Some((_, ref mut body)) = current {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some(sec) = current {
        sections.push(sec);
    }
    sections
}

/// Sections we care about for quality scoring.
fn is_tracked_section(name: &str) -> bool {
    let lower = name.to_lowercase();
    // Strip `[REQUIRED]` / `[CONDITIONAL: ...]` labels
    let clean = lower.split('[').next().unwrap_or(&lower).trim().to_string();

    matches!(
        clean.as_str(),
        "context"
            | "goals & non-goals"
            | "goals"
            | "objective"
            | "objectives"
            | "acceptance criteria"
            | "requirements"
            | "risks"
            | "risk considerations"
            | "tasks"
            | "decomposition"
            | "implementation notes"
            | "success criteria"
    )
}

/// True if the section body has real non-placeholder content.
fn section_is_filled(body: &str) -> bool {
    let meaningful_lines: Vec<&str> = body
        .lines()
        .map(|l| l.trim())
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with("<!--")
                && !l.starts_with("*This")
                && !l.starts_with("*Delete")
                && !l.contains("Delete if")
                && !l.contains("Delete sections")
                && !l.starts_with('{')
                && !l.ends_with('}')
        })
        .collect();

    // Need at least 2 meaningful lines AND no placeholder-only lines
    let has_content = meaningful_lines.len() >= 2;
    let no_placeholders = !body.contains('{');
    has_content && no_placeholders
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY_DOC: &str = r#"---
id: test
---
# Test Initiative

## Context **[REQUIRED]**

{Describe the context here}

## Goals **[REQUIRED]**

{Goals go here}
"#;

    const FILLED_DOC: &str = r#"---
id: test
---
# Test Initiative

## Context **[REQUIRED]**

This initiative implements the CSV parser for the File Processing Toolkit.
It addresses the need to ingest structured data from CSV files.

## Goals **[REQUIRED]**

- Parse CSV files with proper header detection and type inference
- Handle edge cases: empty files, malformed input, quoted fields
- Export to unified data model for downstream processing

## Acceptance Criteria

- [ ] Auto-detect CSV delimiter (comma, tab, semicolon)
- [ ] Handle quoted fields with embedded commas
- [ ] Type inference for numeric, date, and string columns
"#;

    #[test]
    fn test_empty_doc_scores_low() {
        let score = score_content(EMPTY_DOC);
        assert!(score.placeholder_count >= 2);
        assert!(score.completeness_percent < 50.0);
        assert!(!score.is_complete());
    }

    #[test]
    fn test_filled_doc_scores_high() {
        let score = score_content(FILLED_DOC);
        assert_eq!(score.placeholder_count, 0);
        assert!(score.completeness_percent >= 70.0);
        assert!(score.content_lines >= 5);
    }

    #[test]
    fn test_placeholder_counting() {
        assert_eq!(count_placeholders("{one} and {two}"), 2);
        assert_eq!(count_placeholders("no placeholders here"), 0);
        assert_eq!(count_placeholders("{nested {inner}}"), 1);
    }

    #[test]
    fn test_strip_frontmatter() {
        let doc = "---\nid: test\n---\n## Section\ncontent";
        let body = strip_frontmatter(doc);
        assert!(!body.contains("id: test"));
        assert!(body.contains("## Section"));
    }
}
