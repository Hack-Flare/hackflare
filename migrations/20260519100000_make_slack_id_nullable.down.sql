UPDATE users SET slack_id = '' WHERE slack_id IS NULL;
ALTER TABLE users ALTER COLUMN slack_id SET NOT NULL;
ALTER TABLE users ADD UNIQUE (slack_id);
