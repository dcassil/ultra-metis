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

/// Status of a design change proposal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Proposed,
    UnderReview,
    Approved,
    Rejected,
    Applied,
}

impl ProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProposalStatus::Proposed => "proposed",
            ProposalStatus::UnderReview => "under-review",
            ProposalStatus::Approved => "approved",
            ProposalStatus::Rejected => "rejected",
            ProposalStatus::Applied => "applied",
        }
    }
}

impl std::fmt::Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ProposalStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "proposed" => Ok(ProposalStatus::Proposed),
            "under-review" | "under_review" | "underreview" => Ok(ProposalStatus::UnderReview),
            "approved" => Ok(ProposalStatus::Approved),
            "rejected" => Ok(ProposalStatus::Rejected),
            "applied" => Ok(ProposalStatus::Applied),
            _ => Err(format!("Unknown proposal status: {}", s)),
        }
    }
}

/// A DesignChangeProposal captures a proposed modification to an existing design artifact.
/// Phases: Draft → Review → Decided
#[derive(Debug)]
pub struct DesignChangeProposal {
    core: DocumentCore,
    /// Short code of the artifact targeted by this proposal
    pub target_artifact: String,
    /// Current workflow status of the proposal
    pub proposal_status: ProposalStatus,
    /// Optional reviewer assigned to this proposal
    pub reviewer: Option<String>,
}

impl DesignChangeProposal {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        parent_id: Option<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        target_artifact: String,
        proposal_status: ProposalStatus,
        reviewer: Option<String>,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            parent_id,
            tags,
            archived,
            short_code,
            target_artifact,
            proposal_status,
            reviewer,
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
        target_artifact: String,
        proposal_status: ProposalStatus,
        reviewer: Option<String>,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("design_change_proposal_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("design_change_proposal_content", &context)
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
            target_artifact,
            proposal_status,
            reviewer,
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
        target_artifact: String,
        proposal_status: ProposalStatus,
        reviewer: Option<String>,
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
            target_artifact,
            proposal_status,
            reviewer,
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
        if level != "design_change_proposal" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'design_change_proposal', found '{}'",
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

        let target_artifact =
            FrontmatterParser::extract_optional_string(&fm_map, "target_artifact")
                .unwrap_or_default();

        let proposal_status =
            FrontmatterParser::extract_optional_string(&fm_map, "proposal_status")
                .and_then(|s| s.parse::<ProposalStatus>().ok())
                .unwrap_or(ProposalStatus::Proposed);

        let reviewer = FrontmatterParser::extract_optional_string(&fm_map, "reviewer");

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
            target_artifact,
            proposal_status,
            reviewer,
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

        context.insert("target_artifact", &self.target_artifact);
        context.insert("proposal_status", self.proposal_status.as_str());

        let reviewer_str = self.reviewer.as_deref().unwrap_or("NULL");
        context.insert("reviewer", reviewer_str);

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
                "DesignChangeProposal title cannot be empty".to_string(),
            ));
        }
        if self.target_artifact.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "target_artifact".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_design_change_proposal() -> DesignChangeProposal {
        DesignChangeProposal::new(
            "Replace sync HTTP client with async in api-service".to_string(),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DCP-0001".to_string(),
            "DC-0042".to_string(),
            ProposalStatus::Proposed,
            Some("Tech Lead".to_string()),
        )
        .unwrap()
    }

    #[test]
    fn test_design_change_proposal_creation() {
        let dcp = make_design_change_proposal();

        assert_eq!(
            dcp.title(),
            "Replace sync HTTP client with async in api-service"
        );
        assert_eq!(dcp.phase().unwrap(), Phase::Draft);
        assert_eq!(dcp.target_artifact, "DC-0042");
        assert_eq!(dcp.proposal_status, ProposalStatus::Proposed);
        assert_eq!(dcp.reviewer.as_deref(), Some("Tech Lead"));
        assert!(dcp.validate().is_ok());
    }

    #[test]
    fn test_proposal_status_roundtrip() {
        for status in &[
            ProposalStatus::Proposed,
            ProposalStatus::UnderReview,
            ProposalStatus::Approved,
            ProposalStatus::Rejected,
            ProposalStatus::Applied,
        ] {
            let s = status.as_str();
            let parsed: ProposalStatus = s.parse().unwrap();
            assert_eq!(parsed, *status);
        }
    }

    #[test]
    fn test_design_change_proposal_no_reviewer() {
        let dcp = DesignChangeProposal::new(
            "Refactor config loading".to_string(),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DCP-0002".to_string(),
            "DC-0010".to_string(),
            ProposalStatus::Proposed,
            None,
        )
        .unwrap();

        assert!(dcp.reviewer.is_none());
        assert!(dcp.validate().is_ok());
    }

    #[tokio::test]
    async fn test_design_change_proposal_roundtrip() {
        let dcp = make_design_change_proposal();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-dcp.md");

        dcp.to_file(&file_path).await.unwrap();
        let loaded = DesignChangeProposal::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), dcp.title());
        assert_eq!(loaded.phase().unwrap(), dcp.phase().unwrap());
        assert_eq!(loaded.target_artifact, dcp.target_artifact);
        assert_eq!(loaded.proposal_status, dcp.proposal_status);
        assert_eq!(loaded.reviewer, dcp.reviewer);
    }
}
