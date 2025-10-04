use serde::Deserialize;
use std::path::PathBuf;

/// Top-level configuration for the short code generator.
#[derive(Clone, Debug, Deserialize)]
pub struct ShortenerConfig {
    pub length: usize,
    pub alphabet: Option<String>,
    pub engine: EngineConfig,
    pub bit_layout: Option<BitLayoutConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EngineConfig {
    pub kind: EngineKind,
    pub nanoid: Option<NanoIdConfig>,
    pub sequence: Option<SequenceConfig>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineKind {
    Sequence,
    Nanoid,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct NanoIdConfig {}

#[derive(Clone, Debug, Deserialize)]
pub struct SequenceConfig {
    pub block_size: u64,
    pub persist_interval: u64,
    pub state_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BitLayoutConfig {
    pub enabled: bool,
    pub region_bits: u8,
    pub shard_bits: u8,
    pub payload_bits: u8,
    pub region_id: u8,
    pub shard_id: u16,
}

impl ShortenerConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.length < 5 {
            return Err("shortener.length must be >= 5".into());
        }

        if let Some(alpha) = &self.alphabet {
            if alpha.chars().count() < 2 {
                return Err("shortener.alphabet must contain at least 2 distinct chars".into());
            }
            let mut chars: Vec<_> = alpha.chars().collect();
            chars.sort_unstable();
            if chars.windows(2).any(|w| w[0] == w[1]) {
                return Err("shortener.alphabet has duplicate characters".into());
            }
        }

        match self.engine.kind {
            EngineKind::Nanoid => {}
            EngineKind::Sequence => {
                let seq = self
                    .engine
                    .sequence
                    .as_ref()
                    .ok_or("engine.sequence must be provided when kind=Sequence")?;
                if seq.block_size == 0 {
                    return Err("engine.sequence.block_size must be > 0".into());
                }
                if seq.persist_interval == 0 {
                    return Err("engine.sequence.persist_interval must be > 0".into());
                }
            }
        }

        if let Some(b) = &self.bit_layout {
            if b.enabled {
                todo!("bit layout validation not implemented yet");
            }
        }

        Ok(())
    }
}
