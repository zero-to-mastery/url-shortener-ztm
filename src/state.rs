//! # Application State
//!
//! This module defines the application state that is shared across all request handlers.
//! The state contains all the necessary dependencies and configuration that handlers
//! need to process requests.
//!
//! ## State Contents
//!
//! The application state includes:
//! - Database connection for URL storage and retrieval
//! - API key for authentication
//! - Template directory path for web interface rendering
//!
//! ## Thread Safety
//!
//! The state is designed to be safely shared across multiple threads and async tasks.
//! All contained types implement `Send + Sync` to ensure thread safety.
//!
//! ## Usage
//!
//! The state is automatically injected into handlers using Axum's state extraction:
//!
//! ```rust,no_run
//! use axum::extract::State;
//! use url_shortener_ztm_lib::state::AppState;
//!
//! async fn handler(State(state): State<AppState>) -> String {
//!     // Access database, API key, and template directory
//!     format!("API key: {}", state.api_key)
//! }
//! ```

use crate::configuration::Settings;
use crate::core::security::jwt::JwtKeys;

use crate::database::UrlDatabase;
use crate::features::{auth::AuthService, users::UserService};

use crate::generator::ShortCodeGenerator;
use crate::shortcode::bloom_filter::BloomState;
use axum_macros::FromRef;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

/// Application state shared across all request handlers.
///
/// This struct contains all the dependencies and configuration needed by handlers
/// to process requests. It's designed to be safely shared across multiple threads
/// and async tasks.
///
/// # Thread Safety
///
/// This struct implements `Clone` and `FromRef` to work with Axum's state system.
/// All contained types are `Send + Sync` to ensure thread safety.
///
/// # Fields
///
/// * `database` - Database connection for URL storage operations
/// * `api_key` - UUID-based API key for authentication
/// * `template_dir` - Directory path containing Tera template files
///
/// # Examples
///
/// ```rust,no_run
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
///     max_connections: Some(16),
///     min_connections: Some(4),
/// };
/// let database = Arc::new(SqliteUrlDatabase::from_config(&config).await?);
/// let api_key = Uuid::new_v4();
/// let template_dir = "templates".to_string();
/// // let settings = get_configuration().expect("Failed to read configuration");
///
/// // let state = AppState::new(database, api_key, template_dir, settings);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, FromRef)]
pub struct AppState {
    /// Database connection for URL storage and retrieval operations
    pub database: Arc<dyn UrlDatabase>,
    /// Short code generator for creating unique short URLs
    pub code_generator: Arc<dyn ShortCodeGenerator>,
    pub blooms: BloomState,
    /// The set of characters that can be used when generating short codes. \
    /// Typically includes alphanumeric characters (e.g., `a-z`, `A-Z`, `0-9`).
    pub allowed_chars: HashSet<char>,
    /// UUID-based API key for authenticating protected endpoints
    pub api_key: Uuid,
    /// Directory path containing Tera template files for web interface
    pub template_dir: String,
    pub jwt: JwtKeys,
    pub config: Settings,

    // pub db_pool: Arc<db::DbPool>,
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
}

impl AppState {}
