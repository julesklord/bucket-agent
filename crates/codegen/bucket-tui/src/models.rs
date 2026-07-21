//! `bucket models` subcommand.

use anyhow::Result;
use bucket_agent_core::agent::config::{Config as AgentConfig, resolve_credentials, resolve_model_list};
use bucket_agent_core::cli_models::{AuthStatus, list_models};
use tokio_util::sync::CancellationToken;

use crate::app::cli::ModelsArgs;
use crate::client_identity::{PAGER_CLIENT_TYPE, PAGER_CLIENT_VERSION};

pub async fn run_models_command(agent_config: &AgentConfig, args: &ModelsArgs) -> Result<()> {
    match &args.diagnose {
        Some(model_id) => diagnose_model(agent_config, model_id.as_deref()),
        None => list_available_models(agent_config).await,
    }
}

async fn list_available_models(agent_config: &AgentConfig) -> Result<()> {
    match AuthStatus::resolve(agent_config) {
        AuthStatus::ApiKey => println!("You are using BUCKET_API_KEY."),
        AuthStatus::LoggedIn(host) => println!("You are logged in with {}.", host),
        AuthStatus::ModelCredentials(model) => {
            println!("Model '{model}' is using its own API key.");
        }
        AuthStatus::DeploymentKey => println!("You are authenticated via deployment key."),
        AuthStatus::NotAuthenticated => println!("You are not authenticated."),
    }
    println!();

    let cancel = CancellationToken::new();
    let spawned =
        crate::acp::spawn::spawn_bucket_shell(agent_config.clone(), &cancel, None).await?;

    let state = list_models(&spawned.channel.tx, PAGER_CLIENT_TYPE, PAGER_CLIENT_VERSION).await?;

    println!("Default model: {}", state.current_model_id.0);
    println!();
    println!("Available models:");
    for m in state.available_models {
        if m.model_id == state.current_model_id {
            println!("  * {} (default)", m.model_id.0);
        } else {
            println!("  - {}", m.model_id.0);
        }
    }

    cancel.cancel();
    Ok(())
}

/// Print routing/auth diagnostics for a single model (or the default).
fn diagnose_model(agent_config: &AgentConfig, model_id: Option<&str>) -> Result<()> {
    let models = resolve_model_list(agent_config, None);
    let target = match model_id {
        Some(id) => id.to_owned(),
        None => agent_config
            .models
            .default
            .clone()
            .unwrap_or_else(|| "<none>".to_owned()),
    };

    let model = match models.get(&target) {
        Some(m) => m,
        None => {
            eprintln!("Model '{target}' not found in config.");
            eprintln!();
            eprintln!("Available models:");
            for name in models.keys() {
                eprintln!("  - {name}");
            }
            std::process::exit(1);
        }
    };

    let info = model.info();
    let backend_str = serde_json::to_string(&info.api_backend)
        .unwrap_or_default()
        .trim_matches('"')
        .to_owned();
    let endpoint = format!(
        "{}/{}",
        info.base_url.trim_end_matches('/'),
        match backend_str.as_str() {
            "chat_completions" => "chat/completions",
            "responses" => "responses",
            "messages" => "messages",
            _ => "chat/completions",
        }
    );
    let auth_str = serde_json::to_string(&info.auth_scheme)
        .unwrap_or_default()
        .trim_matches('"')
        .to_owned();

    println!("Model:    {target}");
    println!("  model:          {}", info.model);
    if let Some(ref name) = info.name {
        println!("  name:           {name}");
    }
    println!("  base_url:       {}", info.base_url);
    println!("  endpoint:       {endpoint}");
    println!("  api_backend:    {backend_str}");
    println!("  auth_scheme:    {auth_str}");
    println!("  context_window: {}", info.context_window);
    if let Some(max) = info.max_completion_tokens {
        println!("  max_tokens:     {max}");
    }
    if !info.extra_headers.is_empty() {
        println!("  extra_headers:");
        for (k, v) in &info.extra_headers {
            println!("    {k}: {v}");
        }
    }

    let has_key = model.has_own_credentials();
    println!("  has_api_key:    {has_key}");

    let creds = resolve_credentials(model, None);
    println!("  resolved:");
    println!("    auth_type:    {:?}", creds.auth_type);
    println!("    auth_scheme:  {:?}", creds.auth_scheme);
    println!("    base_url:     {}", creds.base_url);
    println!("    has_key:      {}", creds.api_key.is_some());

    Ok(())
}
