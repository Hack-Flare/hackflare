-- Migrate existing user IDs from HCA-provided format to hf!<uuid> format.
--
-- This drops FK constraints, updates all IDs, and re-adds constraints so that
-- existing users, sessions, API keys, and DNS zone ownership all remain intact
-- with the new ID format.

ALTER TABLE user_sessions DROP CONSTRAINT user_sessions_user_id_fkey;
ALTER TABLE api_keys DROP CONSTRAINT api_keys_user_id_fkey;

DO $$
DECLARE
    user_rec RECORD;
    new_id TEXT;
BEGIN
    FOR user_rec IN SELECT id FROM users LOOP
        new_id := 'hf!' || gen_random_uuid()::TEXT;

        UPDATE user_sessions SET user_id = new_id WHERE user_id = user_rec.id;
        UPDATE api_keys SET user_id = new_id WHERE user_id = user_rec.id;
        UPDATE dns_zones SET user_id = new_id WHERE user_id = user_rec.id;

        UPDATE users SET id = new_id WHERE id = user_rec.id;
    END LOOP;
END $$;

ALTER TABLE user_sessions
    ADD CONSTRAINT user_sessions_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE api_keys
    ADD CONSTRAINT api_keys_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
