// shortcode/mod.rs
use crate::database::UrlDatabase;
use anyhow::{Context, Result, anyhow};
use fastbloom_rs::{BloomFilter, FilterBuilder, Hashes, Membership};
use parking_lot::RwLock;
use std::{env, sync::Arc};

pub const S2L_SNAPSHOT_KEY: &str = "short_to_long";
const EXPECTED: u64 = 10_000_000;
const FPP: f64 = 0.01;
const PAGE: u64 = 50_000;

pub trait ProbSet: Send + Sync {
    fn may_contain(&self, key: &str) -> bool;
    fn insert(&self, key: &str);
    fn snapshot(&self) -> Result<Vec<u8>>;

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
pub struct BloomState {
    pub s2l: Arc<dyn ProbSet>,
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

    pub fn from_snapshot(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 4 {
            return Err(anyhow!("Bloom snapshot payload too small"));
        }
        let hashes = u32::from_be_bytes(bytes[..4].try_into()?);
        let body = &bytes[4..];
        assert_eq!(body.len() % 8, 0);

        let mut words = Vec::<u64>::with_capacity(body.len() / 8);
        for chunk in body.chunks_exact(8) {
            words.push(u64::from_ne_bytes(chunk.try_into()?));
        }
        let filter = fastbloom_rs::BloomFilter::from_u64_array(&words, hashes);

        Ok(Self {
            inner: RwLock::new(filter),
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

    fn snapshot(&self) -> Result<Vec<u8>> {
        let bf = self.inner.read();
        let mut payload = Vec::with_capacity(4 + bf.get_u8_array().len());
        payload.extend_from_slice(&bf.hashes().to_be_bytes());
        payload.extend_from_slice(bf.get_u8_array());
        Ok(payload)
    }
}

pub async fn build_bloom_state(db: &Arc<dyn UrlDatabase>) -> Result<BloomState> {
    if let Some(bytes) = db
        .load_bloom_snapshot(S2L_SNAPSHOT_KEY)
        .await
        .context("failed to load s2l bloom snapshot from database")?
    {
        let s2l = LocalBloom::from_snapshot(&bytes)
            .context("failed to decode s2l bloom snapshot payload")?;
        tracing::info!("Loaded Bloom snapshot from database.");
        return Ok(BloomState { s2l: Arc::new(s2l) });
    }

    // First-time build: pull data from DB in pages
    let mut shorts: Vec<Vec<u8>> = Vec::new();

    let mut offset: u64 = 0;

    loop {
        let batch = db.list_short_codes(offset, PAGE).await?;
        if batch.is_empty() {
            break;
        }
        for rec in &batch {
            shorts.push(rec.as_bytes().to_vec());
        }
        offset += batch.len() as u64;
        if batch.len() < PAGE as usize {
            break;
        }
    }

    let s2l = LocalBloom::from_items(shorts.iter().map(|v| &v[..]), EXPECTED, FPP);

    if not_disable_bf_snapshots() {
        match s2l.snapshot() {
            Ok(bytes) => {
                if let Err(err) = db
                    .save_bloom_snapshot(S2L_SNAPSHOT_KEY, &bytes)
                    .await
                    .context("failed to persist s2l bloom snapshot to database")
                {
                    tracing::warn!(error = %err, "failed to persist s2l Bloom snapshot");
                }
            }
            Err(err) => {
                tracing::warn!(error = %err, "unable to serialize s2l Bloom snapshot");
            }
        }
    }

    Ok(BloomState { s2l: Arc::new(s2l) })
}

pub(crate) fn not_disable_bf_snapshots() -> bool {
    !matches!(
        env::var("BLOOM_SNAPSHOTS").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}
