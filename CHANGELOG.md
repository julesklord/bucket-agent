# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [v0.1.7] - 2026-07-23

### Changed
- **Response Latency Optimizations**: Reduced model discovery HTTP timeout (`FETCH_TIMEOUT`) and background `models.dev` sync timeout to 3s for faster initial prompt responses without stalls.

---

## [v0.1.6] - 2026-07-23

### Added
- **Dynamic Context Window Discovery**: Integrated model context window metadata resolution from `models.dev` API registry cache.
- **Offline & Atomic Cache Management**: Background sync with atomic `.tmp` swap writing to `~/.bucket/models.json` ensuring fast startup, full offline support, and zero race conditions.
- **Welcome Screen ModelPicker**: Enabled `ModelPicker` modal access directly on the TUI welcome screen before active agent sessions.
- **Optimized Build Times**: Reduced CI/CD release workflow execution times by caching `dotslash` prebuilt binaries and enabling `codegen-units=16`.

### Fixed
- **Memory Leak Fix**: Fixed memory leak in modal state management (`Box::leak` in `modals.rs`).
- **Model ID Matching**: Added robust model ID normalization to match models across different provider naming conventions.

---

## [v0.1.5] - 2026-07-22

### Added
- **Smart Model Picker Modal**: Introduced an interactive `ModelPicker` modal in the TUI allowing users to search and select models with developer/model formatting.
- **Visual Assets**: Updated documentation screenshots with the new interface design in `README.md`.

---

## [v0.1.4] - 2026-07-21

### Added
- **Dynamic BYOK Model Discovery**: Added dynamic model discovery for BYOK providers (Ollama, OpenAI-compatible, etc.) directly on agent startup.
- **TUI Model Notifications**: Automatically notify TUI when new models are discovered on startup.

### Fixed
- **BYOK Credentials Resolution**: Fixed bug where `env_key` was missing on discovered model entries, ensuring API keys resolve correctly.

---

## [v0.1.3] - 2026-07-21

### Added
- **Model Diagnostics**: Added `--diagnose` flag to `bucket models` CLI command for troubleshooting provider connectivity and model availability.
- **Configurable Speech-to-Text**: Added `BUCKET_VOICE_API_BASE` environment variable support to make STT endpoints configurable.
- **Provider Testing**: Added end-to-end integration tests for Anthropic and OpenAI-compatible provider configurations.

### Changed
- **Vendor Neutrality**: Cleaned up remaining upstream-specific branding strings and updated decoupling documentation.
- **Helpful Error Messages**: Added API backend hints and guidance to 404/400 sampling error responses.

---

## [v0.1.2] - 2026-07-19

### Changed
- **First-Party Auth Refactoring**: Renamed internal authentication references from xAI-specific flags to generic `is_first_party_auth`.
- **Session API Compatibility**: Supported dual session update endpoints (`_bucket/session/update` and `_x.ai/session/update`).

### Fixed
- **Provider Model Auto-Detection**: Fixed auto-detection of BYOK provider models during cross-compilation builds.
- **Import & Lifetime Fixes**: Resolved visibility and reference lifetime compilation errors in `bucket-tui`.
- **Interactive Login**: Fixed credential caching issue by skipping cached WebLogin credentials to force fresh interactive logins.

---

## [v0.1.1] - 2026-07-19

### Added
- **BYOK Provider Configuration**: Added provider selector menu, API key format guides, persistent `providers.toml`, environment variable overrides, and live hot-reloading.
- **Feedback Command**: Added GitHub issue template and automatic browser opening when executing `/feedback`.

### Changed
- **Decoupling**: Removed legacy mixpanel tracking and decoupled proprietary xAI URL validations.

### Fixed
- **Release Workflows**: Fixed sha256 checksum generation, aarch64 cross-linker configuration, and workspace code formatting.
- **Trusted URL Matching**: Restored default `cli_chat_proxy_base_url` for URL matching tests.

---

## [v0.1.0] - 2026-07-19

### Added
- **Initial Fork & Workspace Setup**: Separated and configured standalone TUI coding agent workspace.
- **Atomic FS Writes**: Implemented safe atomic writes across `LocalFs`.
- **Zero Outbound Connections**: Ensured agent defaults to zero outbound connections for security and privacy.
