-- Add down migration script here
BEGIN;

DROP VIEW   IF EXISTS all_short_codes;

DROP TRIGGER IF EXISTS urls_block_overlap    ON urls;
DROP TRIGGER IF EXISTS aliases_block_overlap ON aliases;

DROP FUNCTION IF EXISTS trg_code_not_in_aliases();
DROP FUNCTION IF EXISTS trg_alias_not_in_urls();

DROP INDEX IF EXISTS aliases_target_id_idx;
DROP TABLE IF EXISTS aliases;

DROP INDEX IF EXISTS uniq_url_hash;
DROP TABLE IF EXISTS urls;

COMMIT;