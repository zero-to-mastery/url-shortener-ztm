PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS bloom_snapshots (
  name TEXT PRIMARY KEY,
  data BLOB NOT NULL,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
