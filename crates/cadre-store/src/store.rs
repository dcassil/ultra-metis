//! Document store — file-based persistence for Cadre documents.
//!
//! Documents are stored as markdown+frontmatter files in `.cadre/docs/`.
//! Each file is named by its short code (e.g., `PROJ-V-0001.md`).

use crate::config::ProjectConfig;
use crate::error::{Result, StoreError};
use cadre_core::domain::documents::hierarchy::HierarchyValidator;
use cadre_core::domain::documents::traits::{Document, DocumentValidationError};
use cadre_core::domain::documents::types::{
    Complexity, DocumentId, DocumentType, Phase, RiskLevel, StoryType, Tag,
};
use cadre_core::{
    AnalysisBaseline, Architecture, ArchitectureCatalogEntry, CrossReference, DesignContext,
    DurableInsightNote, Epic, Initiative, ProductDoc, QualityRecord, ReferenceArchitecture,
    RulesConfig, Story, Task, Vision,
};
use chrono;
use std::path::{Path, PathBuf};

const DOCS_DIR: &str = "docs";

/// Summary of a document for listing
#[derive(Debug, Clone)]
pub struct DocumentSummary {
    pub short_code: String,
    pub title: String,
    pub document_type: String,
    pub phase: String,
    pub parent_id: Option<String>,
    pub archived: bool,
}

/// A type-erased document wrapper for operations that need to work across types
pub enum AnyDocument {
    Vision(Vision),
    Initiative(Initiative),
    Task(Task),
    ProductDoc(ProductDoc),
    Epic(Epic),
    Story(Story),
    DesignContext(DesignContext),
    AnalysisBaseline(AnalysisBaseline),
    QualityRecord(QualityRecord),
    RulesConfig(RulesConfig),
    DurableInsightNote(DurableInsightNote),
    CrossReference(CrossReference),
    Architecture(Architecture),
    ArchitectureCatalogEntry(ArchitectureCatalogEntry),
    ReferenceArchitecture(ReferenceArchitecture),
}

impl AnyDocument {
    pub fn short_code(&self) -> &str {
        match self {
            AnyDocument::Vision(d) => &d.metadata().short_code,
            AnyDocument::Initiative(d) => &d.metadata().short_code,
            AnyDocument::Task(d) => &d.metadata().short_code,
            AnyDocument::ProductDoc(d) => &d.metadata().short_code,
            AnyDocument::Epic(d) => &d.metadata().short_code,
            AnyDocument::Story(d) => &d.metadata().short_code,
            AnyDocument::DesignContext(d) => &d.metadata().short_code,
            AnyDocument::AnalysisBaseline(d) => &d.metadata().short_code,
            AnyDocument::QualityRecord(d) => &d.metadata().short_code,
            AnyDocument::RulesConfig(d) => &d.metadata().short_code,
            AnyDocument::DurableInsightNote(d) => &d.metadata().short_code,
            AnyDocument::CrossReference(d) => &d.metadata().short_code,
            AnyDocument::Architecture(d) => &d.metadata().short_code,
            AnyDocument::ArchitectureCatalogEntry(d) => &d.metadata().short_code,
            AnyDocument::ReferenceArchitecture(d) => &d.metadata().short_code,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            AnyDocument::Vision(d) => d.title(),
            AnyDocument::Initiative(d) => d.title(),
            AnyDocument::Task(d) => d.title(),
            AnyDocument::ProductDoc(d) => d.title(),
            AnyDocument::Epic(d) => d.title(),
            AnyDocument::Story(d) => d.title(),
            AnyDocument::DesignContext(d) => d.title(),
            AnyDocument::AnalysisBaseline(d) => d.title(),
            AnyDocument::QualityRecord(d) => d.title(),
            AnyDocument::RulesConfig(d) => d.title(),
            AnyDocument::DurableInsightNote(d) => d.title(),
            AnyDocument::CrossReference(d) => d.title(),
            AnyDocument::Architecture(d) => d.title(),
            AnyDocument::ArchitectureCatalogEntry(d) => d.title(),
            AnyDocument::ReferenceArchitecture(d) => d.title(),
        }
    }

    pub fn document_type(&self) -> DocumentType {
        match self {
            AnyDocument::Vision(d) => d.document_type(),
            AnyDocument::Initiative(d) => d.document_type(),
            AnyDocument::Task(d) => d.document_type(),
            AnyDocument::ProductDoc(d) => d.document_type(),
            AnyDocument::Epic(d) => d.document_type(),
            AnyDocument::Story(d) => d.document_type(),
            AnyDocument::DesignContext(d) => d.document_type(),
            AnyDocument::AnalysisBaseline(_) => DocumentType::AnalysisBaseline,
            AnyDocument::QualityRecord(_) => DocumentType::QualityRecord,
            AnyDocument::RulesConfig(_) => DocumentType::RulesConfig,
            AnyDocument::DurableInsightNote(_) => DocumentType::DurableInsightNote,
            AnyDocument::CrossReference(_) => DocumentType::CrossReference,
            AnyDocument::Architecture(_) => DocumentType::Architecture,
            AnyDocument::ArchitectureCatalogEntry(_) => DocumentType::ArchitectureCatalogEntry,
            AnyDocument::ReferenceArchitecture(_) => DocumentType::ReferenceArchitecture,
        }
    }

    pub fn phase(&self) -> std::result::Result<Phase, DocumentValidationError> {
        match self {
            AnyDocument::Vision(d) => d.phase(),
            AnyDocument::Initiative(d) => d.phase(),
            AnyDocument::Task(d) => d.phase(),
            AnyDocument::ProductDoc(d) => d.phase(),
            AnyDocument::Epic(d) => d.phase(),
            AnyDocument::Story(d) => d.phase(),
            AnyDocument::DesignContext(d) => d.phase(),
            AnyDocument::AnalysisBaseline(d) => d.phase(),
            AnyDocument::QualityRecord(d) => d.phase(),
            AnyDocument::RulesConfig(d) => d.phase(),
            AnyDocument::DurableInsightNote(d) => d.phase(),
            AnyDocument::CrossReference(d) => d.phase(),
            AnyDocument::Architecture(d) => d.phase(),
            AnyDocument::ArchitectureCatalogEntry(d) => d.phase(),
            AnyDocument::ReferenceArchitecture(d) => d.phase(),
        }
    }

    pub fn parent_id(&self) -> Option<String> {
        match self {
            AnyDocument::Vision(d) => d.parent_id().map(|id| id.to_string()),
            AnyDocument::Initiative(d) => d.parent_id().map(|id| id.to_string()),
            AnyDocument::Task(d) => d.parent_id().map(|id| id.to_string()),
            AnyDocument::Epic(d) => d.parent_id().map(|id| id.to_string()),
            AnyDocument::Story(d) => d.parent_id().map(|id| id.to_string()),
            AnyDocument::Architecture(d) => d.parent_id().map(|s| s.to_string()),
            // Types without hierarchical parents
            AnyDocument::ProductDoc(_)
            | AnyDocument::DesignContext(_)
            | AnyDocument::AnalysisBaseline(_)
            | AnyDocument::QualityRecord(_)
            | AnyDocument::RulesConfig(_)
            | AnyDocument::DurableInsightNote(_)
            | AnyDocument::CrossReference(_)
            | AnyDocument::ArchitectureCatalogEntry(_)
            | AnyDocument::ReferenceArchitecture(_) => None,
        }
    }

    pub fn archived(&self) -> bool {
        match self {
            AnyDocument::Vision(d) => d.archived(),
            AnyDocument::Initiative(d) => d.archived(),
            AnyDocument::Task(d) => d.archived(),
            AnyDocument::ProductDoc(d) => d.archived(),
            AnyDocument::Epic(d) => d.archived(),
            AnyDocument::Story(d) => d.archived(),
            AnyDocument::DesignContext(d) => d.archived(),
            AnyDocument::AnalysisBaseline(d) => d.archived(),
            AnyDocument::QualityRecord(d) => d.archived(),
            AnyDocument::RulesConfig(d) => d.archived(),
            AnyDocument::DurableInsightNote(d) => d.archived(),
            AnyDocument::CrossReference(d) => d.archived(),
            AnyDocument::Architecture(d) => d.archived(),
            AnyDocument::ArchitectureCatalogEntry(d) => d.archived(),
            AnyDocument::ReferenceArchitecture(d) => d.archived(),
        }
    }

    pub fn to_content(&self) -> std::result::Result<String, DocumentValidationError> {
        match self {
            AnyDocument::Vision(d) => d.to_content(),
            AnyDocument::Initiative(d) => d.to_content(),
            AnyDocument::Task(d) => d.to_content(),
            AnyDocument::ProductDoc(d) => d.to_content(),
            AnyDocument::Epic(d) => d.to_content(),
            AnyDocument::Story(d) => d.to_content(),
            AnyDocument::DesignContext(d) => d.to_content(),
            AnyDocument::AnalysisBaseline(d) => d.to_content(),
            AnyDocument::QualityRecord(d) => d.to_content(),
            AnyDocument::RulesConfig(d) => d.to_content(),
            AnyDocument::DurableInsightNote(d) => d.to_content(),
            AnyDocument::CrossReference(d) => d.to_content(),
            AnyDocument::Architecture(d) => d.to_content(),
            AnyDocument::ArchitectureCatalogEntry(d) => d.to_content(),
            AnyDocument::ReferenceArchitecture(d) => d.to_content(),
        }
    }

    pub fn transition_phase(
        &mut self,
        target: Option<Phase>,
    ) -> std::result::Result<Phase, DocumentValidationError> {
        match self {
            AnyDocument::Vision(d) => d.transition_phase(target),
            AnyDocument::Initiative(d) => d.transition_phase(target),
            AnyDocument::Task(d) => d.transition_phase(target),
            AnyDocument::ProductDoc(d) => d.transition_phase(target),
            AnyDocument::Epic(d) => d.transition_phase(target),
            AnyDocument::Story(d) => d.transition_phase(target),
            AnyDocument::DesignContext(d) => d.transition_phase(target),
            AnyDocument::RulesConfig(d) => d.transition_phase(target),
            AnyDocument::ArchitectureCatalogEntry(d) => d.transition_phase(target),
            AnyDocument::ReferenceArchitecture(d) => d.transition_phase(target),
            // Governance types without built-in transition_phase: use DocumentType-based transitions
            AnyDocument::Architecture(_)
            | AnyDocument::AnalysisBaseline(_)
            | AnyDocument::QualityRecord(_)
            | AnyDocument::DurableInsightNote(_)
            | AnyDocument::CrossReference(_) => {
                let doc_type = self.document_type();
                let current = self.phase()?;
                let new_phase = match target {
                    Some(phase) => {
                        if !doc_type.can_transition(current, phase) {
                            return Err(DocumentValidationError::InvalidPhaseTransition {
                                from: current,
                                to: phase,
                            });
                        }
                        phase
                    }
                    None => doc_type.next_phase(current).unwrap_or(current),
                };
                // Can't mutate through self borrow, so we need to match again
                self.update_phase_tag(new_phase);
                Ok(new_phase)
            }
        }
    }

    /// Internal helper to update phase tag on governance types
    fn update_phase_tag(&mut self, new_phase: Phase) {
        let core = match self {
            AnyDocument::Architecture(d) => d.core_mut(),
            AnyDocument::AnalysisBaseline(d) => d.core_mut(),
            AnyDocument::QualityRecord(d) => d.core_mut(),
            AnyDocument::DurableInsightNote(d) => d.core_mut(),
            AnyDocument::CrossReference(d) => d.core_mut(),
            _ => return, // other types have their own transition_phase
        };
        core.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
        core.tags.push(Tag::Phase(new_phase));
        core.metadata.updated_at = chrono::Utc::now();
    }

    pub fn to_summary(&self) -> DocumentSummary {
        DocumentSummary {
            short_code: self.short_code().to_string(),
            title: self.title().to_string(),
            document_type: self.document_type().to_string(),
            phase: self
                .phase()
                .map(|p| p.to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            parent_id: self.parent_id(),
            archived: self.archived(),
        }
    }

    /// Get the full content string for search
    pub fn full_content(&self) -> std::result::Result<String, DocumentValidationError> {
        self.to_content()
    }
}

/// File-based document store
pub struct DocumentStore {
    /// Root path of the project (parent of `.cadre/`)
    project_path: PathBuf,
}

impl DocumentStore {
    /// Create a new store for the given project path.
    /// Normalizes the path: if it ends with `.cadre` or `.metis`, strips the
    /// suffix so the store always operates from the project root.
    pub fn new(project_path: &Path) -> Self {
        let normalized = match project_path.file_name().and_then(|f| f.to_str()) {
            Some(".cadre" | ".metis") => {
                project_path.parent().unwrap_or(project_path).to_path_buf()
            }
            _ => project_path.to_path_buf(),
        };
        Self {
            project_path: normalized,
        }
    }

    /// Get the `.cadre/` directory
    fn metis_dir(&self) -> PathBuf {
        ProjectConfig::project_dir(&self.project_path)
    }

    /// Get the docs directory
    fn docs_dir(&self) -> PathBuf {
        self.metis_dir().join(DOCS_DIR)
    }

    /// Get the file path for a document by short code
    fn doc_path(&self, short_code: &str) -> PathBuf {
        self.docs_dir().join(format!("{}.md", short_code))
    }

    /// Initialize a new project
    pub fn initialize(&self, prefix: &str) -> Result<()> {
        let metis_dir = self.metis_dir();
        if metis_dir.exists() {
            return Err(StoreError::AlreadyInitialized {
                path: self.project_path.display().to_string(),
            });
        }

        std::fs::create_dir_all(&metis_dir)?;
        std::fs::create_dir_all(self.docs_dir())?;

        let config = ProjectConfig::new(prefix);
        config.save(&self.project_path)?;

        Ok(())
    }

    /// Check if the project is initialized
    pub fn is_initialized(&self) -> bool {
        self.metis_dir().exists()
    }

    /// Load the project config
    fn load_config(&self) -> Result<ProjectConfig> {
        ProjectConfig::load(&self.project_path)
    }

    /// Save the project config
    fn save_config(&self, config: &ProjectConfig) -> Result<()> {
        config.save(&self.project_path)
    }

    /// Detect document type from short code (e.g., "PROJ-V-0001" -> Vision)
    #[allow(dead_code)]
    fn detect_type_from_short_code(short_code: &str) -> Result<DocumentType> {
        let parts: Vec<&str> = short_code.split('-').collect();
        if parts.len() < 3 {
            return Err(StoreError::InvalidDocumentType(format!(
                "Invalid short code format: {}",
                short_code
            )));
        }
        // The type prefix is the second part (may be multi-char like PD, DC, AB, etc.)
        match parts[1] {
            "V" => Ok(DocumentType::Vision),
            "I" => Ok(DocumentType::Initiative),
            "T" => Ok(DocumentType::Task),
            "E" => Ok(DocumentType::Epic),
            "S" => Ok(DocumentType::Story),
            "A" => Ok(DocumentType::Adr),
            "PD" => Ok(DocumentType::ProductDoc),
            "DC" => Ok(DocumentType::DesignContext),
            "SP" => Ok(DocumentType::Specification),
            "AB" => Ok(DocumentType::AnalysisBaseline),
            "QR" => Ok(DocumentType::QualityRecord),
            "RC" => Ok(DocumentType::RulesConfig),
            "DIN" => Ok(DocumentType::DurableInsightNote),
            "XR" => Ok(DocumentType::CrossReference),
            "AR" => Ok(DocumentType::Architecture),
            "ACE" => Ok(DocumentType::ArchitectureCatalogEntry),
            "RA" => Ok(DocumentType::ReferenceArchitecture),
            other => Err(StoreError::InvalidDocumentType(format!(
                "Unknown type prefix: {}",
                other
            ))),
        }
    }

    /// Detect document type from file content (frontmatter `level` field)
    fn detect_type_from_content(content: &str) -> Result<DocumentType> {
        // Quick parse: look for `level: <type>` in frontmatter
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("level:") {
                let level = trimmed
                    .strip_prefix("level:")
                    .unwrap()
                    .trim()
                    .trim_matches('"');
                return level
                    .parse::<DocumentType>()
                    .map_err(StoreError::InvalidDocumentType);
            }
            // Stop at end of frontmatter
            if trimmed == "---" && content.starts_with("---") && !trimmed.is_empty() {
                // We may have passed the opening ---, check if this is the closing one
                let first_newline = content.find('\n').unwrap_or(0);
                if line.as_ptr() as usize > content.as_ptr() as usize + first_newline {
                    break;
                }
            }
        }
        Err(StoreError::InvalidDocumentType(
            "Could not detect document type from content".to_string(),
        ))
    }

    /// Parse a document from its file content
    fn parse_document(content: &str) -> Result<AnyDocument> {
        let doc_type = Self::detect_type_from_content(content)?;
        match doc_type {
            DocumentType::Vision => {
                let doc = Vision::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::Vision(doc))
            }
            DocumentType::Initiative => {
                let doc = Initiative::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::Initiative(doc))
            }
            DocumentType::Task => {
                let doc = Task::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::Task(doc))
            }
            DocumentType::AnalysisBaseline => {
                let doc = AnalysisBaseline::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::AnalysisBaseline(doc))
            }
            DocumentType::QualityRecord => {
                let doc = QualityRecord::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::QualityRecord(doc))
            }
            DocumentType::RulesConfig => {
                let doc = RulesConfig::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::RulesConfig(doc))
            }
            DocumentType::DurableInsightNote => {
                let doc = DurableInsightNote::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::DurableInsightNote(doc))
            }
            DocumentType::CrossReference => {
                let doc = CrossReference::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::CrossReference(doc))
            }
            DocumentType::Architecture => {
                let doc = Architecture::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::Architecture(doc))
            }
            DocumentType::ArchitectureCatalogEntry => {
                let doc = ArchitectureCatalogEntry::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::ArchitectureCatalogEntry(doc))
            }
            DocumentType::ReferenceArchitecture => {
                let doc = ReferenceArchitecture::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::ReferenceArchitecture(doc))
            }
            DocumentType::ProductDoc => {
                let doc = ProductDoc::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::ProductDoc(doc))
            }
            DocumentType::Epic => {
                let doc = Epic::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::Epic(doc))
            }
            DocumentType::Story => {
                let doc = Story::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::Story(doc))
            }
            DocumentType::DesignContext => {
                let doc = DesignContext::from_content(content)
                    .map_err(|e| StoreError::Validation(e.to_string()))?;
                Ok(AnyDocument::DesignContext(doc))
            }
            other => Err(StoreError::InvalidDocumentType(format!(
                "Unsupported document type for store operations: {}",
                other
            ))),
        }
    }

    /// Create a new document
    pub fn create_document(
        &self,
        doc_type: &str,
        title: &str,
        parent_short_code: Option<&str>,
    ) -> Result<String> {
        let mut config = self.load_config()?;
        let document_type: DocumentType = doc_type
            .parse()
            .map_err(|e: String| StoreError::InvalidDocumentType(e))?;

        let type_prefix = document_type.short_code_prefix();
        let short_code = config.next_short_code(type_prefix);

        // Validate parent existence and hierarchy
        let parent_type = if let Some(parent_sc) = parent_short_code {
            let parent_doc = self.read_document(parent_sc).map_err(|e| match e {
                StoreError::DocumentNotFound { short_code } => StoreError::Validation(format!(
                    "Parent document '{}' not found. Use list_documents to see available documents.",
                    short_code
                )),
                other => other,
            })?;
            Some(parent_doc.document_type())
        } else {
            None
        };

        HierarchyValidator::validate_parent(document_type, parent_type)
            .map_err(StoreError::Validation)?;

        let parent_id = parent_short_code.map(DocumentId::from);

        let doc = match document_type {
            DocumentType::Vision => {
                let v = Vision::new(
                    title.to_string(),
                    vec![Tag::Phase(Phase::Draft)],
                    false,
                    short_code.clone(),
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::Vision(v)
            }
            DocumentType::Initiative => {
                let i = Initiative::new(
                    title.to_string(),
                    parent_id,
                    vec![],
                    vec![Tag::Phase(Phase::Discovery)],
                    false,
                    Complexity::M,
                    short_code.clone(),
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::Initiative(i)
            }
            DocumentType::Task => {
                let t = Task::new(
                    title.to_string(),
                    parent_id,
                    vec![],
                    vec![Tag::Phase(Phase::Todo)],
                    false,
                    short_code.clone(),
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::Task(t)
            }
            DocumentType::AnalysisBaseline => {
                let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
                let ab = AnalysisBaseline::new(
                    title.to_string(),
                    vec![Tag::Phase(Phase::Draft)],
                    false,
                    short_code.clone(),
                    None,
                    today,
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::AnalysisBaseline(ab)
            }
            DocumentType::QualityRecord => {
                let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
                let qr = QualityRecord::new(
                    title.to_string(),
                    vec![Tag::Phase(Phase::Draft)],
                    false,
                    short_code.clone(),
                    None,
                    today,
                    cadre_core::domain::documents::quality_record::QualityStatus::Pass,
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::QualityRecord(qr)
            }
            DocumentType::RulesConfig => {
                let rc = RulesConfig::new(
                    title.to_string(),
                    vec![Tag::Phase(Phase::Draft)],
                    false,
                    short_code.clone(),
                    cadre_core::domain::documents::rules_config::ProtectionLevel::Standard,
                    cadre_core::domain::documents::rules_config::RuleScope::Repository,
                    None,
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::RulesConfig(rc)
            }
            DocumentType::DurableInsightNote => {
                let din = DurableInsightNote::new(
                    title.to_string(),
                    String::new(),
                    cadre_core::InsightCategory::SubsystemQuirk,
                    cadre_core::InsightScope::new(),
                    vec![Tag::Phase(Phase::Draft)],
                    false,
                    short_code.clone(),
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::DurableInsightNote(din)
            }
            DocumentType::CrossReference => {
                let xr = CrossReference::new(
                    title.to_string(),
                    vec![Tag::Phase(Phase::Draft)],
                    false,
                    short_code.clone(),
                    String::new(),
                    String::new(),
                    cadre_core::RelationshipType::References,
                    String::new(),
                    false,
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::CrossReference(xr)
            }
            DocumentType::Architecture => {
                let ar = Architecture::new(
                    title.to_string(),
                    short_code.clone(),
                    parent_short_code.map(|s| s.to_string()),
                    None,
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::Architecture(ar)
            }
            DocumentType::ArchitectureCatalogEntry => {
                let ace = ArchitectureCatalogEntry::new(
                    title.to_string(),
                    vec![Tag::Phase(Phase::Draft)],
                    false,
                    short_code.clone(),
                    String::new(),
                    String::new(),
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::ArchitectureCatalogEntry(ace)
            }
            DocumentType::ReferenceArchitecture => {
                let ra = ReferenceArchitecture::new(
                    title.to_string(),
                    vec![Tag::Phase(Phase::Draft)],
                    false,
                    short_code.clone(),
                    None,
                    false,
                    cadre_core::ArchitectureStatus::Draft,
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::ReferenceArchitecture(ra)
            }
            DocumentType::ProductDoc => {
                let pd = ProductDoc::new(
                    title.to_string(),
                    vec![Tag::Phase(Phase::Draft)],
                    false,
                    short_code.clone(),
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::ProductDoc(pd)
            }
            DocumentType::Epic => {
                let e = Epic::new(
                    title.to_string(),
                    parent_id,
                    vec![],
                    vec![Tag::Phase(Phase::Discovery)],
                    false,
                    Complexity::M,
                    short_code.clone(),
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::Epic(e)
            }
            DocumentType::Story => {
                let s = Story::new(
                    title.to_string(),
                    vec![Tag::Phase(Phase::Discovery)],
                    false,
                    short_code.clone(),
                    parent_id,
                    StoryType::Feature,
                    RiskLevel::Medium,
                    None,
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::Story(s)
            }
            DocumentType::DesignContext => {
                let dc = DesignContext::new(
                    title.to_string(),
                    vec![Tag::Phase(Phase::Draft)],
                    false,
                    short_code.clone(),
                    vec![],
                )
                .map_err(|e| StoreError::Validation(e.to_string()))?;
                AnyDocument::DesignContext(dc)
            }
            other => {
                return Err(StoreError::InvalidDocumentType(format!(
                    "Document type '{}' not yet supported for creation",
                    other
                )));
            }
        };

        let content = doc
            .to_content()
            .map_err(|e| StoreError::Serialization(e.to_string()))?;
        let path = self.doc_path(&short_code);
        std::fs::write(&path, content)?;

        self.save_config(&config)?;
        Ok(short_code)
    }

    /// Read a document by short code
    pub fn read_document(&self, short_code: &str) -> Result<AnyDocument> {
        let path = self.doc_path(short_code);
        if !path.exists() {
            return Err(StoreError::DocumentNotFound {
                short_code: short_code.to_string(),
            });
        }
        let content = std::fs::read_to_string(&path)?;
        Self::parse_document(&content)
    }

    /// Read raw content of a document
    pub fn read_document_raw(&self, short_code: &str) -> Result<String> {
        let path = self.doc_path(short_code);
        if !path.exists() {
            return Err(StoreError::DocumentNotFound {
                short_code: short_code.to_string(),
            });
        }
        Ok(std::fs::read_to_string(&path)?)
    }

    /// List all documents, optionally filtering by parent
    pub fn list_documents(&self, include_archived: bool) -> Result<Vec<DocumentSummary>> {
        self.list_documents_with_options(include_archived, None)
    }

    /// List documents with optional parent filter
    pub fn list_documents_with_options(
        &self,
        include_archived: bool,
        parent_id: Option<&str>,
    ) -> Result<Vec<DocumentSummary>> {
        let docs_dir = self.docs_dir();
        if !docs_dir.exists() {
            return Ok(vec![]);
        }

        let mut summaries = Vec::new();
        for entry in walkdir::WalkDir::new(&docs_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            let content = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if let Ok(doc) = Self::parse_document(&content) {
                if !include_archived && doc.archived() {
                    continue;
                }
                let summary = doc.to_summary();
                if let Some(filter_parent) = parent_id {
                    match &summary.parent_id {
                        Some(pid) if pid == filter_parent => {}
                        _ => continue,
                    }
                }
                summaries.push(summary);
            }
        }

        summaries.sort_by(|a, b| a.short_code.cmp(&b.short_code));
        Ok(summaries)
    }

    /// Find child documents of a specific type under a given parent
    pub fn find_children_by_type(
        &self,
        parent_short_code: &str,
        document_type: &str,
    ) -> Result<Vec<DocumentSummary>> {
        let docs = self.list_documents_with_options(false, Some(parent_short_code))?;
        Ok(docs
            .into_iter()
            .filter(|d| d.document_type.eq_ignore_ascii_case(document_type))
            .collect())
    }

    /// Edit a document using search and replace
    pub fn edit_document(&self, short_code: &str, search: &str, replace: &str) -> Result<()> {
        self.edit_document_with_options(short_code, search, replace, false)
    }

    /// Transition a document to the next phase (or a specific phase)
    pub fn transition_phase(&self, short_code: &str, target_phase: Option<&str>) -> Result<String> {
        self.transition_phase_with_options(short_code, target_phase, false)
    }

    /// Search documents by text query
    pub fn search_documents(&self, query: &str) -> Result<Vec<DocumentSummary>> {
        self.search_documents_with_options(query, None, None, false)
    }

    /// Archive a document (and all its children)
    pub fn archive_document(&self, short_code: &str) -> Result<()> {
        let doc = self.read_document(short_code)?;

        if doc.archived() {
            return Err(StoreError::Validation(format!(
                "Document '{}' is already archived",
                short_code
            )));
        }

        // Archive the document itself
        self.set_archived(short_code)?;

        // Cascade: archive all child documents
        let all_docs = self.list_documents(false)?;
        for child in all_docs {
            if child.parent_id.as_deref() == Some(short_code) {
                // Recursively archive children (handles nested hierarchies)
                let _ = self.archive_document(&child.short_code);
            }
        }

        Ok(())
    }

    /// Set a single document's archived flag to true
    fn set_archived(&self, short_code: &str) -> Result<()> {
        let path = self.doc_path(short_code);
        let content = std::fs::read_to_string(&path)?;
        let new_content = content.replace("archived: false", "archived: true");
        std::fs::write(&path, new_content)?;
        Ok(())
    }

    /// Edit a document using search and replace, with optional replace_all
    pub fn edit_document_with_options(
        &self,
        short_code: &str,
        search: &str,
        replace: &str,
        replace_all: bool,
    ) -> Result<()> {
        let path = self.doc_path(short_code);
        if !path.exists() {
            return Err(StoreError::DocumentNotFound {
                short_code: short_code.to_string(),
            });
        }

        let content = std::fs::read_to_string(&path)?;
        if !content.contains(search) {
            return Err(StoreError::EditFailed(format!(
                "Search text not found in document {}",
                short_code
            )));
        }

        let new_content = if replace_all {
            content.replace(search, replace)
        } else {
            content.replacen(search, replace, 1)
        };

        // Validate the edited content still has valid frontmatter
        if let Err(e) = Self::parse_document(&new_content) {
            return Err(StoreError::Validation(format!(
                "Edit would corrupt document frontmatter: {}. Edit rolled back.",
                e
            )));
        }

        std::fs::write(&path, new_content)?;
        Ok(())
    }

    /// Transition a document to the next phase, with optional force flag
    pub fn transition_phase_with_options(
        &self,
        short_code: &str,
        target_phase: Option<&str>,
        force: bool,
    ) -> Result<String> {
        let mut doc = self.read_document(short_code)?;

        let target = match target_phase {
            Some(p) => Some(p.parse::<Phase>().map_err(StoreError::Validation)?),
            None => None,
        };

        let old_phase = doc
            .phase()
            .map_err(|e| StoreError::Validation(e.to_string()))?;

        // If force is set, we still validate the phase sequence but skip exit criteria
        let new_phase = if force {
            match target {
                Some(phase) => {
                    // Force still validates the phase transition is in the valid sequence
                    let doc_type = doc.document_type();
                    if !doc_type.can_transition(old_phase, phase) {
                        return Err(StoreError::Validation(format!(
                            "Cannot transition from '{}' to '{}' even with force (invalid sequence for {})",
                            old_phase, phase, doc_type
                        )));
                    }
                    // Directly update the phase tag without exit criteria check
                    // Helper macro-like approach: get core_mut for any variant
                    match &mut doc {
                        AnyDocument::Vision(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                        AnyDocument::Initiative(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                        AnyDocument::Task(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                        AnyDocument::Architecture(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                        AnyDocument::AnalysisBaseline(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                        AnyDocument::QualityRecord(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                        AnyDocument::RulesConfig(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                        AnyDocument::DurableInsightNote(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                        AnyDocument::CrossReference(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                        AnyDocument::ArchitectureCatalogEntry(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                        AnyDocument::ReferenceArchitecture(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                        AnyDocument::ProductDoc(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                        AnyDocument::Epic(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                        AnyDocument::Story(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                        AnyDocument::DesignContext(d) => {
                            let c = d.core_mut();
                            c.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
                            c.tags.push(Tag::Phase(phase));
                            c.metadata.updated_at = chrono::Utc::now();
                        }
                    }
                    phase
                }
                None => {
                    // Auto-advance with force: use normal transition
                    doc.transition_phase(None)
                        .map_err(|e| StoreError::Validation(e.to_string()))?
                }
            }
        } else {
            doc.transition_phase(target)
                .map_err(|e| StoreError::Validation(e.to_string()))?
        };

        if old_phase == new_phase {
            return Err(StoreError::Validation(format!(
                "Document '{}' is already in terminal phase '{}'. No further transitions are possible.",
                short_code, old_phase
            )));
        }

        let content = doc
            .to_content()
            .map_err(|e| StoreError::Serialization(e.to_string()))?;
        let path = self.doc_path(short_code);
        std::fs::write(&path, content)?;

        Ok(format!("{} -> {}", old_phase, new_phase))
    }

    /// Search documents with filtering options
    pub fn search_documents_with_options(
        &self,
        query: &str,
        document_type: Option<&str>,
        limit: Option<usize>,
        include_archived: bool,
    ) -> Result<Vec<DocumentSummary>> {
        let docs_dir = self.docs_dir();
        if !docs_dir.exists() {
            return Ok(vec![]);
        }

        let type_filter: Option<DocumentType> = match document_type {
            Some(t) => Some(
                t.parse::<DocumentType>()
                    .map_err(StoreError::InvalidDocumentType)?,
            ),
            None => None,
        };

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for entry in walkdir::WalkDir::new(&docs_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            let content = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if content.to_lowercase().contains(&query_lower) {
                if let Ok(doc) = Self::parse_document(&content) {
                    if !include_archived && doc.archived() {
                        continue;
                    }
                    if let Some(ref tf) = type_filter {
                        if doc.document_type() != *tf {
                            continue;
                        }
                    }
                    results.push(doc.to_summary());
                }
            }
        }

        results.sort_by(|a, b| a.short_code.cmp(&b.short_code));

        if let Some(lim) = limit {
            results.truncate(lim);
        }

        Ok(results)
    }

    /// Reassign a task to a different parent or to/from the backlog
    pub fn reassign_parent(
        &self,
        short_code: &str,
        new_parent_id: Option<&str>,
        _backlog_category: Option<&str>,
    ) -> Result<String> {
        // 1. Read and verify the document is a task
        let doc = self.read_document(short_code)?;
        if doc.document_type() != DocumentType::Task {
            return Err(StoreError::Validation(format!(
                "Only tasks can be reassigned. '{}' is a {}.",
                short_code,
                doc.document_type()
            )));
        }

        let old_parent = doc.parent_id().unwrap_or_else(|| "backlog".to_string());

        // 2. Validate new parent if provided
        if let Some(parent_sc) = new_parent_id {
            let parent_doc = self.read_document(parent_sc)?;
            let parent_type = parent_doc.document_type();

            // Validate parent type is valid for tasks
            if !matches!(
                parent_type,
                DocumentType::Initiative | DocumentType::Epic | DocumentType::Story
            ) {
                return Err(StoreError::Validation(format!(
                    "Task parent must be an Initiative, Epic, or Story. '{}' is a {}.",
                    parent_sc, parent_type
                )));
            }

            // Validate parent is in an appropriate phase
            if let Ok(phase) = parent_doc.phase() {
                if parent_type == DocumentType::Initiative
                    && !matches!(phase, Phase::Decompose | Phase::Active)
                {
                    return Err(StoreError::Validation(format!(
                        "Target initiative '{}' must be in 'decompose' or 'active' phase (currently '{}').",
                        parent_sc, phase
                    )));
                }
            }
        }

        // 3. Update parent_id in the raw content
        let raw = self.read_document_raw(short_code)?;
        let new_parent_value = new_parent_id.unwrap_or("NULL");

        // Find and replace the parent_id line in frontmatter
        let new_content = Self::replace_frontmatter_field(&raw, "parent_id", new_parent_value)?;

        // 4. Validate the new content still parses
        if let Err(e) = Self::parse_document(&new_content) {
            return Err(StoreError::Validation(format!(
                "Reassignment would corrupt document: {}. Operation rolled back.",
                e
            )));
        }

        // 5. Write back
        let path = self.doc_path(short_code);
        std::fs::write(&path, new_content)?;

        let new_parent_display = new_parent_id.unwrap_or("backlog");
        Ok(format!(
            "Task {} reassigned: {} -> {}",
            short_code, old_parent, new_parent_display
        ))
    }

    /// Replace a frontmatter field value in raw document content
    fn replace_frontmatter_field(content: &str, field: &str, new_value: &str) -> Result<String> {
        let prefix = format!("{}:", field);
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut found = false;
        let mut in_frontmatter = false;
        let mut frontmatter_start_seen = false;

        for line in &mut lines {
            let trimmed = line.trim();
            if trimmed == "---" {
                if !frontmatter_start_seen {
                    frontmatter_start_seen = true;
                    in_frontmatter = true;
                    continue;
                } else {
                    break; // End of frontmatter
                }
            }
            if in_frontmatter && trimmed.starts_with(&prefix) {
                *line = format!("{}: {}", field, new_value);
                found = true;
                break;
            }
        }

        if !found {
            return Err(StoreError::EditFailed(format!(
                "Field '{}' not found in document frontmatter",
                field
            )));
        }

        // Preserve trailing newline if original had one
        let mut result = lines.join("\n");
        if content.ends_with('\n') {
            result.push('\n');
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_store() -> (tempfile::TempDir, DocumentStore) {
        let dir = tempdir().unwrap();
        let store = DocumentStore::new(dir.path());
        store.initialize("TEST").unwrap();
        (dir, store)
    }

    #[test]
    fn test_initialize_project() {
        let dir = tempdir().unwrap();
        let store = DocumentStore::new(dir.path());

        assert!(!store.is_initialized());
        store.initialize("MYPROJ").unwrap();
        assert!(store.is_initialized());

        let config = store.load_config().unwrap();
        assert_eq!(config.prefix, "MYPROJ");
        assert_eq!(config.next_counter, 1);
    }

    #[test]
    fn test_initialize_already_exists() {
        let (_dir, store) = setup_store();
        let result = store.initialize("TEST2");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_and_read_vision() {
        let (_dir, store) = setup_store();

        let short_code = store.create_document("vision", "My Vision", None).unwrap();
        assert_eq!(short_code, "TEST-V-0001");

        let doc = store.read_document(&short_code).unwrap();
        assert_eq!(doc.title(), "My Vision");
        assert_eq!(doc.document_type(), DocumentType::Vision);
        assert_eq!(doc.phase().unwrap(), Phase::Draft);
    }

    #[test]
    fn test_create_and_read_initiative() {
        let (_dir, store) = setup_store();

        let v_code = store
            .create_document("vision", "Parent Vision", None)
            .unwrap();
        let i_code = store
            .create_document("initiative", "My Initiative", Some(&v_code))
            .unwrap();
        assert_eq!(i_code, "TEST-I-0002");

        let doc = store.read_document(&i_code).unwrap();
        assert_eq!(doc.title(), "My Initiative");
        assert_eq!(doc.document_type(), DocumentType::Initiative);
        assert_eq!(doc.phase().unwrap(), Phase::Discovery);
    }

    #[test]
    fn test_create_and_read_task() {
        let (_dir, store) = setup_store();

        let i_code = store
            .create_document("initiative", "Parent Init", None)
            .unwrap();
        let t_code = store
            .create_document("task", "My Task", Some(&i_code))
            .unwrap();
        assert_eq!(t_code, "TEST-T-0002");

        let doc = store.read_document(&t_code).unwrap();
        assert_eq!(doc.title(), "My Task");
        assert_eq!(doc.document_type(), DocumentType::Task);
    }

    #[test]
    fn test_list_documents() {
        let (_dir, store) = setup_store();

        store.create_document("vision", "Vision 1", None).unwrap();
        store.create_document("vision", "Vision 2", None).unwrap();
        store.create_document("initiative", "Init 1", None).unwrap();

        let docs = store.list_documents(false).unwrap();
        assert_eq!(docs.len(), 3);
    }

    #[test]
    fn test_edit_document() {
        let (_dir, store) = setup_store();

        let code = store.create_document("vision", "My Vision", None).unwrap();

        store
            .edit_document(&code, "# My Vision", "# My Updated Vision")
            .unwrap();

        let raw = store.read_document_raw(&code).unwrap();
        assert!(raw.contains("# My Updated Vision"));
    }

    #[test]
    fn test_transition_phase() {
        let (_dir, store) = setup_store();

        let code = store.create_document("vision", "My Vision", None).unwrap();
        assert_eq!(
            store.read_document(&code).unwrap().phase().unwrap(),
            Phase::Draft
        );

        let result = store.transition_phase(&code, None).unwrap();
        assert!(result.contains("review"));

        assert_eq!(
            store.read_document(&code).unwrap().phase().unwrap(),
            Phase::Review
        );
    }

    #[test]
    fn test_search_documents() {
        let (_dir, store) = setup_store();

        store
            .create_document("vision", "Alpha Vision", None)
            .unwrap();
        store
            .create_document("vision", "Beta Vision", None)
            .unwrap();

        let results = store.search_documents("Alpha").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Alpha Vision");
    }

    #[test]
    fn test_archive_document() {
        let (_dir, store) = setup_store();

        let code = store.create_document("vision", "My Vision", None).unwrap();
        store.archive_document(&code).unwrap();

        let doc = store.read_document(&code).unwrap();
        assert!(doc.archived());

        // Archived docs excluded by default in list
        let docs = store.list_documents(false).unwrap();
        assert_eq!(docs.len(), 0);

        // But included when flag is set
        let docs = store.list_documents(true).unwrap();
        assert_eq!(docs.len(), 1);
    }

    #[test]
    fn test_document_not_found() {
        let (_dir, store) = setup_store();
        let result = store.read_document("TEST-V-9999");
        assert!(result.is_err());
    }

    #[test]
    fn test_full_workflow() {
        let (_dir, store) = setup_store();

        // Create vision -> initiative -> task hierarchy
        let v = store
            .create_document("vision", "Product Vision", None)
            .unwrap();
        let i = store
            .create_document("initiative", "Feature Work", Some(&v))
            .unwrap();
        let t = store
            .create_document("task", "Implement Widget", Some(&i))
            .unwrap();

        // Transition vision through phases
        store.transition_phase(&v, None).unwrap(); // draft -> review
        store.transition_phase(&v, None).unwrap(); // review -> published

        // Transition initiative through phases
        store.transition_phase(&i, None).unwrap(); // discovery -> design
        store.transition_phase(&i, None).unwrap(); // design -> ready
        store.transition_phase(&i, None).unwrap(); // ready -> decompose
        store.transition_phase(&i, None).unwrap(); // decompose -> active

        // Transition task
        store.transition_phase(&t, None).unwrap(); // todo -> active
        store.transition_phase(&t, None).unwrap(); // active -> completed

        // Verify final states
        assert_eq!(
            store.read_document(&v).unwrap().phase().unwrap(),
            Phase::Published
        );
        assert_eq!(
            store.read_document(&i).unwrap().phase().unwrap(),
            Phase::Active
        );
        assert_eq!(
            store.read_document(&t).unwrap().phase().unwrap(),
            Phase::Completed
        );

        // List should have all 3
        let docs = store.list_documents(false).unwrap();
        assert_eq!(docs.len(), 3);
    }

    #[test]
    fn test_create_with_nonexistent_parent() {
        let (_dir, store) = setup_store();
        let result = store.create_document("task", "Orphan Task", Some("TEST-I-9999"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("not found"),
            "Error should mention parent not found, got: {}",
            err
        );
    }

    #[test]
    fn test_create_task_with_wrong_parent_type() {
        let (_dir, store) = setup_store();
        let v_code = store.create_document("vision", "A Vision", None).unwrap();
        let result = store.create_document("task", "Bad Task", Some(&v_code));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Vision"),
            "Error should mention wrong parent type, got: {}",
            err
        );
    }

    #[test]
    fn test_create_initiative_with_valid_vision_parent() {
        let (_dir, store) = setup_store();
        let v_code = store.create_document("vision", "A Vision", None).unwrap();
        let result = store.create_document("initiative", "Good Init", Some(&v_code));
        assert!(result.is_ok());
    }

    #[test]
    fn test_edit_corrupting_frontmatter_rolls_back() {
        let (_dir, store) = setup_store();
        let code = store.create_document("vision", "My Vision", None).unwrap();

        // Try to corrupt the frontmatter by replacing the closing ---
        let result = store.edit_document(&code, "---\n\n#", "BROKEN\n\n#");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("corrupt") || err.contains("frontmatter"),
            "Should mention frontmatter corruption, got: {}",
            err
        );

        // Original document should still be readable
        let doc = store.read_document(&code).unwrap();
        assert_eq!(doc.title(), "My Vision");
    }

    #[test]
    fn test_edit_valid_body_succeeds() {
        let (_dir, store) = setup_store();
        let code = store.create_document("vision", "My Vision", None).unwrap();
        let result = store.edit_document(&code, "# My Vision", "# My Updated Vision");
        assert!(result.is_ok());

        let _doc = store.read_document(&code).unwrap();
        let raw = store.read_document_raw(&code).unwrap();
        assert!(raw.contains("# My Updated Vision"));
    }

    #[test]
    fn test_archive_already_archived_errors() {
        let (_dir, store) = setup_store();
        let code = store.create_document("vision", "V", None).unwrap();
        store.archive_document(&code).unwrap();

        let result = store.archive_document(&code);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("already archived"),
            "Should mention already archived, got: {}",
            err
        );
    }

    #[test]
    fn test_archive_cascades_to_children() {
        let (_dir, store) = setup_store();
        let i_code = store.create_document("initiative", "Init", None).unwrap();
        let t1 = store
            .create_document("task", "Task 1", Some(&i_code))
            .unwrap();
        let t2 = store
            .create_document("task", "Task 2", Some(&i_code))
            .unwrap();

        store.archive_document(&i_code).unwrap();

        // All should be archived
        let doc_i = store.read_document(&i_code).unwrap();
        let doc_t1 = store.read_document(&t1).unwrap();
        let doc_t2 = store.read_document(&t2).unwrap();
        assert!(doc_i.archived());
        assert!(doc_t1.archived(), "Child task 1 should be archived");
        assert!(doc_t2.archived(), "Child task 2 should be archived");

        // None should appear in non-archived list
        let docs = store.list_documents(false).unwrap();
        assert_eq!(docs.len(), 0);
    }

    #[test]
    fn test_terminal_phase_vision_errors() {
        let (_dir, store) = setup_store();
        let code = store.create_document("vision", "V", None).unwrap();
        store.transition_phase(&code, None).unwrap(); // draft -> review
        store.transition_phase(&code, None).unwrap(); // review -> published

        // Auto-advance from terminal should error
        let result = store.transition_phase(&code, None);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("terminal phase"),
            "Should mention terminal phase, got: {}",
            err
        );
    }

    #[test]
    fn test_terminal_phase_task_errors() {
        let (_dir, store) = setup_store();
        let i_code = store.create_document("initiative", "I", None).unwrap();
        let t_code = store.create_document("task", "T", Some(&i_code)).unwrap();
        store.transition_phase(&t_code, None).unwrap(); // todo -> active
        store.transition_phase(&t_code, None).unwrap(); // active -> completed

        let result = store.transition_phase(&t_code, None);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("terminal phase"),
            "Should mention terminal phase, got: {}",
            err
        );
    }

    #[test]
    fn test_terminal_phase_explicit_same_phase_errors() {
        let (_dir, store) = setup_store();
        let code = store.create_document("vision", "V", None).unwrap();
        store.transition_phase(&code, None).unwrap(); // draft -> review
        store.transition_phase(&code, None).unwrap(); // review -> published

        // Explicit transition to same terminal phase should also error
        let result = store.transition_phase(&code, Some("published"));
        assert!(result.is_err());
    }

    #[test]
    fn test_create_vision_rejects_parent() {
        let (_dir, store) = setup_store();
        let v_code = store
            .create_document("vision", "First Vision", None)
            .unwrap();
        let result = store.create_document("vision", "Child Vision", Some(&v_code));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("cannot have a parent") || err.contains("top-level"),
            "Error should say vision cannot have parent, got: {}",
            err
        );
    }

    // ===== Comprehensive Negative-Path Tests =====

    #[test]
    fn test_create_empty_title_currently_succeeds() {
        // NOTE: Empty titles are not yet validated at the store level.
        // The domain layer has validate() for this but it's not called during new().
        // This test documents current behavior — a future task should add title validation.
        let (_dir, store) = setup_store();
        let result = store.create_document("vision", "", None);
        assert!(
            result.is_ok(),
            "Empty title currently succeeds (gap to fix)"
        );
    }

    #[test]
    fn test_create_invalid_doc_type_errors() {
        let (_dir, store) = setup_store();
        let result = store.create_document("nonexistent", "Title", None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StoreError::InvalidDocumentType(_)
        ));
    }

    #[test]
    fn test_read_nonexistent_document() {
        let (_dir, store) = setup_store();
        let result: std::result::Result<_, _> = store.read_document("TEST-V-9999");
        match result {
            Err(StoreError::DocumentNotFound { .. }) => {}
            Err(other) => panic!("Expected DocumentNotFound, got: {}", other),
            Ok(_) => panic!("Expected error, got Ok"),
        }
    }

    #[test]
    fn test_edit_search_text_not_found() {
        let (_dir, store) = setup_store();
        let code = store.create_document("vision", "V", None).unwrap();
        let result = store.edit_document(&code, "NONEXISTENT TEXT", "replacement");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StoreError::EditFailed(_)));
    }

    #[test]
    fn test_edit_nonexistent_document() {
        let (_dir, store) = setup_store();
        let result = store.edit_document("TEST-V-9999", "search", "replace");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StoreError::DocumentNotFound { .. }
        ));
    }

    #[test]
    fn test_transition_invalid_phase_string() {
        let (_dir, store) = setup_store();
        let code = store.create_document("vision", "V", None).unwrap();
        let result = store.transition_phase(&code, Some("nonexistent_phase"));
        assert!(result.is_err());
    }

    #[test]
    fn test_transition_invalid_skip_phases() {
        let (_dir, store) = setup_store();
        let code = store.create_document("vision", "V", None).unwrap();
        // Try to skip from draft directly to published
        let result = store.transition_phase(&code, Some("published"));
        assert!(result.is_err(), "Should not allow skipping phases");
    }

    #[test]
    fn test_transition_nonexistent_document() {
        let (_dir, store) = setup_store();
        let result = store.transition_phase("TEST-V-9999", None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StoreError::DocumentNotFound { .. }
        ));
    }

    #[test]
    fn test_archive_nonexistent_document() {
        let (_dir, store) = setup_store();
        let result = store.archive_document("TEST-V-9999");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StoreError::DocumentNotFound { .. }
        ));
    }

    #[test]
    fn test_initialize_already_initialized() {
        let (_dir, store) = setup_store();
        // Store is already initialized by setup_store
        let result = store.initialize("TEST");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StoreError::AlreadyInitialized { .. }
        ));
    }

    #[test]
    fn test_create_task_without_parent_errors() {
        let (_dir, store) = setup_store();
        let result = store.create_document("task", "Orphan Task", None);
        assert!(result.is_err(), "Task without parent should be rejected");
    }

    // ===== reassign_parent Tests =====

    #[test]
    fn test_reassign_parent_between_initiatives() {
        let (_dir, store) = setup_store();
        let i1 = store.create_document("initiative", "Init 1", None).unwrap();
        let i2 = store.create_document("initiative", "Init 2", None).unwrap();
        let t = store.create_document("task", "My Task", Some(&i1)).unwrap();

        // Move i2 to decompose phase so it can accept tasks
        store.transition_phase(&i2, None).unwrap(); // discovery -> design
        store.transition_phase(&i2, None).unwrap(); // design -> ready
        store.transition_phase(&i2, None).unwrap(); // ready -> decompose

        let result = store.reassign_parent(&t, Some(&i2), None).unwrap();
        assert!(result.contains(&i2));

        // Verify the task now has i2 as parent
        let doc = store.read_document(&t).unwrap();
        assert_eq!(doc.parent_id().unwrap(), i2);
    }

    #[test]
    fn test_reassign_parent_to_backlog() {
        let (_dir, store) = setup_store();
        let i = store.create_document("initiative", "Init", None).unwrap();
        let t = store.create_document("task", "My Task", Some(&i)).unwrap();

        let result = store.reassign_parent(&t, None, Some("bug")).unwrap();
        assert!(result.contains("backlog"));

        // Verify the task has no parent
        let raw = store.read_document_raw(&t).unwrap();
        assert!(raw.contains("parent_id: NULL"));
    }

    #[test]
    fn test_reassign_parent_rejects_non_task() {
        let (_dir, store) = setup_store();
        let i = store.create_document("initiative", "Init", None).unwrap();

        let result = store.reassign_parent(&i, None, Some("bug"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Only tasks"));
    }

    #[test]
    fn test_reassign_parent_rejects_wrong_phase() {
        let (_dir, store) = setup_store();
        let i1 = store.create_document("initiative", "Init 1", None).unwrap();
        let i2 = store.create_document("initiative", "Init 2", None).unwrap();
        let t = store.create_document("task", "My Task", Some(&i1)).unwrap();

        // i2 is in discovery phase — cannot accept tasks
        let result = store.reassign_parent(&t, Some(&i2), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("decompose"));
    }

    #[test]
    fn test_reassign_parent_rejects_invalid_parent_type() {
        let (_dir, store) = setup_store();
        let v = store.create_document("vision", "V", None).unwrap();
        let i = store.create_document("initiative", "I", None).unwrap();
        let t = store.create_document("task", "T", Some(&i)).unwrap();

        let result = store.reassign_parent(&t, Some(&v), None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Initiative, Epic, or Story"));
    }

    // ===== edit_document_with_options Tests =====

    #[test]
    fn test_edit_replace_all() {
        let (_dir, store) = setup_store();
        let code = store.create_document("vision", "My Vision", None).unwrap();

        // Add some repeated text
        store
            .edit_document(
                &code,
                "# My Vision",
                "# My Vision\n\nfoo bar\n\nfoo bar\n\nfoo bar",
            )
            .unwrap();

        // Replace all occurrences
        store
            .edit_document_with_options(&code, "foo", "baz", true)
            .unwrap();

        let raw = store.read_document_raw(&code).unwrap();
        assert!(!raw.contains("foo"), "All 'foo' should be replaced");
        assert_eq!(
            raw.matches("baz").count(),
            3,
            "Should have 3 'baz' occurrences"
        );
    }

    #[test]
    fn test_edit_replace_first_only() {
        let (_dir, store) = setup_store();
        let code = store.create_document("vision", "My Vision", None).unwrap();

        store
            .edit_document(&code, "# My Vision", "# My Vision\n\nfoo bar\n\nfoo bar")
            .unwrap();

        // Replace only first occurrence (default)
        store
            .edit_document_with_options(&code, "foo", "baz", false)
            .unwrap();

        let raw = store.read_document_raw(&code).unwrap();
        assert_eq!(raw.matches("baz").count(), 1, "Should have 1 'baz'");
        assert_eq!(
            raw.matches("foo").count(),
            1,
            "Should have 1 remaining 'foo'"
        );
    }

    // ===== search_documents_with_options Tests =====

    #[test]
    fn test_search_with_type_filter() {
        let (_dir, store) = setup_store();
        store
            .create_document("vision", "Alpha Vision", None)
            .unwrap();
        let _i_code = store
            .create_document("initiative", "Alpha Initiative", None)
            .unwrap();

        let results = store
            .search_documents_with_options("Alpha", Some("vision"), None, false)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_type, "vision");
    }

    #[test]
    fn test_search_with_limit() {
        let (_dir, store) = setup_store();
        store.create_document("vision", "Test A", None).unwrap();
        store.create_document("vision", "Test B", None).unwrap();
        store.create_document("vision", "Test C", None).unwrap();

        let results = store
            .search_documents_with_options("Test", None, Some(2), false)
            .unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_with_include_archived() {
        let (_dir, store) = setup_store();
        let code = store
            .create_document("vision", "Archived Vision", None)
            .unwrap();
        store.archive_document(&code).unwrap();

        // Without include_archived
        let results = store
            .search_documents_with_options("Archived", None, None, false)
            .unwrap();
        assert_eq!(results.len(), 0);

        // With include_archived
        let results = store
            .search_documents_with_options("Archived", None, None, true)
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    // ===== transition_phase_with_options (force) Tests =====

    #[test]
    fn test_transition_force_bypasses_criteria() {
        let (_dir, store) = setup_store();
        let code = store.create_document("vision", "V", None).unwrap();

        // Force transition should work the same as normal for valid transitions
        let result = store
            .transition_phase_with_options(&code, Some("review"), true)
            .unwrap();
        assert!(result.contains("review"));

        let doc = store.read_document(&code).unwrap();
        assert_eq!(doc.phase().unwrap(), Phase::Review);
    }

    #[test]
    fn test_transition_force_still_validates_sequence() {
        let (_dir, store) = setup_store();
        let code = store.create_document("vision", "V", None).unwrap();

        // Force cannot skip phases
        let result = store.transition_phase_with_options(&code, Some("published"), true);
        assert!(result.is_err(), "Force should not allow skipping phases");
    }

    // ===== list_documents_with_options (parent filter) Tests =====

    #[test]
    fn test_list_with_parent_filter() {
        let (_dir, store) = setup_store();
        let v_code = store
            .create_document("vision", "Parent Vision", None)
            .unwrap();
        let _i1 = store
            .create_document("initiative", "Child Init 1", Some(&v_code))
            .unwrap();
        let _i2 = store
            .create_document("initiative", "Child Init 2", Some(&v_code))
            .unwrap();
        let _i3 = store
            .create_document("initiative", "Orphan Init", None)
            .unwrap();

        // Filter by parent
        let children = store
            .list_documents_with_options(false, Some(&v_code))
            .unwrap();
        assert_eq!(children.len(), 2, "Should find exactly 2 children");
        for child in &children {
            assert_eq!(child.parent_id.as_deref(), Some(v_code.as_str()));
        }
    }

    #[test]
    fn test_list_with_parent_filter_no_match() {
        let (_dir, store) = setup_store();
        store.create_document("vision", "V", None).unwrap();

        let children = store
            .list_documents_with_options(false, Some("NONEXISTENT"))
            .unwrap();
        assert_eq!(children.len(), 0);
    }

    #[test]
    fn test_list_without_parent_filter_returns_all() {
        let (_dir, store) = setup_store();
        let v_code = store.create_document("vision", "V", None).unwrap();
        store
            .create_document("initiative", "I", Some(&v_code))
            .unwrap();

        let all = store.list_documents_with_options(false, None).unwrap();
        assert_eq!(all.len(), 2, "Should return all documents without filter");
    }

    // ===== find_children_by_type Tests =====

    #[test]
    fn test_find_children_by_type() {
        let (_dir, store) = setup_store();
        let v_code = store.create_document("vision", "V", None).unwrap();
        let i_code = store
            .create_document("initiative", "Init 1", Some(&v_code))
            .unwrap();
        store
            .create_document("initiative", "Init 2", Some(&v_code))
            .unwrap();
        store
            .create_document("task", "Task 1", Some(&i_code))
            .unwrap();

        let inits = store.find_children_by_type(&v_code, "initiative").unwrap();
        assert_eq!(inits.len(), 2);
        for i in &inits {
            assert_eq!(i.document_type, "initiative");
        }

        let tasks = store.find_children_by_type(&i_code, "task").unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].document_type, "task");
    }

    #[test]
    fn test_find_children_by_type_no_match() {
        let (_dir, store) = setup_store();
        let v_code = store.create_document("vision", "V", None).unwrap();
        store
            .create_document("initiative", "I", Some(&v_code))
            .unwrap();

        let tasks = store.find_children_by_type(&v_code, "task").unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_path_normalization_strips_cadre_suffix() {
        let dir = tempdir().unwrap();
        let store = DocumentStore::new(dir.path());
        store.initialize("NORM").unwrap();

        // Create store using path with .cadre suffix — should normalize to project root
        let cadre_path = dir.path().join(".cadre");
        let store2 = DocumentStore::new(&cadre_path);
        assert!(store2.is_initialized());

        // Should be able to create documents through the normalized path
        let code = store2.create_document("vision", "Test", None).unwrap();
        assert_eq!(code, "NORM-V-0001");
    }

    #[test]
    fn test_path_normalization_strips_metis_suffix() {
        let dir = tempdir().unwrap();
        // Initialize using a path that has .metis as the last component
        // First init normally
        let store = DocumentStore::new(dir.path());
        store.initialize("NORM2").unwrap();

        // Create a .metis directory to simulate the old layout
        let metis_path = dir.path().join(".metis");
        std::fs::create_dir_all(&metis_path).unwrap();

        // Store created with .metis path should normalize to parent
        let store2 = DocumentStore::new(&metis_path);
        assert!(store2.is_initialized());
    }
}
