use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout};
use tokio::sync::mpsc;
use tokio::sync::Mutex;

/// An event captured from a supervised process's output streams.
///
/// Events are classified by type and category, then batched and sent to the
/// control service for real-time session monitoring.
#[derive(Debug, Clone, Serialize)]
pub struct OutputEvent {
    /// The kind of event: `"output_line"` for normal output, `"approval_request"`
    /// when the process is asking for human approval.
    pub event_type: String,

    /// Classification of the line: `"info"`, `"warning"`, `"error"`, or `"summary"`.
    pub category: Option<String>,

    /// The raw text content of the output line.
    pub content: String,

    /// Additional structured data. For approval requests this contains the
    /// parsed question and options extracted from the JSON output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    /// Monotonically increasing sequence number for ordering events.
    pub sequence: i64,
}

/// Reads stdout and stderr from a supervised child process, classifies each
/// line, batches the resulting [`OutputEvent`]s, and sends them through an mpsc
/// channel for upstream consumption (typically posting to the control service).
pub struct OutputCapture {
    session_id: String,
    stdout: ChildStdout,
    stderr: ChildStderr,
    event_sender: mpsc::Sender<(String, Vec<OutputEvent>)>,
    sequence: Arc<AtomicI64>,
}

impl OutputCapture {
    /// Create a new output capture for the given session.
    ///
    /// Events are sent as `(session_id, Vec<OutputEvent>)` batches through
    /// `event_sender`.
    pub fn new(
        session_id: String,
        stdout: ChildStdout,
        stderr: ChildStderr,
        event_sender: mpsc::Sender<(String, Vec<OutputEvent>)>,
    ) -> Self {
        Self {
            session_id,
            stdout,
            stderr,
            event_sender,
            sequence: Arc::new(AtomicI64::new(0)),
        }
    }

    /// Start reading from stdout and stderr in background tasks.
    ///
    /// Spawns three tokio tasks:
    /// 1. A stdout reader that classifies each line and pushes to a shared buffer.
    /// 2. A stderr reader that does the same with `"error"` as the default category.
    /// 3. A flusher that drains the buffer every 500ms or when 10 events accumulate.
    pub fn start(self) {
        let buffer = Arc::new(Mutex::new(Vec::<OutputEvent>::new()));

        // Spawn stdout reader
        let stdout_buf = Arc::clone(&buffer);
        let stdout_seq = Arc::clone(&self.sequence);
        tokio::spawn(async move {
            read_stream(self.stdout, false, stdout_buf, stdout_seq).await;
        });

        // Spawn stderr reader
        let stderr_buf = Arc::clone(&buffer);
        let stderr_seq = Arc::clone(&self.sequence);
        tokio::spawn(async move {
            read_stream(self.stderr, true, stderr_buf, stderr_seq).await;
        });

        // Spawn flusher
        let flush_buf = Arc::clone(&buffer);
        let session_id = self.session_id;
        let sender = self.event_sender;
        tokio::spawn(async move {
            flush_loop(flush_buf, session_id, sender).await;
        });
    }
}

/// Read lines from a stream and push classified events into the shared buffer.
async fn read_stream<R: tokio::io::AsyncRead + Unpin>(
    stream: R,
    is_stderr: bool,
    buffer: Arc<Mutex<Vec<OutputEvent>>>,
    sequence: Arc<AtomicI64>,
) {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let event = classify_line(&line, is_stderr, &sequence);
        buffer.lock().await.push(event);
    }
}

/// Periodically flush the event buffer, sending batches to the channel.
///
/// Flushes every 500ms or whenever the buffer has accumulated 10 or more events.
async fn flush_loop(
    buffer: Arc<Mutex<Vec<OutputEvent>>>,
    session_id: String,
    sender: mpsc::Sender<(String, Vec<OutputEvent>)>,
) {
    let mut interval = tokio::time::interval(Duration::from_millis(500));

    loop {
        interval.tick().await;

        let events = {
            let mut buf = buffer.lock().await;
            if buf.is_empty() {
                continue;
            }
            std::mem::take(&mut *buf)
        };

        if sender.send((session_id.clone(), events)).await.is_err() {
            // Receiver dropped — session is done.
            tracing::debug!(session_id = %session_id, "Event sender closed, stopping flush loop");
            break;
        }
    }
}

/// Classify a single output line into an [`OutputEvent`].
///
/// Classification rules (applied in priority order):
/// 1. Lines containing JSON with `"type":"tool_use"` or `"ask":true` are
///    classified as `"approval_request"` events with parsed metadata.
/// 2. Lines from stderr default to category `"error"`.
/// 3. Lines containing `"ERROR"` or `"error:"` are category `"error"`.
/// 4. Lines containing `"warning"` or `"WARN"` (case-insensitive) are category `"warning"`.
/// 5. Everything else is category `"info"`.
pub(crate) fn classify_line(
    line: &str,
    is_stderr: bool,
    sequence: &AtomicI64,
) -> OutputEvent {
    let seq = sequence.fetch_add(1, Ordering::Relaxed);

    // Check for approval-request JSON patterns first
    if let Some(metadata) = detect_approval_request(line) {
        return OutputEvent {
            event_type: "approval_request".to_string(),
            category: Some("info".to_string()),
            content: line.to_string(),
            metadata: Some(metadata),
            sequence: seq,
        };
    }

    let category = if is_stderr {
        "error".to_string()
    } else if line.contains("ERROR") || line.contains("error:") {
        "error".to_string()
    } else if line.to_lowercase().contains("warning") || line.to_lowercase().contains("warn") {
        "warning".to_string()
    } else {
        "info".to_string()
    };

    OutputEvent {
        event_type: "output_line".to_string(),
        category: Some(category),
        content: line.to_string(),
        metadata: None,
        sequence: seq,
    }
}

/// Attempt to detect an approval request in a JSON-formatted output line.
///
/// Looks for lines that parse as JSON and contain either `"type":"tool_use"`
/// or `"ask":true`, which indicate Claude Code is requesting human approval.
fn detect_approval_request(line: &str) -> Option<serde_json::Value> {
    // Quick heuristic check before attempting a full parse
    let trimmed = line.trim();
    if !trimmed.starts_with('{') {
        return None;
    }

    let parsed: serde_json::Value = serde_json::from_str(trimmed).ok()?;

    let is_tool_use = parsed.get("type").and_then(serde_json::Value::as_str) == Some("tool_use");
    let is_ask = parsed.get("ask").and_then(serde_json::Value::as_bool) == Some(true);

    if is_tool_use || is_ask {
        Some(parsed)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seq() -> AtomicI64 {
        AtomicI64::new(0)
    }

    // ---- Line classification tests ----

    #[test]
    fn test_classify_info_line() {
        let event = classify_line("Building project...", false, &seq());
        assert_eq!(event.event_type, "output_line");
        assert_eq!(event.category.as_deref(), Some("info"));
        assert_eq!(event.content, "Building project...");
        assert!(event.metadata.is_none());
    }

    #[test]
    fn test_classify_error_line_from_stderr() {
        let event = classify_line("something went wrong", true, &seq());
        assert_eq!(event.event_type, "output_line");
        assert_eq!(event.category.as_deref(), Some("error"));
    }

    #[test]
    fn test_classify_error_keyword_in_stdout() {
        let event = classify_line("ERROR: compilation failed", false, &seq());
        assert_eq!(event.category.as_deref(), Some("error"));

        let event2 = classify_line("error: could not find module", false, &seq());
        assert_eq!(event2.category.as_deref(), Some("error"));
    }

    #[test]
    fn test_classify_warning_case_insensitive() {
        let event = classify_line("warning: unused variable", false, &seq());
        assert_eq!(event.category.as_deref(), Some("warning"));

        let event2 = classify_line("WARN: disk space low", false, &seq());
        assert_eq!(event2.category.as_deref(), Some("warning"));

        let event3 = classify_line("This has a Warning in the middle", false, &seq());
        assert_eq!(event3.category.as_deref(), Some("warning"));
    }

    #[test]
    fn test_classify_stderr_takes_precedence_over_warning_keyword() {
        // stderr lines are always "error" regardless of content
        let event = classify_line("warning: something", true, &seq());
        assert_eq!(event.category.as_deref(), Some("error"));
    }

    // ---- Approval detection tests ----

    #[test]
    fn test_detect_approval_tool_use() {
        let line = r#"{"type":"tool_use","name":"Edit","input":{"file":"test.rs"}}"#;
        let event = classify_line(line, false, &seq());
        assert_eq!(event.event_type, "approval_request");
        assert!(event.metadata.is_some());
        let meta = event.metadata.unwrap();
        assert_eq!(meta["type"], "tool_use");
        assert_eq!(meta["name"], "Edit");
    }

    #[test]
    fn test_detect_approval_ask_true() {
        let line = r#"{"ask":true,"question":"Allow running bash command?"}"#;
        let event = classify_line(line, false, &seq());
        assert_eq!(event.event_type, "approval_request");
        assert!(event.metadata.is_some());
        let meta = event.metadata.unwrap();
        assert_eq!(meta["ask"], true);
    }

    #[test]
    fn test_no_approval_for_regular_json() {
        let line = r#"{"status":"ok","count":42}"#;
        let event = classify_line(line, false, &seq());
        assert_eq!(event.event_type, "output_line");
        assert_eq!(event.category.as_deref(), Some("info"));
        assert!(event.metadata.is_none());
    }

    #[test]
    fn test_no_approval_for_non_json() {
        let line = "Just a normal line with {braces}";
        let result = detect_approval_request(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_approval_detection_with_whitespace() {
        let line = r#"  {"type":"tool_use","name":"Bash"}  "#;
        let event = classify_line(line, false, &seq());
        assert_eq!(event.event_type, "approval_request");
    }

    // ---- Serialization tests ----

    #[test]
    fn test_output_event_serialization() {
        let event = OutputEvent {
            event_type: "output_line".to_string(),
            category: Some("info".to_string()),
            content: "Hello world".to_string(),
            metadata: None,
            sequence: 0,
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["event_type"], "output_line");
        assert_eq!(json["category"], "info");
        assert_eq!(json["content"], "Hello world");
        // metadata should be absent due to skip_serializing_if
        assert!(json.get("metadata").is_none());
        assert_eq!(json["sequence"], 0);
    }

    #[test]
    fn test_output_event_serialization_with_metadata() {
        let event = OutputEvent {
            event_type: "approval_request".to_string(),
            category: Some("info".to_string()),
            content: r#"{"ask":true}"#.to_string(),
            metadata: Some(serde_json::json!({"ask": true, "question": "Allow?"})),
            sequence: 5,
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["event_type"], "approval_request");
        assert!(json.get("metadata").is_some());
        assert_eq!(json["metadata"]["question"], "Allow?");
        assert_eq!(json["sequence"], 5);
    }

    // ---- Sequence counter tests ----

    #[test]
    fn test_sequence_counter_increments() {
        let counter = seq();
        let e1 = classify_line("first", false, &counter);
        let e2 = classify_line("second", false, &counter);
        let e3 = classify_line("third", false, &counter);
        assert_eq!(e1.sequence, 0);
        assert_eq!(e2.sequence, 1);
        assert_eq!(e3.sequence, 2);
    }

    // ---- Batch accumulation tests ----

    #[tokio::test]
    async fn test_batch_accumulation_via_buffer() {
        let buffer = Arc::new(Mutex::new(Vec::<OutputEvent>::new()));
        let counter = Arc::new(AtomicI64::new(0));

        // Simulate adding events to the buffer
        for i in 0..10 {
            let event = classify_line(&format!("line {i}"), false, &counter);
            buffer.lock().await.push(event);
        }

        // Drain the buffer as the flusher would
        let events = {
            let mut buf = buffer.lock().await;
            std::mem::take(&mut *buf)
        };

        assert_eq!(events.len(), 10);
        assert_eq!(events[0].content, "line 0");
        assert_eq!(events[9].content, "line 9");
        assert_eq!(events[0].sequence, 0);
        assert_eq!(events[9].sequence, 9);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_flush_loop_sends_batches() {
        let buffer = Arc::new(Mutex::new(Vec::<OutputEvent>::new()));
        let (tx, mut rx) = mpsc::channel::<(String, Vec<OutputEvent>)>(16);
        let counter = AtomicI64::new(0);

        // Pre-fill the buffer with events
        {
            let mut buf = buffer.lock().await;
            for i in 0..3 {
                let event = classify_line(&format!("line {i}"), false, &counter);
                buf.push(event);
            }
        }

        // Start the flush loop in a background task
        let flush_buf = Arc::clone(&buffer);
        let handle = tokio::spawn(async move {
            flush_loop(flush_buf, "test-session".to_string(), tx).await;
        });

        // The flush loop uses a 500ms interval. Wait for the first batch.
        let result = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await;
        let (session_id, events) = result
            .expect("should receive within timeout")
            .expect("channel should not be closed");

        assert_eq!(session_id, "test-session");
        assert_eq!(events.len(), 3);

        // Cleanup: abort the flush task
        handle.abort();
    }
}
