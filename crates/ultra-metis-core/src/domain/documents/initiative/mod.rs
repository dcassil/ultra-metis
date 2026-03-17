use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{Document, DocumentCore, DocumentTemplate, DocumentValidationError};
use super::types::{Complexity, DocumentId, DocumentType, Phase, Tag};
use chrono::Utc;
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// An Initiative groups related Epics under an optional Vision.
/// Retained from original Metis codebase for migration compatibility.
/// Phases: Discovery → Design → Ready → Decompose → Active → Completed
#[derive(Debug)]
pub struct Initiative {
    core: DocumentCore,
    pub estimated_complexity: Complexity,
}

impl Initiative {
    /// Create a new Initiative document with content rendered from the embedded template.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        parent_id: Option<DocumentId>,
        blocked_by: Vec<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        estimated_complexity: Complexity,
        short_code: String,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            parent_id,
            blocked_by,
            tags,
            archived,
            estimated_complexity,
            short_code,
            template_content,
        )
    }

    /// Create a new Initiative document with a custom content template.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_template(
        title: String,
        parent_id: Option<DocumentId>,
        blocked_by: Vec<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        estimated_complexity: Complexity,
        short_code: String,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("initiative_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera.render("initiative_content", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Template render error: {}", e))
        })?;

        let content = DocumentContent::new(&rendered_content);
        // initiative_id stored in epic_id field — derived from own title
        let initiative_id = Some(DocumentId::from_title(&title));

        Ok(Self {
            core: DocumentCore {
                title,
                metadata,
                content,
                parent_id,
                blocked_by,
                tags,
                archived,
                epic_id: initiative_id,
                schema_version: 1,
            },
            estimated_complexity,
        })
    }

    /// Create an Initiative document from existing data (used when loading from file).
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        parent_id: Option<DocumentId>,
        blocked_by: Vec<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        estimated_complexity: Complexity,
    ) -> Self {
        let initiative_id = Some(DocumentId::from_title(&title));

        Self {
            core: DocumentCore {
                title,
                metadata,
                content,
                parent_id,
                blocked_by,
                tags,
                archived,
                epic_id: initiative_id,
                schema_version: 1,
            },
            estimated_complexity,
        }
    }

    pub fn estimated_complexity(&self) -> Complexity {
        self.estimated_complexity
    }

    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        use Phase::*;
        match current {
            Discovery => Some(Design),
            Design => Some(Ready),
            Ready => Some(Decompose),
            Decompose => Some(Active),
            Active => Some(Completed),
            Completed => None,
            _ => None,
        }
    }

    fn update_phase_tag(&mut self, new_phase: Phase) {
        self.core.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
        self.core.tags.push(Tag::Phase(new_phase));
        self.core.metadata.updated_at = Utc::now();
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
        if level != "initiative" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'initiative', found '{}'",
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

        let parent_id = FrontmatterParser::extract_optional_string(&fm_map, "parent_id")
            .map(DocumentId::from);

        let blocked_by = FrontmatterParser::extract_string_array(&fm_map, "blocked_by")
            .unwrap_or_default()
            .into_iter()
            .map(DocumentId::from)
            .collect();

        let estimated_complexity =
            FrontmatterParser::extract_string(&fm_map, "estimated_complexity").and_then(|s| {
                s.parse::<Complexity>().map_err(|e| {
                    DocumentValidationError::InvalidContent(format!(
                        "Invalid estimated_complexity: {}",
                        e
                    ))
                })
            })?;

        let short_code = FrontmatterParser::extract_string(&fm_map, "short_code")?;
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
            blocked_by,
            tags,
            archived,
            estimated_complexity,
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
        tera.add_raw_template("frontmatter", self.frontmatter_template())
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
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
        context.insert(
            "parent_id",
            &self
                .parent_id()
                .map(|id| id.to_string())
                .unwrap_or_default(),
        );

        let blocked_by_list: Vec<String> =
            self.blocked_by().iter().map(|id| id.to_string()).collect();
        context.insert("blocked_by", &blocked_by_list);

        context.insert(
            "estimated_complexity",
            &self.estimated_complexity.to_string(),
        );

        let tag_strings: Vec<String> = self.tags().iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);

        context.insert(
            "initiative_id",
            &self
                .core
                .epic_id
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_else(|| "NULL".to_string()),
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

impl Document for Initiative {
    fn document_type(&self) -> DocumentType {
        DocumentType::Initiative
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
            DocumentType::Initiative.can_transition(current_phase, phase)
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
                "Initiative title cannot be empty".to_string(),
            ));
        }
        // parent_id is optional for Initiatives (can be top-level or under a Vision)
        Ok(())
    }

    fn exit_criteria_met(&self) -> bool {
        false
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

    #[test]
    fn test_initiative_creation() {
        let initiative = Initiative::new(
            "My Initiative".to_string(),
            None,
            vec![],
            vec![Tag::Phase(Phase::Discovery)],
            false,
            Complexity::M,
            "TEST-I-0001".to_string(),
        )
        .unwrap();

        assert_eq!(initiative.title(), "My Initiative");
        assert_eq!(initiative.document_type(), DocumentType::Initiative);
        assert_eq!(initiative.phase().unwrap(), Phase::Discovery);
        assert!(initiative.validate().is_ok());
        assert!(initiative.parent_id().is_none());
        assert_eq!(initiative.estimated_complexity(), Complexity::M);
    }

    #[test]
    fn test_initiative_with_parent() {
        let initiative = Initiative::new(
            "Child Initiative".to_string(),
            Some(DocumentId::from("parent-vision-id")),
            vec![],
            vec![Tag::Phase(Phase::Discovery)],
            false,
            Complexity::L,
            "TEST-I-0002".to_string(),
        )
        .unwrap();

        assert!(initiative.parent_id().is_some());
        assert!(initiative.validate().is_ok());
    }

    #[tokio::test]
    async fn test_initiative_roundtrip() {
        let initiative = Initiative::new(
            "Test Initiative".to_string(),
            Some(DocumentId::from("parent-vision")),
            vec![],
            vec![Tag::Phase(Phase::Discovery)],
            false,
            Complexity::S,
            "TEST-I-0003".to_string(),
        )
        .unwrap();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-initiative.md");

        initiative.to_file(&file_path).await.unwrap();
        let loaded = Initiative::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), initiative.title());
        assert_eq!(loaded.phase().unwrap(), initiative.phase().unwrap());
        assert_eq!(loaded.estimated_complexity(), initiative.estimated_complexity());
        assert_eq!(
            loaded.parent_id().map(|id| id.to_string()),
            initiative.parent_id().map(|id| id.to_string())
        );
    }

    #[test]
    fn test_initiative_transitions() {
        let mut initiative = Initiative::new(
            "Test Initiative".to_string(),
            None,
            vec![],
            vec![Tag::Phase(Phase::Discovery)],
            false,
            Complexity::XS,
            "TEST-I-0004".to_string(),
        )
        .unwrap();

        assert!(initiative.can_transition_to(Phase::Design));
        assert!(!initiative.can_transition_to(Phase::Active));

        assert_eq!(initiative.transition_phase(None).unwrap(), Phase::Design);
        assert_eq!(initiative.transition_phase(None).unwrap(), Phase::Ready);
        assert_eq!(initiative.transition_phase(None).unwrap(), Phase::Decompose);
        assert_eq!(initiative.transition_phase(None).unwrap(), Phase::Active);
        assert_eq!(initiative.transition_phase(None).unwrap(), Phase::Completed);
        assert_eq!(initiative.transition_phase(None).unwrap(), Phase::Completed); // terminal
    }

    #[test]
    fn test_initiative_id_lineage() {
        let initiative = Initiative::new(
            "My Big Initiative".to_string(),
            None,
            vec![],
            vec![Tag::Phase(Phase::Discovery)],
            false,
            Complexity::XL,
            "TEST-I-0005".to_string(),
        )
        .unwrap();

        // initiative_id (stored in epic_id) should be derived from title
        assert_eq!(
            initiative.core.epic_id.as_ref().unwrap().to_string(),
            DocumentId::from_title("My Big Initiative").to_string()
        );
    }
}
