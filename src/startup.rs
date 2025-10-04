//! # Application Startup and Configuration
//!
//! This module handles the complete application startup process, including database
//! initialization, router configuration, and server startup with graceful shutdown.
//!
//! ## Startup Process
//!
//! The application startup follows these steps:
//! 1. Database connection and migration
//! 2. Application state initialization
//! 3. Router configuration with middleware
//! 4. Server binding and startup
//! 5. Graceful shutdown handling
//!
//! ## Router Configuration
//!
//! The application router is organized into several route groups:
//! - **Public API** - Health check and redirect endpoints (no authentication)
//! - **Protected API** - URL shortening endpoint (requires API key)
//! - **Admin Panel** - Web interface for management
//! - **Static Files** - CSS, JavaScript, and other assets
//!
//! ## Middleware Stack
//!
//! The application uses several middleware layers:
//! - **Request ID** - Unique identifier for each request
//! - **Tracing** - Request/response logging and tracing
//! - **API Key Authentication** - For protected endpoints
//!
//! ## Graceful Shutdown
//!
//! The application supports graceful shutdown on:
//! - `SIGINT` (Ctrl+C)
//! - `SIGTERM` (termination signal)
//!
//! ## Usage
//!
//! ```rust,no_run
//! use url_shortener_ztm_lib::startup::Application;
//! use url_shortener_ztm_lib::configuration::get_configuration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = get_configuration()?;
//! let app = Application::build(config).await?;
//! app.run_until_stopped().await?;
//! # Ok(())
//! # }
//! ```

use crate::DatabaseType;
use crate::configuration::Settings;
use crate::database::postgres_sql::PostgresUrlDatabase;
use crate::database::{SqliteUrlDatabase, UrlDatabase};
use crate::generator::build_generator;
use crate::middleware::check_api_key;
use crate::routes::{
    get_admin_dashboard, get_index, get_login, get_redirect, get_register, get_user_profile,
    health_check, post_shorten,
};
use crate::state::AppState;
use crate::telemetry::MakeRequestUuid;
use anyhow::{Context, Result};
use axum::{
    Router,
    http::HeaderName,
    middleware::from_fn_with_state,
    routing::{get, post},
};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;
use tower::ServiceBuilder;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    services::ServeDir,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

/// Handles graceful shutdown signals for the application.
///
/// This function listens for system signals that should trigger a graceful shutdown:
/// - `SIGINT` (Ctrl+C) - User-initiated shutdown
/// - `SIGTERM` (Unix only) - System-initiated shutdown
///
/// The function uses `tokio::select!` to wait for either signal and then returns,
/// allowing the server to complete any in-flight requests before shutting down.
///
/// # Platform Support
///
/// - **Unix systems**: Supports both `SIGINT` and `SIGTERM`
/// - **Windows**: Supports only `SIGINT` (Ctrl+C)
///
/// # Examples
///
/// ```rust,no_run
/// use axum::Router;
/// use tokio::net::TcpListener;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let listener = TcpListener::bind("0.0.0.0:3000").await?;
/// let router = Router::new();
/// axum::serve(listener, router)
///     .with_graceful_shutdown(async {
///         tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
///     })
///     .await?;
/// # Ok(())
/// # }
/// ```
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

/// Main application struct containing all server components.
///
/// This struct holds all the necessary components to run the URL shortener service,
/// including the TCP listener, router, application state, and port information.
///
/// # Fields
///
/// * `port` - The port number the server is listening on
/// * `listener` - TCP listener for incoming connections
/// * `router` - Axum router with all configured routes and middleware
/// * `state` - Application state shared across all handlers
///
/// # Thread Safety
///
/// This struct is designed to be safely moved between threads and async tasks.
/// All contained types implement the necessary traits for thread safety.
///
/// # Examples
///
/// ```rust,no_run
/// use url_shortener_ztm_lib::startup::Application;
/// use url_shortener_ztm_lib::configuration::get_configuration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = get_configuration()?;
/// let app = Application::build(config).await?;
/// println!("Server running on port {}", app.port());
/// app.run_until_stopped().await?;
/// # Ok(())
/// # }
/// ```
#[allow(dead_code)]
pub struct Application {
    /// Port number the server is listening on
    port: u16,
    /// TCP listener for incoming connections
    listener: TcpListener,
    /// Axum router with all configured routes and middleware
    router: Router,
    /// Application state shared across all handlers
    state: AppState,
}

impl Application {
    /// Builds and initializes the application with the given configuration.
    ///
    /// This method performs the complete application initialization process:
    /// 1. Sets up the database connection and runs migrations
    /// 2. Creates the TCP listener on the configured address
    /// 3. Initializes the application state
    /// 4. Builds the router with all routes and middleware
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration settings
    ///
    /// # Returns
    ///
    /// Returns `Ok(Application)` if initialization succeeds, or `Err(anyhow::Error)`
    /// if any step fails.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - Database connection fails
    /// - Database migrations fail
    /// - TCP listener cannot be bound to the configured address
    /// - Router creation fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use url_shortener_ztm_lib::startup::Application;
    /// use url_shortener_ztm_lib::configuration::get_configuration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = get_configuration()?;
    /// let app = Application::build(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build(config: Settings) -> Result<Self, anyhow::Error> {
        // set up the database connection pool and run migrations
        let database: Arc<dyn UrlDatabase> = match config.database.r#type {
            DatabaseType::Sqlite => {
                let db = SqliteUrlDatabase::from_config(&config.database).await?;
                db.migrate().await?;
                Arc::new(db) as Arc<dyn UrlDatabase>
            }
            DatabaseType::Postgres => {
                let db = PostgresUrlDatabase::from_config(&config.database).await?;
                db.migrate().await?;
                Arc::new(db) as Arc<dyn UrlDatabase>
            }
        };
        let code_generator = build_generator(&config.shortener);

        // Set up the TCP listener and application state
        let api_key = config.application.api_key;
        let template_dir = config.application.templates.clone();
        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(address)
            .await
            .context("Unable to obtain a TCP listener...")?;
        let port = listener.local_addr()?.port();
        let state = AppState::new(database, code_generator, api_key, template_dir, config);

        // Build the application router, passing in the application state
        let router = build_router(state.clone())
            .await
            .context("Failed to create the application router.")?;

        Ok(Self {
            port,
            listener,
            router,
            state,
        })
    }

    /// Returns the port number the server is listening on.
    ///
    /// This method provides access to the port number that was assigned to the
    /// TCP listener. This is useful for logging and monitoring purposes.
    ///
    /// # Returns
    ///
    /// Returns the port number as a `u16`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use url_shortener_ztm_lib::startup::Application;
    /// use url_shortener_ztm_lib::configuration::get_configuration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = get_configuration()?;
    /// let app = Application::build(config).await?;
    /// println!("Server running on port {}", app.port());
    /// # Ok(())
    /// # }
    /// ```
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Runs the application server until stopped.
    ///
    /// This method starts the HTTP server and runs it until a shutdown signal
    /// is received. The server will handle graceful shutdown, allowing in-flight
    /// requests to complete before terminating.
    ///
    /// # Graceful Shutdown
    ///
    /// The server will shut down gracefully when it receives:
    /// - `SIGINT` (Ctrl+C)
    /// - `SIGTERM` (Unix only)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the server shuts down cleanly, or `Err(anyhow::Error)`
    /// if there's an error during server operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use url_shortener_ztm_lib::startup::Application;
    /// use url_shortener_ztm_lib::configuration::get_configuration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = get_configuration()?;
    /// let app = Application::build(config).await?;
    /// app.run_until_stopped().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run_until_stopped(self) -> Result<(), anyhow::Error> {
        axum::serve(
            self.listener,
            self.router
                .into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Unable to start the app server...")?;
        Ok(())
    }
}

/// Builds and configures the application router with all routes and middleware.
///
/// This function creates the complete Axum router with all configured routes,
/// middleware layers, and static file serving. The router is organized into
/// several logical groups for better maintainability.
///
/// # Route Groups
///
/// The router is organized into the following groups:
/// - **Public API** - Health check and redirect endpoints (no authentication required)
/// - **Protected API** - URL shortening endpoint (requires API key authentication)
/// - **Admin Panel** - Web interface for management
/// - **Static Files** - CSS, JavaScript, and other assets
///
/// # Middleware Stack
///
/// The following middleware layers are applied in order:
/// 1. **Request ID** - Generates unique identifiers for each request
/// 2. **Tracing** - Logs request/response information
/// 3. **API Key Authentication** - For protected endpoints only
///
/// # Arguments
///
/// * `state` - Application state to be shared with all handlers
///
/// # Returns
///
/// Returns `Ok(Router)` if the router is successfully created, or
/// `Err(anyhow::Error)` if there's an error during configuration.
///
/// # Examples
///
/// ```rust,no_run
/// use url_shortener_ztm_lib::startup::build_router;
/// use url_shortener_ztm_lib::state::AppState;
/// use url_shortener_ztm_lib::DatabaseType;
/// use url_shortener_ztm_lib::database::SqliteUrlDatabase;
/// use url_shortener_ztm_lib::configuration::DatabaseSettings;
/// use std::sync::Arc;
/// use uuid::Uuid;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = DatabaseSettings {
///     r#type: DatabaseType::Sqlite,
///     url: "database.db".to_string(),
///     create_if_missing: true,
/// };
/// let database = Arc::new(SqliteUrlDatabase::from_config(&config).await?);
/// let api_key = Uuid::new_v4();
/// let template_dir = "templates".to_string();
/// // let settings = get_configuration().expect("Failed to read configuration");
/// // let state = AppState::new(database, api_key, template_dir, settings);
/// // let router = build_router(state).await?;
/// # Ok(())
/// # }
/// ```
pub async fn build_router(state: AppState) -> Result<Router, anyhow::Error> {
    // Define the tracing layer for request/response logging
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(
            DefaultMakeSpan::new()
                .include_headers(true)
                .level(Level::INFO),
        )
        .on_response(DefaultOnResponse::new().include_headers(true));
    let x_request_id = HeaderName::from_static("x-request-id");

    // Create rate limiting configuration if enabled
    let rate_limit_layer = if state.config.rate_limiting.enabled {
        let governor_conf = GovernorConfigBuilder::default()
            .per_second(state.config.rate_limiting.requests_per_second)
            .burst_size(state.config.rate_limiting.burst_size)
            .use_headers()
            .finish()
            .context("Failed to create rate limiting configuration")?;

        // Start background cleanup task
        let governor_limiter = governor_conf.limiter().clone();
        let interval = Duration::from_secs(60);
        tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(interval);
            loop {
                cleanup_interval.tick().await;
                tracing::info!("rate limiting storage size: {}", governor_limiter.len());
                governor_limiter.retain_recent();
            }
        });

        Some(GovernorLayer::new(governor_conf))
    } else {
        None
    };

    // Build public routes (no authentication required)
    let public_routes = Router::new()
        .route("/", get(get_index))
        .route("/api/health_check", get(health_check))
        .route("/api/redirect/{id}", get(get_redirect));

    // Build public rate-limited shorten endpoint
    let mut public_shorten = Router::new().route("/api/public/shorten", post(post_shorten));

    if let Some(rate_layer) = rate_limit_layer.clone() {
        public_shorten = public_shorten.layer(rate_layer);
    }

    // Build protected API routes (requires API key)
    let mut protected_api = Router::new()
        .route("/api/shorten", post(post_shorten))
        .route_layer(from_fn_with_state(state.clone(), check_api_key));

    if let Some(rate_layer) = rate_limit_layer {
        protected_api = protected_api.layer(rate_layer);
    }

    // Build protected admin routes (requires API key)
    let protected_admin = Router::new()
        .route("/admin", get(get_admin_dashboard))
        .route("/admin/profile", get(get_user_profile))
        .route("/admin/login", get(get_login))
        .route("/admin/register", get(get_register))
        .route_layer(from_fn_with_state(state.clone(), check_api_key));

    // Merge all routes together
    let router = Router::new()
        .merge(public_routes)
        .merge(public_shorten)
        .merge(protected_api)
        .merge(protected_admin)
        .fallback_service(ServeDir::new("static"))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::new(
                    x_request_id.clone(),
                    MakeRequestUuid,
                ))
                .layer(trace_layer)
                .layer(PropagateRequestIdLayer::new(x_request_id)),
        );

    Ok(router)
}
