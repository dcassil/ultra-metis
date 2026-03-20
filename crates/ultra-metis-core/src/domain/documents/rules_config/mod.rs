use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{Phase, Tag};
use chrono::Utc;
use gray_matter;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;
use tera::{Context, Tera};

/// Protection level determines how a RulesConfig can be modified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtectionLevel {
    /// Standard rules can be edited directly.
    Standard,
    /// Protected rules require a DesignChangeProposal to modify.
    Protected,
}

impl fmt::Display for ProtectionLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtectionLevel::Standard => write!(f, "standard"),
            ProtectionLevel::Protected => write!(f, "protected"),
        }
    }
}

impl FromStr for ProtectionLevel {
    type Err = DocumentValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standard" => Ok(ProtectionLevel::Standard),
            "protected" => Ok(ProtectionLevel::Protected),
            _ => Err(DocumentValidationError::InvalidContent(format!(
                "Unknown protection_level '{}': expected 'standard' or 'protected'",
                s
            ))),
        }
    }
}

/// The scope at which a RulesConfig applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleScope {
    Platform,
    Organization,
    Repository,
    Package,
    Component,
    Task,
}

impl fmt::Display for RuleScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleScope::Platform => write!(f, "platform"),
            RuleScope::Organization => write!(f, "org"),
            RuleScope::Repository => write!(f, "repo"),
            RuleScope::Package => write!(f, "package"),
            RuleScope::Component => write!(f, "component"),
            RuleScope::Task => write!(f, "task"),
        }
    }
}

impl FromStr for RuleScope {
    type Err = DocumentValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "platform" => Ok(RuleScope::Platform),
            "organization" | "org" => Ok(RuleScope::Organization),
            "repository" | "repo" => Ok(RuleScope::Repository),
            "package" => Ok(RuleScope::Package),
            "component" => Ok(RuleScope::Component),
            "task" => Ok(RuleScope::Task),
            _ => Err(DocumentValidationError::InvalidContent(format!(
                "Unknown scope '{}': expected platform/org/repo/package/component/task",
                s
            ))),
        }
    }
}

/// Category of engineering rule stored in a RulesConfig.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleCategory {
    Behavioral,
    Architectural,
    Operational,
    InformationHandling,
    DecisionMaking,
    ValidationQuality,
    ApprovalEscalation,
    ExecutionSafety,
}

impl fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleCategory::Behavioral => write!(f, "behavioral"),
            RuleCategory::Architectural => write!(f, "architectural"),
            RuleCategory::Operational => write!(f, "operational"),
            RuleCategory::InformationHandling => write!(f, "information_handling"),
            RuleCategory::DecisionMaking => write!(f, "decision_making"),
            RuleCategory::ValidationQuality => write!(f, "validation_quality"),
            RuleCategory::ApprovalEscalation => write!(f, "approval_escalation"),
            RuleCategory::ExecutionSafety => write!(f, "execution_safety"),
        }
    }
}

use std::fmt;

/// A RulesConfig stores protected engineering rules that govern system behaviour.
///
/// It is a cross-cutting governance artifact with no required parent.
/// Phases: Draft → Review → Published
#[derive(Debug)]
pub struct RulesConfig {
    core: DocumentCore,
    pub protection_level: ProtectionLevel,
    pub scope: RuleScope,
    pub source_architecture_ref: Option<String>,
}

impl RulesConfig {
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        protection_level: ProtectionLevel,
        scope: RuleScope,
        source_architecture_ref: Option<String>,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            tags,
            archived,
            short_code,
            protection_level,
            scope,
            source_architecture_ref,
            template_content,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_template(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        protection_level: ProtectionLevel,
        scope: RuleScope,
        source_architecture_ref: Option<String>,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("rules_config_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera.render("rules_config_content", &context).map_err(|e| {
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
            protection_level,
            scope,
            source_architecture_ref,
        })
    }

    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        protection_level: ProtectionLevel,
        scope: RuleScope,
        source_architecture_ref: Option<String>,
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
            protection_level,
            scope,
            source_architecture_ref,
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
        if level != "rules_config" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'rules_config', found '{}'",
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

        let protection_level_str = FrontmatterParser::extract_string(&fm_map, "protection_level")?;
        let protection_level = protection_level_str.parse::<ProtectionLevel>()?;

        let scope_str = FrontmatterParser::extract_string(&fm_map, "scope")?;
        let scope = scope_str.parse::<RuleScope>()?;

        let source_architecture_ref =
            FrontmatterParser::extract_optional_string(&fm_map, "source_architecture_ref");

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            tags,
            archived,
            protection_level,
            scope,
            source_architecture_ref,
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

    /// Get mutable access to the document core
    pub fn core_mut(&mut self) -> &mut DocumentCore {
        &mut self.core
    }

    pub fn phase(&self) -> Result<Phase, DocumentValidationError> {
        for tag in self.tags() {
            if let Tag::Phase(phase) = tag {
                return Ok(*phase);
            }
        }
        Err(DocumentValidationError::MissingPhaseTag)
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
        context.insert("protection_level", &self.protection_level.to_string());
        context.insert("scope", &self.scope.to_string());
        context.insert(
            "source_architecture_ref",
            &self
                .source_architecture_ref
                .as_deref()
                .unwrap_or("NULL")
                .to_string(),
        );

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

    fn make_rules_config(title: &str, short_code: &str) -> RulesConfig {
        RulesConfig::new(
            title.to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            short_code.to_string(),
            ProtectionLevel::Standard,
            RuleScope::Repository,
            None,
        )
        .unwrap()
    }

    #[test]
    fn test_rules_config_creation() {
        let rc = make_rules_config("My Rules", "RC-0001");

        assert_eq!(rc.title(), "My Rules");
        assert_eq!(rc.phase().unwrap(), Phase::Draft);
        assert!(!rc.archived());
        assert_eq!(rc.protection_level, ProtectionLevel::Standard);
        assert_eq!(rc.scope, RuleScope::Repository);
        assert!(rc.source_architecture_ref.is_none());
    }

    #[test]
    fn test_rules_config_with_source_ref() {
        let rc = RulesConfig::new(
            "Platform Rules".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "RC-0002".to_string(),
            ProtectionLevel::Protected,
            RuleScope::Platform,
            Some("arch-v2-platform".to_string()),
        )
        .unwrap();

        assert_eq!(rc.protection_level, ProtectionLevel::Protected);
        assert_eq!(rc.scope, RuleScope::Platform);
        assert_eq!(
            rc.source_architecture_ref.as_deref(),
            Some("arch-v2-platform")
        );
    }

    #[tokio::test]
    async fn test_rules_config_roundtrip() {
        let rc = RulesConfig::new(
            "Test Rules Config".to_string(),
            vec![
                Tag::Label("rules_config".to_string()),
                Tag::Phase(Phase::Draft),
            ],
            false,
            "RC-0003".to_string(),
            ProtectionLevel::Protected,
            RuleScope::Component,
            Some("dc-core-arch".to_string()),
        )
        .unwrap();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-rules-config.md");

        rc.to_file(&file_path).await.unwrap();
        let loaded = RulesConfig::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), rc.title());
        assert_eq!(loaded.phase().unwrap(), rc.phase().unwrap());
        assert_eq!(loaded.tags().len(), rc.tags().len());
        assert_eq!(loaded.protection_level, rc.protection_level);
        assert_eq!(loaded.scope, rc.scope);
        assert_eq!(loaded.source_architecture_ref, rc.source_architecture_ref);
    }

    #[test]
    fn test_rules_config_transitions() {
        let mut rc = make_rules_config("Transition Rules", "RC-0004");

        assert!(rc.can_transition_to(Phase::Review));
        assert!(!rc.can_transition_to(Phase::Published));

        let new_phase = rc.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Review);

        let new_phase = rc.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Published);

        // Terminal — stays Published
        let new_phase = rc.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Published);
    }

    #[test]
    fn test_invalid_transition() {
        let mut rc = make_rules_config("Invalid Transition Rules", "RC-0005");

        let err = rc.transition_phase(Some(Phase::Published)).unwrap_err();
        assert!(matches!(
            err,
            DocumentValidationError::InvalidPhaseTransition {
                from: Phase::Draft,
                to: Phase::Published
            }
        ));
    }

    #[test]
    fn test_protection_level_parsing() {
        assert_eq!(
            "standard".parse::<ProtectionLevel>().unwrap(),
            ProtectionLevel::Standard
        );
        assert_eq!(
            "protected".parse::<ProtectionLevel>().unwrap(),
            ProtectionLevel::Protected
        );
        assert!("unknown".parse::<ProtectionLevel>().is_err());
    }

    #[test]
    fn test_rule_scope_parsing() {
        assert_eq!(
            "platform".parse::<RuleScope>().unwrap(),
            RuleScope::Platform
        );
        assert_eq!("org".parse::<RuleScope>().unwrap(), RuleScope::Organization);
        assert_eq!(
            "organization".parse::<RuleScope>().unwrap(),
            RuleScope::Organization
        );
        assert_eq!("repo".parse::<RuleScope>().unwrap(), RuleScope::Repository);
        assert_eq!(
            "repository".parse::<RuleScope>().unwrap(),
            RuleScope::Repository
        );
        assert_eq!("package".parse::<RuleScope>().unwrap(), RuleScope::Package);
        assert_eq!(
            "component".parse::<RuleScope>().unwrap(),
            RuleScope::Component
        );
        assert_eq!("task".parse::<RuleScope>().unwrap(), RuleScope::Task);
        assert!("unknown".parse::<RuleScope>().is_err());
    }

    #[test]
    fn test_from_content_invalid_level() {
        let bad_content = "---\n\
id: test\n\
level: product_doc\n\
title: \"Bad Level\"\n\
short_code: \"RC-0099\"\n\
created_at: 2026-01-01T00:00:00Z\n\
updated_at: 2026-01-01T00:00:00Z\n\
archived: false\n\
tags:\n\
  - \"#phase/draft\"\n\
exit_criteria_met: false\n\
schema_version: 1\n\
epic_id: NULL\n\
protection_level: \"standard\"\n\
scope: \"repo\"\n\
source_architecture_ref: NULL\n\
---\n\
\n\
# Bad Level\n";
        let err = RulesConfig::from_content(bad_content).unwrap_err();
        assert!(matches!(err, DocumentValidationError::InvalidContent(_)));
    }
}
