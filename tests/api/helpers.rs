// tests/api/helpers.rs

// dependencies
use sqlx::SqlitePool;
use std::sync::LazyLock;
use url_shortener_ztm_lib::get_configuration;
use url_shortener_ztm_lib::startup::{Application, get_connection_pool};
use url_shortener_ztm_lib::telemetry::{get_subscriber, init_subscriber};
use uuid::Uuid;

// set up a static variable for the tracing configuration
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

// struct type to represent a test application
pub struct TestApp {
    pub address: String,
    pub _port: u16,
    pub client: reqwest::Client,
    pub pool: SqlitePool,
    pub api_key: Uuid,
}

// Spin up an instance of our application and returns its address (i.e. http://localhost:XXXX)
pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.application.port = 0;
        c.database.database_path = "sqlite::memory:".to_string();
        c
    };

    let pool = get_connection_pool(&configuration.database);

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate the database");

    let api_key = configuration.application.api_key;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to build reqwest client.");

    TestApp {
        address: format!("http://127.0.0.1:{}", application_port),
        _port: application_port,
        client,
        pool,
        api_key,
    }
}
