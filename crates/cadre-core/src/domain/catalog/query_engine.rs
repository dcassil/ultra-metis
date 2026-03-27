//! Catalog query engine for filtering architecture catalog entries.
//!
//! Provides a [`CatalogQuery`] builder and a [`CatalogQueryEngine`] that searches
//! across both built-in and custom catalog entries.

use crate::domain::documents::architecture_catalog_entry::ArchitectureCatalogEntry;
use crate::domain::documents::types::Phase;

/// A query for filtering catalog entries.
#[derive(Debug, Clone, Default)]
pub struct CatalogQuery {
    /// Filter by language (exact match, case-insensitive).
    pub language: Option<String>,
    /// Filter by project type (exact match, case-insensitive).
    pub project_type: Option<String>,
    /// Only return entries in this phase. Defaults to Published if not set.
    pub phase: Option<Phase>,
}

impl CatalogQuery {
    /// Create a new empty query (matches everything).
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by language.
    pub fn with_language(mut self, language: &str) -> Self {
        self.language = Some(language.to_lowercase());
        self
    }

    /// Filter by project type.
    pub fn with_project_type(mut self, project_type: &str) -> Self {
        self.project_type = Some(project_type.to_lowercase());
        self
    }

    /// Filter by phase.
    pub fn with_phase(mut self, phase: Phase) -> Self {
        self.phase = Some(phase);
        self
    }
}

/// Result of a catalog query, containing matched entries with relevance info.
#[derive(Debug)]
pub struct CatalogMatch<'a> {
    /// The matched catalog entry.
    pub entry: &'a ArchitectureCatalogEntry,
    /// Short description of why this matched.
    pub match_reason: String,
}

/// Engine for querying architecture catalog entries.
///
/// Holds references to all available entries (built-in + custom) and
/// filters them based on a [`CatalogQuery`].
pub struct CatalogQueryEngine {
    entries: Vec<ArchitectureCatalogEntry>,
}

impl CatalogQueryEngine {
    /// Create a new engine with the given entries.
    pub fn new(entries: Vec<ArchitectureCatalogEntry>) -> Self {
        Self { entries }
    }

    /// Create a new engine pre-loaded with built-in entries.
    ///
    /// In production this returns an empty engine — use [`Self::with_remote`] or
    /// [`super::custom_loader::build_engine_with_custom`] to load entries from
    /// the external catalog repository.
    pub fn with_builtins() -> Self {
        Self::new(super::builtin_entries::builtin_entries())
    }

    /// Create a new engine with built-in entries plus additional custom entries.
    pub fn with_builtins_and_custom(custom: Vec<ArchitectureCatalogEntry>) -> Self {
        let mut entries = super::builtin_entries::builtin_entries();
        entries.extend(custom);
        Self::new(entries)
    }

    /// Create a new engine loaded from the remote catalog repository.
    pub async fn with_remote() -> Self {
        let fetcher = super::remote_fetcher::RemoteCatalogFetcher::with_defaults();
        let entries = fetcher.fetch().await.unwrap_or_default();
        Self::new(entries)
    }

    /// Create a new engine with remote entries plus additional custom entries.
    pub async fn with_remote_and_custom(custom: Vec<ArchitectureCatalogEntry>) -> Self {
        let fetcher = super::remote_fetcher::RemoteCatalogFetcher::with_defaults();
        let mut entries = fetcher.fetch().await.unwrap_or_default();
        entries.extend(custom);
        Self::new(entries)
    }

    /// Add entries to the engine.
    pub fn add_entries(&mut self, entries: Vec<ArchitectureCatalogEntry>) {
        self.entries.extend(entries);
    }

    /// Return all entries held by the engine.
    pub fn all_entries(&self) -> &[ArchitectureCatalogEntry] {
        &self.entries
    }

    /// Query the catalog and return matching entries.
    pub fn query(&self, query: &CatalogQuery) -> Vec<CatalogMatch<'_>> {
        self.entries
            .iter()
            .filter(|entry| {
                // Phase filter: default to Published
                let required_phase = query.phase.unwrap_or(Phase::Published);
                if let Ok(entry_phase) = entry.phase() {
                    if entry_phase != required_phase {
                        return false;
                    }
                } else {
                    return false;
                }

                // Language filter
                if let Some(ref lang) = query.language {
                    if entry.language.to_lowercase() != *lang {
                        return false;
                    }
                }

                // Project type filter
                if let Some(ref pt) = query.project_type {
                    if entry.project_type.to_lowercase() != *pt {
                        return false;
                    }
                }

                true
            })
            .map(|entry| {
                let reason = format!(
                    "Matches language='{}', project_type='{}'",
                    entry.language, entry.project_type
                );
                CatalogMatch {
                    entry,
                    match_reason: reason,
                }
            })
            .collect()
    }

    /// Find a single entry by language and project type.
    /// Returns None if no match or multiple matches.
    pub fn find_exact(
        &self,
        language: &str,
        project_type: &str,
    ) -> Option<&ArchitectureCatalogEntry> {
        let query = CatalogQuery::new()
            .with_language(language)
            .with_project_type(project_type);
        let matches = self.query(&query);
        if matches.len() == 1 {
            Some(matches[0].entry)
        } else {
            None
        }
    }

    /// List all distinct languages in the catalog.
    pub fn languages(&self) -> Vec<String> {
        let mut langs: Vec<String> = self.entries.iter().map(|e| e.language.clone()).collect();
        langs.sort();
        langs.dedup();
        langs
    }

    /// List all distinct project types for a given language.
    pub fn project_types_for_language(&self, language: &str) -> Vec<String> {
        let lang_lower = language.to_lowercase();
        let mut types: Vec<String> = self
            .entries
            .iter()
            .filter(|e| e.language.to_lowercase() == lang_lower)
            .map(|e| e.project_type.clone())
            .collect();
        types.sort();
        types.dedup();
        types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::catalog::builtin_entries::test_builtin_entries;

    fn test_engine() -> CatalogQueryEngine {
        CatalogQueryEngine::new(test_builtin_entries())
    }

    #[test]
    fn test_engine_with_test_entries() {
        let engine = test_engine();
        assert_eq!(engine.all_entries().len(), 5);
    }

    #[test]
    fn test_production_builtins_empty() {
        let engine = CatalogQueryEngine::with_builtins();
        assert_eq!(engine.all_entries().len(), 0);
    }

    #[test]
    fn test_query_all_javascript() {
        let engine = test_engine();
        let query = CatalogQuery::new().with_language("javascript");
        let results = engine.query(&query);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_query_by_project_type() {
        let engine = test_engine();
        let query = CatalogQuery::new()
            .with_language("javascript")
            .with_project_type("server");
        let results = engine.query(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.project_type, "server");
    }

    #[test]
    fn test_query_case_insensitive() {
        let engine = test_engine();
        let query = CatalogQuery::new()
            .with_language("JavaScript")
            .with_project_type("React-App");
        let results = engine.query(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.project_type, "react-app");
    }

    #[test]
    fn test_query_no_match() {
        let engine = test_engine();
        let query = CatalogQuery::new().with_language("rust");
        let results = engine.query(&query);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_find_exact() {
        let engine = test_engine();
        let entry = engine.find_exact("javascript", "cli-tool");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().project_type, "cli-tool");
    }

    #[test]
    fn test_find_exact_no_match() {
        let engine = test_engine();
        let entry = engine.find_exact("python", "django");
        assert!(entry.is_none());
    }

    #[test]
    fn test_languages() {
        let engine = test_engine();
        let langs = engine.languages();
        assert_eq!(langs, vec!["javascript"]);
    }

    #[test]
    fn test_project_types_for_language() {
        let engine = test_engine();
        let types = engine.project_types_for_language("javascript");
        assert_eq!(types.len(), 5);
        assert!(types.contains(&"server".to_string()));
        assert!(types.contains(&"react-app".to_string()));
        assert!(types.contains(&"component-lib".to_string()));
        assert!(types.contains(&"cli-tool".to_string()));
        assert!(types.contains(&"node-util".to_string()));
    }

    #[test]
    fn test_project_types_for_unknown_language() {
        let engine = test_engine();
        let types = engine.project_types_for_language("rust");
        assert!(types.is_empty());
    }

    #[test]
    fn test_query_phase_filter() {
        let engine = test_engine();
        let query = CatalogQuery::new()
            .with_language("javascript")
            .with_phase(Phase::Draft);
        let results = engine.query(&query);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_empty_query_returns_all_published() {
        let engine = test_engine();
        let query = CatalogQuery::new();
        let results = engine.query(&query);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_with_custom_entries() {
        use crate::domain::documents::content::DocumentContent;
        use crate::domain::documents::metadata::DocumentMetadata;
        use crate::domain::documents::types::Tag;

        let custom = ArchitectureCatalogEntry::from_parts(
            "Custom Rust CLI".to_string(),
            DocumentMetadata::new("CUSTOM-AC-001".to_string()),
            DocumentContent::new("# Custom Rust CLI"),
            vec![Tag::Phase(Phase::Published)],
            false,
            "rust".to_string(),
            "cli".to_string(),
            vec!["src/".to_string()],
            vec!["core".to_string()],
            vec![],
            vec![],
            vec!["snake_case".to_string()],
            vec![],
            vec![],
            vec![],
        );

        let mut entries = test_builtin_entries();
        entries.push(custom);
        let engine = CatalogQueryEngine::new(entries);
        assert_eq!(engine.all_entries().len(), 6);

        let query = CatalogQuery::new().with_language("rust");
        let results = engine.query(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.project_type, "cli");
    }
}
