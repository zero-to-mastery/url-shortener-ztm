// src/lib/configuration.rs

// dependencies
use figment::{
    Figment,
    providers::{Env, Format, Yaml},
};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

// struct type to represent the general settings
#[derive(Clone, Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
}

// struct type to represent individual Application settings
#[derive(Clone, Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

// enum type to represent the runtime environment
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

    let environment_filename = format!("{}.yaml", environment.as_str());

    let settings: Settings = Figment::new()
        .merge(Yaml::file(configuration_directory.join("base.yaml")))
        .merge(Yaml::file(
            configuration_directory.join(environment_filename),
        ))
        .merge(Env::prefixed("APP_").split("__"))
        .extract()?;

    Ok(settings)
}
