use super::mcp::*;
use toml::Value as TomlValue;
/// Resolve a bool from an optional env var > config.toml `[section] key` > false.
///
/// Uses [`crate::agent::config::env_bool`] for consistent env var parsing
/// (`1/true/yes/on/enabled` and their negations).
fn toml_bool_sync(env_var: Option<&str>, section: &str, key: &str) -> bool {
    if let Some(var) = env_var
        && let Some(val) = crate::agent::config::env_bool(var)
    {
        return val;
    }
    let root: TomlValue = match crate::config::load_effective_config() {
        Ok(r) => r,
        Err(_) => return false,
    };
    if let TomlValue::Table(table) = root
        && let Some(TomlValue::Table(s)) = table.get(section)
    {
        s.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
    } else {
        false
    }
}
pub fn load_relay_sync_enabled_sync() -> bool {
    toml_bool_sync(Some("BUCKET_RELAY_SYNC_ENABLED"), "relay", "enabled")
}
/// `[harness]` blocking-upload settings from ONE effective-config parse:
/// `block_for_upload` (default false — prompt handling waits for turn-end
/// uploads when set) and `upload_flush_timeout_secs` (default 60 — budget for
/// that wait).
pub fn load_blocking_upload_config_sync() -> (bool, std::time::Duration) {
    const DEFAULT_FLUSH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
    let root: TomlValue = match crate::config::load_effective_config() {
        Ok(r) => r,
        Err(_) => return (false, DEFAULT_FLUSH_TIMEOUT),
    };
    let harness = match &root {
        TomlValue::Table(table) => table.get("harness"),
        _ => None,
    };
    let block_for_upload = harness
        .and_then(|h| h.get("block_for_upload"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let flush_timeout = harness
        .and_then(|h| h.get("upload_flush_timeout_secs"))
        .and_then(|v| v.as_integer())
        .and_then(|v| u64::try_from(v).ok())
        .map(std::time::Duration::from_secs)
        .unwrap_or(DEFAULT_FLUSH_TIMEOUT);
    (block_for_upload, flush_timeout)
}
pub fn load_dotenv_if_present() {
    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join(".env"));
        if let Some(parent) = cwd.parent() {
            candidates.push(parent.join(".env"));
        }
    }
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        candidates.push(std::path::PathBuf::from(home).join(".env"));
    }

    for path in candidates {
        if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    let line = line.strip_prefix("export ").unwrap_or(line).trim();
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let mut val = value.trim();
                        if (val.starts_with('"') && val.ends_with('"'))
                            || (val.starts_with('\'') && val.ends_with('\''))
                        {
                            if val.len() >= 2 {
                                val = &val[1..val.len() - 1];
                            }
                        }
                        if !key.is_empty() && std::env::var(key).is_err() {
                            unsafe {
                                std::env::set_var(key, val);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn resolve_env_key_for_provider(provider_id: &str) -> Option<String> {
    match provider_id.to_lowercase().as_str() {
        "openai" => std::env::var("OPENAI_API_KEY").ok(),
        "anthropic" => std::env::var("ANTHROPIC_API_KEY")
            .or_else(|_| std::env::var("ANTHROPIC_AUTH_TOKEN"))
            .ok(),
        "nvidia_nim" | "nvidia" => std::env::var("NVIDIA_API_KEY")
            .or_else(|_| std::env::var("NIM_API_KEY"))
            .or_else(|_| std::env::var("NVAPI_KEY"))
            .ok(),
        "openrouter" => std::env::var("OPENROUTER_API_KEY").ok(),
        "groq" => std::env::var("GROQ_API_KEY").ok(),
        "gemini" | "google" => std::env::var("GEMINI_API_KEY")
            .or_else(|_| std::env::var("GOOGLE_API_KEY"))
            .ok(),
        "ollama" => Some("ollama_local".to_string()),
        _ => std::env::var("BUCKET_API_KEY").ok(),
    }
}

pub fn map_provider_env(provider_name: &str, api_key: &str) {
    let p_lower = provider_name.to_lowercase();
    let (url, default_model) = match p_lower.as_str() {
        "openai" => ("https://api.openai.com/v1", "gpt-4o"),
        "anthropic" => ("https://api.anthropic.com/v1", "claude-3-5-sonnet-20241022"),
        "nvidia_nim" | "nvidia" => (
            "https://integrate.api.nvidia.com/v1",
            "meta/llama-3.3-70b-instruct",
        ),
        "openrouter" => (
            "https://openrouter.ai/api/v1",
            "anthropic/claude-3.5-sonnet",
        ),
        "groq" => ("https://api.groq.com/openai/v1", "llama-3.3-70b-versatile"),
        "gemini" | "google" => (
            "https://generativelanguage.googleapis.com/v1beta/openai/",
            "gemini-2.0-flash",
        ),
        "ollama" => ("http://localhost:11434/v1", "llama3.3"),
        _ => (
            if p_lower.starts_with("http://") || p_lower.starts_with("https://") {
                p_lower.as_str()
            } else {
                "https://api.openai.com/v1"
            },
            "default",
        ),
    };

    let resolved_key = if api_key.is_empty() || api_key == "(api key configured)" {
        resolve_env_key_for_provider(&p_lower).unwrap_or_else(|| api_key.to_string())
    } else {
        api_key.to_string()
    };

    unsafe {
        if !resolved_key.is_empty() && resolved_key != "(api key configured)" {
            std::env::set_var("BUCKET_API_KEY", &resolved_key);
        }
        std::env::set_var("BUCKET_MODELS_BASE_URL", url);
        std::env::set_var("BUCKET_BUCKET_API_BASE_URL", url);
        if std::env::var("BUCKET_MODEL").is_err() && default_model != "default" {
            std::env::set_var("BUCKET_MODEL", default_model);
        }
    }
}

pub async fn load_config() -> Config {
    load_dotenv_if_present();

    let root: TomlValue = match crate::config::load_effective_config() {
        Ok(v) => v,
        Err(_) => return Config::default(),
    };
    let mut config = load_config_from_toml(&root);

    let mut provider_applied = false;

    // Apply providers.toml
    if let Ok(home) = std::env::var("BUCKET_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|_| {
            #[allow(deprecated)]
            std::env::home_dir()
                .map(|h| dunce::canonicalize(&h).unwrap_or(h).join(".bucket"))
                .ok_or(())
        })
    {
        let provider_file = home.join("providers.toml");
        if let Ok(content) = std::fs::read_to_string(&provider_file) {
            if let Ok(toml_val) = content.parse::<TomlValue>() {
                if let Some(providers) = toml_val.get("providers").and_then(|v| v.as_table()) {
                    let custom_provider_env = std::env::var("BUCKET_PROVIDER").ok();
                    let custom_model_env = std::env::var("BUCKET_MODEL").ok();

                    if custom_provider_env.is_none() && custom_model_env.is_none() {
                        if let Some((provider_name, api_key_val)) = providers.iter().next() {
                            if let Some(api_key) = api_key_val.as_str() {
                                map_provider_env(provider_name, api_key);
                                provider_applied = true;
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback auto-detection from process environment / .env if providers.toml was not applied
    if !provider_applied {
        for (provider_id, _) in [
            ("openai", "OPENAI_API_KEY"),
            ("anthropic", "ANTHROPIC_API_KEY"),
            ("nvidia_nim", "NVIDIA_API_KEY"),
            ("openrouter", "OPENROUTER_API_KEY"),
            ("groq", "GROQ_API_KEY"),
            ("gemini", "GEMINI_API_KEY"),
        ] {
            if let Some(key) = resolve_env_key_for_provider(provider_id) {
                if !key.trim().is_empty() {
                    map_provider_env(provider_id, &key);
                    break;
                }
            }
        }
    }

    config
}
/// Parse `Config` from a pre-loaded TOML value. Used by both async and sync paths.
pub fn load_config_from_toml(root: &TomlValue) -> Config {
    let table = match root.as_table() {
        Some(t) => t,
        None => return Config::default(),
    };
    fn section<T: serde::de::DeserializeOwned + Default>(
        table: &toml::map::Map<String, TomlValue>,
        key: &str,
    ) -> T {
        table
            .get(key)
            .and_then(|v| v.clone().try_into().ok())
            .unwrap_or_default()
    }
    if let Some(TomlValue::Table(toolset)) = table.get("toolset")
        && toolset.get("use_concise").is_some()
    {
        tracing::warn!(
            "`[toolset] use_concise` is deprecated and no longer has any effect. \
             Set `use_concise = true` on individual model entries in config.toml instead."
        );
    }
    let management_api_key = table
        .get("endpoints")
        .and_then(|v| v.get("management_api_key"))
        .and_then(|v| v.as_str())
        .map(str::to_owned);
    let permission = table
        .get("permission")
        .and_then(|v| v.clone().try_into::<PermissionConfig>().ok());
    let mut config = Config {
        cli: section(table, "cli"),
        models: section(table, "models"),
        ui: section(table, "ui"),
        harness: section(table, "harness"),
        skills: section(table, "skills"),
        compat: section(table, "compat"),
        management_api_key,
        permission,
        diagnostics: section(table, "diagnostics"),
        session: section(table, "session"),
        ask_user_question: table
            .get("toolset")
            .and_then(|t| t.get("ask_user_question"))
            .and_then(|v| v.clone().try_into().ok())
            .unwrap_or_default(),
    };

    if let Ok(model) = std::env::var("BUCKET_MODEL") {
        config.models.default = Some(model);
    }

    config
}
/// Resolve permission config with project override semantics.
///
/// Priority (per approved plan):
/// 1. Nearest project `.bucket/config.toml` with `[permission]` section (from cwd upward)
/// 2. Global `~/.bucket/config.toml` `[permission]` section
///
/// Project `[permission]` overrides global wholesale (no deep merge).
///
/// Returns `(config, source_path)` from the highest-priority config file
/// that contains a `[permission]` section.
pub async fn resolve_permission_config(
    cwd: &std::path::Path,
) -> Option<(PermissionConfig, std::path::PathBuf)> {
    let project_configs = crate::config::find_project_configs(cwd);
    for config_path in project_configs.into_iter().rev() {
        if let Ok(root) = crate::config::load_config_file(&config_path)
            && let Some(perm_val) = root.get("permission")
        {
            match perm_val.clone().try_into::<PermissionConfig>() {
                Ok(perm_config) => {
                    tracing::info!("Loaded [permission] from project");
                    return Some((perm_config, config_path));
                }
                Err(e) => tracing::warn!(error = % e, "Failed to parse [permission]"),
            }
        }
    }
    let global_path = user_config_path();
    load_config().await.permission.map(|cfg| (cfg, global_path))
}
#[cfg(test)]
mod tests {
    use super::*;
    use toml::Value as TomlValue;
    #[test]
    fn test_models_default_parsing() {
        let toml_str = r#"
[models]
default = "bucket-code-fast-1"
"#;
        let root: TomlValue = toml::from_str(toml_str).unwrap();
        if let TomlValue::Table(table) = root
            && let Some(TomlValue::Table(models)) = table.get("models")
        {
            let default = models
                .get("default")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            assert_eq!(default.as_deref(), Some("bucket-code-fast-1"));
        } else {
            panic!("Expected models table");
        }
    }
    #[test]
    fn test_remote_secret_parsing() {
        let toml_str = r#"
[remote]
secret = "my-secret-token"
"#;
        let root: TomlValue = toml::from_str(toml_str).unwrap();
        if let TomlValue::Table(table) = root
            && let Some(TomlValue::Table(remote)) = table.get("remote")
        {
            let secret = remote
                .get("secret")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            assert_eq!(secret, Some("my-secret-token".to_string()));
        } else {
            panic!("Expected remote table");
        }
    }
    #[test]
    fn test_remote_secret_empty_section() {
        let toml_str = r#"
[remote]
"#;
        let root: TomlValue = toml::from_str(toml_str).unwrap();
        if let TomlValue::Table(table) = root
            && let Some(TomlValue::Table(remote)) = table.get("remote")
        {
            let secret = remote
                .get("secret")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            assert!(secret.is_none());
        } else {
            panic!("Expected remote table");
        }
    }
    #[test]
    fn test_remote_secret_no_section() {
        let toml_str = r#"
[models]
default = "bucket-code-fast-1"
"#;
        let root: TomlValue = toml::from_str(toml_str).unwrap();
        if let TomlValue::Table(table) = root {
            let has_remote = table.get("remote").is_some();
            assert!(!has_remote);
        }
    }
    #[test]
    fn test_relay_sync_enabled_true() {
        let toml_str = r#"
[relay]
enabled = true
"#;
        let root: TomlValue = toml::from_str(toml_str).unwrap();
        if let TomlValue::Table(table) = root
            && let Some(TomlValue::Table(relay)) = table.get("relay")
        {
            let enabled = relay
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            assert!(enabled);
        } else {
            panic!("Expected relay table");
        }
    }
    #[test]
    fn test_relay_sync_enabled_false() {
        let toml_str = r#"
[relay]
enabled = false
"#;
        let root: TomlValue = toml::from_str(toml_str).unwrap();
        if let TomlValue::Table(table) = root
            && let Some(TomlValue::Table(relay)) = table.get("relay")
        {
            let enabled = relay
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            assert!(!enabled);
        } else {
            panic!("Expected relay table");
        }
    }
    #[test]
    fn test_relay_sync_default_false() {
        let toml_str = r#"
[relay]
"#;
        let root: TomlValue = toml::from_str(toml_str).unwrap();
        if let TomlValue::Table(table) = root
            && let Some(TomlValue::Table(relay)) = table.get("relay")
        {
            let enabled = relay
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            assert!(!enabled);
        }
    }
    #[test]
    fn test_relay_sync_no_section() {
        let toml_str = r#"
[models]
default = "bucket-code-fast-1"
"#;
        let root: TomlValue = toml::from_str(toml_str).unwrap();
        if let TomlValue::Table(table) = root {
            let has_relay = table.get("relay").is_some();
            assert!(!has_relay);
        }
    }
    #[test]
    fn test_relay_sync_config_struct() {
        let config = RelaySyncConfig {
            enabled: Some(true),
        };
        assert_eq!(config.enabled, Some(true));
        let config_disabled = RelaySyncConfig {
            enabled: Some(false),
        };
        assert_eq!(config_disabled.enabled, Some(false));
        let config_default = RelaySyncConfig::default();
        assert_eq!(config_default.enabled, None);
    }
}
