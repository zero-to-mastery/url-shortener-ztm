//! # URL Shortener Library
//!
//! A high-performance URL shortener service built with modern Rust technologies.
//! This library provides a complete web service for creating shortened URLs and
//! redirecting users to their original destinations.
//!
//! ## Features
//!
//! - **Fast URL shortening**: Generate short, unique identifiers for long URLs using nanoid
//! - **Reliable redirects**: Permanent redirects to original URLs with proper HTTP status codes
//! - **SQLite storage**: Lightweight, file-based database with automatic migrations
//! - **Database abstraction**: Trait-based database layer for easy database switching
//! - **Comprehensive logging**: Structured logging with tracing and request IDs
//! - **Health monitoring**: Built-in health check endpoint
//! - **Web interface**: Admin panel with Tera templates
//! - **API key protection**: Secure API endpoints with UUID-based authentication
//! - **Production ready**: Built for deployment with graceful shutdown handling
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use url_shortener_ztm_lib::configuration::get_configuration;
//! use url_shortener_ztm_lib::startup::Application;
//! use url_shortener_ztm_lib::telemetry::{get_subscriber, init_subscriber};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize tracing
//!     let subscriber = get_subscriber("url-shortener-ztm".into(), "info".into(), std::io::stdout);
//!     init_subscriber(subscriber);
//!
//!     // Read configuration
//!     let configuration = get_configuration().expect("Failed to read configuration files.");
//!
//!     // Build and run the application
//!     let application = Application::build(configuration).await?;
//!     application.run_until_stopped().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The library is organized into several modules:
//!
//! - [`configuration`] - Configuration management with YAML files and environment variables
//! - [`database`] - Database abstraction layer with SQLite implementation
//! - [`errors`] - Comprehensive error handling with custom API error types
//! - [`middleware`] - API key authentication middleware
//! - [`response`] - Standardized JSON response envelope
//! - [`routes`] - HTTP route handlers for all endpoints
//! - [`startup`] - Application startup and router configuration
//! - [`state`] - Application state management
//! - [`telemetry`] - Logging and tracing setup
//! - [`templates`] - Tera template rendering for web interface
//!
//! ## API Endpoints
//!
//! - `POST /api/shorten` - Shorten a URL (requires API key)
//! - `GET /api/redirect/{id}` - Redirect to original URL
//! - `GET /api/health_check` - Health check endpoint
//! - `GET /admin` - Admin web interface
//!
//! ## Configuration
//!
//! The service supports environment-based configuration with YAML files:
//! - `configuration/base.yml` - Base configuration
//! - `configuration/local.yml` - Local development overrides
//! - `configuration/production.yml` - Production settings
//!
//! Environment variables can override any setting using the `APP_` prefix.

// Module declarations
pub mod app;
pub mod configuration;
pub mod core;
pub mod database;
pub mod errors;
pub mod features;
pub mod generator;
pub mod infrastructure;
pub mod middleware;
pub mod models;
pub mod response;
pub mod routes;
pub mod shortcode;
pub mod startup;
pub mod state;
pub mod telemetry;
pub mod templates;

// Re-exports for convenience
pub use configuration::*;
pub use errors::*;
pub use middleware::*;
pub use response::*;
pub use startup::*;
pub use state::*;
pub use telemetry::*;
pub use templates::*;
