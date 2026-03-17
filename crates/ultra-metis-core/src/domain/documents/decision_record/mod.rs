use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{DocumentId, Phase, Tag};
use chrono::{DateTime, Utc};
use gray_matter;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tera::{Context, Tera};

// ---------------------------------------------------------------------------
// Sub-structs
// ---------------------------------------------------------------------------

/// An alternative that was considered but not chosen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alternative {
    /// Title or short name for the alternative.
    pub title: String,
    /// Description of the alternative approach.
    pub description: String,
    /// Why this alternative was rejected.
    pub rejected_reason: String,
}

impl Alternative {
    pub fn new(title: &str, description: &str, rejected_reason: &str) -> Self {
        Self {
            title: title.to_string(),
            description: description.to_string(),
            rejected_reason: rejected_reason.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// DecisionRecord
// ---------------------------------------------------------------------------

/// A durable record of a significant decision made during work execution.
///
/// Captures what was decided, what alternatives were considered, the rationale,
/// who approved it, and links to supporting evidence.
#[derive(Debug)]
pub struct DecisionRecord {
    core: DocumentCore,
    /// The decision that was made.
    pub decision: String,
    /// Context that led to this decision being needed.
    pub decision_context: String,
    /// Alternatives that were considered.
    pub alternatives: Vec<Alternative>,
    /// Rationale for the chosen decision.
    pub rationale: String,
    /// Who approved this decision (optional).
    pub approved_by: Option<String>,
    /// Short codes of supporting evidence documents.
    pub evidence: Vec<String>,
    /// Short codes of related artifacts.
    pub related_artifacts: Vec<String>,
    /// When the decision was made.
    pub timestamp: DateTime<Utc>,
}

impl DecisionRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        decision: String,
        decision_context: String,
        rationale: String,
        timestamp: DateTime<Utc>,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let template_content = include_str!("content.md");
        let mut tera = Tera::default();
        tera.add_raw_template("decision_record_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("decision_record_content", &context)
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
            decision,
            decision_context,
            alternatives: Vec::new(),
            rationale,
            approved_by: None,
            evidence: Vec::new(),
            related_artifacts: Vec::new(),
            timestamp,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        decision: String,
        decision_context: String,
        alternatives: Vec<Alternative>,
        rationale: String,
        approved_by: Option<String>,
        evidence: Vec<String>,
        related_artifacts: Vec<String>,
        timestamp: DateTime<Utc>,
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
            decision,
            decision_context,
            alternatives,
            rationale,
            approved_by,
            evidence,
            related_artifacts,
            timestamp,
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
        if level != "decision_record" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'decision_record', found '{}'",
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

        let decision = FrontmatterParser::extract_string(&fm_map, "decision")?;
        let decision_context =
            FrontmatterParser::extract_string(&fm_map, "decision_context")?;
        let rationale = FrontmatterParser::extract_string(&fm_map, "rationale")?;
        let approved_by =
            FrontmatterParser::extract_optional_string(&fm_map, "approved_by");

        let timestamp_str = FrontmatterParser::extract_string(&fm_map, "timestamp")?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| {
                DocumentValidationError::InvalidContent(
                    "Invalid datetime for timestamp".to_string(),
                )
            })?;

        let evidence =
            FrontmatterParser::extract_string_array(&fm_map, "evidence").unwrap_or_default();
        let related_artifacts =
            FrontmatterParser::extract_string_array(&fm_map, "related_artifacts")
                .unwrap_or_default();
        let alternatives = Self::parse_alternatives(&fm_map)?;

        let metadata =
            DocumentMetadata::from_frontmatter(created_at, updated_at, exit_criteria_met, short_code);
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            tags,
            archived,
            decision,
            decision_context,
            alternatives,
            rationale,
            approved_by,
            evidence,
            related_artifacts,
            timestamp,
        ))
    }

    fn parse_alternatives(
        fm_map: &std::collections::HashMap<String, gray_matter::Pod>,
    ) -> Result<Vec<Alternative>, DocumentValidationError> {
        let arr = match fm_map.get("alternatives") {
            Some(gray_matter::Pod::Array(arr)) => arr,
            _ => return Ok(Vec::new()),
        };

        let mut entries = Vec::new();
        for item in arr {
            if let gray_matter::Pod::Hash(map) = item {
                let title = match map.get("title") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => continue,
                };
                let description = match map.get("description") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => String::new(),
                };
                let rejected_reason = match map.get("rejected_reason") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => String::new(),
                };
                entries.push(Alternative {
                    title,
                    description,
                    rejected_reason,
                });
            }
        }
        Ok(entries)
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

        context.insert("decision", &self.decision);
        context.insert("decision_context", &self.decision_context);
        context.insert("rationale", &self.rationale);
        context.insert(
            "approved_by",
            &self.approved_by.as_deref().unwrap_or("NULL"),
        );
        context.insert("timestamp", &self.timestamp.to_rfc3339());

        let alt_maps: Vec<std::collections::HashMap<&str, String>> = self
            .alternatives
            .iter()
            .map(|a| {
                let mut m = std::collections::HashMap::new();
                m.insert("title", a.title.clone());
                m.insert("description", a.description.clone());
                m.insert("rejected_reason", a.rejected_reason.clone());
                m
            })
            .collect();
        context.insert("alternatives", &alt_maps);

        context.insert("evidence", &self.evidence);
        context.insert("related_artifacts", &self.related_artifacts);

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
                "DecisionRecord title cannot be empty".to_string(),
            ));
        }
        if self.decision.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "decision".to_string(),
            ));
        }
        if self.rationale.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "rationale".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_decision_record() -> DecisionRecord {
        let mut record = DecisionRecord::new(
            "Use JWT for authentication".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DR-0001".to_string(),
            "Use JWT tokens for API authentication".to_string(),
            "Need to choose an auth mechanism for the REST API".to_string(),
            "JWT is stateless, widely supported, and works well with microservices".to_string(),
            Utc::now(),
        )
        .unwrap();

        record.alternatives = vec![
            Alternative::new(
                "Session cookies",
                "Traditional session-based auth with server-side storage",
                "Requires server-side state, harder to scale horizontally",
            ),
            Alternative::new(
                "OAuth2 only",
                "Delegate all auth to external OAuth2 provider",
                "Too complex for initial implementation, adds external dependency",
            ),
        ];
        record.approved_by = Some("tech-lead".to_string());
        record.evidence = vec!["PROJ-AI-0001".to_string()];
        record.related_artifacts = vec!["PROJ-S-0001".to_string(), "PROJ-T-0042".to_string()];
        record
    }

    #[test]
    fn test_decision_record_creation() {
        let record = make_decision_record();
        assert_eq!(record.title(), "Use JWT for authentication");
        assert_eq!(record.phase().unwrap(), Phase::Draft);
        assert_eq!(record.decision, "Use JWT tokens for API authentication");
        assert_eq!(record.alternatives.len(), 2);
        assert_eq!(record.approved_by, Some("tech-lead".to_string()));
        assert_eq!(record.evidence.len(), 1);
        assert_eq!(record.related_artifacts.len(), 2);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_decision_record_empty_title_invalid() {
        let record = DecisionRecord::new(
            String::new(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DR-0002".to_string(),
            "Some decision".to_string(),
            "Some context".to_string(),
            "Some rationale".to_string(),
            Utc::now(),
        )
        .unwrap();
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_decision_record_empty_decision_invalid() {
        let record = DecisionRecord::new(
            "A decision".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DR-0003".to_string(),
            String::new(),
            "Some context".to_string(),
            "Some rationale".to_string(),
            Utc::now(),
        )
        .unwrap();
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_decision_record_empty_rationale_invalid() {
        let record = DecisionRecord::new(
            "A decision".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DR-0004".to_string(),
            "Use X".to_string(),
            "Context".to_string(),
            String::new(),
            Utc::now(),
        )
        .unwrap();
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_alternative_constructor() {
        let alt = Alternative::new("Option A", "Description", "Rejected because");
        assert_eq!(alt.title, "Option A");
        assert_eq!(alt.description, "Description");
        assert_eq!(alt.rejected_reason, "Rejected because");
    }

    #[tokio::test]
    async fn test_decision_record_roundtrip() {
        let record = make_decision_record();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-decision-record.md");

        record.to_file(&file_path).await.unwrap();
        let loaded = DecisionRecord::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), record.title());
        assert_eq!(loaded.phase().unwrap(), record.phase().unwrap());
        assert_eq!(loaded.decision, record.decision);
        assert_eq!(loaded.decision_context, record.decision_context);
        assert_eq!(loaded.rationale, record.rationale);
        assert_eq!(loaded.approved_by, record.approved_by);
        assert_eq!(loaded.alternatives.len(), record.alternatives.len());
        assert_eq!(loaded.alternatives[0].title, record.alternatives[0].title);
        assert_eq!(
            loaded.alternatives[0].rejected_reason,
            record.alternatives[0].rejected_reason
        );
        assert_eq!(loaded.evidence.len(), record.evidence.len());
        assert_eq!(loaded.related_artifacts.len(), record.related_artifacts.len());
    }
}
