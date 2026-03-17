pub mod domain;
pub mod error;

// Planning types
pub use domain::documents::{
    adr::Adr,
    design_context::DesignContext,
    epic::Epic,
    initiative::Initiative,
    product_doc::ProductDoc,
    specification::Specification,
    story::Story,
    task::Task,
    traits::{Document, DocumentValidationError},
    types::{Complexity, DocumentId, DocumentType, Phase, RiskLevel, StoryType, Tag},
    vision::Vision,
    hierarchy::HierarchyValidator,
};

// Governance types
pub use domain::documents::{
    rules_config::RulesConfig,
    analysis_baseline::AnalysisBaseline,
    quality_record::QualityRecord,
    validation_record::ValidationRecord,
    remediation_record::RemediationRecord,
    approval_record::ApprovalRecord,
    constraint_record::ConstraintRecord,
    design_change_proposal::DesignChangeProposal,
    architecture_investigation::ArchitectureInvestigation,
    validation_policy::ValidationPolicy,
    ownership_map::OwnershipMap,
};

// Quality gate types
pub use domain::documents::gate_override::{
    GateOverride,
    GateOverrideAuditEntry,
    OverrideType,
};
pub use domain::documents::quality_gate_config::{
    GateSeverity,
    MetricGateRule,
    QualityGateConfig,
    ThresholdType,
    TransitionGateConfig,
    TrendRequirement,
};

// Architecture types
pub use domain::documents::{
    architecture_catalog_entry::ArchitectureCatalogEntry,
    reference_architecture::{ArchitectureStatus, ReferenceArchitecture},
};

// Memory types
pub use domain::documents::durable_insight_note::{
    DurableInsightNote,
    FeedbackSignal,
    InsightCategory,
    InsightScope,
    NoteStatus,
    ReviewReason,
};

// Execution/Traceability types
pub use domain::documents::execution_record::{
    Disposition,
    EscalationEntry,
    ExecutionMode,
    ExecutionRecord,
    OverrideEntry,
    ToolEntry,
    ValidationEntry,
};
pub use domain::documents::transition_record::{
    CheckResult,
    TransitionRecord,
};
pub use domain::documents::decision_record::{
    Alternative,
    DecisionRecord,
};
pub use domain::documents::cross_reference::{
    CrossReference,
    CrossReferenceEntry,
    RelationshipType,
    TraceabilityIndex,
};

// Quality system
pub use domain::quality::{
    types::{FindingEntry, MetricEntry, MetricDelta, ParsedToolOutput, Severity, TrendDirection},
    parser::{ParserError, ToolOutputParser},
    parsers::eslint::EslintParser,
    parsers::clippy::ClippyParser,
    parsers::typescript::TypeScriptParser,
    parsers::coverage::CoverageParser,
    capture::BaselineCaptureService,
    comparison::{BaselineComparisonEngine, ComparisonResult},
    conformance::ArchitectureConformanceChecker,
    gate_engine::{GateCheckEngine, GateCheckResult, MetricCheckResult},
};

// Catalog system
pub use domain::catalog::builtin_entries;
pub use domain::catalog::query_engine::{CatalogMatch, CatalogQuery, CatalogQueryEngine};
pub use domain::catalog::selection_flow::{
    SelectionFlow, SelectionOption, SelectionResult, TailoringOptions,
};
pub use domain::catalog::custom_loader::{
    build_engine_with_custom, load_custom_entries, catalog_dir, CustomLoadError, CATALOG_DIR_NAME,
};
pub use domain::catalog::brownfield_evaluator::structure_analyzer::{
    NamingConvention, StructureAnalysis, StructureAnalyzer, TestPattern,
};
pub use domain::catalog::brownfield_evaluator::pattern_matcher::{
    MatchResult, PatternMatchScore, PatternMatcher,
};
pub use domain::catalog::brownfield_evaluator::evaluator::{
    BrownfieldEvaluator, EvaluationOutcome, EvaluationResult, EvaluatorConfig,
};

// Bootstrap system
pub use domain::bootstrap::repo_scanner::{
    BuildTool, DetectedLanguage, PackageManager, RepoScanResult, RepoScanner,
};
pub use domain::bootstrap::monorepo_detector::{
    DiscoveredPackage, MonorepoDetector, MonorepoInfo, MonorepoTool, PackageKind,
};
pub use domain::bootstrap::tool_detector::{
    DetectedTool, ToolDetectionResult, ToolDetector,
    ToolCategory as DevToolCategory,
};
pub use domain::bootstrap::init_flow::{
    BootstrapFlow, BootstrapResult, BootstrapSummary, InferredProjectType,
};

// Cognitive operation kernel
pub use domain::operations::operation::{
    CognitiveOperation, EscalationCondition, OperationSpec, OutputKind, ToolCategory,
};
pub use domain::operations::loops::{Condition, LoopDefinition, LoopKind};
pub use domain::operations::workflow::{
    CompletionRule, LoopStep, WorkType, WorkflowTemplate,
};
pub use domain::operations::templates;

// Governance: gates, escalation, autonomy
pub use domain::governance::gates::{
    EvidenceRequirement, GateCheckOutcome, GateDefinition, GateFailureBehavior, GateType,
};
pub use domain::governance::escalation::{
    EscalationDetector, EscalationEvent, EscalationSeverity, EscalationSignals,
    EscalationThresholds, EscalationTrigger,
};
pub use domain::governance::autonomy::{
    AutonomyConfig, AutonomyMode, EscalationLevel, EvidenceLevel, ToleranceLevel,
};

// Template registry
pub use domain::templates::{
    TemplateCategory, TemplateContext, TemplateError, TemplateRegistry, TemplateSet,
};

pub use error::Result;
