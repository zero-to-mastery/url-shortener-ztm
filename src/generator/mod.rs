use std::sync::Arc;

/// Default base62 alphabet: 0-9, A-Z, a-z (62 characters).
pub const DEFAULT_ALPHABET: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b',
    'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z',
];

#[derive(Debug)]
pub enum GeneratorError {
    /// The numeric value exceeded the available encoding space (increase `length`).
    ExhaustedSpace,
    /// I/O error during persistence (used by the sequence engine).
    Io(std::io::Error),
    /// Unexpected internal error.
    Internal(&'static str),
}

impl From<std::io::Error> for GeneratorError {
    fn from(e: std::io::Error) -> Self {
        GeneratorError::Io(e)
    }
}

/// A generator is only responsible for **producing candidate short codes**.
/// It does **not** handle deduplication, database writes, or caching.
pub trait ShortCodeGenerator: Send + Sync {
    /// Generate a new short code.
    fn generate(&self) -> Result<String, GeneratorError>;

    /// Engine name, used for logging/identification.
    fn name(&self) -> &'static str;
}

pub mod config;
mod nanoid;
mod sequence;

pub use nanoid::NanoIdEngine;
pub use sequence::SequenceEngine;

use crate::generator::config::{EngineKind, ShortenerConfig};

/// Factory: builds the appropriate generator engine based on common
/// configuration fields and the engine-specific settings.
pub fn build_generator(cfg: &ShortenerConfig) -> Arc<dyn ShortCodeGenerator> {
    cfg.validate().expect("invalid shortener config");

    match cfg.engine.kind {
        EngineKind::Nanoid => Arc::new(NanoIdEngine::new(cfg.length, cfg.alphabet.clone())),
        EngineKind::Sequence => {
            let seq: &config::SequenceConfig = cfg
                .engine
                .sequence
                .as_ref()
                .expect("sequence config must exist when kind=Sequence");
            Arc::new(SequenceEngine::new(
                cfg.length,
                cfg.alphabet.clone(),
                seq.block_size.max(1),
                seq.persist_interval.max(1),
                seq.state_path.clone(),
            ))
        }
    }
}
