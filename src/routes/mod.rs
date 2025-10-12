//! # HTTP Route Handlers
//!
//! This module contains all HTTP route handlers for the URL shortener service.
//! Each handler is responsible for processing specific HTTP requests and returning
//! appropriate responses.
//!
//! ## Available Routes
//!
//! ### Public API (No Authentication Required)
//! - `GET /api/health_check` - Health check endpoint
//! - `GET /api/redirect/{id}` - Redirect to original URL
//! - `POST /api/public/shorten` - Shorten URL (public endpoint)
//!
//! ### Protected API (Requires API Key)
//! - `POST /api/shorten` - Shorten URL (protected endpoint)
//!
//! ### Admin Panel
//! - `GET /admin` - Web interface for management
//!
//! ## Handler Design
//!
//! All handlers follow these principles:
//! - **Async** - All handlers are async functions
//! - **Error Handling** - Comprehensive error handling with proper HTTP status codes
//! - **Logging** - Request tracing and logging for debugging
//! - **Type Safety** - Strong typing with Axum's extractors
//!
//! ## Response Format
//!
//! All API endpoints return responses in the standardized JSON envelope format:
//! - Success responses include the requested data
//! - Error responses include detailed error information
//! - All responses include timestamps and status codes
//!
//! ## Examples
//!
//! ```rust,no_run
//! use url_shortener_ztm_lib::routes::{health_check, post_shorten};
//! use axum::{Router, routing::get, routing::post};
//!
//! let app = Router::new()
//!     .route("/api/health_check", get(health_check))
//!     .route("/api/shorten", post(post_shorten));
//! ```

// module declarations
pub mod admin;
// Module declarations
pub mod docs;
pub mod health_check;
pub mod index;
pub mod redirect;
pub mod shorten;

// re-exports
pub use admin::*;
pub use docs::*;
// Re-exports for convenience
pub use health_check::*;
pub use index::*;
pub use redirect::*;
pub use shorten::*;
