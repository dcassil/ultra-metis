use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{DocumentId, Phase, Tag};
use chrono::Utc;
use gray_matter;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use tera::{Context, Tera};

/// Severity of a quality gate — determines whether failure blocks transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GateSeverity {
    /// Gate failure blocks the transition.
    Blocking,
    /// Gate failure is reported but does not block.
    Advisory,
}

impl fmt::Display for GateSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GateSeverity::Blocking => write!(f, "blocking"),
            GateSeverity::Advisory => write!(f, "advisory"),
        }
    }
}

impl FromStr for GateSeverity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "blocking" | "block" => Ok(GateSeverity::Blocking),
            "advisory" | "warn" | "warning" => Ok(GateSeverity::Advisory),
            _ => Err(format!("Unknown gate severity: {}", s)),
        }
    }
}

/// Type of threshold check to apply to a metric.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThresholdType {
    /// Metric value must not exceed this absolute value.
    Absolute(f64),
    /// Metric must not regress by more than this percentage relative to baseline.
    RelativeRegression(f64),
    /// Metric must be trending in the specified direction.
    Trend(TrendRequirement),
}

impl ThresholdType {
    /// Serialize to a type string for frontmatter.
    pub fn type_name(&self) -> &'static str {
        match self {
            ThresholdType::Absolute(_) => "absolute",
            ThresholdType::RelativeRegression(_) => "relative",
            ThresholdType::Trend(_) => "trend",
        }
    }

    /// Get the numeric value for serialization (trend uses 0.0 as placeholder).
    pub fn value(&self) -> f64 {
        match self {
            ThresholdType::Absolute(v) => *v,
            ThresholdType::RelativeRegression(v) => *v,
            ThresholdType::Trend(_) => 0.0,
        }
    }

    /// Parse from type name and value.
    pub fn from_parts(type_name: &str, value: f64) -> Result<Self, String> {
        match type_name.to_lowercase().as_str() {
            "absolute" | "abs" => Ok(ThresholdType::Absolute(value)),
            "relative" | "relative_regression" | "rel" => {
                Ok(ThresholdType::RelativeRegression(value))
            }
            "trend" => {
                // value > 0 means improving required, value == 0 means stable ok
                let requirement = if value > 0.0 {
                    TrendRequirement::Improving
                } else {
                    TrendRequirement::NotRegressing
                };
                Ok(ThresholdType::Trend(requirement))
            }
            _ => Err(format!("Unknown threshold type: {}", type_name)),
        }
    }
}

/// What trend direction is required for a trend-based threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendRequirement {
    /// Metric must be improving (getting better).
    Improving,
    /// Metric must not be regressing (stable or improving is OK).
    NotRegressing,
}

impl fmt::Display for TrendRequirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrendRequirement::Improving => write!(f, "improving"),
            TrendRequirement::NotRegressing => write!(f, "not_regressing"),
        }
    }
}

/// A single gate rule: associates a metric name with a threshold and severity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricGateRule {
    /// Name of the metric to check (e.g., "lint_errors", "test_coverage").
    pub metric: String,
    /// The threshold to apply.
    pub threshold: ThresholdType,
    /// Whether failure is blocking or advisory.
    pub severity: GateSeverity,
}

impl MetricGateRule {
    pub fn new(metric: &str, threshold: ThresholdType, severity: GateSeverity) -> Self {
        Self {
            metric: metric.to_string(),
            threshold,
            severity,
        }
    }

    /// Create a blocking absolute threshold rule.
    pub fn blocking_absolute(metric: &str, max_value: f64) -> Self {
        Self::new(
            metric,
            ThresholdType::Absolute(max_value),
            GateSeverity::Blocking,
        )
    }

    /// Create a blocking relative regression threshold rule.
    pub fn blocking_relative(metric: &str, max_regression_pct: f64) -> Self {
        Self::new(
            metric,
            ThresholdType::RelativeRegression(max_regression_pct),
            GateSeverity::Blocking,
        )
    }

    /// Create an advisory absolute threshold rule.
    pub fn advisory_absolute(metric: &str, max_value: f64) -> Self {
        Self::new(
            metric,
            ThresholdType::Absolute(max_value),
            GateSeverity::Advisory,
        )
    }
}

/// Gate configuration override for a specific phase transition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitionGateConfig {
    /// The phase being transitioned from.
    pub from_phase: String,
    /// The phase being transitioned to.
    pub to_phase: String,
    /// Thresholds that override the defaults for this transition.
    pub thresholds: Vec<MetricGateRule>,
}

impl TransitionGateConfig {
    pub fn new(from_phase: &str, to_phase: &str, thresholds: Vec<MetricGateRule>) -> Self {
        Self {
            from_phase: from_phase.to_string(),
            to_phase: to_phase.to_string(),
            thresholds,
        }
    }

    /// Check if this override applies to a given transition.
    pub fn matches(&self, from: &str, to: &str) -> bool {
        self.from_phase == from && self.to_phase == to
    }
}

/// Quality gate configuration for a project.
///
/// Defines default thresholds and per-transition overrides for quality metrics.
/// This is a governance type (does not implement the Document trait).
#[derive(Debug)]
pub struct QualityGateConfig {
    core: DocumentCore,
    /// Default severity for gates without an explicit severity.
    pub gate_severity_default: GateSeverity,
    /// Default threshold rules applied to all transitions.
    pub default_thresholds: Vec<MetricGateRule>,
    /// Per-transition overrides (stricter or relaxed thresholds for specific transitions).
    pub transition_overrides: Vec<TransitionGateConfig>,
}

impl QualityGateConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        gate_severity_default: GateSeverity,
        default_thresholds: Vec<MetricGateRule>,
        transition_overrides: Vec<TransitionGateConfig>,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            tags,
            archived,
            short_code,
            gate_severity_default,
            default_thresholds,
            transition_overrides,
            template_content,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_template(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        gate_severity_default: GateSeverity,
        default_thresholds: Vec<MetricGateRule>,
        transition_overrides: Vec<TransitionGateConfig>,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("quality_gate_config_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("quality_gate_config_content", &context)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template render error: {}", e))
            })?;

        let content = DocumentContent::new(&rendered_content);

        Ok(Self {
            core: DocumentCore {
                title,
                metadata,
                content,
                parent_id: None,
                blocked_by: Vec::new(),
                tags,
                archived,
                epic_id: None,
                schema_version: 1,
            },
            gate_severity_default,
            default_thresholds,
            transition_overrides,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        gate_severity_default: GateSeverity,
        default_thresholds: Vec<MetricGateRule>,
        transition_overrides: Vec<TransitionGateConfig>,
    ) -> Self {
        Self {
            core: DocumentCore {
                title,
                metadata,
                content,
                parent_id: None,
                blocked_by: Vec::new(),
                tags,
                archived,
                epic_id: None,
                schema_version: 1,
            },
            gate_severity_default,
            default_thresholds,
            transition_overrides,
        }
    }

    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e))
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

        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "quality_gate_config" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'quality_gate_config', found '{}'",
                level
            )));
        }

        let title = FrontmatterParser::extract_string(&fm_map, "title")?;
        let archived = FrontmatterParser::extract_bool(&fm_map, "archived").unwrap_or(false);
        let created_at = FrontmatterParser::extract_datetime(&fm_map, "created_at")?;
        let updated_at = FrontmatterParser::extract_datetime(&fm_map, "updated_at")?;
        let exit_criteria_met =
            FrontmatterParser::extract_bool(&fm_map, "exit_criteria_met").unwrap_or(false);
        let tags = FrontmatterParser::extract_tags(&fm_map)?;
        let short_code = FrontmatterParser::extract_string(&fm_map, "short_code")?;

        let gate_severity_default = FrontmatterParser::extract_optional_string(
            &fm_map,
            "gate_severity_default",
        )
        .and_then(|s| GateSeverity::from_str(&s).ok())
        .unwrap_or(GateSeverity::Blocking);

        let default_thresholds = Self::parse_threshold_array(&fm_map, "default_thresholds")?;
        let transition_overrides = Self::parse_transition_overrides(&fm_map)?;

        let metadata =
            DocumentMetadata::from_frontmatter(created_at, updated_at, exit_criteria_met, short_code);
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            tags,
            archived,
            gate_severity_default,
            default_thresholds,
            transition_overrides,
        ))
    }

    fn parse_threshold_array(
        fm_map: &std::collections::HashMap<String, gray_matter::Pod>,
        key: &str,
    ) -> Result<Vec<MetricGateRule>, DocumentValidationError> {
        let arr = match fm_map.get(key) {
            Some(gray_matter::Pod::Array(arr)) => arr,
            Some(_) => {
                return Err(DocumentValidationError::InvalidContent(format!(
                    "{} must be an array",
                    key
                )))
            }
            None => return Ok(Vec::new()),
        };

        let mut rules = Vec::new();
        for item in arr {
            if let gray_matter::Pod::Hash(map) = item {
                let metric = match map.get("metric") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => continue,
                };
                let threshold_type = match map.get("threshold_type") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => continue,
                };
                let threshold_value = match map.get("threshold_value") {
                    Some(gray_matter::Pod::Float(f)) => *f,
                    Some(gray_matter::Pod::Integer(i)) => *i as f64,
                    _ => continue,
                };
                let severity = match map.get("severity") {
                    Some(gray_matter::Pod::String(s)) => {
                        GateSeverity::from_str(s).unwrap_or(GateSeverity::Blocking)
                    }
                    _ => GateSeverity::Blocking,
                };

                let threshold = ThresholdType::from_parts(&threshold_type, threshold_value)
                    .map_err(|e| DocumentValidationError::InvalidContent(e))?;

                rules.push(MetricGateRule {
                    metric,
                    threshold,
                    severity,
                });
            }
        }
        Ok(rules)
    }

    fn parse_transition_overrides(
        fm_map: &std::collections::HashMap<String, gray_matter::Pod>,
    ) -> Result<Vec<TransitionGateConfig>, DocumentValidationError> {
        let arr = match fm_map.get("transition_overrides") {
            Some(gray_matter::Pod::Array(arr)) => arr,
            Some(_) => {
                return Err(DocumentValidationError::InvalidContent(
                    "transition_overrides must be an array".to_string(),
                ))
            }
            None => return Ok(Vec::new()),
        };

        let mut overrides = Vec::new();
        for item in arr {
            if let gray_matter::Pod::Hash(map) = item {
                let from_phase = match map.get("from_phase") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => continue,
                };
                let to_phase = match map.get("to_phase") {
                    Some(gray_matter::Pod::String(s)) => s.clone(),
                    _ => continue,
                };

                let thresholds_arr = match map.get("thresholds") {
                    Some(gray_matter::Pod::Array(arr)) => arr,
                    _ => continue,
                };

                let mut thresholds = Vec::new();
                for t_item in thresholds_arr {
                    if let gray_matter::Pod::Hash(t_map) = t_item {
                        let metric = match t_map.get("metric") {
                            Some(gray_matter::Pod::String(s)) => s.clone(),
                            _ => continue,
                        };
                        let threshold_type = match t_map.get("threshold_type") {
                            Some(gray_matter::Pod::String(s)) => s.clone(),
                            _ => continue,
                        };
                        let threshold_value = match t_map.get("threshold_value") {
                            Some(gray_matter::Pod::Float(f)) => *f,
                            Some(gray_matter::Pod::Integer(i)) => *i as f64,
                            _ => continue,
                        };
                        let severity = match t_map.get("severity") {
                            Some(gray_matter::Pod::String(s)) => {
                                GateSeverity::from_str(s).unwrap_or(GateSeverity::Blocking)
                            }
                            _ => GateSeverity::Blocking,
                        };

                        let threshold =
                            ThresholdType::from_parts(&threshold_type, threshold_value)
                                .map_err(|e| DocumentValidationError::InvalidContent(e))?;

                        thresholds.push(MetricGateRule {
                            metric,
                            threshold,
                            severity,
                        });
                    }
                }

                overrides.push(TransitionGateConfig {
                    from_phase,
                    to_phase,
                    thresholds,
                });
            }
        }
        Ok(overrides)
    }

    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to write file: {}", e))
        })
    }

    pub fn to_content(&self) -> Result<String, DocumentValidationError> {
        let mut tera = Tera::default();
        tera.add_raw_template("frontmatter", include_str!("frontmatter.yaml"))
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("slug", &self.id().to_string());
        context.insert("title", self.title());
        context.insert("short_code", &self.core.metadata.short_code);
        context.insert("created_at", &self.core.metadata.created_at.to_rfc3339());
        context.insert("updated_at", &self.core.metadata.updated_at.to_rfc3339());
        context.insert("archived", &self.core.archived.to_string());
        context.insert(
            "exit_criteria_met",
            &self.core.metadata.exit_criteria_met.to_string(),
        );
        context.insert("gate_severity_default", &self.gate_severity_default.to_string());

        // Serialize default thresholds as maps for Tera
        let threshold_maps: Vec<std::collections::HashMap<&str, String>> = self
            .default_thresholds
            .iter()
            .map(|r| {
                let mut m = std::collections::HashMap::new();
                m.insert("metric", r.metric.clone());
                m.insert("threshold_type", r.threshold.type_name().to_string());
                m.insert("threshold_value", format!("{}", r.threshold.value()));
                m.insert("severity", r.severity.to_string());
                m
            })
            .collect();
        context.insert("default_thresholds", &threshold_maps);

        // Serialize transition overrides
        let override_maps: Vec<serde_json::Value> = self
            .transition_overrides
            .iter()
            .map(|o| {
                let thresholds: Vec<serde_json::Value> = o
                    .thresholds
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "metric": t.metric,
                            "threshold_type": t.threshold.type_name(),
                            "threshold_value": format!("{}", t.threshold.value()),
                            "severity": t.severity.to_string(),
                        })
                    })
                    .collect();
                serde_json::json!({
                    "from_phase": o.from_phase,
                    "to_phase": o.to_phase,
                    "thresholds": thresholds,
                })
            })
            .collect();
        context.insert("transition_overrides", &override_maps);

        let tag_strings: Vec<String> = self.core.tags.iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);

        let frontmatter = tera.render("frontmatter", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Frontmatter render error: {}", e))
        })?;

        let content_body = &self.core.content.body;
        let acceptance_criteria = if let Some(ac) = &self.core.content.acceptance_criteria {
            format!("\n\n## Acceptance Criteria\n\n{}", ac)
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

    /// Get the effective thresholds for a given transition.
    /// Returns the transition-specific overrides if they exist, otherwise the defaults.
    pub fn thresholds_for_transition(
        &self,
        from_phase: &str,
        to_phase: &str,
    ) -> &[MetricGateRule] {
        for override_config in &self.transition_overrides {
            if override_config.matches(from_phase, to_phase) {
                return &override_config.thresholds;
            }
        }
        &self.default_thresholds
    }

    // Convenience accessors

    pub fn id(&self) -> DocumentId {
        DocumentId::from_title(&self.core.title)
    }

    pub fn title(&self) -> &str {
        &self.core.title
    }

    pub fn tags(&self) -> &[Tag] {
        &self.core.tags
    }

    pub fn archived(&self) -> bool {
        self.core.archived
    }

    pub fn phase(&self) -> Result<Phase, DocumentValidationError> {
        for tag in &self.core.tags {
            if let Tag::Phase(phase) = tag {
                return Ok(*phase);
            }
        }
        Err(DocumentValidationError::MissingPhaseTag)
    }

    fn update_phase_tag(&mut self, new_phase: Phase) {
        self.core.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
        self.core.tags.push(Tag::Phase(new_phase));
        self.core.metadata.updated_at = Utc::now();
    }

    pub fn validate(&self) -> Result<(), DocumentValidationError> {
        if self.core.title.trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "QualityGateConfig title cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_gate_config() -> QualityGateConfig {
        QualityGateConfig::new(
            "Project Quality Gates".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QGC-0001".to_string(),
            GateSeverity::Blocking,
            vec![
                MetricGateRule::blocking_absolute("lint_errors", 0.0),
                MetricGateRule::blocking_relative("test_coverage", 5.0),
                MetricGateRule::advisory_absolute("warnings", 20.0),
            ],
            vec![TransitionGateConfig::new(
                "active",
                "completed",
                vec![
                    MetricGateRule::blocking_absolute("lint_errors", 0.0),
                    MetricGateRule::blocking_absolute("warnings", 0.0),
                ],
            )],
        )
        .unwrap()
    }

    #[test]
    fn test_gate_config_creation() {
        let config = make_gate_config();

        assert_eq!(config.title(), "Project Quality Gates");
        assert_eq!(config.phase().unwrap(), Phase::Draft);
        assert_eq!(config.gate_severity_default, GateSeverity::Blocking);
        assert_eq!(config.default_thresholds.len(), 3);
        assert_eq!(config.transition_overrides.len(), 1);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_gate_config_empty_title_invalid() {
        let config = QualityGateConfig::new(
            String::new(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QGC-0002".to_string(),
            GateSeverity::Blocking,
            vec![],
            vec![],
        )
        .unwrap();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_gate_severity_parsing() {
        assert_eq!(
            "blocking".parse::<GateSeverity>().unwrap(),
            GateSeverity::Blocking
        );
        assert_eq!(
            "advisory".parse::<GateSeverity>().unwrap(),
            GateSeverity::Advisory
        );
        assert_eq!(
            "warn".parse::<GateSeverity>().unwrap(),
            GateSeverity::Advisory
        );
        assert!("invalid".parse::<GateSeverity>().is_err());
    }

    #[test]
    fn test_threshold_type_from_parts() {
        let abs = ThresholdType::from_parts("absolute", 10.0).unwrap();
        assert_eq!(abs, ThresholdType::Absolute(10.0));

        let rel = ThresholdType::from_parts("relative", 5.0).unwrap();
        assert_eq!(rel, ThresholdType::RelativeRegression(5.0));

        let trend = ThresholdType::from_parts("trend", 1.0).unwrap();
        assert_eq!(
            trend,
            ThresholdType::Trend(TrendRequirement::Improving)
        );

        let trend_stable = ThresholdType::from_parts("trend", 0.0).unwrap();
        assert_eq!(
            trend_stable,
            ThresholdType::Trend(TrendRequirement::NotRegressing)
        );

        assert!(ThresholdType::from_parts("unknown", 0.0).is_err());
    }

    #[test]
    fn test_metric_gate_rule_constructors() {
        let blocking = MetricGateRule::blocking_absolute("lint_errors", 0.0);
        assert_eq!(blocking.metric, "lint_errors");
        assert_eq!(blocking.severity, GateSeverity::Blocking);
        assert_eq!(blocking.threshold, ThresholdType::Absolute(0.0));

        let advisory = MetricGateRule::advisory_absolute("warnings", 20.0);
        assert_eq!(advisory.severity, GateSeverity::Advisory);

        let relative = MetricGateRule::blocking_relative("coverage", 5.0);
        assert_eq!(
            relative.threshold,
            ThresholdType::RelativeRegression(5.0)
        );
    }

    #[test]
    fn test_transition_gate_config_matches() {
        let tgc = TransitionGateConfig::new(
            "active",
            "completed",
            vec![MetricGateRule::blocking_absolute("lint_errors", 0.0)],
        );
        assert!(tgc.matches("active", "completed"));
        assert!(!tgc.matches("ready", "active"));
    }

    #[test]
    fn test_thresholds_for_transition_override() {
        let config = make_gate_config();

        // Transition with override
        let active_completed = config.thresholds_for_transition("active", "completed");
        assert_eq!(active_completed.len(), 2);

        // Transition without override — falls back to defaults
        let ready_active = config.thresholds_for_transition("ready", "active");
        assert_eq!(ready_active.len(), 3);
    }

    #[tokio::test]
    async fn test_gate_config_roundtrip() {
        let config = make_gate_config();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-gate-config.md");

        config.to_file(&file_path).await.unwrap();
        let loaded = QualityGateConfig::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), config.title());
        assert_eq!(loaded.phase().unwrap(), config.phase().unwrap());
        assert_eq!(loaded.gate_severity_default, config.gate_severity_default);
        assert_eq!(loaded.default_thresholds.len(), config.default_thresholds.len());
        assert_eq!(
            loaded.transition_overrides.len(),
            config.transition_overrides.len()
        );

        // Verify default threshold details
        for (loaded_t, orig_t) in loaded
            .default_thresholds
            .iter()
            .zip(config.default_thresholds.iter())
        {
            assert_eq!(loaded_t.metric, orig_t.metric);
            assert_eq!(loaded_t.severity, orig_t.severity);
        }

        // Verify transition override details
        let loaded_override = &loaded.transition_overrides[0];
        let orig_override = &config.transition_overrides[0];
        assert_eq!(loaded_override.from_phase, orig_override.from_phase);
        assert_eq!(loaded_override.to_phase, orig_override.to_phase);
        assert_eq!(
            loaded_override.thresholds.len(),
            orig_override.thresholds.len()
        );
    }
}
