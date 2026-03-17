//! Bootstrap module for repo-aware initialization.
//!
//! Provides scanning, detection, and orchestration for bootstrapping
//! Ultra-Metis in a new or existing repository. All modules operate
//! on file path lists -- no filesystem I/O in the domain layer.
//!
//! Components:
//! - [`repo_scanner`]: Detects languages, package managers, and build tools
//! - [`monorepo_detector`]: Detects monorepo patterns and discovers packages
//! - [`tool_detector`]: Detects linters, formatters, and test runners
//! - [`init_flow`]: Orchestrates the full bootstrap flow

pub mod repo_scanner;
pub mod monorepo_detector;
pub mod tool_detector;
pub mod init_flow;
