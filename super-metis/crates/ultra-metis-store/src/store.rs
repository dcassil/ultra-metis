//! Document store — file-based persistence for Ultra-Metis documents.
//!
//! Documents are stored as markdown+frontmatter files in `.ultra-metis/docs/`.
//! Each file is named by its short code (e.g., `PROJ-V-0001.md`).

use crate::config::ProjectConfig;
use crate::error::{Result, StoreError};
use std::path::{Path, PathBuf};
use ultra_metis_core::domain::documents::hierarchy::HierarchyValidator;
use ultra_metis_core::domain::documents::traits::{Document, DocumentValidationError};
use ultra_metis_core::domain::documents::types::{DocumentType, Phase, Tag, Complexity, DocumentId};
use ultra_metis_core::{Initiative, Task, Vision};

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
}

impl AnyDocument {
    pub fn short_code(&self) -> &str {
        match self {
            AnyDocument::Vision(d) => &d.metadata().short_code,
            AnyDocument::Initiative(d) => &d.metadata().short_code,
            AnyDocument::Task(d) => &d.metadata().short_code,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            AnyDocument::Vision(d) => d.title(),
            AnyDocument::Initiative(d) => d.title(),
            AnyDocument::Task(d) => d.title(),
        }
    }

    pub fn document_type(&self) -> DocumentType {
        match self {
            AnyDocument::Vision(d) => d.document_type(),
            AnyDocument::Initiative(d) => d.document_type(),
            AnyDocument::Task(d) => d.document_type(),
        }
    }

    pub fn phase(&self) -> std::result::Result<Phase, DocumentValidationError> {
        match self {
            AnyDocument::Vision(d) => d.phase(),
            AnyDocument::Initiative(d) => d.phase(),
            AnyDocument::Task(d) => d.phase(),
        }
    }

    pub fn parent_id(&self) -> Option<String> {
        match self {
            AnyDocument::Vision(d) => d.parent_id().map(|id| id.to_string()),
            AnyDocument::Initiative(d) => d.parent_id().map(|id| id.to_string()),
            AnyDocument::Task(d) => d.parent_id().map(|id| id.to_string()),
        }
    }

    pub fn archived(&self) -> bool {
        match self {
            AnyDocument::Vision(d) => d.archived(),
            AnyDocument::Initiative(d) => d.archived(),
            AnyDocument::Task(d) => d.archived(),
        }
    }

    pub fn to_content(&self) -> std::result::Result<String, DocumentValidationError> {
        match self {
            AnyDocument::Vision(d) => d.to_content(),
            AnyDocument::Initiative(d) => d.to_content(),
            AnyDocument::Task(d) => d.to_content(),
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
        }
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
    /// Root path of the project (parent of `.ultra-metis/`)
    project_path: PathBuf,
}

impl DocumentStore {
    /// Create a new store for the given project path
    pub fn new(project_path: &Path) -> Self {
        Self {
            project_path: project_path.to_path_buf(),
        }
    }

    /// Get the `.ultra-metis/` directory
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
    fn detect_type_from_short_code(short_code: &str) -> Result<DocumentType> {
        let parts: Vec<&str> = short_code.split('-').collect();
        if parts.len() < 3 {
            return Err(StoreError::InvalidDocumentType(format!(
                "Invalid short code format: {}",
                short_code
            )));
        }
        // The type prefix is the second part
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
                    .map_err(|e| StoreError::InvalidDocumentType(e));
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

    /// List all documents
    pub fn list_documents(&self, include_archived: bool) -> Result<Vec<DocumentSummary>> {
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

            match Self::parse_document(&content) {
                Ok(doc) => {
                    if !include_archived && doc.archived() {
                        continue;
                    }
                    summaries.push(doc.to_summary());
                }
                Err(_) => continue,
            }
        }

        summaries.sort_by(|a, b| a.short_code.cmp(&b.short_code));
        Ok(summaries)
    }

    /// Edit a document using search and replace
    pub fn edit_document(&self, short_code: &str, search: &str, replace: &str) -> Result<()> {
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

        let new_content = content.replacen(search, replace, 1);

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

    /// Transition a document to the next phase (or a specific phase)
    pub fn transition_phase(
        &self,
        short_code: &str,
        target_phase: Option<&str>,
    ) -> Result<String> {
        let mut doc = self.read_document(short_code)?;

        let target = match target_phase {
            Some(p) => Some(
                p.parse::<Phase>()
                    .map_err(|e| StoreError::Validation(e))?,
            ),
            None => None,
        };

        let old_phase = doc
            .phase()
            .map_err(|e| StoreError::Validation(e.to_string()))?;

        let new_phase = doc
            .transition_phase(target)
            .map_err(|e| StoreError::Validation(e.to_string()))?;

        // Detect terminal phase: if phase didn't change, the document is at a terminal phase
        if old_phase == new_phase {
            return Err(StoreError::Validation(format!(
                "Document '{}' is already in terminal phase '{}'. No further transitions are possible.",
                short_code, old_phase
            )));
        }

        // Write the updated document back
        let content = doc
            .to_content()
            .map_err(|e| StoreError::Serialization(e.to_string()))?;
        let path = self.doc_path(short_code);
        std::fs::write(&path, content)?;

        Ok(format!("{} -> {}", old_phase, new_phase))
    }

    /// Search documents by text query
    pub fn search_documents(&self, query: &str) -> Result<Vec<DocumentSummary>> {
        let docs_dir = self.docs_dir();
        if !docs_dir.exists() {
            return Ok(vec![]);
        }

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
                match Self::parse_document(&content) {
                    Ok(doc) => results.push(doc.to_summary()),
                    Err(_) => continue,
                }
            }
        }

        results.sort_by(|a, b| a.short_code.cmp(&b.short_code));
        Ok(results)
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

        let v_code = store.create_document("vision", "Parent Vision", None).unwrap();
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
        store
            .create_document("initiative", "Init 1", None)
            .unwrap();

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
        let v = store.create_document("vision", "Product Vision", None).unwrap();
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

        let doc = store.read_document(&code).unwrap();
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
        let t1 = store.create_document("task", "Task 1", Some(&i_code)).unwrap();
        let t2 = store.create_document("task", "Task 2", Some(&i_code)).unwrap();

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
        let v_code = store.create_document("vision", "First Vision", None).unwrap();
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
        assert!(result.is_ok(), "Empty title currently succeeds (gap to fix)");
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
}
