//! Architecture catalog module.
//!
//! Provides the built-in catalog entries, a query engine for filtering entries
//! by language and project type, a selection flow for choosing and persisting
//! a reference architecture, and custom catalog entry loading.

pub mod brownfield_evaluator;
pub mod builtin_entries;
pub mod custom_loader;
pub mod query_engine;
pub mod remote_fetcher;
pub mod selection_flow;
