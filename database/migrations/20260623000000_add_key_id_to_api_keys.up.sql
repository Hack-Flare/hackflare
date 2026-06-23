ALTER TABLE api_keys ADD COLUMN key_id TEXT;
CREATE UNIQUE INDEX idx_api_keys_key_id ON api_keys(key_id);
