//! Provider model discovery.
//!
//! When a BYOK provider is configured (via the provider config modal or
//! `providers.toml`), this module fetches the list of available models from
//! the provider's API and returns them as [`ModelEntry`]s ready to merge
//! into the agent's catalog.
//!
//! Supported providers:
//! - **OpenAI-compatible** (`/v1/models`): OpenAI, NVIDIA NIM, OpenRouter, Groq, Gemini
//! - **Ollama** (`/api/tags`): local Ollama instances
//! - **Anthropic**: hardcoded list (no public models endpoint)

use indexmap::IndexMap;
use std::time::Duration;

use crate::agent::config::{ModelEntry, ModelInfo};
use crate::sampling::ApiBackend;
use bucket_sampler::AuthScheme;

const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

/// Resolve the base URL for a known provider name.
///
/// Returns `None` for unknown providers (custom URLs are handled by the caller
/// reading `BUCKET_MODELS_BASE_URL` directly).
pub fn resolve_provider_base_url(provider: &str) -> Option<&'static str> {
    match provider.to_lowercase().as_str() {
        "openai" => Some("https://api.openai.com/v1"),
        "anthropic" => Some("https://api.anthropic.com/v1"),
        "nvidia_nim" | "nvidia" => Some("https://integrate.api.nvidia.com/v1"),
        "openrouter" => Some("https://openrouter.ai/api/v1"),
        "groq" => Some("https://api.groq.com/openai/v1"),
        "gemini" | "google" => {
            Some("https://generativelanguage.googleapis.com/v1beta/openai")
        }
        "ollama" => Some("http://localhost:11434"),
        _ => None,
    }
}

/// Whether the provider is Anthropic (needs hardcoded model list).
pub fn is_anthropic_provider(provider: &str) -> bool {
    provider.to_lowercase() == "anthropic"
}

/// Whether the provider is Ollama (uses `/api/tags` instead of `/v1/models`).
pub fn is_ollama_provider(provider: &str) -> bool {
    provider.to_lowercase() == "ollama"
}

// ── OpenAI-compatible /v1/models ───────────────────────────────────────

#[derive(serde::Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModel>,
}

#[derive(serde::Deserialize)]
struct OpenAiModel {
    id: String,
    #[serde(default)]
    owned_by: Option<String>,
}

fn fetch_openai_models(
    base_url: &str,
    api_key: &str,
) -> Result<Vec<OpenAiModelEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let client = reqwest::blocking::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .build()?;
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .send()?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()).into());
    }

    let body: OpenAiModelsResponse = resp.json()?;
    let entries: Vec<OpenAiModelEntry> = body
        .data
        .into_iter()
        .map(|m| OpenAiModelEntry {
            id: m.id,
            owned_by: m.owned_by,
        })
        .collect();
    tracing::info!(count = entries.len(), "fetched models from provider");
    Ok(entries)
}

struct OpenAiModelEntry {
    id: String,
    owned_by: Option<String>,
}

fn openai_entry_to_model(
    entry: &OpenAiModelEntry,
    base_url: &str,
) -> (String, ModelEntry) {
    let key = entry.id.clone();
    let info = ModelInfo {
        id: None,
        model: entry.id.clone(),
        base_url: base_url.to_string(),
        name: None,
        description: entry.owned_by.as_deref().map(|o| format!("by {o}")),
        max_completion_tokens: None,
        temperature: None,
        top_p: None,
        api_backend: ApiBackend::ChatCompletions,
        auth_scheme: AuthScheme::Bearer,
        extra_headers: IndexMap::new(),
        context_window: std::num::NonZeroU64::new(128_000)
            .expect("128000 is non-zero"),
        auto_compact_threshold_percent: None,
        system_prompt_label: None,
        use_concise: false,
        agent_type: "bucket-build-plan".to_string(),
        inference_idle_timeout_secs: None,
        max_retries: None,
        hidden: false,
        user_selectable: true,
        supported_in_api: true,
        reasoning_effort: None,
        supports_reasoning_effort: false,
        reasoning_efforts: Vec::new(),
        supports_backend_search: false,
        compactions_remaining: None,
        compaction_at_tokens: None,
        show_model_fingerprint: false,
        stream_tool_calls: None,
        laziness_detector: Default::default(),
    };
    let entry = ModelEntry {
        info,
        api_key: None,
        env_key: None,
        api_base_url: Some(base_url.to_string()),
    };
    (key, entry)
}

// ── Ollama /api/tags ──────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(serde::Deserialize)]
struct OllamaModel {
    name: String,
    #[serde(default)]
    size: Option<u64>,
}

fn fetch_ollama_models(
    base_url: &str,
) -> Result<Vec<OllamaModelEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));
    let client = reqwest::blocking::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .build()?;
    let resp = client.get(&url).send()?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()).into());
    }

    let body: OllamaTagsResponse = resp.json()?;
    let entries: Vec<OllamaModelEntry> = body
        .models
        .into_iter()
        .map(|m| OllamaModelEntry {
            name: m.name,
            size: m.size,
        })
        .collect();
    tracing::info!(count = entries.len(), "fetched models from Ollama");
    Ok(entries)
}

struct OllamaModelEntry {
    name: String,
    #[allow(dead_code)]
    size: Option<u64>,
}

fn ollama_entry_to_model(
    entry: &OllamaModelEntry,
    base_url: &str,
) -> (String, ModelEntry) {
    // Ollama model names can include tags like "llama3.3:latest" — strip
    // the tag for the routing slug since Ollama's OpenAI-compat layer
    // accepts the bare name.
    let model_slug = entry.name.split(':').next().unwrap_or(&entry.name).to_string();
    let key = model_slug.clone();
    let info = ModelInfo {
        id: None,
        model: model_slug,
        base_url: base_url.to_string(),
        name: Some(entry.name.clone()),
        description: Some(format!("Local Ollama model")),
        max_completion_tokens: None,
        temperature: None,
        top_p: None,
        api_backend: ApiBackend::ChatCompletions,
        auth_scheme: AuthScheme::Bearer,
        extra_headers: IndexMap::new(),
        context_window: std::num::NonZeroU64::new(128_000)
            .expect("128000 is non-zero"),
        auto_compact_threshold_percent: None,
        system_prompt_label: None,
        use_concise: false,
        agent_type: "bucket-build-plan".to_string(),
        inference_idle_timeout_secs: None,
        max_retries: None,
        hidden: false,
        user_selectable: true,
        supported_in_api: true,
        reasoning_effort: None,
        supports_reasoning_effort: false,
        reasoning_efforts: Vec::new(),
        supports_backend_search: false,
        compactions_remaining: None,
        compaction_at_tokens: None,
        show_model_fingerprint: false,
        stream_tool_calls: None,
        laziness_detector: Default::default(),
    };
    let entry = ModelEntry {
        info,
        api_key: None,
        env_key: None,
        api_base_url: Some(base_url.to_string()),
    };
    (key, entry)
}

// ── Anthropic hardcoded ───────────────────────────────────────────────

fn anthropic_models() -> Vec<(&'static str, &'static str, u64)> {
    vec![
        ("claude-sonnet-4-20250514", "Claude Sonnet 4", 200_000),
        ("claude-opus-4-20250514", "Claude Opus 4", 200_000),
        ("claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet", 200_000),
        ("claude-3-5-haiku-20241022", "Claude 3.5 Haiku", 200_000),
        ("claude-3-opus-20240229", "Claude 3 Opus", 200_000),
        ("claude-3-haiku-20240307", "Claude 3 Haiku", 200_000),
    ]
}

fn anthropic_model_entries(base_url: &str) -> IndexMap<String, ModelEntry> {
    let mut map = IndexMap::new();
    for (model_id, name, context_window) in anthropic_models() {
        let info = ModelInfo {
            id: None,
            model: model_id.to_string(),
            base_url: base_url.to_string(),
            name: Some(name.to_string()),
            description: None,
            max_completion_tokens: None,
            temperature: None,
            top_p: None,
            api_backend: ApiBackend::Messages,
            auth_scheme: AuthScheme::XApiKey,
            extra_headers: IndexMap::new(),
            context_window: std::num::NonZeroU64::new(context_window)
                .expect("context_window is non-zero"),
            auto_compact_threshold_percent: None,
            system_prompt_label: None,
            use_concise: false,
            agent_type: "bucket-build-plan".to_string(),
            inference_idle_timeout_secs: None,
            max_retries: None,
            hidden: false,
            user_selectable: true,
            supported_in_api: true,
            reasoning_effort: None,
            supports_reasoning_effort: false,
            reasoning_efforts: Vec::new(),
            supports_backend_search: false,
            compactions_remaining: None,
            compaction_at_tokens: None,
            show_model_fingerprint: false,
            stream_tool_calls: None,
            laziness_detector: Default::default(),
        };
        let entry = ModelEntry {
            info,
            api_key: None,
            env_key: None,
            api_base_url: Some(base_url.to_string()),
        };
        map.insert(model_id.to_string(), entry);
    }
    map
}

// ── Orchestrator ──────────────────────────────────────────────────────

/// Discover models from the currently configured provider.
///
/// Reads `BUCKET_MODELS_BASE_URL` and `BUCKET_API_KEY` from the environment
/// (set by [`map_provider_env`]). Returns `None` if no provider is configured
/// or if the fetch fails.
pub fn discover_provider_models() -> Option<IndexMap<String, ModelEntry>> {
    let base_url = std::env::var("BUCKET_MODELS_BASE_URL").ok()?;
    let api_key = std::env::var("BUCKET_API_KEY").ok().filter(|k| !k.is_empty())?;

    // Determine provider type from the base URL
    let is_ollama = base_url.contains("localhost:11434") || base_url.contains("127.0.0.1:11434");
    let is_anthropic = base_url.contains("anthropic.com");

    let result = if is_ollama {
        fetch_ollama_models(&base_url)
            .map(|entries| {
                entries
                    .iter()
                    .map(|e| ollama_entry_to_model(e, &base_url))
                    .collect()
            })
    } else if is_anthropic {
        // Anthropic doesn't have a public /v1/models endpoint
        Ok(anthropic_model_entries(&base_url))
    } else {
        // OpenAI-compatible endpoint
        fetch_openai_models(&base_url, &api_key)
            .map(|entries| {
                entries
                    .iter()
                    .map(|e| openai_entry_to_model(e, &base_url))
                    .collect()
            })
    };

    match result {
        Ok(map) => {
            tracing::info!(
                count = map.len(),
                base_url = %base_url,
                "discovered provider models"
            );
            Some(map)
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                base_url = %base_url,
                "failed to discover provider models"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_provider_base_url_known() {
        assert_eq!(
            resolve_provider_base_url("openai"),
            Some("https://api.openai.com/v1")
        );
        assert_eq!(
            resolve_provider_base_url("anthropic"),
            Some("https://api.anthropic.com/v1")
        );
        assert_eq!(
            resolve_provider_base_url("ollama"),
            Some("http://localhost:11434")
        );
        assert_eq!(
            resolve_provider_base_url("groq"),
            Some("https://api.groq.com/openai/v1")
        );
    }

    #[test]
    fn resolve_provider_base_url_unknown() {
        assert_eq!(resolve_provider_base_url("custom"), None);
    }

    #[test]
    fn is_anthropic_provider_check() {
        assert!(is_anthropic_provider("anthropic"));
        assert!(is_anthropic_provider("Anthropic"));
        assert!(!is_anthropic_provider("openai"));
    }

    #[test]
    fn is_ollama_provider_check() {
        assert!(is_ollama_provider("ollama"));
        assert!(is_ollama_provider("Ollama"));
        assert!(!is_ollama_provider("openai"));
    }

    #[test]
    fn openai_entry_to_model_basic() {
        let entry = OpenAiModelEntry {
            id: "gpt-4o".to_string(),
            owned_by: Some("openai".to_string()),
        };
        let (key, model) = openai_entry_to_model(&entry, "https://api.openai.com/v1");
        assert_eq!(key, "gpt-4o");
        assert_eq!(model.info.model, "gpt-4o");
        assert_eq!(model.info.base_url, "https://api.openai.com/v1");
        assert_eq!(model.info.api_backend, ApiBackend::ChatCompletions);
        assert_eq!(model.info.auth_scheme, AuthScheme::Bearer);
        assert!(model.info.user_selectable);
    }

    #[test]
    fn ollama_entry_to_model_strips_tag() {
        let entry = OllamaModelEntry {
            name: "llama3.3:latest".to_string(),
            size: Some(4_000_000_000),
        };
        let (key, model) = ollama_entry_to_model(&entry, "http://localhost:11434");
        assert_eq!(key, "llama3.3");
        assert_eq!(model.info.model, "llama3.3");
        assert_eq!(model.info.name.as_deref(), Some("llama3.3:latest"));
    }

    #[test]
    fn anthropic_models_have_correct_backend() {
        let models = anthropic_model_entries("https://api.anthropic.com/v1");
        for (key, model) in &models {
            assert_eq!(model.info.api_backend, ApiBackend::Messages, "Anthropic model {key} should use messages backend");
            assert_eq!(model.info.auth_scheme, AuthScheme::XApiKey, "Anthropic model {key} should use x-api-key auth");
        }
    }
}
