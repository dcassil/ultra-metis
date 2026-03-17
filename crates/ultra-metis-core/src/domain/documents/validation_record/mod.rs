use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{DocumentId, Phase, Tag};
use chrono::Utc;
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};
use serde::{Deserialize, Serialize};

/// Result of an individual validation run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationResult {
    Passed,
    Failed,
    Skipped,
}

impl ValidationResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            ValidationResult::Passed => "Passed",
            ValidationResult::Failed => "Failed",
            ValidationResult::Skipped => "Skipped",
        }
    }
}

impl std::str::FromStr for ValidationResult {
    type Err = DocumentValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Passed" => Ok(ValidationResult::Passed),
            "Failed" => Ok(ValidationResult::Failed),
            "Skipped" => Ok(ValidationResult::Skipped),
            other => Err(DocumentValidationError::InvalidContent(format!(
                "Unknown ValidationResult: '{}'",
                other
            ))),
        }
    }
}

/// A ValidationRecord captures a single validation run, its inputs, and its outcome.
#[derive(Debug)]
pub struct ValidationRecord {
    core: DocumentCore,
    pub validation_type: String,
    pub result: ValidationResult,
    pub related_artifact: Option<String>,
    pub required: bool,
}

impl ValidationRecord {
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        validation_type: String,
        result: ValidationResult,
        related_artifact: Option<String>,
        required: bool,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            tags,
            archived,
            short_code,
            validation_type,
            result,
            related_artifact,
            required,
            template_content,
        )
    }

    pub fn new_with_template(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        validation_type: String,
        result: ValidationResult,
        related_artifact: Option<String>,
        required: bool,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("validation_record_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("validation_record_content", &context)
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
            validation_type,
            result,
            related_artifact,
            required,
        })
    }

    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        validation_type: String,
        result: ValidationResult,
        related_artifact: Option<String>,
        required: bool,
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
            validation_type,
            result,
            related_artifact,
            required,
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
        if level != "validation_record" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'validation_record', found '{}'",
                level
            )));
        }

        let short_code = FrontmatterParser::extract_string(&fm_map, "short_code")?;
        let metadata =
            DocumentMetadata::from_frontmatter(created_at, updated_at, exit_criteria_met, short_code);
        let content = DocumentContent::from_markdown(&parsed.content);

        let validation_type = FrontmatterParser::extract_string(&fm_map, "validation_type")?;
        let result_str = FrontmatterParser::extract_string(&fm_map, "result")?;
        let result = result_str.parse::<ValidationResult>()?;
        let related_artifact = FrontmatterParser::extract_optional_string(&fm_map, "related_artifact");
        let required = FrontmatterParser::extract_bool(&fm_map, "required").unwrap_or(false);

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            tags,
            archived,
            validation_type,
            result,
            related_artifact,
            required,
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

        context.insert("validation_type", &self.validation_type);
        context.insert("result", self.result.as_str());
        let related_artifact_val = self.related_artifact.as_deref().unwrap_or("NULL");
        context.insert("related_artifact", related_artifact_val);
        context.insert("required", &self.required.to_string());

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
    fn test_validation_record_creation() {
        let record = ValidationRecord::new(
            "Unit Test Run".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-VR-0001".to_string(),
            "unit-test".to_string(),
            ValidationResult::Passed,
            Some("FEAT-0001".to_string()),
            true,
        )
        .unwrap();

        assert_eq!(record.title(), "Unit Test Run");
        assert_eq!(record.phase().unwrap(), Phase::Draft);
        assert_eq!(record.result, ValidationResult::Passed);
        assert_eq!(record.validation_type, "unit-test");
        assert!(record.required);
    }

    #[tokio::test]
    async fn test_validation_record_roundtrip() {
        let record = ValidationRecord::new(
            "Lint Check Run".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-VR-0002".to_string(),
            "lint".to_string(),
            ValidationResult::Failed,
            None,
            false,
        )
        .unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test-validation-record.md");

        record.to_file(&file_path).await.unwrap();
        let loaded = ValidationRecord::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), record.title());
        assert_eq!(loaded.phase().unwrap(), record.phase().unwrap());
        assert_eq!(loaded.result, record.result);
        assert_eq!(loaded.validation_type, record.validation_type);
        assert_eq!(loaded.related_artifact, record.related_artifact);
        assert_eq!(loaded.required, record.required);
    }

    #[test]
    fn test_validation_result_parsing() {
        assert_eq!("Passed".parse::<ValidationResult>().unwrap(), ValidationResult::Passed);
        assert_eq!("Failed".parse::<ValidationResult>().unwrap(), ValidationResult::Failed);
        assert_eq!("Skipped".parse::<ValidationResult>().unwrap(), ValidationResult::Skipped);
        assert!("unknown".parse::<ValidationResult>().is_err());
    }
}
