-- migrations/20250917043645_url_shortener_ztm.up.sql
-- Create urls table
CREATE TABLE urls (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL
);
