//! Rule protection enforcement and edit guards.
//!
//! The `EditGuard` is the central enforcement mechanism that prevents direct
//! modification of protected RulesConfig documents. When an edit is attempted,
//! the guard checks protection level and either permits or rejects the operation.

use crate::domain::documents::rule_change_proposal::{RuleChangeProposal, RuleChangeStatus};
use crate::domain::documents::rules_config::{ProtectionLevel, RulesConfig};
use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// EditGuardError
// ---------------------------------------------------------------------------

/// Errors returned by the edit guard when an operation is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditGuardError {
    /// The rule is protected and requires a RuleChangeProposal.
    ProtectedRule {
        rule_short_code: String,
        rule_title: String,
    },
    /// A RuleChangeProposal exists but is not yet approved.
    ProposalNotApproved {
        rule_short_code: String,
        proposal_short_code: String,
        current_status: RuleChangeStatus,
    },
    /// A force override was used — the edit is permitted but audited.
    ForceOverrideUsed {
        rule_short_code: String,
        reason: String,
    },
}

impl fmt::Display for EditGuardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProtectedRule {
                rule_short_code,
                rule_title,
            } => write!(
                f,
                "Cannot edit protected rule '{rule_title}' ({rule_short_code}). Create a RuleChangeProposal to modify this rule."
            ),
            Self::ProposalNotApproved {
                rule_short_code,
                proposal_short_code,
                current_status,
            } => write!(
                f,
                "Cannot apply changes to rule '{rule_short_code}': proposal '{proposal_short_code}' has status '{current_status}' (must be 'approved')"
            ),
            Self::ForceOverrideUsed {
                rule_short_code,
                reason,
            } => write!(
                f,
                "Force override applied to protected rule '{rule_short_code}': {reason}"
            ),
        }
    }
}

impl std::error::Error for EditGuardError {}

// ---------------------------------------------------------------------------
// EditDecision
// ---------------------------------------------------------------------------

/// The result of an edit guard check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditDecision {
    /// The edit is permitted — rule is not protected or has an approved proposal.
    Permitted,
    /// The edit is permitted via an approved RuleChangeProposal.
    PermittedViaProposal { proposal_short_code: String },
    /// The edit is permitted via force override (audited).
    PermittedViaForceOverride { reason: String },
    /// The edit is rejected.
    Rejected(EditGuardError),
}

impl EditDecision {
    pub fn is_permitted(&self) -> bool {
        !matches!(self, Self::Rejected(_))
    }

    pub fn is_force_override(&self) -> bool {
        matches!(self, Self::PermittedViaForceOverride { .. })
    }
}

// ---------------------------------------------------------------------------
// ForceOverride
// ---------------------------------------------------------------------------

/// A force override request with mandatory reason and audit fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceOverride {
    /// Who is requesting the override.
    pub requested_by: String,
    /// Why the override is necessary.
    pub reason: String,
}

// ---------------------------------------------------------------------------
// EditGuard
// ---------------------------------------------------------------------------

/// The edit guard checks whether an edit to a RulesConfig is permitted.
///
/// Decision logic:
/// 1. Standard protection -> always permitted
/// 2. Protected + approved proposal -> permitted via proposal
/// 3. Protected + force override -> permitted (audited)
/// 4. Protected + no approved proposal -> rejected
pub struct EditGuard;

impl EditGuard {
    /// Check whether an edit to the given RulesConfig is permitted.
    ///
    /// # Arguments
    /// - `rule`: The RulesConfig document being edited.
    /// - `approved_proposal`: An optional RuleChangeProposal that authorizes the edit.
    /// - `force_override`: An optional force override request (for emergencies).
    pub fn check_edit(
        rule: &RulesConfig,
        approved_proposal: Option<&RuleChangeProposal>,
        force_override: Option<&ForceOverride>,
    ) -> EditDecision {
        // Standard rules can always be edited directly.
        if rule.protection_level == ProtectionLevel::Standard {
            return EditDecision::Permitted;
        }

        // Protected rule — check for approved proposal.
        if let Some(proposal) = approved_proposal {
            if proposal.proposal_status == RuleChangeStatus::Approved {
                return EditDecision::PermittedViaProposal {
                    proposal_short_code: proposal.metadata().short_code.clone(),
                };
            }
            // Proposal exists but is not approved.
            return EditDecision::Rejected(EditGuardError::ProposalNotApproved {
                rule_short_code: rule.metadata().short_code.clone(),
                proposal_short_code: proposal.metadata().short_code.clone(),
                current_status: proposal.proposal_status,
            });
        }

        // Check for force override.
        if let Some(override_req) = force_override {
            return EditDecision::PermittedViaForceOverride {
                reason: override_req.reason.clone(),
            };
        }

        // No proposal, no override — reject.
        EditDecision::Rejected(EditGuardError::ProtectedRule {
            rule_short_code: rule.metadata().short_code.clone(),
            rule_title: rule.title().to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// AuditEntry
// ---------------------------------------------------------------------------

/// An audit entry recorded when a force override or proposal-based edit occurs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditAuditEntry {
    /// When the edit occurred.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The short code of the rule that was edited.
    pub rule_short_code: String,
    /// How the edit was authorized.
    pub authorization: EditAuthorization,
    /// Description of what was changed.
    pub change_description: String,
}

/// How an edit to a protected rule was authorized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditAuthorization {
    /// Via an approved RuleChangeProposal.
    Proposal { proposal_short_code: String },
    /// Via force override (emergency).
    ForceOverride {
        requested_by: String,
        reason: String,
    },
}

impl EditAuditEntry {
    /// Create an audit entry for a proposal-based edit.
    pub fn from_proposal(
        rule_short_code: &str,
        proposal_short_code: &str,
        change_description: &str,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            rule_short_code: rule_short_code.to_string(),
            authorization: EditAuthorization::Proposal {
                proposal_short_code: proposal_short_code.to_string(),
            },
            change_description: change_description.to_string(),
        }
    }

    /// Create an audit entry for a force override.
    pub fn from_force_override(
        rule_short_code: &str,
        requested_by: &str,
        reason: &str,
        change_description: &str,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            rule_short_code: rule_short_code.to_string(),
            authorization: EditAuthorization::ForceOverride {
                requested_by: requested_by.to_string(),
                reason: reason.to_string(),
            },
            change_description: change_description.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::documents::rule_change_proposal::RuleChangeType;
    use crate::domain::documents::rules_config::RuleScope;
    use crate::domain::documents::types::{Phase, Tag};

    fn make_standard_rule() -> RulesConfig {
        RulesConfig::new(
            "Standard Rules".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "RC-0001".to_string(),
            ProtectionLevel::Standard,
            RuleScope::Repository,
            None,
        )
        .unwrap()
    }

    fn make_protected_rule() -> RulesConfig {
        RulesConfig::new(
            "Protected Rules".to_string(),
            vec![Tag::Phase(Phase::Published)],
            false,
            "RC-0002".to_string(),
            ProtectionLevel::Protected,
            RuleScope::Repository,
            None,
        )
        .unwrap()
    }

    fn make_approved_proposal() -> RuleChangeProposal {
        RuleChangeProposal::new(
            "Update protected rules".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "RCP-0001".to_string(),
            "RC-0002".to_string(),
            RuleChangeType::Modify,
            RuleChangeStatus::Approved,
            Some("Reviewer".to_string()),
        )
        .unwrap()
    }

    fn make_proposed_proposal() -> RuleChangeProposal {
        RuleChangeProposal::new(
            "Pending proposal".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "RCP-0002".to_string(),
            "RC-0002".to_string(),
            RuleChangeType::Modify,
            RuleChangeStatus::Proposed,
            None,
        )
        .unwrap()
    }

    #[test]
    fn test_standard_rule_always_permitted() {
        let rule = make_standard_rule();
        let decision = EditGuard::check_edit(&rule, None, None);
        assert_eq!(decision, EditDecision::Permitted);
    }

    #[test]
    fn test_protected_rule_rejected_without_proposal() {
        let rule = make_protected_rule();
        let decision = EditGuard::check_edit(&rule, None, None);
        assert!(!decision.is_permitted());
        match decision {
            EditDecision::Rejected(EditGuardError::ProtectedRule {
                rule_short_code, ..
            }) => {
                assert_eq!(rule_short_code, "RC-0002");
            }
            _ => panic!("Expected ProtectedRule rejection"),
        }
    }

    #[test]
    fn test_protected_rule_permitted_with_approved_proposal() {
        let rule = make_protected_rule();
        let proposal = make_approved_proposal();
        let decision = EditGuard::check_edit(&rule, Some(&proposal), None);
        assert!(decision.is_permitted());
        match decision {
            EditDecision::PermittedViaProposal {
                proposal_short_code,
            } => {
                assert_eq!(proposal_short_code, "RCP-0001");
            }
            _ => panic!("Expected PermittedViaProposal"),
        }
    }

    #[test]
    fn test_protected_rule_rejected_with_unapproved_proposal() {
        let rule = make_protected_rule();
        let proposal = make_proposed_proposal();
        let decision = EditGuard::check_edit(&rule, Some(&proposal), None);
        assert!(!decision.is_permitted());
        match decision {
            EditDecision::Rejected(EditGuardError::ProposalNotApproved {
                current_status, ..
            }) => {
                assert_eq!(current_status, RuleChangeStatus::Proposed);
            }
            _ => panic!("Expected ProposalNotApproved rejection"),
        }
    }

    #[test]
    fn test_protected_rule_permitted_with_force_override() {
        let rule = make_protected_rule();
        let override_req = ForceOverride {
            requested_by: "admin".to_string(),
            reason: "Production emergency".to_string(),
        };
        let decision = EditGuard::check_edit(&rule, None, Some(&override_req));
        assert!(decision.is_permitted());
        assert!(decision.is_force_override());
    }

    #[test]
    fn test_approved_proposal_takes_precedence_over_force_override() {
        let rule = make_protected_rule();
        let proposal = make_approved_proposal();
        let override_req = ForceOverride {
            requested_by: "admin".to_string(),
            reason: "Just in case".to_string(),
        };
        // When both are provided, proposal takes precedence
        let decision = EditGuard::check_edit(&rule, Some(&proposal), Some(&override_req));
        assert!(decision.is_permitted());
        assert!(!decision.is_force_override());
    }

    #[test]
    fn test_audit_entry_from_proposal() {
        let entry =
            EditAuditEntry::from_proposal("RC-0002", "RCP-0001", "Updated testing requirements");
        assert_eq!(entry.rule_short_code, "RC-0002");
        assert!(matches!(
            entry.authorization,
            EditAuthorization::Proposal { .. }
        ));
    }

    #[test]
    fn test_audit_entry_from_force_override() {
        let entry = EditAuditEntry::from_force_override(
            "RC-0002",
            "admin",
            "Production emergency",
            "Disabled rate limiting rule",
        );
        assert_eq!(entry.rule_short_code, "RC-0002");
        assert!(matches!(
            entry.authorization,
            EditAuthorization::ForceOverride { .. }
        ));
    }

    #[test]
    fn test_edit_guard_error_display() {
        let err = EditGuardError::ProtectedRule {
            rule_short_code: "RC-0001".to_string(),
            rule_title: "My Rules".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Cannot edit protected rule"));
        assert!(msg.contains("RuleChangeProposal"));
    }
}
