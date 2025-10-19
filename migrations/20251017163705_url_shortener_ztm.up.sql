-- sqlite_url_shortener.up.sql
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS urls (
  id        INTEGER PRIMARY KEY,
  code      TEXT NOT NULL UNIQUE,
  url       TEXT NOT NULL,
  url_hash  BLOB NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS aliases (
  alias      TEXT PRIMARY KEY,
  target_id  INTEGER NOT NULL REFERENCES urls(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS aliases_target_id_idx ON aliases(target_id);

CREATE VIEW IF NOT EXISTS all_short_codes AS
  SELECT u.code AS code, u.id AS target_id, u.url AS url, 'code'  AS source
  FROM urls u
  UNION ALL
  SELECT a.alias AS code, a.target_id, u.url,       'alias' AS source
  FROM aliases a
  JOIN urls u ON u.id = a.target_id;

/* Trigger: forbid alias == primary code (INSERT/UPDATE) */
CREATE TRIGGER IF NOT EXISTS aliases_block_overlap_ins
BEFORE INSERT ON aliases
WHEN EXISTS (SELECT 1 FROM urls WHERE code = NEW.alias)
BEGIN
  SELECT RAISE(ABORT, 'alias conflicts with existing primary code');
END;

CREATE TRIGGER IF NOT EXISTS aliases_block_overlap_upd
BEFORE UPDATE OF alias ON aliases
WHEN EXISTS (SELECT 1 FROM urls WHERE code = NEW.alias)
BEGIN
  SELECT RAISE(ABORT, 'alias conflicts with existing primary code');
END;

/* Trigger: forbid code == any alias (INSERT/UPDATE) */
CREATE TRIGGER IF NOT EXISTS urls_block_overlap_ins
BEFORE INSERT ON urls
WHEN EXISTS (SELECT 1 FROM aliases WHERE alias = NEW.code)
BEGIN
  SELECT RAISE(ABORT, 'code conflicts with existing alias');
END;

CREATE TRIGGER IF NOT EXISTS urls_block_overlap_upd
BEFORE UPDATE OF code ON urls
WHEN EXISTS (SELECT 1 FROM aliases WHERE alias = NEW.code)
BEGIN
  SELECT RAISE(ABORT, 'code conflicts with existing alias');
END;

DROP VIEW IF EXISTS primary_codes_no_alias;
CREATE VIEW primary_codes_no_alias AS
SELECT u.code AS code
FROM urls u
WHERE NOT EXISTS (
    SELECT 1 FROM aliases a WHERE a.target_id = u.id
);