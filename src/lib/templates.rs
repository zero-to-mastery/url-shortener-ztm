// src/lib/templates.rs

// renderer for Tera templates

// dependencies
use std::sync::OnceLock;
use tera::{Error, Tera};

// static variable to hold the initialized templates
static COMPILED_TEMPLATES: OnceLock<Tera> = OnceLock::new();

// function to load the Tera templates from the base html files in the /templates folder
fn load_templates() -> Result<Tera, Error> {
    let templates = Tera::new("templates/**/*")?;
    Ok(templates)
}

// function to build the templates
pub fn build_templates() -> Result<&'static Tera, Error> {
    let templates = load_templates()?;
    Ok(COMPILED_TEMPLATES.get_or_init(|| templates))
}
