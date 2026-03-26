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
    #[serde(default)]
    pub run_rules: RunRules,
    #[serde(default)]
    pub expected_outputs: ExpectedOutputs,
    #[serde(default)]
    pub verification: Vec<VerificationCheck>,
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

/// Budget and stopping constraints for a benchmark run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunRules {
    /// Maximum input+output tokens allowed for the entire run.
    #[serde(default = "default_token_budget")]
    pub token_budget: u64,
    /// Maximum wall-clock seconds for the entire run.
    #[serde(default = "default_time_budget_secs")]
    pub time_budget_secs: u64,
    /// Stop after this many consecutive failures without progress.
    #[serde(default = "default_max_consecutive_failures")]
    pub max_consecutive_failures: u32,
    /// Tool surfaces the system under test is allowed to use.
    #[serde(default)]
    pub allowed_tool_surfaces: Vec<String>,
}

impl Default for RunRules {
    fn default() -> Self {
        Self {
            token_budget: default_token_budget(),
            time_budget_secs: default_time_budget_secs(),
            max_consecutive_failures: default_max_consecutive_failures(),
            allowed_tool_surfaces: Vec::new(),
        }
    }
}

fn default_token_budget() -> u64 {
    500_000
}
fn default_time_budget_secs() -> u64 {
    600
}
fn default_max_consecutive_failures() -> u32 {
    3
}

/// What the agent is expected to produce, used for scoring.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpectedOutputs {
    /// Documents the agent must create (by role, e.g. "initiative", "task").
    #[serde(default)]
    pub required_docs: Vec<RequiredDoc>,
    /// Expected parent-child hierarchy links.
    #[serde(default)]
    pub expected_hierarchy: Vec<HierarchyLink>,
    /// Architecture constraints that must be preserved.
    #[serde(default)]
    pub architecture_constraints: Vec<ArchitectureConstraint>,
}

/// A document the agent is expected to create.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequiredDoc {
    pub role: String,
    #[serde(default)]
    pub min_count: Option<u32>,
    #[serde(default)]
    pub required_sections: Vec<String>,
}

/// An expected hierarchy relationship.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HierarchyLink {
    pub parent_role: String,
    pub child_role: String,
    #[serde(default)]
    pub min_children: Option<u32>,
}

/// An architecture constraint the system must respect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchitectureConstraint {
    pub name: String,
    pub description: String,
    /// Grep-style patterns that should or should not appear in generated code.
    #[serde(default)]
    pub boundary_patterns: Vec<BoundaryPattern>,
}

/// A pattern to check for architecture boundary conformance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoundaryPattern {
    pub pattern: String,
    /// "must_exist" or "must_not_exist"
    pub kind: String,
    #[serde(default)]
    pub file_glob: Option<String>,
}

/// A deterministic verification check to run after a benchmark.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationCheck {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub expected_exit_code: Option<i32>,
    #[serde(default)]
    pub expected_stdout_contains: Vec<String>,
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

    /// List all scenario pack directories under a parent path.
    pub fn discover(scenarios_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut packs = Vec::new();
        if !scenarios_dir.is_dir() {
            return Ok(packs);
        }
        for entry in fs::read_dir(scenarios_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join(MANIFEST_FILE);
                if manifest_path.exists() {
                    packs.push(path);
                }
            }
        }
        packs.sort();
        Ok(packs)
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
        run_rules: RunRules::default(),
        expected_outputs: ExpectedOutputs::default(),
        verification: Vec::new(),
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
        // New fields default when omitted
        assert_eq!(loaded.manifest.run_rules.token_budget, 500_000);
        assert!(loaded.manifest.expected_outputs.required_docs.is_empty());
        assert!(loaded.manifest.verification.is_empty());
    }

    #[test]
    fn load_manifest_with_full_schema() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(
            temp.path().join(MANIFEST_FILE),
            r#"{
  "id": "full-schema-pack",
  "title": "Full Schema Pack",
  "documents": {
    "vision": "vision.md"
  },
  "expectations": {
    "benchmark_tracks": ["document_generation", "build_outcome"],
    "architecture_focus": ["module boundaries"]
  },
  "run_rules": {
    "token_budget": 200000,
    "time_budget_secs": 300,
    "max_consecutive_failures": 2,
    "allowed_tool_surfaces": ["mcp", "cli"]
  },
  "expected_outputs": {
    "required_docs": [
      {"role": "initiative", "min_count": 2, "required_sections": ["Objective", "Acceptance Criteria"]},
      {"role": "task", "min_count": 4}
    ],
    "expected_hierarchy": [
      {"parent_role": "vision", "child_role": "initiative", "min_children": 2},
      {"parent_role": "initiative", "child_role": "task", "min_children": 1}
    ],
    "architecture_constraints": [
      {
        "name": "module-isolation",
        "description": "Each module must have its own directory",
        "boundary_patterns": [
          {"pattern": "mod\\.rs$", "kind": "must_exist", "file_glob": "src/*/mod.rs"}
        ]
      }
    ]
  },
  "verification": [
    {"name": "cargo-build", "command": "cargo build", "expected_exit_code": 0},
    {"name": "cargo-test", "command": "cargo test", "expected_exit_code": 0, "expected_stdout_contains": ["test result: ok"]}
  ]
}"#,
        )
        .unwrap();
        fs::write(temp.path().join("vision.md"), "# Full Vision").unwrap();

        let loaded = LoadedScenarioPack::load(temp.path()).unwrap();

        assert_eq!(loaded.manifest.id, "full-schema-pack");
        assert_eq!(loaded.manifest.run_rules.token_budget, 200_000);
        assert_eq!(loaded.manifest.run_rules.time_budget_secs, 300);
        assert_eq!(loaded.manifest.run_rules.max_consecutive_failures, 2);
        assert_eq!(
            loaded.manifest.run_rules.allowed_tool_surfaces,
            vec!["mcp".to_string(), "cli".to_string()]
        );

        assert_eq!(loaded.manifest.expected_outputs.required_docs.len(), 2);
        assert_eq!(
            loaded.manifest.expected_outputs.required_docs[0].role,
            "initiative"
        );
        assert_eq!(
            loaded.manifest.expected_outputs.required_docs[0].min_count,
            Some(2)
        );
        assert_eq!(
            loaded.manifest.expected_outputs.required_docs[0]
                .required_sections
                .len(),
            2
        );

        assert_eq!(loaded.manifest.expected_outputs.expected_hierarchy.len(), 2);
        assert_eq!(
            loaded
                .manifest
                .expected_outputs
                .architecture_constraints
                .len(),
            1
        );
        assert_eq!(
            loaded.manifest.expected_outputs.architecture_constraints[0]
                .boundary_patterns
                .len(),
            1
        );

        assert_eq!(loaded.manifest.verification.len(), 2);
        assert_eq!(loaded.manifest.verification[0].name, "cargo-build");
        assert_eq!(
            loaded.manifest.verification[1]
                .expected_stdout_contains
                .len(),
            1
        );
    }

    #[test]
    fn discover_scenario_packs() {
        let temp = tempfile::tempdir().unwrap();

        // Create two valid scenario packs
        let pack_a = temp.path().join("pack-a");
        let pack_b = temp.path().join("pack-b");
        let not_a_pack = temp.path().join("not-a-pack");

        fs::create_dir_all(&pack_a).unwrap();
        fs::create_dir_all(&pack_b).unwrap();
        fs::create_dir_all(&not_a_pack).unwrap();

        fs::write(
            pack_a.join(MANIFEST_FILE),
            r#"{"id":"a","title":"A","documents":{"vision":"v.md"}}"#,
        )
        .unwrap();
        fs::write(
            pack_b.join(MANIFEST_FILE),
            r#"{"id":"b","title":"B","documents":{"vision":"v.md"}}"#,
        )
        .unwrap();
        // not_a_pack has no scenario.json

        let found = LoadedScenarioPack::discover(temp.path()).unwrap();
        assert_eq!(found.len(), 2);
        assert!(found[0].ends_with("pack-a"));
        assert!(found[1].ends_with("pack-b"));
    }

    #[test]
    fn run_rules_defaults() {
        let rules = RunRules::default();
        assert_eq!(rules.token_budget, 500_000);
        assert_eq!(rules.time_budget_secs, 600);
        assert_eq!(rules.max_consecutive_failures, 3);
        assert!(rules.allowed_tool_surfaces.is_empty());
    }
}
