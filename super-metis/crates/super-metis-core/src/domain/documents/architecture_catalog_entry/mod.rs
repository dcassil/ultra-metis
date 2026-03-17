use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{Phase, Tag};
use chrono::Utc;
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// Extract a string array from frontmatter, returning empty vec for null/missing.
fn extract_string_array_or_empty(
    map: &std::collections::HashMap<String, gray_matter::Pod>,
    key: &str,
) -> Vec<String> {
    match map.get(key) {
        Some(gray_matter::Pod::Array(arr)) => {
            arr.iter()
                .filter_map(|item| {
                    if let gray_matter::Pod::String(s) = item {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

/// A reusable architecture pattern in the catalog.
///
/// Represents well-known architecture patterns (e.g., "Rust CLI with workspace",
/// "Next.js monorepo with Turborepo") that can be selected during repo setup.
/// Phases: Draft -> Review -> Published
#[derive(Debug)]
pub struct ArchitectureCatalogEntry {
    core: DocumentCore,
    pub language: String,
    pub project_type: String,
    pub folder_layout: Vec<String>,
    pub layers: Vec<String>,
    pub module_boundaries: Vec<String>,
    pub dependency_rules: Vec<String>,
    pub naming_conventions: Vec<String>,
    pub anti_patterns: Vec<String>,
    pub rules_seed_hints: Vec<String>,
    pub analysis_expectations: Vec<String>,
}

impl ArchitectureCatalogEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        language: String,
        project_type: String,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            tags,
            archived,
            short_code,
            language,
            project_type,
            template_content,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_template(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        language: String,
        project_type: String,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("architecture_catalog_entry_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("architecture_catalog_entry_content", &context)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template render error: {}", e))
            })?;

        let content = DocumentContent::new(&rendered_content);

        Ok(Self {
            core: DocumentCore {
                title,
                metadata,
                content,
                parent_id: None,
                blocked_by: Vec::new(),
                tags,
                archived,
                epic_id: None,
                schema_version: 1,
            },
            language,
            project_type,
            folder_layout: Vec::new(),
            layers: Vec::new(),
            module_boundaries: Vec::new(),
            dependency_rules: Vec::new(),
            naming_conventions: Vec::new(),
            anti_patterns: Vec::new(),
            rules_seed_hints: Vec::new(),
            analysis_expectations: Vec::new(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        language: String,
        project_type: String,
        folder_layout: Vec<String>,
        layers: Vec<String>,
        module_boundaries: Vec<String>,
        dependency_rules: Vec<String>,
        naming_conventions: Vec<String>,
        anti_patterns: Vec<String>,
        rules_seed_hints: Vec<String>,
        analysis_expectations: Vec<String>,
    ) -> Self {
        Self {
            core: DocumentCore {
                title,
                metadata,
                content,
                parent_id: None,
                blocked_by: Vec::new(),
                tags,
                archived,
                epic_id: None,
                schema_version: 1,
            },
            language,
            project_type,
            folder_layout,
            layers,
            module_boundaries,
            dependency_rules,
            naming_conventions,
            anti_patterns,
            rules_seed_hints,
            analysis_expectations,
        }
    }

    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e))
        })?;
        Self::from_content(&raw_content)
    }

    pub fn from_content(raw_content: &str) -> Result<Self, DocumentValidationError> {
        let parsed = gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(raw_content);

        let frontmatter = parsed.data.ok_or_else(|| {
            DocumentValidationError::MissingRequiredField("frontmatter".to_string())
        })?;

        let fm_map = match frontmatter {
            gray_matter::Pod::Hash(map) => map,
            _ => {
                return Err(DocumentValidationError::InvalidContent(
                    "Frontmatter must be a hash/map".to_string(),
                ))
            }
        };

        let title = FrontmatterParser::extract_string(&fm_map, "title")?;
        let archived = FrontmatterParser::extract_bool(&fm_map, "archived").unwrap_or(false);
        let created_at = FrontmatterParser::extract_datetime(&fm_map, "created_at")?;
        let updated_at = FrontmatterParser::extract_datetime(&fm_map, "updated_at")?;
        let exit_criteria_met =
            FrontmatterParser::extract_bool(&fm_map, "exit_criteria_met").unwrap_or(false);
        let tags = FrontmatterParser::extract_tags(&fm_map)?;

        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "architecture_catalog_entry" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'architecture_catalog_entry', found '{}'",
                level
            )));
        }

        let short_code = FrontmatterParser::extract_string(&fm_map, "short_code")?;
        let metadata =
            DocumentMetadata::from_frontmatter(created_at, updated_at, exit_criteria_met, short_code);
        let content = DocumentContent::from_markdown(&parsed.content);

        let language = FrontmatterParser::extract_string(&fm_map, "language")?;
        let project_type = FrontmatterParser::extract_string(&fm_map, "project_type")?;
        let folder_layout = extract_string_array_or_empty(&fm_map, "folder_layout");
        let layers = extract_string_array_or_empty(&fm_map, "layers");
        let module_boundaries = extract_string_array_or_empty(&fm_map, "module_boundaries");
        let dependency_rules = extract_string_array_or_empty(&fm_map, "dependency_rules");
        let naming_conventions = extract_string_array_or_empty(&fm_map, "naming_conventions");
        let anti_patterns = extract_string_array_or_empty(&fm_map, "anti_patterns");
        let rules_seed_hints = extract_string_array_or_empty(&fm_map, "rules_seed_hints");
        let analysis_expectations = extract_string_array_or_empty(&fm_map, "analysis_expectations");

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            tags,
            archived,
            language,
            project_type,
            folder_layout,
            layers,
            module_boundaries,
            dependency_rules,
            naming_conventions,
            anti_patterns,
            rules_seed_hints,
            analysis_expectations,
        ))
    }

    // --- accessors ---

    pub fn title(&self) -> &str {
        &self.core.title
    }

    pub fn metadata(&self) -> &DocumentMetadata {
        &self.core.metadata
    }

    pub fn content(&self) -> &DocumentContent {
        &self.core.content
    }

    pub fn tags(&self) -> &[Tag] {
        &self.core.tags
    }

    pub fn archived(&self) -> bool {
        self.core.archived
    }

    pub fn phase(&self) -> Result<Phase, DocumentValidationError> {
        for tag in self.tags() {
            if let Tag::Phase(phase) = tag {
                return Ok(*phase);
            }
        }
        Err(DocumentValidationError::MissingPhaseTag)
    }

    // --- phase management ---

    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        match current {
            Phase::Draft => Some(Phase::Review),
            Phase::Review => Some(Phase::Published),
            Phase::Published => None,
            _ => None,
        }
    }

    fn valid_transitions_from(current: Phase) -> Vec<Phase> {
        match current {
            Phase::Draft => vec![Phase::Review],
            Phase::Review => vec![Phase::Published],
            _ => vec![],
        }
    }

    pub fn can_transition_to(&self, phase: Phase) -> bool {
        if let Ok(current_phase) = self.phase() {
            Self::valid_transitions_from(current_phase).contains(&phase)
        } else {
            false
        }
    }

    fn update_phase_tag(&mut self, new_phase: Phase) {
        self.core.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
        self.core.tags.push(Tag::Phase(new_phase));
        self.core.metadata.updated_at = Utc::now();
    }

    pub fn transition_phase(
        &mut self,
        target_phase: Option<Phase>,
    ) -> Result<Phase, DocumentValidationError> {
        let current_phase = self.phase()?;
        let new_phase = match target_phase {
            Some(phase) => {
                if !self.can_transition_to(phase) {
                    return Err(DocumentValidationError::InvalidPhaseTransition {
                        from: current_phase,
                        to: phase,
                    });
                }
                phase
            }
            None => match Self::next_phase_in_sequence(current_phase) {
                Some(next) => next,
                None => return Ok(current_phase),
            },
        };
        self.update_phase_tag(new_phase);
        Ok(new_phase)
    }

    // --- serialisation ---

    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to write file: {}", e))
        })
    }

    pub fn to_content(&self) -> Result<String, DocumentValidationError> {
        let mut tera = Tera::default();
        tera.add_raw_template("frontmatter", include_str!("frontmatter.yaml"))
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("slug", &self.core.metadata.short_code);
        context.insert("title", self.title());
        context.insert("short_code", &self.core.metadata.short_code);
        context.insert("created_at", &self.core.metadata.created_at.to_rfc3339());
        context.insert("updated_at", &self.core.metadata.updated_at.to_rfc3339());
        context.insert("archived", &self.archived().to_string());
        context.insert(
            "exit_criteria_met",
            &self.core.metadata.exit_criteria_met.to_string(),
        );

        let tag_strings: Vec<String> = self.tags().iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);
        context.insert("epic_id", "NULL");
        context.insert("language", &self.language);
        context.insert("project_type", &self.project_type);
        context.insert("folder_layout", &self.folder_layout);
        context.insert("layers", &self.layers);
        context.insert("module_boundaries", &self.module_boundaries);
        context.insert("dependency_rules", &self.dependency_rules);
        context.insert("naming_conventions", &self.naming_conventions);
        context.insert("anti_patterns", &self.anti_patterns);
        context.insert("rules_seed_hints", &self.rules_seed_hints);
        context.insert("analysis_expectations", &self.analysis_expectations);

        let frontmatter = tera.render("frontmatter", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Frontmatter render error: {}", e))
        })?;

        let content_body = &self.content().body;
        let acceptance_criteria = if let Some(ac) = &self.content().acceptance_criteria {
            format!("\n\n## Acceptance Criteria\n\n{}", ac)
        } else {
            String::new()
        };

        Ok(format!(
            "---\n{}\n---\n\n{}{}",
            frontmatter.trim_end(),
            content_body,
            acceptance_criteria
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_catalog_entry(title: &str, short_code: &str) -> ArchitectureCatalogEntry {
        ArchitectureCatalogEntry::new(
            title.to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            short_code.to_string(),
            "rust".to_string(),
            "cli".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn test_catalog_entry_creation() {
        let entry = make_catalog_entry("Rust CLI Pattern", "AC-0001");

        assert_eq!(entry.title(), "Rust CLI Pattern");
        assert_eq!(entry.phase().unwrap(), Phase::Draft);
        assert!(!entry.archived());
        assert_eq!(entry.language, "rust");
        assert_eq!(entry.project_type, "cli");
        assert!(entry.folder_layout.is_empty());
        assert!(entry.layers.is_empty());
        assert!(entry.dependency_rules.is_empty());
    }

    #[test]
    fn test_catalog_entry_with_fields() {
        let entry = ArchitectureCatalogEntry::from_parts(
            "Next.js App".to_string(),
            DocumentMetadata::new("AC-0002".to_string()),
            DocumentContent::new("# Next.js App"),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "typescript".to_string(),
            "web-app".to_string(),
            vec!["src/".to_string(), "src/components/".to_string()],
            vec!["presentation".to_string(), "domain".to_string()],
            vec!["components".to_string(), "hooks".to_string()],
            vec!["no circular imports".to_string()],
            vec!["PascalCase for components".to_string()],
            vec!["god components".to_string()],
            vec!["enforce-layer-boundaries".to_string()],
            vec!["lint-clean".to_string(), "type-check-clean".to_string()],
        );

        assert_eq!(entry.language, "typescript");
        assert_eq!(entry.project_type, "web-app");
        assert_eq!(entry.folder_layout.len(), 2);
        assert_eq!(entry.layers.len(), 2);
        assert_eq!(entry.module_boundaries.len(), 2);
        assert_eq!(entry.dependency_rules.len(), 1);
        assert_eq!(entry.naming_conventions.len(), 1);
        assert_eq!(entry.anti_patterns.len(), 1);
        assert_eq!(entry.rules_seed_hints.len(), 1);
        assert_eq!(entry.analysis_expectations.len(), 2);
    }

    #[test]
    fn test_catalog_entry_content_roundtrip() {
        let entry = ArchitectureCatalogEntry::from_parts(
            "Rust Workspace".to_string(),
            DocumentMetadata::new("AC-0003".to_string()),
            DocumentContent::new("# Rust Workspace\n\nA Rust workspace pattern."),
            vec![
                Tag::Label("architecture_catalog_entry".to_string()),
                Tag::Phase(Phase::Draft),
            ],
            false,
            "rust".to_string(),
            "workspace".to_string(),
            vec!["crates/".to_string(), "src/".to_string()],
            vec!["core".to_string(), "api".to_string()],
            vec!["crate boundaries".to_string()],
            vec!["no cross-crate internal imports".to_string()],
            vec!["snake_case modules".to_string()],
            vec!["single mega-crate".to_string()],
            vec!["enforce-crate-boundaries".to_string()],
            vec!["cargo-clippy-clean".to_string()],
        );

        let serialized = entry.to_content().unwrap();
        let loaded = ArchitectureCatalogEntry::from_content(&serialized).unwrap();

        assert_eq!(loaded.title(), entry.title());
        assert_eq!(loaded.phase().unwrap(), entry.phase().unwrap());
        assert_eq!(loaded.language, entry.language);
        assert_eq!(loaded.project_type, entry.project_type);
        assert_eq!(loaded.folder_layout, entry.folder_layout);
        assert_eq!(loaded.layers, entry.layers);
        assert_eq!(loaded.module_boundaries, entry.module_boundaries);
        assert_eq!(loaded.dependency_rules, entry.dependency_rules);
        assert_eq!(loaded.naming_conventions, entry.naming_conventions);
        assert_eq!(loaded.anti_patterns, entry.anti_patterns);
        assert_eq!(loaded.rules_seed_hints, entry.rules_seed_hints);
        assert_eq!(loaded.analysis_expectations, entry.analysis_expectations);
    }

    #[tokio::test]
    async fn test_catalog_entry_file_roundtrip() {
        let entry = ArchitectureCatalogEntry::from_parts(
            "File Roundtrip Test".to_string(),
            DocumentMetadata::new("AC-0004".to_string()),
            DocumentContent::new("# File Roundtrip Test"),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "javascript".to_string(),
            "library".to_string(),
            vec!["src/".to_string(), "dist/".to_string()],
            vec![],
            vec![],
            vec!["no default exports".to_string()],
            vec![],
            vec![],
            vec![],
            vec![],
        );

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-catalog-entry.md");

        entry.to_file(&file_path).await.unwrap();
        let loaded = ArchitectureCatalogEntry::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), entry.title());
        assert_eq!(loaded.language, entry.language);
        assert_eq!(loaded.project_type, entry.project_type);
        assert_eq!(loaded.folder_layout, entry.folder_layout);
        assert_eq!(loaded.dependency_rules, entry.dependency_rules);
    }

    #[test]
    fn test_catalog_entry_transitions() {
        let mut entry = make_catalog_entry("Transition Test", "AC-0005");

        assert!(entry.can_transition_to(Phase::Review));
        assert!(!entry.can_transition_to(Phase::Published));

        let new_phase = entry.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Review);

        let new_phase = entry.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Published);

        // Terminal
        let new_phase = entry.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Published);
    }

    #[test]
    fn test_catalog_entry_invalid_transition() {
        let mut entry = make_catalog_entry("Invalid Transition", "AC-0006");

        let err = entry.transition_phase(Some(Phase::Published)).unwrap_err();
        assert!(matches!(
            err,
            DocumentValidationError::InvalidPhaseTransition {
                from: Phase::Draft,
                to: Phase::Published
            }
        ));
    }

    #[test]
    fn test_from_content_invalid_level() {
        let bad_content = "---\n\
id: test\n\
level: rules_config\n\
title: \"Bad Level\"\n\
short_code: \"AC-0099\"\n\
created_at: 2026-01-01T00:00:00Z\n\
updated_at: 2026-01-01T00:00:00Z\n\
archived: false\n\
tags:\n\
  - \"#phase/draft\"\n\
exit_criteria_met: false\n\
schema_version: 1\n\
epic_id: NULL\n\
language: \"rust\"\n\
project_type: \"cli\"\n\
folder_layout: []\n\
layers: []\n\
module_boundaries: []\n\
dependency_rules: []\n\
naming_conventions: []\n\
anti_patterns: []\n\
rules_seed_hints: []\n\
analysis_expectations: []\n\
---\n\
\n\
# Bad Level\n";
        let err = ArchitectureCatalogEntry::from_content(bad_content).unwrap_err();
        assert!(matches!(err, DocumentValidationError::InvalidContent(_)));
    }

    #[test]
    fn test_catalog_entry_empty_arrays() {
        let entry = make_catalog_entry("Empty Arrays", "AC-0007");

        let serialized = entry.to_content().unwrap();
        let loaded = ArchitectureCatalogEntry::from_content(&serialized).unwrap();

        assert!(loaded.folder_layout.is_empty());
        assert!(loaded.layers.is_empty());
        assert!(loaded.module_boundaries.is_empty());
        assert!(loaded.dependency_rules.is_empty());
        assert!(loaded.naming_conventions.is_empty());
        assert!(loaded.anti_patterns.is_empty());
        assert!(loaded.rules_seed_hints.is_empty());
        assert!(loaded.analysis_expectations.is_empty());
    }
}
