//! # Telemetry and Logging
//!
//! This module provides structured logging and request tracing capabilities for the
//! URL shortener service. It sets up comprehensive observability using the `tracing`
//! ecosystem with JSON formatting and request correlation.
//!
//! ## Features
//!
//! - **Structured Logging** - JSON-formatted logs with consistent fields
//! - **Request Tracing** - Full request lifecycle tracking with correlation IDs
//! - **Environment-based Filtering** - Configurable log levels via environment variables
//! - **Bunyan Formatting** - Human-readable and machine-parseable log output
//! - **Request ID Generation** - Unique identifiers for request correlation
//!
//! ## Log Format
//!
//! Logs are formatted as JSON with the following structure:
//!
//! ```json
//! {
//!   "v": 0,
//!   "name": "url-shortener-ztm",
//!   "msg": "Request completed",
//!   "level": 30,
//!   "time": "2025-01-18T12:00:00Z",
//!   "request_id": "550e8400-e29b-41d4-a716-446655440000",
//!   "method": "POST",
//!   "path": "/api/shorten",
//!   "status": 200
//! }
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use url_shortener_ztm_lib::telemetry::{get_subscriber, init_subscriber};
//!
//! // Initialize logging
//! let subscriber = get_subscriber("my-app".into(), "info".into(), std::io::stdout);
//! init_subscriber(subscriber);
//!
//! // Use tracing macros
//! tracing::info!("Application started");
//! tracing::error!("Something went wrong");
//! ```

use axum::http::Request;
use tower_http::request_id::{MakeRequestId, RequestId};
use tracing::Subscriber;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};
use uuid::Uuid;

/// Request ID generator that creates UUID-based request identifiers.
///
/// This struct implements the `MakeRequestId` trait to generate unique request IDs
/// for each incoming HTTP request. The IDs are used for request correlation and
/// tracing across the application.
///
/// # ID Format
///
/// Request IDs are generated as UUID v4 strings, providing globally unique
/// identifiers that are suitable for distributed systems.
///
/// # Examples
///
/// ```rust
/// use url_shortener_ztm_lib::telemetry::MakeRequestUuid;
/// use tower_http::request_id::MakeRequestId;
/// use axum::http::Request;
///
/// let mut generator = MakeRequestUuid;
/// let request = Request::new(());
/// let request_id = generator.make_request_id(&request);
/// assert!(request_id.is_some());
/// ```
#[derive(Clone)]
pub struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    /// Generates a new UUID-based request ID for the given request.
    ///
    /// This method creates a unique identifier for each request, which is used
    /// for tracing and correlation across the application's logging and monitoring.
    ///
    /// # Arguments
    ///
    /// * `_` - The HTTP request (unused, but required by the trait)
    ///
    /// # Returns
    ///
    /// Returns `Some(RequestId)` containing a UUID v4 string, or `None` if
    /// ID generation fails (which should not happen in normal operation).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use url_shortener_ztm_lib::telemetry::MakeRequestUuid;
    /// use tower_http::request_id::MakeRequestId;
    /// use axum::http::Request;
    ///
    /// let mut generator = MakeRequestUuid;
    /// let request = Request::new(());
    /// let request_id = generator.make_request_id(&request);
    /// assert!(request_id.is_some());
    /// ```
    fn make_request_id<B>(&mut self, _: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string();

        Some(RequestId::new(request_id.parse().unwrap()))
    }
}

/// Creates a configured tracing subscriber for structured logging.
///
/// This function sets up a comprehensive logging subscriber with JSON formatting,
/// environment-based filtering, and request correlation capabilities.
///
/// # Arguments
///
/// * `name` - Application name used in log output
/// * `env_filter` - Default log level filter (overridden by `RUST_LOG` environment variable)
/// * `sink` - Output destination for log messages (e.g., `std::io::stdout`, `std::io::stderr`)
///
/// # Returns
///
/// Returns a configured `Subscriber` that can be used with `init_subscriber`.
///
/// # Environment Variables
///
/// The log level can be controlled using the `RUST_LOG` environment variable:
///
/// ```bash
/// # Set log level for the entire application
/// RUST_LOG=info cargo run
///
/// # Set different levels for different modules
/// RUST_LOG=url_shortener_ztm_lib=debug,axum=info cargo run
///
/// # Enable all logs
/// RUST_LOG=trace cargo run
/// ```
///
/// # Examples
///
/// ```rust,no_run
/// use url_shortener_ztm_lib::telemetry::{get_subscriber, init_subscriber};
///
/// // Basic setup
/// let subscriber = get_subscriber("my-app".into(), "info".into(), std::io::stdout);
/// init_subscriber(subscriber);
///
/// // With custom log level
/// let subscriber = get_subscriber("my-app".into(), "debug".into(), std::io::stderr);
/// init_subscriber(subscriber);
/// ```
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Sync + Send
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Initializes the global tracing subscriber.
///
/// This function sets up the global logging infrastructure by:
/// 1. Redirecting standard log macros to the tracing subscriber
/// 2. Setting the provided subscriber as the global default
///
/// # Arguments
///
/// * `subscriber` - The tracing subscriber to use for logging
///
/// # Panics
///
/// This function will panic if:
/// - The logger has already been initialized
/// - The subscriber cannot be set as the global default
///
/// # Examples
///
/// ```rust,no_run
/// use url_shortener_ztm_lib::telemetry::{get_subscriber, init_subscriber};
///
/// // Initialize logging
/// let subscriber = get_subscriber("my-app".into(), "info".into(), std::io::stdout);
/// init_subscriber(subscriber);
///
/// // Now you can use tracing macros
/// tracing::info!("Application started");
/// tracing::error!("Something went wrong");
/// ```
pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    // Redirect logs to subscriber
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
