use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{DocumentId, Phase, Tag};
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// An ApprovalRecord captures a formal approval decision for an artifact.
/// Phases: Draft → Approved / Rejected
#[derive(Debug)]
pub struct ApprovalRecord {
    core: DocumentCore,
    /// Short code of the artifact that was approved
    pub approved_artifact: String,
    /// Name of the approver or approving body
    pub approver: String,
    /// ISO date of the approval (e.g. "2026-03-16")
    pub approval_date: String,
    /// Category of approval (e.g. "rule-change", "phase-transition", "design-change")
    pub approval_type: String,
}

impl ApprovalRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        parent_id: Option<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        approved_artifact: String,
        approver: String,
        approval_date: String,
        approval_type: String,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            parent_id,
            tags,
            archived,
            short_code,
            approved_artifact,
            approver,
            approval_date,
            approval_type,
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
        approved_artifact: String,
        approver: String,
        approval_date: String,
        approval_type: String,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("approval_record_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("approval_record_content", &context)
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
            approved_artifact,
            approver,
            approval_date,
            approval_type,
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
        approved_artifact: String,
        approver: String,
        approval_date: String,
        approval_type: String,
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
            approved_artifact,
            approver,
            approval_date,
            approval_type,
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
        if level != "approval_record" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'approval_record', found '{}'",
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

        let approved_artifact =
            FrontmatterParser::extract_optional_string(&fm_map, "approved_artifact")
                .unwrap_or_default();
        let approver =
            FrontmatterParser::extract_optional_string(&fm_map, "approver").unwrap_or_default();
        let approval_date = FrontmatterParser::extract_optional_string(&fm_map, "approval_date")
            .unwrap_or_default();
        let approval_type = FrontmatterParser::extract_optional_string(&fm_map, "approval_type")
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
            approved_artifact,
            approver,
            approval_date,
            approval_type,
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

        context.insert("approved_artifact", &self.approved_artifact);
        context.insert("approver", &self.approver);
        context.insert("approval_date", &self.approval_date);
        context.insert("approval_type", &self.approval_type);

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

    pub fn validate(&self) -> Result<(), DocumentValidationError> {
        if self.core.title.trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "ApprovalRecord title cannot be empty".to_string(),
            ));
        }
        if self.approved_artifact.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "approved_artifact".to_string(),
            ));
        }
        if self.approver.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "approver".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_approval_record() -> ApprovalRecord {
        ApprovalRecord::new(
            "Approve phase transition for Epic E-0001".to_string(),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-AR-0001".to_string(),
            "E-0001".to_string(),
            "Architecture Board".to_string(),
            "2026-03-16".to_string(),
            "phase-transition".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn test_approval_record_creation() {
        let ar = make_approval_record();

        assert_eq!(ar.title(), "Approve phase transition for Epic E-0001");
        assert_eq!(ar.phase().unwrap(), Phase::Draft);
        assert_eq!(ar.approved_artifact, "E-0001");
        assert_eq!(ar.approver, "Architecture Board");
        assert_eq!(ar.approval_date, "2026-03-16");
        assert_eq!(ar.approval_type, "phase-transition");
        assert!(ar.validate().is_ok());
    }

    #[test]
    fn test_approval_record_validation_requires_artifact() {
        let ar = ApprovalRecord::new(
            "Some approval".to_string(),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-AR-0002".to_string(),
            String::new(),
            "Approver".to_string(),
            "2026-03-16".to_string(),
            "rule-change".to_string(),
        )
        .unwrap();
        assert!(ar.validate().is_err());
    }

    #[tokio::test]
    async fn test_approval_record_roundtrip() {
        let ar = make_approval_record();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-approval-record.md");

        ar.to_file(&file_path).await.unwrap();
        let loaded = ApprovalRecord::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), ar.title());
        assert_eq!(loaded.phase().unwrap(), ar.phase().unwrap());
        assert_eq!(loaded.approved_artifact, ar.approved_artifact);
        assert_eq!(loaded.approver, ar.approver);
        assert_eq!(loaded.approval_date, ar.approval_date);
        assert_eq!(loaded.approval_type, ar.approval_type);
    }
}
