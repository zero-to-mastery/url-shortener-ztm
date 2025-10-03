--migrations/pg/20251003014824_url_shortener_ztm_pg.down.sql
-- Rollback for 20251003014824_url_shortener_ztm_pg.up.sql
DROP TABLE IF EXISTS urls;
