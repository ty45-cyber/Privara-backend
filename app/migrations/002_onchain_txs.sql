CREATE TABLE onchain_txs (
    tx_hash      VARCHAR(66) PRIMARY KEY,
    status       ENUM('pending','success','reverted') NOT NULL DEFAULT 'pending',
    explorer_url VARCHAR(512) NOT NULL,
    submitted_by VARCHAR(255) NOT NULL,
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);