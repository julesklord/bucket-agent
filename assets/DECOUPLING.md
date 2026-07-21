> Reviewed status: 2026-07-20  
> Immediate objective: Run Bucket without mandatory login and connect third-party providers from `~/.bucket/config.toml`  
> Document scope: Inference runtime, model configuration, auth, billing, telemetry, and diagnostics

---

## Executive Summary

Bucket Agent must no longer depend on an xAI account to boot or run the basic agentic loop. The correct path for third-party providers is not to write a new plugin yet: the current path is to declare models in `~/.bucket/config.toml` under `[model.<id>]`, select the correct `api_backend`, and resolve credentials using `api_key`, `env_key`, or a global variable.

The most common failure when connecting third parties is confusing these three parts:

| Component | What it controls | Correct value |
|---|---|---|
| `base_url` | Base URL where Bucket appends the backend path | Must end in `/v1` for OpenAI- and Anthropic-style APIs |
| `api_backend` | Request shape and final endpoint | `chat_completions`, `responses`, or `messages` |
| `auth_scheme` | Authentication header | `bearer` by default, `x_api_key` for direct Anthropic |

The sampler constructs endpoints like this:

| `api_backend` | Final endpoint |
|---|---|
| `chat_completions` | `{base_url}/chat/completions` |
| `responses` | `{base_url}/responses` |
| `messages` | `{base_url}/messages` |

Therefore, `base_url = "https://api.openai.com/v1"` produces `https://api.openai.com/v1/chat/completions`, and `base_url = "https://api.anthropic.com/v1"` produces `https://api.anthropic.com/v1/messages`.

---

## Current Decoupling Status

| Area | Status | Decision |
|---|---|---|
| Mandatory login | Closed for BYOK/local use | Users can boot with their own models |
| Billing UI | Functionally closed via capabilities | TUI must hide billing when the provider does not support it |
| Third-party model config | Operational | Primary source is `[model.*]` |
| OpenAI-compatible local/remote | Operational | Use `api_backend = "chat_completions"` unless provider supports Responses |
| Direct Anthropic | Operational with correct config | Use `api_backend = "messages"` and `auth_scheme = "x_api_key"` |
| External telemetry | Must remain opt-in | Must not send data without a configured endpoint |
| Update checker | Must point to fork or be disabled | Must not depend on proprietary endpoints |
| External plugin API for providers | Not required for v1 | First consolidate the `[model.*]` contract |

---

## Multiprovider Configuration Contract

The current contract lives in `~/.bucket/config.toml`.

The `[models]` table defines global defaults. Each `[model.<id>]` defines a routable model. The `<id>` is the name appearing in the picker, in `/model`, and in `bucket -m`.

Important fields:

| Field | Required | Usage |
|---|---|---|
| `model` | Yes | Identifier sent to the provider |
| `base_url` | Yes for third-party providers | Base URL without final endpoint |
| `name` | No | Human-readable name in UI |
| `api_key` | No | Direct secret in config; useful only for local testing |
| `env_key` | No | Environment variable containing the secret; recommended |
| `api_backend` | No | Default: `chat_completions` |
| `auth_scheme` | No | Default: `bearer`; use `x_api_key` for direct Anthropic |
| `extra_headers` | No | Non-secret headers or provider-required headers |
| `context_window` | Recommended | Window size used by auto-compaction |
| `max_completion_tokens` | Recommended | Output limit per turn |
| `temperature` | No | Request temperature |
| `top_p` | No | Nucleus sampling |

Credential resolution order:

1. `api_key` in the `[model.<id>]` block
2. First non-empty variable listed in `env_key`
3. Session token, if present
4. `BUCKET_API_KEY` as global fallback

Practical recommendation: for third parties, always use `env_key`; avoid `api_key` in version-controlled files.

---

## Minimum Working Configurations

### Local Ollama

Start Ollama and pull the model:

```sh
ollama serve
ollama pull qwen2.5-coder:latest

```

Config:

```toml
[models]
default = "ollama-coder"

[model.ollama-coder]
model = "qwen2.5-coder:latest"
base_url = "http://localhost:11434/v1"
name = "Qwen 2.5 Coder (Ollama)"
api_backend = "chat_completions"
context_window = 32768
max_completion_tokens = 8192

```

Direct provider test:

```sh
curl -s http://localhost:11434/v1/models

```

Bucket test:

```sh
bucket -m ollama-coder -p "Respond with only: connected"

```

### OpenAI

Export key:

```sh
export OPENAI_API_KEY="sk-proj-replace-this-value"

```

Config:

```toml
[models]
default = "openai-gpt-4o"

[model.openai-gpt-4o]
model = "gpt-4o"
base_url = "[https://api.openai.com/v1](https://api.openai.com/v1)"
name = "GPT-4o"
env_key = "OPENAI_API_KEY"
api_backend = "chat_completions"
context_window = 128000
max_completion_tokens = 8192

```

Direct test:

```sh
curl -s [https://api.openai.com/v1/models](https://api.openai.com/v1/models) \\
  -H "Authorization: Bearer $OPENAI_API_KEY"

```

Bucket test:

```sh
bucket -m openai-gpt-4o -p "Respond with only: connected"

```

### OpenAI Responses API

Use this backend only with models and providers supporting `/v1/responses`.

```toml
[models]
default = "openai-responses"

[model.openai-responses]
model = "gpt-4.1"
base_url = "[https://api.openai.com/v1](https://api.openai.com/v1)"
name = "GPT 4.1 Responses"
env_key = "OPENAI_API_KEY"
api_backend = "responses"
context_window = 1047576
max_completion_tokens = 32768

```

Bucket test:

```sh
bucket -m openai-responses -p "Respond with only: connected"

```

### Direct Anthropic

Anthropic does not use `Authorization: Bearer` in the direct Messages API. It must use `x-api-key`.

Export key:

```sh
export ANTHROPIC_API_KEY="sk-ant-replace-this-value"

```

Config:

```toml
[models]
default = "claude-sonnet"

[model.claude-sonnet]
model = "claude-sonnet-4-5"
base_url = "[https://api.anthropic.com/v1](https://api.anthropic.com/v1)"
name = "Claude Sonnet"
env_key = "ANTHROPIC_API_KEY"
api_backend = "messages"
auth_scheme = "x_api_key"
context_window = 200000
max_completion_tokens = 8192
extra_headers = { "anthropic-version" = "2023-06-01" }

```

Direct test:

```sh
curl -s [https://api.anthropic.com/v1/messages](https://api.anthropic.com/v1/messages) \\
  -H "x-api-key: $ANTHROPIC_API_KEY" \\
  -H "anthropic-version: 2023-06-01" \\
  -H "content-type: application/json" \\
  -d '{"model":"claude-sonnet-4-5","max_tokens":16,"messages":[{"role":"user","content":"Respond with only: connected"}]}'

```

Bucket test:

```sh
bucket -m claude-sonnet -p "Respond with only: connected"

```

### OpenRouter

OpenRouter uses an OpenAI Chat Completions-compatible endpoint.

```sh
export OPENROUTER_API_KEY="sk-or-replace-this-value"

```

```toml
[models]
default = "openrouter-claude"

[model.openrouter-claude]
model = "anthropic/claude-3.5-sonnet"
base_url = "[https://openrouter.ai/api/v1](https://openrouter.ai/api/v1)"
name = "Claude via OpenRouter"
env_key = "OPENROUTER_API_KEY"
api_backend = "chat_completions"
context_window = 200000
max_completion_tokens = 8192
extra_headers = { "HTTP-Referer" = "[https://github.com/julesklord/bucket-agent](https://github.com/julesklord/bucket-agent)", "X-Title" = "Bucket Agent" }

```

Direct test:

```sh
curl -s [https://openrouter.ai/api/v1/models](https://openrouter.ai/api/v1/models) \\
  -H "Authorization: Bearer $OPENROUTER_API_KEY"

```

Bucket test:

```sh
bucket -m openrouter-claude -p "Respond with only: connected"

```

### Together AI

```sh
export TOGETHER_API_KEY="replace-this-value"

```

```toml
[models]
default = "together-qwen"

[model.together-qwen]
model = "Qwen/Qwen2.5-Coder-32B-Instruct"
base_url = "[https://api.together.xyz/v1](https://api.together.xyz/v1)"
name = "Qwen Coder via Together"
env_key = "TOGETHER_API_KEY"
api_backend = "chat_completions"
context_window = 32768
max_completion_tokens = 8192

```

Direct test:

```sh
curl -s [https://api.together.xyz/v1/models](https://api.together.xyz/v1/models) \\
  -H "Authorization: Bearer $TOGETHER_API_KEY"

```

Bucket test:

```sh
bucket -m together-qwen -p "Respond with only: connected"

```

### LM Studio

LM Studio exposes an OpenAI-compatible local server.

```toml
[models]
default = "lmstudio-local"

[model.lmstudio-local]
model = "local-model"
base_url = "http://localhost:1234/v1"
name = "LM Studio Local"
api_backend = "chat_completions"
context_window = 32768
max_completion_tokens = 8192

```

Direct test:

```sh
curl -s http://localhost:1234/v1/models

```

Bucket test:

```sh
bucket -m lmstudio-local -p "Respond with only: connected"

```

---

## Recommended Complete File for Testing

This file allows testing a local provider, an OpenAI-compatible provider, and direct Anthropic without mixing secrets into TOML.

```toml
[models]
default = "ollama-coder"

[model.ollama-coder]
model = "qwen2.5-coder:latest"
base_url = "http://localhost:11434/v1"
name = "Qwen 2.5 Coder (Ollama)"
api_backend = "chat_completions"
context_window = 32768
max_completion_tokens = 8192

[model.openai-gpt-4o]
model = "gpt-4o"
base_url = "[https://api.openai.com/v1](https://api.openai.com/v1)"
name = "GPT-4o"
env_key = "OPENAI_API_KEY"
api_backend = "chat_completions"
context_window = 128000
max_completion_tokens = 8192

[model.claude-sonnet]
model = "claude-sonnet-4-5"
base_url = "[https://api.anthropic.com/v1](https://api.anthropic.com/v1)"
name = "Claude Sonnet"
env_key = "ANTHROPIC_API_KEY"
api_backend = "messages"
auth_scheme = "x_api_key"
context_window = 200000
max_completion_tokens = 8192
extra_headers = { "anthropic-version" = "2023-06-01" }

```

Validation:

```sh
bucket models
bucket -m ollama-coder -p "Respond with only: ollama ok"
bucket -m openai-gpt-4o -p "Respond with only: openai ok"
bucket -m claude-sonnet -p "Respond with only: anthropic ok"

```

---

## Diagnostic Checklist

### 1. Confirm Bucket sees the model

```sh
bucket models

```

If the model does not appear:

| Cause | Fix |
| --- | --- |
| Miswritten TOML header | Use `[model.my-id]`, not `[models.my-id]` |
| Invalid `context_window` | Use a positive integer |
| Invalid `api_backend` | Use `chat_completions`, `responses`, or `messages` |
| File in wrong path | Must be located at `~/.bucket/config.toml` |

### 2. Confirm environment variable exists

```sh
printenv OPENAI_API_KEY
printenv ANTHROPIC_API_KEY
printenv OPENROUTER_API_KEY
printenv TOGETHER_API_KEY

```

If `printenv` returns nothing, Bucket will not see the key either. Export it in the same shell where you run `bucket`.

### 3. Confirm endpoint responds

For OpenAI-compatible:

```sh
curl -i [https://api.openai.com/v1/models](https://api.openai.com/v1/models) \\
  -H "Authorization: Bearer $OPENAI_API_KEY"

```

For Anthropic:

```sh
curl -i [https://api.anthropic.com/v1/messages](https://api.anthropic.com/v1/messages) \\
  -H "x-api-key: $ANTHROPIC_API_KEY" \\
  -H "anthropic-version: 2023-06-01" \\
  -H "content-type: application/json" \\
  -d '{"model":"claude-sonnet-4-5","max_tokens":16,"messages":[{"role":"user","content":"ping"}]}'

```

For Ollama:

```sh
curl -i http://localhost:11434/v1/models

```

### 4. Enable sampler logs

```sh
RUST_LOG=bucket_sampler=debug,bucket_agent_core=debug BUCKET_LOG_FILE=/tmp/bucket.log bucket -m ollama-coder -p "ping"
tail -n 200 /tmp/bucket.log

```

Look for these fields:

| Log field | Interpretation |
| --- | --- |
| `base_url` | Must be the provider host |
| `model` | Must be the model ID sent to the provider |
| `api_backend` | Must match the expected endpoint |
| `auth_scheme` | `Bearer` or `XApiKey` depending on provider |
| `has_api_key` | Must be `true` for remote providers requiring auth |
| `has_authorization_header` | Must be `true` for OpenAI-compatible bearer |
| `has_x_api_key_header` | Must be `true` for direct Anthropic |

### 5. Read error by type

| Error | Probable meaning | Fix |
| --- | --- | --- |
| 401 Unauthorized | Missing or invalid key, or incorrect header | Check `env_key`, `auth_scheme`, and current shell |
| 404 Not Found | Incorrect `base_url` or incorrect backend | Ensure `/v1` and the correct `api_backend` |
| 400 Bad Request | Payload incompatible with provider | Change backend or model; try `chat_completions` first |
| 429 Rate Limit | Provider rate limit reached | Change model, account, or wait |
| Connection refused | Local provider stopped or wrong port | Start Ollama/LM Studio and verify port |
| Serialization error | Provider does not return expected format | Use a real compatible backend |

---

## Closure Decisions

### 1. The public `ChatProvider` trait does not block v1

The original plan proposed extracting a public `ChatProvider` trait. That remains desirable for a mature plugin API, but does not block current third-party connectivity. The runtime already routes via model configuration and `bucket-sampler`.

Decision: v1 closes with `[model.*]` as the stable user contract. The public trait is deferred to a later phase.

### 2. `ProviderCapabilities` is already the right boundary for UI

The UI must not deduce billing from the model name or host. It must inspect capabilities. If a provider does not report billing, credit bars, subscription gates, and commercial banners are hidden.

Decision: Retain `ProviderCapabilities` and treat any BYOK/local provider as having no billing.

### 3. Anthropic requires `auth_scheme = "x_api_key"`

Setting `api_backend = "messages"` alone is insufficient. Authentication must switch from bearer to `x-api-key`.

Decision: All official documentation must present Anthropic like this:

```toml
api_backend = "messages"
auth_scheme = "x_api_key"
env_key = "ANTHROPIC_API_KEY"
extra_headers = { "anthropic-version" = "2023-06-01" }

```

### 4. `extra_headers` should not be the primary place for secrets

Although `extra_headers` works for custom headers, primary keys should enter via `api_key` or `env_key` so the sampler can properly record auth state and apply `auth_scheme`.

Decision: Use `extra_headers` for version, routing, or metadata headers; use `env_key` for keys.

### 5. Global default `BUCKET_API_KEY` is a fallback, not multiprovider config

`BUCKET_API_KEY` exists for compatibility. For multiple providers simultaneously, each model must declare its own `env_key`.

Decision: Multiprovider configs must prefer `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `OPENROUTER_API_KEY`, `TOGETHER_API_KEY`, or per-model custom names.

---

## Post-Closure Action Items

| Priority | Task | Expected Result | Status |
| --- | --- | --- | --- |
| High | Add direct Anthropic config test | Guarantee `env_key` flows to `x-api-key` header | Done (3 E2E tests) |
| Medium | Add model diagnostic command | Display final URL, backend, auth scheme, and non-sensitive headers | Done (`bucket models --diagnose`) |
| Medium | Improve error handling for wrong backend | Concrete 404/400 messages suggesting correct `api_backend` | Done |
| Medium | Decouple Voice/STT from hardcoded `api.x.ai` | Configurable via `[voice].api_base` or `BUCKET_VOICE_API_BASE` | Done |
| Medium | Replace xAI-specific strings | Remove `console.x.ai` and `docs.x.ai` hardcoded refs | Done |
| Low | Design provider plugin API | Allow external providers outside the main binary | Pending |

---

## Definition of Done

Decoupling v1 is considered done when these tests pass on a clean machine:

```sh
bucket models
bucket -m ollama-coder -p "Respond with only: ok"
bucket -m openai-gpt-4o -p "Respond with only: ok"
bucket -m claude-sonnet -p "Respond with only: ok"

```

And the following conditions are met:

| Condition | Expected State |
| --- | --- |
| Boot without login | Bucket opens without requiring a browser |
| Local model | Ollama responds via `/v1/chat/completions` |
| Remote OpenAI-compatible | Auth via `Authorization: Bearer` |
| Direct Anthropic | Auth via `x-api-key` |
| Billing | Does not appear for BYOK/local |
| Telemetry | Off by default |
| Logs | Allow inspecting `base_url`, `model`, `api_backend`, and `auth_scheme` |

With this, Bucket remains usable as a multiprovider agent without relying on proprietary infrastructure for the main inference flow.
"""

file_path = "bucket-agent-decoupling-report-en.md"
with open(file_path, "w", encoding="utf-8") as f:
f.write(content)

print(f"File saved successfully: {file_path}")

```

```text?code_stdout&code_event_index=1
File saved successfully: bucket-agent-decoupling-report-en.md


```

Your Markdown file is ready


# Bucket Agent - Decoupling Completion and Providers

> Reviewed status: 2026-07-20
> Immediate objective: Run Bucket without mandatory login and connect third-party providers from `~/.bucket/config.toml`
> Document scope: Inference runtime, model configuration, auth, billing, telemetry, and diagnostics

---

## Executive Summary

Bucket Agent must no longer depend on an xAI account to boot or run the basic agentic loop. The correct path for third-party providers is not to write a new plugin yet: the current path is to declare models in `~/.bucket/config.toml` under `[model.<id>]`, select the correct `api_backend`, and resolve credentials using `api_key`, `env_key`, or a global variable.

The most common failure when connecting third parties is confusing these three parts:

| Component | What it controls | Correct value |
| --- | --- | --- |
| `base_url` | Base URL where Bucket appends the backend path | Must end in `/v1` for OpenAI- and Anthropic-style APIs |
| `api_backend` | Request shape and final endpoint | `chat_completions`, `responses`, or `messages` |
| `auth_scheme` | Authentication header | `bearer` by default, `x_api_key` for direct Anthropic |

The sampler constructs endpoints like this:

| `api_backend` | Final endpoint |
| --- | --- |
| `chat_completions` | `{base_url}/chat/completions` |
| `responses` | `{base_url}/responses` |
| `messages` | `{base_url}/messages` |

Therefore, `base_url = "[https://api.openai.com/v1](https://api.openai.com/v1)"` produces `[https://api.openai.com/v1/chat/completions](https://api.openai.com/v1/chat/completions)`, and `base_url = "[https://api.anthropic.com/v1](https://api.anthropic.com/v1)"` produces `[https://api.anthropic.com/v1/messages](https://api.anthropic.com/v1/messages)`.

---

## Current Decoupling Status

| Area | Status | Decision |
| --- | --- | --- |
| Mandatory login | Closed for BYOK/local use | Users can boot with their own models |
| Billing UI | Functionally closed via capabilities | TUI must hide billing when the provider does not support it |
| Third-party model config | Operational | Primary source is `[model.*]` |
| OpenAI-compatible local/remote | Operational | Use `api_backend = "chat_completions"` unless provider supports Responses |
| Direct Anthropic | Operational with correct config | Use `api_backend = "messages"` and `auth_scheme = "x_api_key"` |
| External telemetry | Must remain opt-in | Must not send data without a configured endpoint |
| Update checker | Must point to fork or be disabled | Must not depend on proprietary endpoints |
| External plugin API for providers | Not required for v1 | First consolidate the `[model.*]` contract |

---

## Multiprovider Configuration Contract

The current contract lives in `~/.bucket/config.toml`.

The `[models]` table defines global defaults. Each `[model.<id>]` defines a routable model. The `<id>` is the name appearing in the picker, in `/model`, and in `bucket -m`.

Important fields:

| Field | Required | Usage |
| --- | --- | --- |
| `model` | Yes | Identifier sent to the provider |
| `base_url` | Yes for third-party providers | Base URL without final endpoint |
| `name` | No | Human-readable name in UI |
| `api_key` | No | Direct secret in config; useful only for local testing |
| `env_key` | No | Environment variable containing the secret; recommended |
| `api_backend` | No | Default: `chat_completions` |
| `auth_scheme` | No | Default: `bearer`; use `x_api_key` for direct Anthropic |
| `extra_headers` | No | Non-secret headers or provider-required headers |
| `context_window` | Recommended | Window size used by auto-compaction |
| `max_completion_tokens` | Recommended | Output limit per turn |
| `temperature` | No | Request temperature |
| `top_p` | No | Nucleus sampling |

Credential resolution order:

1. `api_key` in the `[model.<id>]` block
2. First non-empty variable listed in `env_key`
3. Session token, if present
4. `BUCKET_API_KEY` as global fallback

Practical recommendation: for third parties, always use `env_key`; avoid `api_key` in version-controlled files.

---

## Minimum Working Configurations

### Local Ollama

Start Ollama and pull the model:

```sh
ollama serve
ollama pull qwen2.5-coder:latest

```

Config:

```toml
[models]
default = "ollama-coder"

[model.ollama-coder]
model = "qwen2.5-coder:latest"
base_url = "http://localhost:11434/v1"
name = "Qwen 2.5 Coder (Ollama)"
api_backend = "chat_completions"
context_window = 32768
max_completion_tokens = 8192

```

Direct provider test:

```sh
curl -s http://localhost:11434/v1/models

```

Bucket test:

```sh
bucket -m ollama-coder -p "Respond with only: connected"

```

### OpenAI

Export key:

```sh
export OPENAI_API_KEY="sk-proj-replace-this-value"

```

Config:

```toml
[models]
default = "openai-gpt-4o"

[model.openai-gpt-4o]
model = "gpt-4o"
base_url = "https://api.openai.com/v1"
name = "GPT-4o"
env_key = "OPENAI_API_KEY"
api_backend = "chat_completions"
context_window = 128000
max_completion_tokens = 8192

```

Direct test:

```sh
curl -s https://api.openai.com/v1/models \
  -H "Authorization: Bearer $OPENAI_API_KEY"

```

Bucket test:

```sh
bucket -m openai-gpt-4o -p "Respond with only: connected"

```

### OpenAI Responses API

Use this backend only with models and providers supporting `/v1/responses`.

```toml
[models]
default = "openai-responses"

[model.openai-responses]
model = "gpt-4.1"
base_url = "https://api.openai.com/v1"
name = "GPT 4.1 Responses"
env_key = "OPENAI_API_KEY"
api_backend = "responses"
context_window = 1047576
max_completion_tokens = 32768

```

Bucket test:

```sh
bucket -m openai-responses -p "Respond with only: connected"

```

### Direct Anthropic

Anthropic does not use `Authorization: Bearer` in the direct Messages API. It must use `x-api-key`.

Export key:

```sh
export ANTHROPIC_API_KEY="sk-ant-replace-this-value"

```

Config:

```toml
[models]
default = "claude-sonnet"

[model.claude-sonnet]
model = "claude-sonnet-4-5"
base_url = "https://api.anthropic.com/v1"
name = "Claude Sonnet"
env_key = "ANTHROPIC_API_KEY"
api_backend = "messages"
auth_scheme = "x_api_key"
context_window = 200000
max_completion_tokens = 8192
extra_headers = { "anthropic-version" = "2023-06-01" }

```

Direct test:

```sh
curl -s https://api.anthropic.com/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{"model":"claude-sonnet-4-5","max_tokens":16,"messages":[{"role":"user","content":"Respond with only: connected"}]}'

```

Bucket test:

```sh
bucket -m claude-sonnet -p "Respond with only: connected"

```

### OpenRouter

OpenRouter uses an OpenAI Chat Completions-compatible endpoint.

```sh
export OPENROUTER_API_KEY="sk-or-replace-this-value"

```

```toml
[models]
default = "openrouter-claude"

[model.openrouter-claude]
model = "anthropic/claude-3.5-sonnet"
base_url = "https://openrouter.ai/api/v1"
name = "Claude via OpenRouter"
env_key = "OPENROUTER_API_KEY"
api_backend = "chat_completions"
context_window = 200000
max_completion_tokens = 8192
extra_headers = { "HTTP-Referer" = "https://github.com/julesklord/bucket-agent", "X-Title" = "Bucket Agent" }

```

Direct test:

```sh
curl -s https://openrouter.ai/api/v1/models \
  -H "Authorization: Bearer $OPENROUTER_API_KEY"

```

Bucket test:

```sh
bucket -m openrouter-claude -p "Respond with only: connected"

```

### Together AI

```sh
export TOGETHER_API_KEY="replace-this-value"

```

```toml
[models]
default = "together-qwen"

[model.together-qwen]
model = "Qwen/Qwen2.5-Coder-32B-Instruct"
base_url = "https://api.together.xyz/v1"
name = "Qwen Coder via Together"
env_key = "TOGETHER_API_KEY"
api_backend = "chat_completions"
context_window = 32768
max_completion_tokens = 8192

```

Direct test:

```sh
curl -s https://api.together.xyz/v1/models \
  -H "Authorization: Bearer $TOGETHER_API_KEY"

```

Bucket test:

```sh
bucket -m together-qwen -p "Respond with only: connected"

```

### LM Studio

LM Studio exposes an OpenAI-compatible local server.

```toml
[models]
default = "lmstudio-local"

[model.lmstudio-local]
model = "local-model"
base_url = "http://localhost:1234/v1"
name = "LM Studio Local"
api_backend = "chat_completions"
context_window = 32768
max_completion_tokens = 8192

```

Direct test:

```sh
curl -s http://localhost:1234/v1/models

```

Bucket test:

```sh
bucket -m lmstudio-local -p "Respond with only: connected"

```

---

## Recommended Complete File for Testing

This file allows testing a local provider, an OpenAI-compatible provider, and direct Anthropic without mixing secrets into TOML.

```toml
[models]
default = "ollama-coder"

[model.ollama-coder]
model = "qwen2.5-coder:latest"
base_url = "http://localhost:11434/v1"
name = "Qwen 2.5 Coder (Ollama)"
api_backend = "chat_completions"
context_window = 32768
max_completion_tokens = 8192

[model.openai-gpt-4o]
model = "gpt-4o"
base_url = "https://api.openai.com/v1"
name = "GPT-4o"
env_key = "OPENAI_API_KEY"
api_backend = "chat_completions"
context_window = 128000
max_completion_tokens = 8192

[model.claude-sonnet]
model = "claude-sonnet-4-5"
base_url = "https://api.anthropic.com/v1"
name = "Claude Sonnet"
env_key = "ANTHROPIC_API_KEY"
api_backend = "messages"
auth_scheme = "x_api_key"
context_window = 200000
max_completion_tokens = 8192
extra_headers = { "anthropic-version" = "2023-06-01" }

```

Validation:

```sh
bucket models
bucket -m ollama-coder -p "Respond with only: ollama ok"
bucket -m openai-gpt-4o -p "Respond with only: openai ok"
bucket -m claude-sonnet -p "Respond with only: anthropic ok"

```

---

## Diagnostic Checklist

### 1. Confirm Bucket sees the model

```sh
bucket models

```

If the model does not appear:

| Cause | Fix |
| --- | --- |
| Miswritten TOML header | Use `[model.my-id]`, not `[models.my-id]` |
| Invalid `context_window` | Use a positive integer |
| Invalid `api_backend` | Use `chat_completions`, `responses`, or `messages` |
| File in wrong path | Must be located at `~/.bucket/config.toml` |

### 2. Confirm environment variable exists

```sh
printenv OPENAI_API_KEY
printenv ANTHROPIC_API_KEY
printenv OPENROUTER_API_KEY
printenv TOGETHER_API_KEY

```

If `printenv` returns nothing, Bucket will not see the key either. Export it in the same shell where you run `bucket`.

### 3. Confirm endpoint responds

For OpenAI-compatible:

```sh
curl -i https://api.openai.com/v1/models \
  -H "Authorization: Bearer $OPENAI_API_KEY"

```

For Anthropic:

```sh
curl -i https://api.anthropic.com/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{"model":"claude-sonnet-4-5","max_tokens":16,"messages":[{"role":"user","content":"ping"}]}'

```

For Ollama:

```sh
curl -i http://localhost:11434/v1/models

```

### 4. Enable sampler logs

```sh
RUST_LOG=bucket_sampler=debug,bucket_agent_core=debug BUCKET_LOG_FILE=/tmp/bucket.log bucket -m ollama-coder -p "ping"
tail -n 200 /tmp/bucket.log

```

Look for these fields:

| Log field | Interpretation |
| --- | --- |
| `base_url` | Must be the provider host |
| `model` | Must be the model ID sent to the provider |
| `api_backend` | Must match the expected endpoint |
| `auth_scheme` | `Bearer` or `XApiKey` depending on provider |
| `has_api_key` | Must be `true` for remote providers requiring auth |
| `has_authorization_header` | Must be `true` for OpenAI-compatible bearer |
| `has_x_api_key_header` | Must be `true` for direct Anthropic |

### 5. Read error by type

| Error | Probable meaning | Fix |
| --- | --- | --- |
| 401 Unauthorized | Missing or invalid key, or incorrect header | Check `env_key`, `auth_scheme`, and current shell |
| 404 Not Found | Incorrect `base_url` or incorrect backend | Ensure `/v1` and the correct `api_backend` |
| 400 Bad Request | Payload incompatible with provider | Change backend or model; try `chat_completions` first |
| 429 Rate Limit | Provider rate limit reached | Change model, account, or wait |
| Connection refused | Local provider stopped or wrong port | Start Ollama/LM Studio and verify port |
| Serialization error | Provider does not return expected format | Use a real compatible backend |

---

## Closure Decisions

### 1. The public `ChatProvider` trait does not block v1

The original plan proposed extracting a public `ChatProvider` trait. That remains desirable for a mature plugin API, but does not block current third-party connectivity. The runtime already routes via model configuration and `bucket-sampler`.

Decision: v1 closes with `[model.*]` as the stable user contract. The public trait is deferred to a later phase.

### 2. `ProviderCapabilities` is already the right boundary for UI

The UI must not deduce billing from the model name or host. It must inspect capabilities. If a provider does not report billing, credit bars, subscription gates, and commercial banners are hidden.

Decision: Retain `ProviderCapabilities` and treat any BYOK/local provider as having no billing.

### 3. Anthropic requires `auth_scheme = "x_api_key"`

Setting `api_backend = "messages"` alone is insufficient. Authentication must switch from bearer to `x-api-key`.

Decision: All official documentation must present Anthropic like this:

```toml
api_backend = "messages"
auth_scheme = "x_api_key"
env_key = "ANTHROPIC_API_KEY"
extra_headers = { "anthropic-version" = "2023-06-01" }

```

### 4. `extra_headers` should not be the primary place for secrets

Although `extra_headers` works for custom headers, primary keys should enter via `api_key` or `env_key` so the sampler can properly record auth state and apply `auth_scheme`.

Decision: Use `extra_headers` for version, routing, or metadata headers; use `env_key` for keys.

### 5. Global default `BUCKET_API_KEY` is a fallback, not multiprovider config

`BUCKET_API_KEY` exists for compatibility. For multiple providers simultaneously, each model must declare its own `env_key`.

Decision: Multiprovider configs must prefer `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `OPENROUTER_API_KEY`, `TOGETHER_API_KEY`, or per-model custom names.

---

## Post-Closure Action Items

| Priority | Task | Expected Result | Status |
| --- | --- | --- | --- |
| High | Add direct Anthropic config test | Guarantee `env_key` flows to `x-api-key` header | Done (3 E2E tests) |
| Medium | Add model diagnostic command | Display final URL, backend, auth scheme, and non-sensitive headers | Done (`bucket models --diagnose`) |
| Medium | Improve error handling for wrong backend | Concrete 404/400 messages suggesting correct `api_backend` | Done |
| Medium | Decouple Voice/STT from hardcoded `api.x.ai` | Configurable via `[voice].api_base` or `BUCKET_VOICE_API_BASE` | Done |
| Medium | Replace xAI-specific strings | Remove `console.x.ai` and `docs.x.ai` hardcoded refs | Done |
| Low | Design provider plugin API | Allow external providers outside the main binary | Pending |

---

## Definition of Done

Decoupling v1 is considered done when these tests pass on a clean machine:

```sh
bucket models
bucket -m ollama-coder -p "Respond with only: ok"
bucket -m openai-gpt-4o -p "Respond with only: ok"
bucket -m claude-sonnet -p "Respond with only: ok"

```

And the following conditions are met:

| Condition | Expected State |
| --- | --- |
| Boot without login | Bucket opens without requiring a browser |
| Local model | Ollama responds via `/v1/chat/completions` |
| Remote OpenAI-compatible | Auth via `Authorization: Bearer` |
| Direct Anthropic | Auth via `x-api-key` |
| Billing | Does not appear for BYOK/local |
| Telemetry | Off by default |
| Logs | Allow inspecting `base_url`, `model`, `api_backend`, and `auth_scheme` |

With this, Bucket remains usable as a multiprovider agent without relying on proprietary infrastructure for the main inference flow.
