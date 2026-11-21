CREATE TABLE IF NOT EXISTS severity_cache (
    tenant_id INTEGER NOT NULL DEFAULT 1,
    template_hash TEXT NOT NULL,
    severity TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (tenant_id, template_hash)
);
