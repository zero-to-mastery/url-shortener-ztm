// src/lib/configuration.rs

// dependencies
use figment::providers::Env;
use figment::{
    Figment,
    providers::{Format, Yaml},
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
        writeln!(f, "  Database Type: {:?}", self.database.r#type)?;
        writeln!(f, "  Database URL: {}", self.database.url)?;
        writeln!(
            f,
            "  Create if Missing: {}",
            self.database.create_if_missing
        )?;
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

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    Sqlite,
    Postgres,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseSettings {
    pub r#type: DatabaseType,
    #[serde(alias = "database_path")]
    pub url: String,
    #[serde(default)]
    pub create_if_missing: bool,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        match self.r#type {
            DatabaseType::Sqlite => {
                if self.url == ":memory:" {
                    "sqlite::memory:".to_string()
                } else {
                    self.url.clone()
                }
            }
            _ => self.url.clone(),
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
