// tests/api/helpers.rs

// dependencies
use axum::http::StatusCode;
use reqwest::header::CONTENT_TYPE;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};
use url_shortener_ztm_lib::core::security::jwt::JwtKeys;
use url_shortener_ztm_lib::database::{SqliteUrlDatabase, UrlDatabase};
use url_shortener_ztm_lib::generator::{self, build_generator};
use url_shortener_ztm_lib::get_configuration;
use url_shortener_ztm_lib::routes::shorten::normalize_url;
use url_shortener_ztm_lib::shortcode::bloom_filter::build_bloom_state;
use url_shortener_ztm_lib::startup::build_router;
use url_shortener_ztm_lib::startup::build_services;
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
    pub _database: Arc<dyn UrlDatabase>,
    pub api_key: Uuid,
    pub base_url: String,
}

// Spin up an instance of our application and returns its address (i.e. http://localhost:XXXX)
pub async fn spawn_app() -> TestApp {
    // Ensure that the tracing is only initialized once
    LazyLock::force(&TRACING);
    unsafe { std::env::set_var("BLOOM_SNAPSHOTS", "1") };

    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.application.port = 0;
        c.database.url = "sqlite::memory:".to_string();
        // Use more lenient rate limiting for tests (higher rate, smaller burst)
        c.rate_limiting.requests_per_second = 100; // 100 req/sec for fast tests
        c.rate_limiting.burst_size = 2; // Smaller burst for predictable testing
        c
    };

    // Create database and run migrations
    let sqlite_db = SqliteUrlDatabase::from_config(&configuration.database)
        .await
        .expect("Failed to create database");

    sqlite_db.migrate().await.expect("Failed to run migrations");
    let database: Arc<dyn UrlDatabase> = Arc::new(sqlite_db);
    let code_generator = build_generator(&configuration.shortener);

    let allowed_chars: HashSet<char> = {
        let mut set: HashSet<char> = HashSet::new();
        if let Some(alpha) = &configuration.shortener.alphabet {
            set.extend(alpha.chars());
        } else {
            set.extend(generator::DEFAULT_ALPHABET);
        }

        set
    };

    // Store the API key for use in tests
    let api_key = configuration.application.api_key;
    let blooms = build_bloom_state(&database).await.unwrap();
    let jwt = JwtKeys::new(configuration.application.api_key.as_bytes());

    let (auth_svc, user_svc) = build_services(&configuration, &jwt).await.unwrap();

    let test_app_state = AppState {
        // db_pool: Arc::new(db_pool),
        code_generator,
        blooms,
        allowed_chars,
        api_key: configuration.application.api_key,
        template_dir: configuration.application.templates.clone(),
        config: configuration.clone(),
        auth_service: auth_svc,
        user_service: user_svc,
        jwt,
        database: database.clone(),
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
        axum::serve(
            listener,
            test_app
                .with_state(test_app_state.clone())
                .into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("Failed to serve application")
    });

    // Create an HTTP client for making requests to the application
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to build reqwest client.");

    let base_url = configuration.application.base_url.clone();

    TestApp {
        address: format!("http://127.0.0.1:{}", test_app_port),
        _port: test_app_port,
        client,
        _database: database,
        api_key,
        base_url,
    }
}

// convenience helpers to reduce boilerplate in tests
impl TestApp {
    // Build a full URL from a path
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.address, path)
    }

    // Build a full API URL from an API path
    pub fn api(&self, path: &str) -> String {
        if path.starts_with("/api/") {
            self.url(path)
        } else {
            self.url(&format!("/api/{}", path))
        }
    }

    // Simple GET request
    #[allow(dead_code)]
    pub async fn get(&self, path: &str) -> reqwest::Response {
        self.client
            .get(self.url(path))
            .send()
            .await
            .expect("Failed to execute GET request")
    }

    // Simple API GET request
    pub async fn get_api(&self, path: &str) -> reqwest::Response {
        self.client
            .get(self.api(path))
            .send()
            .await
            .expect("Failed to execute GET request")
    }

    // POST raw body to API path
    #[allow(dead_code)]
    pub async fn post_api_body(&self, path: &str, body: impl Into<String>) -> reqwest::Response {
        let body_str = body.into();
        // Validate the URL using normalize_url function
        match normalize_url(&body_str) {
            Ok(_) => self
                .client
                .post(self.api(path))
                .body(body_str)
                .send()
                .await
                .expect("Failed to execute POST request"),
            Err(_) => {
                // If URL is invalid, return a 422 response
                self.client
                    .post(self.api(path))
                    .body(body_str)
                    .send()
                    .await
                    .expect("Failed to execute POST request")
            }
        }
    }

    // Authenticated POST with API key header
    pub async fn post_api_with_key(
        &self,
        path: &str,
        body: impl Into<String>,
    ) -> reqwest::Response {
        let body_str = body.into();
        // Validate the URL using normalize_url function
        match normalize_url(&body_str) {
            Ok(_) => self
                .client
                .post(self.api(path))
                .header("x-api-key", self.api_key.to_string())
                .body(body_str)
                .send()
                .await
                .expect("Failed to execute POST request"),
            Err(_) => {
                // If URL is invalid, return a 422 response
                self.client
                    .post(self.api(path))
                    .header("x-api-key", self.api_key.to_string())
                    .body(body_str)
                    .send()
                    .await
                    .expect("Failed to execute POST request")
            }
        }
    }

    // Admin route helpers
    #[allow(dead_code)]
    pub async fn get_admin_dashboard(&self) -> reqwest::Response {
        self.client
            .get(self.url("/admin"))
            .send()
            .await
            .expect("Failed to execute GET request")
    }

    #[allow(dead_code)]
    pub async fn get_admin_dashboard_with_api_key(&self) -> reqwest::Response {
        self.client
            .get(self.url("/admin"))
            .header("x-api-key", self.api_key.to_string())
            .send()
            .await
            .expect("Failed to execute GET request")
    }

    #[allow(dead_code)]
    pub async fn get_admin_login(&self) -> reqwest::Response {
        self.client
            .get(self.url("/admin/login"))
            .send()
            .await
            .expect("Failed to execute GET request")
    }

    #[allow(dead_code)]
    pub async fn get_admin_register(&self) -> reqwest::Response {
        self.client
            .get(self.url("/admin/register"))
            .send()
            .await
            .expect("Failed to execute GET request")
    }

    #[allow(dead_code)]
    pub async fn get_admin_profile(&self) -> reqwest::Response {
        self.client
            .get(self.url("/admin/profile"))
            .send()
            .await
            .expect("Failed to execute GET request")
    }
}

// Assertion helpers
pub async fn assert_json_ok(response: reqwest::Response) -> Value {
    assert!(response.status().is_success());
    assert_eq!(response.status(), StatusCode::OK);

    let ct = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(ct.starts_with("application/json"));

    let body: Value = response.json().await.expect("Response was not valid JSON");

    assert_eq!(body.get("success").and_then(Value::as_bool), Some(true));
    assert_eq!(body.get("message").and_then(Value::as_str), Some("ok"));
    assert_eq!(body.get("status").and_then(Value::as_u64), Some(200));
    assert!(body.get("time").and_then(Value::as_str).is_some());
    assert!(body.get("data").is_some());

    body
}

pub async fn assert_redirect_to(
    response: reqwest::Response,
    expected_location: &str,
    status: StatusCode,
) {
    assert_eq!(response.status(), status);
    let location_header = response
        .headers()
        .get("location")
        .expect("No location header found in response");
    assert_eq!(location_header, expected_location);
}
