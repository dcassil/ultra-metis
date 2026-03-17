//! Template Registry — centralized, type-safe access to all document templates.
//!
//! Provides a single entry point for looking up templates by [`DocumentType`],
//! rendering content with Tera, and supporting context-aware rendering with
//! parent document data. Also supports project-level template overrides from
//! a `.metis/templates/` directory.

use super::documents::types::DocumentType;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

/// Categories of templates that each document type provides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateCategory {
    /// YAML frontmatter template
    Frontmatter,
    /// Markdown content body template
    Content,
    /// Acceptance criteria checklist template
    AcceptanceCriteria,
}

impl fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateCategory::Frontmatter => write!(f, "frontmatter.yaml"),
            TemplateCategory::Content => write!(f, "content.md"),
            TemplateCategory::AcceptanceCriteria => write!(f, "acceptance_criteria.md"),
        }
    }
}

/// A complete set of templates for a single document type.
#[derive(Debug, Clone)]
pub struct TemplateSet {
    pub frontmatter: String,
    pub content: String,
    pub acceptance_criteria: String,
}

impl TemplateSet {
    /// Get a template by category.
    pub fn get(&self, category: TemplateCategory) -> &str {
        match category {
            TemplateCategory::Frontmatter => &self.frontmatter,
            TemplateCategory::Content => &self.content,
            TemplateCategory::AcceptanceCriteria => &self.acceptance_criteria,
        }
    }
}

/// Context data for context-aware template rendering.
///
/// Provides optional parent document information and project configuration
/// that templates can reference using Tera's `default()` filter.
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    /// Title of the parent document (e.g., the Epic title for a Story)
    pub parent_title: Option<String>,
    /// Short code of the parent document (e.g., "PROJ-E-0001")
    pub parent_short_code: Option<String>,
    /// Type of the parent document (e.g., "epic")
    pub parent_type: Option<String>,
    /// Project name from configuration
    pub project_name: Option<String>,
    /// Detected languages in the project
    pub detected_languages: Vec<String>,
    /// Additional key-value pairs for template rendering
    pub extra: HashMap<String, String>,
}

impl TemplateContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_parent(mut self, title: &str, short_code: &str, doc_type: &str) -> Self {
        self.parent_title = Some(title.to_string());
        self.parent_short_code = Some(short_code.to_string());
        self.parent_type = Some(doc_type.to_string());
        self
    }

    pub fn with_project_name(mut self, name: &str) -> Self {
        self.project_name = Some(name.to_string());
        self
    }

    /// Convert to a Tera Context, merging with an existing base context.
    pub fn to_tera_context(&self, base: &mut Context) {
        base.insert(
            "parent_title",
            &self.parent_title.as_deref().unwrap_or(""),
        );
        base.insert(
            "parent_short_code",
            &self.parent_short_code.as_deref().unwrap_or(""),
        );
        base.insert(
            "parent_type",
            &self.parent_type.as_deref().unwrap_or(""),
        );
        base.insert(
            "project_name",
            &self.project_name.as_deref().unwrap_or(""),
        );
        base.insert("detected_languages", &self.detected_languages);
        for (key, value) in &self.extra {
            base.insert(key, value);
        }
    }
}

/// Error type for template operations.
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Unknown document type for templates: {0}")]
    UnknownDocumentType(String),

    #[error("Template parse error: {0}")]
    ParseError(String),

    #[error("Template render error: {0}")]
    RenderError(String),

    #[error("Custom template load error: {0}")]
    LoadError(String),
}

/// Centralized registry for all document type templates.
///
/// Provides lookup by [`DocumentType`], rendering with Tera,
/// context-aware rendering, and support for project-level overrides.
pub struct TemplateRegistry {
    builtin: HashMap<DocumentType, TemplateSet>,
    custom_templates_dir: Option<PathBuf>,
    custom_cache: HashMap<DocumentType, TemplateSet>,
}

impl TemplateRegistry {
    /// Create a new registry with all built-in templates.
    pub fn new() -> Self {
        let mut builtin = HashMap::new();

        // Core planning types (implement Document trait)
        builtin.insert(
            DocumentType::ProductDoc,
            TemplateSet {
                frontmatter: include_str!("../documents/product_doc/frontmatter.yaml").to_string(),
                content: include_str!("../documents/product_doc/content.md").to_string(),
                acceptance_criteria: include_str!(
                    "../documents/product_doc/acceptance_criteria.md"
                )
                .to_string(),
            },
        );

        builtin.insert(
            DocumentType::DesignContext,
            TemplateSet {
                frontmatter: include_str!("../documents/design_context/frontmatter.yaml")
                    .to_string(),
                content: include_str!("../documents/design_context/content.md").to_string(),
                acceptance_criteria: include_str!(
                    "../documents/design_context/acceptance_criteria.md"
                )
                .to_string(),
            },
        );

        builtin.insert(
            DocumentType::Epic,
            TemplateSet {
                frontmatter: include_str!("../documents/epic/frontmatter.yaml").to_string(),
                content: include_str!("../documents/epic/content.md").to_string(),
                acceptance_criteria: include_str!("../documents/epic/acceptance_criteria.md")
                    .to_string(),
            },
        );

        builtin.insert(
            DocumentType::Story,
            TemplateSet {
                frontmatter: include_str!("../documents/story/frontmatter.yaml").to_string(),
                content: include_str!("../documents/story/content.md").to_string(),
                acceptance_criteria: include_str!("../documents/story/acceptance_criteria.md")
                    .to_string(),
            },
        );

        builtin.insert(
            DocumentType::Task,
            TemplateSet {
                frontmatter: include_str!("../documents/task/frontmatter.yaml").to_string(),
                content: include_str!("../documents/task/content.md").to_string(),
                acceptance_criteria: include_str!("../documents/task/acceptance_criteria.md")
                    .to_string(),
            },
        );

        builtin.insert(
            DocumentType::Adr,
            TemplateSet {
                frontmatter: include_str!("../documents/adr/frontmatter.yaml").to_string(),
                content: include_str!("../documents/adr/content.md").to_string(),
                acceptance_criteria: include_str!("../documents/adr/acceptance_criteria.md")
                    .to_string(),
            },
        );

        builtin.insert(
            DocumentType::Specification,
            TemplateSet {
                frontmatter: include_str!("../documents/specification/frontmatter.yaml")
                    .to_string(),
                content: include_str!("../documents/specification/content.md").to_string(),
                acceptance_criteria: include_str!(
                    "../documents/specification/acceptance_criteria.md"
                )
                .to_string(),
            },
        );

        builtin.insert(
            DocumentType::Vision,
            TemplateSet {
                frontmatter: include_str!("../documents/vision/frontmatter.yaml").to_string(),
                content: include_str!("../documents/vision/content.md").to_string(),
                acceptance_criteria: include_str!("../documents/vision/acceptance_criteria.md")
                    .to_string(),
            },
        );

        builtin.insert(
            DocumentType::Initiative,
            TemplateSet {
                frontmatter: include_str!("../documents/initiative/frontmatter.yaml").to_string(),
                content: include_str!("../documents/initiative/content.md").to_string(),
                acceptance_criteria: include_str!(
                    "../documents/initiative/acceptance_criteria.md"
                )
                .to_string(),
            },
        );

        Self {
            builtin,
            custom_templates_dir: None,
            custom_cache: HashMap::new(),
        }
    }

    /// Create a registry with a custom templates directory for project-level overrides.
    ///
    /// Custom templates are loaded from `{dir}/{doc_type}/content.md` etc.
    /// If a custom template exists, it takes priority over the built-in one.
    pub fn with_custom_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        let dir = dir.as_ref().to_path_buf();
        if dir.exists() {
            self.custom_templates_dir = Some(dir.clone());
            self.load_custom_templates(&dir);
        }
        self
    }

    /// Load custom templates from a directory.
    fn load_custom_templates(&mut self, dir: &Path) {
        let doc_types = [
            (DocumentType::ProductDoc, "product_doc"),
            (DocumentType::DesignContext, "design_context"),
            (DocumentType::Epic, "epic"),
            (DocumentType::Story, "story"),
            (DocumentType::Task, "task"),
            (DocumentType::Adr, "adr"),
            (DocumentType::Specification, "specification"),
            (DocumentType::Vision, "vision"),
            (DocumentType::Initiative, "initiative"),
        ];

        for (doc_type, dir_name) in &doc_types {
            let type_dir = dir.join(dir_name);
            if !type_dir.exists() {
                continue;
            }

            let builtin = self.builtin.get(doc_type).cloned().unwrap_or_else(|| {
                TemplateSet {
                    frontmatter: String::new(),
                    content: String::new(),
                    acceptance_criteria: String::new(),
                }
            });

            let frontmatter = Self::read_custom_file(&type_dir, "frontmatter.yaml")
                .unwrap_or(builtin.frontmatter);
            let content =
                Self::read_custom_file(&type_dir, "content.md").unwrap_or(builtin.content);
            let acceptance_criteria =
                Self::read_custom_file(&type_dir, "acceptance_criteria.md")
                    .unwrap_or(builtin.acceptance_criteria);

            self.custom_cache.insert(
                *doc_type,
                TemplateSet {
                    frontmatter,
                    content,
                    acceptance_criteria,
                },
            );
        }
    }

    /// Read a custom template file, returning None if it doesn't exist.
    fn read_custom_file(type_dir: &Path, filename: &str) -> Option<String> {
        let path = type_dir.join(filename);
        std::fs::read_to_string(&path).ok()
    }

    /// Get the template set for a document type.
    ///
    /// Returns custom templates if available, otherwise built-in templates.
    pub fn get(&self, doc_type: DocumentType) -> Option<&TemplateSet> {
        self.custom_cache
            .get(&doc_type)
            .or_else(|| self.builtin.get(&doc_type))
    }

    /// Get a specific template category for a document type.
    pub fn get_template(
        &self,
        doc_type: DocumentType,
        category: TemplateCategory,
    ) -> Option<&str> {
        self.get(doc_type).map(|set| set.get(category))
    }

    /// Render a content template for the given document type with a title.
    pub fn render_content(
        &self,
        doc_type: DocumentType,
        title: &str,
    ) -> Result<String, TemplateError> {
        let template_set = self
            .get(doc_type)
            .ok_or_else(|| TemplateError::UnknownDocumentType(doc_type.to_string()))?;

        let mut tera = Tera::default();
        let template_name = format!("{}_content", doc_type);
        tera.add_raw_template(&template_name, &template_set.content)
            .map_err(|e| TemplateError::ParseError(e.to_string()))?;

        let mut context = Context::new();
        context.insert("title", title);

        tera.render(&template_name, &context)
            .map_err(|e| TemplateError::RenderError(e.to_string()))
    }

    /// Render a content template with full context (parent doc data, project config).
    pub fn render_with_context(
        &self,
        doc_type: DocumentType,
        title: &str,
        template_context: &TemplateContext,
    ) -> Result<String, TemplateError> {
        let template_set = self
            .get(doc_type)
            .ok_or_else(|| TemplateError::UnknownDocumentType(doc_type.to_string()))?;

        let mut tera = Tera::default();
        let template_name = format!("{}_content", doc_type);
        tera.add_raw_template(&template_name, &template_set.content)
            .map_err(|e| TemplateError::ParseError(e.to_string()))?;

        let mut context = Context::new();
        context.insert("title", title);
        template_context.to_tera_context(&mut context);

        tera.render(&template_name, &context)
            .map_err(|e| TemplateError::RenderError(e.to_string()))
    }

    /// Validate that a custom template can be parsed by Tera.
    pub fn validate_template(template_content: &str) -> Result<(), TemplateError> {
        let mut tera = Tera::default();
        tera.add_raw_template("validation_check", template_content)
            .map_err(|e| TemplateError::ParseError(e.to_string()))?;
        Ok(())
    }

    /// Get all registered document types.
    pub fn document_types(&self) -> Vec<DocumentType> {
        let mut types: Vec<DocumentType> = self.builtin.keys().copied().collect();
        types.sort_by_key(|t| t.to_string());
        types
    }

    /// Check if a document type has a custom override.
    pub fn has_custom_override(&self, doc_type: DocumentType) -> bool {
        self.custom_cache.contains_key(&doc_type)
    }

    /// Get the custom templates directory path, if configured.
    pub fn custom_dir(&self) -> Option<&Path> {
        self.custom_templates_dir.as_deref()
    }

    /// Validate all custom templates in a directory.
    ///
    /// Returns a list of (doc_type, category, error) for any templates that fail Tera parsing.
    pub fn validate_custom_dir<P: AsRef<Path>>(
        dir: P,
    ) -> Vec<(String, TemplateCategory, TemplateError)> {
        let dir = dir.as_ref();
        let mut errors = Vec::new();

        let doc_type_dirs = [
            "product_doc",
            "design_context",
            "epic",
            "story",
            "task",
            "adr",
            "specification",
            "vision",
            "initiative",
        ];

        let categories = [
            (TemplateCategory::Frontmatter, "frontmatter.yaml"),
            (TemplateCategory::Content, "content.md"),
            (TemplateCategory::AcceptanceCriteria, "acceptance_criteria.md"),
        ];

        for doc_type_dir in &doc_type_dirs {
            let type_dir = dir.join(doc_type_dir);
            if !type_dir.exists() {
                continue;
            }

            for (category, filename) in &categories {
                let file_path = type_dir.join(filename);
                if !file_path.exists() {
                    continue;
                }

                match std::fs::read_to_string(&file_path) {
                    Ok(content) => {
                        if let Err(e) = Self::validate_template(&content) {
                            errors.push((doc_type_dir.to_string(), *category, e));
                        }
                    }
                    Err(e) => {
                        errors.push((
                            doc_type_dir.to_string(),
                            *category,
                            TemplateError::LoadError(e.to_string()),
                        ));
                    }
                }
            }
        }

        errors
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_all_document_types() {
        let registry = TemplateRegistry::new();
        let expected_types = vec![
            DocumentType::ProductDoc,
            DocumentType::DesignContext,
            DocumentType::Epic,
            DocumentType::Story,
            DocumentType::Task,
            DocumentType::Adr,
            DocumentType::Specification,
            DocumentType::Vision,
            DocumentType::Initiative,
        ];

        for doc_type in &expected_types {
            assert!(
                registry.get(*doc_type).is_some(),
                "Missing templates for {:?}",
                doc_type
            );
        }
    }

    #[test]
    fn test_template_set_has_all_categories() {
        let registry = TemplateRegistry::new();

        for doc_type in registry.document_types() {
            let set = registry.get(doc_type).unwrap();
            assert!(!set.frontmatter.is_empty(), "Empty frontmatter for {}", doc_type);
            assert!(!set.content.is_empty(), "Empty content for {}", doc_type);
            assert!(
                !set.acceptance_criteria.is_empty(),
                "Empty acceptance_criteria for {}",
                doc_type
            );
        }
    }

    #[test]
    fn test_get_template_by_category() {
        let registry = TemplateRegistry::new();

        let content = registry
            .get_template(DocumentType::Epic, TemplateCategory::Content)
            .unwrap();
        assert!(content.contains("{{ title }}"));

        let frontmatter = registry
            .get_template(DocumentType::Epic, TemplateCategory::Frontmatter)
            .unwrap();
        assert!(frontmatter.contains("level: epic"));
    }

    #[test]
    fn test_render_content_basic() {
        let registry = TemplateRegistry::new();

        let rendered = registry
            .render_content(DocumentType::Epic, "My Test Epic")
            .unwrap();
        assert!(rendered.contains("# My Test Epic"));
        assert!(rendered.contains("## Context"));
    }

    #[test]
    fn test_render_content_all_types() {
        let registry = TemplateRegistry::new();

        for doc_type in registry.document_types() {
            let result = registry.render_content(doc_type, "Test Title");
            assert!(
                result.is_ok(),
                "Failed to render content for {}: {:?}",
                doc_type,
                result.err()
            );
            let rendered = result.unwrap();
            assert!(
                rendered.contains("# Test Title"),
                "Rendered content for {} missing title heading",
                doc_type
            );
        }
    }

    #[test]
    fn test_render_with_context() {
        let registry = TemplateRegistry::new();

        let ctx = TemplateContext::new()
            .with_parent("Parent Epic", "PROJ-E-0001", "epic")
            .with_project_name("my-project");

        let rendered = registry
            .render_with_context(DocumentType::Task, "My Task", &ctx)
            .unwrap();
        assert!(rendered.contains("# My Task"));
    }

    #[test]
    fn test_render_with_empty_context() {
        let registry = TemplateRegistry::new();

        let ctx = TemplateContext::new();

        let rendered = registry
            .render_with_context(DocumentType::Story, "My Story", &ctx)
            .unwrap();
        assert!(rendered.contains("# My Story"));
    }

    #[test]
    fn test_template_category_display() {
        assert_eq!(TemplateCategory::Frontmatter.to_string(), "frontmatter.yaml");
        assert_eq!(TemplateCategory::Content.to_string(), "content.md");
        assert_eq!(
            TemplateCategory::AcceptanceCriteria.to_string(),
            "acceptance_criteria.md"
        );
    }

    #[test]
    fn test_validate_template_valid() {
        assert!(TemplateRegistry::validate_template("# {{ title }}\n\nSome content").is_ok());
    }

    #[test]
    fn test_validate_template_invalid() {
        // Unclosed tag
        assert!(TemplateRegistry::validate_template("# {{ title }\n").is_err());
    }

    #[test]
    fn test_custom_template_override() {
        let temp_dir = tempfile::tempdir().unwrap();
        let epic_dir = temp_dir.path().join("epic");
        std::fs::create_dir_all(&epic_dir).unwrap();
        std::fs::write(
            epic_dir.join("content.md"),
            "# {{ title }}\n\n## Custom Section\n\nCustom epic template",
        )
        .unwrap();

        let registry = TemplateRegistry::new().with_custom_dir(temp_dir.path());

        assert!(registry.has_custom_override(DocumentType::Epic));
        assert!(!registry.has_custom_override(DocumentType::Task));

        let content = registry
            .get_template(DocumentType::Epic, TemplateCategory::Content)
            .unwrap();
        assert!(content.contains("Custom Section"));

        // Built-in frontmatter should still be available since we didn't override it
        let fm = registry
            .get_template(DocumentType::Epic, TemplateCategory::Frontmatter)
            .unwrap();
        assert!(fm.contains("level: epic"));
    }

    #[test]
    fn test_custom_dir_nonexistent() {
        let registry = TemplateRegistry::new().with_custom_dir("/nonexistent/path");
        // Should gracefully fall back to builtins
        assert!(registry.get(DocumentType::Epic).is_some());
        assert!(!registry.has_custom_override(DocumentType::Epic));
    }

    #[test]
    fn test_render_custom_template() {
        let temp_dir = tempfile::tempdir().unwrap();
        let task_dir = temp_dir.path().join("task");
        std::fs::create_dir_all(&task_dir).unwrap();
        std::fs::write(
            task_dir.join("content.md"),
            "# {{ title }}\n\n## Custom Task Section\n\nThis is a custom task template.",
        )
        .unwrap();

        let registry = TemplateRegistry::new().with_custom_dir(temp_dir.path());

        let rendered = registry
            .render_content(DocumentType::Task, "Custom Task")
            .unwrap();
        assert!(rendered.contains("# Custom Task"));
        assert!(rendered.contains("Custom Task Section"));
    }

    #[test]
    fn test_template_context_builder() {
        let ctx = TemplateContext::new()
            .with_parent("My Epic", "PROJ-E-0001", "epic")
            .with_project_name("ultra-metis");

        assert_eq!(ctx.parent_title.as_deref(), Some("My Epic"));
        assert_eq!(ctx.parent_short_code.as_deref(), Some("PROJ-E-0001"));
        assert_eq!(ctx.parent_type.as_deref(), Some("epic"));
        assert_eq!(ctx.project_name.as_deref(), Some("ultra-metis"));
    }

    #[test]
    fn test_document_types_returned() {
        let registry = TemplateRegistry::new();
        let types = registry.document_types();
        assert_eq!(types.len(), 9);
    }

    #[test]
    fn test_task_context_aware_rendering_with_parent() {
        let registry = TemplateRegistry::new();

        let ctx = TemplateContext::new()
            .with_parent("Improve API Performance", "PROJ-E-0001", "epic");

        let rendered = registry
            .render_with_context(DocumentType::Task, "Optimize Database Queries", &ctx)
            .unwrap();
        assert!(rendered.contains("# Optimize Database Queries"));
        assert!(rendered.contains("Improve API Performance"));
        assert!(rendered.contains("PROJ-E-0001"));
    }

    #[test]
    fn test_task_context_aware_rendering_without_parent() {
        let registry = TemplateRegistry::new();

        let ctx = TemplateContext::new();

        let rendered = registry
            .render_with_context(DocumentType::Task, "Standalone Task", &ctx)
            .unwrap();
        assert!(rendered.contains("# Standalone Task"));
        // Should not contain parent section when no parent is provided
        assert!(!rendered.contains("**Parent**:"));
    }

    #[test]
    fn test_story_context_aware_rendering_with_parent() {
        let registry = TemplateRegistry::new();

        let ctx = TemplateContext::new()
            .with_parent("User Auth Epic", "PROJ-E-0005", "epic");

        let rendered = registry
            .render_with_context(DocumentType::Story, "Add OAuth Login", &ctx)
            .unwrap();
        assert!(rendered.contains("# Add OAuth Login"));
        assert!(rendered.contains("User Auth Epic"));
        assert!(rendered.contains("PROJ-E-0005"));
    }

    #[test]
    fn test_story_context_aware_rendering_without_parent() {
        let registry = TemplateRegistry::new();

        let ctx = TemplateContext::new();

        let rendered = registry
            .render_with_context(DocumentType::Story, "Orphan Story", &ctx)
            .unwrap();
        assert!(rendered.contains("# Orphan Story"));
        assert!(!rendered.contains("**Epic**:"));
    }

    #[test]
    fn test_context_with_extra_fields() {
        let registry = TemplateRegistry::new();

        let mut ctx = TemplateContext::new();
        ctx.extra.insert("custom_field".to_string(), "custom_value".to_string());

        // Should render without error even with extra fields not in template
        let rendered = registry
            .render_with_context(DocumentType::Epic, "Extra Fields Epic", &ctx)
            .unwrap();
        assert!(rendered.contains("# Extra Fields Epic"));
    }

    #[test]
    fn test_render_all_types_with_context() {
        let registry = TemplateRegistry::new();

        let ctx = TemplateContext::new()
            .with_parent("Parent Doc", "PROJ-PD-0001", "product_doc")
            .with_project_name("test-project");

        for doc_type in registry.document_types() {
            let result = registry.render_with_context(doc_type, "Context Test", &ctx);
            assert!(
                result.is_ok(),
                "Failed to render with context for {}: {:?}",
                doc_type,
                result.err()
            );
        }
    }

    #[test]
    fn test_validate_custom_dir_valid_templates() {
        let temp_dir = tempfile::tempdir().unwrap();
        let epic_dir = temp_dir.path().join("epic");
        std::fs::create_dir_all(&epic_dir).unwrap();
        std::fs::write(
            epic_dir.join("content.md"),
            "# {{ title }}\n\n## Custom Section\n",
        )
        .unwrap();

        let errors = TemplateRegistry::validate_custom_dir(temp_dir.path());
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors.len());
    }

    #[test]
    fn test_validate_custom_dir_invalid_template() {
        let temp_dir = tempfile::tempdir().unwrap();
        let task_dir = temp_dir.path().join("task");
        std::fs::create_dir_all(&task_dir).unwrap();
        // Invalid Tera: unclosed tag
        std::fs::write(task_dir.join("content.md"), "# {{ title }\n").unwrap();

        let errors = TemplateRegistry::validate_custom_dir(temp_dir.path());
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, "task");
        assert_eq!(errors[0].1, TemplateCategory::Content);
    }

    #[test]
    fn test_validate_custom_dir_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let errors = TemplateRegistry::validate_custom_dir(temp_dir.path());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_custom_multiple_type_overrides() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Override both epic and task
        let epic_dir = temp_dir.path().join("epic");
        std::fs::create_dir_all(&epic_dir).unwrap();
        std::fs::write(
            epic_dir.join("content.md"),
            "# {{ title }}\n\n## Custom Epic\n",
        )
        .unwrap();

        let task_dir = temp_dir.path().join("task");
        std::fs::create_dir_all(&task_dir).unwrap();
        std::fs::write(
            task_dir.join("content.md"),
            "# {{ title }}\n\n## Custom Task\n",
        )
        .unwrap();

        let registry = TemplateRegistry::new().with_custom_dir(temp_dir.path());

        assert!(registry.has_custom_override(DocumentType::Epic));
        assert!(registry.has_custom_override(DocumentType::Task));
        assert!(!registry.has_custom_override(DocumentType::Story));

        let epic_content = registry
            .get_template(DocumentType::Epic, TemplateCategory::Content)
            .unwrap();
        assert!(epic_content.contains("Custom Epic"));

        let task_content = registry
            .get_template(DocumentType::Task, TemplateCategory::Content)
            .unwrap();
        assert!(task_content.contains("Custom Task"));

        // Story should still use builtin
        let story_content = registry
            .get_template(DocumentType::Story, TemplateCategory::Content)
            .unwrap();
        assert!(story_content.contains("Objective"));
    }

    #[test]
    fn test_custom_dir_accessor() {
        let temp_dir = tempfile::tempdir().unwrap();
        let epic_dir = temp_dir.path().join("epic");
        std::fs::create_dir_all(&epic_dir).unwrap();
        std::fs::write(epic_dir.join("content.md"), "# {{ title }}").unwrap();

        let registry = TemplateRegistry::new().with_custom_dir(temp_dir.path());
        assert!(registry.custom_dir().is_some());

        let default_registry = TemplateRegistry::new();
        assert!(default_registry.custom_dir().is_none());
    }

    #[test]
    fn test_custom_override_partial_files_only() {
        let temp_dir = tempfile::tempdir().unwrap();
        let adr_dir = temp_dir.path().join("adr");
        std::fs::create_dir_all(&adr_dir).unwrap();
        // Only override acceptance_criteria, not content or frontmatter
        std::fs::write(
            adr_dir.join("acceptance_criteria.md"),
            "- [ ] Custom ADR criterion\n- [ ] Another criterion\n",
        )
        .unwrap();

        let registry = TemplateRegistry::new().with_custom_dir(temp_dir.path());

        assert!(registry.has_custom_override(DocumentType::Adr));

        // Content and frontmatter should fall back to builtin
        let content = registry
            .get_template(DocumentType::Adr, TemplateCategory::Content)
            .unwrap();
        assert!(content.contains("{{ title }}"));

        // Acceptance criteria should be custom
        let ac = registry
            .get_template(DocumentType::Adr, TemplateCategory::AcceptanceCriteria)
            .unwrap();
        assert!(ac.contains("Custom ADR criterion"));
    }

    // -------------------------------------------------------------------------
    // Template Quality Tests
    // Verify structural completeness of the four core document templates.
    // -------------------------------------------------------------------------

    fn render_core_template(doc_type: DocumentType) -> String {
        let registry = TemplateRegistry::new();
        let ctx = TemplateContext::new().with_project_name("TEST");
        registry
            .render_with_context(doc_type, "Test Document", &ctx)
            .unwrap_or_else(|e| panic!("Template render failed for {:?}: {}", doc_type, e))
    }

    #[test]
    fn quality_task_template_has_acceptance_criteria() {
        let rendered = render_core_template(DocumentType::Task);
        assert!(
            rendered.contains("## Acceptance Criteria"),
            "Task template must have an Acceptance Criteria section"
        );
    }

    #[test]
    fn quality_task_template_has_status_updates() {
        let rendered = render_core_template(DocumentType::Task);
        assert!(
            rendered.contains("## Status Updates"),
            "Task template must have a Status Updates section for working memory"
        );
    }

    #[test]
    fn quality_task_template_has_objective() {
        let rendered = render_core_template(DocumentType::Task);
        assert!(
            rendered.contains("## Objective"),
            "Task template must have an Objective section"
        );
    }

    #[test]
    fn quality_task_template_acceptance_criteria_uses_checkboxes() {
        let raw = TemplateRegistry::new()
            .get_template(DocumentType::Task, TemplateCategory::Content)
            .unwrap()
            .to_string();
        assert!(
            raw.contains("- [ ]"),
            "Task template Acceptance Criteria must use checkbox list format (- [ ])"
        );
    }

    #[test]
    fn quality_initiative_template_has_detailed_design() {
        let rendered = render_core_template(DocumentType::Initiative);
        assert!(
            rendered.contains("## Detailed Design"),
            "Initiative template must have a Detailed Design section"
        );
    }

    #[test]
    fn quality_initiative_template_has_status_updates() {
        let rendered = render_core_template(DocumentType::Initiative);
        assert!(
            rendered.contains("## Status Updates"),
            "Initiative template must have a Status Updates section"
        );
    }

    #[test]
    fn quality_adr_template_has_rationale() {
        let rendered = render_core_template(DocumentType::Adr);
        assert!(
            rendered.contains("## Rationale"),
            "ADR template must have a Rationale section explaining why the decision was made"
        );
    }

    #[test]
    fn quality_adr_template_has_structured_consequences() {
        let rendered = render_core_template(DocumentType::Adr);
        assert!(
            rendered.contains("### Positive") && rendered.contains("### Negative"),
            "ADR template Consequences must have Positive and Negative sub-sections"
        );
    }

    #[test]
    fn quality_vision_template_has_success_criteria() {
        let rendered = render_core_template(DocumentType::Vision);
        assert!(
            rendered.contains("## Success Criteria"),
            "Vision template must have a Success Criteria section"
        );
    }

    #[test]
    fn quality_all_core_templates_have_conditional_guidance() {
        let core_types = [
            DocumentType::Vision,
            DocumentType::Initiative,
            DocumentType::Task,
            DocumentType::Adr,
        ];
        let registry = TemplateRegistry::new();
        for doc_type in core_types {
            let raw = registry
                .get_template(doc_type, TemplateCategory::Content)
                .unwrap_or_else(|| panic!("Missing content template for {:?}", doc_type));
            let has_guidance = raw.contains("DELETE")
                || raw.contains("CONDITIONAL")
                || raw.contains("if not applicable")
                || raw.contains("REQUIRED");
            assert!(
                has_guidance,
                "{:?} template has no required/conditional section guidance",
                doc_type
            );
        }
    }

    #[test]
    fn quality_core_templates_have_html_comment_guidance() {
        // Templates should have inline HTML comments providing guidance to writers,
        // not just bare {placeholder} lines with no context.
        let core_types = [
            DocumentType::Vision,
            DocumentType::Initiative,
            DocumentType::Task,
            DocumentType::Adr,
        ];
        let registry = TemplateRegistry::new();
        for doc_type in core_types {
            let raw = registry
                .get_template(doc_type, TemplateCategory::Content)
                .unwrap_or_else(|| panic!("Missing content template for {:?}", doc_type));
            assert!(
                raw.contains("<!--"),
                "{:?} template has no HTML comment guidance — add <!-- ... --> comments to guide writers",
                doc_type
            );
        }
    }

    #[test]
    fn quality_all_core_templates_render_without_parent() {
        let core_types = [
            DocumentType::Vision,
            DocumentType::Initiative,
            DocumentType::Task,
            DocumentType::Adr,
        ];
        let registry = TemplateRegistry::new();
        let ctx = TemplateContext::new();
        for doc_type in core_types {
            let result = registry.render_with_context(doc_type, "Orphan Document", &ctx);
            assert!(
                result.is_ok(),
                "{:?} template must render without errors when no parent is provided: {:?}",
                doc_type,
                result.err()
            );
        }
    }

    #[test]
    fn quality_all_core_templates_render_with_parent() {
        let core_types = [
            DocumentType::Vision,
            DocumentType::Initiative,
            DocumentType::Task,
            DocumentType::Adr,
        ];
        let registry = TemplateRegistry::new();
        let ctx = TemplateContext::new()
            .with_parent("Parent Vision", "PROJ-V-0001", "vision")
            .with_project_name("test-project");
        for doc_type in core_types {
            let result = registry.render_with_context(doc_type, "Child Document", &ctx);
            assert!(
                result.is_ok(),
                "{:?} template must render without errors when parent context is provided: {:?}",
                doc_type,
                result.err()
            );
        }
    }
}
