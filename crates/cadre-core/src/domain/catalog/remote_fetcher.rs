//! Remote catalog fetcher.
//!
//! Clones or pulls the external architecture catalog repository into a local
//! cache directory and loads all [`ArchitectureCatalogEntry`] documents from it.

use crate::domain::documents::architecture_catalog_entry::ArchitectureCatalogEntry;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::process::Command;
use tracing::{debug, warn};

/// Default repository URL for the public architecture catalog.
pub const DEFAULT_REPO_URL: &str =
    "https://github.com/dcassil/cadre-architecture-docs.git";

/// Default subdirectory name within the cache dir.
const CACHE_SUBDIR: &str = "cadre-architecture-docs";

/// Errors that can occur during remote catalog fetching.
#[derive(Debug, Error)]
pub enum FetchError {
    #[error("git is not installed or not on PATH")]
    GitNotFound,

    #[error("git command failed: {0}")]
    GitCommand(String),

    #[error("failed to read cache directory: {0}")]
    CacheRead(String),

    #[error("failed to parse catalog entry {path}: {reason}")]
    ParseEntry { path: String, reason: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Fetches architecture catalog entries from a remote git repository.
///
/// On first use, performs a shallow clone. On subsequent uses, pulls updates.
/// Falls back to cached data or an empty catalog when offline.
pub struct RemoteCatalogFetcher {
    repo_url: String,
    cache_dir: PathBuf,
}

impl RemoteCatalogFetcher {
    /// Create a fetcher with explicit repo URL and cache directory.
    pub fn new(repo_url: &str, cache_dir: PathBuf) -> Self {
        Self {
            repo_url: repo_url.to_string(),
            cache_dir,
        }
    }

    /// Create a fetcher with default settings.
    ///
    /// Uses the public `dcassil/cadre-architecture-docs` repo and caches
    /// under `~/.cadre/catalog-cache/`.
    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_REPO_URL, default_cache_dir())
    }

    /// The full path to the cloned repository within the cache.
    pub fn repo_path(&self) -> PathBuf {
        self.cache_dir.join(CACHE_SUBDIR)
    }

    /// Fetch entries from the remote repo, updating the local cache.
    ///
    /// Strategy:
    /// 1. If cache exists, `git pull --ff-only`
    /// 2. If no cache, `git clone --depth 1`
    /// 3. If git operations fail but cache exists, use stale cache
    /// 4. If git operations fail and no cache, return empty vec
    pub async fn fetch(&self) -> Result<Vec<ArchitectureCatalogEntry>, FetchError> {
        let repo_path = self.repo_path();

        if repo_path.join(".git").exists() {
            match self.pull(&repo_path).await {
                Ok(()) => debug!("catalog cache updated"),
                Err(e) => {
                    warn!(
                        "failed to update catalog cache, using stale data: {e}"
                    );
                }
            }
        } else {
            match self.clone_repo(&repo_path).await {
                Ok(()) => debug!("catalog cache cloned"),
                Err(e) => {
                    warn!("failed to clone catalog repo: {e}");
                    return Ok(Vec::new());
                }
            }
        }

        self.load_entries(&repo_path).await
    }

    /// Load entries only from the existing cache, without any network calls.
    ///
    /// Returns an empty vec if no cache exists.
    pub async fn fetch_cached_only(
        &self,
    ) -> Result<Vec<ArchitectureCatalogEntry>, FetchError> {
        let repo_path = self.repo_path();
        if !repo_path.exists() {
            return Ok(Vec::new());
        }
        self.load_entries(&repo_path).await
    }

    /// Shallow-clone the repo into the cache directory.
    async fn clone_repo(&self, repo_path: &Path) -> Result<(), FetchError> {
        if let Some(parent) = repo_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                FetchError::CacheRead(format!(
                    "failed to create cache dir {}: {e}",
                    parent.display()
                ))
            })?;
        }

        let output = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                &self.repo_url,
                &repo_path.to_string_lossy(),
            ])
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    FetchError::GitNotFound
                } else {
                    FetchError::Io(e)
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FetchError::GitCommand(format!(
                "clone failed: {stderr}"
            )));
        }

        Ok(())
    }

    /// Pull updates into an existing cache.
    async fn pull(&self, repo_path: &Path) -> Result<(), FetchError> {
        let output = Command::new("git")
            .args(["-C", &repo_path.to_string_lossy(), "pull", "--ff-only"])
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    FetchError::GitNotFound
                } else {
                    FetchError::Io(e)
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // If pull fails (e.g., diverged), delete and re-clone
            warn!("pull failed ({stderr}), re-cloning...");
            std::fs::remove_dir_all(repo_path).map_err(|e| {
                FetchError::CacheRead(format!(
                    "failed to remove stale cache: {e}"
                ))
            })?;
            return self.clone_repo(repo_path).await;
        }

        Ok(())
    }

    /// Walk the repo directory and load all `.md` files as catalog entries.
    ///
    /// Expects structure: `{language}/{project-type}.md`
    /// Skips README.md and any files that fail to parse.
    async fn load_entries(
        &self,
        repo_path: &Path,
    ) -> Result<Vec<ArchitectureCatalogEntry>, FetchError> {
        let mut entries = Vec::new();

        for dir_entry in walkdir::WalkDir::new(repo_path)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = dir_entry.path();

            // Only process .md files
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            // Skip README, .github, etc.
            let relative = path.strip_prefix(repo_path).unwrap_or(path);
            let relative_str = relative.to_string_lossy();
            if relative_str.starts_with('.')
                || relative_str.starts_with("README")
            {
                continue;
            }

            match ArchitectureCatalogEntry::from_file(path).await {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    warn!(
                        "skipping {}: {e}",
                        relative.display()
                    );
                }
            }
        }

        debug!("loaded {} entries from catalog cache", entries.len());
        Ok(entries)
    }
}

/// Resolve the default cache directory.
///
/// Respects `XDG_CACHE_HOME` if set, otherwise uses `~/.cadre/catalog-cache/`.
pub fn default_cache_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        PathBuf::from(xdg).join("cadre").join("catalog-cache")
    } else if let Some(home) = dirs::home_dir() {
        home.join(".cadre").join("catalog-cache")
    } else {
        // Last resort — use a temp-like path
        PathBuf::from("/tmp").join("cadre-catalog-cache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_cache_dir_has_cadre_prefix() {
        let dir = default_cache_dir();
        let dir_str = dir.to_string_lossy();
        assert!(
            dir_str.contains("cadre") && dir_str.contains("catalog-cache"),
            "expected cadre/catalog-cache in path, got: {dir_str}"
        );
    }

    #[test]
    fn test_repo_path() {
        let fetcher =
            RemoteCatalogFetcher::new("https://example.com/repo.git", PathBuf::from("/tmp/cache"));
        assert_eq!(
            fetcher.repo_path(),
            PathBuf::from("/tmp/cache/cadre-architecture-docs")
        );
    }

    #[test]
    fn test_with_defaults_uses_correct_url() {
        let fetcher = RemoteCatalogFetcher::with_defaults();
        assert_eq!(fetcher.repo_url, DEFAULT_REPO_URL);
    }

    #[tokio::test]
    async fn test_fetch_cached_only_empty_dir() {
        let dir = tempdir().unwrap();
        let fetcher = RemoteCatalogFetcher::new("https://example.com/repo.git", dir.path().into());
        let entries = fetcher.fetch_cached_only().await.unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_load_entries_from_mock_dir() {
        use crate::domain::documents::architecture_catalog_entry::ArchitectureCatalogEntry;
        use crate::domain::documents::content::DocumentContent;
        use crate::domain::documents::metadata::DocumentMetadata;
        use crate::domain::documents::types::{Phase, Tag};

        let dir = tempdir().unwrap();
        let repo_dir = dir.path().join(CACHE_SUBDIR);
        let js_dir = repo_dir.join("javascript");
        std::fs::create_dir_all(&js_dir).unwrap();

        // Create a valid catalog entry file
        let entry = ArchitectureCatalogEntry::from_parts(
            "Test JS Server".to_string(),
            DocumentMetadata::new("TEST-AC-JS-SERVER".to_string()),
            DocumentContent::new("# Test JS Server\n\nA test entry."),
            vec![
                Tag::Label("architecture_catalog_entry".to_string()),
                Tag::Phase(Phase::Published),
            ],
            false,
            "javascript".to_string(),
            "server".to_string(),
            vec!["src/".to_string()],
            vec!["routes".to_string(), "handlers".to_string()],
            vec!["routes".to_string()],
            vec!["routes -> handlers".to_string()],
            vec!["camelCase".to_string()],
            vec!["god modules".to_string()],
            vec!["enforce-layers".to_string()],
            vec!["eslint-clean".to_string()],
        );
        entry
            .to_file(js_dir.join("server.md"))
            .await
            .unwrap();

        // Also create a README that should be skipped
        std::fs::write(repo_dir.join("README.md"), "# Readme").unwrap();

        let fetcher = RemoteCatalogFetcher::new("https://example.com/repo.git", dir.path().into());
        let entries = fetcher.load_entries(&repo_dir).await.unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].language, "javascript");
        assert_eq!(entries[0].project_type, "server");
    }

    #[tokio::test]
    async fn test_load_entries_skips_invalid_files() {
        let dir = tempdir().unwrap();
        let repo_dir = dir.path().join(CACHE_SUBDIR);
        let lang_dir = repo_dir.join("rust");
        std::fs::create_dir_all(&lang_dir).unwrap();

        // Write an invalid .md file
        std::fs::write(lang_dir.join("bad.md"), "not valid frontmatter at all").unwrap();

        let fetcher = RemoteCatalogFetcher::new("https://example.com/repo.git", dir.path().into());
        let entries = fetcher.load_entries(&repo_dir).await.unwrap();

        // Invalid file is skipped, not an error
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_load_entries_multiple_languages() {
        use crate::domain::documents::architecture_catalog_entry::ArchitectureCatalogEntry;
        use crate::domain::documents::content::DocumentContent;
        use crate::domain::documents::metadata::DocumentMetadata;
        use crate::domain::documents::types::{Phase, Tag};

        let dir = tempdir().unwrap();
        let repo_dir = dir.path().join(CACHE_SUBDIR);

        for (lang, ptype, code) in [
            ("javascript", "server", "TEST-AC-JS-SRV"),
            ("rust", "cli-tool", "TEST-AC-RS-CLI"),
            ("python", "web-app", "TEST-AC-PY-WEB"),
        ] {
            let lang_dir = repo_dir.join(lang);
            std::fs::create_dir_all(&lang_dir).unwrap();

            let entry = ArchitectureCatalogEntry::from_parts(
                format!("Test {lang} {ptype}"),
                DocumentMetadata::new(code.to_string()),
                DocumentContent::new(&format!("# Test {lang} {ptype}")),
                vec![
                    Tag::Label("architecture_catalog_entry".to_string()),
                    Tag::Phase(Phase::Published),
                ],
                false,
                lang.to_string(),
                ptype.to_string(),
                vec!["src/".to_string()],
                vec!["core".to_string()],
                vec![],
                vec!["no cycles".to_string()],
                vec!["snake_case".to_string()],
                vec!["god modules".to_string()],
                vec!["enforce-layers".to_string()],
                vec!["lint-clean".to_string()],
            );
            entry
                .to_file(lang_dir.join(format!("{ptype}.md")))
                .await
                .unwrap();
        }

        let fetcher = RemoteCatalogFetcher::new("https://example.com/repo.git", dir.path().into());
        let entries = fetcher.load_entries(&repo_dir).await.unwrap();

        assert_eq!(entries.len(), 3);
        let languages: Vec<&str> = entries.iter().map(|e| e.language.as_str()).collect();
        assert!(languages.contains(&"javascript"));
        assert!(languages.contains(&"rust"));
        assert!(languages.contains(&"python"));
    }
}
