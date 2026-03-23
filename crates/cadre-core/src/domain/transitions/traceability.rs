//! Traceability Query Engine -- walk document hierarchies and relationships.
//!
//! Provides an in-memory document graph that supports:
//! - Ancestor queries (walk up the hierarchy to the root)
//! - Descendant queries (walk down to all children/grandchildren)
//! - Sibling queries (other documents sharing the same parent)
//! - Cross-reference relationship queries
//!
//! The graph is built from registered document nodes with parent pointers
//! and explicit cross-reference edges. It does not own the documents
//! themselves -- only their IDs, types, and relationships.

use crate::domain::documents::types::{DocumentId, DocumentType};
use std::collections::{HashMap, HashSet};
use std::fmt;

// ---------------------------------------------------------------------------
// DocumentNode -- lightweight metadata for the graph
// ---------------------------------------------------------------------------

/// A lightweight node in the traceability graph.
///
/// Contains just enough information to navigate the hierarchy
/// without requiring the full document to be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentNode {
    /// The document's unique ID.
    pub id: DocumentId,
    /// The document type.
    pub document_type: DocumentType,
    /// The parent document ID (None for root documents).
    pub parent_id: Option<DocumentId>,
    /// Human-readable title for display purposes.
    pub title: String,
}

impl DocumentNode {
    pub fn new(
        id: DocumentId,
        document_type: DocumentType,
        parent_id: Option<DocumentId>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            id,
            document_type,
            parent_id,
            title: title.into(),
        }
    }

    /// Whether this is a root node (no parent).
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
}

impl fmt::Display for DocumentNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}: {})", self.id, self.document_type, self.title)
    }
}

// ---------------------------------------------------------------------------
// CrossReferenceEdge -- a relationship between two documents
// ---------------------------------------------------------------------------

/// A typed edge between two documents in the traceability graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrossReferenceEdge {
    /// The source document.
    pub source_id: DocumentId,
    /// The target document.
    pub target_id: DocumentId,
    /// The type of relationship.
    pub relationship: EdgeType,
}

/// Types of cross-reference relationships in the traceability graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeType {
    /// General reference link.
    References,
    /// Source blocks target.
    Blocks,
    /// Source is derived from target.
    DerivedFrom,
    /// Source supersedes target.
    Supersedes,
    /// Source validates target.
    Validates,
    /// Source governs target.
    Governs,
}

impl fmt::Display for EdgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::References => write!(f, "references"),
            Self::Blocks => write!(f, "blocks"),
            Self::DerivedFrom => write!(f, "derived_from"),
            Self::Supersedes => write!(f, "supersedes"),
            Self::Validates => write!(f, "validates"),
            Self::Governs => write!(f, "governs"),
        }
    }
}

// ---------------------------------------------------------------------------
// TraceabilityGraph -- the query engine
// ---------------------------------------------------------------------------

/// An in-memory graph for traceability queries across documents.
///
/// Supports hierarchy traversal (ancestors, descendants, siblings) and
/// cross-reference relationship queries. Built incrementally by adding
/// nodes and edges.
#[derive(Debug)]
pub struct TraceabilityGraph {
    /// All registered document nodes, indexed by ID.
    nodes: HashMap<DocumentId, DocumentNode>,
    /// Parent -> children index for fast descendant queries.
    children_index: HashMap<DocumentId, Vec<DocumentId>>,
    /// Cross-reference edges.
    edges: Vec<CrossReferenceEdge>,
}

impl TraceabilityGraph {
    /// Create an empty traceability graph.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            children_index: HashMap::new(),
            edges: Vec::new(),
        }
    }

    // -- Building the graph -------------------------------------------------

    /// Add a document node to the graph.
    ///
    /// If the node has a parent, updates the children index automatically.
    pub fn add_node(&mut self, node: DocumentNode) {
        if let Some(ref parent_id) = node.parent_id {
            self.children_index
                .entry(parent_id.clone())
                .or_default()
                .push(node.id.clone());
        }
        self.nodes.insert(node.id.clone(), node);
    }

    /// Remove a node from the graph by ID. Returns the removed node if found.
    pub fn remove_node(&mut self, id: &DocumentId) -> Option<DocumentNode> {
        if let Some(node) = self.nodes.remove(id) {
            // Remove from parent's children index
            if let Some(ref parent_id) = node.parent_id {
                if let Some(children) = self.children_index.get_mut(parent_id) {
                    children.retain(|c| c != id);
                }
            }
            // Remove this node's children index entry
            self.children_index.remove(id);
            // Remove edges involving this node
            self.edges
                .retain(|e| &e.source_id != id && &e.target_id != id);
            Some(node)
        } else {
            None
        }
    }

    /// Add a cross-reference edge between two documents.
    pub fn add_edge(&mut self, edge: CrossReferenceEdge) {
        self.edges.push(edge);
    }

    /// Add a cross-reference edge by specifying source, target, and relationship.
    pub fn add_reference(
        &mut self,
        source: DocumentId,
        target: DocumentId,
        relationship: EdgeType,
    ) {
        self.add_edge(CrossReferenceEdge {
            source_id: source,
            target_id: target,
            relationship,
        });
    }

    // -- Basic queries ------------------------------------------------------

    /// Get a node by its ID.
    pub fn get_node(&self, id: &DocumentId) -> Option<&DocumentNode> {
        self.nodes.get(id)
    }

    /// Get total number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get total number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get all root nodes (nodes with no parent).
    pub fn roots(&self) -> Vec<&DocumentNode> {
        self.nodes.values().filter(|n| n.is_root()).collect()
    }

    // -- Hierarchy queries --------------------------------------------------

    /// Get the direct parent of a document.
    pub fn parent(&self, id: &DocumentId) -> Option<&DocumentNode> {
        self.nodes
            .get(id)
            .and_then(|node| node.parent_id.as_ref())
            .and_then(|parent_id| self.nodes.get(parent_id))
    }

    /// Get all ancestors of a document, from immediate parent up to root.
    ///
    /// Returns ancestors in order: [parent, grandparent, ..., root].
    pub fn ancestors(&self, id: &DocumentId) -> Vec<&DocumentNode> {
        let mut result = Vec::new();
        let mut current_id = id.clone();
        let mut visited = HashSet::new();

        while let Some(node) = self.nodes.get(&current_id) {
            if let Some(ref parent_id) = node.parent_id {
                if visited.contains(parent_id) {
                    break; // cycle protection
                }
                if let Some(parent) = self.nodes.get(parent_id) {
                    result.push(parent);
                    visited.insert(parent_id.clone());
                    current_id = parent_id.clone();
                } else {
                    break; // parent not in graph
                }
            } else {
                break; // reached root
            }
        }

        result
    }

    /// Get the direct children of a document.
    pub fn children(&self, id: &DocumentId) -> Vec<&DocumentNode> {
        self.children_index
            .get(id)
            .map(|child_ids| {
                child_ids
                    .iter()
                    .filter_map(|child_id| self.nodes.get(child_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all descendants of a document (children, grandchildren, etc.).
    ///
    /// Returns descendants in breadth-first order.
    pub fn descendants(&self, id: &DocumentId) -> Vec<&DocumentNode> {
        let mut result = Vec::new();
        let mut queue = vec![id.clone()];
        let mut visited = HashSet::new();
        visited.insert(id.clone());

        while let Some(current_id) = queue.first().cloned() {
            queue.remove(0);

            if let Some(child_ids) = self.children_index.get(&current_id) {
                for child_id in child_ids {
                    if visited.insert(child_id.clone()) {
                        if let Some(child) = self.nodes.get(child_id) {
                            result.push(child);
                            queue.push(child_id.clone());
                        }
                    }
                }
            }
        }

        result
    }

    /// Get siblings of a document (other documents with the same parent).
    ///
    /// Does not include the document itself.
    pub fn siblings(&self, id: &DocumentId) -> Vec<&DocumentNode> {
        let parent_id = self.nodes.get(id).and_then(|n| n.parent_id.as_ref());

        match parent_id {
            Some(parent_id) => self
                .children_index
                .get(parent_id)
                .map(|children| {
                    children
                        .iter()
                        .filter(|c| *c != id)
                        .filter_map(|c| self.nodes.get(c))
                        .collect()
                })
                .unwrap_or_default(),
            None => Vec::new(),
        }
    }

    /// Get the full lineage path from root down to the given document.
    ///
    /// Returns [root, ..., grandparent, parent, self].
    pub fn lineage(&self, id: &DocumentId) -> Vec<&DocumentNode> {
        let mut ancestors = self.ancestors(id);
        ancestors.reverse(); // root first
        if let Some(node) = self.nodes.get(id) {
            ancestors.push(node);
        }
        ancestors
    }

    // -- Cross-reference queries --------------------------------------------

    /// Get all outgoing edges from a document (where it is the source).
    pub fn outgoing_edges(&self, id: &DocumentId) -> Vec<&CrossReferenceEdge> {
        self.edges.iter().filter(|e| &e.source_id == id).collect()
    }

    /// Get all incoming edges to a document (where it is the target).
    pub fn incoming_edges(&self, id: &DocumentId) -> Vec<&CrossReferenceEdge> {
        self.edges.iter().filter(|e| &e.target_id == id).collect()
    }

    /// Get all edges of a specific type from a document.
    pub fn outgoing_edges_of_type(
        &self,
        id: &DocumentId,
        edge_type: EdgeType,
    ) -> Vec<&CrossReferenceEdge> {
        self.edges
            .iter()
            .filter(|e| &e.source_id == id && e.relationship == edge_type)
            .collect()
    }

    /// Get all documents that a given document blocks.
    pub fn blocks(&self, id: &DocumentId) -> Vec<&DocumentNode> {
        self.outgoing_edges_of_type(id, EdgeType::Blocks)
            .iter()
            .filter_map(|e| self.nodes.get(&e.target_id))
            .collect()
    }

    /// Get all documents that block a given document.
    pub fn blocked_by(&self, id: &DocumentId) -> Vec<&DocumentNode> {
        self.edges
            .iter()
            .filter(|e| &e.target_id == id && e.relationship == EdgeType::Blocks)
            .filter_map(|e| self.nodes.get(&e.source_id))
            .collect()
    }

    /// Get all documents related to a given document (any edge direction).
    pub fn related(&self, id: &DocumentId) -> Vec<(&DocumentNode, &CrossReferenceEdge)> {
        let mut result = Vec::new();

        for edge in &self.edges {
            if &edge.source_id == id {
                if let Some(target) = self.nodes.get(&edge.target_id) {
                    result.push((target, edge));
                }
            } else if &edge.target_id == id {
                if let Some(source) = self.nodes.get(&edge.source_id) {
                    result.push((source, edge));
                }
            }
        }

        result
    }
}

impl Default for TraceabilityGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a sample hierarchy:
    ///   product-doc
    ///     ├── epic-1
    ///     │     ├── story-1
    ///     │     │     ├── task-1
    ///     │     │     └── task-2
    ///     │     └── story-2
    ///     │           └── task-3
    ///     └── epic-2
    ///           └── story-3
    fn build_sample_graph() -> TraceabilityGraph {
        let mut graph = TraceabilityGraph::new();

        graph.add_node(DocumentNode::new(
            DocumentId::from("product-doc"),
            DocumentType::ProductDoc,
            None,
            "Product Doc",
        ));
        graph.add_node(DocumentNode::new(
            DocumentId::from("epic-1"),
            DocumentType::Epic,
            Some(DocumentId::from("product-doc")),
            "Epic 1",
        ));
        graph.add_node(DocumentNode::new(
            DocumentId::from("epic-2"),
            DocumentType::Epic,
            Some(DocumentId::from("product-doc")),
            "Epic 2",
        ));
        graph.add_node(DocumentNode::new(
            DocumentId::from("story-1"),
            DocumentType::Story,
            Some(DocumentId::from("epic-1")),
            "Story 1",
        ));
        graph.add_node(DocumentNode::new(
            DocumentId::from("story-2"),
            DocumentType::Story,
            Some(DocumentId::from("epic-1")),
            "Story 2",
        ));
        graph.add_node(DocumentNode::new(
            DocumentId::from("story-3"),
            DocumentType::Story,
            Some(DocumentId::from("epic-2")),
            "Story 3",
        ));
        graph.add_node(DocumentNode::new(
            DocumentId::from("task-1"),
            DocumentType::Task,
            Some(DocumentId::from("story-1")),
            "Task 1",
        ));
        graph.add_node(DocumentNode::new(
            DocumentId::from("task-2"),
            DocumentType::Task,
            Some(DocumentId::from("story-1")),
            "Task 2",
        ));
        graph.add_node(DocumentNode::new(
            DocumentId::from("task-3"),
            DocumentType::Task,
            Some(DocumentId::from("story-2")),
            "Task 3",
        ));

        graph
    }

    #[test]
    fn test_empty_graph() {
        let graph = TraceabilityGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
        assert!(graph.roots().is_empty());
    }

    #[test]
    fn test_add_and_get_node() {
        let mut graph = TraceabilityGraph::new();
        graph.add_node(DocumentNode::new(
            DocumentId::from("doc-1"),
            DocumentType::Task,
            None,
            "Doc 1",
        ));

        assert_eq!(graph.node_count(), 1);
        let node = graph.get_node(&DocumentId::from("doc-1")).unwrap();
        assert_eq!(node.title, "Doc 1");
        assert!(node.is_root());
    }

    #[test]
    fn test_roots() {
        let graph = build_sample_graph();
        let roots = graph.roots();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].id, DocumentId::from("product-doc"));
    }

    #[test]
    fn test_parent() {
        let graph = build_sample_graph();

        let parent = graph.parent(&DocumentId::from("epic-1")).unwrap();
        assert_eq!(parent.id, DocumentId::from("product-doc"));

        let parent = graph.parent(&DocumentId::from("task-1")).unwrap();
        assert_eq!(parent.id, DocumentId::from("story-1"));

        assert!(graph.parent(&DocumentId::from("product-doc")).is_none());
    }

    #[test]
    fn test_ancestors() {
        let graph = build_sample_graph();

        let ancestors = graph.ancestors(&DocumentId::from("task-1"));
        assert_eq!(ancestors.len(), 3);
        assert_eq!(ancestors[0].id, DocumentId::from("story-1"));
        assert_eq!(ancestors[1].id, DocumentId::from("epic-1"));
        assert_eq!(ancestors[2].id, DocumentId::from("product-doc"));
    }

    #[test]
    fn test_ancestors_root() {
        let graph = build_sample_graph();
        let ancestors = graph.ancestors(&DocumentId::from("product-doc"));
        assert!(ancestors.is_empty());
    }

    #[test]
    fn test_children() {
        let graph = build_sample_graph();

        let children = graph.children(&DocumentId::from("epic-1"));
        assert_eq!(children.len(), 2);

        let child_ids: HashSet<_> = children.iter().map(|c| c.id.clone()).collect();
        assert!(child_ids.contains(&DocumentId::from("story-1")));
        assert!(child_ids.contains(&DocumentId::from("story-2")));
    }

    #[test]
    fn test_children_leaf() {
        let graph = build_sample_graph();
        let children = graph.children(&DocumentId::from("task-1"));
        assert!(children.is_empty());
    }

    #[test]
    fn test_descendants() {
        let graph = build_sample_graph();

        let descendants = graph.descendants(&DocumentId::from("epic-1"));
        assert_eq!(descendants.len(), 5); // story-1, story-2, task-1, task-2, task-3

        let desc_ids: HashSet<_> = descendants.iter().map(|d| d.id.clone()).collect();
        assert!(desc_ids.contains(&DocumentId::from("story-1")));
        assert!(desc_ids.contains(&DocumentId::from("story-2")));
        assert!(desc_ids.contains(&DocumentId::from("task-1")));
        assert!(desc_ids.contains(&DocumentId::from("task-2")));
        assert!(desc_ids.contains(&DocumentId::from("task-3")));
    }

    #[test]
    fn test_descendants_of_root() {
        let graph = build_sample_graph();
        let descendants = graph.descendants(&DocumentId::from("product-doc"));
        assert_eq!(descendants.len(), 8); // everything except root
    }

    #[test]
    fn test_siblings() {
        let graph = build_sample_graph();

        let siblings = graph.siblings(&DocumentId::from("story-1"));
        assert_eq!(siblings.len(), 1);
        assert_eq!(siblings[0].id, DocumentId::from("story-2"));

        let siblings = graph.siblings(&DocumentId::from("task-1"));
        assert_eq!(siblings.len(), 1);
        assert_eq!(siblings[0].id, DocumentId::from("task-2"));
    }

    #[test]
    fn test_siblings_root_has_none() {
        let graph = build_sample_graph();
        let siblings = graph.siblings(&DocumentId::from("product-doc"));
        assert!(siblings.is_empty());
    }

    #[test]
    fn test_siblings_only_child() {
        let graph = build_sample_graph();
        let siblings = graph.siblings(&DocumentId::from("story-3"));
        assert!(siblings.is_empty()); // only child of epic-2
    }

    #[test]
    fn test_lineage() {
        let graph = build_sample_graph();

        let lineage = graph.lineage(&DocumentId::from("task-1"));
        assert_eq!(lineage.len(), 4);
        assert_eq!(lineage[0].id, DocumentId::from("product-doc"));
        assert_eq!(lineage[1].id, DocumentId::from("epic-1"));
        assert_eq!(lineage[2].id, DocumentId::from("story-1"));
        assert_eq!(lineage[3].id, DocumentId::from("task-1"));
    }

    #[test]
    fn test_lineage_root() {
        let graph = build_sample_graph();
        let lineage = graph.lineage(&DocumentId::from("product-doc"));
        assert_eq!(lineage.len(), 1);
        assert_eq!(lineage[0].id, DocumentId::from("product-doc"));
    }

    #[test]
    fn test_cross_reference_edges() {
        let mut graph = build_sample_graph();

        // task-1 blocks task-3
        graph.add_reference(
            DocumentId::from("task-1"),
            DocumentId::from("task-3"),
            EdgeType::Blocks,
        );
        // story-1 references epic-2
        graph.add_reference(
            DocumentId::from("story-1"),
            DocumentId::from("epic-2"),
            EdgeType::References,
        );

        assert_eq!(graph.edge_count(), 2);

        let outgoing = graph.outgoing_edges(&DocumentId::from("task-1"));
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].relationship, EdgeType::Blocks);

        let incoming = graph.incoming_edges(&DocumentId::from("task-3"));
        assert_eq!(incoming.len(), 1);
    }

    #[test]
    fn test_blocks_and_blocked_by() {
        let mut graph = build_sample_graph();

        graph.add_reference(
            DocumentId::from("task-1"),
            DocumentId::from("task-3"),
            EdgeType::Blocks,
        );

        let blocks = graph.blocks(&DocumentId::from("task-1"));
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id, DocumentId::from("task-3"));

        let blocked_by = graph.blocked_by(&DocumentId::from("task-3"));
        assert_eq!(blocked_by.len(), 1);
        assert_eq!(blocked_by[0].id, DocumentId::from("task-1"));
    }

    #[test]
    fn test_related() {
        let mut graph = build_sample_graph();

        graph.add_reference(
            DocumentId::from("task-1"),
            DocumentId::from("task-3"),
            EdgeType::Blocks,
        );
        graph.add_reference(
            DocumentId::from("story-2"),
            DocumentId::from("task-1"),
            EdgeType::Validates,
        );

        let related = graph.related(&DocumentId::from("task-1"));
        assert_eq!(related.len(), 2);
    }

    #[test]
    fn test_outgoing_edges_of_type() {
        let mut graph = build_sample_graph();

        graph.add_reference(
            DocumentId::from("task-1"),
            DocumentId::from("task-2"),
            EdgeType::Blocks,
        );
        graph.add_reference(
            DocumentId::from("task-1"),
            DocumentId::from("task-3"),
            EdgeType::References,
        );

        let blocks = graph.outgoing_edges_of_type(&DocumentId::from("task-1"), EdgeType::Blocks);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].target_id, DocumentId::from("task-2"));
    }

    #[test]
    fn test_remove_node() {
        let mut graph = build_sample_graph();

        graph.add_reference(
            DocumentId::from("task-1"),
            DocumentId::from("task-3"),
            EdgeType::Blocks,
        );

        assert_eq!(graph.node_count(), 9);
        let removed = graph.remove_node(&DocumentId::from("task-1"));
        assert!(removed.is_some());
        assert_eq!(graph.node_count(), 8);

        // Edge should be removed too
        assert_eq!(graph.edge_count(), 0);

        // Should be gone from parent's children
        let siblings = graph.siblings(&DocumentId::from("task-2"));
        assert!(siblings.is_empty()); // task-1 removed, task-2 is now only child
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut graph = build_sample_graph();
        assert!(graph
            .remove_node(&DocumentId::from("nonexistent"))
            .is_none());
    }

    #[test]
    fn test_document_node_display() {
        let node = DocumentNode::new(
            DocumentId::from("task-1"),
            DocumentType::Task,
            None,
            "My Task",
        );
        let display = node.to_string();
        assert!(display.contains("task-1"));
        assert!(display.contains("task"));
        assert!(display.contains("My Task"));
    }

    #[test]
    fn test_edge_type_display() {
        assert_eq!(EdgeType::References.to_string(), "references");
        assert_eq!(EdgeType::Blocks.to_string(), "blocks");
        assert_eq!(EdgeType::DerivedFrom.to_string(), "derived_from");
        assert_eq!(EdgeType::Supersedes.to_string(), "supersedes");
        assert_eq!(EdgeType::Validates.to_string(), "validates");
        assert_eq!(EdgeType::Governs.to_string(), "governs");
    }

    #[test]
    fn test_default_trait() {
        let graph = TraceabilityGraph::default();
        assert_eq!(graph.node_count(), 0);
    }
}
