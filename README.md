<div align="center">

# PRIVARA LITE
### Confidential Financial Operations Hub

[![SoSoValue](https://img.shields.io/badge/Market%20Data-SoSoValue%20API-6C47FF?style=flat-square)](https://sosovalue.com)
[![Rust](https://img.shields.io/badge/Backend-Rust%20%2B%20Axum-CE422B?style=flat-square)](https://axum.rs)
[![React](https://img.shields.io/badge/Frontend-Vite%20%2B%20React-61DAFB?style=flat-square)](https://vitejs.dev)
[![AES-256-GCM](https://img.shields.io/badge/Encryption-AES--256--GCM-00C896?style=flat-square)](#)
[![Akindo](https://img.shields.io/badge/Buildathon-SoSoValue%20×%20Akindo-E8A020?style=flat-square)](https://app.akindo.io)

> *The only financial operations platform where treasury decisions are gated
> by live institutional market intelligence — and every sensitive field
> is encrypted until the right person unlocks it.*

**[Live Demo](https://privara-lite.vercel.app)** · **[GitHub](https://github.com/[YOUR_HANDLE]/privara-lite)** · **[Demo Video](#)**

</div>

---

## The Problem

Organizations handle payroll in spreadsheets. Treasury approvals happen over
email. Governance votes are tallied by the same person who wrote the proposal.
And every financial decision is made in complete ignorance of live market
conditions.

A CFO approves a $500K deployment during peak ETF outflows and bearish
institutional sentiment — without knowing it.

## The Solution

Privara Lite is a **four-module confidential operations system** that encrypts
sensitive financial data, enforces multi-party approval workflows, and gates
every treasury decision with live institutional intelligence from SoSoValue.

---

## SoSoValue Integration — The Core

This is not a cosmetic integration. SoSoValue powers every treasury decision.

### Endpoint 1 — BTC Spot ETF Metrics
POST https://api.sosovalue.xyz/openapi/v2/etf/currentEtfDataMetrics
Header: x-soso-api-key: <key>
Body:   {"type": "us-btc-spot"}
Returns `dailyNetInflow`, `totalNetAssets`, `dailyTotalValueTraded`,
`cumNetInflow`. Used to compute flow direction and volatility proxy.

### Endpoint 2 — AI News Sentiment Feed
GET https://openapi.sosovalue.com/api/v1/news/featured/currency
?pageNum=1&pageSize=30&categoryList=1,2,5,6
Header: x-soso-api-key: <key>
Returns AI-tagged news items. Used for single-pass O(n) sentiment
scoring via bullish/bearish tag frequency analysis.

### Endpoint 3 — Coin List (BTC Price)
POST https://openapi.sosovalue.com/openapi/v1/data/default/coin/list
Header: x-soso-api-key: <key>
Body:   {"pageNum":1,"pageSize":20,"sortField":"marketCap","sortOrder":"desc"}
BTC real-time price with 24h change. First tier of three-layer
price fallback (SoSoValue → CoinGecko → Binance).

### Composite Risk Formula — O(1)
sentiment_penalty = (100 - sentiment_score) / 3
flow_penalty      = f(dailyNetInflow)   // -15 to +35
btc_penalty       = f(btc_24h_change)   // -5 to +15
size_penalty      = f(amount)           // 0 to +20
composite = sentiment_penalty + flow_penalty + btc_penalty + size_penalty
composite ≥ 40  →  HIGH    "Delay 48h"
composite ≥ 20  →  MEDIUM  "Proceed with full quorum"
composite < 20  →  LOW     "Conditions favorable"

### Why This Wins
Every SoSoValue call is visible in the browser network tab during judging.
The Market Intelligence widget on the dashboard shows live ETF flows, BTC
price, sentiment score, trending news tags, and SoDEX TVL — all attributed
to SoSoValue, all updating in real time via WebSocket.

---

## Four Modules

### 1. Encrypted Payroll
- CSV upload → AES-256-GCM encryption (ring crate, hardware-accelerated)
- Role-gated reveal: HR/Finance decrypt → employees see `██████████`
- Auditors get CSV export; every reveal logged immutably
- KES + USD multi-currency support

### 2. Treasury Approvals
- Multi-party threshold approval (configurable 1–5 approvers)
- SoSoValue risk score on every request before any approver acts
- Claude Sonnet AI briefing: structured analysis grounded in live market data
- SoDEX testnet on-chain execution with tx hash and explorer link
- Automatic status progression: pending → approved → executed

### 3. Private Governance
- AES-256-GCM encrypted ballots — votes stored as ciphertext
- One vote per proposal enforced at the data layer
- Admin-only tally reveal with animated YES/NO/ABSTAIN bars
- Active proposals with live countdown to voting deadline

### 4. Selective Audit Disclosure
- Auditors request access to specific resources
- Admins grant or deny at resource level
- Every decision written to an immutable, hash-referenced audit log
- One-click signed PDF audit report download

---

## Architecture
┌─────────────────────────────────────────────────┐
│            Vite + React Frontend                 │
│  Landing · Dashboard · Payroll · Treasury        │
│  Governance · Audit                              │
│  MarketIntel widget (SoSoValue — live WS feed)   │
│  DemoBar (role switcher — no login required)     │
└─────────────────┬───────────────────────────────┘
│ REST/JSON + WebSocket
┌─────────────────▼───────────────────────────────┐
│          Rust + Axum Gateway                     │
│  JWT Auth Middleware (role-based)                │
├──────────┬──────────┬──────────┬────────────────┤
│ Payroll  │ Treasury │ Gov      │ Audit + Reports │
│ Service  │ Service  │ Service  │ Service         │
├──────────┴──────────┴──────────┴────────────────┤
│       AES-256-GCM Encryption (ring crate)        │
├─────────────────────────────────────────────────┤
│              MySQL + sqlx                        │
│       (compile-time verified queries)            │
└─────────────────┬───────────────────────────────┘
│
┌───────────┴────────────┐
│                        │
┌─────▼──────────┐   ┌─────────▼──────────┐
│  SoSoValue API │   │   SoDEX API         │
│  ETF Metrics   │   │   Liquidity + Exec  │
│  News Feed     │   │   Testnet Tx        │
│  Coin List     │   └────────────────────┘
└────────────────┘
│
┌─────▼──────────────┐
│  Fallback Chain     │
│  CoinGecko → Binance│
└────────────────────┘

---

## Tech Stack

| Layer | Technology | Why |
|---|---|---|
| Backend | Rust + Axum | O(1) routing, zero-cost abstractions, hardware AES |
| Database | MySQL + sqlx | Compile-time query verification |
| Encryption | AES-256-GCM (ring) | Hardware-accelerated, NIST-approved |
| Auth | JWT (jsonwebtoken) | Stateless, O(1) verify, demo flag in payload |
| Frontend | Vite + React | Fast HMR, code splitting |
| Market Data | SoSoValue API | Primary — ETF + news + coin |
| Fallback | CoinGecko + Binance | Automatic tier-2 and tier-3 failover |
| DeFi Layer | SoDEX | TVL signal + testnet execution |
| AI Briefing | Claude Sonnet | Grounded treasury analysis |
| Realtime | WebSocket | 30s market feed refresh |
| Deploy | Docker Compose | One-command stack |

---

## Quick Start

### Prerequisites
- Rust 1.78+  ·  MySQL 8.0+  ·  Node.js 20+
- SoSoValue API key — [sosovalue.com/developer](https://sosovalue.com/developer)
- Anthropic API key — [console.anthropic.com](https://console.anthropic.com)

### Docker (recommended — one command)
```bash
git clone https://github.com/[YOUR_HANDLE]/privara-lite
cd privara-lite
cp .env.example .env
# Fill in API keys
docker compose up --build
```

Frontend → http://localhost:5173
Backend  → http://localhost:8080

### Manual
```bash
# Backend
cd backend && cargo run --release

# Frontend
cd frontend && npm install && npm run dev
```

---

## Demo Credentials

All accounts use password: **`Privara2026!`**

| Role | Email | Can See |
|---|---|---|
| **Admin** | admin@privara.io | Everything + tally votes + decide audits |
| **HR** | hr@privara.io | Upload + decrypt payroll |
| **Finance** | finance@privara.io | Treasury create + approve + risk score |
| **Auditor** | auditor@privara.io | Request access + download PDF reports |
| **Employee** | employee@privara.io | Masked payroll (██████████) |

### Instant Demo (no password)
The landing page has one-click role buttons — judges explore without creating
an account. Or:
POST /demo/login  { "role": "finance" }
→ returns a 2-hour demo JWT instantly

---

## API Reference
Auth
POST   /auth/register
POST   /auth/login
POST   /demo/login                              ← instant role access
Market Intelligence (SoSoValue powered)
GET    /market/intelligence                     ← ETF + news + price + SoDEX
GET    /market/etf                              ← dashboard widget feed
POST   /market/briefing                         ← Claude AI treasury briefing
WS     /ws/market                               ← 30s live push feed
Payroll
POST   /payroll/upload                          ← CSV → AES-256-GCM
GET    /payroll/batches
GET    /payroll/batches/:id                     ← role-aware: decrypt or mask
GET    /payroll/batches/:id/audit-export        ← CSV download (admin/auditor)
Treasury
POST   /treasury/requests
GET    /treasury/requests
GET    /treasury/requests/pending
POST   /treasury/requests/:id/approve
POST   /treasury/requests/:id/reject
GET    /treasury/requests/:id/risk-score        ← SoSoValue composite score
POST   /treasury/requests/:id/execute           ← SoDEX testnet execution
Governance
POST   /governance/proposals
GET    /governance/proposals
POST   /governance/proposals/:id/vote           ← AES-encrypted ballot
GET    /governance/proposals/:id/results        ← admin-only tally
Audit
POST   /audit/requests
GET    /audit/requests
POST   /audit/requests/:id/decide
GET    /audit/logs
GET    /audit/reports/:type/:id/pdf             ← signed PDF download
GET    /audit/reports/full/pdf                  ← full system audit

---

## Environment Variables

```env
DATABASE_URL=mysql://root:password@localhost:3306/privara_lite
JWT_SECRET=<32+ char secret>
ENCRYPTION_KEY=<64-char hex = 32 bytes AES-256>

# SoSoValue — primary data layer
SOSOVALUE_API_KEY=<from sosovalue.com/developer>
SOSOVALUE_API_URL=https://api.sosovalue.xyz
SOSOVALUE_NEWS_URL=https://openapi.sosovalue.com

# SoDEX — DeFi liquidity + execution
SODEX_API_URL=https://api.sodex.io

# Claude AI — treasury briefings
ANTHROPIC_API_KEY=<from console.anthropic.com>
```

---

## Wave History

### Wave 1 — AlphaOS (SoSoValue × Akindo)
Bloomberg terminal aesthetic for crypto market intelligence.
LangGraph agent platform with 64 files, Solana integration,
and SoSoValue as the primary data source for token analysis.

### Wave 2 — Privara Lite (SoSoValue × Akindo) ← Current
Confidential financial operations for organizations.
SoSoValue ETF flows + news sentiment gate real treasury decisions.
AES-256-GCM payroll encryption. Private governance voting.
Immutable audit trail. Claude Sonnet briefing layer.
SoDEX on-chain execution with testnet tx verification.

---

## License

MIT — Built for SoSoValue × Akindo Buildathon Wave 2

---

<div align="center">

**Meridian Capital Partners runs Privara Lite.**
*47 employees. 4 months of payroll. $1.2M in treasury movements.
3 governance votes. 2 KPMG audit requests. All encrypted. All auditable.*

[Live Demo →]https://privara-lite-frontend-1dvk.vercel.app/

</div>
