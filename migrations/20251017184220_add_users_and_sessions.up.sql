-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id            TEXT PRIMARY KEY,             -- uuid (string) for portability
    email         TEXT NOT NULL UNIQUE,         -- unique email
    password_hash TEXT NOT NULL,                -- argon2/scrypt/bcrypt hash
    created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Basic index to speed up lookups by email
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Create sessions table for session-based auth (future expansion)
CREATE TABLE IF NOT EXISTS sessions (
    id             TEXT PRIMARY KEY,            -- session id (uuid or random)
    user_id        TEXT NOT NULL,               -- FK to users.id
    created_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at     DATETIME NOT NULL,           -- absolute expiry
    last_active_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user_agent     TEXT,                        -- optional metadata
    ip_address     TEXT,                        -- optional metadata
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Useful indexes for sessions
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);

-- Add owner_id column to urls to support per-user ownership
ALTER TABLE urls ADD COLUMN owner_id TEXT NULL;

-- Optional: create index for owner_id to list a user's URLs quickly
CREATE INDEX IF NOT EXISTS idx_urls_owner_id ON urls(owner_id);

-- Note: We do not add a strict FK on urls.owner_id yet to keep backward
-- compatibility with existing rows (NULL allowed). Once all rows are owned,
-- you can migrate data and enforce FK if desired:
-- ALTER TABLE urls ADD CONSTRAINT fk_urls_owner
--   FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE SET NULL;


