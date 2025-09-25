// src/main.rs

// the application's main entry point

// dependencies
use url_shortener_ztm_lib::configuration::get_configuration;
use url_shortener_ztm_lib::startup::Application;
use url_shortener_ztm_lib::telemetry::{get_subscriber, init_subscriber};

/// main function, program entry point
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing::info!("Initializing tracing...");
    let subscriber = get_subscriber("url-shortener-ztm".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // read in the app configuration and log it, panic if the configuration can't be read
    tracing::info!("Reading configuration...");
    let configuration = get_configuration().expect("Failed to read configuration files.");
    tracing::info!("Configuration: {:?}", configuration);

    // build an instance of the application and run it
    tracing::info!("Starting up the application...");
    let application = Application::build(configuration.clone()).await?;
    application.run_until_stopped().await?;

    Ok(())
}
