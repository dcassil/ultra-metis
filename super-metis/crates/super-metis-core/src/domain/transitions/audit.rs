//! Transition Audit Log and Blocked Reason Tracking.
//!
//! Provides an in-memory audit log that records all transition attempts
//! (successful, blocked, forced, invalid) and supports queries by document,
//! phase, actor, and time range. Also provides blocked-reason tracking
//! for documents that are in a blocked state.

use super::enforcer::{EnforcementOutcome, EnforcementResult};
use super::hooks::TransitionEvent;
use crate::domain::documents::types::{DocumentId, DocumentType, Phase};
use chrono::{DateTime, Utc};
use std::fmt;

// ---------------------------------------------------------------------------
// AuditEntry -- a single audit log entry
// ---------------------------------------------------------------------------

/// A single entry in the transition audit log.
///
/// Records the full context of a transition attempt including the outcome,
/// which checks ran and their results, and timestamps.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// Unique sequential ID for this entry.
    pub entry_id: u64,
    /// When this transition was attempted.
    pub timestamp: DateTime<Utc>,
    /// The document that was being transitioned.
    pub document_id: DocumentId,
    /// The document type.
    pub document_type: DocumentType,
    /// Phase before the transition attempt.
    pub from_phase: Phase,
    /// Target phase of the transition attempt.
    pub to_phase: Phase,
    /// Who or what triggered the transition.
    pub actor: String,
    /// Whether this was a forced transition.
    pub forced: bool,
    /// The outcome of the enforcement.
    pub outcome: EnforcementOutcome,
    /// Names and results of pre-checks that ran.
    pub check_results: Vec<AuditCheckEntry>,
    /// Names and results of post-actions that ran.
    pub action_results: Vec<AuditActionEntry>,
}

/// Simplified check result for audit storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditCheckEntry {
    pub name: String,
    pub passed: bool,
    pub blocking: bool,
    pub message: String,
}

/// Simplified action result for audit storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditActionEntry {
    pub name: String,
    pub success: bool,
    pub message: String,
}

impl fmt::Display for AuditEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] #{}: {} ({}) {} -> {} by {} [{}]{}",
            self.timestamp.format("%Y-%m-%dT%H:%M:%S"),
            self.entry_id,
            self.document_id,
            self.document_type,
            self.from_phase,
            self.to_phase,
            self.actor,
            self.outcome,
            if self.forced { " FORCED" } else { "" }
        )
    }
}

impl AuditEntry {
    /// Create an audit entry from an enforcement result.
    pub fn from_enforcement(entry_id: u64, result: &EnforcementResult) -> Self {
        let check_results: Vec<AuditCheckEntry> = result
            .pre_check_results
            .iter()
            .map(|r| AuditCheckEntry {
                name: r.check_name.clone(),
                passed: r.passed,
                blocking: r.blocking,
                message: r.message.clone(),
            })
            .collect();

        let action_results: Vec<AuditActionEntry> = result
            .post_action_results
            .iter()
            .map(|r| AuditActionEntry {
                name: r.action_name.clone(),
                success: r.success,
                message: r.message.clone(),
            })
            .collect();

        Self {
            entry_id,
            timestamp: Utc::now(),
            document_id: result.event.document_id.clone(),
            document_type: result.event.document_type,
            from_phase: result.event.from_phase,
            to_phase: result.event.to_phase,
            actor: result.event.actor.clone(),
            forced: result.event.forced,
            outcome: result.outcome,
            check_results,
            action_results,
        }
    }
}

// ---------------------------------------------------------------------------
// TransitionAuditLog -- the audit log
// ---------------------------------------------------------------------------

/// In-memory audit log for all transition attempts.
///
/// Records every transition attempt with full context. Supports queries
/// by document, phase, actor, outcome, and time range.
#[derive(Debug)]
pub struct TransitionAuditLog {
    entries: Vec<AuditEntry>,
    next_id: u64,
}

impl TransitionAuditLog {
    /// Create a new empty audit log.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 1,
        }
    }

    /// Record an enforcement result in the audit log.
    pub fn record(&mut self, result: &EnforcementResult) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let entry = AuditEntry::from_enforcement(id, result);
        self.entries.push(entry);
        id
    }

    /// Get all entries in the log.
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Get the total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get an entry by its ID.
    pub fn get(&self, entry_id: u64) -> Option<&AuditEntry> {
        self.entries.iter().find(|e| e.entry_id == entry_id)
    }

    // -- Query methods ------------------------------------------------------

    /// Get all entries for a specific document.
    pub fn by_document(&self, document_id: &DocumentId) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| &e.document_id == document_id)
            .collect()
    }

    /// Get all entries by a specific actor.
    pub fn by_actor(&self, actor: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.actor == actor)
            .collect()
    }

    /// Get all entries with a specific outcome.
    pub fn by_outcome(&self, outcome: EnforcementOutcome) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.outcome == outcome)
            .collect()
    }

    /// Get all entries for transitions to a specific phase.
    pub fn by_to_phase(&self, phase: Phase) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.to_phase == phase)
            .collect()
    }

    /// Get all entries within a time range (inclusive).
    pub fn by_time_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= from && e.timestamp <= to)
            .collect()
    }

    /// Get the most recent entry for a specific document.
    pub fn latest_for_document(&self, document_id: &DocumentId) -> Option<&AuditEntry> {
        self.entries
            .iter()
            .rev()
            .find(|e| &e.document_id == document_id)
    }

    /// Get all forced transitions.
    pub fn forced_transitions(&self) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.forced).collect()
    }
}

impl Default for TransitionAuditLog {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// BlockedReason -- tracks why a document is blocked
// ---------------------------------------------------------------------------

/// Structured reason for why a document is in a blocked state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedReason {
    /// The document that is blocked.
    pub document_id: DocumentId,
    /// When the block was recorded.
    pub blocked_at: DateTime<Utc>,
    /// Who recorded the block.
    pub blocked_by_actor: String,
    /// The category of the block.
    pub category: BlockCategory,
    /// Human-readable explanation.
    pub reason: String,
    /// IDs of documents that are blocking this one (if dependency-based).
    pub blocking_documents: Vec<DocumentId>,
    /// When the block was resolved (None if still blocked).
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Category of block reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockCategory {
    /// Blocked by a dependency on another document.
    Dependency,
    /// Blocked by a failing quality gate.
    QualityGate,
    /// Blocked by a manual hold from a human.
    ManualHold,
    /// Blocked by a missing prerequisite.
    MissingPrerequisite,
    /// Blocked for an external reason.
    External,
}

impl fmt::Display for BlockCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dependency => write!(f, "dependency"),
            Self::QualityGate => write!(f, "quality_gate"),
            Self::ManualHold => write!(f, "manual_hold"),
            Self::MissingPrerequisite => write!(f, "missing_prerequisite"),
            Self::External => write!(f, "external"),
        }
    }
}

impl BlockedReason {
    /// Create a new blocked reason.
    pub fn new(
        document_id: DocumentId,
        actor: impl Into<String>,
        category: BlockCategory,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            document_id,
            blocked_at: Utc::now(),
            blocked_by_actor: actor.into(),
            category,
            reason: reason.into(),
            blocking_documents: Vec::new(),
            resolved_at: None,
        }
    }

    /// Create a dependency block.
    pub fn dependency(
        document_id: DocumentId,
        actor: impl Into<String>,
        reason: impl Into<String>,
        blocking_docs: Vec<DocumentId>,
    ) -> Self {
        Self {
            document_id,
            blocked_at: Utc::now(),
            blocked_by_actor: actor.into(),
            category: BlockCategory::Dependency,
            reason: reason.into(),
            blocking_documents: blocking_docs,
            resolved_at: None,
        }
    }

    /// Whether this block has been resolved.
    pub fn is_resolved(&self) -> bool {
        self.resolved_at.is_some()
    }

    /// Mark this block as resolved.
    pub fn resolve(&mut self) {
        self.resolved_at = Some(Utc::now());
    }
}

impl fmt::Display for BlockedReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} blocked ({}): {}{}",
            self.document_id,
            self.category,
            self.reason,
            if self.is_resolved() {
                " [RESOLVED]"
            } else {
                ""
            }
        )
    }
}

// ---------------------------------------------------------------------------
// BlockedReasonTracker -- manages blocked reasons for documents
// ---------------------------------------------------------------------------

/// Tracks blocked reasons for documents.
///
/// Maintains a history of all block reasons and their resolution status.
#[derive(Debug, Default)]
pub struct BlockedReasonTracker {
    reasons: Vec<BlockedReason>,
}

impl BlockedReasonTracker {
    /// Create a new empty tracker.
    pub fn new() -> Self {
        Self {
            reasons: Vec::new(),
        }
    }

    /// Record a new blocked reason.
    pub fn record(&mut self, reason: BlockedReason) {
        self.reasons.push(reason);
    }

    /// Get all active (unresolved) blocks for a document.
    pub fn active_blocks(&self, document_id: &DocumentId) -> Vec<&BlockedReason> {
        self.reasons
            .iter()
            .filter(|r| &r.document_id == document_id && !r.is_resolved())
            .collect()
    }

    /// Get all blocks (active and resolved) for a document.
    pub fn all_blocks(&self, document_id: &DocumentId) -> Vec<&BlockedReason> {
        self.reasons
            .iter()
            .filter(|r| &r.document_id == document_id)
            .collect()
    }

    /// Resolve all active blocks for a document.
    pub fn resolve_all(&mut self, document_id: &DocumentId) {
        for reason in &mut self.reasons {
            if &reason.document_id == document_id && !reason.is_resolved() {
                reason.resolve();
            }
        }
    }

    /// Resolve blocks of a specific category for a document.
    pub fn resolve_by_category(
        &mut self,
        document_id: &DocumentId,
        category: BlockCategory,
    ) {
        for reason in &mut self.reasons {
            if &reason.document_id == document_id
                && reason.category == category
                && !reason.is_resolved()
            {
                reason.resolve();
            }
        }
    }

    /// Check if a document has any active blocks.
    pub fn is_blocked(&self, document_id: &DocumentId) -> bool {
        self.reasons
            .iter()
            .any(|r| &r.document_id == document_id && !r.is_resolved())
    }

    /// Get total count of tracked reasons.
    pub fn total_count(&self) -> usize {
        self.reasons.len()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::documents::types::DocumentId;
    use crate::domain::transitions::enforcer::TransitionEnforcer;
    use crate::domain::transitions::hooks::{PreCheckResult, PreTransitionCheck};
    use crate::domain::transitions::registry::HookRegistry;

    fn make_enforcement_result(
        from: Phase,
        to: Phase,
        outcome: EnforcementOutcome,
    ) -> EnforcementResult {
        let event = TransitionEvent::new(
            DocumentId::from("doc-1"),
            DocumentType::Task,
            from,
            to,
            "test-actor",
            false,
        );
        EnforcementResult {
            event,
            outcome,
            pre_check_results: Vec::new(),
            post_action_results: Vec::new(),
        }
    }

    // -- AuditLog tests --

    #[test]
    fn test_empty_audit_log() {
        let log = TransitionAuditLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn test_record_and_retrieve() {
        let mut log = TransitionAuditLog::new();
        let result = make_enforcement_result(
            Phase::Active,
            Phase::Completed,
            EnforcementOutcome::Allowed,
        );

        let id = log.record(&result);
        assert_eq!(id, 1);
        assert_eq!(log.len(), 1);

        let entry = log.get(1).unwrap();
        assert_eq!(entry.document_id, DocumentId::from("doc-1"));
        assert_eq!(entry.from_phase, Phase::Active);
        assert_eq!(entry.to_phase, Phase::Completed);
        assert_eq!(entry.outcome, EnforcementOutcome::Allowed);
    }

    #[test]
    fn test_sequential_ids() {
        let mut log = TransitionAuditLog::new();
        let r1 = make_enforcement_result(Phase::Todo, Phase::Active, EnforcementOutcome::Allowed);
        let r2 = make_enforcement_result(
            Phase::Active,
            Phase::Completed,
            EnforcementOutcome::Allowed,
        );

        assert_eq!(log.record(&r1), 1);
        assert_eq!(log.record(&r2), 2);
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn test_query_by_document() {
        let mut log = TransitionAuditLog::new();

        let event1 = TransitionEvent::new(
            DocumentId::from("doc-a"),
            DocumentType::Task,
            Phase::Todo,
            Phase::Active,
            "actor",
            false,
        );
        let event2 = TransitionEvent::new(
            DocumentId::from("doc-b"),
            DocumentType::Task,
            Phase::Todo,
            Phase::Active,
            "actor",
            false,
        );

        let r1 = EnforcementResult {
            event: event1,
            outcome: EnforcementOutcome::Allowed,
            pre_check_results: Vec::new(),
            post_action_results: Vec::new(),
        };
        let r2 = EnforcementResult {
            event: event2,
            outcome: EnforcementOutcome::Allowed,
            pre_check_results: Vec::new(),
            post_action_results: Vec::new(),
        };

        log.record(&r1);
        log.record(&r2);

        let doc_a = DocumentId::from("doc-a");
        let results = log.by_document(&doc_a);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id, doc_a);
    }

    #[test]
    fn test_query_by_actor() {
        let mut log = TransitionAuditLog::new();
        let result = make_enforcement_result(
            Phase::Active,
            Phase::Completed,
            EnforcementOutcome::Allowed,
        );
        log.record(&result);

        assert_eq!(log.by_actor("test-actor").len(), 1);
        assert_eq!(log.by_actor("other-actor").len(), 0);
    }

    #[test]
    fn test_query_by_outcome() {
        let mut log = TransitionAuditLog::new();
        let allowed = make_enforcement_result(
            Phase::Todo,
            Phase::Active,
            EnforcementOutcome::Allowed,
        );
        let blocked = make_enforcement_result(
            Phase::Active,
            Phase::Completed,
            EnforcementOutcome::Blocked,
        );

        log.record(&allowed);
        log.record(&blocked);

        assert_eq!(log.by_outcome(EnforcementOutcome::Allowed).len(), 1);
        assert_eq!(log.by_outcome(EnforcementOutcome::Blocked).len(), 1);
        assert_eq!(log.by_outcome(EnforcementOutcome::Forced).len(), 0);
    }

    #[test]
    fn test_query_by_to_phase() {
        let mut log = TransitionAuditLog::new();
        let result = make_enforcement_result(
            Phase::Active,
            Phase::Completed,
            EnforcementOutcome::Allowed,
        );
        log.record(&result);

        assert_eq!(log.by_to_phase(Phase::Completed).len(), 1);
        assert_eq!(log.by_to_phase(Phase::Active).len(), 0);
    }

    #[test]
    fn test_latest_for_document() {
        let mut log = TransitionAuditLog::new();
        let r1 = make_enforcement_result(
            Phase::Todo,
            Phase::Active,
            EnforcementOutcome::Allowed,
        );
        let r2 = make_enforcement_result(
            Phase::Active,
            Phase::Completed,
            EnforcementOutcome::Allowed,
        );
        log.record(&r1);
        log.record(&r2);

        let latest = log.latest_for_document(&DocumentId::from("doc-1")).unwrap();
        assert_eq!(latest.to_phase, Phase::Completed);
    }

    #[test]
    fn test_forced_transitions() {
        let mut log = TransitionAuditLog::new();
        let event = TransitionEvent::new(
            DocumentId::from("doc-1"),
            DocumentType::Task,
            Phase::Active,
            Phase::Completed,
            "actor",
            true,
        );
        let result = EnforcementResult {
            event,
            outcome: EnforcementOutcome::Forced,
            pre_check_results: Vec::new(),
            post_action_results: Vec::new(),
        };
        log.record(&result);

        assert_eq!(log.forced_transitions().len(), 1);
    }

    #[test]
    fn test_audit_entry_display() {
        let result = make_enforcement_result(
            Phase::Active,
            Phase::Completed,
            EnforcementOutcome::Allowed,
        );
        let mut log = TransitionAuditLog::new();
        log.record(&result);

        let entry = log.get(1).unwrap();
        let display = entry.to_string();
        assert!(display.contains("doc-1"));
        assert!(display.contains("active"));
        assert!(display.contains("completed"));
        assert!(display.contains("allowed"));
    }

    #[test]
    fn test_audit_from_real_enforcement() {
        let mut registry = HookRegistry::new();
        registry.register_pre_check(PreTransitionCheck::new("gate", |_| {
            PreCheckResult::pass("gate", "ok")
        }));

        let enforcer = TransitionEnforcer::new(&registry);
        let event = TransitionEvent::new(
            DocumentId::from("real-doc"),
            DocumentType::Task,
            Phase::Active,
            Phase::Completed,
            "claude",
            false,
        );
        let result = enforcer.enforce(&event);

        let mut log = TransitionAuditLog::new();
        log.record(&result);

        let entry = log.get(1).unwrap();
        assert_eq!(entry.check_results.len(), 1);
        assert!(entry.check_results[0].passed);
        assert_eq!(entry.check_results[0].name, "gate");
    }

    // -- BlockedReason tests --

    #[test]
    fn test_blocked_reason_creation() {
        let reason = BlockedReason::new(
            DocumentId::from("task-1"),
            "actor",
            BlockCategory::QualityGate,
            "Lint errors exceed threshold",
        );

        assert_eq!(reason.category, BlockCategory::QualityGate);
        assert!(!reason.is_resolved());
        assert!(reason.blocking_documents.is_empty());
    }

    #[test]
    fn test_blocked_reason_dependency() {
        let reason = BlockedReason::dependency(
            DocumentId::from("task-2"),
            "actor",
            "Waiting on task-1",
            vec![DocumentId::from("task-1")],
        );

        assert_eq!(reason.category, BlockCategory::Dependency);
        assert_eq!(reason.blocking_documents.len(), 1);
    }

    #[test]
    fn test_blocked_reason_resolve() {
        let mut reason = BlockedReason::new(
            DocumentId::from("task-1"),
            "actor",
            BlockCategory::ManualHold,
            "On hold",
        );

        assert!(!reason.is_resolved());
        reason.resolve();
        assert!(reason.is_resolved());
    }

    #[test]
    fn test_blocked_reason_display() {
        let reason = BlockedReason::new(
            DocumentId::from("task-1"),
            "actor",
            BlockCategory::External,
            "Waiting on API",
        );
        let display = reason.to_string();
        assert!(display.contains("task-1"));
        assert!(display.contains("external"));
        assert!(display.contains("Waiting on API"));
    }

    #[test]
    fn test_block_category_display() {
        assert_eq!(BlockCategory::Dependency.to_string(), "dependency");
        assert_eq!(BlockCategory::QualityGate.to_string(), "quality_gate");
        assert_eq!(BlockCategory::ManualHold.to_string(), "manual_hold");
        assert_eq!(
            BlockCategory::MissingPrerequisite.to_string(),
            "missing_prerequisite"
        );
        assert_eq!(BlockCategory::External.to_string(), "external");
    }

    // -- BlockedReasonTracker tests --

    #[test]
    fn test_tracker_empty() {
        let tracker = BlockedReasonTracker::new();
        assert!(!tracker.is_blocked(&DocumentId::from("any")));
        assert_eq!(tracker.total_count(), 0);
    }

    #[test]
    fn test_tracker_record_and_query() {
        let mut tracker = BlockedReasonTracker::new();
        let doc_id = DocumentId::from("task-1");

        tracker.record(BlockedReason::new(
            doc_id.clone(),
            "actor",
            BlockCategory::QualityGate,
            "Tests failing",
        ));

        assert!(tracker.is_blocked(&doc_id));
        assert_eq!(tracker.active_blocks(&doc_id).len(), 1);
        assert_eq!(tracker.all_blocks(&doc_id).len(), 1);
    }

    #[test]
    fn test_tracker_resolve_all() {
        let mut tracker = BlockedReasonTracker::new();
        let doc_id = DocumentId::from("task-1");

        tracker.record(BlockedReason::new(
            doc_id.clone(),
            "actor",
            BlockCategory::QualityGate,
            "Tests failing",
        ));
        tracker.record(BlockedReason::new(
            doc_id.clone(),
            "actor",
            BlockCategory::ManualHold,
            "On hold",
        ));

        assert_eq!(tracker.active_blocks(&doc_id).len(), 2);
        tracker.resolve_all(&doc_id);
        assert!(!tracker.is_blocked(&doc_id));
        assert_eq!(tracker.active_blocks(&doc_id).len(), 0);
        assert_eq!(tracker.all_blocks(&doc_id).len(), 2); // history preserved
    }

    #[test]
    fn test_tracker_resolve_by_category() {
        let mut tracker = BlockedReasonTracker::new();
        let doc_id = DocumentId::from("task-1");

        tracker.record(BlockedReason::new(
            doc_id.clone(),
            "actor",
            BlockCategory::QualityGate,
            "Tests failing",
        ));
        tracker.record(BlockedReason::new(
            doc_id.clone(),
            "actor",
            BlockCategory::ManualHold,
            "On hold",
        ));

        tracker.resolve_by_category(&doc_id, BlockCategory::QualityGate);

        assert!(tracker.is_blocked(&doc_id)); // still blocked by manual hold
        assert_eq!(tracker.active_blocks(&doc_id).len(), 1);
        assert_eq!(
            tracker.active_blocks(&doc_id)[0].category,
            BlockCategory::ManualHold
        );
    }

    #[test]
    fn test_tracker_multiple_documents() {
        let mut tracker = BlockedReasonTracker::new();
        let doc_a = DocumentId::from("task-a");
        let doc_b = DocumentId::from("task-b");

        tracker.record(BlockedReason::new(
            doc_a.clone(),
            "actor",
            BlockCategory::Dependency,
            "waiting",
        ));
        tracker.record(BlockedReason::new(
            doc_b.clone(),
            "actor",
            BlockCategory::External,
            "API down",
        ));

        assert!(tracker.is_blocked(&doc_a));
        assert!(tracker.is_blocked(&doc_b));
        assert_eq!(tracker.total_count(), 2);

        tracker.resolve_all(&doc_a);
        assert!(!tracker.is_blocked(&doc_a));
        assert!(tracker.is_blocked(&doc_b));
    }
}
