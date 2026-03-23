use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{DocumentId, Phase, Tag};
use chrono::Utc;
use gray_matter;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tera::{Context, Tera};

/// A ConstraintRecord documents a design or operational constraint that must be respected.
/// Phases: Draft → Active → Retired
#[derive(Debug)]
pub struct ConstraintRecord {
    core: DocumentCore,
    /// Category of constraint (e.g. "technical", "business", "regulatory")
    pub constraint_type: String,
    /// What this constraint applies to (e.g. a component, service, or team)
    pub scope: String,
    /// Whether this constraint is currently being enforced
    pub enforced: bool,
}

impl ConstraintRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        parent_id: Option<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        constraint_type: String,
        scope: String,
        enforced: bool,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            parent_id,
            tags,
            archived,
            short_code,
            constraint_type,
            scope,
            enforced,
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
        constraint_type: String,
        scope: String,
        enforced: bool,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("constraint_record_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("constraint_record_content", &context)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template render error: {}", e))
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
            constraint_type,
            scope,
            enforced,
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
        constraint_type: String,
        scope: String,
        enforced: bool,
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
            constraint_type,
            scope,
            enforced,
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

        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "constraint_record" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'constraint_record', found '{}'",
                level
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

        let constraint_type =
            FrontmatterParser::extract_optional_string(&fm_map, "constraint_type")
                .unwrap_or_default();
        let scope =
            FrontmatterParser::extract_optional_string(&fm_map, "scope").unwrap_or_default();
        let enforced = FrontmatterParser::extract_bool(&fm_map, "enforced").unwrap_or(false);

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
            constraint_type,
            scope,
            enforced,
        ))
    }

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
            .map(|id| id.to_string())
            .unwrap_or_else(|| "NULL".to_string());
        context.insert("parent_id", &parent_id_str);

        context.insert("constraint_type", &self.constraint_type);
        context.insert("scope", &self.scope);
        context.insert("enforced", &self.enforced.to_string());

        let tag_strings: Vec<String> = self.core.tags.iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);

        let frontmatter = tera.render("frontmatter", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Frontmatter render error: {}", e))
        })?;

        let content_body = &self.core.content.body;
        let acceptance_criteria = if let Some(ac) = &self.core.content.acceptance_criteria {
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

    fn update_phase_tag(&mut self, new_phase: Phase) {
        self.core.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
        self.core.tags.push(Tag::Phase(new_phase));
        self.core.metadata.updated_at = Utc::now();
    }

    pub fn validate(&self) -> Result<(), DocumentValidationError> {
        if self.core.title.trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "ConstraintRecord title cannot be empty".to_string(),
            ));
        }
        if self.constraint_type.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "constraint_type".to_string(),
            ));
        }
        if self.scope.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "scope".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_constraint_record() -> ConstraintRecord {
        ConstraintRecord::new(
            "No direct database access from HTTP handlers".to_string(),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-CR-0001".to_string(),
            "technical".to_string(),
            "api-service".to_string(),
            true,
        )
        .unwrap()
    }

    #[test]
    fn test_constraint_record_creation() {
        let cr = make_constraint_record();

        assert_eq!(cr.title(), "No direct database access from HTTP handlers");
        assert_eq!(cr.phase().unwrap(), Phase::Draft);
        assert_eq!(cr.constraint_type, "technical");
        assert_eq!(cr.scope, "api-service");
        assert!(cr.enforced);
        assert!(cr.validate().is_ok());
    }

    #[test]
    fn test_constraint_record_validation_requires_type() {
        let cr = ConstraintRecord::new(
            "Some constraint".to_string(),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-CR-0002".to_string(),
            String::new(),
            "some-scope".to_string(),
            false,
        )
        .unwrap();
        assert!(cr.validate().is_err());
    }

    #[tokio::test]
    async fn test_constraint_record_roundtrip() {
        let cr = make_constraint_record();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-constraint-record.md");

        cr.to_file(&file_path).await.unwrap();
        let loaded = ConstraintRecord::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), cr.title());
        assert_eq!(loaded.phase().unwrap(), cr.phase().unwrap());
        assert_eq!(loaded.constraint_type, cr.constraint_type);
        assert_eq!(loaded.scope, cr.scope);
        assert_eq!(loaded.enforced, cr.enforced);
    }
}
