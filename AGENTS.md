# AGENTS

## Project

Rust workspace (edition 2024, toolchain **1.92.0** pinned via `rust-toolchain.toml`).
Binary artifact: **`bucket`** — a full-screen TUI AI coding agent.
Multiprovider: Ollama, OpenAI, Anthropic, any OpenAI-compatible backend.

## Key crates

| Crate | Role |
|-------|------|
| `bucket-bin` | Composition root; builds the `bucket` binary |
| `bucket-tui` | TUI: scrollback, prompt, modals, rendering |
| `bucket-agent-core` | Agent runtime + leader/stdio/headless entry |
| `bucket-tools` | Tool implementations (terminal, file edit, search…) |
| `bucket-workspace` | Host filesystem, VCS, execution, checkpoints |

## Build commands

```sh
cargo run -p bucket-bin              # build + launch
cargo build -p bucket-bin --release  # release binary
cargo check -p bucket-bin            # fast validation
cargo test -p bucket-agent-core                 # test a specific crate
cargo clippy -p <crate>                      # lint a specific crate
cargo fmt --all                              # format all
```

**Always target specific crates** — full-workspace builds are slow.

## Prerequisites

- **DotSlash** must be on PATH (`cargo install dotslash`) — needed for `bin/protoc`.
- **protoc** — resolved via DotSlash from `bin/protoc`, or fallback to PATH/`$PROTOC`.
- macOS and Linux supported; Windows best-effort.

## Root `Cargo.toml` is generated

Do not edit it. Edit per-crate `Cargo.toml` files. The root file defines workspace members, dependency versions, profiles, and lints.

## Profiles

| Profile | Purpose |
|---------|---------|
| `release` | Default release; incremental, panic=abort |
| `release-dist` | Shipping to users; thin LTO, codegen-units=1, debug=1 |
| `x-prod` | Latency-sensitive services; thin LTO, panic=unwind |
| `release-dist-jemalloc` | Alias for desktop release workflow |

## Lint / format config

- `clippy.toml` bans `std::fs::canonicalize`, `std::path::Path::canonicalize`, and `tokio::fs::canonicalize` — use `dunce::canonicalize` or `bucket_tools::util::fs` helpers instead.
- Workspace clippy lints allow: `doc_lazy_continuation`, `needless_lifetimes`, `too_many_arguments`, `uninlined_format_args`, `useless_format`.
- `rustfmt.toml`: `use_field_init_shorthand = true`.

## Architecture

Three crate directories:
- `crates/codegen/` — main application crates (60+ crates)
- `crates/common/` — shared leaf crates (tool-protocol, tracing, test-utils, etc.)
- `crates/build/` — build-time crates (proto codegen)
- `prod/mc/` — production proxy types
- `third_party/` — vendored source (Mermaid diagram stack)

User guide lives at `crates/codegen/bucket-tui/docs/user-guide/`.
