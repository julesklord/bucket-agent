# Custom Models

Bucket Agent is multiprovider. This guide explains how to connect to any model backend — local or remote, free or paid.

---

## Default Behavior

By default, Bucket starts without a configured model. If you set `XAI_API_KEY`, it uses xAI's `grok-build` model. Otherwise, configure a model in `~/.bucket/config.toml`.

List all available models:

```bash
bucket models
```

---

## Selecting a Model

### CLI Flag

```bash
bucket -p "Hello" -m ollama-coder
```

### Slash Command

In the TUI, switch models during a session:

```
/model ollama-coder
```

Or use the alias:

```
/m ollama-coder
```

### Model Picker (Ctrl+M)

Press `Ctrl+M` from the scrollback pane to open the model picker. It lists all available models and lets you switch with a single keystroke.

### Config Default

Set a persistent default in `~/.bucket/config.toml`:

```toml
[models]
default = "ollama-coder"
```

---

## Supported API Backends

Bucket supports three API backends, configured via `api_backend` in `[model.*]`:

| Value | API | Default |
|-------|-----|---------|
| `"chat_completions"` | OpenAI Chat Completions (`/v1/chat/completions`) | Yes |
| `"responses"` | OpenAI Responses (`/v1/responses`) | |
| `"messages"` | Anthropic Messages (`/v1/messages`) | |

---

## Configuring Custom Models

Add endpoints in `~/.bucket/config.toml` under `[model.<name>]` sections:

```toml
[model.my-model]
model                = "model-id"                       # Model identifier sent to the API
base_url             = "https://api.example.com/v1"    # OpenAI-compatible endpoint
name                 = "Display Name"                   # Shown in the model picker
description          = "Model description"              # Optional
api_key              = "sk-..."                         # API key for this provider (optional)
env_key              = "MY_API_KEY"                     # Env var holding the API key (optional)
api_backend          = "chat_completions"               # "chat_completions", "responses", or "messages"
temperature          = 0.7
top_p                = 0.95
max_completion_tokens = 8192
context_window       = 128000
extra_headers        = { "x-api-key" = "sk-..." }      # Extra request headers (optional)
```

### Credential resolution

Bucket resolves the API key in this order:

1. The `api_key` field in the model config
2. The environment variable(s) named by `env_key` — first set, non-empty value wins
3. Your signed-in session token (from `bucket login`)
4. The `XAI_API_KEY` environment variable (global fallback)

---

## Provider Examples

### Ollama (Local, no API key)

Run models locally with [Ollama](https://ollama.ai):

```toml
[model.ollama-coder]
model    = "qwen2.5-coder:latest"
base_url = "http://localhost:11434/v1"
name     = "Qwen 2.5 Coder (Ollama)"
```

Make sure Ollama is running (`ollama serve`) and the model is pulled (`ollama pull qwen2.5-coder:latest`).

Other popular models:

```toml
[model.ollama-llama]
model    = "llama3.2:latest"
base_url = "http://localhost:11434/v1"
name     = "Llama 3.2 (Ollama)"

[model.ollama-deepseek]
model    = "deepseek-coder-v2:latest"
base_url = "http://localhost:11434/v1"
name     = "DeepSeek Coder V2 (Ollama)"
```

### xAI (grok-build)

```toml
# Or just: export XAI_API_KEY="xai-..."
[model.grok-build]
model   = "grok-build"
env_key = "XAI_API_KEY"
```

### Anthropic (Claude)

```toml
[model.claude-opus]
model       = "claude-opus-4-6"
base_url    = "https://api.anthropic.com/v1"
name        = "Claude Opus 4.6"
api_backend = "messages"
context_window = 200000
extra_headers = { "x-api-key" = "sk-ant-...", "anthropic-version" = "2023-06-01" }
```

### OpenAI

```toml
[model.gpt-4o]
model   = "gpt-4o"
base_url = "https://api.openai.com/v1"
name    = "GPT-4o"
env_key = "OPENAI_API_KEY"
```

### Together AI

```toml
[model.together-mixtral]
model    = "mistralai/Mixtral-8x7B-Instruct-v0.1"
base_url = "https://api.together.xyz/v1"
name     = "Mixtral 8x7B"
env_key  = "TOGETHER_API_KEY"
```

### Any local OpenAI-compatible server

```toml
[model.local-llama]
model    = "llama-3.1-70b"
base_url = "http://localhost:8080/v1"
name     = "Local Llama"
temperature = 0.8
```

---

## Overriding Built-in Models

Override specific fields of built-in models without redefining everything:

```toml
[model.grok-build]
api_key     = "my-api-key"
temperature = 0.5
```

Unspecified fields inherit from the built-in defaults.

---

## Custom Models Endpoint

Point Bucket at a custom OpenAI-compatible `/v1/models` endpoint:

```bash
export GROK_MODELS_BASE_URL="https://api.acme.com/v1"
export XAI_API_KEY="xai-..."
bucket
```

Or via config file:

```toml
[endpoints]
models_base_url = "https://api.acme.com/v1"
```

---

## Web Search Model

Configure the model used by the `web_search` tool:

```toml
[models]
web_search = "grok-4.20-multi-agent"
```

---

## Enterprise Deployment Example

```toml
[cli]
auto_update = false

[auth]
auth_provider_command = "/usr/local/bin/my-company-auth-provider"
auth_provider_label   = "Acme Corp"
auth_token_ttl        = 3600

[models]
default = "company-model"

[model.company-model]
model          = "grok-build"
base_url       = "https://ai-proxy.acme.com/"
name           = "Bucket (Acme Proxy)"
context_window = 128000

[features]
telemetry = false
```

---

## Troubleshooting

### Model not found

```bash
bucket models   # list all configured and built-in models
```

### Connection errors

```bash
curl -s http://localhost:11434/v1/models   # Ollama
curl -s https://api.example.com/v1/models -H "Authorization: Bearer $XAI_API_KEY"
```

### Debug logging

```bash
RUST_LOG=debug GROK_LOG_FILE=/tmp/bucket.log bucket
tail -f /tmp/bucket.log
```

Look for log entries containing `model` or `sampling` to trace model selection and API calls.
