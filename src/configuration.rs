//! # Configuration Management
//!
//! This module handles application configuration using a layered approach:
//! 1. Base configuration from YAML files
//! 2. Environment-specific overrides
//! 3. Environment variable overrides
//!
//! ## Configuration Files
//!
//! - `configuration/base.yml` - Base configuration shared across environments
//! - `configuration/local.yml` - Local development overrides
//! - `configuration/production.yml` - Production environment settings
//!
//! ## Environment Variables
//!
//! Any configuration value can be overridden using environment variables with the `APP_` prefix.
//! Use double underscores (`__`) to separate nested keys:
//!
//! ```bash
//! APP_APPLICATION__PORT=3000
//! APP_APPLICATION__API_KEY=your-api-key-here
//! APP_DATABASE__DATABASE_PATH=./my-database.db
//! ```
//!
//! ## Example Configuration
//!
//! ```yaml
//! # configuration/base.yml
//! application:
//!   port: 8000
//!   host: "127.0.0.1"
//!   api_key: "e4125dd1-3d3e-43a1-bc9c-dc0ba12ad4b5"
//!   templates: "templates"
//!
//! database:
//!   url: "database.db"
//!   create_if_missing: true
//! ```

use figment::{
    Figment,
    providers::{Env, Format, Yaml},
};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use std::fmt;
use uuid::Uuid;

use crate::generator::config::ShortenerConfig;

/// Complete application settings containing all configuration sections.
///
/// This struct represents the entire application configuration, including
/// application-specific settings and database configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    /// Application-specific settings (server, API, templates)
    pub application: ApplicationSettings,
    /// Database connection and configuration settings
    pub database: DatabaseSettings,
    pub rate_limiting: RateLimitingSettings,
    pub shortener: ShortenerConfig,
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Application Settings:")?;
        writeln!(f, "  Host: {}", self.application.host)?;
        writeln!(f, "  Port: {}", self.application.port)?;
        writeln!(f, "  API Key: {}", self.application.api_key)?;
        writeln!(f, "  Templates: {}", self.application.templates)?;
        writeln!(f, "Database Settings:")?;
        writeln!(f, "  Database Type: {:?}", self.database.r#type)?;
        writeln!(f, "  Database URL: {}", self.database.url)?;
        writeln!(
            f,
            "  Create if Missing: {}",
            self.database.create_if_missing
        )?;
        writeln!(f, "Rate Limiting Settings:")?;
        writeln!(f, "  Enabled: {}", self.rate_limiting.enabled)?;
        writeln!(
            f,
            "  Requests per second: {}",
            self.rate_limiting.requests_per_second
        )?;
        writeln!(f, "  Burst size: {}", self.rate_limiting.burst_size)?;
        Ok(())
    }
}

/// Application-specific configuration settings.
///
/// Contains settings related to the HTTP server, API authentication,
/// and template rendering.
#[derive(Clone, Debug, Deserialize)]
pub struct ApplicationSettings {
    /// Port number for the HTTP server to listen on
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    /// Host address for the HTTP server to bind to
    pub host: String,
    /// UUID-based API key for authenticating requests to protected endpoints
    pub api_key: Uuid,
    /// Directory path containing Tera template files
    pub templates: String,
}

/// Supported database types.
///
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    Sqlite,
    Postgres,
}

/// Database configuration settings.
///
/// Contains settings for database connection and initialization.
#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseSettings {
    /// Type of the database (e.g., SQLite, PostgreSQL)
    pub r#type: DatabaseType,
    /// Path to the SQLite database file (or ":memory:" for in-memory database)
    #[serde(alias = "database_path")]
    pub url: String,
    /// Whether to create the database file if it doesn't exist
    #[serde(default)]
    pub create_if_missing: bool,
    #[serde(default)]
    pub max_connections: Option<u32>,
    #[serde(default)]
    pub min_connections: Option<u32>,
}

// struct type to represent rate limiting settings
#[derive(Clone, Debug, Deserialize)]
pub struct RateLimitingSettings {
    pub enabled: bool,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub requests_per_second: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub burst_size: u32,
}

impl DatabaseSettings {
    /// Generates the SQLite connection string from the database path.
    ///
    /// # Returns
    ///
    /// - `"sqlite::memory:"` if the database path is `":memory:"`
    /// - `"sqlite:{path}"` for file-based databases
    ///
    /// # Examples
    ///
    /// ```rust
    /// use url_shortener_ztm_lib::DatabaseType;
    /// use url_shortener_ztm_lib::configuration::DatabaseSettings;
    ///
    /// let config = DatabaseSettings {
    ///     r#type: DatabaseType::Sqlite,
    ///     url: "database.db".to_string(),
    ///     create_if_missing: true,
    ///     max_connections: Some(16),
    ///     min_connections: Some(4),
    /// };
    /// assert_eq!(config.connection_string(), "sqlite:database.db");
    ///
    /// let memory_config = DatabaseSettings {
    ///     r#type: DatabaseType::Sqlite,
    ///     url: ":memory:".to_string(),
    ///     create_if_missing: true,
    ///     max_connections: Some(16),
    ///     min_connections: Some(4),
    /// };
    /// assert_eq!(memory_config.connection_string(), "sqlite::memory:");
    /// ```
    pub fn connection_string(&self) -> String {
        match self.r#type {
            DatabaseType::Sqlite => {
                if self.url == ":memory:" {
                    "sqlite::memory:".to_string()
                } else {
                    format!("sqlite:{}", self.url)
                }
            }
            _ => self.url.clone(),
        }
    }
}

/// Runtime environment configuration.
///
/// Determines which configuration file to load and affects
/// various application behaviors.
#[derive(Clone, Debug)]
pub enum Environment {
    /// Local development environment
    Local,
    /// Production environment
    Production,
}

impl Environment {
    /// Returns the string representation of the environment.
    ///
    /// # Returns
    ///
    /// - `"local"` for `Environment::Local`
    /// - `"production"` for `Environment::Production`
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    /// Attempts to create an `Environment` from a string.
    ///
    /// # Arguments
    ///
    /// * `s` - The environment string (case-insensitive)
    ///
    /// # Returns
    ///
    /// - `Ok(Environment::Local)` for "local"
    /// - `Ok(Environment::Production)` for "production"
    /// - `Err(String)` for any other value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use url_shortener_ztm_lib::configuration::Environment;
    ///
    /// assert!(Environment::try_from("local".to_string()).is_ok());
    /// assert!(Environment::try_from("PRODUCTION".to_string()).is_ok());
    /// assert!(Environment::try_from("staging".to_string()).is_err());
    /// ```
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}

/// Loads application configuration from files and environment variables.
///
/// This function implements a layered configuration system:
/// 1. Loads base configuration from `configuration/base.yml`
/// 2. Loads environment-specific overrides from `configuration/{environment}.yml`
/// 3. Applies environment variable overrides with `APP_` prefix
///
/// # Environment Detection
///
/// The environment is determined by the `APP_ENVIRONMENT` environment variable.
/// If not set, defaults to `"local"`.
///
/// # Environment Variables
///
/// Any configuration value can be overridden using environment variables:
/// - `APP_APPLICATION__PORT=3000`
/// - `APP_APPLICATION__API_KEY=your-key-here`
/// - `APP_DATABASE__DATABASE_PATH=./my-db.db`
///
/// # Returns
///
/// Returns `Ok(Settings)` if configuration is successfully loaded, or
/// `Err(Box<figment::Error>)` if there's an error reading files or parsing configuration.
///
/// # Errors
///
/// This function will return an error if:
/// - The current directory cannot be determined
/// - Configuration files cannot be read
/// - The `APP_ENVIRONMENT` variable contains an invalid value
/// - Configuration parsing fails
/// - Environment variable parsing fails
///
/// # Examples
///
/// ```rust,no_run
/// use url_shortener_ztm_lib::configuration::get_configuration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Load configuration with default environment (local)
/// let config = get_configuration()?;
///
/// // Load configuration with custom environment
/// unsafe { std::env::set_var("APP_ENVIRONMENT", "production"); }
/// let config = get_configuration()?;
/// # Ok(())
/// # }
/// ```
pub fn get_configuration() -> Result<Settings, Box<figment::Error>> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    let environment_filename = format!("{}.yml", environment.as_str());

    let settings: Settings = Figment::new()
        .merge(Yaml::file(configuration_directory.join("base.yml")))
        .merge(Yaml::file(configuration_directory.join("generator.yml")))
        .merge(Yaml::file(
            configuration_directory.join(environment_filename),
        ))
        .merge(Env::prefixed("APP_").split("__"))
        .extract()?;

    Ok(settings)
}
