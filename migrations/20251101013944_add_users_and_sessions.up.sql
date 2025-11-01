-- Add up migration script here

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS users (
    id                  TEXT PRIMARY KEY,
    email               TEXT NOT NULL COLLATE NOCASE UNIQUE,
    password_hash       BLOB NOT NULL,
    display_name        TEXT,
    is_email_verified   INTEGER NOT NULL DEFAULT 0 CHECK (is_email_verified IN (0,1)),
    created_at          DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    last_login_at       DATETIME,
    jwt_token_version   INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS refresh_token_devices (
    id                INTEGER PRIMARY KEY,
    user_id           TEXT NOT NULL,
    device_id         TEXT NOT NULL,
    current_hash      BLOB NOT NULL,
    previous_hash     BLOB,
    created_at        DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    last_rotated_at   DATETIME,
    absolute_expires  DATETIME NOT NULL,
    revoked_at        DATETIME,
    user_agent        TEXT,
    ip                TEXT,
    UNIQUE(user_id, device_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS email_verifications (
    id           INTEGER PRIMARY KEY,
    user_id      TEXT NOT NULL,
    code         TEXT NOT NULL CHECK (length(code) = 8),
    created_at   DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    expires_at   DATETIME NOT NULL,
    used_at      DATETIME,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS email_change_requests (
    id             INTEGER PRIMARY KEY,
    user_id        TEXT NOT NULL,
    old_email      TEXT NOT NULL COLLATE NOCASE,
    new_email      TEXT NOT NULL COLLATE NOCASE,
    code           TEXT NOT NULL CHECK (length(code) = 8),
    created_at     DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    expires_at     DATETIME,
    confirmed_at   DATETIME,
    applied_at     DATETIME,
    cancelled_at   DATETIME,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS sign_in_attempts (
    id           INTEGER PRIMARY KEY,
    user_id      TEXT NOT NULL,
    success      INTEGER NOT NULL CHECK (success IN (0,1)),
    ip           TEXT,
    created_at   DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE IF NOT EXISTS api_keys (
    id            TEXT PRIMARY KEY, 
    user_id       TEXT NOT NULL,
    name          TEXT,
    key_prefix    TEXT NOT NULL,
    token_hash    BLOB NOT NULL,
    created_at    DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    last_used_at  DATETIME,
    revoked_at    DATETIME,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

ALTER TABLE urls ADD COLUMN created_by TEXT REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE urls ADD COLUMN password_hash BLOB;
ALTER TABLE urls ADD COLUMN expires_at DATETIME;
ALTER TABLE urls ADD COLUMN description TEXT;

ALTER TABLE aliases ADD COLUMN created_by TEXT REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE aliases ADD COLUMN password_hash BLOB;
ALTER TABLE aliases ADD COLUMN expires_at DATETIME;
ALTER TABLE aliases ADD COLUMN description TEXT;
