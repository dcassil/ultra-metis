use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{Document, DocumentCore, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, Tag};
use chrono::Utc;
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// A DesignContext captures design decisions, constraints, and references linked to planning artifacts.
///
/// It is a cross-cutting artifact that can be referenced by Epics and Stories.
/// Phases: Draft → Review → Published → Superseded
#[derive(Debug)]
pub struct DesignContext {
    core: DocumentCore,
    pub design_references: Vec<String>,
}

impl DesignContext {
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        design_references: Vec<String>,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            tags,
            archived,
            short_code,
            design_references,
            template_content,
        )
    }

    pub fn new_with_template(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        design_references: Vec<String>,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("design_context_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {e}"))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("design_context_content", &context)
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
            design_references,
        })
    }

    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        design_references: Vec<String>,
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
            design_references,
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

        let title = FrontmatterParser::extract_string(&fm_map, "title")?;
        let archived = FrontmatterParser::extract_bool(&fm_map, "archived").unwrap_or(false);
        let created_at = FrontmatterParser::extract_datetime(&fm_map, "created_at")?;
        let updated_at = FrontmatterParser::extract_datetime(&fm_map, "updated_at")?;
        let exit_criteria_met =
            FrontmatterParser::extract_bool(&fm_map, "exit_criteria_met").unwrap_or(false);
        let tags = FrontmatterParser::extract_tags(&fm_map)?;

        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "design_context" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'design_context', found '{level}'"
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

        let design_references =
            FrontmatterParser::extract_string_array(&fm_map, "design_references")
                .unwrap_or_default();

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            tags,
            archived,
            design_references,
        ))
    }

    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        match current {
            Phase::Draft => Some(Phase::Review),
            Phase::Review => Some(Phase::Published),
            Phase::Published => Some(Phase::Superseded),
            Phase::Superseded => None,
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

        let tag_strings: Vec<String> = self.tags().iter().map(super::types::Tag::to_str).collect();
        context.insert("tags", &tag_strings);
        context.insert("design_references", &self.design_references);

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

impl Document for DesignContext {
    fn document_type(&self) -> DocumentType {
        DocumentType::DesignContext
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
            DocumentType::DesignContext.can_transition(current_phase, phase)
        } else {
            false
        }
    }

    fn parent_id(&self) -> Option<&DocumentId> {
        None
    }

    fn blocked_by(&self) -> &[DocumentId] {
        &[]
    }

    fn validate(&self) -> Result<(), DocumentValidationError> {
        if self.title().trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "DesignContext title cannot be empty".to_string(),
            ));
        }
        if self.parent_id().is_some() {
            return Err(DocumentValidationError::InvalidParent(
                "DesignContexts cannot have parents".to_string(),
            ));
        }
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
    fn test_design_context_creation() {
        let dc = DesignContext::new(
            "My Design Context".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DC-0001".to_string(),
            vec![],
        )
        .unwrap();

        assert_eq!(dc.title(), "My Design Context");
        assert_eq!(dc.document_type(), DocumentType::DesignContext);
        assert_eq!(dc.phase().unwrap(), Phase::Draft);
        assert!(dc.validate().is_ok());
        assert!(dc.parent_id().is_none());
        assert!(dc.design_references.is_empty());
    }

    #[test]
    fn test_design_context_creation_with_references() {
        let dc = DesignContext::new(
            "Design System".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DC-0002".to_string(),
            vec![
                "https://figma.com/design/abc123".to_string(),
                "https://www.w3.org/TR/WCAG21/".to_string(),
            ],
        )
        .unwrap();

        assert_eq!(dc.design_references.len(), 2);
        assert_eq!(dc.design_references[0], "https://figma.com/design/abc123");
    }

    #[tokio::test]
    async fn test_design_context_roundtrip() {
        let dc = DesignContext::new(
            "Test Design Context".to_string(),
            vec![
                Tag::Label("design_context".to_string()),
                Tag::Phase(Phase::Draft),
            ],
            false,
            "TEST-DC-0003".to_string(),
            vec!["https://example.com/design-spec".to_string()],
        )
        .unwrap();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-design-context.md");

        dc.to_file(&file_path).await.unwrap();
        let loaded = DesignContext::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), dc.title());
        assert_eq!(loaded.phase().unwrap(), dc.phase().unwrap());
        assert_eq!(loaded.tags().len(), dc.tags().len());
        assert_eq!(loaded.design_references, dc.design_references);
    }

    #[tokio::test]
    async fn test_design_context_roundtrip_empty_references() {
        let dc = DesignContext::new(
            "No References Context".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DC-0004".to_string(),
            vec![],
        )
        .unwrap();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-design-context-empty.md");

        dc.to_file(&file_path).await.unwrap();
        let loaded = DesignContext::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), dc.title());
        assert!(loaded.design_references.is_empty());
    }

    #[test]
    fn test_design_context_transitions() {
        let mut dc = DesignContext::new(
            "Test Design Context".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DC-0005".to_string(),
            vec![],
        )
        .unwrap();

        assert!(dc.can_transition_to(Phase::Review));
        assert!(!dc.can_transition_to(Phase::Published));
        assert!(!dc.can_transition_to(Phase::Superseded));

        let new_phase = dc.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Review);

        let new_phase = dc.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Published);

        let new_phase = dc.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Superseded);

        let new_phase = dc.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Superseded); // terminal
    }

    #[test]
    fn test_design_context_cannot_be_blocked() {
        let dc = DesignContext::new(
            "Test Design Context".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DC-0006".to_string(),
            vec![],
        )
        .unwrap();

        assert!(dc.blocked_by().is_empty());
    }

    #[test]
    fn test_design_context_invalid_transition() {
        let mut dc = DesignContext::new(
            "Test Design Context".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DC-0007".to_string(),
            vec![],
        )
        .unwrap();

        let result = dc.transition_phase(Some(Phase::Published));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DocumentValidationError::InvalidPhaseTransition { .. }
        ));
    }
}
