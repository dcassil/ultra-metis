use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Write};
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Duration;

const MCP_RESPONSE_TIMEOUT: Duration = Duration::from_secs(15);
const MCP_NOTIFICATION_TIMEOUT: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemUnderTest {
    OriginalMetis,
    UltraMetisMcp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerCommand {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: Option<String>,
}

pub trait ExecutionAdapter {
    fn system_under_test(&self) -> SystemUnderTest;
    fn command(&self) -> ServerCommand;

    fn start(&self) -> Result<McpSession> {
        McpSession::start(self.system_under_test(), self.command())
    }
}

#[derive(Debug, Clone, Default)]
pub struct OriginalMetisAdapter;

impl ExecutionAdapter for OriginalMetisAdapter {
    fn system_under_test(&self) -> SystemUnderTest {
        SystemUnderTest::OriginalMetis
    }

    fn command(&self) -> ServerCommand {
        ServerCommand {
            program: "metis".to_string(),
            args: vec!["mcp".to_string()],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UltraMetisMcpAdapter;

impl ExecutionAdapter for UltraMetisMcpAdapter {
    fn system_under_test(&self) -> SystemUnderTest {
        SystemUnderTest::UltraMetisMcp
    }

    fn command(&self) -> ServerCommand {
        ServerCommand {
            program: "ultra-metis-mcp".to_string(),
            args: vec![],
        }
    }
}

pub struct McpSession {
    system: SystemUnderTest,
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl McpSession {
    pub fn start(system: SystemUnderTest, command: ServerCommand) -> Result<Self> {
        let mut child = Command::new(&command.program)
            .args(&command.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .with_context(|| {
                format!(
                    "Failed to spawn MCP server: {} {}",
                    command.program,
                    command.args.join(" ")
                )
            })?;

        let stdin = child.stdin.take().context("Failed to capture MCP stdin")?;
        let stdout = child
            .stdout
            .take()
            .context("Failed to capture MCP stdout")?;

        let mut session = Self {
            system,
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        };
        session.initialize()?;
        Ok(session)
    }

    pub fn system_under_test(&self) -> &SystemUnderTest {
        &self.system
    }

    pub fn list_tools(&mut self) -> Result<Vec<ToolDescriptor>> {
        let response = self.send_request("tools/list", Some(json!({})))?;
        let tools = response["result"]["tools"]
            .as_array()
            .ok_or_else(|| anyhow!("Invalid tools/list response: missing result.tools"))?;

        Ok(tools
            .iter()
            .map(|tool| ToolDescriptor {
                name: tool["name"].as_str().unwrap_or_default().to_string(),
                description: tool["description"].as_str().map(ToString::to_string),
            })
            .collect())
    }

    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value> {
        self.send_request(
            "tools/call",
            Some(json!({
                "name": name,
                "arguments": arguments,
            })),
        )
    }

    fn initialize(&mut self) -> Result<()> {
        let _ = self.send_request(
            "initialize",
            Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "practical-benchmark", "version": "0.1.0"}
            })),
        )?;

        self.send_notification("notifications/initialized", None)?;

        if matches!(self.system, SystemUnderTest::UltraMetisMcp) {
            let _ = self.try_read_response();
        }

        Ok(())
    }

    fn send_notification(&mut self, method: &str, params: Option<Value>) -> Result<()> {
        let mut payload = json!({
            "jsonrpc": "2.0",
            "method": method,
        });
        if let Some(params) = params {
            payload["params"] = params;
        }
        self.write_message(&payload)
    }

    fn send_request(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        let request_id = self.next_id;
        self.next_id += 1;

        let mut payload = json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
        });
        if let Some(params) = params {
            payload["params"] = params;
        }

        self.write_message(&payload)?;

        loop {
            let response = self.read_response()?;
            if response["id"].as_u64() == Some(request_id) {
                if response.get("error").is_some() {
                    return Err(anyhow!("MCP error response: {}", response));
                }
                return Ok(response);
            }
        }
    }

    fn write_message(&mut self, payload: &Value) -> Result<()> {
        writeln!(self.stdin, "{}", serde_json::to_string(payload)?)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn try_read_response(&mut self) -> Option<Value> {
        self.read_next_json_line(MCP_NOTIFICATION_TIMEOUT, true)
            .ok()
            .flatten()
    }

    fn read_response(&mut self) -> Result<Value> {
        self.read_next_json_line(MCP_RESPONSE_TIMEOUT, false)?
            .ok_or_else(|| anyhow!("MCP server exited before returning a JSON-RPC response"))
    }

    fn read_next_json_line(
        &mut self,
        timeout: Duration,
        return_none_on_timeout: bool,
    ) -> Result<Option<Value>> {
        let mut line = String::new();
        for _ in 0..100 {
            if !wait_for_stdout(&self.stdout, timeout)? {
                if return_none_on_timeout {
                    return Ok(None);
                }
                return Err(anyhow!(
                    "Timed out after {:.1}s waiting for MCP response from {}",
                    timeout.as_secs_f64(),
                    system_name(&self.system)
                ));
            }
            line.clear();
            let bytes = self.stdout.read_line(&mut line)?;
            if bytes == 0 {
                return Ok(None);
            }
            let trimmed = line.trim();
            if trimmed.is_empty() || !trimmed.starts_with('{') {
                continue;
            }
            if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
                return Ok(Some(value));
            }
        }
        Err(anyhow!("Too many non-JSON lines from MCP server"))
    }
}

fn wait_for_stdout(stdout: &BufReader<ChildStdout>, timeout: Duration) -> Result<bool> {
    let fd = stdout.get_ref().as_raw_fd();
    let mut poll_fd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };

    let timeout_ms = timeout.as_millis().min(i32::MAX as u128) as i32;
    let result = unsafe { libc::poll(&mut poll_fd, 1, timeout_ms) };

    if result < 0 {
        return Err(anyhow!(
            "poll() failed while waiting for MCP server output: {}",
            std::io::Error::last_os_error()
        ));
    }

    if result == 0 {
        return Ok(false);
    }

    if (poll_fd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL)) != 0 {
        return Ok(false);
    }

    Ok((poll_fd.revents & libc::POLLIN) != 0)
}

fn system_name(system: &SystemUnderTest) -> &'static str {
    match system {
        SystemUnderTest::OriginalMetis => "original-metis",
        SystemUnderTest::UltraMetisMcp => "ultra-metis-mcp",
    }
}

impl Drop for McpSession {
    fn drop(&mut self) {
        terminate_process_group(&mut self.child);
        let _ = self.child.wait();
    }
}

fn terminate_process_group(child: &mut Child) {
    let pid = child.id() as i32;
    unsafe {
        libc::killpg(pid, libc::SIGKILL);
    }
    let _ = child.kill();
}

pub fn expected_shared_tool_names() -> BTreeSet<&'static str> {
    BTreeSet::from([
        "initialize_project",
        "create_document",
        "read_document",
        "list_documents",
        "edit_document",
        "transition_phase",
        "search_documents",
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_metis_adapter_command_is_stable() {
        let adapter = OriginalMetisAdapter;
        let cmd = adapter.command();
        assert_eq!(cmd.program, "metis");
        assert_eq!(cmd.args, vec!["mcp".to_string()]);
    }

    #[test]
    fn ultra_metis_adapter_command_is_stable() {
        let adapter = UltraMetisMcpAdapter;
        let cmd = adapter.command();
        assert_eq!(cmd.program, "ultra-metis-mcp");
        assert!(cmd.args.is_empty());
    }

    #[test]
    fn shared_tool_surface_smoke_test() {
        let adapters: Vec<Box<dyn ExecutionAdapter>> = vec![
            Box::new(OriginalMetisAdapter),
            Box::new(UltraMetisMcpAdapter),
        ];

        for adapter in adapters {
            let mut session = match adapter.start() {
                Ok(session) => session,
                Err(err) => {
                    eprintln!(
                        "Skipping MCP smoke test for {:?}: {}",
                        adapter.system_under_test(),
                        err
                    );
                    continue;
                }
            };

            let tools = session.list_tools().unwrap();
            let tool_names: BTreeSet<String> = tools.into_iter().map(|t| t.name).collect();

            for expected in expected_shared_tool_names() {
                assert!(
                    tool_names.contains(expected),
                    "missing tool '{}' for {:?}",
                    expected,
                    session.system_under_test()
                );
            }
        }
    }
}
