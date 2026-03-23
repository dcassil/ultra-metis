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

/// Result of a pre-transition check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckResult {
    /// Name of the check (e.g., "quality_gate", "exit_criteria").
    pub name: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Details about the check result.
    pub details: String,
}

impl CheckResult {
    pub fn new(name: &str, passed: bool, details: &str) -> Self {
        Self {
            name: name.to_string(),
            passed,
            details: details.to_string(),
        }
    }

    /// Create a passing check result.
    pub fn passed(name: &str, details: &str) -> Self {
        Self::new(name, true, details)
    }

    /// Create a failing check result.
    pub fn failed(name: &str, details: &str) -> Self {
        Self::new(name, false, details)
    }
}

// ---------------------------------------------------------------------------
// TransitionRecord
// ---------------------------------------------------------------------------

/// A durable record of a phase transition for a document.
///
/// Captures who triggered the transition, when, from/to phases, which
/// pre-transition checks ran, and whether the transition was forced.
#[derive(Debug)]
pub struct TransitionRecord {
    core: DocumentCore,
    /// Short code of the document that was transitioned.
    pub document_ref: String,
    /// Phase before the transition.
    pub from_phase: String,
    /// Phase after the transition.
    pub to_phase: String,
    /// Who or what triggered the transition.
    pub actor: String,
    /// When the transition occurred.
    pub timestamp: DateTime<Utc>,
    /// Pre-transition checks that were evaluated.
    pub checks_run: Vec<CheckResult>,
    /// Reason for the transition (optional).
    pub reason: Option<String>,
    /// Whether the transition was force-overridden.
    pub forced: bool,
}

impl TransitionRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        document_ref: String,
        from_phase: String,
        to_phase: String,
        actor: String,
        timestamp: DateTime<Utc>,
        forced: bool,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let template_content = include_str!("content.md");
        let mut tera = Tera::default();
        tera.add_raw_template("transition_record_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("transition_record_content", &context)
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
            document_ref,
            from_phase,
            to_phase,
            actor,
            timestamp,
            checks_run: Vec::new(),
            reason: None,
            forced,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        document_ref: String,
        from_phase: String,
        to_phase: String,
        actor: String,
        timestamp: DateTime<Utc>,
        checks_run: Vec<CheckResult>,
        reason: Option<String>,
        forced: bool,
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
            document_ref,
            from_phase,
            to_phase,
            actor,
            timestamp,
            checks_run,
            reason,
            forced,
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
        if level != "transition_record" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'transition_record', found '{}'",
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

        let document_ref = FrontmatterParser::extract_string(&fm_map, "document_ref")?;
        let from_phase = FrontmatterParser::extract_string(&fm_map, "from_phase")?;
        let to_phase = FrontmatterParser::extract_string(&fm_map, "to_phase")?;
        let actor = FrontmatterParser::extract_string(&fm_map, "actor")?;
        let forced = FrontmatterParser::extract_bool(&fm_map, "forced").unwrap_or(false);

        let timestamp_str = FrontmatterParser::extract_string(&fm_map, "timestamp")?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| {
                DocumentValidationError::InvalidContent(
                    "Invalid datetime for timestamp".to_string(),
                )
            })?;

        let reason = FrontmatterParser::extract_optional_string(&fm_map, "reason");
        let checks_run = Self::parse_check_results(&fm_map)?;

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
            tags,
            archived,
            document_ref,
            from_phase,
            to_phase,
            actor,
            timestamp,
            checks_run,
            reason,
            forced,
        ))
    }

    fn parse_check_results(
        fm_map: &std::collections::HashMap<String, gray_matter::Pod>,
    ) -> Result<Vec<CheckResult>, DocumentValidationError> {
        let arr = match fm_map.get("checks_run") {
            Some(gray_matter::Pod::Array(arr)) => arr,
            _ => return Ok(Vec::new()),
        };

        let mut entries = Vec::new();
        for item in arr {
            if let gray_matter::Pod::Hash(map) = item {
                let name = match map.get("name") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => continue,
                };
                let passed = match map.get("passed") {
                    Some(gray_matter::Pod::Boolean(b)) => *b,
                    _ => false,
                };
                let details = match map.get("details") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => String::new(),
                };
                entries.push(CheckResult {
                    name,
                    passed,
                    details,
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

        context.insert("document_ref", &self.document_ref);
        context.insert("from_phase", &self.from_phase);
        context.insert("to_phase", &self.to_phase);
        context.insert("actor", &self.actor);
        context.insert("timestamp", &self.timestamp.to_rfc3339());
        context.insert("forced", &self.forced.to_string());
        context.insert("reason", &self.reason.as_deref().unwrap_or("NULL"));

        let check_maps: Vec<serde_json::Value> = self
            .checks_run
            .iter()
            .map(|c| {
                serde_json::json!({
                    "name": c.name,
                    "passed": c.passed,
                    "details": c.details,
                })
            })
            .collect();
        context.insert("checks_run", &check_maps);

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

    /// Check if all pre-transition checks passed.
    pub fn all_checks_passed(&self) -> bool {
        self.checks_run.iter().all(|c| c.passed)
    }

    pub fn validate(&self) -> Result<(), DocumentValidationError> {
        if self.core.title.trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "TransitionRecord title cannot be empty".to_string(),
            ));
        }
        if self.document_ref.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "document_ref".to_string(),
            ));
        }
        if self.from_phase.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "from_phase".to_string(),
            ));
        }
        if self.to_phase.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "to_phase".to_string(),
            ));
        }
        if self.actor.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "actor".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_transition_record() -> TransitionRecord {
        let mut record = TransitionRecord::new(
            "Transition: PROJ-T-0042 active -> completed".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-TR-0001".to_string(),
            "PROJ-T-0042".to_string(),
            "active".to_string(),
            "completed".to_string(),
            "claude-agent".to_string(),
            Utc::now(),
            false,
        )
        .unwrap();

        record.checks_run = vec![
            CheckResult::passed("quality_gate", "All metrics within thresholds"),
            CheckResult::passed("exit_criteria", "All acceptance criteria met"),
        ];
        record.reason = Some("All work completed and verified".to_string());
        record
    }

    #[test]
    fn test_transition_record_creation() {
        let record = make_transition_record();
        assert_eq!(
            record.title(),
            "Transition: PROJ-T-0042 active -> completed"
        );
        assert_eq!(record.phase().unwrap(), Phase::Draft);
        assert_eq!(record.document_ref, "PROJ-T-0042");
        assert_eq!(record.from_phase, "active");
        assert_eq!(record.to_phase, "completed");
        assert_eq!(record.actor, "claude-agent");
        assert!(!record.forced);
        assert_eq!(record.checks_run.len(), 2);
        assert!(record.all_checks_passed());
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_transition_record_empty_title_invalid() {
        let record = TransitionRecord::new(
            String::new(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-TR-0002".to_string(),
            "PROJ-T-0001".to_string(),
            "todo".to_string(),
            "active".to_string(),
            "human".to_string(),
            Utc::now(),
            false,
        )
        .unwrap();
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_transition_record_empty_fields_invalid() {
        let record = TransitionRecord::new(
            "Some transition".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-TR-0003".to_string(),
            String::new(), // empty document_ref
            "active".to_string(),
            "completed".to_string(),
            "human".to_string(),
            Utc::now(),
            false,
        )
        .unwrap();
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_check_result_helpers() {
        let pass = CheckResult::passed("test", "ok");
        assert!(pass.passed);

        let fail = CheckResult::failed("test", "not ok");
        assert!(!fail.passed);
    }

    #[test]
    fn test_all_checks_passed() {
        let mut record = make_transition_record();
        assert!(record.all_checks_passed());

        record
            .checks_run
            .push(CheckResult::failed("blocker", "Something failed"));
        assert!(!record.all_checks_passed());
    }

    #[tokio::test]
    async fn test_transition_record_roundtrip() {
        let record = make_transition_record();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-transition-record.md");

        record.to_file(&file_path).await.unwrap();
        let loaded = TransitionRecord::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), record.title());
        assert_eq!(loaded.phase().unwrap(), record.phase().unwrap());
        assert_eq!(loaded.document_ref, record.document_ref);
        assert_eq!(loaded.from_phase, record.from_phase);
        assert_eq!(loaded.to_phase, record.to_phase);
        assert_eq!(loaded.actor, record.actor);
        assert_eq!(loaded.forced, record.forced);
        assert_eq!(loaded.checks_run.len(), record.checks_run.len());
        assert_eq!(loaded.checks_run[0].name, record.checks_run[0].name);
        assert_eq!(loaded.checks_run[0].passed, record.checks_run[0].passed);
        assert_eq!(loaded.reason, record.reason);
    }
}
