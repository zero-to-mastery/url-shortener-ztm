// src/lib/startup.rs

// contains all the startup and configuration logic for the application

// dependencies
use crate::configuration::{DatabaseSettings, Settings};
use crate::middleware::check_api_key;
use crate::routes::{get_redirect, health_check, post_shorten};
use crate::state::AppState;
use crate::telemetry::MakeRequestUuid;
use anyhow::{Context, Result};
use axum::{Router, http::HeaderName, middleware::from_fn_with_state, routing::{get, post}};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

// utility function to gracefully shut down the app
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

// struct type to represent the application, wraps an Axum Router type
#[allow(dead_code)]
pub struct Application {
    port: u16,
    listener: TcpListener,
    router: Router,
    state: AppState,
}

// methods for the Application type
impl Application {
    // builds the router for the application
    pub async fn build(config: Settings) -> Result<Self, anyhow::Error> {
        
        // set up the database connection pool and run migrations
        let db_pool = get_connection_pool(&config.database);
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .context("Failed to migrate the database.")?;

        // set up the TCP listener and application state
        let api_key = config.application.api_key;
        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(address)
            .await
            .context("Unable to obtain a TCP listener...")?;
        let port = listener.local_addr()?.port();
        let state = AppState::new(db_pool, api_key);

        // build the application router, passing in the application state
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

    // utility function to return the application port, if needed
    pub fn port(&self) -> u16 {
        self.port
    }

    // function to run the app until stopped
    pub async fn run_until_stopped(self) -> Result<(), anyhow::Error> {
        axum::serve(self.listener, self.router)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .context("Unable to start the app server...")?;
        Ok(())
    }
}

// function to get a database connection pool
pub fn get_connection_pool(config: &DatabaseSettings) -> SqlitePool {
    SqlitePool::connect_lazy(config.connection_string().as_str())
        .expect("Failed to create the database connection pool.")
}

// function which configures and creates the application router
pub async fn build_router(state: AppState) -> Result<Router, anyhow::Error> {
    // define the tracing layer
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(
            DefaultMakeSpan::new()
                .include_headers(true)
                .level(Level::INFO),
        )
        .on_response(DefaultOnResponse::new().include_headers(true));
    let x_request_id = HeaderName::from_static("x-request-id");

    
    let secure_api = Router::new()
            .route("/", post(post_shorten))
            .route_layer(from_fn_with_state(state.clone(), check_api_key));

    // build the router with tracing
    let router = Router::new()
        .route("/health_check", get(health_check))
        .route("/{id}", get(get_redirect))
        .merge(secure_api)
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
