DROP INDEX IF EXISTS idx_api_keys_key_id;
ALTER TABLE api_keys DROP COLUMN key_id;
