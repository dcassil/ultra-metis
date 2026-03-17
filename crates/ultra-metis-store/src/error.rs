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
            StoreError::Io(e) => format!("File system error: {}. Check file permissions and disk space.", e),
            StoreError::NotInitialized { path } => format!(
                "No Metis workspace found at '{}'. Run initialize_project first.",
                path
            ),
            StoreError::AlreadyInitialized { path } => format!(
                "A Metis workspace already exists at '{}'. Use the existing workspace or choose a different directory.",
                path
            ),
            StoreError::DocumentNotFound { short_code } => format!(
                "Document '{}' not found. Use list_documents to see available documents.",
                short_code
            ),
            StoreError::InvalidDocumentType(msg) => format!(
                "{}. Valid document types: vision, initiative, task, adr.",
                msg
            ),
            StoreError::Validation(msg) => msg.clone(),
            StoreError::Config(msg) => format!("Configuration error: {}.", msg),
            StoreError::Serialization(msg) => format!(
                "Failed to serialize document: {}. The document may have invalid structure.",
                msg
            ),
            StoreError::EditFailed(msg) => format!(
                "{}. Use read_document to view current content and verify your search text.",
                msg
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
