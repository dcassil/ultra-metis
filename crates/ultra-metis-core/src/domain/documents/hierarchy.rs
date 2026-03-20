use super::types::DocumentType;

/// Hierarchy validation for document type relationships.
///
/// Ultra-Metis hierarchy:
///   ProductDoc (top-level, no parent)
///     └── Epic (parent: ProductDoc)
///           └── Story (parent: Epic)
///                 └── Task (parent: Story or Epic)
///
/// Cross-cutting (no required parent):
///   DesignContext, Adr, Specification
///
/// Legacy (retained for migration):
///   Vision (top-level), Initiative (parent: Vision)
pub struct HierarchyValidator;

impl HierarchyValidator {
    /// Check if a given parent type is valid for a child type.
    /// Returns Ok(()) if valid, Err with reason if not.
    pub fn validate_parent(
        child_type: DocumentType,
        parent_type: Option<DocumentType>,
    ) -> Result<(), String> {
        match child_type {
            // Top-level types: no parent allowed
            DocumentType::ProductDoc => {
                if parent_type.is_some() {
                    Err("ProductDoc is top-level and cannot have a parent".to_string())
                } else {
                    Ok(())
                }
            }

            // Epic: parent must be ProductDoc
            DocumentType::Epic => match parent_type {
                Some(DocumentType::ProductDoc) => Ok(()),
                Some(other) => Err(format!(
                    "Epic parent must be a ProductDoc, found {:?}",
                    other
                )),
                None => Err("Epic requires a ProductDoc parent".to_string()),
            },

            // Story: parent must be an Epic
            DocumentType::Story => match parent_type {
                Some(DocumentType::Epic) => Ok(()),
                Some(other) => Err(format!("Story parent must be an Epic, found {:?}", other)),
                None => Err("Story requires an Epic parent".to_string()),
            },

            // Task: parent must be a Story, Epic, or Initiative (legacy)
            DocumentType::Task => match parent_type {
                Some(DocumentType::Story)
                | Some(DocumentType::Epic)
                | Some(DocumentType::Initiative) => Ok(()),
                Some(other) => Err(format!(
                    "Task parent must be a Story, Epic, or Initiative, found {:?}",
                    other
                )),
                None => Err("Task requires a parent (Story, Epic, or Initiative)".to_string()),
            },

            // Cross-cutting types: parent is optional
            DocumentType::DesignContext | DocumentType::Adr | DocumentType::Specification => Ok(()),

            // Governance/architecture types: cross-cutting, no required parent
            DocumentType::AnalysisBaseline
            | DocumentType::QualityRecord
            | DocumentType::RulesConfig
            | DocumentType::DurableInsightNote
            | DocumentType::CrossReference
            | DocumentType::ArchitectureCatalogEntry
            | DocumentType::ReferenceArchitecture => Ok(()),

            // Legacy types
            DocumentType::Vision => {
                if parent_type.is_some() {
                    Err("Vision is top-level and cannot have a parent".to_string())
                } else {
                    Ok(())
                }
            }
            DocumentType::Initiative => match parent_type {
                Some(DocumentType::Vision) | None => Ok(()),
                Some(other) => Err(format!(
                    "Initiative parent must be a Vision, found {:?}",
                    other
                )),
            },
        }
    }

    /// Check if a document type requires a parent
    pub fn requires_parent(doc_type: DocumentType) -> bool {
        matches!(
            doc_type,
            DocumentType::Epic | DocumentType::Story | DocumentType::Task
        )
    }

    /// Get the expected parent types for a given document type
    pub fn valid_parent_types(doc_type: DocumentType) -> Vec<DocumentType> {
        match doc_type {
            DocumentType::Epic => vec![DocumentType::ProductDoc],
            DocumentType::Story => vec![DocumentType::Epic],
            DocumentType::Task => vec![
                DocumentType::Story,
                DocumentType::Epic,
                DocumentType::Initiative,
            ],
            DocumentType::Initiative => vec![DocumentType::Vision],
            _ => vec![], // top-level or cross-cutting
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_doc_no_parent() {
        assert!(HierarchyValidator::validate_parent(DocumentType::ProductDoc, None).is_ok());
        assert!(HierarchyValidator::validate_parent(
            DocumentType::ProductDoc,
            Some(DocumentType::Epic)
        )
        .is_err());
    }

    #[test]
    fn test_epic_requires_product_doc_parent() {
        assert!(HierarchyValidator::validate_parent(
            DocumentType::Epic,
            Some(DocumentType::ProductDoc)
        )
        .is_ok());
        assert!(HierarchyValidator::validate_parent(DocumentType::Epic, None).is_err());
        assert!(HierarchyValidator::validate_parent(
            DocumentType::Epic,
            Some(DocumentType::Vision)
        )
        .is_err());
    }

    #[test]
    fn test_story_requires_epic_parent() {
        assert!(
            HierarchyValidator::validate_parent(DocumentType::Story, Some(DocumentType::Epic))
                .is_ok()
        );
        assert!(HierarchyValidator::validate_parent(DocumentType::Story, None).is_err());
        assert!(HierarchyValidator::validate_parent(
            DocumentType::Story,
            Some(DocumentType::ProductDoc)
        )
        .is_err());
    }

    #[test]
    fn test_task_requires_story_epic_or_initiative_parent() {
        assert!(
            HierarchyValidator::validate_parent(DocumentType::Task, Some(DocumentType::Story))
                .is_ok()
        );
        assert!(
            HierarchyValidator::validate_parent(DocumentType::Task, Some(DocumentType::Epic))
                .is_ok()
        );
        assert!(HierarchyValidator::validate_parent(
            DocumentType::Task,
            Some(DocumentType::Initiative)
        )
        .is_ok());
        assert!(HierarchyValidator::validate_parent(DocumentType::Task, None).is_err());
        assert!(HierarchyValidator::validate_parent(
            DocumentType::Task,
            Some(DocumentType::ProductDoc)
        )
        .is_err());
    }

    #[test]
    fn test_cross_cutting_types_accept_any_parent() {
        assert!(HierarchyValidator::validate_parent(DocumentType::DesignContext, None).is_ok());
        assert!(HierarchyValidator::validate_parent(
            DocumentType::DesignContext,
            Some(DocumentType::Epic)
        )
        .is_ok());
        assert!(HierarchyValidator::validate_parent(DocumentType::Adr, None).is_ok());
        assert!(HierarchyValidator::validate_parent(DocumentType::Specification, None).is_ok());
    }

    #[test]
    fn test_legacy_types() {
        assert!(HierarchyValidator::validate_parent(DocumentType::Vision, None).is_ok());
        assert!(HierarchyValidator::validate_parent(
            DocumentType::Initiative,
            Some(DocumentType::Vision)
        )
        .is_ok());
    }

    #[test]
    fn test_requires_parent() {
        assert!(HierarchyValidator::requires_parent(DocumentType::Epic));
        assert!(HierarchyValidator::requires_parent(DocumentType::Story));
        assert!(HierarchyValidator::requires_parent(DocumentType::Task));
        assert!(!HierarchyValidator::requires_parent(
            DocumentType::ProductDoc
        ));
        assert!(!HierarchyValidator::requires_parent(
            DocumentType::DesignContext
        ));
        assert!(!HierarchyValidator::requires_parent(DocumentType::Adr));
    }

    #[test]
    fn test_valid_parent_types() {
        assert_eq!(
            HierarchyValidator::valid_parent_types(DocumentType::Epic),
            vec![DocumentType::ProductDoc]
        );
        assert_eq!(
            HierarchyValidator::valid_parent_types(DocumentType::Story),
            vec![DocumentType::Epic]
        );
        assert_eq!(
            HierarchyValidator::valid_parent_types(DocumentType::Task),
            vec![
                DocumentType::Story,
                DocumentType::Epic,
                DocumentType::Initiative
            ]
        );
        assert!(HierarchyValidator::valid_parent_types(DocumentType::ProductDoc).is_empty());
    }
}
