use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{Document, DocumentCore, DocumentTemplate, DocumentValidationError};
use super::types::{Complexity, DocumentId, DocumentType, Phase, Tag};
use chrono::Utc;
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// An Epic document groups related stories under a ProductDoc.
/// Replaces Initiative in the new Cadre hierarchy.
/// Phases: Discovery → Design → Ready → Decompose → Active → Completed
#[derive(Debug)]
pub struct Epic {
    core: DocumentCore,
    estimated_complexity: Complexity,
}

impl Epic {
    /// Create a new Epic document with content rendered from the embedded template.
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

    /// Create a new Epic document with a custom content template.
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
        tera.add_raw_template("epic_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {e}"))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera.render("epic_content", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Template render error: {e}"))
        })?;

        let content = DocumentContent::new(&rendered_content);
        let epic_id = Some(DocumentId::from_title(&title));

        Ok(Self {
            core: DocumentCore {
                title,
                metadata,
                content,
                parent_id,
                blocked_by,
                tags,
                archived,
                epic_id,
                schema_version: 1,
            },
            estimated_complexity,
        })
    }

    /// Create an Epic document from existing data (used when loading from file).
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
        let epic_id = Some(DocumentId::from_title(&title));

        Self {
            core: DocumentCore {
                title,
                metadata,
                content,
                parent_id,
                blocked_by,
                tags,
                archived,
                epic_id,
                schema_version: 1,
            },
            estimated_complexity,
        }
    }

    pub fn estimated_complexity(&self) -> Complexity {
        self.estimated_complexity
    }

    /// Get the next phase in the Epic sequence.
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

    /// Update the phase tag in the document's tags.
    fn update_phase_tag(&mut self, new_phase: Phase) {
        self.core.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
        self.core.tags.push(Tag::Phase(new_phase));
        self.core.metadata.updated_at = Utc::now();
    }

    /// Create an Epic document by reading and parsing a file.
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to read file: {e}"))
        })?;
        Self::from_content(&raw_content)
    }

    /// Create an Epic document from a raw file content string.
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

        // Verify document type
        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "epic" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'epic', found '{level}'"
            )));
        }

        // Extract required fields
        let title = FrontmatterParser::extract_string(&fm_map, "title")?;
        let archived = FrontmatterParser::extract_bool(&fm_map, "archived").unwrap_or(false);

        // Parse timestamps
        let created_at = FrontmatterParser::extract_datetime(&fm_map, "created_at")?;
        let updated_at = FrontmatterParser::extract_datetime(&fm_map, "updated_at")?;
        let exit_criteria_met =
            FrontmatterParser::extract_bool(&fm_map, "exit_criteria_met").unwrap_or(false);

        // Parse tags
        let tags = FrontmatterParser::extract_tags(&fm_map)?;

        // Extract epic-specific fields
        let parent_id =
            FrontmatterParser::extract_optional_string(&fm_map, "parent_id").map(DocumentId::from);

        let blocked_by = FrontmatterParser::extract_string_array(&fm_map, "blocked_by")
            .unwrap_or_default()
            .into_iter()
            .map(DocumentId::from)
            .collect();

        let estimated_complexity =
            FrontmatterParser::extract_string(&fm_map, "estimated_complexity").and_then(|s| {
                s.parse::<Complexity>().map_err(|e| {
                    DocumentValidationError::InvalidContent(format!(
                        "Invalid estimated_complexity: {e}"
                    ))
                })
            })?;

        // Build metadata and content
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

    /// Write the Epic document to a file.
    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to write file: {e}"))
        })
    }

    /// Convert the Epic document to its markdown string representation using templates.
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
        context.insert(
            "parent_id",
            &self
                .parent_id()
                .map(std::string::ToString::to_string)
                .unwrap_or_default(),
        );

        let blocked_by_list: Vec<String> =
            self.blocked_by().iter().map(std::string::ToString::to_string).collect();
        context.insert("blocked_by", &blocked_by_list);

        context.insert(
            "estimated_complexity",
            &self.estimated_complexity.to_string(),
        );

        let tag_strings: Vec<String> = self.tags().iter().map(super::types::Tag::to_str).collect();
        context.insert("tags", &tag_strings);

        context.insert(
            "epic_id",
            &self
                .core
                .epic_id
                .as_ref()
                .map(std::string::ToString::to_string)
                .unwrap_or_else(|| "NULL".to_string()),
        );

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

impl Document for Epic {
    fn document_type(&self) -> DocumentType {
        DocumentType::Epic
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
            DocumentType::Epic.can_transition(current_phase, phase)
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
                "Epic title cannot be empty".to_string(),
            ));
        }
        if self.parent_id().is_none() {
            return Err(DocumentValidationError::MissingRequiredField(
                "Epics must have a parent ProductDoc".to_string(),
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

    #[tokio::test]
    async fn test_epic_from_content() {
        let content = r##"---
id: test-epic
level: epic
title: "Test Epic"
short_code: TEST-E-9001
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
parent_id: my-product-doc
blocked_by: []
archived: false
estimated_complexity: "L"

tags:
  - "#epic"
  - "#phase/discovery"

exit_criteria_met: false
schema_version: 1
epic_id: test-epic
---

# Test Epic

## Context

This is a test epic for our system.

## Acceptance Criteria

- [ ] Context is clearly defined
- [ ] Approach is documented
"##;

        let epic = Epic::from_content(content).unwrap();

        assert_eq!(epic.title(), "Test Epic");
        assert_eq!(epic.document_type(), DocumentType::Epic);
        assert!(!epic.archived());
        assert_eq!(epic.tags().len(), 2);
        assert_eq!(epic.phase().unwrap(), Phase::Discovery);
        assert_eq!(epic.estimated_complexity(), Complexity::L);
        assert!(epic.content().has_acceptance_criteria());
        assert_eq!(epic.parent_id().unwrap().to_string(), "my-product-doc");

        // Round-trip test: write to file and read back
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-epic.md");

        epic.to_file(&file_path).await.unwrap();
        let loaded = Epic::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), epic.title());
        assert_eq!(loaded.phase().unwrap(), epic.phase().unwrap());
        assert_eq!(loaded.content().body, epic.content().body);
        assert_eq!(loaded.archived(), epic.archived());
        assert_eq!(loaded.estimated_complexity(), epic.estimated_complexity());
        assert_eq!(loaded.tags().len(), epic.tags().len());
    }

    #[test]
    fn test_epic_complexity_parsing() {
        assert_eq!("XS".parse::<Complexity>().unwrap(), Complexity::XS);
        assert_eq!("S".parse::<Complexity>().unwrap(), Complexity::S);
        assert_eq!("M".parse::<Complexity>().unwrap(), Complexity::M);
        assert_eq!("L".parse::<Complexity>().unwrap(), Complexity::L);
        assert_eq!("XL".parse::<Complexity>().unwrap(), Complexity::XL);
        assert_eq!("xs".parse::<Complexity>().unwrap(), Complexity::XS);
        assert_eq!("s".parse::<Complexity>().unwrap(), Complexity::S);
        assert!("invalid".parse::<Complexity>().is_err());
    }

    #[test]
    fn test_epic_invalid_level() {
        let content = r##"---
id: test-doc
level: product_doc
title: "Test ProductDoc"
short_code: TEST-PD-0001
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
archived: false
estimated_complexity: "M"
tags:
  - "#phase/draft"
exit_criteria_met: false
---

# Test ProductDoc
"##;

        let result = Epic::from_content(content);
        assert!(result.is_err());
        match result.unwrap_err() {
            DocumentValidationError::InvalidContent(msg) => {
                assert!(msg.contains("Expected level 'epic'"));
            }
            _ => panic!("Expected InvalidContent error"),
        }
    }

    #[tokio::test]
    async fn test_epic_validation() {
        let epic = Epic::new(
            "Test Epic".to_string(),
            Some(DocumentId::from("parent-product-doc")),
            vec![],
            vec![Tag::Label("epic".to_string()), Tag::Phase(Phase::Discovery)],
            false,
            Complexity::M,
            "TEST-E-0301".to_string(),
        )
        .expect("Failed to create epic");

        assert!(epic.validate().is_ok());

        // Round-trip test
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-epic.md");

        epic.to_file(&file_path).await.unwrap();
        let loaded = Epic::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), epic.title());
        assert_eq!(loaded.estimated_complexity(), epic.estimated_complexity());
        assert!(loaded.validate().is_ok());

        // Validation failure: no parent
        let epic_no_parent = Epic::new(
            "Test Epic".to_string(),
            None,
            vec![],
            vec![Tag::Phase(Phase::Discovery)],
            false,
            Complexity::M,
            "TEST-E-0302".to_string(),
        )
        .expect("Failed to create epic");

        assert!(epic_no_parent.validate().is_err());
    }

    #[tokio::test]
    async fn test_epic_phase_transitions() {
        let epic = Epic::new(
            "Test Epic".to_string(),
            Some(DocumentId::from("parent-product-doc")),
            vec![],
            vec![Tag::Phase(Phase::Discovery)],
            false,
            Complexity::M,
            "TEST-E-0401".to_string(),
        )
        .expect("Failed to create epic");

        assert!(epic.can_transition_to(Phase::Design));
        assert!(!epic.can_transition_to(Phase::Active));
        assert!(!epic.can_transition_to(Phase::Completed));

        // Round-trip and verify phase survives serialization
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-epic.md");

        epic.to_file(&file_path).await.unwrap();
        let loaded = Epic::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.phase().unwrap(), epic.phase().unwrap());
        assert!(loaded.can_transition_to(Phase::Design));
        assert!(!loaded.can_transition_to(Phase::Active));
        assert!(!loaded.can_transition_to(Phase::Completed));
    }

    #[tokio::test]
    async fn test_epic_full_phase_sequence() {
        let mut epic = Epic::new(
            "Sequence Epic".to_string(),
            Some(DocumentId::from("parent-product-doc")),
            vec![],
            vec![Tag::Phase(Phase::Discovery)],
            false,
            Complexity::S,
            "TEST-E-0501".to_string(),
        )
        .expect("Failed to create epic");

        assert_eq!(epic.transition_phase(None).unwrap(), Phase::Design);
        assert_eq!(epic.transition_phase(None).unwrap(), Phase::Ready);
        assert_eq!(epic.transition_phase(None).unwrap(), Phase::Decompose);
        assert_eq!(epic.transition_phase(None).unwrap(), Phase::Active);
        assert_eq!(epic.transition_phase(None).unwrap(), Phase::Completed);
        // Terminal — stays at Completed
        assert_eq!(epic.transition_phase(None).unwrap(), Phase::Completed);
    }

    #[test]
    fn test_complexity_display() {
        assert_eq!(Complexity::XS.to_string(), "XS");
        assert_eq!(Complexity::S.to_string(), "S");
        assert_eq!(Complexity::M.to_string(), "M");
        assert_eq!(Complexity::L.to_string(), "L");
        assert_eq!(Complexity::XL.to_string(), "XL");
    }

    #[test]
    fn test_epic_id_lineage() {
        let epic = Epic::new(
            "My Epic Feature".to_string(),
            Some(DocumentId::from("parent-product-doc")),
            vec![],
            vec![Tag::Phase(Phase::Discovery)],
            false,
            Complexity::XL,
            "TEST-E-0601".to_string(),
        )
        .expect("Failed to create epic");

        // epic_id should be set to self (derived from title)
        assert_eq!(
            epic.core.epic_id.as_ref().unwrap().to_string(),
            DocumentId::from_title("My Epic Feature").to_string()
        );
    }

    #[test]
    fn test_epic_blocked_by() {
        let blocker = DocumentId::from("blocking-epic-001");
        let epic = Epic::new(
            "Blocked Epic".to_string(),
            Some(DocumentId::from("parent-product-doc")),
            vec![blocker.clone()],
            vec![Tag::Phase(Phase::Discovery)],
            false,
            Complexity::M,
            "TEST-E-0701".to_string(),
        )
        .expect("Failed to create epic");

        assert_eq!(epic.blocked_by().len(), 1);
        assert_eq!(epic.blocked_by()[0], blocker);
    }
}
