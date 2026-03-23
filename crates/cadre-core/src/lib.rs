pub mod domain;
pub mod error;

// Planning types
pub use domain::documents::{
    adr::Adr,
    design_context::DesignContext,
    epic::Epic,
    hierarchy::HierarchyValidator,
    initiative::Initiative,
    product_doc::ProductDoc,
    specification::Specification,
    story::Story,
    task::Task,
    traits::{Document, DocumentValidationError},
    types::{Complexity, DocumentId, DocumentType, Phase, RiskLevel, StoryType, Tag},
    vision::Vision,
};

// Governance types
pub use domain::documents::{
    analysis_baseline::AnalysisBaseline, approval_record::ApprovalRecord,
    architecture_investigation::ArchitectureInvestigation, constraint_record::ConstraintRecord,
    design_change_proposal::DesignChangeProposal, ownership_map::OwnershipMap,
    quality_record::QualityRecord, remediation_record::RemediationRecord,
    rules_config::RulesConfig, validation_policy::ValidationPolicy,
    validation_record::ValidationRecord,
};

// Quality gate types
pub use domain::documents::gate_override::{GateOverride, GateOverrideAuditEntry, OverrideType};
pub use domain::documents::quality_gate_config::{
    GateSeverity, MetricGateRule, QualityGateConfig, ThresholdType, TransitionGateConfig,
    TrendRequirement,
};

// Architecture types
pub use domain::documents::architecture::Architecture;
pub use domain::documents::{
    architecture_catalog_entry::ArchitectureCatalogEntry,
    reference_architecture::{ArchitectureStatus, ReferenceArchitecture},
};

// Memory types
pub use domain::documents::durable_insight_note::{
    DurableInsightNote, FeedbackSignal, InsightCategory, InsightScope, NoteStatus, ReviewReason,
};

// Execution/Traceability types
pub use domain::documents::cross_reference::{
    CrossReference, CrossReferenceEntry, RelationshipType, TraceabilityIndex,
};
pub use domain::documents::decision_record::{Alternative, DecisionRecord};
pub use domain::documents::execution_record::{
    Disposition, EscalationEntry, ExecutionMode, ExecutionRecord, OverrideEntry, ToolEntry,
    ValidationEntry,
};
pub use domain::documents::transition_record::{CheckResult, TransitionRecord};

// Quality system
pub use domain::quality::{
    capture::BaselineCaptureService,
    comparison::{BaselineComparisonEngine, ComparisonResult},
    conformance::ArchitectureConformanceChecker,
    gate_engine::{GateCheckEngine, GateCheckResult, MetricCheckResult},
    parser::{ParserError, ToolOutputParser},
    parsers::clippy::ClippyParser,
    parsers::coverage::CoverageParser,
    parsers::eslint::EslintParser,
    parsers::typescript::TypeScriptParser,
    types::{FindingEntry, MetricDelta, MetricEntry, ParsedToolOutput, Severity, TrendDirection},
};

// Catalog system
pub use domain::catalog::brownfield_evaluator::evaluator::{
    BrownfieldEvaluator, EvaluationOutcome, EvaluationResult, EvaluatorConfig,
};
pub use domain::catalog::brownfield_evaluator::pattern_matcher::{
    MatchResult, PatternMatchScore, PatternMatcher,
};
pub use domain::catalog::brownfield_evaluator::structure_analyzer::{
    NamingConvention, StructureAnalysis, StructureAnalyzer, TestPattern,
};
pub use domain::catalog::builtin_entries;
pub use domain::catalog::custom_loader::{
    build_engine_with_custom, catalog_dir, load_custom_entries, CustomLoadError, CATALOG_DIR_NAME,
};
pub use domain::catalog::query_engine::{CatalogMatch, CatalogQuery, CatalogQueryEngine};
pub use domain::catalog::selection_flow::{
    SelectionFlow, SelectionOption, SelectionResult, TailoringOptions,
};

// Bootstrap system
pub use domain::bootstrap::init_flow::{
    BootstrapFlow, BootstrapResult, BootstrapSummary, InferredProjectType,
};
pub use domain::bootstrap::monorepo_detector::{
    DiscoveredPackage, MonorepoDetector, MonorepoInfo, MonorepoTool, PackageKind,
};
pub use domain::bootstrap::repo_scanner::{
    BuildTool, DetectedLanguage, PackageManager, RepoScanResult, RepoScanner,
};
pub use domain::bootstrap::tool_detector::{
    DetectedTool, ToolCategory as DevToolCategory, ToolDetectionResult, ToolDetector,
};

// Cognitive operation kernel
pub use domain::operations::loops::{Condition, LoopDefinition, LoopKind};
pub use domain::operations::operation::{
    CognitiveOperation, EscalationCondition, OperationSpec, OutputKind, ToolCategory,
};
pub use domain::operations::templates;
pub use domain::operations::workflow::{CompletionRule, LoopStep, WorkType, WorkflowTemplate};

// Governance: gates, escalation, autonomy
pub use domain::governance::autonomy::{
    AutonomyConfig, AutonomyMode, EscalationLevel, EvidenceLevel, ToleranceLevel,
};
pub use domain::governance::escalation::{
    EscalationDetector, EscalationEvent, EscalationSeverity, EscalationSignals,
    EscalationThresholds, EscalationTrigger,
};
pub use domain::governance::gates::{
    EvidenceRequirement, GateCheckOutcome, GateDefinition, GateFailureBehavior, GateType,
};

// Template registry
pub use domain::templates::{
    TemplateCategory, TemplateContext, TemplateError, TemplateRegistry, TemplateSet,
};

pub use error::Result;
