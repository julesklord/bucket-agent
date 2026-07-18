//! ToolBridge: re-exported from `bucket-tools`.
//!
//! The bridge implementation now lives in `bucket_tools::bridge`.
//! This module re-exports everything for backward compatibility.

pub use bucket_tools::bridge::{ToolBridge, ToolBridgeResult};
