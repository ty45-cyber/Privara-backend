CREATE TABLE organizations (
    id          CHAR(36) PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE users (
    id            CHAR(36) PRIMARY KEY,
    org_id        CHAR(36) NOT NULL,
    email         VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role          ENUM('admin','member','auditor') NOT NULL DEFAULT 'member',
    created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (org_id) REFERENCES organizations(id) ON DELETE CASCADE
);

CREATE TABLE payroll_entries (
    id                   CHAR(36) PRIMARY KEY,
    org_id               CHAR(36) NOT NULL,
    created_by           CHAR(36) NOT NULL,
    contractor_name_enc  TEXT NOT NULL,
    amount_enc           TEXT NOT NULL,
    currency             VARCHAR(10) NOT NULL,
    wallet_address_enc   TEXT NOT NULL,
    zk_commitment        VARCHAR(64) NOT NULL,
    status               ENUM('pending','processed','audited') NOT NULL DEFAULT 'pending',
    audit_token          CHAR(36) NULL,
    created_at           DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (org_id) REFERENCES organizations(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE proposals (
    id              CHAR(36) PRIMARY KEY,
    org_id          CHAR(36) NOT NULL,
    creator_id      CHAR(36) NOT NULL,
    title_enc       TEXT NOT NULL,
    description_enc TEXT NOT NULL,
    quorum          INT NOT NULL DEFAULT 3,
    status          ENUM('active','passed','rejected','expired') NOT NULL DEFAULT 'active',
    ends_at         DATETIME NOT NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (org_id) REFERENCES organizations(id) ON DELETE CASCADE,
    FOREIGN KEY (creator_id) REFERENCES users(id)
);

CREATE TABLE votes (
    id               CHAR(36) PRIMARY KEY,
    proposal_id      CHAR(36) NOT NULL,
    voter_commitment VARCHAR(64) NOT NULL,
    vote_enc         TEXT NOT NULL,
    created_at       DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (proposal_id) REFERENCES proposals(id) ON DELETE CASCADE
);