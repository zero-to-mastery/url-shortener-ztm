// src/bin/shuttle.rs

// dependencies
use shuttle_axum::ShuttleAxum;
use shuttle_runtime::SecretStore;
use url_shortener_ztm_lib::configuration::ShuttleAppConfig;
use url_shortener_ztm_lib::startup::Application;
use url_shortener_ztm_lib::telemetry::{get_subscriber, init_subscriber};

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleAxum {
  // Initialize structured logging with tracing
    tracing::info!("Initializing tracing...");
    let subscriber = get_subscriber("url-shortener-ztm".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Load configuration
    tracing::info!("Loading service configuration...");
    let shuttle_app_config = ShuttleAppConfig::try_from(&secret_store)?;

    // Build the application with database connection and router setup
    tracing::info!("Starting up the application...");
    let application = Application::build(shuttle_app_config.clone()).await?;

    // Serve with Shuttle
    Ok(application.router.into())
}