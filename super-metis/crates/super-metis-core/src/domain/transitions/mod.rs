//! Transition Hook System and Enforcement.
//!
//! This module provides the infrastructure for pre-transition and post-transition
//! hooks, a hook registry for managing registered hooks, a transition enforcer
//! that validates hooks before allowing phase transitions, an audit log for
//! recording all transition history, and traceability queries for walking
//! document hierarchies.

pub mod hooks;
pub mod registry;
pub mod enforcer;
pub mod audit;
pub mod traceability;
