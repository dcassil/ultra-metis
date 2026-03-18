//! File persistence layer for Ultra-Metis documents.
//!
//! Manages a `.ultra-metis/` project directory containing:
//! - `config.toml` with project prefix and short code counter
//! - Markdown+frontmatter document files organized by type

pub mod code_index;
pub mod config;
pub mod error;
pub mod store;

pub use code_index::{CodeIndex, CodeIndexer, CodeSymbol, SymbolKind};
pub use config::ProjectConfig;
pub use error::StoreError;
pub use store::DocumentStore;
