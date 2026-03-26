use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{Document, DocumentCore, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, Tag};
use chrono::Utc;
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// An ADR (Architecture Decision Record) captures a significant architectural decision.
/// Parent is optional. Phases: Draft → Discussion → Decided → Superseded
#[derive(Debug)]
pub struct Adr {
    core: DocumentCore,
    /// Optional sequential ADR number (e.g., 1, 2, 42)
    pub number: Option<u32>,
    /// Name of the decision maker or decision-making body
    pub decision_maker: String,
}

impl Adr {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        parent_id: Option<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        number: Option<u32>,
        decision_maker: String,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            parent_id,
            tags,
            archived,
            short_code,
            number,
            decision_maker,
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
        number: Option<u32>,
        decision_maker: String,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("adr_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {e}"))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera.render("adr_content", &context).map_err(|e| {
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
                epic_id: None,
                schema_version: 1,
            },
            number,
            decision_maker,
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
        number: Option<u32>,
        decision_maker: String,
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
            number,
            decision_maker,
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
        if level != "adr" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'adr', found '{level}'"
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

        // number is optional — stored as integer in frontmatter
        let number = match fm_map.get("number") {
            Some(gray_matter::Pod::Integer(n)) => Some(*n as u32),
            _ => None,
        };

        let decision_maker = FrontmatterParser::extract_optional_string(&fm_map, "decision_maker")
            .unwrap_or_default();

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
            number,
            decision_maker,
        ))
    }

    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        match current {
            Phase::Draft => Some(Phase::Discussion),
            Phase::Discussion => Some(Phase::Decided),
            Phase::Decided => Some(Phase::Superseded),
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

        let parent_id_str = self
            .core
            .parent_id
            .as_ref()
            .map(std::string::ToString::to_string)
            .unwrap_or_else(|| "NULL".to_string());
        context.insert("parent_id", &parent_id_str);

        let number_str = self
            .number
            .map(|n| n.to_string())
            .unwrap_or_else(|| "NULL".to_string());
        context.insert("number", &number_str);
        context.insert("decision_maker", &self.decision_maker);

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

impl Document for Adr {
    fn document_type(&self) -> DocumentType {
        DocumentType::Adr
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
            DocumentType::Adr.can_transition(current_phase, phase)
        } else {
            false
        }
    }

    fn parent_id(&self) -> Option<&DocumentId> {
        self.core.parent_id.as_ref()
    }

    fn blocked_by(&self) -> &[DocumentId] {
        &[]
    }

    fn validate(&self) -> Result<(), DocumentValidationError> {
        if self.title().trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "ADR title cannot be empty".to_string(),
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

    fn make_adr() -> Adr {
        Adr::new(
            "Use PostgreSQL for primary storage".to_string(),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-A-0001".to_string(),
            Some(1),
            "Architecture Team".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn test_adr_creation() {
        let adr = make_adr();

        assert_eq!(adr.title(), "Use PostgreSQL for primary storage");
        assert_eq!(adr.document_type(), DocumentType::Adr);
        assert_eq!(adr.phase().unwrap(), Phase::Draft);
        assert_eq!(adr.number, Some(1));
        assert_eq!(adr.decision_maker, "Architecture Team");
        assert!(adr.validate().is_ok());
        assert!(adr.parent_id().is_none());
    }

    #[test]
    fn test_adr_no_number() {
        let adr = Adr::new(
            "Adopt Rust for CLI tooling".to_string(),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-A-0002".to_string(),
            None,
            String::new(),
        )
        .unwrap();

        assert!(adr.number.is_none());
        assert!(adr.validate().is_ok());
    }

    #[tokio::test]
    async fn test_adr_roundtrip() {
        let adr = make_adr();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-adr.md");

        adr.to_file(&file_path).await.unwrap();
        let loaded = Adr::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), adr.title());
        assert_eq!(loaded.phase().unwrap(), adr.phase().unwrap());
        assert_eq!(loaded.decision_maker, adr.decision_maker);
    }

    #[test]
    fn test_adr_transitions() {
        let mut adr = make_adr();

        assert!(adr.can_transition_to(Phase::Discussion));
        assert!(!adr.can_transition_to(Phase::Decided));

        assert_eq!(adr.transition_phase(None).unwrap(), Phase::Discussion);
        assert_eq!(adr.transition_phase(None).unwrap(), Phase::Decided);
        assert_eq!(adr.transition_phase(None).unwrap(), Phase::Superseded);
        assert_eq!(adr.transition_phase(None).unwrap(), Phase::Superseded); // terminal
    }

    #[test]
    fn test_adr_with_parent() {
        let adr = Adr::new(
            "ADR with parent context".to_string(),
            Some(DocumentId::from("parent-epic-id")),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-A-0003".to_string(),
            None,
            "Tech Lead".to_string(),
        )
        .unwrap();

        assert!(adr.parent_id().is_some());
        assert!(adr.validate().is_ok());
    }
}
