use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{DocumentId, Phase, Tag};
use chrono::{DateTime, Utc};
use gray_matter;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use tera::{Context, Tera};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Execution mode for a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Single agent executing autonomously.
    SingleAgent,
    /// Multiple agents orchestrated together.
    Orchestrated,
    /// Human-driven manual execution.
    Manual,
}

impl fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SingleAgent => write!(f, "single_agent"),
            Self::Orchestrated => write!(f, "orchestrated"),
            Self::Manual => write!(f, "manual"),
        }
    }
}

impl FromStr for ExecutionMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "single_agent" | "single" => Ok(Self::SingleAgent),
            "orchestrated" | "multi_agent" => Ok(Self::Orchestrated),
            "manual" | "human" => Ok(Self::Manual),
            _ => Err(format!("Unknown execution mode: {s}")),
        }
    }
}

/// Final disposition of an execution run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Disposition {
    /// Run completed successfully.
    Completed,
    /// Run failed.
    Failed,
    /// Run is blocked by a dependency.
    Blocked,
    /// Run was abandoned.
    Abandoned,
    /// Run is still in progress.
    InProgress,
}

impl fmt::Display for Disposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Blocked => write!(f, "blocked"),
            Self::Abandoned => write!(f, "abandoned"),
            Self::InProgress => write!(f, "in_progress"),
        }
    }
}

impl FromStr for Disposition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "completed" | "complete" | "done" => Ok(Self::Completed),
            "failed" | "failure" => Ok(Self::Failed),
            "blocked" => Ok(Self::Blocked),
            "abandoned" | "cancelled" => Ok(Self::Abandoned),
            "in_progress" | "running" | "active" => Ok(Self::InProgress),
            _ => Err(format!("Unknown disposition: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Sub-structs
// ---------------------------------------------------------------------------

/// An entry recording a tool that was run during execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolEntry {
    pub name: String,
    pub arguments: String,
    pub result_summary: String,
}

impl ToolEntry {
    pub fn new(name: &str, arguments: &str, result_summary: &str) -> Self {
        Self {
            name: name.to_string(),
            arguments: arguments.to_string(),
            result_summary: result_summary.to_string(),
        }
    }
}

/// An entry recording a validation that was run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationEntry {
    pub name: String,
    pub passed: bool,
    pub details: String,
}

impl ValidationEntry {
    pub fn new(name: &str, passed: bool, details: &str) -> Self {
        Self {
            name: name.to_string(),
            passed,
            details: details.to_string(),
        }
    }
}

/// An escalation raised during execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscalationEntry {
    pub reason: String,
    pub escalated_to: String,
    pub resolution: String,
}

impl EscalationEntry {
    pub fn new(reason: &str, escalated_to: &str, resolution: &str) -> Self {
        Self {
            reason: reason.to_string(),
            escalated_to: escalated_to.to_string(),
            resolution: resolution.to_string(),
        }
    }
}

/// An override applied during execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverrideEntry {
    pub rule: String,
    pub reason: String,
    pub approved_by: String,
}

impl OverrideEntry {
    pub fn new(rule: &str, reason: &str, approved_by: &str) -> Self {
        Self {
            rule: rule.to_string(),
            reason: reason.to_string(),
            approved_by: approved_by.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// ExecutionRecord
// ---------------------------------------------------------------------------

/// A durable record of a work execution run, linking intent to outcome.
///
/// This is the audit spine of the system -- captures what happened, why, and
/// with what evidence for any piece of work.
#[derive(Debug)]
pub struct ExecutionRecord {
    core: DocumentCore,
    /// The artifact (story/task short code) that initiated this execution.
    pub initiating_artifact: String,
    /// How the execution was performed.
    pub execution_mode: ExecutionMode,
    /// When execution started.
    pub started_at: DateTime<Utc>,
    /// When execution completed (None if still running).
    pub completed_at: Option<DateTime<Utc>>,
    /// Documents consulted during execution.
    pub context_sources: Vec<String>,
    /// Architecture document consulted, if any.
    pub architecture_consulted: Option<String>,
    /// Rules consulted during execution.
    pub rules_consulted: Vec<String>,
    /// Note IDs fetched during execution.
    pub notes_fetched: Vec<String>,
    /// Tools that were run.
    pub tools_run: Vec<ToolEntry>,
    /// File paths modified during execution.
    pub files_touched: Vec<String>,
    /// Validations that were run.
    pub validations_run: Vec<ValidationEntry>,
    /// Artifacts updated during execution.
    pub artifacts_updated: Vec<String>,
    /// Decisions made (links to DecisionRecord short codes).
    pub decisions_made: Vec<String>,
    /// Escalations raised during execution.
    pub escalations: Vec<EscalationEntry>,
    /// Overrides applied during execution.
    pub overrides: Vec<OverrideEntry>,
    /// Final disposition of the execution.
    pub final_disposition: Disposition,
}

impl ExecutionRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        initiating_artifact: String,
        execution_mode: ExecutionMode,
        started_at: DateTime<Utc>,
        final_disposition: Disposition,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let template_content = include_str!("content.md");
        let mut tera = Tera::default();
        tera.add_raw_template("execution_record_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {e}"))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("execution_record_content", &context)
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
            initiating_artifact,
            execution_mode,
            started_at,
            completed_at: None,
            context_sources: Vec::new(),
            architecture_consulted: None,
            rules_consulted: Vec::new(),
            notes_fetched: Vec::new(),
            tools_run: Vec::new(),
            files_touched: Vec::new(),
            validations_run: Vec::new(),
            artifacts_updated: Vec::new(),
            decisions_made: Vec::new(),
            escalations: Vec::new(),
            overrides: Vec::new(),
            final_disposition,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        initiating_artifact: String,
        execution_mode: ExecutionMode,
        started_at: DateTime<Utc>,
        completed_at: Option<DateTime<Utc>>,
        context_sources: Vec<String>,
        architecture_consulted: Option<String>,
        rules_consulted: Vec<String>,
        notes_fetched: Vec<String>,
        tools_run: Vec<ToolEntry>,
        files_touched: Vec<String>,
        validations_run: Vec<ValidationEntry>,
        artifacts_updated: Vec<String>,
        decisions_made: Vec<String>,
        escalations: Vec<EscalationEntry>,
        overrides: Vec<OverrideEntry>,
        final_disposition: Disposition,
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
            initiating_artifact,
            execution_mode,
            started_at,
            completed_at,
            context_sources,
            architecture_consulted,
            rules_consulted,
            notes_fetched,
            tools_run,
            files_touched,
            validations_run,
            artifacts_updated,
            decisions_made,
            escalations,
            overrides,
            final_disposition,
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
        if level != "execution_record" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'execution_record', found '{level}'"
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

        let initiating_artifact =
            FrontmatterParser::extract_string(&fm_map, "initiating_artifact")?;
        let execution_mode_str = FrontmatterParser::extract_string(&fm_map, "execution_mode")?;
        let execution_mode = ExecutionMode::from_str(&execution_mode_str)
            .map_err(DocumentValidationError::InvalidContent)?;

        let started_at_str = FrontmatterParser::extract_string(&fm_map, "started_at")?;
        let started_at = DateTime::parse_from_rfc3339(&started_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| {
                DocumentValidationError::InvalidContent(
                    "Invalid datetime for started_at".to_string(),
                )
            })?;

        let completed_at = FrontmatterParser::extract_optional_string(&fm_map, "completed_at")
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let final_disposition_str =
            FrontmatterParser::extract_string(&fm_map, "final_disposition")?;
        let final_disposition = Disposition::from_str(&final_disposition_str)
            .map_err(DocumentValidationError::InvalidContent)?;

        let context_sources =
            FrontmatterParser::extract_string_array(&fm_map, "context_sources").unwrap_or_default();
        let architecture_consulted =
            FrontmatterParser::extract_optional_string(&fm_map, "architecture_consulted");
        let rules_consulted =
            FrontmatterParser::extract_string_array(&fm_map, "rules_consulted").unwrap_or_default();
        let notes_fetched =
            FrontmatterParser::extract_string_array(&fm_map, "notes_fetched").unwrap_or_default();
        let files_touched =
            FrontmatterParser::extract_string_array(&fm_map, "files_touched").unwrap_or_default();
        let artifacts_updated =
            FrontmatterParser::extract_string_array(&fm_map, "artifacts_updated")
                .unwrap_or_default();
        let decisions_made =
            FrontmatterParser::extract_string_array(&fm_map, "decisions_made").unwrap_or_default();

        let tools_run = Self::parse_tool_entries(&fm_map)?;
        let validations_run = Self::parse_validation_entries(&fm_map)?;
        let escalations = Self::parse_escalation_entries(&fm_map)?;
        let overrides = Self::parse_override_entries(&fm_map)?;

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
            initiating_artifact,
            execution_mode,
            started_at,
            completed_at,
            context_sources,
            architecture_consulted,
            rules_consulted,
            notes_fetched,
            tools_run,
            files_touched,
            validations_run,
            artifacts_updated,
            decisions_made,
            escalations,
            overrides,
            final_disposition,
        ))
    }

    fn parse_tool_entries(
        fm_map: &std::collections::HashMap<String, gray_matter::Pod>,
    ) -> Result<Vec<ToolEntry>, DocumentValidationError> {
        let arr = match fm_map.get("tools_run") {
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
                let arguments = match map.get("arguments") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => String::new(),
                };
                let result_summary = match map.get("result_summary") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => String::new(),
                };
                entries.push(ToolEntry {
                    name,
                    arguments,
                    result_summary,
                });
            }
        }
        Ok(entries)
    }

    fn parse_validation_entries(
        fm_map: &std::collections::HashMap<String, gray_matter::Pod>,
    ) -> Result<Vec<ValidationEntry>, DocumentValidationError> {
        let arr = match fm_map.get("validations_run") {
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
                entries.push(ValidationEntry {
                    name,
                    passed,
                    details,
                });
            }
        }
        Ok(entries)
    }

    fn parse_escalation_entries(
        fm_map: &std::collections::HashMap<String, gray_matter::Pod>,
    ) -> Result<Vec<EscalationEntry>, DocumentValidationError> {
        let arr = match fm_map.get("escalations") {
            Some(gray_matter::Pod::Array(arr)) => arr,
            _ => return Ok(Vec::new()),
        };

        let mut entries = Vec::new();
        for item in arr {
            if let gray_matter::Pod::Hash(map) = item {
                let reason = match map.get("reason") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => continue,
                };
                let escalated_to = match map.get("escalated_to") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => String::new(),
                };
                let resolution = match map.get("resolution") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => String::new(),
                };
                entries.push(EscalationEntry {
                    reason,
                    escalated_to,
                    resolution,
                });
            }
        }
        Ok(entries)
    }

    fn parse_override_entries(
        fm_map: &std::collections::HashMap<String, gray_matter::Pod>,
    ) -> Result<Vec<OverrideEntry>, DocumentValidationError> {
        let arr = match fm_map.get("overrides") {
            Some(gray_matter::Pod::Array(arr)) => arr,
            _ => return Ok(Vec::new()),
        };

        let mut entries = Vec::new();
        for item in arr {
            if let gray_matter::Pod::Hash(map) = item {
                let rule = match map.get("rule") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => continue,
                };
                let reason = match map.get("reason") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => String::new(),
                };
                let approved_by = match map.get("approved_by") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => String::new(),
                };
                entries.push(OverrideEntry {
                    rule,
                    reason,
                    approved_by,
                });
            }
        }
        Ok(entries)
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

        context.insert("initiating_artifact", &self.initiating_artifact);
        context.insert("execution_mode", &self.execution_mode.to_string());
        context.insert("started_at", &self.started_at.to_rfc3339());
        context.insert(
            "completed_at",
            &self
                .completed_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "NULL".to_string()),
        );
        context.insert("final_disposition", &self.final_disposition.to_string());

        context.insert("context_sources", &self.context_sources);
        context.insert(
            "architecture_consulted",
            &self.architecture_consulted.as_deref().unwrap_or("NULL"),
        );
        context.insert("rules_consulted", &self.rules_consulted);
        context.insert("notes_fetched", &self.notes_fetched);
        context.insert("files_touched", &self.files_touched);
        context.insert("artifacts_updated", &self.artifacts_updated);
        context.insert("decisions_made", &self.decisions_made);

        // Serialize complex sub-structs as maps
        let tool_maps: Vec<std::collections::HashMap<&str, String>> = self
            .tools_run
            .iter()
            .map(|t| {
                let mut m = std::collections::HashMap::new();
                m.insert("name", t.name.clone());
                m.insert("arguments", t.arguments.clone());
                m.insert("result_summary", t.result_summary.clone());
                m
            })
            .collect();
        context.insert("tools_run", &tool_maps);

        let validation_maps: Vec<serde_json::Value> = self
            .validations_run
            .iter()
            .map(|v| {
                serde_json::json!({
                    "name": v.name,
                    "passed": v.passed,
                    "details": v.details,
                })
            })
            .collect();
        context.insert("validations_run", &validation_maps);

        let escalation_maps: Vec<std::collections::HashMap<&str, String>> = self
            .escalations
            .iter()
            .map(|e| {
                let mut m = std::collections::HashMap::new();
                m.insert("reason", e.reason.clone());
                m.insert("escalated_to", e.escalated_to.clone());
                m.insert("resolution", e.resolution.clone());
                m
            })
            .collect();
        context.insert("escalations", &escalation_maps);

        let override_maps: Vec<std::collections::HashMap<&str, String>> = self
            .overrides
            .iter()
            .map(|o| {
                let mut m = std::collections::HashMap::new();
                m.insert("rule", o.rule.clone());
                m.insert("reason", o.reason.clone());
                m.insert("approved_by", o.approved_by.clone());
                m
            })
            .collect();
        context.insert("overrides", &override_maps);

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
                "ExecutionRecord title cannot be empty".to_string(),
            ));
        }
        if self.initiating_artifact.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "initiating_artifact".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_execution_record() -> ExecutionRecord {
        let mut record = ExecutionRecord::new(
            "Run: implement login feature".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-ER-0001".to_string(),
            "PROJ-T-0042".to_string(),
            ExecutionMode::SingleAgent,
            Utc::now(),
            Disposition::Completed,
        )
        .unwrap();

        record.context_sources = vec!["PROJ-S-0001".to_string(), "PROJ-D-0003".to_string()];
        record.architecture_consulted = Some("PROJ-ARCH-0001".to_string());
        record.rules_consulted = vec!["PROJ-RC-0001".to_string()];
        record.notes_fetched = vec!["note-auth-pattern".to_string()];
        record.tools_run = vec![ToolEntry::new("cargo test", "--all", "42 tests passed")];
        record.files_touched = vec!["src/auth.rs".to_string(), "src/login.rs".to_string()];
        record.validations_run = vec![ValidationEntry::new("clippy", true, "No warnings")];
        record.artifacts_updated = vec!["PROJ-T-0042".to_string()];
        record.decisions_made = vec!["PROJ-DR-0001".to_string()];
        record.completed_at = Some(Utc::now());
        record
    }

    #[test]
    fn test_execution_record_creation() {
        let record = make_execution_record();
        assert_eq!(record.title(), "Run: implement login feature");
        assert_eq!(record.phase().unwrap(), Phase::Draft);
        assert_eq!(record.execution_mode, ExecutionMode::SingleAgent);
        assert_eq!(record.final_disposition, Disposition::Completed);
        assert_eq!(record.context_sources.len(), 2);
        assert_eq!(record.tools_run.len(), 1);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_execution_record_empty_title_invalid() {
        let record = ExecutionRecord::new(
            String::new(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-ER-0002".to_string(),
            "PROJ-T-0001".to_string(),
            ExecutionMode::Manual,
            Utc::now(),
            Disposition::InProgress,
        )
        .unwrap();
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_execution_record_empty_artifact_invalid() {
        let record = ExecutionRecord::new(
            "Some run".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-ER-0003".to_string(),
            String::new(),
            ExecutionMode::Manual,
            Utc::now(),
            Disposition::InProgress,
        )
        .unwrap();
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_execution_mode_parsing() {
        assert_eq!(
            "single_agent".parse::<ExecutionMode>().unwrap(),
            ExecutionMode::SingleAgent
        );
        assert_eq!(
            "orchestrated".parse::<ExecutionMode>().unwrap(),
            ExecutionMode::Orchestrated
        );
        assert_eq!(
            "manual".parse::<ExecutionMode>().unwrap(),
            ExecutionMode::Manual
        );
        assert!("invalid".parse::<ExecutionMode>().is_err());
    }

    #[test]
    fn test_disposition_parsing() {
        assert_eq!(
            "completed".parse::<Disposition>().unwrap(),
            Disposition::Completed
        );
        assert_eq!(
            "failed".parse::<Disposition>().unwrap(),
            Disposition::Failed
        );
        assert_eq!(
            "blocked".parse::<Disposition>().unwrap(),
            Disposition::Blocked
        );
        assert_eq!(
            "abandoned".parse::<Disposition>().unwrap(),
            Disposition::Abandoned
        );
        assert_eq!(
            "in_progress".parse::<Disposition>().unwrap(),
            Disposition::InProgress
        );
        assert!("unknown".parse::<Disposition>().is_err());
    }

    #[tokio::test]
    async fn test_execution_record_roundtrip() {
        let record = make_execution_record();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-execution-record.md");

        record.to_file(&file_path).await.unwrap();
        let loaded = ExecutionRecord::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), record.title());
        assert_eq!(loaded.phase().unwrap(), record.phase().unwrap());
        assert_eq!(loaded.execution_mode, record.execution_mode);
        assert_eq!(loaded.final_disposition, record.final_disposition);
        assert_eq!(loaded.initiating_artifact, record.initiating_artifact);
        assert_eq!(loaded.context_sources.len(), record.context_sources.len());
        assert_eq!(loaded.rules_consulted.len(), record.rules_consulted.len());
        assert_eq!(loaded.tools_run.len(), record.tools_run.len());
        assert_eq!(loaded.tools_run[0].name, record.tools_run[0].name);
        assert_eq!(loaded.files_touched.len(), record.files_touched.len());
        assert_eq!(loaded.validations_run.len(), record.validations_run.len());
        assert_eq!(loaded.decisions_made.len(), record.decisions_made.len());
        assert_eq!(loaded.architecture_consulted, record.architecture_consulted);
    }
}
