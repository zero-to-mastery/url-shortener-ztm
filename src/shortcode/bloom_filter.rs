// shortcode/mod.rs
use crate::database::UrlDatabase;
use anyhow::{Context, Result};
use fastbloom_rs::{BloomFilter, FilterBuilder, Membership};
use parking_lot::RwLock;
use std::{env, fs, path::Path, sync::Arc};

pub const S2L_SNAPSHOT: &str = "./data/s2lbf_snapshot.bin";
pub const L2S_SNAPSHOT: &str = "./data/l2sbf_snapshot.bin";
const EXPECTED: u64 = 100_000_000;
const FPP: f64 = 0.01;
const PAGE: u64 = 50_000;

pub trait ProbSet: Send + Sync {
    fn may_contain(&self, key: &str) -> bool;
    fn insert(&self, key: &str);
    fn save_to_file_with_hashes(&self, path: &str) -> Result<()>;

    fn extend<'a, I>(&self, items: I)
    where
        I: IntoIterator<Item = &'a str>,
        Self: Sized,
    {
        for k in items {
            self.insert(k);
        }
    }
}

#[derive(Clone)]
pub struct BloomPair {
    pub s2l: Arc<dyn ProbSet>,
    pub l2s: Arc<dyn ProbSet>,
}

pub struct LocalBloom {
    inner: RwLock<BloomFilter>,
}

impl LocalBloom {
    pub fn _new(expected: u64, fpp: f64) -> Self {
        let bf = FilterBuilder::new(expected, fpp).build_bloom_filter();
        Self {
            inner: RwLock::new(bf),
        }
    }
    pub fn from_items<I, S>(items: I, expected: u64, fpp: f64) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<[u8]>,
    {
        let mut bf = FilterBuilder::new(expected, fpp).build_bloom_filter();
        for s in items {
            bf.add(s.as_ref());
        }
        Self {
            inner: RwLock::new(bf),
        }
    }
    pub fn from_file(path: &str) -> Result<Self> {
        let bf = BloomFilter::from_file_with_hashes(path);
        Ok(Self {
            inner: RwLock::new(bf),
        })
    }
}

impl ProbSet for LocalBloom {
    fn may_contain(&self, key: &str) -> bool {
        self.inner.read().contains(key.as_bytes())
    }
    fn insert(&self, key: &str) {
        self.inner.write().add(key.as_bytes())
    }

    fn save_to_file_with_hashes(&self, path: &str) -> Result<()> {
        // Create the parent directory for the snapshot if it doesn't exist
        if let Some(parent) = Path::new(path).parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create directory for Bloom snapshot {}",
                    parent.display()
                )
            })?;
        }
        let mut bf = self.inner.write();
        bf.save_to_file_with_hashes(path);

        Ok(())
    }
}

pub async fn build_bloom_pair(db: &Arc<dyn UrlDatabase>) -> Result<BloomPair> {
    let s2l_path = Path::new(S2L_SNAPSHOT);
    let l2s_path = Path::new(L2S_SNAPSHOT);

    // Prefer to load from snapshots if they exist
    if s2l_path.exists() && l2s_path.exists() {
        let s2l = LocalBloom::from_file(s2l_path.to_str().unwrap())?;
        let l2s = LocalBloom::from_file(l2s_path.to_str().unwrap())?;
        tracing::info!("Loaded Bloom snapshots.");
        return Ok(BloomPair {
            s2l: Arc::new(s2l),
            l2s: Arc::new(l2s),
        });
    }

    // First-time build: pull data from DB in pages
    let mut shorts: Vec<Vec<u8>> = Vec::new();
    let longs: Vec<Vec<u8>> = Vec::new();

    let mut offset: u64 = 0;

    loop {
        let batch = db.list_short_codes(offset, PAGE).await?;
        if batch.is_empty() {
            break;
        }
        for rec in &batch {
            shorts.push(rec.as_bytes().to_vec());
            // longs.push(rec.url.as_bytes().to_vec());
        }
        offset += batch.len() as u64;
        if batch.len() < PAGE as usize {
            break;
        }
    }

    let s2l = LocalBloom::from_items(shorts.iter().map(|v| &v[..]), EXPECTED, FPP);
    let l2s = LocalBloom::from_items(longs.iter().map(|v| &v[..]), EXPECTED, FPP);

    if not_disable_bf_snapshots() {
        if let Err(err) = s2l.save_to_file_with_hashes(s2l_path.to_str().unwrap()) {
            tracing::warn!(error = %err, "failed to persist s2l Bloom snapshot");
        }
        if let Err(err) = l2s.save_to_file_with_hashes(l2s_path.to_str().unwrap()) {
            tracing::warn!(error = %err, "failed to persist l2s Bloom snapshot");
        }
    }

    Ok(BloomPair {
        s2l: Arc::new(s2l),
        l2s: Arc::new(l2s),
    })
}

pub(crate) fn not_disable_bf_snapshots() -> bool {
    !matches!(
        env::var("BLOOM_SNAPSHOTS").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}
