--migrations/pg/20251003014824_url_shortener_ztm_pg.up.sql
-- Create urls table
CREATE TABLE urls (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ulrs_url ON urls(url);