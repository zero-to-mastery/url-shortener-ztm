//! # URL Shortener Application
//!
//! The main entry point for the URL shortener service. This binary application
//! initializes the web server, loads configuration, and starts the HTTP service.
//!
//! ## Usage
//!
//! ```bash
//! # Run the application
//! cargo run
//!
//! # Run with custom environment
//! APP_ENVIRONMENT=production cargo run
//!
//! # Run with custom configuration
//! APP_APPLICATION__PORT=3000 cargo run
//! ```
//!
//! ## Configuration
//!
//! The application reads configuration from YAML files in the `configuration/` directory
//! and environment variables. See the library documentation for more details.

use url_shortener_ztm_lib::configuration::get_configuration;
use url_shortener_ztm_lib::startup::Application;
use url_shortener_ztm_lib::telemetry::{get_subscriber, init_subscriber};
use uuid::Uuid;

/// Main function - the application entry point.
///
/// This function:
/// 1. Initializes structured logging with tracing
/// 2. Loads application configuration from files and environment variables
/// 3. Builds and starts the HTTP server
/// 4. Runs until stopped (graceful shutdown on SIGINT/SIGTERM)
///
/// # Errors
///
/// Returns an error if:
/// - Configuration cannot be loaded
/// - Database connection fails
/// - Server fails to start
/// - Any other critical error occurs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging with tracing
    tracing::info!("Initializing tracing...");
    let subscriber = get_subscriber("url-shortener-ztm".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Load application configuration from YAML files and environment variables
    tracing::info!("Reading configuration...");
    let configuration = get_configuration().expect("Failed to read configuration files.");
    tracing::info!(%configuration, "Configuration loaded");

    // Detect default development API key and emit a prominent warning
    // This warns developers if they are accidentally using the insecure default key.
    let default_dev_key = Uuid::parse_str("e4125dd1-3d3e-43a1-bc9c-dc0ba12ad4b5").unwrap();
    if configuration.application.api_key == default_dev_key {
        // Use a big banner to make this stand out in logs
        tracing::warn!(
            "\n============================================================\n\
            SECURITY WARNING: USING DEFAULT DEVELOPMENT API KEY\n\
            ----------------------------------------------------\n\
            The configured API key matches the development default.\n\
            This key is PUBLIC and MUST NOT be used in production.\n\
\n\
            To set a secure key (UUID v4):\n\
              - env var: APP_APPLICATION__API_KEY=<uuid-v4>\n\
              - generate: `uuidgen` (Linux/macOS) or `[guid]::NewGuid()` (PowerShell)\n\
\n\
            Current environment: {env}\n\
============================================================",
            env = std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "local".into())
        );
    }

    // Build the application with database connection and router setup
    tracing::info!("Starting up the application...");
    let application = Application::build(configuration.clone()).await?;

    // Run the server until stopped (handles graceful shutdown)
    application.run_until_stopped().await?;

    Ok(())
}
