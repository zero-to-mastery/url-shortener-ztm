//! # Template Rendering
//!
//! This module provides template rendering capabilities using the Tera template engine.
//! It handles loading and compiling HTML templates for the web interface.
//!
//! ## Features
//!
//! - **Tera Integration** - Uses the Tera template engine for HTML rendering
//! - **Template Caching** - Templates are compiled once and cached for performance
//! - **Error Handling** - Comprehensive error handling for template operations
//! - **Static Compilation** - Templates are loaded at startup for optimal performance
//!
//! ## Template Directory Structure
//!
//! Templates should be placed in the configured template directory:
//!
//! ```text
//! templates/
//! ├── base.html          # Base template with common layout
//! └── index.html         # Home page template
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use url_shortener_ztm_lib::templates::build_templates;
//! use url_shortener_ztm_lib::DatabaseType;
//! use url_shortener_ztm_lib::state::AppState;
//! use tera::Context;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use url_shortener_ztm_lib::database::SqliteUrlDatabase;
//! use url_shortener_ztm_lib::configuration::DatabaseSettings;
//! use std::sync::Arc;
//! use uuid::Uuid;
//!
//! let config = DatabaseSettings {
//!     r#type: DatabaseType::Sqlite,
//!     url: "database.db".to_string(),
//!     create_if_missing: true,
//!     max_connections: Some(16),
//!     min_connections: Some(4),
//! };
//! let database = Arc::new(SqliteUrlDatabase::from_config(&config).await?);
//! let api_key = Uuid::new_v4();
//! // let settings = get_configuration().expect("Failed to read configuration");
//! // let state = AppState::new(database, api_key, "templates".to_string(), settings);
//! // let templates = build_templates(state)?;
//!
//! let mut context = Context::new();
//! context.insert("title", "My Page");
//! context.insert("message", "Hello, World!");
//!
//! // let html = templates.render("index.html", &context)?;
//! # Ok(())
//! # }
//! ```

use crate::AppState;
use std::sync::OnceLock;
use tera::{Error, Tera};

/// Global template cache for compiled Tera templates.
///
/// This static variable holds the compiled templates to avoid recompiling
/// them on every request. Templates are loaded once at startup and cached
/// for the lifetime of the application.
static COMPILED_TEMPLATES: OnceLock<Tera> = OnceLock::new();

/// Loads and compiles Tera templates from the specified directory.
///
/// This function scans the template directory for HTML files and compiles
/// them into a Tera instance. The templates are then ready for rendering.
///
/// # Arguments
///
/// * `template_dir` - Path to the directory containing template files
///
/// # Returns
///
/// Returns `Ok(Tera)` if templates are successfully loaded and compiled,
/// or `Err(Error)` if there's an error reading or parsing templates.
///
/// # Template Discovery
///
/// Tera automatically discovers template files with the following extensions:
/// - `.html`
/// - `.tera`
/// - `.jinja`
/// - `.jinja2`
///
/// # Internal Usage
///
/// This function is used internally by [`build_templates`] and is not
/// intended for direct use by library consumers.
fn load_templates(template_dir: String) -> Result<Tera, Error> {
    let templates = Tera::new(&template_dir)?;
    Ok(templates)
}

/// Builds and returns the compiled template engine.
///
/// This function loads templates from the application state's template directory
/// and returns a reference to the compiled Tera instance. Templates are compiled
/// once and cached for the lifetime of the application.
///
/// # Arguments
///
/// * `state` - Application state containing the template directory path
///
/// # Returns
///
/// Returns `Ok(&'static Tera)` if templates are successfully loaded and compiled,
/// or `Err(Error)` if there's an error reading or parsing templates.
///
/// # Caching
///
/// Templates are compiled once and cached in a static variable. Subsequent
/// calls to this function will return the same compiled instance.
///
/// # Examples
///
/// ```rust,no_run
/// use url_shortener_ztm_lib::templates::build_templates;
/// use url_shortener_ztm_lib::DatabaseType;
/// use url_shortener_ztm_lib::state::AppState;
/// use tera::Context;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use url_shortener_ztm_lib::database::SqliteUrlDatabase;
/// use url_shortener_ztm_lib::configuration::DatabaseSettings;
/// use std::sync::Arc;
/// use uuid::Uuid;
///
/// let config = DatabaseSettings {
///     r#type: DatabaseType::Sqlite,
///     url: "database.db".to_string(),
///     create_if_missing: true,
///     max_connections: Some(16),
///     min_connections: Some(4),
/// };
/// let database = Arc::new(SqliteUrlDatabase::from_config(&config).await?);
/// let api_key = Uuid::new_v4();
/// // let settings = get_configuration().expect("Failed to read configuration");
/// // let state = AppState::new(database, api_key, "templates".to_string(), settings);
/// // let templates = build_templates(state)?;
///
/// let mut context = Context::new();
/// context.insert("title", "My Page");
/// // let html = templates.render("index.html", &context)?;
/// # Ok(())
/// # }
/// ```
pub fn build_templates(state: AppState) -> Result<&'static Tera, Error> {
    let dir = state.template_dir;
    let templates = load_templates(dir)?;
    Ok(COMPILED_TEMPLATES.get_or_init(|| templates))
}
