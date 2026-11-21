CREATE TABLE IF NOT EXISTS alert_state (
    tenant_id INTEGER NOT NULL DEFAULT 1,
    template_hash TEXT NOT NULL,
    last_sent_at TEXT NOT NULL,
    PRIMARY KEY (tenant_id, template_hash)
);
