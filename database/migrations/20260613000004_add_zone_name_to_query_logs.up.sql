ALTER TABLE dns_query_logs ADD COLUMN zone_name TEXT NOT NULL DEFAULT '';
CREATE INDEX idx_dns_query_logs_zone_name ON dns_query_logs (zone_name);
