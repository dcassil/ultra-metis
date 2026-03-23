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

/// Lifecycle status of an architecture investigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestigationStatus {
    Open,
    InProgress,
    Concluded,
    Archived,
}

impl InvestigationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            InvestigationStatus::Open => "open",
            InvestigationStatus::InProgress => "in-progress",
            InvestigationStatus::Concluded => "concluded",
            InvestigationStatus::Archived => "archived",
        }
    }
}

impl std::fmt::Display for InvestigationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for InvestigationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(InvestigationStatus::Open),
            "in-progress" | "in_progress" | "inprogress" => Ok(InvestigationStatus::InProgress),
            "concluded" => Ok(InvestigationStatus::Concluded),
            "archived" => Ok(InvestigationStatus::Archived),
            _ => Err(format!("Unknown investigation status: {}", s)),
        }
    }
}

/// An ArchitectureInvestigation captures a focused inquiry triggered by anomalies,
/// proposals, or open questions in the system architecture.
/// Phases: Draft → Active → Concluded
#[derive(Debug)]
pub struct ArchitectureInvestigation {
    core: DocumentCore,
    /// Short codes of artifacts or events that triggered this investigation
    pub trigger_refs: Vec<String>,
    /// Current lifecycle status of the investigation
    pub investigation_status: InvestigationStatus,
}

impl ArchitectureInvestigation {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        parent_id: Option<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        trigger_refs: Vec<String>,
        investigation_status: InvestigationStatus,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            parent_id,
            tags,
            archived,
            short_code,
            trigger_refs,
            investigation_status,
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
        trigger_refs: Vec<String>,
        investigation_status: InvestigationStatus,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("architecture_investigation_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("architecture_investigation_content", &context)
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
            trigger_refs,
            investigation_status,
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
        trigger_refs: Vec<String>,
        investigation_status: InvestigationStatus,
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
            trigger_refs,
            investigation_status,
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
        if level != "architecture_investigation" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'architecture_investigation', found '{}'",
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

        let trigger_refs =
            FrontmatterParser::extract_string_array(&fm_map, "trigger_refs").unwrap_or_default();

        let investigation_status =
            FrontmatterParser::extract_optional_string(&fm_map, "investigation_status")
                .and_then(|s| s.parse::<InvestigationStatus>().ok())
                .unwrap_or(InvestigationStatus::Open);

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
            trigger_refs,
            investigation_status,
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

        context.insert("trigger_refs", &self.trigger_refs);
        context.insert("investigation_status", self.investigation_status.as_str());

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
                "ArchitectureInvestigation title cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_architecture_investigation() -> ArchitectureInvestigation {
        ArchitectureInvestigation::new(
            "Investigate latency spike in query-service after migration".to_string(),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-AI-0001".to_string(),
            vec!["DC-0033".to_string(), "DCP-0007".to_string()],
            InvestigationStatus::Open,
        )
        .unwrap()
    }

    #[test]
    fn test_architecture_investigation_creation() {
        let ai = make_architecture_investigation();

        assert_eq!(
            ai.title(),
            "Investigate latency spike in query-service after migration"
        );
        assert_eq!(ai.phase().unwrap(), Phase::Draft);
        assert_eq!(ai.trigger_refs, vec!["DC-0033", "DCP-0007"]);
        assert_eq!(ai.investigation_status, InvestigationStatus::Open);
        assert!(ai.validate().is_ok());
    }

    #[test]
    fn test_investigation_status_roundtrip() {
        for status in &[
            InvestigationStatus::Open,
            InvestigationStatus::InProgress,
            InvestigationStatus::Concluded,
            InvestigationStatus::Archived,
        ] {
            let s = status.as_str();
            let parsed: InvestigationStatus = s.parse().unwrap();
            assert_eq!(parsed, *status);
        }
    }

    #[test]
    fn test_architecture_investigation_no_triggers() {
        let ai = ArchitectureInvestigation::new(
            "Investigate memory usage in worker pool".to_string(),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-AI-0002".to_string(),
            vec![],
            InvestigationStatus::Open,
        )
        .unwrap();

        assert!(ai.trigger_refs.is_empty());
        assert!(ai.validate().is_ok());
    }

    #[tokio::test]
    async fn test_architecture_investigation_roundtrip() {
        let ai = make_architecture_investigation();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-ai.md");

        ai.to_file(&file_path).await.unwrap();
        let loaded = ArchitectureInvestigation::from_file(&file_path)
            .await
            .unwrap();

        assert_eq!(loaded.title(), ai.title());
        assert_eq!(loaded.phase().unwrap(), ai.phase().unwrap());
        assert_eq!(loaded.trigger_refs, ai.trigger_refs);
        assert_eq!(loaded.investigation_status, ai.investigation_status);
    }
}
