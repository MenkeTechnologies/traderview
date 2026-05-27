```
 ████████╗██████╗  █████╗ ██████╗ ███████╗██████╗ ██╗   ██╗██╗███████╗██╗    ██╗
 ╚══██╔══╝██╔══██╗██╔══██╗██╔══██╗██╔════╝██╔══██╗██║   ██║██║██╔════╝██║    ██║
    ██║   ██████╔╝███████║██║  ██║█████╗  ██████╔╝██║   ██║██║█████╗  ██║ █╗ ██║
    ██║   ██╔══██╗██╔══██║██║  ██║██╔══╝  ██╔══██╗╚██╗ ██╔╝██║██╔══╝  ██║███╗██║
    ██║   ██║  ██║██║  ██║██████╔╝███████╗██║  ██║ ╚████╔╝ ██║███████╗╚███╔███╔╝
    ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ ╚══════╝╚═╝  ╚═╝  ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝
```

[![Tauri](https://img.shields.io/badge/tauri-v2-05d9e8.svg)](https://tauri.app)
[![Axum](https://img.shields.io/badge/axum-0.7-ff2a6d.svg)](https://github.com/tokio-rs/axum)
[![Postgres](https://img.shields.io/badge/postgres-embedded%20%2B%20external-d300c5.svg)](https://www.postgresql.org/)
[![Brokers](https://img.shields.io/badge/brokers-12_importers-ff2a6d.svg)](#0x0a-importing-trades)
[![Reports](https://img.shields.io/badge/reports-20+_TraderVue_parity-d300c5.svg)](#0x0c-status)
[![Asset classes](https://img.shields.io/badge/assets-stocks_options_futures_forex-39ff14.svg)](#0x0c-status)
[![Schedule C](https://img.shields.io/badge/schedule_C-business_expenses_%2B_receipts-39ff14.svg)](#0x0aa-business-expenses-schedule-c)
[![Crates](https://img.shields.io/badge/crates-7-39ff14.svg)](#0x03-crate-graph)
[![Docs](https://img.shields.io/badge/docs-online-05d9e8.svg)](https://menketechnologies.github.io/traderview/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

### `[TRADERVUE-STYLE TRADING JOURNAL // FULL FEATURE PARITY // SELF-HOSTED]`

> *"One workspace. Desktop with embedded Postgres. Multi-user web with axum. Same crates, same UI, same FIFO roll-up."*
>
> *"Executions are the atom. Trades are FIFO-derived. The journal is markdown."*

`traderview` is a TraderVue-style trading journal: import broker CSV → atomic execution rows → FIFO trade roll-up → equity curve + summary stats + per-trade / per-day markdown journal. **One Rust workspace ships two binaries** — a Tauri v2 desktop app that downloads and runs an embedded Postgres on first launch (single-user, auto-login), and an axum web server that talks to an external Postgres (multi-user, JWT auth, registration). Both binaries import the same `traderview-{core,db,import}` library crates; the frontend (vanilla JS + uPlot) is shared verbatim between them. By [MenkeTechnologies](https://github.com/MenkeTechnologies).

### [`Read the Docs`](https://menketechnologies.github.io/traderview/) &middot; [`Engineering Report`](https://menketechnologies.github.io/traderview/report.html) · [`Source`](https://github.com/MenkeTechnologies/traderview) · [`strykelang`](https://github.com/MenkeTechnologies/strykelang) · [`zshrs`](https://github.com/MenkeTechnologies/zshrs)

---

## Table of Contents

- [\[0x00\] Overview](#0x00-overview)
- [\[0x01\] Architecture](#0x01-architecture)
- [\[0x02\] Dual Deploy Targets](#0x02-dual-deploy-targets)
- [\[0x03\] Crate Graph](#0x03-crate-graph)
- [\[0x04\] Schema](#0x04-schema)
- [\[0x05\] HTTP API](#0x05-http-api)
- [\[0x06\] Frontend](#0x06-frontend)
- [\[0x07\] Installation](#0x07-installation)
- [\[0x08\] Running — Desktop](#0x08-running--desktop)
- [\[0x09\] Running — Web](#0x09-running--web)
- [\[0x0A\] Importing Trades](#0x0a-importing-trades)
- [\[0x0AA\] Business Expenses (Schedule C)](#0x0aa-business-expenses-schedule-c)
- [\[0x0B\] Configuration](#0x0b-configuration)
- [\[0x0C\] Roadmap](#0x0c-roadmap)
- [\[0xFF\] License](#0xff-license)

---

## [0x00] OVERVIEW

- **One workspace, two binaries** — `traderview-desktop` (Tauri v2 + embedded Postgres) and `server` (axum + external Postgres) both depend on the same `traderview-{core,db,import}` library crates. No code is duplicated between desktop and web.
- **Executions are the atom** — every broker fill is one row in `executions`. Trades are FIFO-derived from those rows and materialized into `trades` for fast UI queries. Re-running the roll-up is always safe.
- **FIFO trade roll-up** — `traderview-core::rollup` matches buy/sell pairs in first-in-first-out order per `(account_id, symbol)`. Open positions stay in `status='open'`; fully-closed positions get `gross_pnl`, `exit_avg`, `closed_at`.
- **Embedded Postgres on the desktop** — `postgresql_embedded` downloads a portable PostgreSQL on first launch (~80 MB, cached in `~/.theseus`), stores data under `$APP_DATA_DIR/traderview/pg/`, and shuts it down cleanly on app exit. Zero external dependencies for desktop users.
- **Multi-user web on the same crates** — the axum binary swaps the embedded pool for an external `DATABASE_URL`, layers in argon2 password hashing + JWT bearer auth, and serves the same vanilla-JS frontend.
- **Vanilla JS + uPlot frontend** — zero npm, zero build step, zero JS framework. Five views (Dashboard, Trades, Journal, Import, Accounts). uPlot draws the equity curve.
- **Webull-first importer** — `traderview-import::webull` parses Webull `Account Statement → Orders` CSV. Dedupes by `(broker_order_id, executed_at, symbol, side, qty, price)` so re-importing the same statement is idempotent.
- **MIT licensed**, single-author, single-language workspace.

---

## [0x01] ARCHITECTURE

```
                    ┌──────────────────────────────────────┐
                    │  frontend/  (vanilla JS + uPlot)     │
                    │  dashboard / trades / journal / ...  │
                    └──────────────┬───────────────────────┘
                                   │ HTTP /api/*
                  ┌────────────────┴────────────────┐
                  │                                 │
        ┌─────────▼──────────────┐     ┌────────────▼─────────────┐
        │  src-tauri             │     │  traderview-web (axum)   │
        │  (traderview-desktop)  │     │  bin: server             │
        │  embedded postgres     │     │  external postgres       │
        │  auto-login local user │     │  argon2 + JWT auth       │
        └─────────┬──────────────┘     └────────────┬─────────────┘
                  │                                 │
                  └────────────────┬────────────────┘
                                   │
                  ┌────────────────▼─────────────────┐
                  │  traderview-{core, db, import}   │
                  │  shared library crates           │
                  └──────────────────────────────────┘
```

The desktop and web binaries are thin shells. All domain logic, all SQL, all broker parsing lives in the three library crates. The decision *"embedded vs external Postgres"* and *"local auto-user vs multi-user with auth"* is the only thing that distinguishes them.

---

## [0x02] DUAL DEPLOY TARGETS

| Mode    | Binary                 | Postgres            | Auth                          | Audience                |
|---------|------------------------|---------------------|-------------------------------|-------------------------|
| Desktop | `traderview-desktop`   | embedded (`theseus`)| auto-login `local` user       | single user, offline    |
| Web     | `server`               | external (`DATABASE_URL`) | argon2 + JWT bearer     | multi-user, hosted      |

Same schema, same migrations, same FIFO roll-up, same frontend, same API surface. Swap the pool + auth layer and the rest of the stack is identical.

---

## [0x03] CRATE GRAPH

| Crate                        | Lines | Purpose                                                                       |
|------------------------------|-------|-------------------------------------------------------------------------------|
| `traderview-core`            | ~1,500 | Domain models (24 types), FIFO roll-up + 6 tests, per-asset P&L, statistics (20+ reports), risk + R-multiple, MFE/MAE excursion, liquidity, slug |
| `traderview-db`              | ~1,800 | 16 modules: accounts, trades, executions, tags, journal, screenshots, imports, mentorships, shares, comments, forum, prices (yfinance fetcher), settings, plans, users, embedded |
| `traderview-import`          | ~1,000 | Generic ColumnMap CSV parser + 12 broker presets (Webull, IBKR Flex, TD, Schwab, TradeStation, Lightspeed, DAS, ThinkOrSwim, E*Trade, Fidelity, TradeZero, Robinhood, Generic) + 5 tests |
| `traderview-expense`         | ~1,200 | Schedule C business-expense parsers (Amazon, BoA, Chase, Apple Card — CSV / XLSX / PDF via `calamine` + `lopdf`), merchant→category rule engine + seed, cross-account transfer dedup |
| `traderview-ocr`             | ~600  | Pure-Rust receipt OCR (PaddleOCR DBNet + SVTR via `tract-onnx`, no C deps) + PDF text-layer extraction + amount/date/merchant regex parsing + Jaccard match scoring |
| `traderview-web`             | ~2,500 | 16 route modules — 50+ HTTP endpoints (auth, accounts, trades, executions, tags, journal, screenshots, imports, 20 reports, mentorships, shares, comments, forum, charts/bars, settings, plans) |
| `src-tauri` (`traderview-desktop`) | ~150  | Tauri v2 shell — spawns embedded Postgres + axum on localhost          |

**Dependency direction** is one-way: `desktop` and `web` both depend on `core + db + import`. Neither depends on the other. `import` and `db` both depend on `core`. Nothing depends on `desktop` or `web`.

---

## [0x04] SCHEMA

Single migration (`migrations/0001_initial.sql`, 149 LOC) defines 9 tables:

| Table          | Purpose                                                                    |
|----------------|----------------------------------------------------------------------------|
| `users`        | Real users (web) or one auto-created `local` user (desktop). Nullable email/hash for desktop. |
| `accounts`     | Broker accounts (`webull`, `ibkr`, `tos`, …). One user → many accounts.    |
| `executions`   | One row per fill — the atom. Unique on `(account_id, broker_order_id, executed_at, symbol, side, qty, price)` when `broker_order_id` is non-null. |
| `trades`       | FIFO-derived from `executions`. Materialized for fast UI queries.          |
| `trade_executions` | Many-to-many: which executions composed each trade.                    |
| `trade_tags`   | Free-form tags per trade.                                                  |
| `journal_entries`  | Per-trade or per-day markdown notes.                                   |
| `imports`      | Audit trail — every CSV upload + the rows it produced.                     |
| `_sqlx_migrations` | sqlx migration tracker.                                                |

Sides are typed enums: `side_t = (buy, sell, short, cover)` for executions; `trade_side_t = (long, short)` and `trade_status_t = (open, closed)` for trades. Money is `NUMERIC(20, 8)` — no floats anywhere in the schema.

---

## [0x05] HTTP API

10 routes, all under `/api/`:

| Method | Path                  | Purpose                                          |
|--------|-----------------------|--------------------------------------------------|
| GET    | `/health`             | Liveness probe                                   |
| GET    | `/config`             | Server config + auth mode (`local` vs `multi`)   |
| POST   | `/auth/register`      | Create user (web mode only)                      |
| POST   | `/auth/login`         | Exchange email/password for JWT bearer token     |
| GET    | `/auth/me`            | Current user from bearer token                   |
| GET    | `/accounts`           | List accounts for current user                   |
| GET    | `/trades`             | List trades (filterable by symbol / status / date) |
| GET    | `/stats/summary`      | Summary stats — wins / losses / expectancy / win rate |
| GET    | `/stats/equity`       | Equity curve points for uPlot                    |
| GET    | `/journal/:day`       | Markdown journal entries for a calendar day      |

Desktop mode auto-logs in as the local user; the frontend talks to the embedded server on a random localhost port via `fetch`. Web mode requires `POST /api/auth/login` → returns JWT → sent as `Authorization: Bearer …` on subsequent calls.

---

## [0x06] FRONTEND

`frontend/` is **vanilla JS + uPlot**. Zero npm, zero bundler, zero framework. Five top-level views, all rendered into `<main id="app">`:

| View      | File             | What it shows                                          |
|-----------|------------------|--------------------------------------------------------|
| Dashboard | `js/app.js`      | Summary stats + equity curve (uPlot)                   |
| Trades    | `js/trades.js`   | Filterable, sortable trade table                       |
| Journal   | `js/journal.js`  | Per-day markdown journal entries                       |
| Import    | `js/import.js`   | Broker CSV upload + dedupe report                      |
| Accounts  | `js/auth.js`     | Accounts list + add/edit                               |

`js/api.js` wraps `fetch` with the JWT header in web mode and a no-op header in desktop mode. `js/charts.js` owns all uPlot setup so chart code lives in one place.

uPlot is vendored under `frontend/lib/` by `./scripts/vendor-uplot.sh` — pinned, reproducible, no CDN at runtime.

---

## [0x07] INSTALLATION

```sh
git clone https://github.com/MenkeTechnologies/traderview
cd traderview

# Vendor uPlot into frontend/lib/ (one-time, reproducible)
./scripts/vendor-uplot.sh
```

Build prerequisites: Rust stable, `pnpm` or any Node for `tauri-cli` if you want the desktop dev loop, Docker (optional) for the web Postgres.

---

## [0x08] RUNNING — DESKTOP

```sh
# First launch downloads PostgreSQL (~80 MB), cached under ~/.theseus
cargo tauri dev
```

The desktop app:
- Downloads + extracts a portable PostgreSQL on first launch (via `postgresql_embedded`).
- Stores cluster data under `$APP_DATA_DIR/traderview/pg/`.
- Auto-creates a single `local` user with `is_local = true` and auto-logs in.
- Starts the axum router on a random localhost port; the WebView talks to it via `fetch`.
- Shuts the embedded Postgres down cleanly on window close.

Release build:

```sh
cargo tauri build
```

---

## [0x09] RUNNING — WEB

```sh
# 1. Bring up Postgres (or point at an existing one)
docker compose up -d postgres

# 2. Configure
export DATABASE_URL=postgres://traderview:traderview@localhost:5432/traderview
export TRADERVIEW_JWT_SECRET=$(openssl rand -hex 32)

# 3. Run
cargo run -p traderview-web --bin server
```

Open <http://localhost:8080>, register (or log in), import a Webull CSV.

`TRADERVIEW_JWT_SECRET` is required in web mode — `server` refuses to start without it. Rotate the secret to invalidate all outstanding tokens.

---

## [0x0A] IMPORTING TRADES

**Webull** — export `Account Statement → Orders` as CSV. Drop the file into the Import view. The parser:

1. Inserts the raw row into `imports` for audit.
2. Maps each row to an `execution` and inserts under the dedupe key `(account_id, broker_order_id, executed_at, symbol, side, qty, price)`.
3. Re-runs the FIFO roll-up for affected `(account_id, symbol)` pairs and updates `trades`.

Re-importing the same CSV is idempotent — the dedupe constraint silently drops duplicate fills. Importing a *new* statement that overlaps an old one is also safe; only the new fills are inserted.

Other brokers (`ibkr`, `tos`, schwab, fidelity) are scheduled — the importer ships a generic mapping-wizard backend once the Webull baseline is solid.

---

## [0x0AA] BUSINESS EXPENSES (SCHEDULE C)

A separate flow from trade ingestion. Tracks business-deductible spending so the year-end Schedule C is one click away.

### What goes in

| Source            | Formats supported  | Account kind   | Notes                                                  |
|-------------------|--------------------|----------------|--------------------------------------------------------|
| Amazon orders     | CSV, XLSX          | marketplace    | Header-less position schema (23 cols). Total at col 7. |
| Bank of America   | CSV, XLSX          | bank           | Two-section export; parser skips the summary block.    |
| Chase             | CSV                | credit_card    | Header-based; respects Chase's signed `Amount`.        |
| Apple Card        | PDF                | credit_card    | Born-digital monthly statement; PDF text layer.        |

Format detection is automatic — drop in whatever the export tool gave you. ZIP-magic bytes route through `calamine` (xlsx/ods/xls), `%PDF` routes through `lopdf`, everything else is treated as CSV. No format flag needed.

### Categorization

Transactions are tagged with one of 23 IRS Schedule C lines (8 through 27a: Advertising, Car & truck, Supplies, Travel, Meals, Utilities, etc.). The merchant→category mapping is a learned rule table — first time you see `STAPLES.COM` you tag it `office`, every future row from that merchant auto-categorizes. Default seed covers ~70 common US merchants (AWS, Adobe, Uber/Lyft, Chevron/Shell, Starbucks, etc.).

The `meals_50` category has a 0.5 deduction percentage baked in so the year-end report applies the IRS 50% rule without you doing math.

### Transfer dedup

When a credit-card payment shows up in **both** your bank statement (money out) and the credit-card statement (money in), the importer detects the pair by amount + date proximity + account kind and marks both as `is_transfer = true`. Schedule C report excludes them.

### Receipts (image / PDF)

Drop a JPG, PNG, WebP, or PDF onto the Expenses tab. OCR runs in the background using pure-Rust PaddleOCR (DBNet + SVTR via `tract-onnx`) — no Tesseract, no system libraries, no C dependencies. PDF receipts use the text-layer fast path via `lopdf`; scanned PDFs prompt you to re-upload as an image.

OCR models are not bundled in the repo (size). On first OCR call, drop PaddleOCR English mobile model files into `$APP_DATA_DIR/traderview/models/paddleocr/` as:

| File             | Source model                                      |
|------------------|---------------------------------------------------|
| `det.onnx`       | DBNet text detection (e.g. `en_PP-OCRv4_det`)     |
| `rec.onnx`       | SVTR text recognition (e.g. `en_PP-OCRv4_rec`)    |
| `line_ori.onnx`  | Text-line orientation classifier (per-line skew)  |
| `doc_ori.onnx`   | Document orientation classifier (page rotation)   |
| `dict.txt`       | Character dictionary for the recognition model    |

The two orientation models matter for phone-camera receipts — PaddleOCR's pipeline corrects line and page rotation natively, so you don't have to hold the camera straight to get a clean OCR.

To compile in the OCR engine itself, build with the `ocr-engine` feature. The flag propagates from the binary down through `traderview-web` to `traderview-ocr`:

```sh
# Web server with OCR enabled
cargo run -p traderview-web --features ocr-engine --bin server

# Desktop app with OCR enabled
cargo tauri dev --features ocr-engine
```

Heavy first-time compile (~5 min cold) because `tract-onnx` + `ndarray` get pulled in — leave the flag off until you actually need receipt OCR. PDF text-layer extraction (born-digital receipts and Apple Card statements) works without the flag.

After OCR, the receipt is matched against your last week of transactions using amount + date + merchant-token Jaccard scoring. You confirm the best match and the receipt is permanently attached to the transaction row.

### Schedule C report

`GET /api/expense/report/schedule_c?year=2026` (or the UI button) returns per-line totals for the calendar year, applying each category's deduction percentage. The report also surfaces:

- uncategorized business expenses that aren't rolled in yet
- excluded transfers + excluded personal rows (transparency on what was filtered out)

---

## [0x0B] CONFIGURATION

| Variable                  | Mode    | Default                                              | Purpose                              |
|---------------------------|---------|------------------------------------------------------|--------------------------------------|
| `DATABASE_URL`            | web     | *(required)*                                         | Postgres connection string           |
| `TRADERVIEW_JWT_SECRET`   | web     | *(required)*                                         | HMAC secret for JWT signing          |
| `TRADERVIEW_BIND`         | web     | `0.0.0.0:8080`                                       | axum listen address                  |
| `TRADERVIEW_CORS_ORIGIN`  | web     | `*`                                                  | CORS allowlist                       |
| `TRADERVIEW_LOG`          | both    | `info`                                               | `tracing-subscriber` env-filter      |
| `$APP_DATA_DIR/traderview/pg/` | desktop | platform default via `dirs`                     | Embedded Postgres data dir           |

The desktop app stores its Postgres cluster under the OS-appropriate app-data directory (`~/Library/Application Support/com.menketechnologies.traderview/pg/` on macOS).

---

## [0x0C] STATUS

| Phase | Item                                                       | Status |
|-------|------------------------------------------------------------|--------|
| 1     | Workspace + crate scaffold + initial schema                | done   |
| 2     | 12 broker importers + generic CSV mapping framework        | done   |
| 3     | FIFO trade roll-up (`traderview-core::rollup`, 6 unit tests) | done |
| 4     | Trades UI — filters, tags, multi-asset, sort, drill-down    | done   |
| 5     | Equity curve + 20 stat reports + drawdown + Sharpe/Sortino  | done   |
| 6     | Journal — markdown, per-trade + per-day, mood               | done   |
| 7     | Multi-asset: stocks / options / futures / forex             | done   |
| 8     | Screenshots — per-trade attachments via multipart           | done   |
| 9     | Mentorship — request / accept / revoke read-only access     | done   |
| 10    | Public trade shares + threaded comments                     | done   |
| 11    | Community forum — 6 seeded categories, threads, posts       | done   |
| 12    | Candlestick chart engine — uPlot custom OHLC + entry/exit marks | done |
| 13    | Price-data fetcher (yfinance) + `price_bars` cache          | done   |
| 14    | MFE / MAE / exit-efficiency from price bars                 | done   |
| 15    | R-multiple risk reports + per-trade stop/risk inputs        | done   |
| 16    | Trade plans (pre-trade) + saved filter sets                 | done   |
| 17    | GitHub Actions CI + release matrix + Homebrew tap formula    | done   |
| —     | Cloud sync — encrypted snapshot to S3 / R2                   | future |

---

## [0xFF] LICENSE

MIT License — Jacob Menke ([MenkeTechnologies](https://github.com/MenkeTechnologies)). See [LICENSE](LICENSE).
