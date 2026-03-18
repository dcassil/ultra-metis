use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{DocumentId, Phase, Tag};
use gray_matter;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use tera::{Context, Tera};

// ---------------------------------------------------------------------------
// RelationshipType enum
// ---------------------------------------------------------------------------

/// Types of relationships between documents in the traceability index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Parent-child hierarchy (source is parent of target).
    ParentChild,
    /// Source governs target (e.g., a rule governs a task).
    Governs,
    /// Source references target (general reference link).
    References,
    /// Source is derived from target.
    DerivedFrom,
    /// Source supersedes target (newer version).
    Supersedes,
    /// Source conflicts with target.
    ConflictsWith,
    /// Source validates target (e.g., a test validates a story).
    Validates,
    /// Source blocks target.
    Blocks,
    /// Source is approved by target (e.g., task approved by approval record).
    ApprovedBy,
}

impl fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelationshipType::ParentChild => write!(f, "parent_child"),
            RelationshipType::Governs => write!(f, "governs"),
            RelationshipType::References => write!(f, "references"),
            RelationshipType::DerivedFrom => write!(f, "derived_from"),
            RelationshipType::Supersedes => write!(f, "supersedes"),
            RelationshipType::ConflictsWith => write!(f, "conflicts_with"),
            RelationshipType::Validates => write!(f, "validates"),
            RelationshipType::Blocks => write!(f, "blocks"),
            RelationshipType::ApprovedBy => write!(f, "approved_by"),
        }
    }
}

impl FromStr for RelationshipType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "parent_child" | "parent" | "child" => Ok(RelationshipType::ParentChild),
            "governs" | "governed_by" => Ok(RelationshipType::Governs),
            "references" | "referenced_by" | "ref" => Ok(RelationshipType::References),
            "derived_from" | "derives" => Ok(RelationshipType::DerivedFrom),
            "supersedes" | "superseded_by" => Ok(RelationshipType::Supersedes),
            "conflicts_with" | "conflicts" => Ok(RelationshipType::ConflictsWith),
            "validates" | "validated_by" => Ok(RelationshipType::Validates),
            "blocks" | "blocked_by" => Ok(RelationshipType::Blocks),
            "approved_by" | "approves" => Ok(RelationshipType::ApprovedBy),
            _ => Err(format!("Unknown relationship type: {}", s)),
        }
    }
}

impl RelationshipType {
    /// Get the inverse relationship type for traversal in the opposite direction.
    pub fn inverse(&self) -> Self {
        match self {
            RelationshipType::ParentChild => RelationshipType::ParentChild,
            RelationshipType::Governs => RelationshipType::Governs,
            RelationshipType::References => RelationshipType::References,
            RelationshipType::DerivedFrom => RelationshipType::DerivedFrom,
            RelationshipType::Supersedes => RelationshipType::Supersedes,
            RelationshipType::ConflictsWith => RelationshipType::ConflictsWith,
            RelationshipType::Validates => RelationshipType::Validates,
            RelationshipType::Blocks => RelationshipType::Blocks,
            RelationshipType::ApprovedBy => RelationshipType::ApprovedBy,
        }
    }

    /// Whether this relationship type is inherently symmetric.
    pub fn is_symmetric(&self) -> bool {
        matches!(
            self,
            RelationshipType::References | RelationshipType::ConflictsWith
        )
    }

    /// All available relationship types.
    pub fn all() -> &'static [RelationshipType] {
        &[
            RelationshipType::ParentChild,
            RelationshipType::Governs,
            RelationshipType::References,
            RelationshipType::DerivedFrom,
            RelationshipType::Supersedes,
            RelationshipType::ConflictsWith,
            RelationshipType::Validates,
            RelationshipType::Blocks,
            RelationshipType::ApprovedBy,
        ]
    }
}

// ---------------------------------------------------------------------------
// CrossReference
// ---------------------------------------------------------------------------

/// A typed cross-reference between two documents in the traceability index.
///
/// Each cross-reference links a source document to a target document with a
/// specific relationship type. Together, all cross-references form a queryable
/// graph of document relationships.
#[derive(Debug)]
pub struct CrossReference {
    core: DocumentCore,
    /// Short code of the source document.
    pub source_ref: String,
    /// Short code of the target document.
    pub target_ref: String,
    /// Type of relationship between source and target.
    pub relationship_type: RelationshipType,
    /// Human-readable description of this relationship.
    pub description: String,
    /// Whether this reference should be traversable in both directions.
    pub bidirectional: bool,
}

impl CrossReference {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        source_ref: String,
        target_ref: String,
        relationship_type: RelationshipType,
        description: String,
        bidirectional: bool,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let template_content = include_str!("content.md");
        let mut tera = Tera::default();
        tera.add_raw_template("cross_reference_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("cross_reference_content", &context)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template render error: {}", e))
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
            source_ref,
            target_ref,
            relationship_type,
            description,
            bidirectional,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        source_ref: String,
        target_ref: String,
        relationship_type: RelationshipType,
        description: String,
        bidirectional: bool,
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
            source_ref,
            target_ref,
            relationship_type,
            description,
            bidirectional,
        }
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
        if level != "cross_reference" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'cross_reference', found '{}'",
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
        let short_code = FrontmatterParser::extract_string(&fm_map, "short_code")?;

        let source_ref = FrontmatterParser::extract_string(&fm_map, "source_ref")?;
        let target_ref = FrontmatterParser::extract_string(&fm_map, "target_ref")?;
        let relationship_type_str =
            FrontmatterParser::extract_string(&fm_map, "relationship_type")?;
        let relationship_type = RelationshipType::from_str(&relationship_type_str)
            .map_err(|e| DocumentValidationError::InvalidContent(e))?;
        let description =
            FrontmatterParser::extract_string(&fm_map, "description").unwrap_or_default();
        let bidirectional =
            FrontmatterParser::extract_bool(&fm_map, "bidirectional").unwrap_or(false);

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
            tags,
            archived,
            source_ref,
            target_ref,
            relationship_type,
            description,
            bidirectional,
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
        tera.add_raw_template("frontmatter", include_str!("frontmatter.yaml"))
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("slug", &self.id().to_string());
        context.insert("title", self.title());
        context.insert("short_code", &self.core.metadata.short_code);
        context.insert("created_at", &self.core.metadata.created_at.to_rfc3339());
        context.insert("updated_at", &self.core.metadata.updated_at.to_rfc3339());
        context.insert("archived", &self.core.archived.to_string());
        context.insert(
            "exit_criteria_met",
            &self.core.metadata.exit_criteria_met.to_string(),
        );

        context.insert("source_ref", &self.source_ref);
        context.insert("target_ref", &self.target_ref);
        context.insert("relationship_type", &self.relationship_type.to_string());
        context.insert("description", &self.description);
        context.insert("bidirectional", &self.bidirectional.to_string());

        let tag_strings: Vec<String> = self.core.tags.iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);

        let frontmatter = tera.render("frontmatter", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Frontmatter render error: {}", e))
        })?;

        let content_body = &self.core.content.body;
        let acceptance_criteria = if let Some(ac) = &self.core.content.acceptance_criteria {
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

    /// Check if this cross-reference involves a given document (as source or target).
    pub fn involves(&self, short_code: &str) -> bool {
        self.source_ref == short_code || self.target_ref == short_code
    }

    /// Get the "other" document in this relationship given one side.
    pub fn other_side(&self, short_code: &str) -> Option<&str> {
        if self.source_ref == short_code {
            Some(&self.target_ref)
        } else if self.target_ref == short_code {
            Some(&self.source_ref)
        } else {
            None
        }
    }

    // Convenience accessors

    pub fn id(&self) -> DocumentId {
        DocumentId::from_title(&self.core.title)
    }

    pub fn title(&self) -> &str {
        &self.core.title
    }

    pub fn tags(&self) -> &[Tag] {
        &self.core.tags
    }

    pub fn archived(&self) -> bool {
        self.core.archived
    }

    pub fn phase(&self) -> Result<Phase, DocumentValidationError> {
        for tag in &self.core.tags {
            if let Tag::Phase(phase) = tag {
                return Ok(*phase);
            }
        }
        Err(DocumentValidationError::MissingPhaseTag)
    }

    pub fn validate(&self) -> Result<(), DocumentValidationError> {
        if self.core.title.trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "CrossReference title cannot be empty".to_string(),
            ));
        }
        if self.source_ref.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "source_ref".to_string(),
            ));
        }
        if self.target_ref.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "target_ref".to_string(),
            ));
        }
        if self.source_ref == self.target_ref {
            return Err(DocumentValidationError::InvalidContent(
                "source_ref and target_ref cannot be the same document".to_string(),
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// TraceabilityIndex (in-memory index of cross-references)
// ---------------------------------------------------------------------------

/// An in-memory index for querying cross-references efficiently.
///
/// This provides graph-query operations over a collection of CrossReference
/// documents without requiring SQLite. It can be populated from files and
/// used for traversal queries like ancestors, descendants, and relationships.
#[derive(Debug, Default)]
pub struct TraceabilityIndex {
    /// All cross-references in the index.
    entries: Vec<CrossReferenceEntry>,
}

/// A lightweight entry in the traceability index (no full document content).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossReferenceEntry {
    pub source_ref: String,
    pub target_ref: String,
    pub relationship_type: RelationshipType,
    pub bidirectional: bool,
}

impl CrossReferenceEntry {
    pub fn new(
        source_ref: &str,
        target_ref: &str,
        relationship_type: RelationshipType,
        bidirectional: bool,
    ) -> Self {
        Self {
            source_ref: source_ref.to_string(),
            target_ref: target_ref.to_string(),
            relationship_type,
            bidirectional,
        }
    }
}

impl From<&CrossReference> for CrossReferenceEntry {
    fn from(xref: &CrossReference) -> Self {
        Self {
            source_ref: xref.source_ref.clone(),
            target_ref: xref.target_ref.clone(),
            relationship_type: xref.relationship_type,
            bidirectional: xref.bidirectional,
        }
    }
}

impl TraceabilityIndex {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a cross-reference entry to the index.
    pub fn add(&mut self, entry: CrossReferenceEntry) {
        self.entries.push(entry);
    }

    /// Add a CrossReference document to the index.
    pub fn add_from_document(&mut self, xref: &CrossReference) {
        self.entries.push(CrossReferenceEntry::from(xref));
    }

    /// Get all entries in the index.
    pub fn entries(&self) -> &[CrossReferenceEntry] {
        &self.entries
    }

    /// Total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the index is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Find all relationships where the given document is the source.
    pub fn outgoing(&self, short_code: &str) -> Vec<&CrossReferenceEntry> {
        self.entries
            .iter()
            .filter(|e| {
                e.source_ref == short_code || (e.bidirectional && e.target_ref == short_code)
            })
            .collect()
    }

    /// Find all relationships where the given document is the target.
    pub fn incoming(&self, short_code: &str) -> Vec<&CrossReferenceEntry> {
        self.entries
            .iter()
            .filter(|e| {
                e.target_ref == short_code || (e.bidirectional && e.source_ref == short_code)
            })
            .collect()
    }

    /// Find all relationships involving a given document (as source or target).
    pub fn involving(&self, short_code: &str) -> Vec<&CrossReferenceEntry> {
        self.entries
            .iter()
            .filter(|e| e.source_ref == short_code || e.target_ref == short_code)
            .collect()
    }

    /// Find all relationships of a specific type from a given document.
    pub fn outgoing_of_type(
        &self,
        short_code: &str,
        rel_type: RelationshipType,
    ) -> Vec<&CrossReferenceEntry> {
        self.entries
            .iter()
            .filter(|e| {
                e.relationship_type == rel_type
                    && (e.source_ref == short_code
                        || (e.bidirectional && e.target_ref == short_code))
            })
            .collect()
    }

    /// Find all relationships of a specific type targeting a given document.
    pub fn incoming_of_type(
        &self,
        short_code: &str,
        rel_type: RelationshipType,
    ) -> Vec<&CrossReferenceEntry> {
        self.entries
            .iter()
            .filter(|e| {
                e.relationship_type == rel_type
                    && (e.target_ref == short_code
                        || (e.bidirectional && e.source_ref == short_code))
            })
            .collect()
    }

    /// Walk ancestors: follow parent_child relationships upward from a document.
    pub fn ancestors(&self, short_code: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = short_code.to_string();
        let mut visited = std::collections::HashSet::new();

        loop {
            if !visited.insert(current.clone()) {
                break; // cycle detection
            }
            let parent = self.entries.iter().find(|e| {
                e.relationship_type == RelationshipType::ParentChild && e.target_ref == current
            });
            match parent {
                Some(entry) => {
                    result.push(entry.source_ref.clone());
                    current = entry.source_ref.clone();
                }
                None => break,
            }
        }
        result
    }

    /// Walk descendants: follow parent_child relationships downward from a document.
    pub fn descendants(&self, short_code: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();

        queue.push_back(short_code.to_string());
        visited.insert(short_code.to_string());

        while let Some(current) = queue.pop_front() {
            let children: Vec<String> = self
                .entries
                .iter()
                .filter(|e| {
                    e.relationship_type == RelationshipType::ParentChild && e.source_ref == current
                })
                .map(|e| e.target_ref.clone())
                .collect();

            for child in children {
                if visited.insert(child.clone()) {
                    result.push(child.clone());
                    queue.push_back(child);
                }
            }
        }
        result
    }

    /// Find siblings: documents that share the same parent.
    pub fn siblings(&self, short_code: &str) -> Vec<String> {
        // Find the parent first
        let parent = self.entries.iter().find(|e| {
            e.relationship_type == RelationshipType::ParentChild && e.target_ref == short_code
        });

        match parent {
            Some(parent_entry) => {
                let parent_ref = &parent_entry.source_ref;
                self.entries
                    .iter()
                    .filter(|e| {
                        e.relationship_type == RelationshipType::ParentChild
                            && e.source_ref == *parent_ref
                            && e.target_ref != short_code
                    })
                    .map(|e| e.target_ref.clone())
                    .collect()
            }
            None => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_cross_reference() -> CrossReference {
        CrossReference::new(
            "PROJ-S-0001 governs PROJ-T-0042".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-XR-0001".to_string(),
            "PROJ-S-0001".to_string(),
            "PROJ-T-0042".to_string(),
            RelationshipType::Governs,
            "Story governs the implementation task".to_string(),
            false,
        )
        .unwrap()
    }

    #[test]
    fn test_cross_reference_creation() {
        let xref = make_cross_reference();
        assert_eq!(xref.title(), "PROJ-S-0001 governs PROJ-T-0042");
        assert_eq!(xref.source_ref, "PROJ-S-0001");
        assert_eq!(xref.target_ref, "PROJ-T-0042");
        assert_eq!(xref.relationship_type, RelationshipType::Governs);
        assert!(!xref.bidirectional);
        assert!(xref.validate().is_ok());
    }

    #[test]
    fn test_cross_reference_empty_title_invalid() {
        let xref = CrossReference::new(
            String::new(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-XR-0002".to_string(),
            "A".to_string(),
            "B".to_string(),
            RelationshipType::References,
            String::new(),
            false,
        )
        .unwrap();
        assert!(xref.validate().is_err());
    }

    #[test]
    fn test_cross_reference_self_reference_invalid() {
        let xref = CrossReference::new(
            "Self ref".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-XR-0003".to_string(),
            "PROJ-T-0001".to_string(),
            "PROJ-T-0001".to_string(),
            RelationshipType::References,
            String::new(),
            false,
        )
        .unwrap();
        assert!(xref.validate().is_err());
    }

    #[test]
    fn test_involves_and_other_side() {
        let xref = make_cross_reference();
        assert!(xref.involves("PROJ-S-0001"));
        assert!(xref.involves("PROJ-T-0042"));
        assert!(!xref.involves("PROJ-T-0099"));

        assert_eq!(xref.other_side("PROJ-S-0001"), Some("PROJ-T-0042"));
        assert_eq!(xref.other_side("PROJ-T-0042"), Some("PROJ-S-0001"));
        assert_eq!(xref.other_side("PROJ-T-0099"), None);
    }

    #[test]
    fn test_relationship_type_parsing() {
        assert_eq!(
            "parent_child".parse::<RelationshipType>().unwrap(),
            RelationshipType::ParentChild
        );
        assert_eq!(
            "governs".parse::<RelationshipType>().unwrap(),
            RelationshipType::Governs
        );
        assert_eq!(
            "references".parse::<RelationshipType>().unwrap(),
            RelationshipType::References
        );
        assert_eq!(
            "derived_from".parse::<RelationshipType>().unwrap(),
            RelationshipType::DerivedFrom
        );
        assert_eq!(
            "supersedes".parse::<RelationshipType>().unwrap(),
            RelationshipType::Supersedes
        );
        assert_eq!(
            "conflicts_with".parse::<RelationshipType>().unwrap(),
            RelationshipType::ConflictsWith
        );
        assert_eq!(
            "validates".parse::<RelationshipType>().unwrap(),
            RelationshipType::Validates
        );
        assert_eq!(
            "blocks".parse::<RelationshipType>().unwrap(),
            RelationshipType::Blocks
        );
        assert_eq!(
            "approved_by".parse::<RelationshipType>().unwrap(),
            RelationshipType::ApprovedBy
        );
        assert!("invalid".parse::<RelationshipType>().is_err());
    }

    #[test]
    fn test_relationship_type_symmetric() {
        assert!(RelationshipType::References.is_symmetric());
        assert!(RelationshipType::ConflictsWith.is_symmetric());
        assert!(!RelationshipType::Governs.is_symmetric());
        assert!(!RelationshipType::ParentChild.is_symmetric());
    }

    #[test]
    fn test_relationship_type_all() {
        assert_eq!(RelationshipType::all().len(), 9);
    }

    #[tokio::test]
    async fn test_cross_reference_roundtrip() {
        let xref = make_cross_reference();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-cross-reference.md");

        xref.to_file(&file_path).await.unwrap();
        let loaded = CrossReference::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), xref.title());
        assert_eq!(loaded.source_ref, xref.source_ref);
        assert_eq!(loaded.target_ref, xref.target_ref);
        assert_eq!(loaded.relationship_type, xref.relationship_type);
        assert_eq!(loaded.description, xref.description);
        assert_eq!(loaded.bidirectional, xref.bidirectional);
    }

    // TraceabilityIndex tests

    fn build_test_index() -> TraceabilityIndex {
        let mut index = TraceabilityIndex::new();
        // Vision -> Initiative -> Task hierarchy
        index.add(CrossReferenceEntry::new(
            "PROJ-V-0001",
            "PROJ-I-0001",
            RelationshipType::ParentChild,
            false,
        ));
        index.add(CrossReferenceEntry::new(
            "PROJ-I-0001",
            "PROJ-T-0001",
            RelationshipType::ParentChild,
            false,
        ));
        index.add(CrossReferenceEntry::new(
            "PROJ-I-0001",
            "PROJ-T-0002",
            RelationshipType::ParentChild,
            false,
        ));
        // Story governs task
        index.add(CrossReferenceEntry::new(
            "PROJ-S-0001",
            "PROJ-T-0001",
            RelationshipType::Governs,
            false,
        ));
        // Bidirectional conflict
        index.add(CrossReferenceEntry::new(
            "PROJ-T-0001",
            "PROJ-T-0003",
            RelationshipType::ConflictsWith,
            true,
        ));
        index
    }

    #[test]
    fn test_index_basic_operations() {
        let index = build_test_index();
        assert_eq!(index.len(), 5);
        assert!(!index.is_empty());
    }

    #[test]
    fn test_index_outgoing() {
        let index = build_test_index();
        let out = index.outgoing("PROJ-I-0001");
        assert_eq!(out.len(), 2); // Two child tasks
    }

    #[test]
    fn test_index_incoming() {
        let index = build_test_index();
        let inc = index.incoming("PROJ-T-0001");
        // parent_child from PROJ-I-0001, governs from PROJ-S-0001, conflict (bidirectional)
        assert_eq!(inc.len(), 3);
    }

    #[test]
    fn test_index_involving() {
        let index = build_test_index();
        let inv = index.involving("PROJ-T-0001");
        // parent_child target, governs target, conflicts_with source
        assert_eq!(inv.len(), 3);
    }

    #[test]
    fn test_index_typed_queries() {
        let index = build_test_index();
        let governs = index.outgoing_of_type("PROJ-S-0001", RelationshipType::Governs);
        assert_eq!(governs.len(), 1);
        assert_eq!(governs[0].target_ref, "PROJ-T-0001");

        let governed_by = index.incoming_of_type("PROJ-T-0001", RelationshipType::Governs);
        assert_eq!(governed_by.len(), 1);
    }

    #[test]
    fn test_index_ancestors() {
        let index = build_test_index();
        let ancestors = index.ancestors("PROJ-T-0001");
        assert_eq!(ancestors, vec!["PROJ-I-0001", "PROJ-V-0001"]);
    }

    #[test]
    fn test_index_descendants() {
        let index = build_test_index();
        let desc = index.descendants("PROJ-V-0001");
        assert_eq!(desc.len(), 3); // I-0001, T-0001, T-0002
        assert!(desc.contains(&"PROJ-I-0001".to_string()));
        assert!(desc.contains(&"PROJ-T-0001".to_string()));
        assert!(desc.contains(&"PROJ-T-0002".to_string()));
    }

    #[test]
    fn test_index_siblings() {
        let index = build_test_index();
        let siblings = index.siblings("PROJ-T-0001");
        assert_eq!(siblings, vec!["PROJ-T-0002"]);

        let siblings2 = index.siblings("PROJ-T-0002");
        assert_eq!(siblings2, vec!["PROJ-T-0001"]);
    }

    #[test]
    fn test_index_no_ancestors() {
        let index = build_test_index();
        let ancestors = index.ancestors("PROJ-V-0001");
        assert!(ancestors.is_empty());
    }

    #[test]
    fn test_index_no_descendants() {
        let index = build_test_index();
        let desc = index.descendants("PROJ-T-0001");
        assert!(desc.is_empty());
    }

    #[test]
    fn test_index_add_from_document() {
        let xref = make_cross_reference();
        let mut index = TraceabilityIndex::new();
        index.add_from_document(&xref);
        assert_eq!(index.len(), 1);
        assert_eq!(index.entries()[0].source_ref, "PROJ-S-0001");
    }
}
