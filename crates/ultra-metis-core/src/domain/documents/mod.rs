pub mod content;
pub mod factory;
pub mod helpers;
pub mod hierarchy;
pub mod metadata;
/// Document domain module
pub mod traits;
pub mod types;

// New Ultra-Metis planning types
pub mod design_context;
pub mod epic;
pub mod product_doc;
pub mod story;

// Governance types
pub mod analysis_baseline;
pub mod quality_record;
pub mod remediation_record;
pub mod rules_config;
pub mod validation_record;

// Retained types
pub mod adr;
pub mod specification;
pub mod task;

// Legacy types for migration
pub mod initiative;
pub mod vision;

// Governance types (lighter-weight, do not implement Document trait)
pub mod approval_record;
pub mod architecture_investigation;
pub mod constraint_record;
pub mod design_change_proposal;
pub mod ownership_map;
pub mod rule_change_proposal;
pub mod validation_policy;

// Quality gate types (lighter-weight, do not implement Document trait)
pub mod gate_override;
pub mod quality_gate_config;

// Architecture types (lighter-weight, do not implement Document trait)
pub mod architecture_catalog_entry;
pub mod reference_architecture;

// Memory types (lighter-weight, do not implement Document trait)
pub mod durable_insight_note;

// Execution/Traceability types (lighter-weight, do not implement Document trait)
pub mod cross_reference;
pub mod decision_record;
pub mod execution_record;
pub mod transition_record;
