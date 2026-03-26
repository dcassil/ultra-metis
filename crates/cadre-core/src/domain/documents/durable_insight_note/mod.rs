use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{DocumentCore, DocumentValidationError};
use super::types::{DocumentId, Phase, Tag};
use chrono::{DateTime, Utc};
use gray_matter;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use tera::{Context, Tera};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Status lifecycle for a durable insight note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NoteStatus {
    /// Note is actively served during scope-based fetches.
    Active,
    /// Note has been flagged for potential removal based on usage patterns.
    PruneCandidate,
    /// Note requires human review before any automated action.
    NeedsHumanReview,
    /// Note has been retired and excluded from active queries.
    Archived,
}

impl fmt::Display for NoteStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::PruneCandidate => write!(f, "prune_candidate"),
            Self::NeedsHumanReview => write!(f, "needs_human_review"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

impl FromStr for NoteStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "prune_candidate" | "prune-candidate" => Ok(Self::PruneCandidate),
            "needs_human_review" | "needs-human-review" => Ok(Self::NeedsHumanReview),
            "archived" => Ok(Self::Archived),
            _ => Err(format!("Unknown note status: {s}")),
        }
    }
}

/// Reason a note has been flagged for human review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReviewReason {
    /// Note conflicts with a documented architecture decision.
    ConflictsWithArchitecture,
    /// Note conflicts with another active note.
    ConflictsWithNote,
    /// The architecture may have changed since this note was written.
    StaleArchitecture,
    /// The note content itself may be outdated.
    StaleNoteContent,
    /// Automated pruning would be risky for this note.
    RiskyToAutoPrune,
}

impl fmt::Display for ReviewReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictsWithArchitecture => write!(f, "conflicts_with_architecture"),
            Self::ConflictsWithNote => write!(f, "conflicts_with_note"),
            Self::StaleArchitecture => write!(f, "stale_architecture"),
            Self::StaleNoteContent => write!(f, "stale_note_content"),
            Self::RiskyToAutoPrune => write!(f, "risky_to_auto_prune"),
        }
    }
}

impl FromStr for ReviewReason {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "conflicts_with_architecture" | "conflicts-with-architecture" => {
                Ok(Self::ConflictsWithArchitecture)
            }
            "conflicts_with_note" | "conflicts-with-note" => Ok(Self::ConflictsWithNote),
            "stale_architecture" | "stale-architecture" => Ok(Self::StaleArchitecture),
            "stale_note_content" | "stale-note-content" => Ok(Self::StaleNoteContent),
            "risky_to_auto_prune" | "risky-to-auto-prune" => Ok(Self::RiskyToAutoPrune),
            _ => Err(format!("Unknown review reason: {s}")),
        }
    }
}

/// Category of insight captured by a note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InsightCategory {
    HotspotWarning,
    RecurringFailure,
    MisleadingName,
    ValidationHint,
    LocalException,
    BoundaryWarning,
    SubsystemQuirk,
}

impl fmt::Display for InsightCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HotspotWarning => write!(f, "hotspot_warning"),
            Self::RecurringFailure => write!(f, "recurring_failure"),
            Self::MisleadingName => write!(f, "misleading_name"),
            Self::ValidationHint => write!(f, "validation_hint"),
            Self::LocalException => write!(f, "local_exception"),
            Self::BoundaryWarning => write!(f, "boundary_warning"),
            Self::SubsystemQuirk => write!(f, "subsystem_quirk"),
        }
    }
}

impl FromStr for InsightCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hotspot_warning" | "hotspot-warning" => Ok(Self::HotspotWarning),
            "recurring_failure" | "recurring-failure" => Ok(Self::RecurringFailure),
            "misleading_name" | "misleading-name" => Ok(Self::MisleadingName),
            "validation_hint" | "validation-hint" => Ok(Self::ValidationHint),
            "local_exception" | "local-exception" => Ok(Self::LocalException),
            "boundary_warning" | "boundary-warning" => Ok(Self::BoundaryWarning),
            "subsystem_quirk" | "subsystem-quirk" => Ok(Self::SubsystemQuirk),
            _ => Err(format!("Unknown insight category: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Scope
// ---------------------------------------------------------------------------

/// Defines the scope to which a durable insight note applies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InsightScope {
    /// Repository name (e.g., "cadre").
    pub repo: Option<String>,
    /// Package / crate name.
    pub package: Option<String>,
    /// Logical subsystem label.
    pub subsystem: Option<String>,
    /// File paths the insight is relevant to.
    pub paths: Vec<String>,
    /// Symbol names (functions, types, modules) the insight is relevant to.
    pub symbols: Vec<String>,
}

impl InsightScope {
    pub fn new() -> Self {
        Self {
            repo: None,
            package: None,
            subsystem: None,
            paths: Vec::new(),
            symbols: Vec::new(),
        }
    }

    /// Check if this scope matches a query scope (any overlap counts as a match).
    pub fn matches(&self, query: &Self) -> bool {
        if let (Some(r), Some(qr)) = (&self.repo, &query.repo) {
            if r == qr {
                return true;
            }
        }
        if let (Some(p), Some(qp)) = (&self.package, &query.package) {
            if p == qp {
                return true;
            }
        }
        if let (Some(s), Some(qs)) = (&self.subsystem, &query.subsystem) {
            if s == qs {
                return true;
            }
        }
        for path in &self.paths {
            for qpath in &query.paths {
                if path == qpath {
                    return true;
                }
            }
        }
        for sym in &self.symbols {
            for qsym in &query.symbols {
                if sym == qsym {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for InsightScope {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Feedback
// ---------------------------------------------------------------------------

/// Feedback signal recorded for a note after it was fetched during a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeedbackSignal {
    Helpful,
    Meh,
    Harmful,
}

impl fmt::Display for FeedbackSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Helpful => write!(f, "helpful"),
            Self::Meh => write!(f, "meh"),
            Self::Harmful => write!(f, "harmful"),
        }
    }
}

impl FromStr for FeedbackSignal {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "helpful" | "thumbs_up" | "good" => Ok(Self::Helpful),
            "meh" | "neutral" | "ok" => Ok(Self::Meh),
            "harmful" | "thumbs_down" | "bad" => Ok(Self::Harmful),
            _ => Err(format!("Unknown feedback signal: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// DurableInsightNote
// ---------------------------------------------------------------------------

/// A durable insight note — lightweight, scoped, self-pruning repo memory.
///
/// Notes capture reusable local knowledge: hotspot warnings, recurring failure
/// signatures, misleading names, validation hints, subsystem quirks. They are
/// fetched by scope at task start, scored at task wrap-up, and automatically
/// pruned when they stop proving useful.
///
/// This is a governance type (does not implement the Document trait).
#[derive(Debug)]
pub struct DurableInsightNote {
    core: DocumentCore,
    /// The insight text itself.
    pub note: String,
    /// Category of insight.
    pub category: InsightCategory,
    /// Scope to which this note applies.
    pub scope: InsightScope,
    /// Current lifecycle status.
    pub status: NoteStatus,
    /// Reason for human review (when status is NeedsHumanReview).
    pub review_reason: Option<ReviewReason>,
    /// Number of times this note has been fetched.
    pub fetch_count: u32,
    /// When this note was last fetched by a scope query.
    pub last_fetched_at: Option<DateTime<Utc>>,
    /// Number of "helpful" feedback signals.
    pub thumbs_up_count: u32,
    /// Number of "meh" feedback signals.
    pub meh_count: u32,
    /// Number of "harmful" feedback signals.
    pub thumbs_down_count: u32,
    /// When feedback was last recorded.
    pub last_feedback_at: Option<DateTime<Utc>>,
}

impl DurableInsightNote {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        note: String,
        category: InsightCategory,
        scope: InsightScope,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            note,
            category,
            scope,
            tags,
            archived,
            short_code,
            template_content,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_template(
        title: String,
        note: String,
        category: InsightCategory,
        scope: InsightScope,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("durable_insight_note_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {e}"))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera
            .render("durable_insight_note_content", &context)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template render error: {e}"))
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
            note,
            category,
            scope,
            status: NoteStatus::Active,
            review_reason: None,
            fetch_count: 0,
            last_fetched_at: None,
            thumbs_up_count: 0,
            meh_count: 0,
            thumbs_down_count: 0,
            last_feedback_at: None,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
        note: String,
        category: InsightCategory,
        scope: InsightScope,
        status: NoteStatus,
        review_reason: Option<ReviewReason>,
        fetch_count: u32,
        last_fetched_at: Option<DateTime<Utc>>,
        thumbs_up_count: u32,
        meh_count: u32,
        thumbs_down_count: u32,
        last_feedback_at: Option<DateTime<Utc>>,
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
            note,
            category,
            scope,
            status,
            review_reason,
            fetch_count,
            last_fetched_at,
            thumbs_up_count,
            meh_count,
            thumbs_down_count,
            last_feedback_at,
        }
    }

    // -----------------------------------------------------------------------
    // Fetch tracking
    // -----------------------------------------------------------------------

    /// Record that this note was fetched during a scope query.
    pub fn record_fetch(&mut self) {
        self.fetch_count += 1;
        self.last_fetched_at = Some(Utc::now());
        self.core.metadata.updated_at = Utc::now();
    }

    // -----------------------------------------------------------------------
    // Feedback scoring
    // -----------------------------------------------------------------------

    /// Record a feedback signal for this note.
    pub fn record_feedback(&mut self, signal: FeedbackSignal) {
        match signal {
            FeedbackSignal::Helpful => self.thumbs_up_count += 1,
            FeedbackSignal::Meh => self.meh_count += 1,
            FeedbackSignal::Harmful => self.thumbs_down_count += 1,
        }
        self.last_feedback_at = Some(Utc::now());
        self.core.metadata.updated_at = Utc::now();
    }

    /// Total number of feedback signals received.
    pub fn total_feedback(&self) -> u32 {
        self.thumbs_up_count + self.meh_count + self.thumbs_down_count
    }

    /// Ratio of harmful feedback to total feedback (0.0 if no feedback).
    pub fn harmful_ratio(&self) -> f64 {
        let total = self.total_feedback();
        if total == 0 {
            return 0.0;
        }
        f64::from(self.thumbs_down_count) / f64::from(total)
    }

    /// Ratio of helpful feedback to total feedback (0.0 if no feedback).
    pub fn helpful_ratio(&self) -> f64 {
        let total = self.total_feedback();
        if total == 0 {
            return 0.0;
        }
        f64::from(self.thumbs_up_count) / f64::from(total)
    }

    // -----------------------------------------------------------------------
    // Prune candidate detection
    // -----------------------------------------------------------------------

    /// Check if this note should be marked as a prune candidate.
    ///
    /// Criteria:
    /// - Not fetched for longer than `stale_days` days
    /// - Harmful ratio exceeds `harmful_threshold`
    /// - Meh accumulates without positive signal and total feedback >= `min_feedback`
    /// - Enough uses (`min_uses`) without demonstrated value (helpful_ratio < `value_threshold`)
    pub fn should_be_prune_candidate(
        &self,
        stale_days: i64,
        harmful_threshold: f64,
        min_feedback: u32,
        min_uses: u32,
        value_threshold: f64,
    ) -> bool {
        // Already not active — skip
        if self.status != NoteStatus::Active {
            return false;
        }

        // Check stale: not fetched for too long
        if let Some(last_fetched) = self.last_fetched_at {
            let days_since = (Utc::now() - last_fetched).num_days();
            if days_since >= stale_days {
                return true;
            }
        }

        // Check harmful ratio
        if self.total_feedback() > 0 && self.harmful_ratio() >= harmful_threshold {
            return true;
        }

        // Check meh accumulation without positive signal
        if self.total_feedback() >= min_feedback && self.thumbs_up_count == 0 && self.meh_count > 0
        {
            return true;
        }

        // Check enough uses without demonstrated value
        if self.fetch_count >= min_uses
            && self.total_feedback() >= min_feedback
            && self.helpful_ratio() < value_threshold
        {
            return true;
        }

        false
    }

    /// Mark this note as a prune candidate.
    pub fn mark_prune_candidate(&mut self) {
        self.status = NoteStatus::PruneCandidate;
        self.core.metadata.updated_at = Utc::now();
    }

    // -----------------------------------------------------------------------
    // Human review flagging
    // -----------------------------------------------------------------------

    /// Flag this note for human review with a reason.
    pub fn flag_for_review(&mut self, reason: ReviewReason) {
        self.status = NoteStatus::NeedsHumanReview;
        self.review_reason = Some(reason);
        self.core.metadata.updated_at = Utc::now();
    }

    // -----------------------------------------------------------------------
    // Archival
    // -----------------------------------------------------------------------

    /// Archive this note.
    pub fn archive(&mut self) {
        self.status = NoteStatus::Archived;
        self.core.archived = true;
        self.core.metadata.updated_at = Utc::now();
    }

    // -----------------------------------------------------------------------
    // Serialization
    // -----------------------------------------------------------------------

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

        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "durable_insight_note" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'durable_insight_note', found '{level}'"
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

        let category = FrontmatterParser::extract_optional_string(&fm_map, "category")
            .and_then(|s| InsightCategory::from_str(&s).ok())
            .unwrap_or(InsightCategory::SubsystemQuirk);

        let status = FrontmatterParser::extract_optional_string(&fm_map, "status")
            .and_then(|s| NoteStatus::from_str(&s).ok())
            .unwrap_or(NoteStatus::Active);

        let review_reason = FrontmatterParser::extract_optional_string(&fm_map, "review_reason")
            .and_then(|s| ReviewReason::from_str(&s).ok());

        let scope = Self::parse_scope(&fm_map)?;

        let fetch_count =
            FrontmatterParser::extract_integer(&fm_map, "fetch_count").unwrap_or(0) as u32;

        let last_fetched_at =
            FrontmatterParser::extract_optional_string(&fm_map, "last_fetched_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

        let thumbs_up_count =
            FrontmatterParser::extract_integer(&fm_map, "thumbs_up_count").unwrap_or(0) as u32;
        let meh_count =
            FrontmatterParser::extract_integer(&fm_map, "meh_count").unwrap_or(0) as u32;
        let thumbs_down_count =
            FrontmatterParser::extract_integer(&fm_map, "thumbs_down_count").unwrap_or(0) as u32;

        let last_feedback_at =
            FrontmatterParser::extract_optional_string(&fm_map, "last_feedback_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

        // Extract note from the content body (first non-heading paragraph)
        let note = parsed.content.trim().to_string();

        let metadata = DocumentMetadata::from_frontmatter(
            created_at,
            updated_at,
            exit_criteria_met,
            short_code,
        );
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            tags,
            archived,
            note,
            category,
            scope,
            status,
            review_reason,
            fetch_count,
            last_fetched_at,
            thumbs_up_count,
            meh_count,
            thumbs_down_count,
            last_feedback_at,
        ))
    }

    fn parse_scope(
        fm_map: &std::collections::HashMap<String, gray_matter::Pod>,
    ) -> Result<InsightScope, DocumentValidationError> {
        let scope_map = match fm_map.get("scope") {
            Some(gray_matter::Pod::Hash(map)) => map,
            Some(_) => {
                return Err(DocumentValidationError::InvalidContent(
                    "scope must be a hash/map".to_string(),
                ))
            }
            None => return Ok(InsightScope::new()),
        };

        let repo = match scope_map.get("repo") {
            Some(gray_matter::Pod::String(s)) if !s.is_empty() && s != "NULL" => Some(s.clone()),
            _ => None,
        };
        let package = match scope_map.get("package") {
            Some(gray_matter::Pod::String(s)) if !s.is_empty() && s != "NULL" => Some(s.clone()),
            _ => None,
        };
        let subsystem = match scope_map.get("subsystem") {
            Some(gray_matter::Pod::String(s)) if !s.is_empty() && s != "NULL" => Some(s.clone()),
            _ => None,
        };

        let paths = match scope_map.get("paths") {
            Some(gray_matter::Pod::Array(arr)) => arr
                .iter()
                .filter_map(|item| {
                    if let gray_matter::Pod::String(s) = item {
                        if !s.is_empty() && s != "NULL" {
                            Some(s.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect(),
            _ => Vec::new(),
        };

        let symbols = match scope_map.get("symbols") {
            Some(gray_matter::Pod::Array(arr)) => arr
                .iter()
                .filter_map(|item| {
                    if let gray_matter::Pod::String(s) = item {
                        if !s.is_empty() && s != "NULL" {
                            Some(s.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect(),
            _ => Vec::new(),
        };

        Ok(InsightScope {
            repo,
            package,
            subsystem,
            paths,
            symbols,
        })
    }

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

        context.insert("category", &self.category.to_string());
        context.insert("status", &self.status.to_string());
        context.insert(
            "review_reason",
            &self
                .review_reason
                .map(|r| r.to_string())
                .unwrap_or_else(|| "NULL".to_string()),
        );

        context.insert("scope_repo", &self.scope.repo.as_deref().unwrap_or("NULL"));
        context.insert(
            "scope_package",
            &self.scope.package.as_deref().unwrap_or("NULL"),
        );
        context.insert(
            "scope_subsystem",
            &self.scope.subsystem.as_deref().unwrap_or("NULL"),
        );
        context.insert("scope_paths", &self.scope.paths);
        context.insert("scope_symbols", &self.scope.symbols);

        context.insert("fetch_count", &self.fetch_count);
        context.insert(
            "last_fetched_at",
            &self
                .last_fetched_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "NULL".to_string()),
        );
        context.insert("thumbs_up_count", &self.thumbs_up_count);
        context.insert("meh_count", &self.meh_count);
        context.insert("thumbs_down_count", &self.thumbs_down_count);
        context.insert(
            "last_feedback_at",
            &self
                .last_feedback_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "NULL".to_string()),
        );

        let tag_strings: Vec<String> = self.core.tags.iter().map(super::types::Tag::to_str).collect();
        context.insert("tags", &tag_strings);

        let frontmatter = tera.render("frontmatter", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Frontmatter render error: {e}"))
        })?;

        let content_body = &self.core.content.body;
        let acceptance_criteria = if let Some(ac) = &self.core.content.acceptance_criteria {
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

    // -----------------------------------------------------------------------
    // Convenience accessors
    // -----------------------------------------------------------------------

    pub fn id(&self) -> DocumentId {
        DocumentId::from_title(&self.core.title)
    }

    pub fn title(&self) -> &str {
        &self.core.title
    }

    pub fn metadata(&self) -> &DocumentMetadata {
        &self.core.metadata
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

    /// Get mutable access to the document core
    pub fn core_mut(&mut self) -> &mut DocumentCore {
        &mut self.core
    }

    pub fn validate(&self) -> Result<(), DocumentValidationError> {
        if self.core.title.trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "DurableInsightNote title cannot be empty".to_string(),
            ));
        }
        if self.note.trim().is_empty() {
            return Err(DocumentValidationError::InvalidContent(
                "DurableInsightNote must have non-empty note text".to_string(),
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_scope() -> InsightScope {
        InsightScope {
            repo: Some("cadre".to_string()),
            package: Some("cadre-core".to_string()),
            subsystem: Some("documents".to_string()),
            paths: vec!["src/domain/documents/mod.rs".to_string()],
            symbols: vec!["DocumentCore".to_string()],
        }
    }

    fn make_note() -> DurableInsightNote {
        DurableInsightNote::new(
            "DocumentCore fields are private".to_string(),
            "The DocumentCore struct fields are pub but the struct itself is only constructed internally. Do not try to directly instantiate it from outside the documents module.".to_string(),
            InsightCategory::SubsystemQuirk,
            make_scope(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DIN-0001".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn test_note_creation() {
        let note = make_note();
        assert_eq!(note.title(), "DocumentCore fields are private");
        assert_eq!(note.category, InsightCategory::SubsystemQuirk);
        assert_eq!(note.status, NoteStatus::Active);
        assert_eq!(note.fetch_count, 0);
        assert_eq!(note.thumbs_up_count, 0);
        assert!(note.validate().is_ok());
    }

    #[test]
    fn test_note_empty_title_invalid() {
        let note = DurableInsightNote::new(
            String::new(),
            "Some note".to_string(),
            InsightCategory::HotspotWarning,
            InsightScope::new(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DIN-0002".to_string(),
        )
        .unwrap();
        assert!(note.validate().is_err());
    }

    #[test]
    fn test_note_empty_note_text_invalid() {
        let note = DurableInsightNote::new(
            "Some title".to_string(),
            String::new(),
            InsightCategory::HotspotWarning,
            InsightScope::new(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-DIN-0003".to_string(),
        )
        .unwrap();
        assert!(note.validate().is_err());
    }

    #[test]
    fn test_note_status_parsing() {
        assert_eq!("active".parse::<NoteStatus>().unwrap(), NoteStatus::Active);
        assert_eq!(
            "prune_candidate".parse::<NoteStatus>().unwrap(),
            NoteStatus::PruneCandidate
        );
        assert_eq!(
            "prune-candidate".parse::<NoteStatus>().unwrap(),
            NoteStatus::PruneCandidate
        );
        assert_eq!(
            "needs_human_review".parse::<NoteStatus>().unwrap(),
            NoteStatus::NeedsHumanReview
        );
        assert_eq!(
            "archived".parse::<NoteStatus>().unwrap(),
            NoteStatus::Archived
        );
        assert!("invalid".parse::<NoteStatus>().is_err());
    }

    #[test]
    fn test_review_reason_parsing() {
        assert_eq!(
            "conflicts_with_architecture"
                .parse::<ReviewReason>()
                .unwrap(),
            ReviewReason::ConflictsWithArchitecture
        );
        assert_eq!(
            "stale_note_content".parse::<ReviewReason>().unwrap(),
            ReviewReason::StaleNoteContent
        );
        assert_eq!(
            "risky-to-auto-prune".parse::<ReviewReason>().unwrap(),
            ReviewReason::RiskyToAutoPrune
        );
        assert!("invalid".parse::<ReviewReason>().is_err());
    }

    #[test]
    fn test_insight_category_parsing() {
        assert_eq!(
            "hotspot_warning".parse::<InsightCategory>().unwrap(),
            InsightCategory::HotspotWarning
        );
        assert_eq!(
            "recurring-failure".parse::<InsightCategory>().unwrap(),
            InsightCategory::RecurringFailure
        );
        assert_eq!(
            "misleading_name".parse::<InsightCategory>().unwrap(),
            InsightCategory::MisleadingName
        );
        assert_eq!(
            "validation_hint".parse::<InsightCategory>().unwrap(),
            InsightCategory::ValidationHint
        );
        assert_eq!(
            "subsystem_quirk".parse::<InsightCategory>().unwrap(),
            InsightCategory::SubsystemQuirk
        );
        assert!("invalid".parse::<InsightCategory>().is_err());
    }

    #[test]
    fn test_feedback_signal_parsing() {
        assert_eq!(
            "helpful".parse::<FeedbackSignal>().unwrap(),
            FeedbackSignal::Helpful
        );
        assert_eq!(
            "meh".parse::<FeedbackSignal>().unwrap(),
            FeedbackSignal::Meh
        );
        assert_eq!(
            "harmful".parse::<FeedbackSignal>().unwrap(),
            FeedbackSignal::Harmful
        );
        assert!("invalid".parse::<FeedbackSignal>().is_err());
    }

    #[test]
    fn test_fetch_tracking() {
        let mut note = make_note();
        assert_eq!(note.fetch_count, 0);
        assert!(note.last_fetched_at.is_none());

        note.record_fetch();
        assert_eq!(note.fetch_count, 1);
        assert!(note.last_fetched_at.is_some());

        note.record_fetch();
        assert_eq!(note.fetch_count, 2);
    }

    #[test]
    fn test_feedback_scoring() {
        let mut note = make_note();
        assert_eq!(note.total_feedback(), 0);
        assert_eq!(note.harmful_ratio(), 0.0);

        note.record_feedback(FeedbackSignal::Helpful);
        note.record_feedback(FeedbackSignal::Helpful);
        note.record_feedback(FeedbackSignal::Meh);
        note.record_feedback(FeedbackSignal::Harmful);

        assert_eq!(note.thumbs_up_count, 2);
        assert_eq!(note.meh_count, 1);
        assert_eq!(note.thumbs_down_count, 1);
        assert_eq!(note.total_feedback(), 4);
        assert!((note.harmful_ratio() - 0.25).abs() < f64::EPSILON);
        assert!((note.helpful_ratio() - 0.5).abs() < f64::EPSILON);
        assert!(note.last_feedback_at.is_some());
    }

    #[test]
    fn test_scope_matching() {
        let scope = make_scope();

        // Match by repo
        let mut query = InsightScope::new();
        query.repo = Some("cadre".to_string());
        assert!(scope.matches(&query));

        // Match by package
        let mut query = InsightScope::new();
        query.package = Some("cadre-core".to_string());
        assert!(scope.matches(&query));

        // Match by symbol
        let mut query = InsightScope::new();
        query.symbols = vec!["DocumentCore".to_string()];
        assert!(scope.matches(&query));

        // No match
        let mut query = InsightScope::new();
        query.repo = Some("other-repo".to_string());
        assert!(!scope.matches(&query));

        // Empty scopes don't match
        let query = InsightScope::new();
        assert!(!scope.matches(&query));
    }

    #[test]
    fn test_prune_candidate_harmful_ratio() {
        let mut note = make_note();
        // Record feedback: 3 harmful out of 4
        note.record_feedback(FeedbackSignal::Helpful);
        note.record_feedback(FeedbackSignal::Harmful);
        note.record_feedback(FeedbackSignal::Harmful);
        note.record_feedback(FeedbackSignal::Harmful);

        // harmful_threshold = 0.5 => 0.75 > 0.5 => should prune
        assert!(note.should_be_prune_candidate(30, 0.5, 3, 5, 0.3));
    }

    #[test]
    fn test_prune_candidate_meh_without_positive() {
        let mut note = make_note();
        note.record_feedback(FeedbackSignal::Meh);
        note.record_feedback(FeedbackSignal::Meh);
        note.record_feedback(FeedbackSignal::Meh);

        // min_feedback = 3, 0 thumbs_up, meh > 0 => should prune
        assert!(note.should_be_prune_candidate(30, 0.5, 3, 5, 0.3));
    }

    #[test]
    fn test_prune_candidate_low_value() {
        let mut note = make_note();
        note.fetch_count = 10;
        note.record_feedback(FeedbackSignal::Meh);
        note.record_feedback(FeedbackSignal::Meh);
        note.record_feedback(FeedbackSignal::Meh);
        note.record_feedback(FeedbackSignal::Helpful);

        // fetch_count=10 >= min_uses=5, total_feedback=4 >= min_feedback=3,
        // helpful_ratio = 0.25 < value_threshold=0.3 => should prune
        assert!(note.should_be_prune_candidate(30, 0.5, 3, 5, 0.3));
    }

    #[test]
    fn test_not_prune_candidate_when_healthy() {
        let mut note = make_note();
        note.record_fetch();
        note.record_feedback(FeedbackSignal::Helpful);
        note.record_feedback(FeedbackSignal::Helpful);
        note.record_feedback(FeedbackSignal::Meh);

        // healthy: high helpful ratio, recently fetched
        assert!(!note.should_be_prune_candidate(30, 0.5, 3, 5, 0.3));
    }

    #[test]
    fn test_flag_for_review() {
        let mut note = make_note();
        note.flag_for_review(ReviewReason::ConflictsWithArchitecture);
        assert_eq!(note.status, NoteStatus::NeedsHumanReview);
        assert_eq!(
            note.review_reason,
            Some(ReviewReason::ConflictsWithArchitecture)
        );
    }

    #[test]
    fn test_archive() {
        let mut note = make_note();
        note.archive();
        assert_eq!(note.status, NoteStatus::Archived);
        assert!(note.archived());
    }

    #[test]
    fn test_mark_prune_candidate() {
        let mut note = make_note();
        note.mark_prune_candidate();
        assert_eq!(note.status, NoteStatus::PruneCandidate);
    }

    #[tokio::test]
    async fn test_note_roundtrip() {
        let note = make_note();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-note.md");

        note.to_file(&file_path).await.unwrap();
        let loaded = DurableInsightNote::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.title(), note.title());
        assert_eq!(loaded.category, note.category);
        assert_eq!(loaded.status, note.status);
        assert_eq!(loaded.fetch_count, note.fetch_count);
        assert_eq!(loaded.thumbs_up_count, note.thumbs_up_count);
        assert_eq!(loaded.meh_count, note.meh_count);
        assert_eq!(loaded.thumbs_down_count, note.thumbs_down_count);
        assert_eq!(loaded.scope.repo, note.scope.repo);
        assert_eq!(loaded.scope.package, note.scope.package);
        assert_eq!(loaded.scope.subsystem, note.scope.subsystem);
        assert_eq!(loaded.scope.paths, note.scope.paths);
        assert_eq!(loaded.scope.symbols, note.scope.symbols);
    }

    #[tokio::test]
    async fn test_note_roundtrip_with_feedback() {
        let mut note = make_note();
        note.record_fetch();
        note.record_fetch();
        note.record_feedback(FeedbackSignal::Helpful);
        note.record_feedback(FeedbackSignal::Meh);
        note.flag_for_review(ReviewReason::StaleNoteContent);

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-note-feedback.md");

        note.to_file(&file_path).await.unwrap();
        let loaded = DurableInsightNote::from_file(&file_path).await.unwrap();

        assert_eq!(loaded.fetch_count, 2);
        assert_eq!(loaded.thumbs_up_count, 1);
        assert_eq!(loaded.meh_count, 1);
        assert_eq!(loaded.thumbs_down_count, 0);
        assert_eq!(loaded.status, NoteStatus::NeedsHumanReview);
        assert_eq!(loaded.review_reason, Some(ReviewReason::StaleNoteContent));
        assert!(loaded.last_fetched_at.is_some());
        assert!(loaded.last_feedback_at.is_some());
    }
}
