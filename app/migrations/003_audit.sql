CREATE TABLE audit_reports (
    id               CHAR(36) PRIMARY KEY,
    org_id           CHAR(36) NOT NULL,
    entry_count      BIGINT NOT NULL DEFAULT 0,
    merkle_root      CHAR(64) NOT NULL,
    report_signature CHAR(64) NOT NULL,
    generated_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (org_id) REFERENCES organizations(id) ON DELETE CASCADE
);

-- Index for fast report lookup per org
CREATE INDEX idx_audit_reports_org ON audit_reports(org_id, generated_at DESC);