// src/lib/lib.rs

// this is the library crate for the url-shortener-v1 project

// module declarations
pub mod configuration;
pub mod errors;
pub mod routes;
pub mod startup;
pub mod telemetry;

// re-exports
pub use configuration::*;
pub use errors::*;
pub use startup::*;
pub use telemetry::*;
