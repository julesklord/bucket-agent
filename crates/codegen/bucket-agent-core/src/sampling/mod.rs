pub mod conversation;
pub mod error;
pub mod types;

// `Client` is the legacy alias used throughout the shell. A later refactor
// retired the bespoke shell HTTP client and points `Client` at the sampler crate's
// `SamplingClient` -- the two have identical method sets, so call-sites
// compile unchanged.
pub use self::conversation::*;
pub use self::error::{ResponseModelMetadata, Result, SamplingError};
pub use self::types::*;
pub use bucket_sampler::ApiBackend;
pub use bucket_sampler::SamplingClient as Client;

// Re-export async-openai Responses API types under `rs` namespace
pub use async_openai::types::responses as rs;

// ---------------------------------------------------------------------------
// bucket-sampler re-exports
// ---------------------------------------------------------------------------
//
// The actual streaming / retry / HTTP-client logic lives in the
// `bucket-sampler` crate. We re-export the public surface here so
// `crate::sampling::{SamplerHandle, SamplerConfig, ...}` paths keep working
// for callers that haven't been ported to spell these directly via
// `bucket_sampler::*`. The shell-side `sampling::client::Config`
// composite was removed when its only remaining role -- session-snapshot
// state for `MvpAgent` -- was migrated to `RefCell<SamplerConfig>` directly.
pub use bucket_sampler::{
    InferenceLatencyStats, OriginClientInfo, RequestId, SamplerActor, SamplerConfig, SamplerHandle,
    SamplingChannel, SamplingClient, SamplingErrorInfo, SamplingErrorKind, SamplingEvent,
};

/// Derive a compact, TUI-friendly error label from a [`SamplingErrorKind`] and the
/// `reason` string that was already computed by the sampler's retry loop.
///
/// For `Api` errors the reason string follows the format produced by
/// [`bucket_sampler::format_sampling_error`]: `"API error (HTTP NNN…): …"`.
/// We extract the status code from that prefix to produce labels like
/// `"HTTP 503 (unavailable)"`. All other variants map to short constant strings.
///
/// This avoids threading the full `SamplingError` into the notification path;
/// the `kind + reason` pair is already available at every call site.
pub fn retry_label_from_kind_and_reason(kind: SamplingErrorKind, reason: &str) -> String {
    match kind {
        SamplingErrorKind::Api | SamplingErrorKind::RateLimited => {
            // Attempt to parse "API error (HTTP NNN…" or "API error (status NNN…"
            // Both prefixes appear in practice; try the newer one first.
            let status_code = reason
                .find("HTTP ")
                .or_else(|| reason.find("status "))
                .and_then(|pos| {
                    let after = &reason[pos..];
                    // Skip "HTTP " or "status "
                    let digits_start = after.find(|c: char| c.is_ascii_digit())?;
                    let digits = &after[digits_start..];
                    let end = digits.find(|c: char| !c.is_ascii_digit()).unwrap_or(digits.len());
                    digits[..end].parse::<u16>().ok()
                });
            if let Some(code) = status_code {
                let hint = match code {
                    429 => " (rate limited)",
                    500 => " (server error)",
                    502 | 503 | 504 => " (unavailable)",
                    520 => " (gateway error)",
                    400 => " (bad request)",
                    401 | 403 => " (auth error)",
                    404 => " (not found)",
                    413 => " (too large)",
                    _ => "",
                };
                format!("HTTP {code}{hint}")
            } else if kind == SamplingErrorKind::RateLimited {
                "rate limited".to_string()
            } else {
                "API error".to_string()
            }
        }
        SamplingErrorKind::Auth => "auth error".to_string(),
        SamplingErrorKind::Http => {
            if reason.contains("timed out") || reason.contains("timeout") {
                "timeout".to_string()
            } else if reason.contains("connection") {
                "connection failed".to_string()
            } else {
                "network error".to_string()
            }
        }
        SamplingErrorKind::Serialization => "parse error".to_string(),
        SamplingErrorKind::EmptyResponse => "empty response".to_string(),
        SamplingErrorKind::MaxTokensTruncation => "max tokens".to_string(),
        SamplingErrorKind::DoomLoopDetected => "reasoning loop".to_string(),
        SamplingErrorKind::IdleTimeout => "model timeout".to_string(),
    }
}
