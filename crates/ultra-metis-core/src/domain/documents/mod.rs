pub mod content;
pub mod factory;
pub mod helpers;
pub mod hierarchy;
pub mod metadata;
/// Document domain module
pub mod traits;
pub mod types;

// New Ultra-Metis planning types
pub mod product_doc;
pub mod design_context;
pub mod epic;
pub mod story;

// Governance types
pub mod rules_config;
pub mod analysis_baseline;
pub mod quality_record;
pub mod validation_record;
pub mod remediation_record;

// Retained types
pub mod task;
pub mod adr;
pub mod specification;

// Legacy types for migration
pub mod vision;
pub mod initiative;

// Governance types (lighter-weight, do not implement Document trait)
pub mod approval_record;
pub mod constraint_record;
pub mod design_change_proposal;
pub mod rule_change_proposal;
pub mod architecture_investigation;
pub mod validation_policy;
pub mod ownership_map;

// Quality gate types (lighter-weight, do not implement Document trait)
pub mod quality_gate_config;
pub mod gate_override;

// Architecture types (lighter-weight, do not implement Document trait)
pub mod architecture_catalog_entry;
pub mod reference_architecture;

// Memory types (lighter-weight, do not implement Document trait)
pub mod durable_insight_note;

// Execution/Traceability types (lighter-weight, do not implement Document trait)
pub mod execution_record;
pub mod transition_record;
pub mod decision_record;
pub mod cross_reference;
