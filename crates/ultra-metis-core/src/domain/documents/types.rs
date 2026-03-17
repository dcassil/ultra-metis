use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Maximum length for document IDs
const MAX_ID_LENGTH: usize = 35;

/// Document identifier - always derived from title as a slug
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(String);

impl DocumentId {
    /// Create a new DocumentId from a raw string
    pub fn new(id: &str) -> Self {
        let capped_id = if id.chars().count() > MAX_ID_LENGTH {
            id.chars().take(MAX_ID_LENGTH).collect::<String>()
        } else {
            id.to_string()
        };
        Self(capped_id)
    }

    /// Create a DocumentId from a title by converting to slug
    pub fn from_title(title: &str) -> Self {
        let slug = Self::title_to_slug(title);
        Self::new(&slug)
    }

    /// Convert title to URL-friendly slug
    pub fn title_to_slug(title: &str) -> String {
        let slug = title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        if slug.chars().count() > MAX_ID_LENGTH {
            let truncated: String = slug.chars().take(MAX_ID_LENGTH).collect();
            if let Some(last_dash) = truncated.rfind('-') {
                if last_dash > MAX_ID_LENGTH / 2 {
                    return truncated[..last_dash].to_string();
                }
            }
            truncated
        } else {
            slug
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DocumentId {
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

impl From<&str> for DocumentId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Parent reference for documents in flexible hierarchy
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParentReference {
    Some(DocumentId),
    None,
    Null,
}

impl ParentReference {
    pub fn to_path_string(&self) -> String {
        match self {
            ParentReference::Some(id) => id.as_str().to_string(),
            ParentReference::None => "root".to_string(),
            ParentReference::Null => "NULL".to_string(),
        }
    }

    pub fn has_parent(&self) -> bool {
        matches!(self, ParentReference::Some(_))
    }

    pub fn parent_id(&self) -> Option<&DocumentId> {
        match self {
            ParentReference::Some(id) => Some(id),
            _ => None,
        }
    }

    pub fn from_option(id: Option<DocumentId>) -> Self {
        match id {
            Some(id) => ParentReference::Some(id),
            None => ParentReference::None,
        }
    }

    pub fn null() -> Self {
        ParentReference::Null
    }
}

impl From<DocumentId> for ParentReference {
    fn from(id: DocumentId) -> Self {
        ParentReference::Some(id)
    }
}

impl From<Option<DocumentId>> for ParentReference {
    fn from(opt: Option<DocumentId>) -> Self {
        ParentReference::from_option(opt)
    }
}

impl fmt::Display for ParentReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_path_string())
    }
}

/// Document type enumeration — Ultra-Metis engineering-oriented hierarchy
///
/// New types: ProductDoc, DesignContext, Epic, Story
/// Retained: Task, Adr, Specification
/// Retained for migration: Vision, Initiative
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentType {
    // New Ultra-Metis planning hierarchy
    ProductDoc,
    DesignContext,
    Epic,
    Story,

    // Retained from Metis
    Task,
    Adr,
    Specification,

    // Retained for backward compatibility / migration
    Vision,
    Initiative,
}

impl fmt::Display for DocumentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DocumentType::ProductDoc => write!(f, "product_doc"),
            DocumentType::DesignContext => write!(f, "design_context"),
            DocumentType::Epic => write!(f, "epic"),
            DocumentType::Story => write!(f, "story"),
            DocumentType::Task => write!(f, "task"),
            DocumentType::Adr => write!(f, "adr"),
            DocumentType::Specification => write!(f, "specification"),
            DocumentType::Vision => write!(f, "vision"),
            DocumentType::Initiative => write!(f, "initiative"),
        }
    }
}

impl FromStr for DocumentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "product_doc" | "productdoc" | "product-doc" => Ok(DocumentType::ProductDoc),
            "design_context" | "designcontext" | "design-context" => {
                Ok(DocumentType::DesignContext)
            }
            "epic" => Ok(DocumentType::Epic),
            "story" => Ok(DocumentType::Story),
            "task" => Ok(DocumentType::Task),
            "adr" => Ok(DocumentType::Adr),
            "specification" => Ok(DocumentType::Specification),
            "vision" => Ok(DocumentType::Vision),
            "initiative" => Ok(DocumentType::Initiative),
            _ => Err(format!("Unknown document type: {}", s)),
        }
    }
}

impl DocumentType {
    /// Get the short code prefix letter(s) for this document type
    pub fn short_code_prefix(&self) -> &'static str {
        match self {
            DocumentType::ProductDoc => "PD",
            DocumentType::DesignContext => "DC",
            DocumentType::Epic => "E",
            DocumentType::Story => "S",
            DocumentType::Task => "T",
            DocumentType::Adr => "A",
            DocumentType::Specification => "SP",
            DocumentType::Vision => "V",
            DocumentType::Initiative => "I",
        }
    }

    /// Get valid transitions from a given phase for this document type.
    pub fn valid_transitions_from(&self, from_phase: Phase) -> Vec<Phase> {
        match self {
            DocumentType::ProductDoc => match from_phase {
                Phase::Draft => vec![Phase::Review],
                Phase::Review => vec![Phase::Published],
                _ => vec![],
            },
            DocumentType::DesignContext => match from_phase {
                Phase::Draft => vec![Phase::Review],
                Phase::Review => vec![Phase::Published],
                Phase::Published => vec![Phase::Superseded],
                _ => vec![],
            },
            DocumentType::Epic => match from_phase {
                Phase::Discovery => vec![Phase::Design],
                Phase::Design => vec![Phase::Ready],
                Phase::Ready => vec![Phase::Decompose],
                Phase::Decompose => vec![Phase::Active],
                Phase::Active => vec![Phase::Completed],
                _ => vec![],
            },
            DocumentType::Story => match from_phase {
                Phase::Discovery => vec![Phase::Design],
                Phase::Design => vec![Phase::Ready],
                Phase::Ready => vec![Phase::Active],
                Phase::Active => vec![Phase::Completed, Phase::Blocked],
                Phase::Blocked => vec![Phase::Ready, Phase::Active],
                _ => vec![],
            },
            DocumentType::Task => match from_phase {
                Phase::Backlog => vec![Phase::Todo],
                Phase::Todo => vec![Phase::Active, Phase::Blocked],
                Phase::Active => vec![Phase::Completed, Phase::Blocked],
                Phase::Blocked => vec![Phase::Todo, Phase::Active],
                _ => vec![],
            },
            DocumentType::Adr => match from_phase {
                Phase::Draft => vec![Phase::Discussion],
                Phase::Discussion => vec![Phase::Decided],
                Phase::Decided => vec![Phase::Superseded],
                _ => vec![],
            },
            DocumentType::Specification => match from_phase {
                Phase::Discovery => vec![Phase::Drafting],
                Phase::Drafting => vec![Phase::Review],
                Phase::Review => vec![Phase::Published],
                _ => vec![],
            },
            // Legacy types retained for migration
            DocumentType::Vision => match from_phase {
                Phase::Draft => vec![Phase::Review],
                Phase::Review => vec![Phase::Published],
                _ => vec![],
            },
            DocumentType::Initiative => match from_phase {
                Phase::Discovery => vec![Phase::Design],
                Phase::Design => vec![Phase::Ready],
                Phase::Ready => vec![Phase::Decompose],
                Phase::Decompose => vec![Phase::Active],
                Phase::Active => vec![Phase::Completed],
                _ => vec![],
            },
        }
    }

    /// Check if a transition from one phase to another is valid for this document type.
    pub fn can_transition(&self, from: Phase, to: Phase) -> bool {
        self.valid_transitions_from(from).contains(&to)
    }

    /// Get the next phase in the natural sequence for this document type.
    pub fn next_phase(&self, current: Phase) -> Option<Phase> {
        self.valid_transitions_from(current).first().copied()
    }

    /// Get the ordered phase sequence for this document type (for display purposes).
    pub fn phase_sequence(&self) -> Vec<Phase> {
        match self {
            DocumentType::ProductDoc => vec![Phase::Draft, Phase::Review, Phase::Published],
            DocumentType::DesignContext => {
                vec![Phase::Draft, Phase::Review, Phase::Published, Phase::Superseded]
            }
            DocumentType::Epic => vec![
                Phase::Discovery,
                Phase::Design,
                Phase::Ready,
                Phase::Decompose,
                Phase::Active,
                Phase::Completed,
            ],
            DocumentType::Story => vec![
                Phase::Discovery,
                Phase::Design,
                Phase::Ready,
                Phase::Active,
                Phase::Completed,
            ],
            DocumentType::Task => vec![
                Phase::Backlog,
                Phase::Todo,
                Phase::Active,
                Phase::Completed,
            ],
            DocumentType::Adr => vec![
                Phase::Draft,
                Phase::Discussion,
                Phase::Decided,
                Phase::Superseded,
            ],
            DocumentType::Specification => {
                vec![Phase::Discovery, Phase::Drafting, Phase::Review, Phase::Published]
            }
            DocumentType::Vision => vec![Phase::Draft, Phase::Review, Phase::Published],
            DocumentType::Initiative => vec![
                Phase::Discovery,
                Phase::Design,
                Phase::Ready,
                Phase::Decompose,
                Phase::Active,
                Phase::Completed,
            ],
        }
    }

    /// Returns true if this is a new Ultra-Metis type (not legacy)
    pub fn is_ultra_metis_type(&self) -> bool {
        matches!(
            self,
            DocumentType::ProductDoc
                | DocumentType::DesignContext
                | DocumentType::Epic
                | DocumentType::Story
        )
    }

    /// Returns true if this is a legacy Metis type retained for migration
    pub fn is_legacy_type(&self) -> bool {
        matches!(self, DocumentType::Vision | DocumentType::Initiative)
    }
}

/// Story type — categorizes the purpose of a Story within an Epic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StoryType {
    Feature,
    Bugfix,
    Refactor,
    Migration,
    ArchitectureChange,
    Investigation,
    Remediation,
    Setup,
}

impl fmt::Display for StoryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StoryType::Feature => write!(f, "feature"),
            StoryType::Bugfix => write!(f, "bugfix"),
            StoryType::Refactor => write!(f, "refactor"),
            StoryType::Migration => write!(f, "migration"),
            StoryType::ArchitectureChange => write!(f, "architecture-change"),
            StoryType::Investigation => write!(f, "investigation"),
            StoryType::Remediation => write!(f, "remediation"),
            StoryType::Setup => write!(f, "setup"),
        }
    }
}

impl FromStr for StoryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "feature" => Ok(StoryType::Feature),
            "bugfix" | "bug-fix" | "bug_fix" => Ok(StoryType::Bugfix),
            "refactor" => Ok(StoryType::Refactor),
            "migration" => Ok(StoryType::Migration),
            "architecture-change" | "architecture_change" | "arch-change" => {
                Ok(StoryType::ArchitectureChange)
            }
            "investigation" => Ok(StoryType::Investigation),
            "remediation" => Ok(StoryType::Remediation),
            "setup" | "bootstrap" | "setup/bootstrap" => Ok(StoryType::Setup),
            _ => Err(format!("Unknown story type: {}", s)),
        }
    }
}

/// Document phase/status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Phase {
    // Shared phases
    Draft,
    Review,
    Published,

    // ADR / DesignContext phases
    Discussion,
    Decided,
    Superseded,

    // Task phases
    Backlog,
    Todo,
    Active,
    Blocked,
    Completed,

    // Epic / Initiative / Story phases
    Design,
    Ready,
    Decompose,
    Discovery,

    // Specification phases
    Drafting,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Phase::Draft => write!(f, "draft"),
            Phase::Review => write!(f, "review"),
            Phase::Published => write!(f, "published"),
            Phase::Discussion => write!(f, "discussion"),
            Phase::Decided => write!(f, "decided"),
            Phase::Superseded => write!(f, "superseded"),
            Phase::Backlog => write!(f, "backlog"),
            Phase::Todo => write!(f, "todo"),
            Phase::Active => write!(f, "active"),
            Phase::Blocked => write!(f, "blocked"),
            Phase::Completed => write!(f, "completed"),
            Phase::Design => write!(f, "design"),
            Phase::Ready => write!(f, "ready"),
            Phase::Decompose => write!(f, "decompose"),
            Phase::Discovery => write!(f, "discovery"),
            Phase::Drafting => write!(f, "drafting"),
        }
    }
}

impl FromStr for Phase {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(Phase::Draft),
            "review" => Ok(Phase::Review),
            "published" => Ok(Phase::Published),
            "discussion" => Ok(Phase::Discussion),
            "decided" => Ok(Phase::Decided),
            "superseded" => Ok(Phase::Superseded),
            "backlog" => Ok(Phase::Backlog),
            "todo" => Ok(Phase::Todo),
            "active" => Ok(Phase::Active),
            "blocked" => Ok(Phase::Blocked),
            "completed" => Ok(Phase::Completed),
            "design" => Ok(Phase::Design),
            "ready" => Ok(Phase::Ready),
            "decompose" => Ok(Phase::Decompose),
            "discovery" => Ok(Phase::Discovery),
            "drafting" => Ok(Phase::Drafting),
            _ => Err(format!("Unknown phase: {}", s)),
        }
    }
}

/// Document tag that can be either a phase or a string label
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tag {
    Phase(Phase),
    Label(String),
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tag::Phase(phase) => write!(f, "#phase/{}", phase),
            Tag::Label(label) => {
                if label.starts_with('#') {
                    write!(f, "{}", label)
                } else {
                    write!(f, "#{}", label)
                }
            }
        }
    }
}

impl From<Phase> for Tag {
    fn from(phase: Phase) -> Self {
        Tag::Phase(phase)
    }
}

impl From<String> for Tag {
    fn from(label: String) -> Self {
        Tag::Label(label)
    }
}

impl From<&str> for Tag {
    fn from(label: &str) -> Self {
        Tag::Label(label.to_string())
    }
}

impl FromStr for Tag {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(phase_str) = s.strip_prefix("#phase/") {
            match phase_str.parse::<Phase>() {
                Ok(phase) => Ok(Tag::Phase(phase)),
                Err(_) => Err(()),
            }
        } else if let Some(stripped) = s.strip_prefix('#') {
            Ok(Tag::Label(stripped.to_string()))
        } else {
            Ok(Tag::Label(s.to_string()))
        }
    }
}

impl Tag {
    pub fn to_str(&self) -> String {
        match self {
            Tag::Phase(phase) => format!("#phase/{}", phase),
            Tag::Label(label) => {
                if label.starts_with('#') {
                    label.clone()
                } else {
                    format!("#{}", label)
                }
            }
        }
    }
}

/// Complexity level for epics (replacing initiative complexity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Complexity {
    XS,
    S,
    M,
    L,
    XL,
}

impl fmt::Display for Complexity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Complexity::XS => write!(f, "XS"),
            Complexity::S => write!(f, "S"),
            Complexity::M => write!(f, "M"),
            Complexity::L => write!(f, "L"),
            Complexity::XL => write!(f, "XL"),
        }
    }
}

impl FromStr for Complexity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "XS" => Ok(Complexity::XS),
            "S" => Ok(Complexity::S),
            "M" => Ok(Complexity::M),
            "L" => Ok(Complexity::L),
            "XL" => Ok(Complexity::XL),
            _ => Err(format!("Invalid complexity: {}", s)),
        }
    }
}

/// Risk level for planning artifacts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "low"),
            RiskLevel::Medium => write!(f, "medium"),
            RiskLevel::High => write!(f, "high"),
            RiskLevel::Critical => write!(f, "critical"),
        }
    }
}

impl FromStr for RiskLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(RiskLevel::Low),
            "medium" | "med" => Ok(RiskLevel::Medium),
            "high" => Ok(RiskLevel::High),
            "critical" | "crit" => Ok(RiskLevel::Critical),
            _ => Err(format!("Invalid risk level: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_to_slug() {
        assert_eq!(
            DocumentId::title_to_slug("Core Document Management Library"),
            "core-document-management-library"
        );
        assert_eq!(
            DocumentId::title_to_slug("ADR-001: Document Format"),
            "adr-001-document-format"
        );
    }

    #[test]
    fn test_id_length_capping() {
        let very_long_title = "This is an extremely long title that should definitely exceed our maximum identifier length limit and needs to be truncated appropriately without breaking";
        let id = DocumentId::from_title(very_long_title);
        assert!(id.as_str().len() <= MAX_ID_LENGTH);
    }

    #[test]
    fn test_document_type_short_code_prefix() {
        assert_eq!(DocumentType::ProductDoc.short_code_prefix(), "PD");
        assert_eq!(DocumentType::DesignContext.short_code_prefix(), "DC");
        assert_eq!(DocumentType::Epic.short_code_prefix(), "E");
        assert_eq!(DocumentType::Story.short_code_prefix(), "S");
        assert_eq!(DocumentType::Task.short_code_prefix(), "T");
        assert_eq!(DocumentType::Vision.short_code_prefix(), "V");
        assert_eq!(DocumentType::Initiative.short_code_prefix(), "I");
    }

    #[test]
    fn test_document_type_parsing() {
        assert_eq!("product_doc".parse::<DocumentType>().unwrap(), DocumentType::ProductDoc);
        assert_eq!("product-doc".parse::<DocumentType>().unwrap(), DocumentType::ProductDoc);
        assert_eq!("design_context".parse::<DocumentType>().unwrap(), DocumentType::DesignContext);
        assert_eq!("epic".parse::<DocumentType>().unwrap(), DocumentType::Epic);
        assert_eq!("story".parse::<DocumentType>().unwrap(), DocumentType::Story);
        assert_eq!("vision".parse::<DocumentType>().unwrap(), DocumentType::Vision);
        assert_eq!("initiative".parse::<DocumentType>().unwrap(), DocumentType::Initiative);
    }

    #[test]
    fn test_story_type_parsing() {
        assert_eq!("feature".parse::<StoryType>().unwrap(), StoryType::Feature);
        assert_eq!("bugfix".parse::<StoryType>().unwrap(), StoryType::Bugfix);
        assert_eq!("bug-fix".parse::<StoryType>().unwrap(), StoryType::Bugfix);
        assert_eq!("refactor".parse::<StoryType>().unwrap(), StoryType::Refactor);
        assert_eq!("migration".parse::<StoryType>().unwrap(), StoryType::Migration);
        assert_eq!("architecture-change".parse::<StoryType>().unwrap(), StoryType::ArchitectureChange);
        assert_eq!("investigation".parse::<StoryType>().unwrap(), StoryType::Investigation);
        assert_eq!("remediation".parse::<StoryType>().unwrap(), StoryType::Remediation);
        assert_eq!("setup".parse::<StoryType>().unwrap(), StoryType::Setup);
    }

    #[test]
    fn test_product_doc_transitions() {
        assert_eq!(
            DocumentType::ProductDoc.valid_transitions_from(Phase::Draft),
            vec![Phase::Review]
        );
        assert_eq!(
            DocumentType::ProductDoc.valid_transitions_from(Phase::Review),
            vec![Phase::Published]
        );
        assert!(DocumentType::ProductDoc
            .valid_transitions_from(Phase::Published)
            .is_empty());
    }

    #[test]
    fn test_epic_transitions() {
        assert_eq!(
            DocumentType::Epic.valid_transitions_from(Phase::Discovery),
            vec![Phase::Design]
        );
        assert_eq!(
            DocumentType::Epic.valid_transitions_from(Phase::Decompose),
            vec![Phase::Active]
        );
        assert!(DocumentType::Epic.can_transition(Phase::Discovery, Phase::Design));
        assert!(!DocumentType::Epic.can_transition(Phase::Discovery, Phase::Active));
    }

    #[test]
    fn test_story_transitions() {
        assert_eq!(
            DocumentType::Story.valid_transitions_from(Phase::Active),
            vec![Phase::Completed, Phase::Blocked]
        );
        assert_eq!(
            DocumentType::Story.valid_transitions_from(Phase::Blocked),
            vec![Phase::Ready, Phase::Active]
        );
        // Story has no Decompose phase
        assert!(!DocumentType::Story
            .phase_sequence()
            .contains(&Phase::Decompose));
    }

    #[test]
    fn test_design_context_transitions() {
        assert!(DocumentType::DesignContext.can_transition(Phase::Published, Phase::Superseded));
        assert!(!DocumentType::DesignContext.can_transition(Phase::Draft, Phase::Published));
    }

    #[test]
    fn test_tag_roundtrip() {
        let tags = vec![
            Tag::Phase(Phase::Draft),
            Tag::Phase(Phase::Completed),
            Tag::Label("urgent".to_string()),
        ];
        for tag in tags {
            let str_repr = tag.to_str();
            let parsed_back = str_repr.parse::<Tag>().unwrap();
            assert_eq!(tag, parsed_back);
        }
    }

    #[test]
    fn test_complexity_parsing() {
        assert_eq!("XS".parse::<Complexity>().unwrap(), Complexity::XS);
        assert_eq!("xl".parse::<Complexity>().unwrap(), Complexity::XL);
        assert!("invalid".parse::<Complexity>().is_err());
    }

    #[test]
    fn test_risk_level_parsing() {
        assert_eq!("low".parse::<RiskLevel>().unwrap(), RiskLevel::Low);
        assert_eq!("critical".parse::<RiskLevel>().unwrap(), RiskLevel::Critical);
        assert!("invalid".parse::<RiskLevel>().is_err());
    }

    #[test]
    fn test_is_ultra_metis_type() {
        assert!(DocumentType::ProductDoc.is_ultra_metis_type());
        assert!(DocumentType::Epic.is_ultra_metis_type());
        assert!(DocumentType::Story.is_ultra_metis_type());
        assert!(DocumentType::DesignContext.is_ultra_metis_type());
        assert!(!DocumentType::Vision.is_ultra_metis_type());
        assert!(!DocumentType::Task.is_ultra_metis_type());
    }

    #[test]
    fn test_is_legacy_type() {
        assert!(DocumentType::Vision.is_legacy_type());
        assert!(DocumentType::Initiative.is_legacy_type());
        assert!(!DocumentType::ProductDoc.is_legacy_type());
        assert!(!DocumentType::Task.is_legacy_type());
    }
}
