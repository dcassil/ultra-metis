use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Information about a discovered repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepoInfo {
    #[serde(rename(serialize = "name"), alias = "name")]
    pub repo_name: String,
    #[serde(rename(serialize = "path"), alias = "path")]
    pub repo_path: String,
    pub cadre_managed: bool,
}

/// Scan the given directories for git repositories.
///
/// For each directory, looks one level deep for directories containing `.git`.
/// Marks repos as `cadre_managed` if they contain a `.cadre` directory.
///
/// # Errors
///
/// Returns an error if a scan directory cannot be read.
pub fn discover_repos(scan_dirs: &[PathBuf]) -> anyhow::Result<Vec<RepoInfo>> {
    let mut repos = Vec::new();

    for dir in scan_dirs {
        if !dir.exists() {
            tracing::warn!(path = %dir.display(), "Scan directory does not exist, skipping");
            continue;
        }

        let entries = std::fs::read_dir(dir)
            .map_err(|e| anyhow::anyhow!("Failed to read directory {}: {e}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(info) = check_repo(&path) {
                    repos.push(info);
                }
            }
        }
    }

    repos.sort_by(|a, b| a.repo_name.cmp(&b.repo_name));
    Ok(repos)
}

fn check_repo(path: &Path) -> Option<RepoInfo> {
    let git_dir = path.join(".git");
    if !git_dir.exists() {
        return None;
    }

    let repo_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let cadre_managed = path.join(".cadre").exists();

    Some(RepoInfo {
        repo_name,
        repo_path: path.to_string_lossy().to_string(),
        cadre_managed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_repos_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let repos = discover_repos(&[tmp.path().to_path_buf()]).unwrap();
        assert!(repos.is_empty());
    }

    #[test]
    fn test_discover_repos_finds_git_repos() {
        let tmp = tempfile::tempdir().unwrap();

        // Create a fake repo with .git directory
        let repo_dir = tmp.path().join("my-project");
        std::fs::create_dir_all(repo_dir.join(".git")).unwrap();

        // Create a non-repo directory
        std::fs::create_dir_all(tmp.path().join("not-a-repo")).unwrap();

        let repos = discover_repos(&[tmp.path().to_path_buf()]).unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].repo_name, "my-project");
        assert!(!repos[0].cadre_managed);
    }

    #[test]
    fn test_discover_repos_detects_cadre_managed() {
        let tmp = tempfile::tempdir().unwrap();

        let repo_dir = tmp.path().join("cadre-project");
        std::fs::create_dir_all(repo_dir.join(".git")).unwrap();
        std::fs::create_dir_all(repo_dir.join(".cadre")).unwrap();

        let repos = discover_repos(&[tmp.path().to_path_buf()]).unwrap();
        assert_eq!(repos.len(), 1);
        assert!(repos[0].cadre_managed);
    }

    #[test]
    fn test_discover_repos_skips_missing_dirs() {
        let repos = discover_repos(&[PathBuf::from("/nonexistent/path/12345")]).unwrap();
        assert!(repos.is_empty());
    }
}
