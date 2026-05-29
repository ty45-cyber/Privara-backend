CREATE TABLE payroll_intents (
    id               CHAR(36) PRIMARY KEY,
    org_id           CHAR(36) NOT NULL,
    payee_address    VARCHAR(42) NOT NULL,
    currency         VARCHAR(10) NOT NULL,
    on_chain_id      BIGINT UNSIGNED NULL,     -- ID from PrivaraIntentQueue.submitIntent()
    status           ENUM('pending','processing','settled','cancelled','expired')
                     NOT NULL DEFAULT 'pending',
    batch_id         CHAR(36) NULL,
    settlement_tx    VARCHAR(66) NULL,
    process_deadline DATETIME NOT NULL,
    created_at       DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (org_id) REFERENCES organizations(id) ON DELETE CASCADE,
    INDEX idx_intents_org_status (org_id, status),
    INDEX idx_intents_deadline   (process_deadline)
);

CREATE TABLE intent_batches (
    id           CHAR(36) PRIMARY KEY,
    tx_hash      VARCHAR(66) NOT NULL,
    processed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);