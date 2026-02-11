//! chart-rs: modular charting engine scaffold.
//!
//! This crate provides a Rust-idiomatic API and a strict architectural split
//! intended for long-term parity work against Lightweight Charts v5.1.

pub mod api;
pub mod core;
pub mod error;
pub mod extensions;
pub mod interaction;
pub mod render;
pub mod telemetry;

#[cfg(feature = "gtk4-adapter")]
pub mod platform_gtk;

pub use api::{ChartEngine, ChartEngineConfig};
pub use error::{ChartError, ChartResult};
