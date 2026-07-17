# Getting Started

Bucket Agent (`bucket`) is an open, multiprovider terminal AI coding agent. It runs as a TUI (Terminal User Interface) that understands your codebase, executes shell commands, edits files, searches the web, and manages tasks.

You can use it interactively as a full-screen TUI, run it headlessly for scripting and CI/CD, or integrate it into editors via the Agent Client Protocol (ACP).

**No xAI account is required.** Use it with Ollama locally, or any OpenAI-compatible backend.

---

## Building from source

Requirements:

- **Rust** — the toolchain is pinned by `rust-toolchain.toml`; `rustup` installs it automatically.
- **[DotSlash](https://dotslash-cli.com)** — install it and ensure `dotslash` is on your `PATH` before building.

```sh
cargo install dotslash

# Build + launch the TUI
cargo run -p xai-grok-pager-bin

# Release binary: target/release/bucket
cargo build -p xai-grok-pager-bin --release
```

---

## First Launch

```bash
bucket
```

Bucket drops straight to the welcome screen. No browser. No account. To make it useful, configure a model (see below).

---

## Quickstart with Ollama

The fastest path to a working local agent:

```sh
# 1. Start Ollama
ollama serve
ollama pull qwen2.5-coder:latest

# 2. Configure Bucket
mkdir -p ~/.bucket
cat > ~/.bucket/config.toml <<'EOF'
[models]
default = "ollama-coder"

[model.ollama-coder]
model    = "qwen2.5-coder:latest"
base_url = "http://localhost:11434/v1"
name     = "Qwen 2.5 Coder (Ollama)"
EOF

# 3. Launch
bucket
```

Press `Enter` and start coding. No API keys. No account.

---

## Quickstart with xAI API Key

If you have an [xAI API key](https://console.x.ai):

```sh
export XAI_API_KEY="xai-..."
bucket
```

Bucket detects the key automatically and uses the default `grok-build` model.

---

## Basic Interaction

Once a model is configured, Bucket presents a full-screen TUI with two main areas:

- **Scrollback** — the conversation history showing your prompts, Bucket's responses, tool calls, file edits, and more.
- **Prompt** — the input area at the bottom where you type messages.

Type a message and press `Enter` to send it. Bucket reads files, runs commands, and edits code as needed.

Press `Tab` to move focus between the prompt and the scrollback. While a turn is running, `Ctrl+C` cancels it. Idle, press `Esc` twice within 800ms to clear a non-empty prompt.

### File References

Use `@` in your prompt to attach files:

```
@src/main.rs              # Attach a file
@src/main.rs:10-50        # Attach lines 10-50
@src/                     # Browse a directory
```

### Permissions

By default, Bucket asks for permission before executing shell commands or editing files:

- Press `Ctrl+O` to toggle always-approve mode
- Use the `--yolo` flag at launch: `bucket --yolo`
- Type `/always-approve` in the prompt to toggle the mode

---

## Key Concepts

### Sessions

Every conversation is a **session**. Sessions are saved automatically and can be resumed:

- Start a new session: `Ctrl+N` or `/new`
- Resume a previous session: `/resume` or `--resume <ID>`
- Continue the most recent session: `bucket -c`

### Slash Commands

Type `/` in the prompt to access commands:

```
/model ollama-coder        # Switch model
/compact                   # Compress conversation history
/always-approve            # Toggle always-approve mode
/new                       # Start a new session
```

See [Slash Commands](04-slash-commands.md) for the complete reference.

### Tools

Bucket has built-in tools for:

| Tool | Description |
|------|-------------|
| `read_file` / `search_replace` | Read and edit files with line-precise changes |
| `grep` | Regex search across your codebase (powered by ripgrep) |
| `list_dir` | List directory contents |
| `run_terminal_command` | Execute shell commands |
| `web_search` / `web_fetch` | Search the web and fetch URLs |
| `todo_write` | Create and manage task lists |
| `spawn_subagent` | Spawn parallel subagent sessions |
| `memory_search` | Search cross-session memory |

---

## Common Launch Options

```bash
# Submit an initial prompt as the first turn
bucket "fix the failing auth test and run it"

# Start in a specific directory
bucket --cwd ~/projects/my-app

# Add project-specific rules
bucket --rules "Always use TypeScript. Prefer functional components."

# Auto-approve all tool executions
bucket --yolo

# Use a specific model
bucket -m ollama-coder

# Resume a previous session
bucket --resume <session-id>

# Continue the most recent session
bucket -c

# Headless mode (for scripts)
bucket -p "Explain this codebase"
```

---

## Headless Mode

Run Bucket non-interactively for scripting, CI/CD, and automation:

```bash
bucket -p "Your prompt here"
```

Output formats:

| Format | Flag | Description |
|--------|------|-------------|
| `plain` | (default) | Human-readable text |
| `json` | `--output-format json` | Single JSON object with `text`, `stopReason`, `sessionId`, and `requestId` |
| `streaming-json` | `--output-format streaming-json` | NDJSON event stream for real-time processing |

Example CI/CD usage:

```bash
bucket -p "Review changes for bugs" --output-format json --yolo | jq -r '.text'
```

---

## Project Rules (AGENTS.md)

Add per-project instructions by creating an `AGENTS.md` file in your repository:

```
~/.bucket/AGENTS.md           # Global rules (apply to all projects)
<repo-root>/AGENTS.md         # Repository-level rules
<cwd>/AGENTS.md               # Directory-level rules (highest priority)
```

Deeper files take precedence. Bucket also reads `CLAUDE.md` files for compatibility.

---

## Where to Go Next

| Document | What You Will Learn |
|----------|-------------------|
| [Providers & Authentication](02-authentication.md) | Ollama, API keys, OIDC, external auth, device code flow |
| [Keyboard Shortcuts](03-keyboard-shortcuts.md) | Complete reference for all key bindings |
| [Slash Commands](04-slash-commands.md) | All available `/` commands |
| [Configuration](05-configuration.md) | config.toml, pager.toml, environment variables |
| [Custom Models](11-custom-models.md) | Ollama, OpenAI-compatible endpoints, BYOK |
