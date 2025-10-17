-- Add down migration script here

-- Down migration for adding users and sessions (PostgreSQL)
SET search_path TO public;

-- Drop indexes
DROP INDEX IF EXISTS idx_urls_owner_id;
DROP INDEX IF EXISTS idx_sessions_expires_at;
DROP INDEX IF EXISTS idx_sessions_user_id;
DROP INDEX IF EXISTS idx_users_email;

-- Remove owner_id column from urls
ALTER TABLE IF EXISTS urls
    DROP COLUMN IF EXISTS owner_id;

-- Drop sessions and users tables
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS users;