//! Rule protection enforcement and edit guards.
//!
//! This module provides the protection enforcement layer for RulesConfig documents.
//! Protected rules cannot be edited directly — changes must go through a
//! RuleChangeProposal workflow. The `EditGuard` checks protection status and
//! either permits or rejects edit operations with actionable error messages.

pub mod protection;
pub mod query;
pub mod seeding;
