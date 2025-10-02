// src/lib/configuration.rs

// dependencies
use figment::{
    Figment,
    providers::{Env, Format, Yaml},
};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use std::fmt;
use uuid::Uuid;

// struct type to represent the general settings
#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub rate_limiting: RateLimitingSettings,
}

// implement the Display trait for the Settings struct
impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Application Settings:")?;
        writeln!(f, "  Host: {}", self.application.host)?;
        writeln!(f, "  Port: {}", self.application.port)?;
        writeln!(f, "  API Key: {}", self.application.api_key)?;
        writeln!(f, "  Templates: {}", self.application.templates)?;
        writeln!(f, "Database Settings:")?;
        writeln!(f, "  Database Path: {}", self.database.database_path)?;
        writeln!(
            f,
            "  Create if Missing: {}",
            self.database.create_if_missing
        )?;
        writeln!(f, "Rate Limiting Settings:")?;
        writeln!(f, "  Enabled: {}", self.rate_limiting.enabled)?;
        writeln!(f, "  Requests per second: {}", self.rate_limiting.requests_per_second)?;
        writeln!(f, "  Burst size: {}", self.rate_limiting.burst_size)?;
        Ok(())
    }
}

// struct type to represent individual Application settings
#[derive(Clone, Debug, Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub api_key: Uuid,
    pub templates: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseSettings {
    pub database_path: String,
    pub create_if_missing: bool,
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
    pub fn connection_string(&self) -> String {
        if self.database_path == ":memory:" {
            "sqlite::memory:".to_string()
        } else {
            format!("sqlite:{}", self.database_path)
        }
    }
}

// enum type to represent the runtime environment
#[derive(Clone, Debug)]
pub enum Environment {
    Local,
    Production,
}

// methods for the Environment type
impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

// implement the TryFrom trait for the Environment type
impl TryFrom<String> for Environment {
    type Error = String;

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

// function to read the configuration from associated files
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
        .merge(Yaml::file(
            configuration_directory.join(environment_filename),
        ))
        .merge(Env::prefixed("APP_").split("__"))
        .extract()?;

    Ok(settings)
}
