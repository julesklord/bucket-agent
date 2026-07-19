pub mod capabilities;
pub use capabilities::ProviderCapabilities;

use async_trait::async_trait;
use bucket_sampler::{RequestId, SamplerConfig, SamplerHandle};
use bucket_sampling_types::{ConversationRequest, ConversationResponse, SamplingError};

/// Provider-agnostic interface for chat inference.
///
/// The agent code should interact with `dyn ChatProvider` rather than
/// concrete sampler/HTTP types. This enables:
/// - Swapping providers at runtime (Ollama, OpenAI, Anthropic, etc.)
/// - External plugin providers (Phase 5)
/// - Easier testing with mock providers
///
/// # Design notes
///
/// The current implementation wraps `SamplerHandle` which already handles
/// multi-backend dispatch (ChatCompletions, Responses, Messages) via the
/// `ApiBackend` enum. Future implementations could provide entirely
/// different backends (e.g., a local model runner, a gRPC-based provider).
#[async_trait]
pub trait ChatProvider: Send + Sync {
    /// Submit a conversation request and await the complete response.
    ///
    /// Streaming events still flow through the shared `SamplingEvent`
    /// channel for live UI updates — this method additionally awaits
    /// the per-request completion so the caller gets a clean `Result`.
    async fn complete(
        &self,
        request: ConversationRequest,
    ) -> Result<(ConversationResponse, bucket_sampler::InferenceLatencyStats), SamplingError>;

    /// Get the current provider capabilities.
    fn capabilities(&self) -> ProviderCapabilities;

    /// Get the current model ID.
    fn model_id(&self) -> &str;

    /// Update the provider configuration (model switch, auth refresh).
    fn update_config(&self, config: SamplerConfig);

    /// Cancel an in-flight request. No-op if the request id is
    /// unknown (already finished or never submitted).
    fn cancel(&self, request_id: RequestId);
}

/// Default `ChatProvider` implementation wrapping a [`SamplerHandle`].
///
/// This delegates all inference to the sampler actor, which handles
/// retry logic, streaming, and multi-backend dispatch internally.
pub struct SamplerProvider {
    handle: SamplerHandle,
    model_id: String,
}

impl SamplerProvider {
    /// Create a new provider wrapping the given sampler handle.
    pub fn new(handle: SamplerHandle, config: SamplerConfig) -> Self {
        Self {
            handle,
            model_id: config.model.clone(),
        }
    }

    /// Access the underlying sampler handle.
    pub fn handle(&self) -> &SamplerHandle {
        &self.handle
    }
}

#[async_trait]
impl ChatProvider for SamplerProvider {
    async fn complete(
        &self,
        request: ConversationRequest,
    ) -> Result<(ConversationResponse, bucket_sampler::InferenceLatencyStats), SamplingError> {
        let request_id = RequestId::random();
        self.handle.submit_and_collect(request_id, request).await
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::default()
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn update_config(&self, config: SamplerConfig) {
        self.handle.update_config(config);
    }

    fn cancel(&self, request_id: RequestId) {
        self.handle.cancel(request_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mock provider for testing that returns a fixed response.
    pub struct MockProvider {
        model_id: String,
    }

    impl MockProvider {
        pub fn new(model_id: impl Into<String>) -> Self {
            Self {
                model_id: model_id.into(),
            }
        }
    }

    #[async_trait]
    impl ChatProvider for MockProvider {
        async fn complete(
            &self,
            _request: ConversationRequest,
        ) -> Result<(ConversationResponse, bucket_sampler::InferenceLatencyStats), SamplingError>
        {
            Err(SamplingError::Auth("mock provider not implemented".into()))
        }

        fn capabilities(&self) -> ProviderCapabilities {
            ProviderCapabilities::default()
        }

        fn model_id(&self) -> &str {
            &self.model_id
        }

        fn update_config(&self, _config: SamplerConfig) {}

        fn cancel(&self, _request_id: RequestId) {}
    }

    #[test]
    fn mock_provider_returns_model_id() {
        let provider = MockProvider::new("test-model");
        assert_eq!(provider.model_id(), "test-model");
    }

    #[test]
    fn default_capabilities_have_no_billing() {
        let provider = MockProvider::new("test-model");
        let caps = provider.capabilities();
        assert!(!caps.has_billing);
        assert!(caps.supports_streaming);
    }
}
