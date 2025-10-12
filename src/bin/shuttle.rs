// src/bin/shuttle.rs

// dependencies
use shuttle_axum::ShuttleAxum;
use url_shortener_ztm_lib::configuration::get_configuration;
use url_shortener_ztm_lib::startup::Application;
use url_shortener_ztm_lib::telemetry::{get_subscriber, init_subscriber};

#[shuttle_runtime::main]
async fn main() -> ShuttleAxum {
  // Initialize structured logging with tracing
    tracing::info!("Initializing tracing...");
    let subscriber = get_subscriber("url-shortener-ztm".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Load application configuration from YAML files and environment variables
    tracing::info!("Reading configuration...");
    let configuration = get_configuration().expect("Failed to read configuration files.");
    tracing::info!(%configuration, "Configuration loaded");

    // Build the application with database connection and router setup
    tracing::info!("Starting up the application...");
    let application = Application::build(configuration.clone()).await?;

    // Serve with Shuttle
    Ok(application.router.into())
}