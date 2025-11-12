DROP FUNCTION IF EXISTS confirm_auth_challenge(uuid, authentication_action, timestamptz);
DROP FUNCTION IF EXISTS create_or_refresh_auth_challenge(uuid, authentication_action, citext, bytea, jsonb, timestamptz, int);
DROP FUNCTION IF EXISTS auth_lock_key(uuid, authentication_action);

DROP TYPE IF EXISTS challenge_upsert_status;

DROP TABLE IF EXISTS authentication_challenges;

DROP TYPE IF EXISTS authentication_action;
