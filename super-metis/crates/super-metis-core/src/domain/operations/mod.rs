//! Cognitive Operation Kernel and Reusable Loops.
//!
//! All higher-level workflows are composed from a fixed set of 12 reusable
//! cognitive operations.  Operations compose into loops, and workflows are
//! predefined compositions of loops with entry/exit conditions.
//!
//! This module defines the **type system** and **composition semantics** --
//! not an execution engine.

pub mod operation;
pub mod loops;
pub mod workflow;
pub mod templates;
