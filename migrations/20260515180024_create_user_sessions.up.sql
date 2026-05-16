CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    ip_address INET NOT NULL,

    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    revoked_at TIMESTAMPTZ
);

CREATE INDEX idx_user_sessions_valid ON user_sessions(user_id)
    WHERE revoked_at IS NULL;

CREATE INDEX idx_user_sessions_cleanup ON user_sessions(
    LEAST(revoked_at, expires_at)
);
