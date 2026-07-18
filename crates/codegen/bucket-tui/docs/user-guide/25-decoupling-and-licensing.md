# Decoupling and Licensing

This document provides a detailed overview of the decoupling plan for **Bucket Agent** from its upstream parent code and explains the licensing and intellectual property (IP) framework governing this project.

---

## 1. The Decoupling Plan (`assets/DECOUPLING.md`)

Bucket Agent started as a fork of the xAI Bucket Build (`d5e79b1`). The original code was tightly coupled to proprietary xAI infrastructure, user accounts, and telemetry sinks. The decoupling plan outlines a 5-phase roadmap to make Bucket fully autonomous, privacy-respecting, and community-driven.

Below is a summary of the 5 phases of the decoupling plan:

### Phase 1: Nominal Cleanup
*   **Rename Crates:** Move physical directories and update names in `Cargo.toml` files to generic, non-proprietary terms (e.g. renaming directories to use the `bucket-` prefix consistently).
*   **Environment Variables:** Map environment variables to standard `BUCKET_*` variables, while keeping deprecated aliases for backward compatibility.
*   **Configuration Directories:** Transition configurations to generic paths like `~/.bucket/` with auto-migration from any existing configuration directories.

### Phase 2: Auth and Billing Decoupling
*   **Remove Subscription Gates:** Eliminate credit bars, subscription screens, and banners.
*   **Provider Capabilities System:** Replace hardcoded billing assumptions with a generic `ProviderCapabilities` struct. Features like the credit bar are hidden automatically when using custom/local providers (e.g. Ollama).
*   **Disable Default Telemetry:** Turn off automatic metrics streaming to proprietary servers. Telemetry is opt-in and requires configuring `BUCKET_TELEMETRY_ENDPOINT` with your own OTLP collector.
*   **Independent Updates:** Update verification points to GitHub Releases rather than proprietary upstream endpoints.

### Phase 3: Runtime Decoupling
*   **ChatProvider Trait:** Introduce a standard interface for completion backends (`ChatProvider`). This allows the runtime to interact seamlessly with Ollama, OpenAI, Anthropic, or mock providers.
*   **Configurable System Prompts:** Clean up company-specific references in system prompts and make them fully configurable in `config.toml`.
*   **Agnostic Session Metadata:** Persist sessions with generic metadata (only `model_id` and `base_url`), removing reliance on hardcoded model lists.

### Phase 4: Independent Infrastructure
*   **CI/CD Workflows:** Maintain independent build and test pipelines on GitHub Actions.
*   **Release Pipelines:** Create standalone release channels and binaries for Linux, macOS, and Windows.
*   **Independent Installer:** Provide a direct curl-based installer (`install.sh`) pointing to GitHub releases.

### Phase 5: Community Extensibility
*   **Plugin API:** Stabilize provider extensions in `bucket-acp` for custom endpoints.
*   **MCP Focus:** Simplify Model Context Protocol configurations.
*   **Skills Registry:** Establish a community repository for sharing `SKILL.md` instruction sets.

---

## 2. Licensing and Credits

Bucket Agent is an open-source project that fully respects the licensing requirements of its upstream codebase.

### Apache License 2.0
*   **First-Party & Upstream Code:** The first-party code, as well as the modifications applied during the decoupling process, are licensed under the **Apache License, Version 2.0**.
*   **Section 4 (Redistribution) Compliance:** 
    *   Any redistributed work includes a copy of the Apache 2.0 License.
    *   Prominent change notices are maintained (such as in `assets/DECOUPLING.md`) indicating modifications to the original files.
    *   All original copyright, patent, trademark, and attribution notices from the source form have been strictly retained (e.g. `Copyright 2023-2026 SpaceXAI`).

### Third-Party and Vendored Licenses
*   **Crates & Tool Ports:** Third-party and vendored code remains under their original licenses. Details of these third-party licenses are documented in:
    *   `THIRD-PARTY-NOTICES` (at the repository root)
    *   `crates/codegen/bucket-tools/THIRD_PARTY_NOTICES.md` (for tool-specific ports)
    *   `third_party/NOTICE` (for the vendored Mermaid-stack)

---

## 3. Intellectual Property and Trademarks

### Trademark Policy
In accordance with **Section 6** of the Apache 2.0 License:
*   This license **does not grant permission** to use the trade names, trademarks, service marks, or product names of the Licensor, except as required for reasonable and customary use in describing the origin of the Work and reproducing the content of the `NOTICE` file.
*   The rename mappings and nominal cleanup phase (Phase 1) ensure that the project operates independently without using proprietary branding, while still maintaining clean, legal attribution of origin.
