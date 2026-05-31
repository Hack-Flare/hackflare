CREATE TABLE IF NOT EXISTS dns_query_logs (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
    query_name TEXT NOT NULL,
    query_type TEXT NOT NULL,
    response_code TEXT NOT NULL,
    source_ip TEXT NOT NULL,
    protocol TEXT NOT NULL,
    response_size INTEGER NOT NULL DEFAULT 0,
    processing_us INTEGER NOT NULL DEFAULT 0,
    answers_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_dns_query_logs_timestamp ON dns_query_logs (timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_dns_query_logs_response_code ON dns_query_logs (response_code);
