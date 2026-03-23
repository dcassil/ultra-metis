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

/// Resolution status for a remediation record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStatus {
    Open,
    InProgress,
    Resolved,
    WontFix,
}

impl ResolutionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResolutionStatus::Open => "Open",
            ResolutionStatus::InProgress => "InProgress",
            ResolutionStatus::Resolved => "Resolved",
            ResolutionStatus::WontFix => "WontFix",
        }
    }
}

impl std::str::FromStr for ResolutionStatus {
    type Err = DocumentValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Open" => Ok(ResolutionStatus::Open),
            "InProgress" => Ok(ResolutionStatus::InProgress),
            "Resolved" => Ok(ResolutionStatus::Resolved),
            "WontFix" => Ok(ResolutionStatus::WontFix),
            other => Err(DocumentValidationError::InvalidContent(format!(
                "Unknown ResolutionStatus: '{}'",
                other
            ))),
        }
    }
}

/// A RemediationRecord tracks a detected problem and the steps taken to fix it.
#[derive(Debug)]
pub struct RemediationRecord {
    core: DocumentCore,
    pub problem_type: String,
    pub affected_scope: String,
    pub is_systemic: bool,
    pub resolution_status: ResolutionStatus,
}

impl RemediationRecord {
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        problem_type: String,
        affected_scope: String,
        is_systemic: bool,
        resolution_status: ResolutionStatus,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            tags,
            archived,
            short_code,
            problem_type,
            affected_scope,
            is_systemic,
            resolution_status,
            template_content,
        )
    }

    pub fn new_with_template(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        problem_type: String,
        affected_scope: String,
        is_systemic: bool,
        resolution_status: ResolutionStatus,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("remediation_record_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("remediation_record_content", &context)
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
            problem_type,
            affected_scope,
            is_systemic,
            resolution_status,
        })
    }

    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        problem_type: String,
        affected_scope: String,
        is_systemic: bool,
        resolution_status: ResolutionStatus,
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
            problem_type,
            affected_scope,
            is_systemic,
            resolution_status,
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
        if level != "remediation_record" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'remediation_record', found '{}'",
                level
            )));
        }

        let short_code = FrontmatterParser::extract_string(&fm_map, "short_code")?;
        let metadata = DocumentMetadata::from_frontmatter(
            created_at,
            updated_at,
            exit_criteria_met,
            short_code,
        );
        let content = DocumentContent::from_markdown(&parsed.content);

        let problem_type = FrontmatterParser::extract_string(&fm_map, "problem_type")?;
        let affected_scope = FrontmatterParser::extract_string(&fm_map, "affected_scope")?;
        let is_systemic = FrontmatterParser::extract_bool(&fm_map, "is_systemic").unwrap_or(false);
        let resolution_status_str =
            FrontmatterParser::extract_string(&fm_map, "resolution_status")?;
        let resolution_status = resolution_status_str.parse::<ResolutionStatus>()?;

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            tags,
            archived,
            problem_type,
            affected_scope,
            is_systemic,
            resolution_status,
        ))
    }

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

    pub fn id(&self) -> DocumentId {
        DocumentId::from_title(&self.core.title)
    }

    pub fn phase(&self) -> Result<Phase, DocumentValidationError> {
        for tag in &self.core.tags {
            if let Tag::Phase(phase) = tag {
                return Ok(*phase);
            }
        }
        Err(DocumentValidationError::MissingPhaseTag)
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
        context.insert("short_code", &self.metadata().short_code);
        context.insert("created_at", &self.metadata().created_at.to_rfc3339());
        context.insert("updated_at", &self.metadata().updated_at.to_rfc3339());
        context.insert("archived", &self.archived().to_string());
        context.insert(
            "exit_criteria_met",
            &self.metadata().exit_criteria_met.to_string(),
        );

        let tag_strings: Vec<String> = self.tags().iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);
        context.insert("epic_id", "NULL");

        context.insert("problem_type", &self.problem_type);
        context.insert("affected_scope", &self.affected_scope);
        context.insert("is_systemic", &self.is_systemic.to_string());
        context.insert("resolution_status", self.resolution_status.as_str());

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

    #[test]
    fn test_remediation_record_creation() {
        let record = RemediationRecord::new(
            "Architecture Drift Fix".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-RR-0001".to_string(),
            "architecture-drift".to_string(),
            "crates/cadre-core".to_string(),
            true,
            ResolutionStatus::Open,
        )
        .unwrap();

        assert_eq!(record.title(), "Architecture Drift Fix");
        assert_eq!(record.phase().unwrap(), Phase::Draft);
        assert_eq!(record.problem_type, "architecture-drift");
        assert_eq!(record.affected_scope, "crates/cadre-core");
        assert!(record.is_systemic);
        assert_eq!(record.resolution_status, ResolutionStatus::Open);
    }

    #[tokio::test]
    async fn test_remediation_record_roundtrip() {
        let record = RemediationRecord::new(
            "Quality Degradation Fix".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-RR-0002".to_string(),
            "quality-degradation".to_string(),
            "src/domain".to_string(),
            false,
            ResolutionStatus::InProgress,
        )
        .unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test-remediation-record.md");

        record.to_file(&file_path).await.unwrap();
        let loaded = RemediationRecord::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), record.title());
        assert_eq!(loaded.phase().unwrap(), record.phase().unwrap());
        assert_eq!(loaded.problem_type, record.problem_type);
        assert_eq!(loaded.affected_scope, record.affected_scope);
        assert_eq!(loaded.is_systemic, record.is_systemic);
        assert_eq!(loaded.resolution_status, record.resolution_status);
    }

    #[test]
    fn test_resolution_status_parsing() {
        assert_eq!(
            "Open".parse::<ResolutionStatus>().unwrap(),
            ResolutionStatus::Open
        );
        assert_eq!(
            "InProgress".parse::<ResolutionStatus>().unwrap(),
            ResolutionStatus::InProgress
        );
        assert_eq!(
            "Resolved".parse::<ResolutionStatus>().unwrap(),
            ResolutionStatus::Resolved
        );
        assert_eq!(
            "WontFix".parse::<ResolutionStatus>().unwrap(),
            ResolutionStatus::WontFix
        );
        assert!("invalid".parse::<ResolutionStatus>().is_err());
    }
}
