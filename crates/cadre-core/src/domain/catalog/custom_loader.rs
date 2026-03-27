//! Custom catalog entry loader.
//!
//! Loads user-defined [`ArchitectureCatalogEntry`] documents from a
//! `.metis/catalog/` directory, allowing projects to extend the built-in
//! catalog with their own architecture patterns.

use crate::domain::documents::architecture_catalog_entry::ArchitectureCatalogEntry;
use crate::domain::documents::traits::DocumentValidationError;
use std::path::{Path, PathBuf};

/// Default subdirectory under `.metis/` for custom catalog entries.
pub const CATALOG_DIR_NAME: &str = "catalog";

/// Resolves the catalog directory path from a project state path.
pub fn catalog_dir(project_state_path: &Path) -> PathBuf {
    project_state_path.join(CATALOG_DIR_NAME)
}

/// Load all custom catalog entries from the given directory.
///
/// Reads all `.md` files in the directory and attempts to parse each as
/// an `ArchitectureCatalogEntry`. Files that fail to parse are skipped
/// with a warning logged.
///
/// Returns a tuple of (loaded entries, errors for files that failed).
pub async fn load_custom_entries(
    dir: &Path,
) -> (Vec<ArchitectureCatalogEntry>, Vec<CustomLoadError>) {
    let mut entries = Vec::new();
    let mut errors = Vec::new();

    if !dir.exists() || !dir.is_dir() {
        return (entries, errors);
    }

    let read_dir = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) => {
            errors.push(CustomLoadError {
                path: dir.to_path_buf(),
                error: format!("Failed to read directory: {e}"),
            });
            return (entries, errors);
        }
    };

    for dir_entry in read_dir {
        let dir_entry = match dir_entry {
            Ok(de) => de,
            Err(e) => {
                errors.push(CustomLoadError {
                    path: dir.to_path_buf(),
                    error: format!("Failed to read directory entry: {e}"),
                });
                continue;
            }
        };

        let path = dir_entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        match ArchitectureCatalogEntry::from_file(&path).await {
            Ok(entry) => entries.push(entry),
            Err(e) => {
                errors.push(CustomLoadError {
                    path: path.clone(),
                    error: format!("Failed to parse catalog entry: {e}"),
                });
            }
        }
    }

    (entries, errors)
}

/// Error from loading a custom catalog entry file.
#[derive(Debug)]
pub struct CustomLoadError {
    /// Path of the file that failed.
    pub path: PathBuf,
    /// Human-readable error description.
    pub error: String,
}

impl std::fmt::Display for CustomLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.error)
    }
}

/// Build a [`CatalogQueryEngine`](super::query_engine::CatalogQueryEngine) with
/// both built-in entries and any custom entries found in the given project state path.
pub async fn build_engine_with_custom(
    project_state_path: &Path,
) -> Result<
    (
        super::query_engine::CatalogQueryEngine,
        Vec<CustomLoadError>,
    ),
    DocumentValidationError,
> {
    let custom_dir = catalog_dir(project_state_path);
    let (custom_entries, errors) = load_custom_entries(&custom_dir).await;
    let engine = super::query_engine::CatalogQueryEngine::with_builtins_and_custom(custom_entries);
    Ok((engine, errors))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::documents::architecture_catalog_entry::ArchitectureCatalogEntry;
    use crate::domain::documents::content::DocumentContent;
    use crate::domain::documents::metadata::DocumentMetadata;
    use crate::domain::documents::types::{Phase, Tag};
    use tempfile::tempdir;

    fn make_custom_entry(
        title: &str,
        short_code: &str,
        language: &str,
        project_type: &str,
    ) -> ArchitectureCatalogEntry {
        ArchitectureCatalogEntry::from_parts(
            title.to_string(),
            DocumentMetadata::new(short_code.to_string()),
            DocumentContent::new(&format!("# {title}")),
            vec![
                Tag::Label("architecture_catalog_entry".to_string()),
                Tag::Phase(Phase::Published),
            ],
            false,
            language.to_string(),
            project_type.to_string(),
            vec!["src/".to_string()],
            vec!["core".to_string()],
            vec![],
            vec!["no cycles".to_string()],
            vec!["snake_case".to_string()],
            vec!["god module".to_string()],
            vec!["enforce-boundaries".to_string()],
            vec!["lint-clean".to_string()],
        )
    }

    #[test]
    fn test_catalog_dir_path() {
        let project_state_path = Path::new("/project/.metis");
        assert_eq!(
            catalog_dir(project_state_path),
            PathBuf::from("/project/.metis/catalog")
        );
    }

    #[tokio::test]
    async fn test_load_from_nonexistent_dir() {
        let (entries, errors) = load_custom_entries(Path::new("/nonexistent/path")).await;
        assert!(entries.is_empty());
        assert!(errors.is_empty()); // non-existent dir is not an error, just empty
    }

    #[tokio::test]
    async fn test_load_from_empty_dir() {
        let dir = tempdir().unwrap();
        let (entries, errors) = load_custom_entries(dir.path()).await;
        assert!(entries.is_empty());
        assert!(errors.is_empty());
    }

    #[tokio::test]
    async fn test_load_custom_entry_from_file() {
        let dir = tempdir().unwrap();

        let entry = make_custom_entry("Go API Server", "CUSTOM-AC-GO-API", "go", "api-server");
        let file_path = dir.path().join("go-api-server.md");
        entry.to_file(&file_path).await.unwrap();

        let (entries, errors) = load_custom_entries(dir.path()).await;
        assert_eq!(entries.len(), 1);
        assert!(errors.is_empty());
        assert_eq!(entries[0].language, "go");
        assert_eq!(entries[0].project_type, "api-server");
    }

    #[tokio::test]
    async fn test_load_skips_non_md_files() {
        let dir = tempdir().unwrap();

        // Write a .txt file - should be skipped
        std::fs::write(dir.path().join("readme.txt"), "Not a catalog entry").unwrap();

        let entry = make_custom_entry("Python CLI", "CUSTOM-AC-PY-CLI", "python", "cli");
        let file_path = dir.path().join("python-cli.md");
        entry.to_file(&file_path).await.unwrap();

        let (entries, errors) = load_custom_entries(dir.path()).await;
        assert_eq!(entries.len(), 1);
        assert!(errors.is_empty());
    }

    #[tokio::test]
    async fn test_load_reports_parse_errors() {
        let dir = tempdir().unwrap();

        // Write an invalid .md file
        std::fs::write(dir.path().join("bad-entry.md"), "not valid frontmatter").unwrap();

        let (entries, errors) = load_custom_entries(dir.path()).await;
        assert!(entries.is_empty());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error.contains("Failed to parse"));
    }

    #[tokio::test]
    async fn test_load_multiple_entries() {
        let dir = tempdir().unwrap();

        let entry1 = make_custom_entry("Rust CLI", "CUSTOM-AC-RS-CLI", "rust", "cli");
        let entry2 = make_custom_entry("Rust Library", "CUSTOM-AC-RS-LIB", "rust", "library");

        entry1
            .to_file(dir.path().join("rust-cli.md"))
            .await
            .unwrap();
        entry2
            .to_file(dir.path().join("rust-lib.md"))
            .await
            .unwrap();

        let (entries, errors) = load_custom_entries(dir.path()).await;
        assert_eq!(entries.len(), 2);
        assert!(errors.is_empty());
    }

    #[tokio::test]
    async fn test_build_engine_with_custom() {
        let dir = tempdir().unwrap();
        let project_state_dir = dir.path().join(".metis");
        let catalog_path = project_state_dir.join("catalog");
        std::fs::create_dir_all(&catalog_path).unwrap();

        let entry = make_custom_entry("Rust Workspace", "CUSTOM-AC-RS-WS", "rust", "workspace");
        entry
            .to_file(catalog_path.join("rust-workspace.md"))
            .await
            .unwrap();

        let (engine, errors) = build_engine_with_custom(&project_state_dir).await.unwrap();
        assert!(errors.is_empty());
        // 5 built-in + 1 custom
        assert_eq!(engine.all_entries().len(), 6);

        // Can query the custom entry
        let result = engine.find_exact("rust", "workspace");
        assert!(result.is_some());
    }
}
