-- Down migration for adding users and sessions

-- Drop indexes
DROP INDEX IF EXISTS idx_urls_owner_id;
DROP INDEX IF EXISTS idx_sessions_expires_at;
DROP INDEX IF EXISTS idx_sessions_user_id;
DROP INDEX IF EXISTS idx_users_email;

-- Remove owner_id column from urls
-- SQLite does not support DROP COLUMN directly; need table rebuild.
-- We'll perform a safe table recreation to drop the column if it exists.

PRAGMA foreign_keys=off;

BEGIN TRANSACTION;

-- Create a temporary table without owner_id
CREATE TABLE IF NOT EXISTS urls_tmp (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL
);

-- Copy data back (ignore owner_id)
INSERT INTO urls_tmp (id, url)
SELECT id, url FROM urls;

-- Replace old table
DROP TABLE urls;
ALTER TABLE urls_tmp RENAME TO urls;

COMMIT;

PRAGMA foreign_keys=on;

-- Drop sessions and users tables
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS users;


