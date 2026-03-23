//! Remediation and Investigation Loop Engine.
//!
//! When quality gates detect regression, this module provides the automated
//! response: triggering investigations, tracking remediation actions, verifying
//! resolution, and detecting recurring issues.

pub mod recurrence;
pub mod trigger;
pub mod types;
pub mod verification;
