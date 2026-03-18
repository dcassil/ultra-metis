use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_FILE: &str = "scenario.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioPackManifest {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub documents: ScenarioDocuments,
    #[serde(default)]
    pub expectations: ScenarioExpectations,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioDocuments {
    pub vision: String,
    #[serde(default)]
    pub specification: Option<String>,
    #[serde(default)]
    pub seed_initiatives: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioExpectations {
    #[serde(default)]
    pub benchmark_tracks: Vec<String>,
    #[serde(default)]
    pub required_document_roles: Vec<String>,
    #[serde(default)]
    pub architecture_focus: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedScenarioPack {
    pub root: PathBuf,
    pub manifest: ScenarioPackManifest,
    pub vision: String,
    pub specification: Option<String>,
    pub seed_initiatives: Vec<ScenarioArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioArtifact {
    pub path: PathBuf,
    pub content: String,
}

impl LoadedScenarioPack {
    pub fn load(scenario_root: &Path) -> Result<Self> {
        let manifest = load_manifest(scenario_root)?;
        let vision_path = scenario_root.join(&manifest.documents.vision);
        let vision = read_required(&vision_path)?;

        let specification = manifest
            .documents
            .specification
            .as_ref()
            .map(|path| read_required(&scenario_root.join(path)))
            .transpose()?;

        let seed_initiatives = manifest
            .documents
            .seed_initiatives
            .iter()
            .map(|path| {
                let artifact_path = scenario_root.join(path);
                Ok(ScenarioArtifact {
                    content: read_required(&artifact_path)?,
                    path: artifact_path,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            root: scenario_root.to_path_buf(),
            manifest,
            vision,
            specification,
            seed_initiatives,
        })
    }
}

fn load_manifest(scenario_root: &Path) -> Result<ScenarioPackManifest> {
    let manifest_path = scenario_root.join(MANIFEST_FILE);
    if manifest_path.exists() {
        let raw = fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read {}", manifest_path.display()))?;
        return serde_json::from_str(&raw)
            .with_context(|| format!("Failed to parse {}", manifest_path.display()));
    }

    Ok(infer_legacy_manifest(scenario_root))
}

fn infer_legacy_manifest(scenario_root: &Path) -> ScenarioPackManifest {
    ScenarioPackManifest {
        id: scenario_root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("scenario")
            .to_string(),
        title: "Legacy Scenario Pack".to_string(),
        description: Some(
            "Inferred from legacy benchmark files without an explicit scenario manifest."
                .to_string(),
        ),
        documents: ScenarioDocuments {
            vision: "vision.md".to_string(),
            specification: Some("spec.md".to_string()),
            seed_initiatives: vec![
                "parse-initiative.md".to_string(),
                "transform-initiative.md".to_string(),
            ],
        },
        expectations: ScenarioExpectations {
            benchmark_tracks: vec![
                "document_generation".to_string(),
                "decomposition".to_string(),
                "build_outcome".to_string(),
            ],
            required_document_roles: vec![
                "vision".to_string(),
                "initiative".to_string(),
                "task".to_string(),
            ],
            architecture_focus: vec![
                "typed data model".to_string(),
                "module boundaries".to_string(),
                "validation flow".to_string(),
            ],
        },
    }
}

fn read_required(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_legacy_scenario_pack_from_existing_layout() {
        let root = PathBuf::from("scenario");
        let loaded = LoadedScenarioPack::load(&root).unwrap();

        assert_eq!(loaded.manifest.documents.vision, "vision.md");
        assert_eq!(loaded.seed_initiatives.len(), 2);
        assert!(loaded.specification.is_some());
        assert!(loaded.vision.contains("File Processing Toolkit"));
    }

    #[test]
    fn load_explicit_manifest() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(
            temp.path().join(MANIFEST_FILE),
            r#"{
  "id": "demo-pack",
  "title": "Demo Pack",
  "documents": {
    "vision": "docs/vision.md",
    "specification": "docs/spec.md",
    "seed_initiatives": ["docs/init-a.md"]
  },
  "expectations": {
    "benchmark_tracks": ["document_generation"]
  }
}"#,
        )
        .unwrap();
        fs::create_dir_all(temp.path().join("docs")).unwrap();
        fs::write(temp.path().join("docs/vision.md"), "vision").unwrap();
        fs::write(temp.path().join("docs/spec.md"), "spec").unwrap();
        fs::write(temp.path().join("docs/init-a.md"), "initiative").unwrap();

        let loaded = LoadedScenarioPack::load(temp.path()).unwrap();

        assert_eq!(loaded.manifest.id, "demo-pack");
        assert_eq!(loaded.manifest.title, "Demo Pack");
        assert_eq!(
            loaded.manifest.expectations.benchmark_tracks,
            vec!["document_generation".to_string()]
        );
        assert_eq!(loaded.seed_initiatives.len(), 1);
        assert_eq!(loaded.specification.as_deref(), Some("spec"));
    }
}
