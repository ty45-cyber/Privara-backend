# Privara — Confidential Infrastructure Layer

> AES-256-GCM off-chain encryption + on-chain FHE payroll and coercion-resistant
> governance for DAOs and fintech. Built on Fhenix CoFHE.

[![Fhenix CoFHE](https://img.shields.io/badge/Fhenix-CoFHE-00e5c8)](https://fhenix.io)
[![Network](https://img.shields.io/badge/Network-Arbitrum%20Sepolia-7b61ff)](https://sepolia.arbiscan.io)
[![Backend](https://img.shields.io/badge/Backend-Rust%20%2B%20Axum-orange)](https://axum.rs)
[![Frontend](https://img.shields.io/badge/Frontend-Vite%20%2B%20React-61dafb)](https://vitejs.dev)

## Live Demo
🌐 **Frontend:** https://privara-frontend-smxb.vercel.app/
🔗 **Payroll Contract:** https://sepolia.arbiscan.io/address/0x...
🔗 **Governance Contract:** https://sepolia.arbiscan.io/address/0x...
🎥 **Demo Video:** https://loom.com/...

---

## The Problem

Today's financial, governance, and AI systems expose sensitive operational data
by default. DAOs leak payroll. Voters face coercion. AI inference requires raw
data exposure. Existing tools force a choice between privacy, compliance,
transparency, and scalability. Privara solves all four simultaneously.
┌─────────────────────────────────────────────────────────────┐
│                         USER BROWSER                        │
│                                                             │
│  cofhejs.encrypt(Encryptable.uint128(amount))               │
│  cofhejs.encrypt(Encryptable.uint8(vote))                   │
│  cofhejs.createPermit() → getPermission()                   │
└──────────────────┬──────────────────────────────────────────┘
│ encrypted blob (never plaintext on wire)
▼
┌─────────────────────────────────────────────────────────────┐
│              FHENIX CoFHE — Arbitrum Sepolia                │
│                                                             │
│  PrivaraPayroll.sol                                         │
│    euint128 amountEnc   ← FHE.asEuint128(InEuint128)        │
│    FHE.allow(stored, payee) — permit-gated sealed output    │
│                                                             │
│  PrivaraGovernance.sol                                      │
│    euint8 yesCount += FHE.asEuint8(eq(vote, 1))             │
│    Homomorphic accumulation — no individual vote decrypted  │
└──────────────────┬──────────────────────────────────────────┘
│ tx hash
▼
┌─────────────────────────────────────────────────────────────┐
│              RUST BACKEND — Axum + SQLx + MySQL             │
│                                                             │
│  AES-256-GCM field encryption (domain-separated keys)       │
│  SHA-256 ZK commitments on payroll amounts                  │
│  Fhenix RPC client — tx verification + network status       │
│  JWT auth (org-scoped claims, 12hr expiry)                  │
│  bcrypt password hashing                                    │
│  MySQL audit trail (onchain_txs table)                      │
└─────────────────────────────────────────────────────────────┘
---

## Tech Stack

| Layer | Technology |
|---|---|
| Backend | Rust + Axum + SQLx + MySQL |
| Contracts | Solidity 0.8.26 + @fhenixprotocol/cofhe-contracts |
| Frontend | Vite + React + cofhejs/web + viem |
| FHE Network | Fhenix CoFHE on Arbitrum Sepolia |
| Off-chain crypto | AES-256-GCM, SHA-256, bcrypt, JWT |
| Testing | Hardhat + cofhe-hardhat-plugin |

---

## Fhenix CoFHE Integration

### What's Encrypted On-Chain

| Contract | Field | FHE Type | Operation |
|---|---|---|---|
| PrivaraPayroll | amount | euint128 | Stored encrypted, sealed per-payee |
| PrivaraGovernance | vote | euint8 | Homomorphic addition (tally) |
| PrivaraGovernance | tally | euint8 × 3 | FHE.sealoutput after deadline |

### SDK Used
`cofhejs/web` — Fhenix's current maintained SDK.
`cofhejs.encrypt(Encryptable.uint128(n))` — encrypts amounts before tx.
`cofhejs.encrypt(Encryptable.uint8(n))` — encrypts votes before tx.
`cofhejs.createPermit()` — issues a sealing permit for output decryption.

---

## Quick Start

### 1. Contracts
```bash
cd contracts
cp .env.example .env       # add DEPLOYER_PRIVATE_KEY
npm install
npm test                   # run with mock CoFHE
npm run deploy:arb-sepolia # deploy to testnet
```

### 2. Backend
```bash
cp .env.example .env       # fill DATABASE_URL, JWT_SECRET, PRIVARA_MASTER_KEY
cargo run
```

### 3. Frontend
```bash
cd frontend
cp .env.example .env       # fill VITE_PAYROLL_ADDRESS, VITE_GOVERNANCE_ADDRESS
npm install
npm run dev
```

---

## Wave Roadmap

| Wave | Deliverable | Status |
|---|---|---|
| Wave 1 | FHE Payroll + Private Governance + Rust backend + React dashboard | ✅ Complete |
| Wave 2 | Privacy-Preserving AI — encrypted inference on sensitive datasets | 🔜 Building |
| Wave 3 | Encrypted Gaming — hidden player actions + provably fair FHE randomness | 🔜 Planned |
| Wave 4 | RWA Compliance Layer — selective identity disclosure + encrypted audit workflows | 🔜 Planned |

---

## Security Model

- All off-chain sensitive fields: AES-256-GCM with domain-separated key derivation
- Voter identity: SHA-256 ZK commitment — never stored raw
- On-chain amounts: FHE-encrypted `euint128` — never visible, not even to validators
- Individual votes: never decrypted — homomorphic accumulation only
- Access control: `FHE.allow(handle, address)` — permit-gated sealed outputs
- Auth: bcrypt (cost 12) + JWT (HMAC-SHA256, 12hr, org-scoped)
- OWASP Top 10 aligned throughout

---

## API Reference

| Method | Path | Auth | Description |
|---|---|---|---|
| POST | /api/auth/register | No | Create org + admin |
| POST | /api/auth/login | No | Login, get JWT |
| GET | /api/payroll | Yes | List encrypted payroll |
| POST | /api/payroll | Yes | Create encrypted entry |
| POST | /api/payroll/:id/audit | Yes (admin) | Generate audit token |
| GET | /api/governance/proposals | Yes | List proposals |
| POST | /api/governance/proposals | Yes | Create encrypted proposal |
| POST | /api/governance/vote | Yes | Cast encrypted vote |
| GET | /api/blockchain/network | No | Fhenix network status |
| POST | /api/blockchain/verify-tx | Yes | Verify on-chain tx |
| GET | /api/blockchain/balance | Yes | Wallet tFHE balance |


---

## Architecture
