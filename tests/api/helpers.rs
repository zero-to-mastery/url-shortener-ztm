// tests/api/helpers.rs

// dependencies
use std::sync::{Arc, LazyLock};
use url_shortener_ztm_lib::database::{SqliteUrlDatabase, UrlDatabase};
use url_shortener_ztm_lib::get_configuration;
use url_shortener_ztm_lib::startup::build_router;
use url_shortener_ztm_lib::state::AppState;
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
    pub database: Arc<dyn UrlDatabase>,
    pub api_key: Uuid,
}

// Spin up an instance of our application and returns its address (i.e. http://localhost:XXXX)
pub async fn spawn_app() -> TestApp {
    // Ensure that the tracing is only initialized once
    LazyLock::force(&TRACING);

    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.application.port = 0;
        c.database.database_path = "sqlite::memory:".to_string();
        c
    };

    // Create database and run migrations
    let sqlite_db = SqliteUrlDatabase::from_config(&configuration.database)
        .await
        .expect("Failed to create database");
    sqlite_db.migrate().await.expect("Failed to run migrations");
    let database = Arc::new(sqlite_db);

    // Store the API key for use in tests
    let api_key = configuration.application.api_key;

    let test_app_state = AppState {
        database: database.clone(),
        api_key,
        template_dir: configuration.application.templates,
    };

    // Launch the application as a background task
    let test_app = build_router(test_app_state.clone())
        .await
        .expect("Failed to build application.");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let test_app_port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, test_app)
            .await
            .expect("Failed to serve application")
    });

    // Create an HTTP client for making requests to the application
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to build reqwest client.");

    TestApp {
        address: format!("http://127.0.0.1:{}", test_app_port),
        _port: test_app_port,
        client,
        database,
        api_key,
    }
}
