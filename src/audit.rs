//! Transaction audit hooks for observing raw EPP exchanges.

use chrono::{DateTime, Utc};

/// Receives structured evidence for raw EPP transaction exchanges.
///
/// Implementations should return quickly. Applications that need durable persistence should
/// forward events to their own queue or channel.
pub trait EppTransactionAuditSink: Send + Sync {
    fn record(&self, event: EppTransactionAuditEvent);
}

/// Evidence captured for a single EPP transaction attempt.
#[derive(Clone, Debug)]
pub struct EppTransactionAuditEvent {
    pub registry: String,
    pub command: Option<&'static str>,
    pub command_type: Option<&'static str>,
    pub client_tr_id: Option<String>,
    pub server_tr_id: Option<String>,
    pub request_at: DateTime<Utc>,
    pub response_at: DateTime<Utc>,
    pub response_code: Option<u16>,
    pub message: Option<String>,
    pub succeeded: bool,
    pub raw_request: String,
    pub raw_response: Option<String>,
    pub error: Option<String>,
}
