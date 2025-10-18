-- Add down migration script here
-- sqlite_url_shortener.up.sql
PRAGMA foreign_keys = ON;

DROP VIEW IF EXISTS all_short_codes;

DROP TRIGGER IF EXISTS aliases_block_overlap_ins;
DROP TRIGGER IF EXISTS aliases_block_overlap_upd;
DROP TRIGGER IF EXISTS urls_block_overlap_ins;
DROP TRIGGER IF EXISTS urls_block_overlap_upd;

DROP INDEX IF EXISTS aliases_target_id_idx;
DROP TABLE IF EXISTS aliases;
DROP TABLE IF EXISTS urls;
