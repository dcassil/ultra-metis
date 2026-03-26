use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{DocumentId, Phase, Tag};
use gray_matter;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tera::{Context, Tera};

/// Overall quality status for a quality record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityStatus {
    Pass,
    Warn,
    Fail,
}

impl QualityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "Pass",
            Self::Warn => "Warn",
            Self::Fail => "Fail",
        }
    }
}

impl std::str::FromStr for QualityStatus {
    type Err = DocumentValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pass" => Ok(Self::Pass),
            "Warn" => Ok(Self::Warn),
            "Fail" => Ok(Self::Fail),
            other => Err(DocumentValidationError::InvalidContent(format!(
                "Unknown QualityStatus: '{other}'"
            ))),
        }
    }
}

/// A QualityRecord captures point-in-time quality measurements compared against a baseline.
#[derive(Debug)]
pub struct QualityRecord {
    core: DocumentCore,
    pub linked_baseline: Option<String>,
    pub record_date: String,
    pub overall_status: QualityStatus,
}

impl QualityRecord {
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        linked_baseline: Option<String>,
        record_date: String,
        overall_status: QualityStatus,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            tags,
            archived,
            short_code,
            linked_baseline,
            record_date,
            overall_status,
            template_content,
        )
    }

    pub fn new_with_template(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        linked_baseline: Option<String>,
        record_date: String,
        overall_status: QualityStatus,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("quality_record_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {e}"))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("quality_record_content", &context)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template render error: {e}"))
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
            linked_baseline,
            record_date,
            overall_status,
        })
    }

    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        linked_baseline: Option<String>,
        record_date: String,
        overall_status: QualityStatus,
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
            linked_baseline,
            record_date,
            overall_status,
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

        let title = FrontmatterParser::extract_string(&fm_map, "title")?;
        let archived = FrontmatterParser::extract_bool(&fm_map, "archived").unwrap_or(false);
        let created_at = FrontmatterParser::extract_datetime(&fm_map, "created_at")?;
        let updated_at = FrontmatterParser::extract_datetime(&fm_map, "updated_at")?;
        let exit_criteria_met =
            FrontmatterParser::extract_bool(&fm_map, "exit_criteria_met").unwrap_or(false);
        let tags = FrontmatterParser::extract_tags(&fm_map)?;

        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "quality_record" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'quality_record', found '{level}'"
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

        let linked_baseline =
            FrontmatterParser::extract_optional_string(&fm_map, "linked_baseline");
        let record_date = FrontmatterParser::extract_string(&fm_map, "record_date")?;
        let overall_status_str = FrontmatterParser::extract_string(&fm_map, "overall_status")?;
        let overall_status = overall_status_str.parse::<QualityStatus>()?;

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            tags,
            archived,
            linked_baseline,
            record_date,
            overall_status,
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

    /// Get mutable access to the document core
    pub fn core_mut(&mut self) -> &mut DocumentCore {
        &mut self.core
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
        context.insert("short_code", &self.metadata().short_code);
        context.insert("created_at", &self.metadata().created_at.to_rfc3339());
        context.insert("updated_at", &self.metadata().updated_at.to_rfc3339());
        context.insert("archived", &self.archived().to_string());
        context.insert(
            "exit_criteria_met",
            &self.metadata().exit_criteria_met.to_string(),
        );

        let tag_strings: Vec<String> = self.tags().iter().map(super::types::Tag::to_str).collect();
        context.insert("tags", &tag_strings);
        context.insert("epic_id", "NULL");

        let linked_baseline_val = self.linked_baseline.as_deref().unwrap_or("NULL");
        context.insert("linked_baseline", linked_baseline_val);
        context.insert("record_date", &self.record_date);
        context.insert("overall_status", self.overall_status.as_str());

        let frontmatter = tera.render("frontmatter", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Frontmatter render error: {e}"))
        })?;

        let content_body = &self.content().body;
        let acceptance_criteria = if let Some(ac) = &self.content().acceptance_criteria {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_record_creation() {
        let record = QualityRecord::new(
            "Q1 Quality Record".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QR-0001".to_string(),
            Some("TEST-AB-0001".to_string()),
            "2026-03-16".to_string(),
            QualityStatus::Pass,
        )
        .unwrap();

        assert_eq!(record.title(), "Q1 Quality Record");
        assert_eq!(record.phase().unwrap(), Phase::Draft);
        assert_eq!(record.overall_status, QualityStatus::Pass);
        assert_eq!(record.linked_baseline, Some("TEST-AB-0001".to_string()));
    }

    #[tokio::test]
    async fn test_quality_record_roundtrip() {
        let record = QualityRecord::new(
            "Roundtrip Quality Record".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QR-0002".to_string(),
            Some("TEST-AB-0002".to_string()),
            "2026-03-16".to_string(),
            QualityStatus::Warn,
        )
        .unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test-quality-record.md");

        record.to_file(&file_path).await.unwrap();
        let loaded = QualityRecord::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), record.title());
        assert_eq!(loaded.phase().unwrap(), record.phase().unwrap());
        assert_eq!(loaded.overall_status, record.overall_status);
        assert_eq!(loaded.linked_baseline, record.linked_baseline);
        assert_eq!(loaded.record_date, record.record_date);
    }

    #[test]
    fn test_quality_status_parsing() {
        assert_eq!(
            "Pass".parse::<QualityStatus>().unwrap(),
            QualityStatus::Pass
        );
        assert_eq!(
            "Warn".parse::<QualityStatus>().unwrap(),
            QualityStatus::Warn
        );
        assert_eq!(
            "Fail".parse::<QualityStatus>().unwrap(),
            QualityStatus::Fail
        );
        assert!("Unknown".parse::<QualityStatus>().is_err());
    }
}
