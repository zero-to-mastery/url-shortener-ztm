// src/lib/startup.rs

// contains all the startup and configuration logic for the application

// dependencies
use crate::configuration::Settings;
use crate::routes::health_check;
use crate::telemetry::MakeRequestUuid;
use anyhow::{Context, Result};
use axum::{Router, http::HeaderName, routing::get};
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
pub struct Application {
    port: u16,
    listener: TcpListener,
    router: Router,
}

// methods for the Application type
impl Application {
    // builds the router for the application
    pub async fn build(config: Settings) -> Result<Self, anyhow::Error> {
        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(address)
            .await
            .context("Unable to obtain a TCP listener...")?;
        let port = listener.local_addr()?.port();

        let router = build_router()
            .await
            .context("Failed to create the application router.")?;

        Ok(Self {
            port,
            listener,
            router,
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

// function which configures and creates the application router
pub async fn build_router() -> Result<Router, anyhow::Error> {
    // define the tracing layer
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(
            DefaultMakeSpan::new()
                .include_headers(true)
                .level(Level::INFO),
        )
        .on_response(DefaultOnResponse::new().include_headers(true));
    let x_request_id = HeaderName::from_static("x-request-id");

    // build the router with tracing
    let router = Router::new()
        .route("/health_check", get(health_check))
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
