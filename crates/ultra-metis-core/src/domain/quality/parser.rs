use super::types::ParsedToolOutput;

/// Trait for pluggable tool output parsers.
///
/// Each implementation handles a specific tool's output format
/// and produces a structured ParsedToolOutput.
pub trait ToolOutputParser {
    /// The name of the tool this parser handles (e.g., "eslint", "clippy").
    fn tool_name(&self) -> &str;

    /// Parse raw tool output into structured data.
    fn parse(&self, input: &str) -> Result<ParsedToolOutput, ParserError>;
}

/// Errors that can occur during tool output parsing.
#[derive(Debug)]
pub enum ParserError {
    /// The input format is invalid or unexpected.
    InvalidFormat(String),
    /// A required field is missing from the input.
    MissingField(String),
    /// JSON deserialization failed.
    JsonError(serde_json::Error),
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParserError::MissingField(field) => write!(f, "Missing field: {}", field),
            ParserError::JsonError(e) => write!(f, "JSON parse error: {}", e),
        }
    }
}

impl std::error::Error for ParserError {}

impl From<serde_json::Error> for ParserError {
    fn from(e: serde_json::Error) -> Self {
        ParserError::JsonError(e)
    }
}
