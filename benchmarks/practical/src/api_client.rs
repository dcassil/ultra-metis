use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::os::unix::process::CommandExt;
use std::process::Stdio;
use std::thread;
use std::time::{Duration, Instant};

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const DEFAULT_MODEL: &str = "claude-haiku-4-5-20251001";
const MAX_TOKENS: u32 = 8192;
const CLAUDE_CLI_TIMEOUT: Duration = Duration::from_secs(20);

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

/// Send a prompt to Claude and return the response with token counts.
///
/// Uses ANTHROPIC_API_KEY for direct HTTP calls if available.
/// Falls back to the `claude` CLI subprocess (uses existing login) if not.
pub async fn ask(system: &str, user_prompt: &str) -> anyhow::Result<ApiResponse> {
    if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        ask_via_http(system, user_prompt).await
    } else {
        ask_via_cli(system, user_prompt)
    }
}

async fn ask_via_http(system: &str, user_prompt: &str) -> anyhow::Result<ApiResponse> {
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

/// Call Claude via the `claude` CLI subprocess (no API key needed — uses existing login).
fn ask_via_cli(system: &str, user_prompt: &str) -> anyhow::Result<ApiResponse> {
    let mut child = std::process::Command::new("claude")
        .args([
            "-p",
            user_prompt,
            "--system-prompt",
            system,
            "--output-format",
            "json",
            "--model",
            "haiku",
            "--no-session-persistence",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .context("Failed to invoke `claude` CLI — is it installed and logged in?")?;

    let start = Instant::now();
    loop {
        if child
            .try_wait()
            .context("Failed while waiting for `claude` CLI")?
            .is_some()
        {
            break;
        }

        if start.elapsed() >= CLAUDE_CLI_TIMEOUT {
            terminate_process_group(&mut child);
            let _ = child.wait();
            return Err(anyhow!(
                "claude CLI timed out after {:.0}s. Verify Claude Code is logged in and prompt execution is allowed in this environment.",
                CLAUDE_CLI_TIMEOUT.as_secs_f64()
            ));
        }

        thread::sleep(Duration::from_millis(100));
    }

    let output = child
        .wait_with_output()
        .context("Failed to capture `claude` CLI output")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = if stderr.trim().is_empty() {
            "claude CLI exited without stderr output. Verify Claude Code is logged in and prompt execution is allowed in this environment.".to_string()
        } else {
            format!("claude CLI exited with error: {}", stderr)
        };
        return Err(anyhow!(detail));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).context("Failed to parse claude CLI JSON output")?;

    if parsed["is_error"].as_bool().unwrap_or(false) {
        return Err(anyhow!(
            "claude CLI returned error: {}",
            parsed["result"].as_str().unwrap_or("unknown")
        ));
    }

    let content = parsed["result"].as_str().unwrap_or("").to_string();
    let input_tokens = parsed["usage"]["input_tokens"].as_u64().unwrap_or(0);
    let output_tokens = parsed["usage"]["output_tokens"].as_u64().unwrap_or(0);

    Ok(ApiResponse {
        content,
        input_tokens,
        output_tokens,
    })
}

fn terminate_process_group(child: &mut std::process::Child) {
    let pid = child.id() as i32;
    unsafe {
        libc::killpg(pid, libc::SIGKILL);
    }
    let _ = child.kill();
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
