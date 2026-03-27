use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoInfo {
    pub repo_name: String,
    pub repo_path: String,
    pub cadre_managed: bool,
}

/// Scan configured directories for git repositories.
///
/// For each directory in `scan_dirs`, looks one level deep for subdirectories
/// containing a `.git` folder. Detects Cadre-managed repos by checking for
/// `.cadre/` or `.metis/` directories.
///
/// Handles symlinks by resolving canonical paths and tracking visited paths
/// to avoid cycles. Permission errors are logged and skipped.
pub fn discover_repos(scan_dirs: &[PathBuf]) -> Vec<RepoInfo> {
    let mut repos = Vec::new();
    let mut visited = HashSet::new();

    for scan_dir in scan_dirs {
        let expanded = expand_home(scan_dir);
        scan_directory(&expanded, &mut repos, &mut visited);
    }

    repos
}

fn scan_directory(dir: &Path, repos: &mut Vec<RepoInfo>, visited: &mut HashSet<PathBuf>) {
    if !dir.exists() {
        tracing::warn!(path = %dir.display(), "Scan directory does not exist, skipping");
        return;
    }

    if !dir.is_dir() {
        tracing::warn!(path = %dir.display(), "Scan path is not a directory, skipping");
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!(path = %dir.display(), error = %e, "Failed to read scan directory, skipping");
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to read directory entry, skipping");
                continue;
            }
        };

        process_entry(&entry, repos, visited);
    }
}

fn process_entry(
    entry: &std::fs::DirEntry,
    repos: &mut Vec<RepoInfo>,
    visited: &mut HashSet<PathBuf>,
) {
    let path = entry.path();

    // Resolve canonical path to handle symlinks and detect cycles
    let canonical = match path.canonicalize() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "Failed to resolve path, skipping");
            return;
        }
    };

    // Skip if we've already visited this canonical path (symlink cycle protection)
    if !visited.insert(canonical.clone()) {
        tracing::debug!(path = %path.display(), "Already visited path (symlink), skipping");
        return;
    }

    // Only process directories
    if !canonical.is_dir() {
        return;
    }

    // Check if this directory is a git repo
    let git_dir = canonical.join(".git");
    if !git_dir.exists() {
        return;
    }

    // Determine if it's Cadre-managed
    let cadre_managed = canonical.join(".cadre").exists() || canonical.join(".metis").exists();

    // Extract repo name from directory name
    let repo_name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("unknown"));

    repos.push(RepoInfo {
        repo_name,
        repo_path: canonical.to_string_lossy().to_string(),
        cadre_managed,
    });
}

/// Public wrapper for `expand_home` used by the config module.
#[must_use]
pub fn expand_home_public(path: &Path) -> PathBuf {
    expand_home(path)
}

/// Expand `~` prefix to the user's home directory.
fn expand_home(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            let remainder = path_str.strip_prefix('~').unwrap_or("");
            let remainder = remainder.strip_prefix('/').unwrap_or(remainder);
            if remainder.is_empty() {
                return home;
            }
            return home.join(remainder);
        }
        tracing::warn!("Could not determine home directory for ~ expansion");
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_git_repo(parent: &Path, name: &str) -> PathBuf {
        let repo_dir = parent.join(name);
        std::fs::create_dir_all(repo_dir.join(".git")).unwrap();
        repo_dir
    }

    fn create_cadre_repo(parent: &Path, name: &str) -> PathBuf {
        let repo_dir = create_git_repo(parent, name);
        std::fs::create_dir_all(repo_dir.join(".cadre")).unwrap();
        repo_dir
    }

    fn create_metis_repo(parent: &Path, name: &str) -> PathBuf {
        let repo_dir = create_git_repo(parent, name);
        std::fs::create_dir_all(repo_dir.join(".metis")).unwrap();
        repo_dir
    }

    #[test]
    fn discovers_git_repos() {
        let tmp = TempDir::new().unwrap();
        let repo_a = create_git_repo(tmp.path(), "repo-a");
        let repo_b = create_git_repo(tmp.path(), "repo-b");

        let repos = discover_repos(&[tmp.path().to_path_buf()]);

        assert_eq!(repos.len(), 2);

        let names: HashSet<&str> = repos.iter().map(|r| r.repo_name.as_str()).collect();
        assert!(names.contains("repo-a"));
        assert!(names.contains("repo-b"));

        // Verify paths resolve to canonical form
        let paths: HashSet<&str> = repos.iter().map(|r| r.repo_path.as_str()).collect();
        assert!(paths.contains(repo_a.canonicalize().unwrap().to_str().unwrap()));
        assert!(paths.contains(repo_b.canonicalize().unwrap().to_str().unwrap()));

        // Non-cadre repos
        for repo in &repos {
            assert!(!repo.cadre_managed);
        }
    }

    #[test]
    fn discovers_empty_directory() {
        let tmp = TempDir::new().unwrap();
        let repos = discover_repos(&[tmp.path().to_path_buf()]);
        assert!(repos.is_empty());
    }

    #[test]
    fn handles_missing_directory() {
        let repos = discover_repos(&[PathBuf::from("/nonexistent/path/that/does/not/exist")]);
        assert!(repos.is_empty());
    }

    #[test]
    fn detects_cadre_managed_repo() {
        let tmp = TempDir::new().unwrap();
        create_cadre_repo(tmp.path(), "cadre-project");

        let repos = discover_repos(&[tmp.path().to_path_buf()]);

        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].repo_name, "cadre-project");
        assert!(repos[0].cadre_managed);
    }

    #[test]
    fn detects_metis_managed_repo() {
        let tmp = TempDir::new().unwrap();
        create_metis_repo(tmp.path(), "metis-project");

        let repos = discover_repos(&[tmp.path().to_path_buf()]);

        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].repo_name, "metis-project");
        assert!(repos[0].cadre_managed);
    }

    #[test]
    fn skips_non_git_directories() {
        let tmp = TempDir::new().unwrap();
        // Create a regular directory (no .git)
        std::fs::create_dir_all(tmp.path().join("not-a-repo")).unwrap();
        // Create a git repo to ensure we still find valid ones
        create_git_repo(tmp.path(), "real-repo");

        let repos = discover_repos(&[tmp.path().to_path_buf()]);

        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].repo_name, "real-repo");
    }

    #[test]
    fn skips_files_in_scan_dir() {
        let tmp = TempDir::new().unwrap();
        // Create a regular file in the scan directory
        std::fs::write(tmp.path().join("some-file.txt"), "hello").unwrap();
        create_git_repo(tmp.path(), "valid-repo");

        let repos = discover_repos(&[tmp.path().to_path_buf()]);

        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].repo_name, "valid-repo");
    }

    #[test]
    fn handles_multiple_scan_dirs() {
        let tmp_a = TempDir::new().unwrap();
        let tmp_b = TempDir::new().unwrap();
        create_git_repo(tmp_a.path(), "alpha");
        create_git_repo(tmp_b.path(), "beta");

        let repos = discover_repos(&[tmp_a.path().to_path_buf(), tmp_b.path().to_path_buf()]);

        assert_eq!(repos.len(), 2);
        let names: HashSet<&str> = repos.iter().map(|r| r.repo_name.as_str()).collect();
        assert!(names.contains("alpha"));
        assert!(names.contains("beta"));
    }

    #[cfg(unix)]
    #[test]
    fn discovers_symlinked_repos() {
        let tmp = TempDir::new().unwrap();
        let actual_repo = create_git_repo(tmp.path(), "actual-repo");
        let link_path = tmp.path().join("linked-repo");
        std::os::unix::fs::symlink(&actual_repo, &link_path).unwrap();

        let repos = discover_repos(&[tmp.path().to_path_buf()]);

        // Both entries point to the same canonical path, so only one should appear
        assert_eq!(repos.len(), 1);
        assert_eq!(
            repos[0].repo_path,
            actual_repo.canonicalize().unwrap().to_string_lossy()
        );
    }

    #[test]
    fn deduplicates_across_scan_dirs() {
        let tmp = TempDir::new().unwrap();
        create_git_repo(tmp.path(), "repo");

        // Same directory listed twice
        let repos = discover_repos(&[tmp.path().to_path_buf(), tmp.path().to_path_buf()]);

        assert_eq!(repos.len(), 1);
    }

    #[test]
    fn expand_home_with_tilde() {
        let expanded = expand_home(Path::new("~/projects"));
        if let Some(home) = dirs::home_dir() {
            assert_eq!(expanded, home.join("projects"));
        }
    }

    #[test]
    fn expand_home_without_tilde() {
        let path = PathBuf::from("/absolute/path");
        let expanded = expand_home(&path);
        assert_eq!(expanded, path);
    }

    #[test]
    fn expand_home_tilde_only() {
        let expanded = expand_home(Path::new("~"));
        if let Some(home) = dirs::home_dir() {
            assert_eq!(expanded, home);
        }
    }
}
