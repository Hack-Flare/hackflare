CREATE TABLE users (
    -- Hack Club Auth's own ID
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    slack_id TEXT NOT NULL UNIQUE,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    verification_status TEXT NOT NULL,
    ysws_eligible BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- HCA Access Data
    hca_access_token TEXT NOT NULL,
    hca_refresh_token TEXT NOT NULL,
    hca_token_expires_at TIMESTAMPTZ NOT NULL
);
