// src/main.rs

// dependencies
use url_shortener_ztm_lib::configuration::get_configuration;
use url_shortener_ztm_lib::startup::Application;
use url_shortener_ztm_lib::telemetry::{get_subscriber, init_subscriber};

// main function, program entry point
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    let subscriber = get_subscriber(
        "axum-api-template-server".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    // read in the app configuration
    let configuration = get_configuration().expect("Failed to read configuration files.");

    // build an instance of the application and run it
    let application = Application::build(configuration.clone()).await?;
    application.run_until_stopped().await?;

    Ok(())
}
