// src/lib/templates.rs

// renderer for Tera templates

// dependencies
use crate::AppState;
use std::sync::OnceLock;
use tera::{Error, Tera};

// static variable to hold the initialized templates
static COMPILED_TEMPLATES: OnceLock<Tera> = OnceLock::new();

// function to load the Tera templates from the base html files in the /templates folder
fn load_templates(template_dir: String) -> Result<Tera, Error> {
    let templates = Tera::new(&template_dir)?;
    Ok(templates)
}

// function to build the templates
pub fn build_templates(state: AppState) -> Result<&'static Tera, Error> {
    let dir = state.template_dir;
    let templates = load_templates(dir)?;
    Ok(COMPILED_TEMPLATES.get_or_init(|| templates))
}
