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

/// The type of change being proposed to a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleChangeType {
    /// Adding a new rule to an existing RulesConfig.
    Add,
    /// Modifying an existing rule's content or parameters.
    Modify,
    /// Removing a rule from a RulesConfig.
    Remove,
    /// Changing the protection level of a rule.
    Reclassify,
}

impl fmt::Display for RuleChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleChangeType::Add => write!(f, "add"),
            RuleChangeType::Modify => write!(f, "modify"),
            RuleChangeType::Remove => write!(f, "remove"),
            RuleChangeType::Reclassify => write!(f, "reclassify"),
        }
    }
}

impl FromStr for RuleChangeType {
    type Err = DocumentValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "add" => Ok(RuleChangeType::Add),
            "modify" | "update" | "change" => Ok(RuleChangeType::Modify),
            "remove" | "delete" => Ok(RuleChangeType::Remove),
            "reclassify" | "reclassification" => Ok(RuleChangeType::Reclassify),
            _ => Err(DocumentValidationError::InvalidContent(format!(
                "Unknown rule change type '{}': expected add/modify/remove/reclassify",
                s
            ))),
        }
    }
}

/// Status of a rule change proposal through its workflow.
///
/// Workflow: Proposed -> UnderReview -> Approved/Rejected -> Applied/Superseded
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleChangeStatus {
    /// Initial state — proposal has been created.
    Proposed,
    /// A reviewer is actively evaluating the proposal.
    UnderReview,
    /// The change has been approved and is ready to apply.
    Approved,
    /// The change was rejected with documented reasons.
    Rejected,
    /// The approved change has been applied to the target RulesConfig.
    Applied,
    /// This proposal was superseded by a newer proposal.
    Superseded,
}

impl fmt::Display for RuleChangeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleChangeStatus::Proposed => write!(f, "proposed"),
            RuleChangeStatus::UnderReview => write!(f, "under-review"),
            RuleChangeStatus::Approved => write!(f, "approved"),
            RuleChangeStatus::Rejected => write!(f, "rejected"),
            RuleChangeStatus::Applied => write!(f, "applied"),
            RuleChangeStatus::Superseded => write!(f, "superseded"),
        }
    }
}

impl FromStr for RuleChangeStatus {
    type Err = DocumentValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "proposed" => Ok(RuleChangeStatus::Proposed),
            "under-review" | "under_review" | "underreview" => Ok(RuleChangeStatus::UnderReview),
            "approved" => Ok(RuleChangeStatus::Approved),
            "rejected" => Ok(RuleChangeStatus::Rejected),
            "applied" => Ok(RuleChangeStatus::Applied),
            "superseded" => Ok(RuleChangeStatus::Superseded),
            _ => Err(DocumentValidationError::InvalidContent(format!(
                "Unknown rule change status '{}': expected proposed/under-review/approved/rejected/applied/superseded",
                s
            ))),
        }
    }
}

impl RuleChangeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleChangeStatus::Proposed => "proposed",
            RuleChangeStatus::UnderReview => "under-review",
            RuleChangeStatus::Approved => "approved",
            RuleChangeStatus::Rejected => "rejected",
            RuleChangeStatus::Applied => "applied",
            RuleChangeStatus::Superseded => "superseded",
        }
    }

    /// Returns the valid next statuses from the current status.
    pub fn valid_transitions(&self) -> Vec<RuleChangeStatus> {
        match self {
            RuleChangeStatus::Proposed => vec![RuleChangeStatus::UnderReview],
            RuleChangeStatus::UnderReview => {
                vec![RuleChangeStatus::Approved, RuleChangeStatus::Rejected]
            }
            RuleChangeStatus::Approved => {
                vec![RuleChangeStatus::Applied, RuleChangeStatus::Superseded]
            }
            RuleChangeStatus::Rejected => vec![], // terminal
            RuleChangeStatus::Applied => vec![RuleChangeStatus::Superseded],
            RuleChangeStatus::Superseded => vec![], // terminal
        }
    }

    /// Check if a transition to the target status is valid.
    pub fn can_transition_to(&self, target: RuleChangeStatus) -> bool {
        self.valid_transitions().contains(&target)
    }

    /// Returns true if this is a terminal status.
    pub fn is_terminal(&self) -> bool {
        self.valid_transitions().is_empty()
    }
}

/// A RuleChangeProposal captures a proposed modification to a protected RulesConfig.
///
/// Protected rules cannot be edited directly — changes must go through this proposal
/// workflow: Proposed -> UnderReview -> Approved -> Applied.
///
/// Document phases: Draft -> Review -> Decided
#[derive(Debug)]
pub struct RuleChangeProposal {
    core: DocumentCore,
    /// Short code of the RulesConfig targeted by this proposal.
    pub target_rule: String,
    /// What kind of change is being proposed.
    pub change_type: RuleChangeType,
    /// Current workflow status.
    pub proposal_status: RuleChangeStatus,
    /// Optional reviewer assigned to evaluate this proposal.
    pub reviewer: Option<String>,
    /// Timestamp when the change was applied (if status is Applied).
    pub applied_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl RuleChangeProposal {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        target_rule: String,
        change_type: RuleChangeType,
        proposal_status: RuleChangeStatus,
        reviewer: Option<String>,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            tags,
            archived,
            short_code,
            target_rule,
            change_type,
            proposal_status,
            reviewer,
            template_content,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_template(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        target_rule: String,
        change_type: RuleChangeType,
        proposal_status: RuleChangeStatus,
        reviewer: Option<String>,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("rule_change_proposal_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("rule_change_proposal_content", &context)
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
            target_rule,
            change_type,
            proposal_status,
            reviewer,
            applied_at: None,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        target_rule: String,
        change_type: RuleChangeType,
        proposal_status: RuleChangeStatus,
        reviewer: Option<String>,
        applied_at: Option<chrono::DateTime<chrono::Utc>>,
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
            target_rule,
            change_type,
            proposal_status,
            reviewer,
            applied_at,
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
        if level != "rule_change_proposal" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'rule_change_proposal', found '{}'",
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

        let target_rule =
            FrontmatterParser::extract_optional_string(&fm_map, "target_rule").unwrap_or_default();

        let change_type_str = FrontmatterParser::extract_optional_string(&fm_map, "change_type")
            .unwrap_or_else(|| "modify".to_string());
        let change_type = change_type_str.parse::<RuleChangeType>()?;

        let proposal_status =
            FrontmatterParser::extract_optional_string(&fm_map, "proposal_status")
                .and_then(|s| s.parse::<RuleChangeStatus>().ok())
                .unwrap_or(RuleChangeStatus::Proposed);

        let reviewer = FrontmatterParser::extract_optional_string(&fm_map, "reviewer");

        let applied_at = FrontmatterParser::extract_optional_string(&fm_map, "applied_at")
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

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
            target_rule,
            change_type,
            proposal_status,
            reviewer,
            applied_at,
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

    // --- status workflow ---

    /// Transition the proposal status through the workflow.
    pub fn advance_status(
        &mut self,
        target: Option<RuleChangeStatus>,
    ) -> Result<RuleChangeStatus, DocumentValidationError> {
        let current = self.proposal_status;
        let new_status = match target {
            Some(s) => {
                if !current.can_transition_to(s) {
                    return Err(DocumentValidationError::InvalidContent(format!(
                        "Cannot transition rule change status from {} to {}",
                        current, s
                    )));
                }
                s
            }
            None => {
                let transitions = current.valid_transitions();
                if transitions.is_empty() {
                    return Ok(current);
                }
                transitions[0]
            }
        };

        self.proposal_status = new_status;
        self.core.metadata.updated_at = Utc::now();

        if new_status == RuleChangeStatus::Applied {
            self.applied_at = Some(Utc::now());
        }

        Ok(new_status)
    }

    // --- phase management ---

    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        match current {
            Phase::Draft => Some(Phase::Review),
            Phase::Review => Some(Phase::Decided),
            _ => None,
        }
    }

    fn valid_transitions_from(current: Phase) -> Vec<Phase> {
        match current {
            Phase::Draft => vec![Phase::Review],
            Phase::Review => vec![Phase::Decided],
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

    pub fn validate(&self) -> Result<(), DocumentValidationError> {
        if self.core.title.trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "RuleChangeProposal title cannot be empty".to_string(),
            ));
        }
        if self.target_rule.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "target_rule".to_string(),
            ));
        }
        Ok(())
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

        context.insert("target_rule", &self.target_rule);
        context.insert("change_type", &self.change_type.to_string());
        context.insert("proposal_status", self.proposal_status.as_str());

        let reviewer_str = self.reviewer.as_deref().unwrap_or("NULL");
        context.insert("reviewer", reviewer_str);

        let applied_at_str = self
            .applied_at
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "NULL".to_string());
        context.insert("applied_at", &applied_at_str);

        let tag_strings: Vec<String> = self.tags().iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);

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

    fn make_rule_change_proposal() -> RuleChangeProposal {
        RuleChangeProposal::new(
            "Relax testing requirement for prototypes".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "RCP-0001".to_string(),
            "RC-0001".to_string(),
            RuleChangeType::Modify,
            RuleChangeStatus::Proposed,
            Some("Tech Lead".to_string()),
        )
        .unwrap()
    }

    #[test]
    fn test_rule_change_proposal_creation() {
        let rcp = make_rule_change_proposal();

        assert_eq!(rcp.title(), "Relax testing requirement for prototypes");
        assert_eq!(rcp.phase().unwrap(), Phase::Draft);
        assert_eq!(rcp.target_rule, "RC-0001");
        assert_eq!(rcp.change_type, RuleChangeType::Modify);
        assert_eq!(rcp.proposal_status, RuleChangeStatus::Proposed);
        assert_eq!(rcp.reviewer.as_deref(), Some("Tech Lead"));
        assert!(rcp.applied_at.is_none());
        assert!(rcp.validate().is_ok());
    }

    #[test]
    fn test_rule_change_type_parsing() {
        assert_eq!(
            "add".parse::<RuleChangeType>().unwrap(),
            RuleChangeType::Add
        );
        assert_eq!(
            "modify".parse::<RuleChangeType>().unwrap(),
            RuleChangeType::Modify
        );
        assert_eq!(
            "update".parse::<RuleChangeType>().unwrap(),
            RuleChangeType::Modify
        );
        assert_eq!(
            "remove".parse::<RuleChangeType>().unwrap(),
            RuleChangeType::Remove
        );
        assert_eq!(
            "delete".parse::<RuleChangeType>().unwrap(),
            RuleChangeType::Remove
        );
        assert_eq!(
            "reclassify".parse::<RuleChangeType>().unwrap(),
            RuleChangeType::Reclassify
        );
        assert!("unknown".parse::<RuleChangeType>().is_err());
    }

    #[test]
    fn test_rule_change_status_parsing() {
        assert_eq!(
            "proposed".parse::<RuleChangeStatus>().unwrap(),
            RuleChangeStatus::Proposed
        );
        assert_eq!(
            "under-review".parse::<RuleChangeStatus>().unwrap(),
            RuleChangeStatus::UnderReview
        );
        assert_eq!(
            "approved".parse::<RuleChangeStatus>().unwrap(),
            RuleChangeStatus::Approved
        );
        assert_eq!(
            "rejected".parse::<RuleChangeStatus>().unwrap(),
            RuleChangeStatus::Rejected
        );
        assert_eq!(
            "applied".parse::<RuleChangeStatus>().unwrap(),
            RuleChangeStatus::Applied
        );
        assert_eq!(
            "superseded".parse::<RuleChangeStatus>().unwrap(),
            RuleChangeStatus::Superseded
        );
        assert!("unknown".parse::<RuleChangeStatus>().is_err());
    }

    #[test]
    fn test_status_workflow_transitions() {
        assert!(RuleChangeStatus::Proposed.can_transition_to(RuleChangeStatus::UnderReview));
        assert!(!RuleChangeStatus::Proposed.can_transition_to(RuleChangeStatus::Applied));

        assert!(RuleChangeStatus::UnderReview.can_transition_to(RuleChangeStatus::Approved));
        assert!(RuleChangeStatus::UnderReview.can_transition_to(RuleChangeStatus::Rejected));
        assert!(!RuleChangeStatus::UnderReview.can_transition_to(RuleChangeStatus::Applied));

        assert!(RuleChangeStatus::Approved.can_transition_to(RuleChangeStatus::Applied));
        assert!(RuleChangeStatus::Approved.can_transition_to(RuleChangeStatus::Superseded));

        assert!(RuleChangeStatus::Rejected.is_terminal());
        assert!(RuleChangeStatus::Superseded.is_terminal());
    }

    #[test]
    fn test_advance_status_workflow() {
        let mut rcp = make_rule_change_proposal();

        assert_eq!(rcp.proposal_status, RuleChangeStatus::Proposed);

        // Proposed -> UnderReview (auto-advance)
        let s = rcp.advance_status(None).unwrap();
        assert_eq!(s, RuleChangeStatus::UnderReview);

        // UnderReview -> Approved (explicit)
        let s = rcp
            .advance_status(Some(RuleChangeStatus::Approved))
            .unwrap();
        assert_eq!(s, RuleChangeStatus::Approved);

        // Approved -> Applied (explicit)
        let s = rcp.advance_status(Some(RuleChangeStatus::Applied)).unwrap();
        assert_eq!(s, RuleChangeStatus::Applied);
        assert!(rcp.applied_at.is_some());
    }

    #[test]
    fn test_advance_status_invalid() {
        let mut rcp = make_rule_change_proposal();

        let err = rcp
            .advance_status(Some(RuleChangeStatus::Applied))
            .unwrap_err();
        assert!(matches!(err, DocumentValidationError::InvalidContent(_)));
    }

    #[test]
    fn test_phase_transitions() {
        let mut rcp = make_rule_change_proposal();

        assert!(rcp.can_transition_to(Phase::Review));
        assert!(!rcp.can_transition_to(Phase::Decided));

        let p = rcp.transition_phase(None).unwrap();
        assert_eq!(p, Phase::Review);

        let p = rcp.transition_phase(None).unwrap();
        assert_eq!(p, Phase::Decided);

        // Terminal
        let p = rcp.transition_phase(None).unwrap();
        assert_eq!(p, Phase::Decided);
    }

    #[test]
    fn test_validation_empty_title() {
        let rcp = RuleChangeProposal::new(
            "".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "RCP-0002".to_string(),
            "RC-0001".to_string(),
            RuleChangeType::Add,
            RuleChangeStatus::Proposed,
            None,
        )
        .unwrap();

        assert!(rcp.validate().is_err());
    }

    #[test]
    fn test_validation_empty_target_rule() {
        let rcp = RuleChangeProposal::new(
            "Some Title".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "RCP-0003".to_string(),
            "".to_string(),
            RuleChangeType::Modify,
            RuleChangeStatus::Proposed,
            None,
        )
        .unwrap();

        assert!(rcp.validate().is_err());
    }

    #[tokio::test]
    async fn test_rule_change_proposal_roundtrip() {
        let rcp = RuleChangeProposal::new(
            "Add dependency direction rule".to_string(),
            vec![
                Tag::Label("rule_change_proposal".to_string()),
                Tag::Phase(Phase::Draft),
            ],
            false,
            "RCP-0004".to_string(),
            "RC-0010".to_string(),
            RuleChangeType::Add,
            RuleChangeStatus::Proposed,
            Some("Architect".to_string()),
        )
        .unwrap();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-rcp.md");

        rcp.to_file(&file_path).await.unwrap();
        let loaded = RuleChangeProposal::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), rcp.title());
        assert_eq!(loaded.phase().unwrap(), rcp.phase().unwrap());
        assert_eq!(loaded.target_rule, rcp.target_rule);
        assert_eq!(loaded.change_type, rcp.change_type);
        assert_eq!(loaded.proposal_status, rcp.proposal_status);
        assert_eq!(loaded.reviewer, rcp.reviewer);
    }

    #[test]
    fn test_from_content_invalid_level() {
        let bad_content = "---\n\
id: test\n\
level: product_doc\n\
title: \"Bad Level\"\n\
short_code: \"RCP-0099\"\n\
created_at: 2026-01-01T00:00:00Z\n\
updated_at: 2026-01-01T00:00:00Z\n\
archived: false\n\
tags:\n\
  - \"#phase/draft\"\n\
exit_criteria_met: false\n\
schema_version: 1\n\
target_rule: \"RC-0001\"\n\
change_type: \"modify\"\n\
proposal_status: \"proposed\"\n\
reviewer: NULL\n\
applied_at: NULL\n\
---\n\
\n\
# Bad Level\n";
        let err = RuleChangeProposal::from_content(bad_content).unwrap_err();
        assert!(matches!(err, DocumentValidationError::InvalidContent(_)));
    }
}
