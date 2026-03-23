//! Gates, Escalation, and Autonomy Model.
//!
//! This module defines the governance framework for Cadre:
//!
//! - **Gates** (`gates`): Typed checkpoints that control workflow progression,
//!   with configurable evidence requirements and failure behaviors.
//! - **Escalation** (`escalation`): Triggers and detection logic for when the
//!   system should pause and involve a human.
//! - **Autonomy** (`autonomy`): Three operating modes (tight, mixed, autonomous)
//!   that control gate strictness, escalation sensitivity, and system permissions.

pub mod autonomy;
pub mod escalation;
pub mod gates;
