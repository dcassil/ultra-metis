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

/// An AnalysisBaseline captures a point-in-time snapshot of quality thresholds.
/// Used as a reference point for quality records to compare against.
#[derive(Debug)]
pub struct AnalysisBaseline {
    core: DocumentCore,
    pub linked_rules_config: Option<String>,
    pub baseline_date: String,
}

impl AnalysisBaseline {
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        linked_rules_config: Option<String>,
        baseline_date: String,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            tags,
            archived,
            short_code,
            linked_rules_config,
            baseline_date,
            template_content,
        )
    }

    pub fn new_with_template(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        linked_rules_config: Option<String>,
        baseline_date: String,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("analysis_baseline_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("analysis_baseline_content", &context)
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
            linked_rules_config,
            baseline_date,
        })
    }

    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        linked_rules_config: Option<String>,
        baseline_date: String,
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
            linked_rules_config,
            baseline_date,
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
        if level != "analysis_baseline" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'analysis_baseline', found '{}'",
                level
            )));
        }

        let short_code = FrontmatterParser::extract_string(&fm_map, "short_code")?;
        let metadata =
            DocumentMetadata::from_frontmatter(created_at, updated_at, exit_criteria_met, short_code);
        let content = DocumentContent::from_markdown(&parsed.content);

        let linked_rules_config = FrontmatterParser::extract_optional_string(&fm_map, "linked_rules_config");
        let baseline_date = FrontmatterParser::extract_string(&fm_map, "baseline_date")?;

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            tags,
            archived,
            linked_rules_config,
            baseline_date,
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

        let linked_rules_config_val = self
            .linked_rules_config
            .as_deref()
            .unwrap_or("NULL");
        context.insert("linked_rules_config", linked_rules_config_val);
        context.insert("baseline_date", &self.baseline_date);

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
    fn test_analysis_baseline_creation() {
        let baseline = AnalysisBaseline::new(
            "Q1 Baseline".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-AB-0001".to_string(),
            Some("RULES-001".to_string()),
            "2026-03-16".to_string(),
        )
        .unwrap();

        assert_eq!(baseline.title(), "Q1 Baseline");
        assert_eq!(baseline.phase().unwrap(), Phase::Draft);
        assert_eq!(baseline.linked_rules_config, Some("RULES-001".to_string()));
        assert_eq!(baseline.baseline_date, "2026-03-16");
    }

    #[tokio::test]
    async fn test_analysis_baseline_roundtrip() {
        let baseline = AnalysisBaseline::new(
            "Roundtrip Baseline".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-AB-0002".to_string(),
            Some("RULES-002".to_string()),
            "2026-03-16".to_string(),
        )
        .unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test-baseline.md");

        baseline.to_file(&file_path).await.unwrap();
        let loaded = AnalysisBaseline::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), baseline.title());
        assert_eq!(loaded.phase().unwrap(), baseline.phase().unwrap());
        assert_eq!(loaded.linked_rules_config, baseline.linked_rules_config);
        assert_eq!(loaded.baseline_date, baseline.baseline_date);
    }

    #[test]
    fn test_analysis_baseline_no_rules_config() {
        let baseline = AnalysisBaseline::new(
            "Minimal Baseline".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-AB-0003".to_string(),
            None,
            "2026-03-16".to_string(),
        )
        .unwrap();

        assert!(baseline.linked_rules_config.is_none());
        let content = baseline.to_content().unwrap();
        assert!(content.contains("level: analysis_baseline"));
    }
}
