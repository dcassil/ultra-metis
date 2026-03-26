use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{Document, DocumentCore, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, RiskLevel, StoryType, Tag};
use chrono::Utc;
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// Parameters specific to Story creation.
pub struct NewStoryParams {
    pub story_type: StoryType,
    pub risk_level: RiskLevel,
    pub epic_id: Option<DocumentId>,
}

/// Parameters for constructing a Story from existing parts.
pub struct StoryParts {
    pub title: String,
    pub metadata: DocumentMetadata,
    pub content: DocumentContent,
    pub tags: Vec<Tag>,
    pub archived: bool,
    pub parent_id: Option<DocumentId>,
    pub blocked_by: Vec<DocumentId>,
    pub story_type: StoryType,
    pub risk_level: RiskLevel,
    pub design_context_refs: Vec<DocumentId>,
    pub epic_id: Option<DocumentId>,
}

/// A Story is an implementable slice of work within an Epic, typed by purpose.
///
/// Phases: Discovery → Design → Ready → Active → Completed
/// Also supports Blocked state: Active → Blocked, Blocked → Ready/Active
/// Parent must be an Epic.
#[derive(Debug)]
pub struct Story {
    core: DocumentCore,
    pub story_type: StoryType,
    pub risk_level: RiskLevel,
    pub design_context_refs: Vec<DocumentId>,
}

impl Story {
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        parent_id: Option<DocumentId>,
        params: NewStoryParams,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            tags,
            archived,
            short_code,
            parent_id,
            params,
            template_content,
        )
    }

    pub fn new_with_template(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        parent_id: Option<DocumentId>,
        params: NewStoryParams,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("story_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {e}"))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera.render("story_content", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Template render error: {e}"))
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
                epic_id: params.epic_id,
                schema_version: 1,
            },
            story_type: params.story_type,
            risk_level: params.risk_level,
            design_context_refs: Vec::new(),
        })
    }

    pub fn from_parts(parts: StoryParts) -> Self {
        Self {
            core: DocumentCore {
                title: parts.title,
                metadata: parts.metadata,
                content: parts.content,
                parent_id: parts.parent_id,
                blocked_by: parts.blocked_by,
                tags: parts.tags,
                archived: parts.archived,
                epic_id: parts.epic_id,
                schema_version: 1,
            },
            story_type: parts.story_type,
            risk_level: parts.risk_level,
            design_context_refs: parts.design_context_refs,
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
        if level != "story" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'story', found '{level}'"
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

        let parent_id = FrontmatterParser::extract_optional_string(&fm_map, "parent_id")
            .map(|s| DocumentId::new(&s));

        let epic_id = FrontmatterParser::extract_optional_string(&fm_map, "epic_id")
            .map(|s| DocumentId::new(&s));

        let blocked_by = FrontmatterParser::extract_string_array(&fm_map, "blocked_by")
            .unwrap_or_default()
            .into_iter()
            .map(|s| DocumentId::new(&s))
            .collect();

        let story_type_str = FrontmatterParser::extract_string(&fm_map, "story_type")?;
        let story_type = story_type_str.parse::<StoryType>().map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Invalid story_type: {e}"))
        })?;

        let risk_level_str = FrontmatterParser::extract_string(&fm_map, "risk_level")?;
        let risk_level = risk_level_str.parse::<RiskLevel>().map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Invalid risk_level: {e}"))
        })?;

        let design_context_refs =
            FrontmatterParser::extract_string_array(&fm_map, "design_context_refs")
                .unwrap_or_default()
                .into_iter()
                .map(|s| DocumentId::new(&s))
                .collect();

        let metadata = DocumentMetadata::from_frontmatter(
            created_at,
            updated_at,
            exit_criteria_met,
            short_code,
        );
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::from_parts(StoryParts {
            title,
            metadata,
            content,
            tags,
            archived,
            parent_id,
            blocked_by,
            story_type,
            risk_level,
            design_context_refs,
            epic_id,
        }))
    }

    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        match current {
            Phase::Discovery => Some(Phase::Design),
            Phase::Design => Some(Phase::Ready),
            Phase::Ready => Some(Phase::Active),
            Phase::Active => Some(Phase::Completed),
            Phase::Completed => None,
            Phase::Blocked => Some(Phase::Ready),
            _ => None,
        }
    }

    fn update_phase_tag(&mut self, new_phase: Phase) {
        self.core.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
        self.core.tags.push(Tag::Phase(new_phase));
        self.core.metadata.updated_at = Utc::now();
    }

    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to write file: {e}"))
        })
    }

    pub fn to_content(&self) -> Result<String, DocumentValidationError> {
        let mut tera = Tera::default();
        tera.add_raw_template("frontmatter", self.frontmatter_template())
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {e}"))
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

        let parent_id_str = self
            .core
            .parent_id
            .as_ref()
            .map(std::string::ToString::to_string)
            .unwrap_or_else(|| "NULL".to_string());
        context.insert("parent_id", &parent_id_str);

        let blocked_by_strings: Vec<String> = self
            .core
            .blocked_by
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        context.insert("blocked_by", &blocked_by_strings);

        let tag_strings: Vec<String> = self.tags().iter().map(super::types::Tag::to_str).collect();
        context.insert("tags", &tag_strings);

        context.insert("story_type", &self.story_type.to_string());
        context.insert("risk_level", &self.risk_level.to_string());

        let design_context_ref_strings: Vec<String> = self
            .design_context_refs
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        context.insert("design_context_refs", &design_context_ref_strings);

        let epic_id_str = self
            .core
            .epic_id
            .as_ref()
            .map(std::string::ToString::to_string)
            .unwrap_or_else(|| "NULL".to_string());
        context.insert("epic_id", &epic_id_str);

        let frontmatter = tera.render("frontmatter", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Frontmatter render error: {e}"))
        })?;

        let content_body = &self.content().body;
        let acceptance_criteria = if let Some(ac) = &self.content().acceptance_criteria {
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
}

impl Document for Story {
    fn document_type(&self) -> DocumentType {
        DocumentType::Story
    }

    fn title(&self) -> &str {
        &self.core.title
    }

    fn metadata(&self) -> &DocumentMetadata {
        &self.core.metadata
    }

    fn content(&self) -> &DocumentContent {
        &self.core.content
    }

    fn core(&self) -> &DocumentCore {
        &self.core
    }

    fn can_transition_to(&self, phase: Phase) -> bool {
        if let Ok(current_phase) = self.phase() {
            DocumentType::Story.can_transition(current_phase, phase)
        } else {
            false
        }
    }

    fn parent_id(&self) -> Option<&DocumentId> {
        self.core.parent_id.as_ref()
    }

    fn blocked_by(&self) -> &[DocumentId] {
        &self.core.blocked_by
    }

    fn validate(&self) -> Result<(), DocumentValidationError> {
        if self.title().trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "Story title cannot be empty".to_string(),
            ));
        }
        if self.parent_id().is_none() {
            return Err(DocumentValidationError::InvalidParent(
                "Story must have a parent Epic".to_string(),
            ));
        }
        Ok(())
    }

    fn exit_criteria_met(&self) -> bool {
        self.core.metadata.exit_criteria_met
    }

    fn template(&self) -> DocumentTemplate {
        DocumentTemplate {
            frontmatter: self.frontmatter_template(),
            content: self.content_template(),
            acceptance_criteria: self.acceptance_criteria_template(),
            file_extension: "md",
        }
    }

    fn frontmatter_template(&self) -> &'static str {
        include_str!("frontmatter.yaml")
    }

    fn content_template(&self) -> &'static str {
        include_str!("content.md")
    }

    fn acceptance_criteria_template(&self) -> &'static str {
        include_str!("acceptance_criteria.md")
    }

    fn transition_phase(
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

    fn core_mut(&mut self) -> &mut DocumentCore {
        &mut self.core
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_story(phase: Phase) -> Story {
        Story::new(
            "Implement login flow".to_string(),
            vec![Tag::Phase(phase)],
            false,
            "TEST-S-0001".to_string(),
            Some(DocumentId::new("parent-epic-id")),
            NewStoryParams {
                story_type: StoryType::Feature,
                risk_level: RiskLevel::Low,
                epic_id: Some(DocumentId::new("my-epic-id")),
            },
        )
        .unwrap()
    }

    #[test]
    fn test_story_creation() {
        let story = make_story(Phase::Discovery);

        assert_eq!(story.title(), "Implement login flow");
        assert_eq!(story.document_type(), DocumentType::Story);
        assert_eq!(story.phase().unwrap(), Phase::Discovery);
        assert_eq!(story.story_type, StoryType::Feature);
        assert_eq!(story.risk_level, RiskLevel::Low);
        assert!(story.design_context_refs.is_empty());
        assert!(story.parent_id().is_some());
        assert!(story.validate().is_ok());
    }

    #[test]
    fn test_story_validate_requires_parent() {
        let story = Story::new(
            "Orphan Story".to_string(),
            vec![Tag::Phase(Phase::Discovery)],
            false,
            "TEST-S-0002".to_string(),
            None,
            NewStoryParams {
                story_type: StoryType::Bugfix,
                risk_level: RiskLevel::Medium,
                epic_id: None,
            },
        )
        .unwrap();

        assert!(story.validate().is_err());
    }

    #[tokio::test]
    async fn test_story_roundtrip() {
        let mut story = make_story(Phase::Ready);
        story.design_context_refs = vec![DocumentId::new("dc-some-design")];
        story.core.blocked_by = vec![DocumentId::new("blocking-story-id")];

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-story.md");

        story.to_file(&file_path).await.unwrap();
        let loaded = Story::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), story.title());
        assert_eq!(loaded.phase().unwrap(), story.phase().unwrap());
        assert_eq!(loaded.story_type, story.story_type);
        assert_eq!(loaded.risk_level, story.risk_level);
        assert_eq!(loaded.tags().len(), story.tags().len());
        assert_eq!(
            loaded.design_context_refs.len(),
            story.design_context_refs.len()
        );
        assert_eq!(loaded.blocked_by().len(), story.blocked_by().len());
        assert_eq!(
            loaded.parent_id().map(std::string::ToString::to_string),
            story.parent_id().map(std::string::ToString::to_string)
        );
        assert_eq!(
            loaded.core.epic_id.as_ref().map(std::string::ToString::to_string),
            story.core.epic_id.as_ref().map(std::string::ToString::to_string)
        );
    }

    #[test]
    fn test_story_phase_transitions() {
        let mut story = make_story(Phase::Discovery);

        assert!(story.can_transition_to(Phase::Design));
        assert!(!story.can_transition_to(Phase::Active));
        assert!(!story.can_transition_to(Phase::Blocked));

        let phase = story.transition_phase(None).unwrap();
        assert_eq!(phase, Phase::Design);

        let phase = story.transition_phase(None).unwrap();
        assert_eq!(phase, Phase::Ready);

        let phase = story.transition_phase(None).unwrap();
        assert_eq!(phase, Phase::Active);

        let phase = story.transition_phase(None).unwrap();
        assert_eq!(phase, Phase::Completed);

        // Terminal state — stays at Completed
        let phase = story.transition_phase(None).unwrap();
        assert_eq!(phase, Phase::Completed);
    }

    #[test]
    fn test_story_blocked_state_transitions() {
        let mut story = make_story(Phase::Active);

        assert!(story.can_transition_to(Phase::Blocked));
        assert!(story.can_transition_to(Phase::Completed));

        let phase = story.transition_phase(Some(Phase::Blocked)).unwrap();
        assert_eq!(phase, Phase::Blocked);

        // Blocked → Ready
        assert!(story.can_transition_to(Phase::Ready));
        assert!(story.can_transition_to(Phase::Active));

        let phase = story.transition_phase(Some(Phase::Active)).unwrap();
        assert_eq!(phase, Phase::Active);

        // Block again, then resume to Ready
        story.transition_phase(Some(Phase::Blocked)).unwrap();
        let phase = story.transition_phase(Some(Phase::Ready)).unwrap();
        assert_eq!(phase, Phase::Ready);
    }

    #[test]
    fn test_story_invalid_transition() {
        let mut story = make_story(Phase::Discovery);

        let result = story.transition_phase(Some(Phase::Completed));
        assert!(result.is_err());

        let result = story.transition_phase(Some(Phase::Blocked));
        assert!(result.is_err());
    }

    #[test]
    fn test_story_type_parsing() {
        assert_eq!("feature".parse::<StoryType>().unwrap(), StoryType::Feature);
        assert_eq!("bugfix".parse::<StoryType>().unwrap(), StoryType::Bugfix);
        assert_eq!("bug-fix".parse::<StoryType>().unwrap(), StoryType::Bugfix);
        assert_eq!(
            "refactor".parse::<StoryType>().unwrap(),
            StoryType::Refactor
        );
        assert_eq!(
            "migration".parse::<StoryType>().unwrap(),
            StoryType::Migration
        );
        assert_eq!(
            "architecture-change".parse::<StoryType>().unwrap(),
            StoryType::ArchitectureChange
        );
        assert_eq!(
            "investigation".parse::<StoryType>().unwrap(),
            StoryType::Investigation
        );
        assert_eq!(
            "remediation".parse::<StoryType>().unwrap(),
            StoryType::Remediation
        );
        assert_eq!("setup".parse::<StoryType>().unwrap(), StoryType::Setup);
        assert!("unknown".parse::<StoryType>().is_err());
    }

    #[test]
    fn test_story_no_decompose_phase() {
        let phases = DocumentType::Story.phase_sequence();
        assert!(!phases.contains(&Phase::Decompose));
        assert!(phases.contains(&Phase::Discovery));
        assert!(phases.contains(&Phase::Design));
        assert!(phases.contains(&Phase::Ready));
        assert!(phases.contains(&Phase::Active));
        assert!(phases.contains(&Phase::Completed));
    }

    #[test]
    fn test_from_content_roundtrip() {
        let story = make_story(Phase::Design);
        let serialized = story.to_content().unwrap();
        let loaded = Story::from_content(&serialized).unwrap();

        assert_eq!(loaded.title(), story.title());
        assert_eq!(loaded.phase().unwrap(), Phase::Design);
        assert_eq!(loaded.story_type, StoryType::Feature);
        assert_eq!(loaded.risk_level, RiskLevel::Low);
    }
}
