//! Application level tracing configuration.

use ::tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Sets up application level tracing using an env logger configuration.
///
/// ### Example Usage
///
/// Run the program with `RUST_LOG=trace cargo run` to capture `TRACE`-level
/// events and upwards.
pub(crate) fn configure() {
    ::tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
}
