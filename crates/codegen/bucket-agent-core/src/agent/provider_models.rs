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

use crate::agent::config::{EnvKeys, ModelEntry, ModelInfo};
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

/// Resolve the provider-specific env var name(s) from a base URL.
///
/// Returns `None` for unknown providers or providers that need no auth (Ollama).
/// This allows `ModelEntry::own_credential()` to resolve the correct BYOK API
/// key at credential resolution time, so it takes precedence over session tokens.
fn env_key_for_base_url(base_url: &str) -> Option<EnvKeys> {
    let url = base_url.to_lowercase();
    if url.contains("nvidia.com") {
        Some(EnvKeys::single("NVIDIA_API_KEY"))
    } else if url.contains("openai.com") {
        Some(EnvKeys::single("OPENAI_API_KEY"))
    } else if url.contains("anthropic.com") {
        Some(EnvKeys::single("ANTHROPIC_API_KEY"))
    } else if url.contains("groq.com") {
        Some(EnvKeys::single("GROQ_API_KEY"))
    } else if url.contains("openrouter.ai") {
        Some(EnvKeys::single("OPENROUTER_API_KEY"))
    } else if url.contains("googleapis.com") || url.contains("google.com") {
        Some(EnvKeys::new(["GEMINI_API_KEY", "GOOGLE_GENERATIVE_AI_API_KEY"]))
    } else {
        None
    }
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
    registry_cache: Option<&std::collections::HashMap<String, u64>>,
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
        context_window: std::num::NonZeroU64::new(estimate_context_window(&entry.id, registry_cache))
            .unwrap_or_else(|| std::num::NonZeroU64::new(128_000).unwrap()),
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
        env_key: env_key_for_base_url(base_url),
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
    registry_cache: Option<&std::collections::HashMap<String, u64>>,
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
        context_window: std::num::NonZeroU64::new(estimate_context_window(&entry.name, registry_cache))
            .unwrap_or_else(|| std::num::NonZeroU64::new(128_000).unwrap()),
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
            env_key: Some(EnvKeys::single("ANTHROPIC_API_KEY")),
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
    let _ = sync_registry_models_cache();
    let registry_cache = load_models_registry_cache();
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
                    .map(|e| ollama_entry_to_model(e, &base_url, registry_cache.as_ref()))
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
                    .map(|e| openai_entry_to_model(e, &base_url, registry_cache.as_ref()))
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

fn sync_registry_models_cache() -> Option<()> {
    let cache_dir = dirs::home_dir()?.join(".bucket");
    if !cache_dir.exists() {
        let _ = std::fs::create_dir_all(&cache_dir);
    }
    let cache_path = cache_dir.join("models.json");
    let tmp_path = cache_dir.join("models.json.tmp");
    
    let is_fresh = if let Ok(metadata) = std::fs::metadata(&cache_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                elapsed < Duration::from_secs(24 * 3600)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    
    if is_fresh {
        return Some(());
    }
    
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build();
        if let Ok(client) = client {
            if let Ok(response) = client.get("https://models.dev/api.json").send() {
                if response.status().is_success() {
                    if let Ok(bytes) = response.bytes() {
                        if std::fs::write(&tmp_path, bytes).is_ok() {
                            let _ = std::fs::rename(&tmp_path, &cache_path);
                        }
                    }
                }
            }
        }
    });
    
    Some(())
}

fn normalize_model_id(id: &str) -> String {
    id.to_lowercase()
        .replace('_', "-")
        .replace('.', "-")
        .split(':')
        .next()
        .unwrap_or("")
        .to_string()
}

fn load_models_registry_cache() -> Option<std::collections::HashMap<String, u64>> {
    let cache_path = dirs::home_dir()?.join(".bucket/models.json");
    if !cache_path.exists() {
        return None;
    }
    let file = std::fs::File::open(cache_path).ok()?;
    let reader = std::io::BufReader::new(file);
    
    #[derive(serde::Deserialize)]
    struct CacheLimit {
        context: u64,
    }
    
    #[derive(serde::Deserialize)]
    struct CacheModel {
        limit: Option<CacheLimit>,
    }
    
    #[derive(serde::Deserialize)]
    struct CacheProvider {
        models: std::collections::HashMap<String, CacheModel>,
    }
    
    let data: std::collections::HashMap<String, CacheProvider> = serde_json::from_reader(reader).ok()?;
    let mut map = std::collections::HashMap::new();
    for provider in data.values() {
        for (m_id, m_info) in &provider.models {
            if let Some(limit) = &m_info.limit {
                map.insert(normalize_model_id(m_id), limit.context);
            }
        }
    }
    Some(map)
}

fn lookup_cache_context_window(model_id: &str, cache_map: Option<&std::collections::HashMap<String, u64>>) -> Option<u64> {
    let cache_map = cache_map?;
    let norm_model = normalize_model_id(model_id);
    
    // 1. Exact match
    if let Some(&context) = cache_map.get(&norm_model) {
        return Some(context);
    }
    
    // 2. Slug match (without provider prefix)
    let slug = norm_model.split('/').last().unwrap_or(&norm_model);
    if let Some(&context) = cache_map.get(slug) {
        return Some(context);
    }

    // 3. Precise substring match
    for (m_id, &context) in cache_map {
        let m_slug = m_id.split('/').last().unwrap_or(m_id);
        if m_slug == slug || (m_slug.contains(slug) && slug.len() > 6) {
            return Some(context);
        }
    }
    
    None
}

fn estimate_context_window(model_id: &str, cache_map: Option<&std::collections::HashMap<String, u64>>) -> u64 {
    if let Some(cached) = lookup_cache_context_window(model_id, cache_map) {
        return cached;
    }
    let lower = model_id.to_lowercase();
    
    // 1. Try parsing suffixes like '128k' or '1m'
    if let Some(pos) = lower.find('k') {
        let bytes = lower.as_bytes();
        let mut start = pos;
        while start > 0 && bytes[start - 1].is_ascii_digit() {
            start -= 1;
        }
        if start < pos {
            if let Ok(num) = lower[start..pos].parse::<u64>() {
                return num * 1_000;
            }
        }
    }
    
    if let Some(pos) = lower.find('m') {
        let bytes = lower.as_bytes();
        let mut start = pos;
        while start > 0 && bytes[start - 1].is_ascii_digit() {
            start -= 1;
        }
        if start < pos {
            if let Ok(num) = lower[start..pos].parse::<u64>() {
                return num * 1_000_000;
            }
        }
    }

    // 2. Family heuristics
    if lower.contains("ultra") {
        1_000_000
    } else if lower.contains("gemini-1.5-pro") {
        2_000_000
    } else if lower.contains("gemini") {
        1_048_576
    } else if lower.contains("claude-3") || lower.contains("claude-3-5") {
        200_000
    } else if lower.contains("llama-3.1") || lower.contains("llama-3.2") || lower.contains("llama-3.3") {
        128_000
    } else if lower.contains("gpt-4o") || lower.contains("gpt-4-turbo") {
        128_000
    } else {
        128_000 // default fallback
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
        let env_key = model.env_key.as_ref().expect("env_key should be set");
        assert_eq!(env_key.primary(), Some("OPENAI_API_KEY"));
    }

    #[test]
    fn openai_entry_nvidia_nim_has_env_key() {
        let entry = OpenAiModelEntry {
            id: "nvidia/llama-3.1-70b-instruct".to_string(),
            owned_by: Some("nvidia".to_string()),
        };
        let (key, model) = openai_entry_to_model(&entry, "https://integrate.api.nvidia.com/v1");
        assert_eq!(key, "nvidia/llama-3.1-70b-instruct");
        let env_key = model.env_key.as_ref().expect("env_key should be set for NVIDIA");
        assert_eq!(env_key.primary(), Some("NVIDIA_API_KEY"));
    }

    #[test]
    fn openai_entry_unknown_provider_has_no_env_key() {
        let entry = OpenAiModelEntry {
            id: "custom-model".to_string(),
            owned_by: None,
        };
        let (_, model) = openai_entry_to_model(&entry, "https://my-custom-api.example.com/v1");
        assert!(model.env_key.is_none(), "unknown provider should have no env_key");
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
            let env_key = model.env_key.as_ref().expect("Anthropic model should have env_key");
            assert_eq!(env_key.primary(), Some("ANTHROPIC_API_KEY"), "Anthropic model {key} env_key");
        }
    }

    #[test]
    fn env_key_for_base_url_nvidia() {
        let key = env_key_for_base_url("https://integrate.api.nvidia.com/v1");
        assert_eq!(key.unwrap().primary(), Some("NVIDIA_API_KEY"));
    }

    #[test]
    fn env_key_for_base_url_openai() {
        let key = env_key_for_base_url("https://api.openai.com/v1");
        assert_eq!(key.unwrap().primary(), Some("OPENAI_API_KEY"));
    }

    #[test]
    fn env_key_for_base_url_anthropic() {
        let key = env_key_for_base_url("https://api.anthropic.com/v1");
        assert_eq!(key.unwrap().primary(), Some("ANTHROPIC_API_KEY"));
    }

    #[test]
    fn env_key_for_base_url_groq() {
        let key = env_key_for_base_url("https://api.groq.com/openai/v1");
        assert_eq!(key.unwrap().primary(), Some("GROQ_API_KEY"));
    }

    #[test]
    fn env_key_for_base_url_openrouter() {
        let key = env_key_for_base_url("https://openrouter.ai/api/v1");
        assert_eq!(key.unwrap().primary(), Some("OPENROUTER_API_KEY"));
    }

    #[test]
    fn env_key_for_base_url_google() {
        let key = env_key_for_base_url("https://generativelanguage.googleapis.com/v1beta/openai");
        let env_keys = key.unwrap();
        assert!(env_keys.names().contains(&"GEMINI_API_KEY"));
    }

    #[test]
    fn env_key_for_base_url_ollama_returns_none() {
        assert!(env_key_for_base_url("http://localhost:11434").is_none());
    }

    #[test]
    fn env_key_for_base_url_unknown_returns_none() {
        assert!(env_key_for_base_url("https://my-custom-api.example.com/v1").is_none());
    }
}
