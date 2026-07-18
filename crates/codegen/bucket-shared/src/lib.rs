//! Shared utilities used by both `bucket-agent-core` and its downstream clients
//! (e.g. `bucket-tui-render`). This crate sits upstream of `bucket-agent-core`
//! so it must never depend on it.

pub mod clipboard;
pub mod placeholder_images;
pub mod session;
pub mod stderr;
pub mod ui_config;
