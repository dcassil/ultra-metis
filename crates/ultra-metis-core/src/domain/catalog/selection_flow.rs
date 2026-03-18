//! Greenfield architecture selection flow.
//!
//! Orchestrates the process of selecting a catalog entry and persisting
//! it as a [`ReferenceArchitecture`] document. The flow:
//!
//! 1. Query the catalog for matching entries (by language/project_type)
//! 2. Present options with tradeoffs
//! 3. User selects one
//! 4. Optionally tailor aspects (layer overrides, extra boundaries, etc.)
//! 5. Persist as a ReferenceArchitecture

use crate::domain::documents::architecture_catalog_entry::ArchitectureCatalogEntry;
use crate::domain::documents::content::DocumentContent;
use crate::domain::documents::reference_architecture::{ArchitectureStatus, ReferenceArchitecture};
use crate::domain::documents::traits::DocumentValidationError;
use crate::domain::documents::types::{Phase, Tag};

use super::query_engine::{CatalogQuery, CatalogQueryEngine};

/// A presentable option for the user to choose from during selection.
#[derive(Debug)]
pub struct SelectionOption {
    /// Index for user selection (0-based).
    pub index: usize,
    /// Title of the catalog entry.
    pub title: String,
    /// Language of the entry.
    pub language: String,
    /// Project type of the entry.
    pub project_type: String,
    /// Layers defined in this architecture.
    pub layers: Vec<String>,
    /// Folder structure overview.
    pub folder_layout: Vec<String>,
    /// Key dependency rules.
    pub dependency_rules: Vec<String>,
    /// Anti-patterns to watch for.
    pub anti_patterns: Vec<String>,
}

impl SelectionOption {
    /// Create from a catalog entry.
    pub fn from_entry(index: usize, entry: &ArchitectureCatalogEntry) -> Self {
        Self {
            index,
            title: entry.title().to_string(),
            language: entry.language.clone(),
            project_type: entry.project_type.clone(),
            layers: entry.layers.clone(),
            folder_layout: entry.folder_layout.clone(),
            dependency_rules: entry.dependency_rules.clone(),
            anti_patterns: entry.anti_patterns.clone(),
        }
    }

    /// Format as a human-readable summary for presentation.
    pub fn summary(&self) -> String {
        let mut s = format!(
            "[{}] {} ({}/{})\n",
            self.index, self.title, self.language, self.project_type
        );
        if !self.layers.is_empty() {
            s.push_str(&format!("    Layers: {}\n", self.layers.join(" -> ")));
        }
        if !self.dependency_rules.is_empty() {
            s.push_str("    Key rules:\n");
            for rule in self.dependency_rules.iter().take(3) {
                s.push_str(&format!("      - {}\n", rule));
            }
        }
        s
    }
}

/// Tailoring options that a user can apply to a selected architecture.
#[derive(Debug, Clone, Default)]
pub struct TailoringOptions {
    /// Override or add layers.
    pub layer_overrides: Vec<String>,
    /// Additional module boundaries beyond the catalog entry defaults.
    pub additional_boundaries: Vec<String>,
    /// Extra dependency rules.
    pub extra_dependency_rules: Vec<String>,
    /// Exceptions to tolerate (e.g., legacy modules that break the pattern).
    pub tolerated_exceptions: Vec<String>,
}

/// The result of a completed selection flow.
#[derive(Debug)]
pub struct SelectionResult {
    /// The created ReferenceArchitecture.
    pub reference_architecture: ReferenceArchitecture,
    /// The short code of the source catalog entry.
    pub source_catalog_short_code: String,
}

/// Orchestrates the greenfield architecture selection flow.
pub struct SelectionFlow<'a> {
    engine: &'a CatalogQueryEngine,
}

impl<'a> SelectionFlow<'a> {
    /// Create a new selection flow backed by the given query engine.
    pub fn new(engine: &'a CatalogQueryEngine) -> Self {
        Self { engine }
    }

    /// Step 1: Discover available options for a language/project_type.
    ///
    /// Returns presentable options the user can choose from.
    pub fn discover_options(
        &self,
        language: &str,
        project_type: Option<&str>,
    ) -> Vec<SelectionOption> {
        let mut query = CatalogQuery::new().with_language(language);
        if let Some(pt) = project_type {
            query = query.with_project_type(pt);
        }

        let matches = self.engine.query(&query);
        matches
            .iter()
            .enumerate()
            .map(|(i, m)| SelectionOption::from_entry(i, m.entry))
            .collect()
    }

    /// Step 2: Select an entry by index from the options list.
    ///
    /// Returns the matching catalog entry, or an error if the index is invalid.
    pub fn select_by_index(
        &self,
        options: &[SelectionOption],
        index: usize,
    ) -> Result<&ArchitectureCatalogEntry, DocumentValidationError> {
        if index >= options.len() {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Selection index {} is out of range (0..{})",
                index,
                options.len()
            )));
        }

        let option = &options[index];
        self.engine
            .find_exact(&option.language, &option.project_type)
            .ok_or_else(|| {
                DocumentValidationError::InvalidContent(format!(
                    "No catalog entry found for {}/{}",
                    option.language, option.project_type
                ))
            })
    }

    /// Step 3: Create a ReferenceArchitecture from a selected catalog entry.
    ///
    /// Applies optional tailoring and produces a persisted reference architecture
    /// linked back to the source catalog entry.
    pub fn create_reference_architecture(
        &self,
        entry: &ArchitectureCatalogEntry,
        short_code: String,
        tailoring: Option<TailoringOptions>,
    ) -> Result<SelectionResult, DocumentValidationError> {
        let tailoring = tailoring.unwrap_or_default();

        let title = format!(
            "Reference Architecture: {} ({})",
            entry.project_type, entry.language
        );

        // Build content from the catalog entry's details
        let content_body = format!(
            "# {}\n\n\
             ## Source\n\n\
             Selected from catalog entry: {}\n\n\
             ## Layers\n\n{}\n\n\
             ## Folder Layout\n\n{}\n\n\
             ## Dependency Rules\n\n{}\n\n\
             ## Naming Conventions\n\n{}\n\n\
             ## Anti-Patterns to Avoid\n\n{}",
            title,
            entry.title(),
            entry
                .layers
                .iter()
                .map(|l| format!("- {}", l))
                .collect::<Vec<_>>()
                .join("\n"),
            entry
                .folder_layout
                .iter()
                .map(|f| format!("- `{}`", f))
                .collect::<Vec<_>>()
                .join("\n"),
            entry
                .dependency_rules
                .iter()
                .map(|r| format!("- {}", r))
                .collect::<Vec<_>>()
                .join("\n"),
            entry
                .naming_conventions
                .iter()
                .map(|n| format!("- {}", n))
                .collect::<Vec<_>>()
                .join("\n"),
            entry
                .anti_patterns
                .iter()
                .map(|a| format!("- {}", a))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        let source_short_code = entry.metadata().short_code.clone();

        let ra = ReferenceArchitecture::from_parts(
            title,
            crate::domain::documents::metadata::DocumentMetadata::new(short_code),
            DocumentContent::new(&content_body),
            vec![
                Tag::Label("reference_architecture".to_string()),
                Tag::Phase(Phase::Draft),
            ],
            false,
            Some(source_short_code.clone()),
            false,
            ArchitectureStatus::Draft,
            tailoring.layer_overrides,
            tailoring.additional_boundaries,
            tailoring.extra_dependency_rules,
            None, // rules_config_ref - populated later by rules seeding
            None, // analysis_baseline_ref - populated later
            tailoring.tolerated_exceptions,
        );

        Ok(SelectionResult {
            reference_architecture: ra,
            source_catalog_short_code: source_short_code,
        })
    }

    /// Convenience: run the full flow from language + project_type to ReferenceArchitecture.
    ///
    /// For programmatic use where the selection is already known.
    pub fn select_and_persist(
        &self,
        language: &str,
        project_type: &str,
        short_code: String,
        tailoring: Option<TailoringOptions>,
    ) -> Result<SelectionResult, DocumentValidationError> {
        let entry = self
            .engine
            .find_exact(language, project_type)
            .ok_or_else(|| {
                DocumentValidationError::InvalidContent(format!(
                    "No catalog entry found for {}/{}",
                    language, project_type
                ))
            })?;

        self.create_reference_architecture(entry, short_code, tailoring)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_engine() -> CatalogQueryEngine {
        CatalogQueryEngine::with_builtins()
    }

    #[test]
    fn test_discover_all_javascript_options() {
        let engine = setup_engine();
        let flow = SelectionFlow::new(&engine);
        let options = flow.discover_options("javascript", None);
        assert_eq!(options.len(), 5);
    }

    #[test]
    fn test_discover_specific_project_type() {
        let engine = setup_engine();
        let flow = SelectionFlow::new(&engine);
        let options = flow.discover_options("javascript", Some("server"));
        assert_eq!(options.len(), 1);
        assert_eq!(options[0].project_type, "server");
    }

    #[test]
    fn test_discover_unknown_language() {
        let engine = setup_engine();
        let flow = SelectionFlow::new(&engine);
        let options = flow.discover_options("go", None);
        assert!(options.is_empty());
    }

    #[test]
    fn test_selection_option_summary() {
        let engine = setup_engine();
        let flow = SelectionFlow::new(&engine);
        let options = flow.discover_options("javascript", Some("server"));
        let summary = options[0].summary();
        assert!(summary.contains("server"));
        assert!(summary.contains("javascript"));
    }

    #[test]
    fn test_select_by_index() {
        let engine = setup_engine();
        let flow = SelectionFlow::new(&engine);
        let options = flow.discover_options("javascript", None);

        let entry = flow.select_by_index(&options, 0).unwrap();
        assert_eq!(entry.language, "javascript");
    }

    #[test]
    fn test_select_by_invalid_index() {
        let engine = setup_engine();
        let flow = SelectionFlow::new(&engine);
        let options = flow.discover_options("javascript", Some("server"));

        let result = flow.select_by_index(&options, 99);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_reference_architecture_from_selection() {
        let engine = setup_engine();
        let flow = SelectionFlow::new(&engine);

        let result = flow
            .select_and_persist("javascript", "server", "RA-TEST-001".to_string(), None)
            .unwrap();

        let ra = &result.reference_architecture;
        assert!(ra.title().contains("server"));
        assert_eq!(
            ra.source_catalog_ref.as_deref(),
            Some("BUILTIN-AC-JS-SERVER")
        );
        assert!(!ra.is_derived);
        assert_eq!(ra.status, ArchitectureStatus::Draft);
        assert!(ra.is_catalog_linked());
    }

    #[test]
    fn test_create_with_tailoring() {
        let engine = setup_engine();
        let flow = SelectionFlow::new(&engine);

        let tailoring = TailoringOptions {
            layer_overrides: vec!["custom-layer".to_string()],
            additional_boundaries: vec!["api-gateway".to_string()],
            extra_dependency_rules: vec!["no direct DB from routes".to_string()],
            tolerated_exceptions: vec!["legacy-module".to_string()],
        };

        let result = flow
            .select_and_persist(
                "javascript",
                "react-app",
                "RA-TEST-002".to_string(),
                Some(tailoring),
            )
            .unwrap();

        let ra = &result.reference_architecture;
        assert_eq!(ra.layer_overrides, vec!["custom-layer"]);
        assert_eq!(ra.additional_boundaries, vec!["api-gateway"]);
        assert_eq!(ra.extra_dependency_rules, vec!["no direct DB from routes"]);
        assert_eq!(ra.tolerated_exceptions, vec!["legacy-module"]);
    }

    #[test]
    fn test_select_and_persist_unknown() {
        let engine = setup_engine();
        let flow = SelectionFlow::new(&engine);

        let result = flow.select_and_persist("python", "django", "RA-TEST-003".to_string(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_reference_architecture_content_includes_entry_details() {
        let engine = setup_engine();
        let flow = SelectionFlow::new(&engine);

        let result = flow
            .select_and_persist("javascript", "cli-tool", "RA-TEST-004".to_string(), None)
            .unwrap();

        let body = &result.reference_architecture.content().body;
        assert!(body.contains("commands"));
        assert!(body.contains("core"));
        assert!(body.contains("Dependency Rules"));
        assert!(body.contains("Anti-Patterns"));
    }

    #[test]
    fn test_reference_architecture_serialization_roundtrip() {
        let engine = setup_engine();
        let flow = SelectionFlow::new(&engine);

        let result = flow
            .select_and_persist("javascript", "node-util", "RA-TEST-005".to_string(), None)
            .unwrap();

        let serialized = result.reference_architecture.to_content().unwrap();
        let loaded = ReferenceArchitecture::from_content(&serialized).unwrap();

        assert_eq!(loaded.title(), result.reference_architecture.title());
        assert_eq!(
            loaded.source_catalog_ref,
            result.reference_architecture.source_catalog_ref
        );
        assert_eq!(loaded.is_derived, result.reference_architecture.is_derived);
        assert_eq!(loaded.status, result.reference_architecture.status);
    }

    #[test]
    fn test_full_interactive_flow() {
        let engine = setup_engine();
        let flow = SelectionFlow::new(&engine);

        // Step 1: Discover
        let options = flow.discover_options("javascript", None);
        assert!(!options.is_empty());

        // Step 2: Select (pick the server option)
        let server_idx = options
            .iter()
            .position(|o| o.project_type == "server")
            .unwrap();
        let entry = flow.select_by_index(&options, server_idx).unwrap();
        assert_eq!(entry.project_type, "server");

        // Step 3: Persist with tailoring
        let tailoring = TailoringOptions {
            tolerated_exceptions: vec!["legacy-auth-module".to_string()],
            ..Default::default()
        };
        let result = flow
            .create_reference_architecture(entry, "RA-FULL-001".to_string(), Some(tailoring))
            .unwrap();

        assert!(result.reference_architecture.is_catalog_linked());
        assert_eq!(
            result.reference_architecture.tolerated_exceptions,
            vec!["legacy-auth-module"]
        );
    }
}
