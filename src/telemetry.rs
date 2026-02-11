//! Telemetry helpers for applications embedding `chart-rs`.
//!
//! This module keeps tracing setup explicit and opt-in.
//! Consumers can either call `init_default_tracing` or wire their own
//! `tracing` subscriber and filters.

/// Initializes a default `tracing` subscriber when the `telemetry` feature is enabled.
///
/// Returns `true` when initialization succeeds.
/// Returns `false` when no initialization is performed (feature disabled) or if a
/// global subscriber was already set by the host application.
#[must_use]
pub fn init_default_tracing() -> bool {
    #[cfg(feature = "telemetry")]
    {
        let builder = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .with_target(false)
            .compact();

        return builder.try_init().is_ok();
    }

    #[cfg(not(feature = "telemetry"))]
    {
        false
    }
}
