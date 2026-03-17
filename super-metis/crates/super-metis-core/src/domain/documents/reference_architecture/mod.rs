use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{Phase, Tag};
use chrono::Utc;
use gray_matter;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::str::FromStr;

use tera::{Context, Tera};

/// Extract a string array from frontmatter, returning empty vec for null/missing.
fn extract_string_array_or_empty(
    map: &std::collections::HashMap<String, gray_matter::Pod>,
    key: &str,
) -> Vec<String> {
    match map.get(key) {
        Some(gray_matter::Pod::Array(arr)) => {
            arr.iter()
                .filter_map(|item| {
                    if let gray_matter::Pod::String(s) = item {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

/// Status of a reference architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchitectureStatus {
    /// Initial state while being configured.
    Draft,
    /// The active governing architecture for this repo.
    Active,
    /// Replaced by a newer reference architecture.
    Superseded,
}

impl fmt::Display for ArchitectureStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchitectureStatus::Draft => write!(f, "draft"),
            ArchitectureStatus::Active => write!(f, "active"),
            ArchitectureStatus::Superseded => write!(f, "superseded"),
        }
    }
}

impl FromStr for ArchitectureStatus {
    type Err = DocumentValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(ArchitectureStatus::Draft),
            "active" => Ok(ArchitectureStatus::Active),
            "superseded" => Ok(ArchitectureStatus::Superseded),
            _ => Err(DocumentValidationError::InvalidContent(format!(
                "Unknown architecture status '{}': expected 'draft', 'active', or 'superseded'",
                s
            ))),
        }
    }
}

/// The selected or derived architecture for a specific repo.
///
/// One per repo (or per workspace in a monorepo). Links to a catalog entry
/// if matched, or contains a derived pattern. Holds references to RulesConfig
/// and AnalysisBaseline that were seeded from this architecture.
/// Phases: Draft -> Review -> Published
#[derive(Debug)]
pub struct ReferenceArchitecture {
    core: DocumentCore,
    pub source_catalog_ref: Option<String>,
    pub is_derived: bool,
    pub status: ArchitectureStatus,
    pub layer_overrides: Vec<String>,
    pub additional_boundaries: Vec<String>,
    pub extra_dependency_rules: Vec<String>,
    pub rules_config_ref: Option<String>,
    pub analysis_baseline_ref: Option<String>,
    pub tolerated_exceptions: Vec<String>,
}

impl ReferenceArchitecture {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        source_catalog_ref: Option<String>,
        is_derived: bool,
        status: ArchitectureStatus,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            tags,
            archived,
            short_code,
            source_catalog_ref,
            is_derived,
            status,
            template_content,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_template(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        source_catalog_ref: Option<String>,
        is_derived: bool,
        status: ArchitectureStatus,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("reference_architecture_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("reference_architecture_content", &context)
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
            source_catalog_ref,
            is_derived,
            status,
            layer_overrides: Vec::new(),
            additional_boundaries: Vec::new(),
            extra_dependency_rules: Vec::new(),
            rules_config_ref: None,
            analysis_baseline_ref: None,
            tolerated_exceptions: Vec::new(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        source_catalog_ref: Option<String>,
        is_derived: bool,
        status: ArchitectureStatus,
        layer_overrides: Vec<String>,
        additional_boundaries: Vec<String>,
        extra_dependency_rules: Vec<String>,
        rules_config_ref: Option<String>,
        analysis_baseline_ref: Option<String>,
        tolerated_exceptions: Vec<String>,
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
            source_catalog_ref,
            is_derived,
            status,
            layer_overrides,
            additional_boundaries,
            extra_dependency_rules,
            rules_config_ref,
            analysis_baseline_ref,
            tolerated_exceptions,
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
        if level != "reference_architecture" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'reference_architecture', found '{}'",
                level
            )));
        }

        let short_code = FrontmatterParser::extract_string(&fm_map, "short_code")?;
        let metadata =
            DocumentMetadata::from_frontmatter(created_at, updated_at, exit_criteria_met, short_code);
        let content = DocumentContent::from_markdown(&parsed.content);

        let source_catalog_ref =
            FrontmatterParser::extract_optional_string(&fm_map, "source_catalog_ref");
        let is_derived = FrontmatterParser::extract_bool(&fm_map, "is_derived").unwrap_or(false);
        let status_str = FrontmatterParser::extract_string(&fm_map, "status")?;
        let status = status_str.parse::<ArchitectureStatus>()?;

        let layer_overrides = extract_string_array_or_empty(&fm_map, "layer_overrides");
        let additional_boundaries =
            extract_string_array_or_empty(&fm_map, "additional_boundaries");
        let extra_dependency_rules =
            extract_string_array_or_empty(&fm_map, "extra_dependency_rules");
        let rules_config_ref =
            FrontmatterParser::extract_optional_string(&fm_map, "rules_config_ref");
        let analysis_baseline_ref =
            FrontmatterParser::extract_optional_string(&fm_map, "analysis_baseline_ref");
        let tolerated_exceptions =
            extract_string_array_or_empty(&fm_map, "tolerated_exceptions");

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            tags,
            archived,
            source_catalog_ref,
            is_derived,
            status,
            layer_overrides,
            additional_boundaries,
            extra_dependency_rules,
            rules_config_ref,
            analysis_baseline_ref,
            tolerated_exceptions,
        ))
    }

    // --- accessors ---

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

    pub fn phase(&self) -> Result<Phase, DocumentValidationError> {
        for tag in self.tags() {
            if let Tag::Phase(phase) = tag {
                return Ok(*phase);
            }
        }
        Err(DocumentValidationError::MissingPhaseTag)
    }

    /// Whether this reference architecture is linked to a catalog entry.
    pub fn is_catalog_linked(&self) -> bool {
        self.source_catalog_ref.is_some() && !self.is_derived
    }

    // --- phase management ---

    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        match current {
            Phase::Draft => Some(Phase::Review),
            Phase::Review => Some(Phase::Published),
            Phase::Published => None,
            _ => None,
        }
    }

    fn valid_transitions_from(current: Phase) -> Vec<Phase> {
        match current {
            Phase::Draft => vec![Phase::Review],
            Phase::Review => vec![Phase::Published],
            _ => vec![],
        }
    }

    pub fn can_transition_to(&self, phase: Phase) -> bool {
        if let Ok(current_phase) = self.phase() {
            Self::valid_transitions_from(current_phase).contains(&phase)
        } else {
            false
        }
    }

    fn update_phase_tag(&mut self, new_phase: Phase) {
        self.core.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
        self.core.tags.push(Tag::Phase(new_phase));
        self.core.metadata.updated_at = Utc::now();
    }

    pub fn transition_phase(
        &mut self,
        target_phase: Option<Phase>,
    ) -> Result<Phase, DocumentValidationError> {
        let current_phase = self.phase()?;
        let new_phase = match target_phase {
            Some(phase) => {
                if !self.can_transition_to(phase) {
                    return Err(DocumentValidationError::InvalidPhaseTransition {
                        from: current_phase,
                        to: phase,
                    });
                }
                phase
            }
            None => match Self::next_phase_in_sequence(current_phase) {
                Some(next) => next,
                None => return Ok(current_phase),
            },
        };
        self.update_phase_tag(new_phase);
        Ok(new_phase)
    }

    // --- serialisation ---

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
        context.insert("slug", &self.core.metadata.short_code);
        context.insert("title", self.title());
        context.insert("short_code", &self.core.metadata.short_code);
        context.insert("created_at", &self.core.metadata.created_at.to_rfc3339());
        context.insert("updated_at", &self.core.metadata.updated_at.to_rfc3339());
        context.insert("archived", &self.archived().to_string());
        context.insert(
            "exit_criteria_met",
            &self.core.metadata.exit_criteria_met.to_string(),
        );

        let tag_strings: Vec<String> = self.tags().iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);
        context.insert("epic_id", "NULL");
        context.insert(
            "source_catalog_ref",
            &self
                .source_catalog_ref
                .as_deref()
                .unwrap_or("NULL")
                .to_string(),
        );
        context.insert("is_derived", &self.is_derived.to_string());
        context.insert("status", &self.status.to_string());
        context.insert("layer_overrides", &self.layer_overrides);
        context.insert("additional_boundaries", &self.additional_boundaries);
        context.insert("extra_dependency_rules", &self.extra_dependency_rules);
        context.insert(
            "rules_config_ref",
            &self
                .rules_config_ref
                .as_deref()
                .unwrap_or("NULL")
                .to_string(),
        );
        context.insert(
            "analysis_baseline_ref",
            &self
                .analysis_baseline_ref
                .as_deref()
                .unwrap_or("NULL")
                .to_string(),
        );
        context.insert("tolerated_exceptions", &self.tolerated_exceptions);

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
    use tempfile::tempdir;

    fn make_catalog_linked(title: &str, short_code: &str) -> ReferenceArchitecture {
        ReferenceArchitecture::new(
            title.to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            short_code.to_string(),
            Some("AC-0001".to_string()),
            false,
            ArchitectureStatus::Draft,
        )
        .unwrap()
    }

    fn make_derived(title: &str, short_code: &str) -> ReferenceArchitecture {
        ReferenceArchitecture::new(
            title.to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            short_code.to_string(),
            None,
            true,
            ArchitectureStatus::Draft,
        )
        .unwrap()
    }

    #[test]
    fn test_catalog_linked_creation() {
        let ra = make_catalog_linked("My Repo Architecture", "RA-0001");

        assert_eq!(ra.title(), "My Repo Architecture");
        assert_eq!(ra.phase().unwrap(), Phase::Draft);
        assert!(!ra.archived());
        assert_eq!(ra.source_catalog_ref.as_deref(), Some("AC-0001"));
        assert!(!ra.is_derived);
        assert_eq!(ra.status, ArchitectureStatus::Draft);
        assert!(ra.is_catalog_linked());
    }

    #[test]
    fn test_derived_creation() {
        let ra = make_derived("Derived Architecture", "RA-0002");

        assert!(ra.source_catalog_ref.is_none());
        assert!(ra.is_derived);
        assert!(!ra.is_catalog_linked());
    }

    #[test]
    fn test_reference_architecture_with_governance_linkage() {
        let ra = ReferenceArchitecture::from_parts(
            "Full Ref Arch".to_string(),
            DocumentMetadata::new("RA-0003".to_string()),
            DocumentContent::new("# Full Ref Arch"),
            vec![Tag::Phase(Phase::Draft)],
            false,
            Some("AC-0001".to_string()),
            false,
            ArchitectureStatus::Active,
            vec!["extra-layer".to_string()],
            vec!["api-boundary".to_string()],
            vec!["no direct db access from handlers".to_string()],
            Some("RC-0001".to_string()),
            Some("AB-0001".to_string()),
            vec!["legacy-module uses old pattern".to_string()],
        );

        assert_eq!(ra.status, ArchitectureStatus::Active);
        assert_eq!(ra.rules_config_ref.as_deref(), Some("RC-0001"));
        assert_eq!(ra.analysis_baseline_ref.as_deref(), Some("AB-0001"));
        assert_eq!(ra.layer_overrides.len(), 1);
        assert_eq!(ra.additional_boundaries.len(), 1);
        assert_eq!(ra.extra_dependency_rules.len(), 1);
        assert_eq!(ra.tolerated_exceptions.len(), 1);
    }

    #[test]
    fn test_content_roundtrip_catalog_linked() {
        let ra = ReferenceArchitecture::from_parts(
            "Roundtrip Catalog".to_string(),
            DocumentMetadata::new("RA-0004".to_string()),
            DocumentContent::new("# Roundtrip Catalog\n\nLinked to catalog."),
            vec![
                Tag::Label("reference_architecture".to_string()),
                Tag::Phase(Phase::Draft),
            ],
            false,
            Some("AC-0010".to_string()),
            false,
            ArchitectureStatus::Draft,
            vec!["override-1".to_string()],
            vec![],
            vec!["dep-rule-1".to_string()],
            Some("RC-0005".to_string()),
            None,
            vec!["exception-1".to_string()],
        );

        let serialized = ra.to_content().unwrap();
        let loaded = ReferenceArchitecture::from_content(&serialized).unwrap();

        assert_eq!(loaded.title(), ra.title());
        assert_eq!(loaded.phase().unwrap(), ra.phase().unwrap());
        assert_eq!(loaded.source_catalog_ref, ra.source_catalog_ref);
        assert_eq!(loaded.is_derived, ra.is_derived);
        assert_eq!(loaded.status, ra.status);
        assert_eq!(loaded.layer_overrides, ra.layer_overrides);
        assert_eq!(loaded.additional_boundaries, ra.additional_boundaries);
        assert_eq!(loaded.extra_dependency_rules, ra.extra_dependency_rules);
        assert_eq!(loaded.rules_config_ref, ra.rules_config_ref);
        assert_eq!(loaded.analysis_baseline_ref, ra.analysis_baseline_ref);
        assert_eq!(loaded.tolerated_exceptions, ra.tolerated_exceptions);
    }

    #[test]
    fn test_content_roundtrip_derived() {
        let ra = ReferenceArchitecture::from_parts(
            "Roundtrip Derived".to_string(),
            DocumentMetadata::new("RA-0005".to_string()),
            DocumentContent::new("# Roundtrip Derived"),
            vec![Tag::Phase(Phase::Draft)],
            false,
            None,
            true,
            ArchitectureStatus::Draft,
            vec![],
            vec![],
            vec![],
            None,
            None,
            vec![],
        );

        let serialized = ra.to_content().unwrap();
        let loaded = ReferenceArchitecture::from_content(&serialized).unwrap();

        assert_eq!(loaded.title(), ra.title());
        assert!(loaded.source_catalog_ref.is_none());
        assert!(loaded.is_derived);
        assert!(loaded.rules_config_ref.is_none());
        assert!(loaded.analysis_baseline_ref.is_none());
    }

    #[tokio::test]
    async fn test_file_roundtrip() {
        let ra = ReferenceArchitecture::from_parts(
            "File Roundtrip".to_string(),
            DocumentMetadata::new("RA-0006".to_string()),
            DocumentContent::new("# File Roundtrip"),
            vec![Tag::Phase(Phase::Draft)],
            false,
            Some("AC-0001".to_string()),
            false,
            ArchitectureStatus::Active,
            vec![],
            vec!["boundary-1".to_string()],
            vec![],
            Some("RC-0001".to_string()),
            Some("AB-0001".to_string()),
            vec![],
        );

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-ref-arch.md");

        ra.to_file(&file_path).await.unwrap();
        let loaded = ReferenceArchitecture::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), ra.title());
        assert_eq!(loaded.source_catalog_ref, ra.source_catalog_ref);
        assert_eq!(loaded.status, ra.status);
        assert_eq!(loaded.additional_boundaries, ra.additional_boundaries);
        assert_eq!(loaded.rules_config_ref, ra.rules_config_ref);
        assert_eq!(loaded.analysis_baseline_ref, ra.analysis_baseline_ref);
    }

    #[test]
    fn test_transitions() {
        let mut ra = make_catalog_linked("Transition Test", "RA-0007");

        assert!(ra.can_transition_to(Phase::Review));
        assert!(!ra.can_transition_to(Phase::Published));

        let new_phase = ra.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Review);

        let new_phase = ra.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Published);

        // Terminal
        let new_phase = ra.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Published);
    }

    #[test]
    fn test_invalid_transition() {
        let mut ra = make_derived("Invalid Transition", "RA-0008");

        let err = ra.transition_phase(Some(Phase::Published)).unwrap_err();
        assert!(matches!(
            err,
            DocumentValidationError::InvalidPhaseTransition {
                from: Phase::Draft,
                to: Phase::Published
            }
        ));
    }

    #[test]
    fn test_architecture_status_parsing() {
        assert_eq!(
            "draft".parse::<ArchitectureStatus>().unwrap(),
            ArchitectureStatus::Draft
        );
        assert_eq!(
            "active".parse::<ArchitectureStatus>().unwrap(),
            ArchitectureStatus::Active
        );
        assert_eq!(
            "superseded".parse::<ArchitectureStatus>().unwrap(),
            ArchitectureStatus::Superseded
        );
        assert!("unknown".parse::<ArchitectureStatus>().is_err());
    }

    #[test]
    fn test_from_content_invalid_level() {
        let bad_content = "---\n\
id: test\n\
level: rules_config\n\
title: \"Bad Level\"\n\
short_code: \"RA-0099\"\n\
created_at: 2026-01-01T00:00:00Z\n\
updated_at: 2026-01-01T00:00:00Z\n\
archived: false\n\
tags:\n\
  - \"#phase/draft\"\n\
exit_criteria_met: false\n\
schema_version: 1\n\
epic_id: NULL\n\
source_catalog_ref: NULL\n\
is_derived: false\n\
status: \"draft\"\n\
layer_overrides: []\n\
additional_boundaries: []\n\
extra_dependency_rules: []\n\
rules_config_ref: NULL\n\
analysis_baseline_ref: NULL\n\
tolerated_exceptions: []\n\
---\n\
\n\
# Bad Level\n";
        let err = ReferenceArchitecture::from_content(bad_content).unwrap_err();
        assert!(matches!(err, DocumentValidationError::InvalidContent(_)));
    }
}
