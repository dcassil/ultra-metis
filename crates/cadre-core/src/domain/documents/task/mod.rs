use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{Document, DocumentCore, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, Tag};
use chrono::Utc;
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// A Task is a concrete unit of work, requiring a parent (Story, Epic, or Initiative).
///
/// Phases: Backlog → Todo → Active → Completed
/// Also supports Blocked state: Active → Blocked, Todo → Blocked, Blocked → Todo/Active
#[derive(Debug)]
pub struct Task {
    core: DocumentCore,
}

impl Task {
    pub fn new(
        title: String,
        parent_id: Option<DocumentId>,
        blocked_by: Vec<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            parent_id,
            blocked_by,
            tags,
            archived,
            short_code,
            template_content,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_template(
        title: String,
        parent_id: Option<DocumentId>,
        blocked_by: Vec<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("task_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {e}"))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera.render("task_content", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Template render error: {e}"))
        })?;

        let content = DocumentContent::new(&rendered_content);

        Ok(Self {
            core: DocumentCore {
                title,
                metadata,
                content,
                parent_id,
                blocked_by,
                tags,
                archived,
                epic_id: None,
                schema_version: 1,
            },
        })
    }

    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        parent_id: Option<DocumentId>,
        blocked_by: Vec<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
    ) -> Self {
        Self {
            core: DocumentCore {
                title,
                metadata,
                content,
                parent_id,
                blocked_by,
                tags,
                archived,
                epic_id: None,
                schema_version: 1,
            },
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
        if level != "task" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'task', found '{level}'"
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

        let blocked_by = FrontmatterParser::extract_string_array(&fm_map, "blocked_by")
            .unwrap_or_default()
            .into_iter()
            .map(DocumentId::from)
            .collect();

        let metadata = DocumentMetadata::from_frontmatter(
            created_at,
            updated_at,
            exit_criteria_met,
            short_code,
        );
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::from_parts(
            title, metadata, content, parent_id, blocked_by, tags, archived,
        ))
    }

    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        match current {
            Phase::Backlog => Some(Phase::Todo),
            Phase::Todo => Some(Phase::Active),
            Phase::Active => Some(Phase::Completed),
            Phase::Completed => None,
            Phase::Blocked => Some(Phase::Todo),
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

impl Document for Task {
    fn document_type(&self) -> DocumentType {
        DocumentType::Task
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
            DocumentType::Task.can_transition(current_phase, phase)
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
                "Task title cannot be empty".to_string(),
            ));
        }
        if self.parent_id().is_none() {
            return Err(DocumentValidationError::MissingRequiredField(
                "Tasks must have a parent (Story, Epic, or Initiative)".to_string(),
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

    fn make_task(phase: Phase) -> Task {
        Task::new(
            "Implement feature X".to_string(),
            Some(DocumentId::from("parent-story-id")),
            vec![],
            vec![Tag::Phase(phase)],
            false,
            "TEST-T-0001".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn test_task_creation() {
        let task = make_task(Phase::Backlog);

        assert_eq!(task.title(), "Implement feature X");
        assert_eq!(task.document_type(), DocumentType::Task);
        assert_eq!(task.phase().unwrap(), Phase::Backlog);
        assert!(task.validate().is_ok());
        assert!(task.parent_id().is_some());
    }

    #[test]
    fn test_task_validate_requires_parent() {
        let task = Task::new(
            "Orphan Task".to_string(),
            None,
            vec![],
            vec![Tag::Phase(Phase::Backlog)],
            false,
            "TEST-T-0002".to_string(),
        )
        .unwrap();

        assert!(task.validate().is_err());
    }

    #[tokio::test]
    async fn test_task_roundtrip() {
        let task = make_task(Phase::Todo);

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-task.md");

        task.to_file(&file_path).await.unwrap();
        let loaded = Task::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), task.title());
        assert_eq!(loaded.phase().unwrap(), task.phase().unwrap());
        assert_eq!(loaded.tags().len(), task.tags().len());
        assert_eq!(
            loaded.parent_id().map(std::string::ToString::to_string),
            task.parent_id().map(std::string::ToString::to_string)
        );
    }

    #[test]
    fn test_task_phase_sequence() {
        let mut task = make_task(Phase::Backlog);

        assert_eq!(task.transition_phase(None).unwrap(), Phase::Todo);
        assert_eq!(task.transition_phase(None).unwrap(), Phase::Active);
        assert_eq!(task.transition_phase(None).unwrap(), Phase::Completed);
        // Terminal — stays at Completed
        assert_eq!(task.transition_phase(None).unwrap(), Phase::Completed);
    }

    #[test]
    fn test_task_blocked_transitions() {
        let mut task = make_task(Phase::Active);

        assert!(task.can_transition_to(Phase::Blocked));
        assert!(task.can_transition_to(Phase::Completed));

        let phase = task.transition_phase(Some(Phase::Blocked)).unwrap();
        assert_eq!(phase, Phase::Blocked);

        // Blocked → Todo
        assert!(task.can_transition_to(Phase::Todo));
        assert!(task.can_transition_to(Phase::Active));

        let phase = task.transition_phase(Some(Phase::Active)).unwrap();
        assert_eq!(phase, Phase::Active);

        // Todo → Blocked
        task.transition_phase(Some(Phase::Completed)).unwrap();
        let mut task2 = make_task(Phase::Todo);
        let phase = task2.transition_phase(Some(Phase::Blocked)).unwrap();
        assert_eq!(phase, Phase::Blocked);
        let phase = task2.transition_phase(Some(Phase::Todo)).unwrap();
        assert_eq!(phase, Phase::Todo);
    }

    #[test]
    fn test_task_invalid_transition() {
        let mut task = make_task(Phase::Backlog);

        let result = task.transition_phase(Some(Phase::Completed));
        assert!(result.is_err());

        let result = task.transition_phase(Some(Phase::Blocked));
        assert!(result.is_err());
    }
}
