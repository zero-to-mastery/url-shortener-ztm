BEGIN;

PRAGMA foreign_keys = OFF;

DROP TABLE IF EXISTS api_keys;
DROP TABLE IF EXISTS sign_in_attempts;
DROP TABLE IF EXISTS email_change_requests;
DROP TABLE IF EXISTS email_verifications;
DROP TABLE IF EXISTS refresh_token_devices;
DROP TABLE IF EXISTS users;

ALTER TABLE urls DROP COLUMN created_by;
ALTER TABLE urls DROP COLUMN password_hash;
ALTER TABLE urls DROP COLUMN expires_at;
ALTER TABLE urls DROP COLUMN description;

ALTER TABLE aliases DROP COLUMN created_by;
ALTER TABLE aliases DROP COLUMN password_hash;
ALTER TABLE aliases DROP COLUMN expires_at;
ALTER TABLE aliases DROP COLUMN description;

PRAGMA foreign_keys = ON;

COMMIT;
