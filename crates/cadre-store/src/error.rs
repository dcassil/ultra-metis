//! Store error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Project not initialized at {path}")]
    NotInitialized { path: String },

    #[error("Project already initialized at {path}")]
    AlreadyInitialized { path: String },

    #[error("Document not found: {short_code}")]
    DocumentNotFound { short_code: String },

    #[error("Invalid document type: {0}")]
    InvalidDocumentType(String),

    #[error("Document validation error: {0}")]
    Validation(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Search/replace failed: {0}")]
    EditFailed(String),
}

impl StoreError {
    /// Returns a user-friendly error message with actionable guidance.
    pub fn user_message(&self) -> String {
        match self {
            Self::Io(e) => format!("File system error: {e}. Check file permissions and disk space."),
            Self::NotInitialized { path } => format!(
                "No Metis workspace found at '{path}'. Run initialize_project first."
            ),
            Self::AlreadyInitialized { path } => format!(
                "A Metis workspace already exists at '{path}'. Use the existing workspace or choose a different directory."
            ),
            Self::DocumentNotFound { short_code } => format!(
                "Document '{short_code}' not found. Use list_documents to see available documents."
            ),
            Self::InvalidDocumentType(msg) => format!(
                "{msg}. Valid document types: vision, initiative, task, adr."
            ),
            Self::Validation(msg) => msg.clone(),
            Self::Config(msg) => format!("Configuration error: {msg}."),
            Self::Serialization(msg) => format!(
                "Failed to serialize document: {msg}. The document may have invalid structure."
            ),
            Self::EditFailed(msg) => format!(
                "{msg}. Use read_document to view current content and verify your search text."
            ),
        }
    }
}

pub type Result<T> = std::result::Result<T, StoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_not_found_user_message() {
        let err = StoreError::DocumentNotFound {
            short_code: "TEST-V-0001".to_string(),
        };
        let msg = err.user_message();
        assert!(msg.contains("TEST-V-0001"));
        assert!(msg.contains("list_documents"));
    }

    #[test]
    fn test_not_initialized_user_message() {
        let err = StoreError::NotInitialized {
            path: "/some/path".to_string(),
        };
        let msg = err.user_message();
        assert!(msg.contains("initialize_project"));
    }

    #[test]
    fn test_edit_failed_user_message() {
        let err = StoreError::EditFailed("Search text not found".to_string());
        let msg = err.user_message();
        assert!(msg.contains("read_document"));
    }

    #[test]
    fn test_invalid_doc_type_user_message() {
        let err = StoreError::InvalidDocumentType("Unknown type 'foo'".to_string());
        let msg = err.user_message();
        assert!(msg.contains("vision"));
        assert!(msg.contains("task"));
    }
}
