# Bucket Agent — Decoupling Plan & Progress Tracker

> **Base:** fork of xAI Bucket Build (`d5e79b1`)
> **Team:** 3-4 contributors
> **Horizon:** 6-9 months
> **Started:** 2025
> **Last updated:** 2026-07-18

---

## Current Status Summary

| Phase | Name | Status | Est. Weeks |
|-------|------|--------|------------|
| 1 | Naming Cleanup | **✅ DONE** | 2-3 |
| 2 | Auth & Billing Decoupling | **🟡 ~90%** | 3-4 |
| 3 | Agent Runtime Decoupling | **✅ DONE (100%)** | 4-6 |
| 4 | Project Infrastructure | **🟡 ~60%** | 2-3 |
| 5 | Community Extensibility | **🟢 In Progress** | ongoing |

**Key metrics (current):**
- Crates in workspace: ~75
- `.rs` files referencing `xai`, `x.ai`, `superbucket`: **0** (all cleaned)
- Phase 3 completion commit: `8c7c1af` — `feat(decoupling): complete Phase 3 agent runtime decoupling and ChatProvider integration`

---

## Phase 1 — Naming Cleanup ✅

**Goal:** No internal names reference `xai` or `bucket` except network protocol (model names, endpoints).

**Completed actions:**
- All crates renamed to `bucket-*` convention (no xAI naming remnants).
- Env vars standardized to `BUCKET_*` namespace.
- Config paths unified under `~/.bucket/`.

**Verified:** `rg` across all `.rs` files returns 0 matches for `xai`, `x.ai`, or `superbucket`.

---

## Phase 2 — Auth & Billing Decoupling 🟡 ~90%

**Goal:** Remove all logic that assumes a xAI/bucket account exists.

### 2.1 Login Screen from bucket.com — ✅ DONE
- `AuthMethodKind::BucketCom` bypassed.
- OIDC issuer made configurable via `BUCKET_OIDC_ISSUER` env var (commit `96ae86b`).
- Auth URL no longer hardcoded to `auth.x.ai`.

### 2.2 Billing/Subscription Logic (926 occurrences) — ✅ DONE
- `ProviderCapabilities` struct gates all billing UI.
- Credit bar, SuperBucket banner, subscription gate all behind `has_billing` flag.
- When `has_billing = false` (Ollama, custom): no billing UI rendered.
- Commit `3db3f80` — `feat: ProviderCapabilities gating + telemetry decoupling (Fase 2)`.

### 2.3 Update Checker — ✅ DONE
- `bucket-updater` URLs made configurable via `CliConfig`.
- Default retargeted from `x.ai/cli` to configurable endpoint.
- Commit `a8a45fa` — `feat(updater): make update URLs configurable via CliConfig (Fase 2.3)`.
- Recommended: point to GitHub Releases of the fork.

### 2.4 Telemetry — ✅ DONE
- External telemetry (OTLP) decoupled — endpoint configurable, defaults to off.
- Internal `unified_log` (structured logging) kept — it's useful.
- Commit `3db3f80` included telemetry decoupling.

### Remaining Phase 2 work:
- [ ] Rename `require_xai_auth` → `require_first_party_auth` (deprecated alias exists, callers still use old name)
- [ ] Soften remaining hardcoded `auth.x.ai` / `accounts.x.ai` defaults in auth config
- [ ] Reduce SuperBucket tier string references or make configurable

---

## Phase 3 — Agent Runtime Decoupling ✅ 100%

**Goal:** `bucket-agent-core` knows nothing about proprietary infrastructure and assumes no specific backend.

### 3.1 Chat Provider — ✅ DONE
- **`ChatProvider` trait:** `crates/codegen/bucket-agent-core/src/provider/mod.rs` (152 lines)
- **`ProviderCapabilities` struct:** `crates/codegen/bucket-agent-core/src/provider/capabilities.rs` (113 lines)
- **`AcpSession` integration:** `chat_provider: Arc<dyn ChatProvider>` in main agent session. Turns and inference dispatched through `ChatProvider` abstraction.
- **Implementations:** `SamplerProvider` (multi-process with `ApiBackend`: ChatCompletions, Messages, Responses) and `MockProvider` for tests.

### 3.2 Model System — ✅ DONE
- Hardcoded proprietary model catalog removed.
- `default_models.json` replaced with neutral config based on `config.toml` + runtime endpoint resolution (`/v1/models`).

### 3.3 System Prompts & Agent Personality — ✅ DONE
- Configurable system prompt injection (`system_prompt_label`, `system_prompt_override_from_meta`, `SessionCommand::ReplaceSystemPrompt`).
- Neutral default identity: "Bucket, an agentic coding assistant".

### 3.4 Session Serialization — ✅ DONE
- Provider-agnostic session metadata (`model_id`, `base_url`).
- Conversation format v1 (`CHAT_FORMAT_VERSION = 1`) using `ConversationItem`.

---

## Phase 4 — Project Infrastructure 🟡 ~60%

**Goal:** Project lives and maintains itself without xAI dependency.

### 4.1 CI/CD — ✅ DONE
- `.github/workflows/ci.yml` — test, clippy, build
- `.github/workflows/release.yml` — release pipeline
- `.github/workflows/rust.yml` — Rust-specific checks

### 4.2 Release Pipeline — ✅ DONE
- Version set to `0.1.0` (commit `678065b`).
- Release script updated (commit `d905a29`).
- Release docs prepared (commit `e229cdb`).

### 4.3 Technical Documentation — ❌ NOT DONE
- `ARCHITECTURE.md` — crate map and responsibilities — **missing**
- `PROVIDERS.md` — how to add a new provider — **missing**
- `HACKING.md` — 5-minute dev env setup — **missing**

### 4.4 Rust Toolchain Config — ✅ DONE
- `rust-toolchain.toml` pinned to `1.92.0` (edition 2024).
- Periodic bump recommended (every 4-6 weeks).

---

## Phase 5 — Community Extensibility 🟢 In Progress

**Goal:** Any contributor can add providers and tools without touching core.

### 5.1 Plugin API for Providers — 🟡 PARTIAL
- `ChatProvider` trait exists and works.
- Need to expose it as stable public API through `bucket-acp` for external crates.

### 5.2 MCP as Extension Vector — ✅ DONE
- MCP support well implemented via `bucket-hub-mcp-adapter`.
- Configuration in `~/.bucket/config.toml` under `[[mcp.server]]`.

### 5.3 Skills as First-Class Citizen — ✅ DONE
- `SKILL.md` system works.
- Public registry (GitHub repo `bucket-agent/skills`) is a future goal.

---

## Dependency Removal Map

```
BEFORE                          →  AFTER
──────────────────────────────────────────────────────────
auth.x.ai (OIDC endpoint)       →  BUCKET_OIDC_ISSUER configurable ✅
bucket.com (login)               →  Generic OIDC provider or none ✅
x.ai/cli (updates)              →  Configurable update URL ✅
OTLP → xAI infra               →  BUCKET_TELEMETRY_ENDPOINT or off ✅
bucket_* (crate names)          →  bucket_* ✅
BUCKET_* (env vars)             →  BUCKET_* (with deprecated aliases) ✅
~/.bucket (config dir)          →  ~/.bucket (with auto-migration) ✅
hardcoded model list            →  config.toml + /v1/models ✅
SuperBucket/billing UI          →  ProviderCapabilities.has_billing ✅
system prompt with xAI refs     →  Configurable, neutral default ✅
```

---

## Priority Order for 3-4 Person Team

| Sprint | Owner | Task |
|--------|-------|------|
| 1 | Agent + 1 human | Phase 1 (done — rename was automatable) |
| 2-3 | 2 people | Phase 2: remaining auth/billing cleanup |
| 4-6 | 3 people | Phase 3 (done — ChatProvider trait + runtime decoupling) |
| 7 | 1 person | Phase 4: docs (ARCHITECTURE.md, PROVIDERS.md, HACKING.md) |
| ongoing | all | Phase 5: extensibility, community, public plugin API |

---

## What NOT to Touch (v1)

- **ACP protocol** — well-designed, xAI-agnostic. Keep.
- **Session system** — solid, only needs nominal cleanup. Keep.
- **TUI engine (ratatui)** — no xAI coupling, only data. Keep.
- **Existing tests** — solid base. Extend, don't rewrite.
- **Hooks system** — generic and powerful. Only rename paths.
- **Subagent architecture** — advanced and valuable. Keep.

---

## Risk: Upstream Divergence

xAI may release major source updates (they did this several times in 2025). Each new upstream release can diverge from the fork in areas already modified.

**Mitigation:** Maintain an `UPSTREAM_DIFF.md` documenting exactly what changed and why, so cherry-picking upstream fixes is mechanical. Rotate "upstream sync" duty monthly.

---

## Commit Reference Log

| Commit | Description |
|--------|-------------|
| `d905a29` | fix(release): update release script readiness check and decoupled test assertions |
| `e229cdb` | docs(release): prepare v0.1.0 release docs, GitHub Actions workflows, and installer |
| `efa6cbf` | fix(auth): restore missing brace in acp_agent match block |
| `3c3848d` | fix(auth): handle LOCAL_METHOD_ID in ACP agent authenticate handler |
| `8c7c1af` | **feat(decoupling): complete Phase 3 agent runtime decoupling and ChatProvider integration** |
| `4bd3b22` | chore: update Cargo.lock for version 0.1.0 |
| `678065b` | chore: set version to 0.1.0 for initial fork release |
| `bf120a3` | feat: prepare release infrastructure and retarget auto-updater |
| `a8a45fa` | feat(updater): make update URLs configurable via CliConfig (Fase 2.3) |
| `96ae86b` | feat: make OIDC issuer configurable via BUCKET_OIDC_ISSUER (Fase 2.1) |
| `3db3f80` | **feat: ProviderCapabilities gating + telemetry decoupling (Fase 2)** |
| `36ec730` | tool: Add generate_workspace.py script and sort workspace members |
