use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tracing::Subscriber;
use tracing_subscriber::Layer;

use crate::client::{ControlClient, LogEntry};
use crate::settings::Settings;

/// A tracing `Layer` that captures log events and forwards them to a background
/// batching task via an unbounded channel.
///
/// This layer is designed to be installed in a `tracing_subscriber` registry.
/// It reads the configured log level from shared `Settings` (non-blocking) and
/// filters events below the threshold. Matching events are serialized into
/// `LogEntry` structs and sent on a channel for the `LogForwarder` to batch
/// and POST to the control service.
pub struct LogForwardingLayer {
    sender: mpsc::UnboundedSender<LogEntry>,
    settings: Arc<RwLock<Settings>>,
}

/// Background task that receives `LogEntry` items from the `LogForwardingLayer`,
/// batches them, and periodically flushes them to the control service API.
///
/// Flushing occurs when either the batch reaches 50 entries or 500ms have
/// elapsed since the last flush, whichever comes first.
pub struct LogForwarder {
    receiver: mpsc::UnboundedReceiver<LogEntry>,
    client: Arc<ControlClient>,
    machine_id: Arc<RwLock<Option<String>>>,
    batch: Vec<LogEntry>,
}

/// Create a new log forwarding layer and its corresponding background forwarder task.
///
/// Returns `(layer, task_handle)` where:
/// - `layer` should be added to the tracing subscriber
/// - `task_handle` is the `JoinHandle` for the background flush loop
pub fn new(
    client: Arc<ControlClient>,
    machine_id: Arc<RwLock<Option<String>>>,
    settings: Arc<RwLock<Settings>>,
) -> (LogForwardingLayer, tokio::task::JoinHandle<()>) {
    let (sender, receiver) = mpsc::unbounded_channel();

    let layer = LogForwardingLayer {
        sender,
        settings,
    };

    let forwarder = LogForwarder {
        receiver,
        client,
        machine_id,
        batch: Vec::new(),
    };

    let handle = tokio::spawn(forwarder.run());

    (layer, handle)
}

/// Convenience constructor that creates a `ControlClient`, wraps it in an `Arc`,
/// and returns the forwarding layer along with the background task handle.
///
/// This is the preferred entry point for both the headless binary and the Tauri
/// desktop app so they don't need to duplicate client-creation logic.
///
/// The `machine_id` should be shared with the runner so it gets populated after
/// registration. Until then the forwarder gracefully drops buffered logs.
pub fn create_layer(
    control_url: &str,
    token: &str,
    machine_id: Arc<RwLock<Option<String>>>,
    settings: Arc<RwLock<Settings>>,
) -> (LogForwardingLayer, tokio::task::JoinHandle<()>) {
    let client = Arc::new(ControlClient::new(control_url, token));
    new(client, machine_id, settings)
}

// ---------------------------------------------------------------------------
// Level helpers
// ---------------------------------------------------------------------------

/// Convert a `tracing::Level` to a numeric rank for comparison.
fn level_rank(level: tracing::Level) -> u8 {
    match level {
        tracing::Level::ERROR => 3,
        tracing::Level::WARN => 2,
        tracing::Level::INFO => 1,
        tracing::Level::DEBUG | tracing::Level::TRACE => 0,
    }
}

/// Convert a config-level string (e.g. `"info"`) to a numeric rank.
fn config_level_rank(level: &str) -> u8 {
    match level {
        "error" => 3,
        "warn" => 2,
        "info" => 1,
        "debug" | "trace" => 0,
        _ => 1, // default to info
    }
}

// ---------------------------------------------------------------------------
// Message visitor
// ---------------------------------------------------------------------------

/// Visitor that extracts the `message` field and any additional fields from a
/// tracing `Event`.
struct MessageVisitor {
    message: String,
    fields: serde_json::Map<String, serde_json::Value>,
}

impl MessageVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
            fields: serde_json::Map::new(),
        }
    }
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        } else {
            self.fields.insert(
                field.name().to_string(),
                serde_json::Value::String(format!("{value:?}")),
            );
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            self.fields.insert(
                field.name().to_string(),
                serde_json::Value::String(value.to_string()),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Layer implementation
// ---------------------------------------------------------------------------

impl<S: Subscriber> Layer<S> for LogForwardingLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        // Read log level from settings without blocking.
        // If the lock is held, fall back to "info".
        let threshold = self
            .settings
            .try_read()
            .map(|s| config_level_rank(&s.log_level))
            .unwrap_or(1); // info

        let event_rank = level_rank(*event.metadata().level());
        if event_rank < threshold {
            return;
        }

        let mut visitor = MessageVisitor::new();
        event.record(&mut visitor);

        let fields = if visitor.fields.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(visitor.fields))
        };

        let entry = LogEntry {
            level: event.metadata().level().to_string().to_lowercase(),
            target: event.metadata().target().to_string(),
            message: visitor.message,
            fields,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Non-blocking send; if the receiver is dropped we silently discard.
        let _ = self.sender.send(entry);
    }
}

// ---------------------------------------------------------------------------
// Forwarder background task
// ---------------------------------------------------------------------------

impl LogForwarder {
    /// Maximum number of entries to buffer before flushing.
    const MAX_BATCH_SIZE: usize = 50;

    /// Run the forwarder loop until the sending side is dropped.
    async fn run(mut self) {
        let flush_interval = tokio::time::Duration::from_millis(500);

        loop {
            // Wait for either a new log entry or the flush timer.
            let maybe_entry = tokio::time::timeout(flush_interval, self.receiver.recv()).await;

            match maybe_entry {
                Ok(Some(entry)) => {
                    self.batch.push(entry);
                    if self.batch.len() >= Self::MAX_BATCH_SIZE {
                        self.flush().await;
                    }
                }
                Ok(None) => {
                    // Channel closed — flush remaining and exit.
                    self.flush().await;
                    break;
                }
                Err(_) => {
                    // Timeout — flush whatever we have.
                    self.flush().await;
                }
            }
        }
    }

    /// Flush the current batch to the control service.
    async fn flush(&mut self) {
        if self.batch.is_empty() {
            return;
        }

        let machine_id = self.machine_id.read().await;
        let Some(id) = machine_id.as_deref() else {
            // Not yet registered — drop the batch.
            self.batch.clear();
            return;
        };

        let id = id.to_string();
        drop(machine_id); // release lock before the HTTP call

        if let Err(e) = self.client.post_machine_logs(&id, &self.batch).await {
            eprintln!("Failed to forward logs: {e}");
        }

        self.batch.clear();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_rank_ordering() {
        assert!(level_rank(tracing::Level::ERROR) > level_rank(tracing::Level::WARN));
        assert!(level_rank(tracing::Level::WARN) > level_rank(tracing::Level::INFO));
        assert!(level_rank(tracing::Level::INFO) > level_rank(tracing::Level::DEBUG));
        assert_eq!(level_rank(tracing::Level::DEBUG), level_rank(tracing::Level::TRACE));
    }

    #[test]
    fn test_config_level_rank_mapping() {
        assert_eq!(config_level_rank("error"), 3);
        assert_eq!(config_level_rank("warn"), 2);
        assert_eq!(config_level_rank("info"), 1);
        assert_eq!(config_level_rank("debug"), 0);
        assert_eq!(config_level_rank("trace"), 0);
        // Unknown defaults to info
        assert_eq!(config_level_rank("bogus"), 1);
    }

    #[test]
    fn test_level_filtering_skips_below_threshold() {
        // An event at DEBUG (rank=0) should be skipped when threshold is INFO (rank=1)
        let event_rank = level_rank(tracing::Level::DEBUG);
        let threshold = config_level_rank("info");
        assert!(event_rank < threshold, "DEBUG should be below INFO threshold");
    }

    #[test]
    fn test_level_filtering_passes_at_threshold() {
        // An event at INFO (rank=1) should pass when threshold is INFO (rank=1)
        let event_rank = level_rank(tracing::Level::INFO);
        let threshold = config_level_rank("info");
        assert!(event_rank >= threshold, "INFO should pass INFO threshold");
    }

    #[test]
    fn test_level_filtering_passes_above_threshold() {
        // An event at ERROR (rank=3) should pass when threshold is INFO (rank=1)
        let event_rank = level_rank(tracing::Level::ERROR);
        let threshold = config_level_rank("info");
        assert!(event_rank >= threshold, "ERROR should pass INFO threshold");
    }

    #[test]
    fn test_message_visitor_starts_empty() {
        let visitor = MessageVisitor::new();
        assert!(visitor.message.is_empty());
        assert!(visitor.fields.is_empty());
    }

    #[test]
    fn test_log_forwarding_layer_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<LogForwardingLayer>();
    }

    #[test]
    fn test_log_forwarder_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<LogForwarder>();
    }
}
