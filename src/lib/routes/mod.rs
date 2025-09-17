// src/lib/routes/mod.rs

// module declarations
pub mod health_check;
pub mod redirect;
pub mod shorten;

// re-exports
pub use health_check::*;
pub use redirect::*;
pub use shorten::*;
