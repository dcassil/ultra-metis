use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{DocumentId, Phase, Tag};
use gray_matter;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use tera::{Context, Tera};

/// Type of override — emergency (bypass now, review later) or approved (pre-approved bypass).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OverrideType {
    /// Emergency bypass — work must proceed immediately, post-hoc review required.
    Emergency,
    /// Pre-approved bypass — override was approved through normal channels before execution.
    Approved,
}

impl fmt::Display for OverrideType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OverrideType::Emergency => write!(f, "emergency"),
            OverrideType::Approved => write!(f, "approved"),
        }
    }
}

impl FromStr for OverrideType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "emergency" => Ok(OverrideType::Emergency),
            "approved" | "pre-approved" => Ok(OverrideType::Approved),
            _ => Err(format!("Unknown override type: {}", s)),
        }
    }
}

/// Lightweight struct representing the override decision (without document overhead).
/// Can be used in-memory before persisting to a full GateOverrideAuditEntry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateOverride {
    /// Who performed the override (agent ID, human name, etc.)
    pub overrider: String,
    /// Timestamp of the override (RFC 3339).
    pub timestamp: String,
    /// Why the override was necessary.
    pub reason: String,
    /// Metric names of gates that were bypassed.
    pub gates_bypassed: Vec<String>,
    /// Type of override.
    pub override_type: OverrideType,
}

impl GateOverride {
    pub fn new(
        overrider: &str,
        reason: &str,
        gates_bypassed: Vec<String>,
        override_type: OverrideType,
    ) -> Self {
        Self {
            overrider: overrider.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            reason: reason.to_string(),
            gates_bypassed,
            override_type,
        }
    }

    /// Validate that required fields are present.
    pub fn validate(&self) -> Result<(), String> {
        if self.overrider.trim().is_empty() {
            return Err("overrider is required".to_string());
        }
        if self.reason.trim().is_empty() {
            return Err("override reason is required".to_string());
        }
        if self.gates_bypassed.is_empty() {
            return Err("at least one bypassed gate is required".to_string());
        }
        Ok(())
    }
}

/// A durable audit record of a quality gate override.
///
/// This is a write-once record type — once created, it should not be edited.
/// It captures who overrode gates, why, which gates were bypassed, and links
/// to the quality record and gate config involved.
#[derive(Debug)]
pub struct GateOverrideAuditEntry {
    core: DocumentCore,
    /// Who performed the override.
    pub overrider: String,
    /// Why the override was necessary.
    pub override_reason: String,
    /// Type of override (emergency or approved).
    pub override_type: OverrideType,
    /// Metric names of gates that were bypassed.
    pub gates_bypassed: Vec<String>,
    /// Short code of the QualityRecord that triggered the gate failure.
    pub linked_quality_record: Option<String>,
    /// Short code of the QualityGateConfig that defined the thresholds.
    pub linked_gate_config: Option<String>,
}

impl GateOverrideAuditEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        overrider: String,
        override_reason: String,
        override_type: OverrideType,
        gates_bypassed: Vec<String>,
        linked_quality_record: Option<String>,
        linked_gate_config: Option<String>,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            tags,
            archived,
            short_code,
            overrider,
            override_reason,
            override_type,
            gates_bypassed,
            linked_quality_record,
            linked_gate_config,
            template_content,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_template(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        overrider: String,
        override_reason: String,
        override_type: OverrideType,
        gates_bypassed: Vec<String>,
        linked_quality_record: Option<String>,
        linked_gate_config: Option<String>,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("gate_override_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("gate_override_content", &context)
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
            overrider,
            override_reason,
            override_type,
            gates_bypassed,
            linked_quality_record,
            linked_gate_config,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        overrider: String,
        override_reason: String,
        override_type: OverrideType,
        gates_bypassed: Vec<String>,
        linked_quality_record: Option<String>,
        linked_gate_config: Option<String>,
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
            overrider,
            override_reason,
            override_type,
            gates_bypassed,
            linked_quality_record,
            linked_gate_config,
        }
    }

    /// Create from a GateOverride decision and linking information.
    pub fn from_override(
        gate_override: &GateOverride,
        short_code: String,
        linked_quality_record: Option<String>,
        linked_gate_config: Option<String>,
        failure_details: &str,
    ) -> Result<Self, DocumentValidationError> {
        let title = format!(
            "Gate Override: {} ({} gates bypassed)",
            gate_override.override_type,
            gate_override.gates_bypassed.len()
        );

        let mut entry = Self::new(
            title,
            vec![Tag::Phase(Phase::Draft)],
            false,
            short_code,
            gate_override.overrider.clone(),
            gate_override.reason.clone(),
            gate_override.override_type,
            gate_override.gates_bypassed.clone(),
            linked_quality_record,
            linked_gate_config,
        )?;

        // Replace template content with actual failure details
        let body = format!(
            "# Gate Override: {} ({} gates bypassed)\n\n## Failed Gates\n\n{}\n\n## Override Justification\n\n{}\n\n## Approval Chain\n\nOverrider: {}\nTimestamp: {}\nType: {}",
            gate_override.override_type,
            gate_override.gates_bypassed.len(),
            failure_details,
            gate_override.reason,
            gate_override.overrider,
            gate_override.timestamp,
            gate_override.override_type,
        );
        entry.core.content = DocumentContent::new(&body);

        Ok(entry)
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
        if level != "gate_override_audit" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'gate_override_audit', found '{}'",
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

        let overrider = FrontmatterParser::extract_string(&fm_map, "overrider")
            .unwrap_or_default();
        let override_reason = FrontmatterParser::extract_string(&fm_map, "override_reason")
            .unwrap_or_default();
        let override_type = FrontmatterParser::extract_optional_string(&fm_map, "override_type")
            .and_then(|s| OverrideType::from_str(&s).ok())
            .unwrap_or(OverrideType::Emergency);
        let gates_bypassed =
            FrontmatterParser::extract_string_array(&fm_map, "gates_bypassed")
                .unwrap_or_default();
        let linked_quality_record =
            FrontmatterParser::extract_optional_string(&fm_map, "linked_quality_record");
        let linked_gate_config =
            FrontmatterParser::extract_optional_string(&fm_map, "linked_gate_config");

        let metadata =
            DocumentMetadata::from_frontmatter(created_at, updated_at, exit_criteria_met, short_code);
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            tags,
            archived,
            overrider,
            override_reason,
            override_type,
            gates_bypassed,
            linked_quality_record,
            linked_gate_config,
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

        context.insert("overrider", &self.overrider);
        context.insert("override_reason", &self.override_reason);
        context.insert("override_type", &self.override_type.to_string());
        context.insert("gates_bypassed", &self.gates_bypassed);
        context.insert(
            "linked_quality_record",
            &self
                .linked_quality_record
                .as_deref()
                .unwrap_or("NULL"),
        );
        context.insert(
            "linked_gate_config",
            &self
                .linked_gate_config
                .as_deref()
                .unwrap_or("NULL"),
        );

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
                "GateOverrideAuditEntry title cannot be empty".to_string(),
            ));
        }
        if self.overrider.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "overrider".to_string(),
            ));
        }
        if self.override_reason.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "override_reason".to_string(),
            ));
        }
        if self.gates_bypassed.is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "gates_bypassed (at least one gate must be listed)".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_override_entry() -> GateOverrideAuditEntry {
        GateOverrideAuditEntry::new(
            "Gate Override: emergency (2 gates bypassed)".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-GOA-0001".to_string(),
            "agent-001".to_string(),
            "Hotfix for production outage, cannot wait for lint cleanup".to_string(),
            OverrideType::Emergency,
            vec!["lint_errors".to_string(), "warnings".to_string()],
            Some("QR-0001".to_string()),
            Some("QGC-0001".to_string()),
        )
        .unwrap()
    }

    #[test]
    fn test_override_entry_creation() {
        let entry = make_override_entry();

        assert_eq!(
            entry.title(),
            "Gate Override: emergency (2 gates bypassed)"
        );
        assert_eq!(entry.overrider, "agent-001");
        assert_eq!(entry.override_type, OverrideType::Emergency);
        assert_eq!(entry.gates_bypassed.len(), 2);
        assert_eq!(
            entry.linked_quality_record.as_deref(),
            Some("QR-0001")
        );
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_override_entry_missing_overrider_invalid() {
        let entry = GateOverrideAuditEntry::new(
            "Test Override".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-GOA-0002".to_string(),
            String::new(), // missing overrider
            "reason".to_string(),
            OverrideType::Emergency,
            vec!["lint_errors".to_string()],
            None,
            None,
        )
        .unwrap();
        assert!(entry.validate().is_err());
    }

    #[test]
    fn test_override_entry_missing_reason_invalid() {
        let entry = GateOverrideAuditEntry::new(
            "Test Override".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-GOA-0003".to_string(),
            "agent-001".to_string(),
            String::new(), // missing reason
            OverrideType::Approved,
            vec!["lint_errors".to_string()],
            None,
            None,
        )
        .unwrap();
        assert!(entry.validate().is_err());
    }

    #[test]
    fn test_override_entry_empty_gates_invalid() {
        let entry = GateOverrideAuditEntry::new(
            "Test Override".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-GOA-0004".to_string(),
            "agent-001".to_string(),
            "reason".to_string(),
            OverrideType::Emergency,
            vec![], // no gates bypassed
            None,
            None,
        )
        .unwrap();
        assert!(entry.validate().is_err());
    }

    #[test]
    fn test_override_type_parsing() {
        assert_eq!(
            "emergency".parse::<OverrideType>().unwrap(),
            OverrideType::Emergency
        );
        assert_eq!(
            "approved".parse::<OverrideType>().unwrap(),
            OverrideType::Approved
        );
        assert_eq!(
            "pre-approved".parse::<OverrideType>().unwrap(),
            OverrideType::Approved
        );
        assert!("invalid".parse::<OverrideType>().is_err());
    }

    #[test]
    fn test_gate_override_struct() {
        let go = GateOverride::new(
            "agent-001",
            "Production hotfix",
            vec!["lint_errors".to_string()],
            OverrideType::Emergency,
        );

        assert_eq!(go.overrider, "agent-001");
        assert!(go.validate().is_ok());

        // Test validation failures
        let bad_overrider = GateOverride::new(
            "",
            "reason",
            vec!["lint".to_string()],
            OverrideType::Emergency,
        );
        assert!(bad_overrider.validate().is_err());

        let bad_reason = GateOverride::new(
            "agent",
            "",
            vec!["lint".to_string()],
            OverrideType::Emergency,
        );
        assert!(bad_reason.validate().is_err());

        let bad_gates = GateOverride::new("agent", "reason", vec![], OverrideType::Emergency);
        assert!(bad_gates.validate().is_err());
    }

    #[test]
    fn test_from_override_convenience() {
        let go = GateOverride::new(
            "agent-001",
            "Hotfix required",
            vec!["lint_errors".to_string(), "warnings".to_string()],
            OverrideType::Emergency,
        );

        let entry = GateOverrideAuditEntry::from_override(
            &go,
            "TEST-GOA-0005".to_string(),
            Some("QR-0001".to_string()),
            Some("QGC-0001".to_string()),
            "lint_errors: 15 (threshold: 10, exceeded by 5)\nwarnings: 25 (threshold: 0, exceeded by 25)",
        )
        .unwrap();

        assert_eq!(entry.overrider, "agent-001");
        assert_eq!(entry.gates_bypassed.len(), 2);
        assert!(entry.validate().is_ok());
        // Content body should include failure details
        let content = entry.to_content().unwrap();
        assert!(content.contains("lint_errors: 15"));
    }

    #[tokio::test]
    async fn test_override_entry_roundtrip() {
        let entry = make_override_entry();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-gate-override.md");

        entry.to_file(&file_path).await.unwrap();
        let loaded = GateOverrideAuditEntry::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), entry.title());
        assert_eq!(loaded.overrider, entry.overrider);
        assert_eq!(loaded.override_reason, entry.override_reason);
        assert_eq!(loaded.override_type, entry.override_type);
        assert_eq!(loaded.gates_bypassed, entry.gates_bypassed);
        assert_eq!(
            loaded.linked_quality_record,
            entry.linked_quality_record
        );
        assert_eq!(loaded.linked_gate_config, entry.linked_gate_config);
    }
}
