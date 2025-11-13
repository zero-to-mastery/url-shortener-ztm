CREATE EXTENSION IF NOT EXISTS citext;

CREATE TYPE authentication_action AS ENUM ('verify_email','reset_password','change_email');

CREATE TABLE IF NOT EXISTS authentication_challenges (
    id           BIGSERIAL PRIMARY KEY,
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action       authentication_action NOT NULL,
    target       CITEXT,
    code_hash    BYTEA NOT NULL,
    attempts     INT  NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    meta         JSONB,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at   TIMESTAMPTZ NOT NULL CHECK (expires_at > created_at),
    confirmed_at TIMESTAMPTZ
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_authentication_challenges_user_id_action
    ON authentication_challenges(user_id, action)
    WHERE confirmed_at IS NULL;



CREATE OR REPLACE FUNCTION auth_lock_key(p_user_id uuid, p_action authentication_action)
RETURNS bigint LANGUAGE sql IMMUTABLE AS $$
    SELECT hashtextextended(p_user_id::text || ':' || p_action::text, 0);
$$;

CREATE TYPE challenge_upsert_status AS ENUM ('inserted','updated','cooldown');
CREATE OR REPLACE FUNCTION create_or_refresh_auth_challenge(
    p_user_id uuid,
    p_action authentication_action,
    p_target citext,
    p_code_hash bytea,
    p_meta jsonb,
    p_expires_at timestamptz,
    p_cooldown_secs int DEFAULT 60
)
RETURNS TABLE(status challenge_upsert_status, challenge_id bigint, seconds_remaining int)
LANGUAGE plpgsql AS $$
DECLARE
    v_id bigint;
    v_created timestamptz;
    v_next timestamptz;
    v_rem int;
BEGIN
    PERFORM pg_advisory_xact_lock(auth_lock_key(p_user_id, p_action));

    SELECT id, created_at
    INTO v_id, v_created
    FROM authentication_challenges
    WHERE user_id = p_user_id AND action = p_action AND confirmed_at IS NULL;

    IF v_id IS NULL THEN
        INSERT INTO authentication_challenges(user_id, action, target, code_hash, meta, expires_at)
        VALUES (p_user_id, p_action, p_target, p_code_hash, p_meta, p_expires_at)
        RETURNING id INTO v_id;

        status := 'inserted'; challenge_id := v_id; seconds_remaining := 0;
        RETURN NEXT; RETURN;
    END IF;

    v_next := v_created + make_interval(secs => p_cooldown_secs);

    IF now() < v_next THEN
        v_rem := GREATEST(CEIL(EXTRACT(EPOCH FROM (v_next - now())))::int, 0);
        status := 'cooldown'; challenge_id := v_id; seconds_remaining := v_rem;
        RETURN NEXT; RETURN;
    END IF;

    UPDATE authentication_challenges
    SET target = p_target,
        code_hash = p_code_hash,
        meta = p_meta,
        created_at = now(),
        expires_at = p_expires_at,
        attempts = 0,
        confirmed_at = NULL
    WHERE id = v_id;

    status := 'updated'; challenge_id := v_id; seconds_remaining := 0;
    RETURN NEXT; RETURN;
END;
$$;

CREATE OR REPLACE FUNCTION confirm_auth_challenge(
    p_user_id uuid,
    p_action authentication_action,
    p_confirmed_at timestamptz DEFAULT now()
)
RETURNS boolean
LANGUAGE plpgsql AS $$
DECLARE
    v_rows int;
BEGIN
    PERFORM pg_advisory_xact_lock(auth_lock_key(p_user_id, p_action));

    UPDATE authentication_challenges
    SET confirmed_at = p_confirmed_at
    WHERE user_id = p_user_id
        AND action = p_action
        AND confirmed_at IS NULL
        AND expires_at > now()
    RETURNING 1
    INTO v_rows;

    RETURN COALESCE(v_rows, 0) = 1;
END;
$$;