use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const DEFAULT_MODEL: &str = "claude-haiku-4-5-20251001";
const MAX_TOKENS: u32 = 8192;

/// A single Claude API call result.
#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub content: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl ApiResponse {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}

#[derive(Serialize)]
struct MessageRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    system: Option<String>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: Vec<ContentBlock>,
    usage: Usage,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u64,
    output_tokens: u64,
}

/// Send a prompt to Claude API and return the response with token counts.
/// Reads API key from `ANTHROPIC_API_KEY` environment variable.
pub async fn ask(system: &str, user_prompt: &str) -> anyhow::Result<ApiResponse> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .context("ANTHROPIC_API_KEY environment variable not set")?;

    let client = reqwest::Client::new();
    let request = MessageRequest {
        model: DEFAULT_MODEL.to_string(),
        max_tokens: MAX_TOKENS,
        messages: vec![Message {
            role: "user".to_string(),
            content: user_prompt.to_string(),
        }],
        system: Some(system.to_string()),
    };

    let response = client
        .post(CLAUDE_API_URL)
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to send request to Claude API")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("Claude API error {}: {}", status, body));
    }

    let msg: MessageResponse = response
        .json()
        .await
        .context("Failed to parse Claude API response")?;

    let content = msg
        .content
        .into_iter()
        .map(|b| b.text)
        .collect::<Vec<_>>()
        .join("\n");

    Ok(ApiResponse {
        content,
        input_tokens: msg.usage.input_tokens,
        output_tokens: msg.usage.output_tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_total_tokens() {
        let r = ApiResponse {
            content: "hello".to_string(),
            input_tokens: 100,
            output_tokens: 50,
        };
        assert_eq!(r.total_tokens(), 150);
    }
}
