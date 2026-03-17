use super::traits::DocumentValidationError;
use super::types::Tag;
use chrono::{DateTime, Utc};
use gray_matter;

/// Helper methods for parsing frontmatter
pub struct FrontmatterParser;

impl FrontmatterParser {
    pub fn extract_string(
        map: &std::collections::HashMap<String, gray_matter::Pod>,
        key: &str,
    ) -> Result<String, DocumentValidationError> {
        match map.get(key) {
            Some(gray_matter::Pod::String(s)) => Ok(s.clone()),
            Some(_) => Err(DocumentValidationError::InvalidContent(format!(
                "{} must be a string",
                key
            ))),
            None => Err(DocumentValidationError::MissingRequiredField(
                key.to_string(),
            )),
        }
    }

    pub fn extract_bool(
        map: &std::collections::HashMap<String, gray_matter::Pod>,
        key: &str,
    ) -> Result<bool, DocumentValidationError> {
        match map.get(key) {
            Some(gray_matter::Pod::Boolean(b)) => Ok(*b),
            Some(_) => Err(DocumentValidationError::InvalidContent(format!(
                "{} must be a boolean",
                key
            ))),
            None => Err(DocumentValidationError::MissingRequiredField(
                key.to_string(),
            )),
        }
    }

    pub fn extract_integer(
        map: &std::collections::HashMap<String, gray_matter::Pod>,
        key: &str,
    ) -> Result<i64, DocumentValidationError> {
        match map.get(key) {
            Some(gray_matter::Pod::Integer(i)) => Ok(*i),
            Some(_) => Err(DocumentValidationError::InvalidContent(format!(
                "{} must be an integer",
                key
            ))),
            None => Err(DocumentValidationError::MissingRequiredField(
                key.to_string(),
            )),
        }
    }

    pub fn extract_datetime(
        map: &std::collections::HashMap<String, gray_matter::Pod>,
        key: &str,
    ) -> Result<DateTime<Utc>, DocumentValidationError> {
        let date_str = Self::extract_string(map, key)?;
        DateTime::parse_from_rfc3339(&date_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| {
                DocumentValidationError::InvalidContent(format!(
                    "Invalid datetime format for {}",
                    key
                ))
            })
    }

    pub fn extract_tags(
        map: &std::collections::HashMap<String, gray_matter::Pod>,
    ) -> Result<Vec<Tag>, DocumentValidationError> {
        match map.get("tags") {
            Some(gray_matter::Pod::Array(arr)) => {
                let mut tags = Vec::new();
                for item in arr {
                    if let gray_matter::Pod::String(tag_str) = item {
                        if let Ok(tag) = tag_str.parse::<Tag>() {
                            tags.push(tag);
                        }
                    }
                }
                Ok(tags)
            }
            Some(_) => Err(DocumentValidationError::InvalidContent(
                "tags must be an array".to_string(),
            )),
            None => Err(DocumentValidationError::MissingRequiredField(
                "tags".to_string(),
            )),
        }
    }

    pub fn extract_string_array(
        map: &std::collections::HashMap<String, gray_matter::Pod>,
        key: &str,
    ) -> Result<Vec<String>, DocumentValidationError> {
        match map.get(key) {
            Some(gray_matter::Pod::Array(arr)) => {
                let mut strings = Vec::new();
                for item in arr {
                    if let gray_matter::Pod::String(s) = item {
                        strings.push(s.clone());
                    }
                }
                Ok(strings)
            }
            Some(_) => Err(DocumentValidationError::InvalidContent(format!(
                "{} must be an array",
                key
            ))),
            None => Err(DocumentValidationError::MissingRequiredField(
                key.to_string(),
            )),
        }
    }

    pub fn extract_optional_string(
        map: &std::collections::HashMap<String, gray_matter::Pod>,
        key: &str,
    ) -> Option<String> {
        match map.get(key) {
            Some(gray_matter::Pod::String(s)) => {
                if s.is_empty() || s == "NULL" {
                    None
                } else {
                    Some(s.clone())
                }
            }
            _ => None,
        }
    }
}
