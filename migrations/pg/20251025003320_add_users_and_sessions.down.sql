-- Add down migration script here
BEGIN;

ALTER TABLE urls
    DROP COLUMN IF EXISTS created_by,
    DROP COLUMN IF EXISTS password_hash,
    DROP COLUMN IF EXISTS expires_at,
    DROP COLUMN IF EXISTS description;

ALTER TABLE aliases
    DROP COLUMN IF EXISTS created_by,
    DROP COLUMN IF EXISTS password_hash,
    DROP COLUMN IF EXISTS expires_at,
    DROP COLUMN IF EXISTS description;

DROP TABLE IF EXISTS api_keys;
DROP TABLE IF EXISTS refresh_token_devices;
DROP TABLE IF EXISTS sign_in_attempts;
DROP TABLE IF EXISTS users;

DROP EXTENSION IF EXISTS citext;

COMMIT;
