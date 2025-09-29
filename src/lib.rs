// src/lib/lib.rs

// this is the library crate for the url-shortener-v1 project

// module declarations
pub mod configuration;
pub mod database;
pub mod errors;
pub mod middleware;
pub mod response;
pub mod routes;
pub mod startup;
pub mod state;
pub mod telemetry;
pub mod templates;

// re-exports
pub use configuration::*;
pub use errors::*;
pub use middleware::*;
pub use response::*;
pub use startup::*;
pub use state::*;
pub use telemetry::*;
pub use templates::*;
