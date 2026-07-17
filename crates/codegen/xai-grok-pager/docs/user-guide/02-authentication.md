# Providers & Authentication

Bucket Agent is multiprovider by design. It works without any authentication (local Ollama), with an API key, or with enterprise SSO. This guide explains all available options.

---

## Local Provider (No Auth Required)

No account, no API key. Just configure a local model and go:

```toml
# ~/.bucket/config.toml
[models]
default = "ollama-coder"

[model.ollama-coder]
model    = "qwen2.5-coder:latest"
base_url = "http://localhost:11434/v1"
name     = "Qwen 2.5 Coder (Ollama)"
```

```bash
ollama serve
ollama pull qwen2.5-coder:latest
bucket
```

Bucket detects non-xAI endpoints automatically and skips the login screen entirely.

---

## API Key (xAI or any provider)

For cloud providers or CI/CD environments:

```bash
# xAI
export XAI_API_KEY="xai-..."
bucket

# Any provider (per-model config)
```

```toml
# ~/.bucket/config.toml
[model.my-model]
model   = "gpt-4o"
base_url = "https://api.openai.com/v1"
env_key = "OPENAI_API_KEY"
```

---

## Browser Login (xAI / grok.com)

If you have an xAI subscription and prefer browser-based SSO:

```bash
bucket login
```

Bucket stores credentials in `~/.bucket/auth.json` and reuses them across sessions. Tokens refresh automatically. To switch accounts or sign out:

```bash
bucket logout
bucket login
```

---

## OIDC (Enterprise SSO)

Authenticate developers through your own Identity Provider (Okta, Azure AD, Auth0, etc.) instead of grok.com.

### 1. Register a public client in your IdP

- Grant type: Authorization Code with PKCE
- Redirect URI: `http://127.0.0.1/callback`
- No client secret (PKCE replaces it)

### 2. Configure the CLI

```toml
# ~/.bucket/config.toml
[grok_com_config.oidc]
issuer    = "https://acme.okta.com"
client_id = "0oa1b2c3d4e5f6g7h8i9"
```

Or via environment variables:

```bash
export GROK_OIDC_ISSUER="https://acme.okta.com"
export GROK_OIDC_CLIENT_ID="0oa1b2c3d4e5f6g7h8i9"
```

### 3. Run `bucket`

The CLI discovers endpoints via `{issuer}/.well-known/openid-configuration`, opens the IdP login page, and stores tokens in `~/.bucket/auth.json`. Tokens auto-refresh via the stored `refresh_token`.

---

## External Auth Provider

When browser-based login is impossible — sandboxed VMs, CI runners, air-gapped networks — delegate authentication to an external binary:

```
+----------------+    sh -c    +-------------------------+
|  Bucket Agent  |------------>|  your auth binary       |
|                |             |                         |
|  reads         |<-- stdout --|  prints token           |
|  auth.json     |             |                         |
|                |   (stderr)  |  prints status/URLs     |--> surfaced to user
+----------------+             +-------------------------+
```

### The stdout / stderr contract

| Stream | What to print | Who sees it |
|--------|---------------|-------------|
| **stdout** | The token — nothing else | Bucket (parsed and stored in auth.json) |
| **stderr** | Login URLs, status messages, errors | The user (shown as a clickable link in the TUI) |

### stdout token format

**Bare string:**

```
eyJhbGciOiJSUzI1NiIs...
```

**JSON** (with optional refresh token, expiry, and issuer):

```json
{"access_token": "eyJhbGciOi...", "refresh_token": "ref-tok", "expires_in": 3600, "issuer": "https://idp.example.com"}
```

### Configuration

```toml
# ~/.bucket/config.toml
[auth]
auth_provider_command = "/usr/local/bin/my-auth-provider"
auth_provider_label   = "Acme Corp"   # optional — customizes the TUI login button
auth_token_ttl        = 3600          # optional — token lifetime in seconds
```

Or via environment variables:

```bash
export GROK_AUTH_PROVIDER_COMMAND="/usr/local/bin/my-auth-provider"
export GROK_AUTH_PROVIDER_LABEL="Acme Corp"
export GROK_AUTH_TOKEN_TTL=3600
```

### Token refresh

When Bucket needs to refresh an expired token, it re-runs your binary with `GROK_AUTH_EXPIRED=1` set:

```bash
#!/bin/sh
if [ "$GROK_AUTH_EXPIRED" = "1" ]; then
    echo "Refreshing token..." >&2
    TOKEN=$(my-company-auth --refresh --silent)
else
    echo "Authenticating via Acme Corp SSO..." >&2
    TOKEN=$(my-company-auth --login --interactive)
fi

if [ -z "$TOKEN" ]; then
    echo "Authentication failed" >&2
    exit 1
fi

echo "{\"access_token\": \"$TOKEN\", \"expires_in\": 3600}"
```

---

## Device Code Flow

For headless environments (SSH, Docker, remote VMs) where no browser is available locally:

```bash
bucket login --device-auth    # or: bucket login --device-code
```

Prints a URL and code to the terminal. Open the URL on any device, enter the code, and complete authentication. Bucket polls until confirmed.

---

## Auth Precedence

Bucket resolves credentials in this order, highest to lowest:

1. **Per-model `api_key` or `env_key`** — set under `[model.<name>]` in `config.toml`. Wins whenever present.
2. **Active session token** — from browser, OIDC, or external-provider login, stored in `~/.bucket/auth.json`.
3. **`XAI_API_KEY`** — fallback when no session token is active.

When more than one login flow is configured:

1. External auth provider (`auth_provider_command`)
2. Enterprise OIDC
3. xAI/grok.com OAuth2 browser login

---

## Credential Files

| File | Contents |
|------|----------|
| `~/.bucket/auth.json` | Stored session tokens (access + refresh) |
| `~/.bucket/config.toml` | Model and auth configuration |

Bucket picks up changes to `~/.bucket/auth.json` automatically — no restart needed.

---

## Troubleshooting

### Debug logging

```bash
GROK_LOG_FILE=/tmp/bucket.log RUST_LOG=debug bucket
tail -f /tmp/bucket.log
```

In headless mode, logs go to stderr:

```bash
RUST_LOG=debug bucket -p "hello" 2> /tmp/bucket.log
```

### Common fixes

- **Login screen at startup** — configure a local model or set `XAI_API_KEY`; see [Custom Models](11-custom-models.md).
- **Token expires too quickly** — set `auth_token_ttl` or return `expires_in` in your auth provider's JSON output.
- **OIDC redirect fails** — ensure your IdP allows loopback redirect URIs (`http://127.0.0.1/callback`).
- **External auth provider not found** — check the `auth_provider_command` path is correct and the binary is executable.
