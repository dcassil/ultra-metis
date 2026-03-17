use super::{
    adr::Adr,
    design_context::DesignContext,
    epic::Epic,
    initiative::Initiative,
    product_doc::ProductDoc,
    specification::Specification,
    story::Story,
    task::Task,
    traits::{Document, DocumentValidationError},
    types::DocumentType,
    vision::Vision,
};
use gray_matter::{engine::YAML, Matter};
use std::path::Path;

/// Factory for creating documents from files or raw content.
/// Determines document type from frontmatter "level", "document_type", or "type" field.
pub struct DocumentFactory;

impl DocumentFactory {
    /// Create a document from a file path.
    /// Reads the file, determines type from frontmatter, then creates the appropriate document.
    pub async fn from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<Box<dyn Document>, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e))
        })?;

        let doc_type = Self::extract_document_type(&raw_content)?;

        match doc_type {
            DocumentType::ProductDoc => {
                let doc = ProductDoc::from_file(path).await?;
                Ok(Box::new(doc))
            }
            DocumentType::DesignContext => {
                let doc = DesignContext::from_file(path).await?;
                Ok(Box::new(doc))
            }
            DocumentType::Epic => {
                let doc = Epic::from_file(path).await?;
                Ok(Box::new(doc))
            }
            DocumentType::Story => {
                let doc = Story::from_file(path).await?;
                Ok(Box::new(doc))
            }
            DocumentType::Task => {
                let doc = Task::from_file(path).await?;
                Ok(Box::new(doc))
            }
            DocumentType::Adr => {
                let doc = Adr::from_file(path).await?;
                Ok(Box::new(doc))
            }
            DocumentType::Specification => {
                let doc = Specification::from_file(path).await?;
                Ok(Box::new(doc))
            }
            DocumentType::Vision => {
                let doc = Vision::from_file(path).await?;
                Ok(Box::new(doc))
            }
            DocumentType::Initiative => {
                let doc = Initiative::from_file(path).await?;
                Ok(Box::new(doc))
            }
        }
    }

    /// Create a document from raw content string.
    pub fn from_content(
        raw_content: &str,
        _filepath: &str,
    ) -> Result<Box<dyn Document>, DocumentValidationError> {
        let doc_type = Self::extract_document_type(raw_content)?;

        match doc_type {
            DocumentType::ProductDoc => {
                let doc = ProductDoc::from_content(raw_content)?;
                Ok(Box::new(doc))
            }
            DocumentType::DesignContext => {
                let doc = DesignContext::from_content(raw_content)?;
                Ok(Box::new(doc))
            }
            DocumentType::Epic => {
                let doc = Epic::from_content(raw_content)?;
                Ok(Box::new(doc))
            }
            DocumentType::Story => {
                let doc = Story::from_content(raw_content)?;
                Ok(Box::new(doc))
            }
            DocumentType::Task => {
                let doc = Task::from_content(raw_content)?;
                Ok(Box::new(doc))
            }
            DocumentType::Adr => {
                let doc = Adr::from_content(raw_content)?;
                Ok(Box::new(doc))
            }
            DocumentType::Specification => {
                let doc = Specification::from_content(raw_content)?;
                Ok(Box::new(doc))
            }
            DocumentType::Vision => {
                let doc = Vision::from_content(raw_content)?;
                Ok(Box::new(doc))
            }
            DocumentType::Initiative => {
                let doc = Initiative::from_content(raw_content)?;
                Ok(Box::new(doc))
            }
        }
    }

    /// Extract document type from frontmatter.
    /// Checks "level", "document_type", and "type" fields in that order.
    fn extract_document_type(raw_content: &str) -> Result<DocumentType, DocumentValidationError> {
        let matter = Matter::<YAML>::new();
        let parsed = matter.parse(raw_content);

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

        // Try "level" first (primary field used by all document types in this codebase),
        // then "document_type", then "type" for compatibility.
        let type_str = if let Some(gray_matter::Pod::String(s)) = fm_map.get("level") {
            s.clone()
        } else if let Some(gray_matter::Pod::String(s)) = fm_map.get("document_type") {
            s.clone()
        } else if let Some(gray_matter::Pod::String(s)) = fm_map.get("type") {
            s.clone()
        } else {
            return Err(DocumentValidationError::MissingRequiredField(
                "level, document_type, or type".to_string(),
            ));
        };

        type_str.parse::<DocumentType>().map_err(|_| {
            DocumentValidationError::InvalidContent(format!("Unknown document type: {}", type_str))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_document_type_level_field() {
        let content = r#"---
level: vision
title: Test Vision
---

# Test Vision
"#;
        let doc_type = DocumentFactory::extract_document_type(content).unwrap();
        assert_eq!(doc_type, DocumentType::Vision);
    }

    #[test]
    fn test_extract_document_type_document_type_field() {
        let content = r#"---
document_type: epic
title: Test Epic
---

# Test Epic
"#;
        let doc_type = DocumentFactory::extract_document_type(content).unwrap();
        assert_eq!(doc_type, DocumentType::Epic);
    }

    #[test]
    fn test_extract_all_document_types() {
        let types = vec![
            ("product_doc", DocumentType::ProductDoc),
            ("design_context", DocumentType::DesignContext),
            ("epic", DocumentType::Epic),
            ("story", DocumentType::Story),
            ("task", DocumentType::Task),
            ("adr", DocumentType::Adr),
            ("specification", DocumentType::Specification),
            ("vision", DocumentType::Vision),
            ("initiative", DocumentType::Initiative),
        ];

        for (level_str, expected_type) in types {
            let content = format!(
                "---\nlevel: {}\ntitle: Test\n---\n\n# Test\n",
                level_str
            );
            let doc_type = DocumentFactory::extract_document_type(&content).unwrap();
            assert_eq!(doc_type, expected_type, "Failed for level: {}", level_str);
        }
    }

    #[test]
    fn test_extract_document_type_missing() {
        let content = r#"---
title: Test Document
---

# Test Document
"#;
        let result = DocumentFactory::extract_document_type(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_document_type_invalid() {
        let content = r#"---
level: unknown_type
title: Test Document
---

# Test Document
"#;
        let result = DocumentFactory::extract_document_type(content);
        assert!(result.is_err());
    }
}
