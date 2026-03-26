use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{Phase, StoryType, Tag};
use chrono::{DateTime, Utc};
use gray_matter;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tera::{Context, Tera};

/// Extract a string array from frontmatter, returning empty vec for null/missing.
fn extract_string_array_or_empty(
    map: &std::collections::HashMap<String, gray_matter::Pod>,
    key: &str,
) -> Vec<String> {
    match map.get(key) {
        Some(gray_matter::Pod::Array(arr)) => arr
            .iter()
            .filter_map(|item| {
                if let gray_matter::Pod::String(s) = item {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// A single architecture checklist item with question, optional answer, and story type relevance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub question: String,
    pub answer: Option<String>,
    pub story_types: Vec<StoryType>,
}

/// A record of an unlock event on an Architecture document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockRecord {
    pub actor: String,
    pub timestamp: DateTime<Utc>,
    pub reason: String,
}

/// Architecture context document linked to a Story.
///
/// Captures the relevant slice of a project's ReferenceArchitecture for a specific
/// Story, including a structured checklist. Created automatically when a Story enters
/// its design phase (via lifecycle hooks). Locked by default; requires explicit unlock
/// to edit.
///
/// This is a governance-type document — it does not implement the full Document trait
/// and does not participate in phase transitions. It is always in a "published" state.
#[derive(Debug)]
pub struct Architecture {
    core: DocumentCore,
    pub source_reference_architecture: Option<String>,
    pub relevant_layers: Vec<String>,
    pub relevant_boundaries: Vec<String>,
    pub applicable_dependency_rules: Vec<String>,
    pub applicable_naming_conventions: Vec<String>,
    pub applicable_anti_patterns: Vec<String>,
    pub checklist: Vec<ChecklistItem>,
    pub locked: bool,
    pub baseline_score: Option<f64>,
    pub completion_score: Option<f64>,
    pub drift_tolerance: f64,
    pub unlock_history: Vec<UnlockRecord>,
}

impl Architecture {
    /// Create a new Architecture document with template content.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        short_code: String,
        parent_id: Option<String>,
        source_reference_architecture: Option<String>,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("architecture_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {e}"))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera.render("architecture_content", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Template render error: {e}"))
        })?;

        let content = DocumentContent::new(&rendered_content);

        Ok(Self {
            core: DocumentCore {
                title,
                metadata,
                content,
                parent_id: parent_id.map(std::convert::Into::into),
                blocked_by: Vec::new(),
                tags: vec![Tag::Phase(Phase::Published)],
                archived: false,
                epic_id: None,
                schema_version: 1,
            },
            source_reference_architecture,
            relevant_layers: Vec::new(),
            relevant_boundaries: Vec::new(),
            applicable_dependency_rules: Vec::new(),
            applicable_naming_conventions: Vec::new(),
            applicable_anti_patterns: Vec::new(),
            checklist: Vec::new(),
            locked: true,
            baseline_score: None,
            completion_score: None,
            drift_tolerance: 0.02,
            unlock_history: Vec::new(),
        })
    }

    /// Construct from all parts (used by deserialization).
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        parent_id: Option<String>,
        source_reference_architecture: Option<String>,
        relevant_layers: Vec<String>,
        relevant_boundaries: Vec<String>,
        applicable_dependency_rules: Vec<String>,
        applicable_naming_conventions: Vec<String>,
        applicable_anti_patterns: Vec<String>,
        checklist: Vec<ChecklistItem>,
        locked: bool,
        baseline_score: Option<f64>,
        completion_score: Option<f64>,
        drift_tolerance: f64,
        unlock_history: Vec<UnlockRecord>,
    ) -> Self {
        Self {
            core: DocumentCore {
                title,
                metadata,
                content,
                parent_id: parent_id.map(std::convert::Into::into),
                blocked_by: Vec::new(),
                tags,
                archived,
                epic_id: None,
                schema_version: 1,
            },
            source_reference_architecture,
            relevant_layers,
            relevant_boundaries,
            applicable_dependency_rules,
            applicable_naming_conventions,
            applicable_anti_patterns,
            checklist,
            locked,
            baseline_score,
            completion_score,
            drift_tolerance,
            unlock_history,
        }
    }

    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to read file: {e}"))
        })?;
        Self::from_content(&raw_content)
    }

    pub fn from_content(raw_content: &str) -> Result<Self, DocumentValidationError> {
        let parsed = gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(raw_content);

        let frontmatter = parsed.data.ok_or_else(|| {
            DocumentValidationError::MissingRequiredField("frontmatter".to_string())
        })?;

        let fm_map = match frontmatter {
            gray_matter::Pod::Hash(map) => map,
            _ => {
                return Err(DocumentValidationError::InvalidContent(
                    "Frontmatter must be a hash/map".to_string(),
                ))
            }
        };

        let title = FrontmatterParser::extract_string(&fm_map, "title")?;
        let archived = FrontmatterParser::extract_bool(&fm_map, "archived").unwrap_or(false);
        let created_at = FrontmatterParser::extract_datetime(&fm_map, "created_at")?;
        let updated_at = FrontmatterParser::extract_datetime(&fm_map, "updated_at")?;
        let exit_criteria_met =
            FrontmatterParser::extract_bool(&fm_map, "exit_criteria_met").unwrap_or(false);
        let tags = FrontmatterParser::extract_tags(&fm_map)?;

        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "architecture" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'architecture', found '{level}'"
            )));
        }

        let short_code = FrontmatterParser::extract_string(&fm_map, "short_code")?;
        let metadata = DocumentMetadata::from_frontmatter(
            created_at,
            updated_at,
            exit_criteria_met,
            short_code,
        );
        let content = DocumentContent::from_markdown(&parsed.content);

        let parent_id = FrontmatterParser::extract_optional_string(&fm_map, "parent_id");
        let source_reference_architecture =
            FrontmatterParser::extract_optional_string(&fm_map, "source_reference_architecture");

        let relevant_layers = extract_string_array_or_empty(&fm_map, "relevant_layers");
        let relevant_boundaries = extract_string_array_or_empty(&fm_map, "relevant_boundaries");
        let applicable_dependency_rules =
            extract_string_array_or_empty(&fm_map, "applicable_dependency_rules");
        let applicable_naming_conventions =
            extract_string_array_or_empty(&fm_map, "applicable_naming_conventions");
        let applicable_anti_patterns =
            extract_string_array_or_empty(&fm_map, "applicable_anti_patterns");

        let locked = FrontmatterParser::extract_bool(&fm_map, "locked").unwrap_or(true);
        let baseline_score = FrontmatterParser::extract_float(&fm_map, "baseline_score");
        let completion_score = FrontmatterParser::extract_float(&fm_map, "completion_score");
        let drift_tolerance =
            FrontmatterParser::extract_float(&fm_map, "drift_tolerance").unwrap_or(0.02);

        let checklist = Self::parse_checklist(&fm_map);
        let unlock_history = Self::parse_unlock_history(&fm_map);

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            tags,
            archived,
            parent_id,
            source_reference_architecture,
            relevant_layers,
            relevant_boundaries,
            applicable_dependency_rules,
            applicable_naming_conventions,
            applicable_anti_patterns,
            checklist,
            locked,
            baseline_score,
            completion_score,
            drift_tolerance,
            unlock_history,
        ))
    }

    fn parse_checklist(
        fm_map: &std::collections::HashMap<String, gray_matter::Pod>,
    ) -> Vec<ChecklistItem> {
        match fm_map.get("checklist") {
            Some(gray_matter::Pod::Array(arr)) => arr
                .iter()
                .filter_map(|item| {
                    if let gray_matter::Pod::Hash(map) = item {
                        let question = match map.get("question") {
                            Some(gray_matter::Pod::String(s)) => s.clone(),
                            _ => return None,
                        };
                        let answer = match map.get("answer") {
                            Some(gray_matter::Pod::String(s)) if s != "null" && s != "NULL" => {
                                Some(s.clone())
                            }
                            _ => None,
                        };
                        let story_types = match map.get("story_types") {
                            Some(gray_matter::Pod::Array(arr)) => arr
                                .iter()
                                .filter_map(|st| {
                                    if let gray_matter::Pod::String(s) = st {
                                        s.parse::<StoryType>().ok()
                                    } else {
                                        None
                                    }
                                })
                                .collect(),
                            _ => Vec::new(),
                        };
                        Some(ChecklistItem {
                            question,
                            answer,
                            story_types,
                        })
                    } else {
                        None
                    }
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    fn parse_unlock_history(
        fm_map: &std::collections::HashMap<String, gray_matter::Pod>,
    ) -> Vec<UnlockRecord> {
        match fm_map.get("unlock_history") {
            Some(gray_matter::Pod::Array(arr)) => arr
                .iter()
                .filter_map(|item| {
                    if let gray_matter::Pod::Hash(map) = item {
                        let actor = match map.get("actor") {
                            Some(gray_matter::Pod::String(s)) => s.clone(),
                            _ => return None,
                        };
                        let timestamp = match map.get("timestamp") {
                            Some(gray_matter::Pod::String(s)) => {
                                DateTime::parse_from_rfc3339(s).ok()?.with_timezone(&Utc)
                            }
                            _ => return None,
                        };
                        let reason = match map.get("reason") {
                            Some(gray_matter::Pod::String(s)) => s.clone(),
                            _ => return None,
                        };
                        Some(UnlockRecord {
                            actor,
                            timestamp,
                            reason,
                        })
                    } else {
                        None
                    }
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    // --- accessors ---

    pub fn title(&self) -> &str {
        &self.core.title
    }

    pub fn metadata(&self) -> &DocumentMetadata {
        &self.core.metadata
    }

    pub fn content(&self) -> &DocumentContent {
        &self.core.content
    }

    pub fn tags(&self) -> &[Tag] {
        &self.core.tags
    }

    pub fn archived(&self) -> bool {
        self.core.archived
    }

    pub fn parent_id(&self) -> Option<&str> {
        self.core.parent_id.as_ref().map(super::types::DocumentId::as_str)
    }

    /// Architecture documents are always in Published phase (no lifecycle).
    pub fn phase(&self) -> Result<Phase, DocumentValidationError> {
        Ok(Phase::Published)
    }

    /// Mutable access to the document core (needed by store for phase tag updates).
    pub fn core_mut(&mut self) -> &mut DocumentCore {
        &mut self.core
    }

    // --- lock/unlock ---

    /// Unlock this document for editing. Records the unlock event.
    pub fn unlock(&mut self, actor: String, reason: String) {
        self.locked = false;
        self.unlock_history.push(UnlockRecord {
            actor,
            timestamp: Utc::now(),
            reason,
        });
        self.core.metadata.updated_at = Utc::now();
    }

    /// Re-lock the document after editing.
    pub fn lock(&mut self) {
        self.locked = true;
        self.core.metadata.updated_at = Utc::now();
    }

    // --- checklist templates ---

    /// Generate checklist items appropriate for the given StoryType.
    pub fn checklist_for_story_type(story_type: StoryType) -> Vec<ChecklistItem> {
        let mut items = Vec::new();

        // Common questions for most types
        let boundary_q = ChecklistItem {
            question: "Does this touch or cross existing module boundaries?".to_string(),
            answer: None,
            story_types: vec![
                StoryType::Feature,
                StoryType::Bugfix,
                StoryType::Refactor,
                StoryType::Migration,
                StoryType::ArchitectureChange,
            ],
        };
        let anti_pattern_q = ChecklistItem {
            question: "Are there known anti-patterns relevant to this area?".to_string(),
            answer: None,
            story_types: vec![
                StoryType::Feature,
                StoryType::Bugfix,
                StoryType::Investigation,
                StoryType::Remediation,
                StoryType::Setup,
            ],
        };

        match story_type {
            StoryType::Feature => {
                items.push(ChecklistItem {
                    question: "Does this introduce new cross-layer dependencies?".to_string(),
                    answer: None,
                    story_types: vec![StoryType::Feature],
                });
                items.push(boundary_q);
                items.push(ChecklistItem {
                    question: "Does this follow naming conventions for the affected layers?"
                        .to_string(),
                    answer: None,
                    story_types: vec![StoryType::Feature],
                });
                items.push(anti_pattern_q);
            }
            StoryType::Bugfix => {
                items.push(ChecklistItem {
                    question: "Does the fix respect existing layer boundaries?".to_string(),
                    answer: None,
                    story_types: vec![StoryType::Bugfix],
                });
                items.push(ChecklistItem {
                    question: "Could the root cause indicate an architecture violation?"
                        .to_string(),
                    answer: None,
                    story_types: vec![StoryType::Bugfix],
                });
            }
            StoryType::Refactor => {
                items.push(ChecklistItem {
                    question: "Does this change any module boundaries?".to_string(),
                    answer: None,
                    story_types: vec![StoryType::Refactor],
                });
                items.push(ChecklistItem {
                    question: "Does this alter dependency direction between layers?".to_string(),
                    answer: None,
                    story_types: vec![StoryType::Refactor],
                });
                items.push(ChecklistItem {
                    question:
                        "Should the ReferenceArchitecture be updated to reflect this refactor?"
                            .to_string(),
                    answer: None,
                    story_types: vec![StoryType::Refactor],
                });
            }
            StoryType::Migration => {
                items.push(ChecklistItem {
                    question: "Does this require updating the ReferenceArchitecture?".to_string(),
                    answer: None,
                    story_types: vec![StoryType::Migration],
                });
                items.push(ChecklistItem {
                    question: "Which tolerated exceptions does this affect?".to_string(),
                    answer: None,
                    story_types: vec![StoryType::Migration],
                });
                items.push(ChecklistItem {
                    question: "Are there new boundaries or layers introduced?".to_string(),
                    answer: None,
                    story_types: vec![StoryType::Migration],
                });
            }
            StoryType::ArchitectureChange => {
                // Full checklist — all questions
                items.push(ChecklistItem {
                    question: "Does this introduce new cross-layer dependencies?".to_string(),
                    answer: None,
                    story_types: vec![StoryType::ArchitectureChange],
                });
                items.push(boundary_q);
                items.push(ChecklistItem {
                    question: "Does this alter dependency direction between layers?".to_string(),
                    answer: None,
                    story_types: vec![StoryType::ArchitectureChange],
                });
                items.push(ChecklistItem {
                    question: "What is the expected conformance impact?".to_string(),
                    answer: None,
                    story_types: vec![StoryType::ArchitectureChange],
                });
                items.push(ChecklistItem {
                    question: "Does this require a new ADR?".to_string(),
                    answer: None,
                    story_types: vec![StoryType::ArchitectureChange],
                });
                items.push(ChecklistItem {
                    question: "Should the ReferenceArchitecture be updated after this change?"
                        .to_string(),
                    answer: None,
                    story_types: vec![StoryType::ArchitectureChange],
                });
                items.push(anti_pattern_q);
            }
            StoryType::Investigation | StoryType::Remediation | StoryType::Setup => {
                // Minimal checklist
                items.push(ChecklistItem {
                    question: "Does this touch or cross existing module boundaries?".to_string(),
                    answer: None,
                    story_types: vec![story_type],
                });
                items.push(anti_pattern_q);
            }
        }

        items
    }

    // --- serialization ---

    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to write file: {e}"))
        })
    }

    pub fn to_content(&self) -> Result<String, DocumentValidationError> {
        let mut tera = Tera::default();
        tera.add_raw_template("frontmatter", include_str!("frontmatter.yaml"))
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {e}"))
            })?;

        let mut context = Context::new();
        context.insert("slug", &self.core.metadata.short_code);
        context.insert("title", self.title());
        context.insert("short_code", &self.core.metadata.short_code);
        context.insert("created_at", &self.core.metadata.created_at.to_rfc3339());
        context.insert("updated_at", &self.core.metadata.updated_at.to_rfc3339());
        context.insert("archived", &self.archived().to_string());
        context.insert(
            "exit_criteria_met",
            &self.core.metadata.exit_criteria_met.to_string(),
        );
        context.insert(
            "parent_id",
            &self
                .parent_id()
                .map(|s| format!("\"{s}\""))
                .unwrap_or_else(|| "NULL".to_string()),
        );

        let tag_strings: Vec<String> = self.tags().iter().map(super::types::Tag::to_str).collect();
        context.insert("tags", &tag_strings);
        context.insert("epic_id", "NULL");

        context.insert(
            "source_reference_architecture",
            &self
                .source_reference_architecture
                .as_ref()
                .map(|s| format!("\"{s}\""))
                .unwrap_or_else(|| "NULL".to_string()),
        );
        context.insert("locked", &self.locked.to_string());
        context.insert(
            "baseline_score",
            &self
                .baseline_score
                .map(|s| s.to_string())
                .unwrap_or_else(|| "null".to_string()),
        );
        context.insert(
            "completion_score",
            &self
                .completion_score
                .map(|s| s.to_string())
                .unwrap_or_else(|| "null".to_string()),
        );
        context.insert("drift_tolerance", &self.drift_tolerance.to_string());

        context.insert("relevant_layers", &self.relevant_layers);
        context.insert("relevant_boundaries", &self.relevant_boundaries);
        context.insert(
            "applicable_dependency_rules",
            &self.applicable_dependency_rules,
        );
        context.insert(
            "applicable_naming_conventions",
            &self.applicable_naming_conventions,
        );
        context.insert("applicable_anti_patterns", &self.applicable_anti_patterns);

        // Serialize checklist for Tera
        let checklist_data: Vec<std::collections::HashMap<String, serde_json::Value>> = self
            .checklist
            .iter()
            .map(|item| {
                let mut map = std::collections::HashMap::new();
                map.insert(
                    "question".to_string(),
                    serde_json::Value::String(item.question.clone()),
                );
                map.insert(
                    "answer".to_string(),
                    match &item.answer {
                        Some(a) => serde_json::Value::String(format!("\"{a}\"")),
                        None => serde_json::Value::String("null".to_string()),
                    },
                );
                let st: Vec<String> = item.story_types.iter().map(std::string::ToString::to_string).collect();
                map.insert("story_types".to_string(), serde_json::json!(st));
                map
            })
            .collect();
        context.insert("checklist", &checklist_data);

        // Serialize unlock history for Tera
        let unlock_data: Vec<std::collections::HashMap<String, String>> = self
            .unlock_history
            .iter()
            .map(|record| {
                let mut map = std::collections::HashMap::new();
                map.insert("actor".to_string(), record.actor.clone());
                map.insert("timestamp".to_string(), record.timestamp.to_rfc3339());
                map.insert("reason".to_string(), record.reason.clone());
                map
            })
            .collect();
        context.insert("unlock_history", &unlock_data);

        let frontmatter = tera.render("frontmatter", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Frontmatter render error: {e}"))
        })?;

        let content_body = &self.content().body;
        let acceptance_criteria = if let Some(ac) = &self.content().acceptance_criteria {
            format!("\n\n## Acceptance Criteria\n\n{ac}")
        } else {
            String::new()
        };

        Ok(format!(
            "---\n{}\n---\n\n{}{}",
            frontmatter.trim_end(),
            content_body,
            acceptance_criteria
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_architecture(title: &str, short_code: &str) -> Architecture {
        Architecture::new(
            title.to_string(),
            short_code.to_string(),
            Some("PROJ-S-0001".to_string()),
            Some("PROJ-RA-0001".to_string()),
        )
        .unwrap()
    }

    #[test]
    fn test_creation() {
        let arch = make_architecture("Auth Story Architecture", "PROJ-AR-0001");

        assert_eq!(arch.title(), "Auth Story Architecture");
        assert_eq!(arch.phase().unwrap(), Phase::Published);
        assert!(!arch.archived());
        assert!(arch.locked);
        assert_eq!(arch.parent_id(), Some("PROJ-S-0001"));
        assert_eq!(
            arch.source_reference_architecture.as_deref(),
            Some("PROJ-RA-0001")
        );
        assert_eq!(arch.drift_tolerance, 0.02);
        assert!(arch.baseline_score.is_none());
        assert!(arch.completion_score.is_none());
        assert!(arch.checklist.is_empty());
        assert!(arch.unlock_history.is_empty());
    }

    #[test]
    fn test_lock_unlock() {
        let mut arch = make_architecture("Lock Test", "PROJ-AR-0002");
        assert!(arch.locked);

        arch.unlock("alice".to_string(), "Updating layer info".to_string());
        assert!(!arch.locked);
        assert_eq!(arch.unlock_history.len(), 1);
        assert_eq!(arch.unlock_history[0].actor, "alice");
        assert_eq!(arch.unlock_history[0].reason, "Updating layer info");

        arch.lock();
        assert!(arch.locked);

        // Second unlock
        arch.unlock("bob".to_string(), "Adding checklist answer".to_string());
        assert_eq!(arch.unlock_history.len(), 2);
    }

    #[test]
    fn test_checklist_feature() {
        let items = Architecture::checklist_for_story_type(StoryType::Feature);
        assert_eq!(items.len(), 4);
        assert!(items[0].question.contains("cross-layer dependencies"));
        assert!(items[1].question.contains("module boundaries"));
    }

    #[test]
    fn test_checklist_bugfix() {
        let items = Architecture::checklist_for_story_type(StoryType::Bugfix);
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_checklist_refactor() {
        let items = Architecture::checklist_for_story_type(StoryType::Refactor);
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_checklist_architecture_change() {
        let items = Architecture::checklist_for_story_type(StoryType::ArchitectureChange);
        assert_eq!(items.len(), 7);
        assert!(items.iter().any(|i| i.question.contains("ADR")));
        assert!(items
            .iter()
            .any(|i| i.question.contains("ReferenceArchitecture")));
    }

    #[test]
    fn test_checklist_investigation() {
        let items = Architecture::checklist_for_story_type(StoryType::Investigation);
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_content_roundtrip() {
        let arch = Architecture::from_parts(
            "Roundtrip Test".to_string(),
            DocumentMetadata::new("PROJ-AR-0003".to_string()),
            DocumentContent::new("# Roundtrip Test\n\nArchitecture context."),
            vec![Tag::Phase(Phase::Published)],
            false,
            Some("PROJ-S-0002".to_string()),
            Some("PROJ-RA-0001".to_string()),
            vec!["presentation".to_string(), "domain".to_string()],
            vec!["components".to_string()],
            vec!["no circular imports".to_string()],
            vec!["PascalCase for components".to_string()],
            vec!["god components".to_string()],
            vec![ChecklistItem {
                question: "Does this cross boundaries?".to_string(),
                answer: Some("Yes, the auth module".to_string()),
                story_types: vec![StoryType::Feature],
            }],
            true,
            Some(0.95),
            None,
            0.02,
            vec![UnlockRecord {
                actor: "alice".to_string(),
                timestamp: Utc::now(),
                reason: "Initial review".to_string(),
            }],
        );

        let serialized = arch.to_content().unwrap();
        let loaded = Architecture::from_content(&serialized).unwrap();

        assert_eq!(loaded.title(), arch.title());
        assert_eq!(loaded.parent_id(), arch.parent_id());
        assert_eq!(
            loaded.source_reference_architecture,
            arch.source_reference_architecture
        );
        assert_eq!(loaded.relevant_layers, arch.relevant_layers);
        assert_eq!(loaded.relevant_boundaries, arch.relevant_boundaries);
        assert_eq!(
            loaded.applicable_dependency_rules,
            arch.applicable_dependency_rules
        );
        assert_eq!(
            loaded.applicable_naming_conventions,
            arch.applicable_naming_conventions
        );
        assert_eq!(
            loaded.applicable_anti_patterns,
            arch.applicable_anti_patterns
        );
        assert_eq!(loaded.locked, arch.locked);
        assert_eq!(loaded.baseline_score, arch.baseline_score);
        assert_eq!(loaded.completion_score, arch.completion_score);
        assert_eq!(loaded.drift_tolerance, arch.drift_tolerance);
        assert_eq!(loaded.checklist.len(), 1);
        assert_eq!(loaded.checklist[0].question, "Does this cross boundaries?");
        assert_eq!(
            loaded.checklist[0].answer.as_deref(),
            Some("Yes, the auth module")
        );
        assert_eq!(loaded.unlock_history.len(), 1);
        assert_eq!(loaded.unlock_history[0].actor, "alice");
    }

    #[tokio::test]
    async fn test_file_roundtrip() {
        let arch = Architecture::from_parts(
            "File Test".to_string(),
            DocumentMetadata::new("PROJ-AR-0004".to_string()),
            DocumentContent::new("# File Test"),
            vec![Tag::Phase(Phase::Published)],
            false,
            Some("PROJ-S-0003".to_string()),
            None,
            vec!["core".to_string()],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            true,
            None,
            None,
            0.05,
            vec![],
        );

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-architecture.md");

        arch.to_file(&file_path).await.unwrap();
        let loaded = Architecture::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), arch.title());
        assert_eq!(loaded.parent_id(), arch.parent_id());
        assert_eq!(loaded.relevant_layers, arch.relevant_layers);
        assert_eq!(loaded.drift_tolerance, 0.05);
    }

    #[test]
    fn test_from_content_invalid_level() {
        let bad_content = "---\n\
id: test\n\
level: rules_config\n\
title: \"Bad Level\"\n\
short_code: \"PROJ-AR-0099\"\n\
created_at: 2026-01-01T00:00:00Z\n\
updated_at: 2026-01-01T00:00:00Z\n\
archived: false\n\
parent_id: NULL\n\
tags:\n\
  - \"#phase/published\"\n\
exit_criteria_met: false\n\
schema_version: 1\n\
epic_id: NULL\n\
source_reference_architecture: NULL\n\
locked: true\n\
baseline_score: null\n\
completion_score: null\n\
drift_tolerance: 0.02\n\
relevant_layers: []\n\
relevant_boundaries: []\n\
applicable_dependency_rules: []\n\
applicable_naming_conventions: []\n\
applicable_anti_patterns: []\n\
checklist: []\n\
unlock_history: []\n\
---\n\
\n\
# Bad Level\n";
        let err = Architecture::from_content(bad_content).unwrap_err();
        assert!(matches!(err, DocumentValidationError::InvalidContent(_)));
    }
}
