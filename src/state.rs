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

use crate::database::UrlDatabase;
use axum_macros::FromRef;
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
/// };
/// let database = Arc::new(SqliteUrlDatabase::from_config(&config).await?);
/// let api_key = Uuid::new_v4();
/// let template_dir = "templates".to_string();
///
/// let state = AppState::new(database, api_key, template_dir);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, FromRef)]
pub struct AppState {
    /// Database connection for URL storage and retrieval operations
    pub database: Arc<dyn UrlDatabase>,
    /// UUID-based API key for authenticating protected endpoints
    pub api_key: Uuid,
    /// Directory path containing Tera template files for web interface
    pub template_dir: String,
}

impl AppState {
    /// Creates a new application state instance.
    ///
    /// This constructor initializes the application state with all required dependencies.
    /// The state will be shared across all request handlers and must be thread-safe.
    ///
    /// # Arguments
    ///
    /// * `database` - Database connection wrapped in `Arc` for shared ownership
    /// * `api_key` - UUID-based API key for authentication
    /// * `template_dir` - Directory path containing Tera template files
    ///
    /// # Returns
    ///
    /// Returns a new `AppState` instance ready for use with Axum handlers.
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
    /// };
    /// let database = Arc::new(SqliteUrlDatabase::from_config(&config).await?);
    /// let api_key = Uuid::parse_str("e4125dd1-3d3e-43a1-bc9c-dc0ba12ad4b5")?;
    /// let template_dir = "templates".to_string();
    ///
    /// let state = AppState::new(database, api_key, template_dir);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(database: Arc<dyn UrlDatabase>, api_key: Uuid, template_dir: String) -> Self {
        Self {
            database,
            api_key,
            template_dir,
        }
    }
}
