use serde::{Deserialize, Serialize};

/// Declares what the current provider supports.
///
/// The TUI consults these flags to decide which UI surfaces to show
/// (credit bar, subscription gate, SuperBucket CTA, etc.).
/// The agent uses them to decide which extensions activate.
///
/// Defaults are permissive: everything enabled except billing, which
/// defaults to `false` (no billing unless the provider says so).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderCapabilities {
    /// Whether the provider has billing/subscription logic.
    /// `false` for Ollama, custom endpoints, BYOK.
    pub has_billing: bool,
    /// Whether the provider enforces credit limits.
    pub has_credit_limit: bool,
    /// Whether the provider has a subscription gate (free→paid paywall).
    pub has_subscription_gate: bool,
    /// Whether the provider supports streaming responses.
    pub supports_streaming: bool,
    /// Whether image generation tools are available.
    pub supports_image_gen: bool,
    /// Whether video generation tools are available.
    pub supports_video_gen: bool,
    /// Maximum context window in tokens (0 = unknown / provider decides).
    pub max_context_tokens: usize,
}

impl Default for ProviderCapabilities {
    fn default() -> Self {
        Self {
            has_billing: false,
            has_credit_limit: false,
            has_subscription_gate: false,
            supports_streaming: true,
            supports_image_gen: true,
            supports_video_gen: true,
            max_context_tokens: 0,
        }
    }
}

impl ProviderCapabilities {
    /// Convenience: should the TUI show any billing-related UI?
    pub fn shows_billing_ui(&self) -> bool {
        self.has_billing || self.has_credit_limit || self.has_subscription_gate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_billing() {
        let caps = ProviderCapabilities::default();
        assert!(!caps.has_billing);
        assert!(!caps.has_credit_limit);
        assert!(!caps.has_subscription_gate);
        assert!(caps.supports_streaming);
        assert!(caps.supports_image_gen);
        assert!(caps.supports_video_gen);
        assert_eq!(caps.max_context_tokens, 0);
    }

    #[test]
    fn shows_billing_ui_only_when_billing_flags_set() {
        let mut caps = ProviderCapabilities::default();
        assert!(!caps.shows_billing_ui());

        caps.has_billing = true;
        assert!(caps.shows_billing_ui());

        caps.has_billing = false;
        caps.has_credit_limit = true;
        assert!(caps.shows_billing_ui());

        caps.has_credit_limit = false;
        caps.has_subscription_gate = true;
        assert!(caps.shows_billing_ui());
    }

    #[test]
    fn serde_roundtrip() {
        let caps = ProviderCapabilities {
            has_billing: true,
            has_credit_limit: false,
            has_subscription_gate: true,
            supports_streaming: true,
            supports_image_gen: false,
            supports_video_gen: false,
            max_context_tokens: 128_000,
        };
        let json = serde_json::to_string(&caps).unwrap();
        let parsed: ProviderCapabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.has_billing, caps.has_billing);
        assert_eq!(parsed.has_subscription_gate, caps.has_subscription_gate);
        assert!(!parsed.supports_image_gen);
        assert_eq!(parsed.max_context_tokens, 128_000);
    }

    #[test]
    fn serde_deserialize_with_missing_fields_uses_defaults() {
        let json = r#"{"has_billing": true}"#;
        let caps: ProviderCapabilities = serde_json::from_str(json).unwrap();
        assert!(caps.has_billing);
        assert!(!caps.has_credit_limit);
        assert!(caps.supports_streaming);
        assert!(caps.supports_image_gen);
    }
}
