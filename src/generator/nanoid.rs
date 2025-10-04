use crate::generator::DEFAULT_ALPHABET;

use super::{GeneratorError, ShortCodeGenerator};

pub struct NanoIdEngine {
    len: usize,
    alphabet: Vec<char>,
}

impl NanoIdEngine {
    pub fn new(len: usize, alphabet: Option<String>) -> Self {
        let alpha = alphabet.unwrap_or_else(|| DEFAULT_ALPHABET.iter().collect());
        NanoIdEngine {
            len,
            alphabet: alpha.chars().collect(),
        }
    }
}

impl ShortCodeGenerator for NanoIdEngine {
    fn generate(&self) -> Result<String, GeneratorError> {
        let len: usize = self.len;
        Ok(nanoid::nanoid!(len, &self.alphabet))
    }

    fn name(&self) -> &'static str {
        "nanoid"
    }
}
