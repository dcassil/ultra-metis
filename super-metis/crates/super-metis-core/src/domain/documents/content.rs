use serde::{Deserialize, Serialize};

/// Document content containing the main body and acceptance criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    pub body: String,
    pub acceptance_criteria: Option<String>,
}

impl DocumentContent {
    pub fn new(body: &str) -> Self {
        Self {
            body: body.to_string(),
            acceptance_criteria: None,
        }
    }

    pub fn with_acceptance_criteria(body: &str, acceptance_criteria: &str) -> Self {
        Self {
            body: body.to_string(),
            acceptance_criteria: Some(acceptance_criteria.to_string()),
        }
    }

    pub fn from_markdown(content: &str) -> Self {
        if let Some(criteria_pos) = content.find("## Acceptance Criteria") {
            let body = content[..criteria_pos].trim().to_string();
            let acceptance_criteria = content[criteria_pos..].trim().to_string();
            Self {
                body,
                acceptance_criteria: Some(acceptance_criteria),
            }
        } else {
            Self::new(content)
        }
    }

    pub fn full_content(&self) -> String {
        match &self.acceptance_criteria {
            Some(criteria) => format!("{}\n\n{}", self.body, criteria),
            None => self.body.clone(),
        }
    }

    pub fn has_acceptance_criteria(&self) -> bool {
        self.acceptance_criteria.is_some()
    }
}
