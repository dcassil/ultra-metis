//! Brownfield architecture evaluation module.
//!
//! Analyzes existing repositories to detect their architecture patterns,
//! score quality, match against catalog entries, and produce a
//! [`ReferenceArchitecture`] document.
//!
//! Components:
//! - [`structure_analyzer`]: Detects folder patterns, layers, naming conventions
//! - [`pattern_matcher`]: Scores existing structure against catalog entries
//! - [`evaluator`]: Orchestrates analysis + matching into evaluation outcomes

pub mod evaluator;
pub mod pattern_matcher;
pub mod structure_analyzer;
