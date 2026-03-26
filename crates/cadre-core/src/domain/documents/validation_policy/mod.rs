use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{DocumentId, Phase, Tag};
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// A ValidationPolicy declares which validation checks are required for a given
/// document type or scope pattern.
/// Phases: Draft → Active → Retired
#[derive(Debug)]
pub struct ValidationPolicy {
    core: DocumentCore,
    /// Pattern describing what this policy applies to (e.g. `story:feature`, `task:*`, `epic:*`)
    pub applies_to: String,
    /// Validation checks that must pass (e.g. `["unit-test", "lint", "type-check"]`)
    pub required_validations: Vec<String>,
}

impl ValidationPolicy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        parent_id: Option<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        applies_to: String,
        required_validations: Vec<String>,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            parent_id,
            tags,
            archived,
            short_code,
            applies_to,
            required_validations,
            template_content,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_template(
        title: String,
        parent_id: Option<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        applies_to: String,
        required_validations: Vec<String>,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("validation_policy_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {e}"))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("validation_policy_content", &context)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template render error: {e}"))
            })?;

        let content = DocumentContent::new(&rendered_content);

        Ok(Self {
            core: DocumentCore {
                title,
                metadata,
                content,
                parent_id,
                blocked_by: Vec::new(),
                tags,
                archived,
                epic_id: None,
                schema_version: 1,
            },
            applies_to,
            required_validations,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        parent_id: Option<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        applies_to: String,
        required_validations: Vec<String>,
    ) -> Self {
        Self {
            core: DocumentCore {
                title,
                metadata,
                content,
                parent_id,
                blocked_by: Vec::new(),
                tags,
                archived,
                epic_id: None,
                schema_version: 1,
            },
            applies_to,
            required_validations,
        }
    }

    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to read file: {e}"))
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

        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "validation_policy" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'validation_policy', found '{level}'"
            )));
        }

        let title = FrontmatterParser::extract_string(&fm_map, "title")?;
        let archived = FrontmatterParser::extract_bool(&fm_map, "archived").unwrap_or(false);
        let created_at = FrontmatterParser::extract_datetime(&fm_map, "created_at")?;
        let updated_at = FrontmatterParser::extract_datetime(&fm_map, "updated_at")?;
        let exit_criteria_met =
            FrontmatterParser::extract_bool(&fm_map, "exit_criteria_met").unwrap_or(false);
        let tags = FrontmatterParser::extract_tags(&fm_map)?;
        let short_code = FrontmatterParser::extract_string(&fm_map, "short_code")?;

        let parent_id =
            FrontmatterParser::extract_optional_string(&fm_map, "parent_id").map(DocumentId::from);

        let applies_to =
            FrontmatterParser::extract_optional_string(&fm_map, "applies_to").unwrap_or_default();

        let required_validations =
            FrontmatterParser::extract_string_array(&fm_map, "required_validations")
                .unwrap_or_default();

        let metadata = DocumentMetadata::from_frontmatter(
            created_at,
            updated_at,
            exit_criteria_met,
            short_code,
        );
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            parent_id,
            tags,
            archived,
            applies_to,
            required_validations,
        ))
    }

    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to write file: {e}"))
        })
    }

    pub fn to_content(&self) -> Result<String, DocumentValidationError> {
        let mut tera = Tera::default();
        tera.add_raw_template("frontmatter", include_str!("frontmatter.yaml"))
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {e}"))
            })?;

        let mut context = Context::new();
        context.insert("slug", &self.id().to_string());
        context.insert("title", self.title());
        context.insert("short_code", &self.core.metadata.short_code);
        context.insert("created_at", &self.core.metadata.created_at.to_rfc3339());
        context.insert("updated_at", &self.core.metadata.updated_at.to_rfc3339());
        context.insert("archived", &self.core.archived.to_string());
        context.insert(
            "exit_criteria_met",
            &self.core.metadata.exit_criteria_met.to_string(),
        );

        let parent_id_str = self
            .core
            .parent_id
            .as_ref()
            .map(std::string::ToString::to_string)
            .unwrap_or_else(|| "NULL".to_string());
        context.insert("parent_id", &parent_id_str);

        context.insert("applies_to", &self.applies_to);
        context.insert("required_validations", &self.required_validations);

        let tag_strings: Vec<String> = self.core.tags.iter().map(super::types::Tag::to_str).collect();
        context.insert("tags", &tag_strings);

        let frontmatter = tera.render("frontmatter", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Frontmatter render error: {e}"))
        })?;

        let content_body = &self.core.content.body;
        let acceptance_criteria = if let Some(ac) = &self.core.content.acceptance_criteria {
            format!("\n\n## Acceptance Criteria\n\n{ac}")
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

    // Convenience accessors

    pub fn id(&self) -> DocumentId {
        DocumentId::from_title(&self.core.title)
    }

    pub fn title(&self) -> &str {
        &self.core.title
    }

    pub fn tags(&self) -> &[Tag] {
        &self.core.tags
    }

    pub fn archived(&self) -> bool {
        self.core.archived
    }

    pub fn phase(&self) -> Result<Phase, DocumentValidationError> {
        for tag in &self.core.tags {
            if let Tag::Phase(phase) = tag {
                return Ok(*phase);
            }
        }
        Err(DocumentValidationError::MissingPhaseTag)
    }

    pub fn validate(&self) -> Result<(), DocumentValidationError> {
        if self.core.title.trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "ValidationPolicy title cannot be empty".to_string(),
            ));
        }
        if self.applies_to.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "applies_to".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_validation_policy() -> ValidationPolicy {
        ValidationPolicy::new(
            "Feature story validation policy".to_string(),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-VP-0001".to_string(),
            "story:feature".to_string(),
            vec![
                "unit-test".to_string(),
                "lint".to_string(),
                "type-check".to_string(),
            ],
        )
        .unwrap()
    }

    #[test]
    fn test_validation_policy_creation() {
        let vp = make_validation_policy();

        assert_eq!(vp.title(), "Feature story validation policy");
        assert_eq!(vp.phase().unwrap(), Phase::Draft);
        assert_eq!(vp.applies_to, "story:feature");
        assert_eq!(
            vp.required_validations,
            vec!["unit-test", "lint", "type-check"]
        );
        assert!(vp.validate().is_ok());
    }

    #[test]
    fn test_validation_policy_requires_applies_to() {
        let vp = ValidationPolicy::new(
            "Some policy".to_string(),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-VP-0002".to_string(),
            String::new(),
            vec!["lint".to_string()],
        )
        .unwrap();
        assert!(vp.validate().is_err());
    }

    #[test]
    fn test_validation_policy_empty_required_validations() {
        let vp = ValidationPolicy::new(
            "Minimal policy".to_string(),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-VP-0003".to_string(),
            "task:*".to_string(),
            vec![],
        )
        .unwrap();

        assert!(vp.required_validations.is_empty());
        assert!(vp.validate().is_ok());
    }

    #[tokio::test]
    async fn test_validation_policy_roundtrip() {
        let vp = make_validation_policy();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-validation-policy.md");

        vp.to_file(&file_path).await.unwrap();
        let loaded = ValidationPolicy::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), vp.title());
        assert_eq!(loaded.phase().unwrap(), vp.phase().unwrap());
        assert_eq!(loaded.applies_to, vp.applies_to);
        assert_eq!(loaded.required_validations, vp.required_validations);
    }
}
