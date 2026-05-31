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
- [\[0x0AB\] Landlord / Rental Property (Schedule E)](#0x0ab-landlord--rental-property-schedule-e)
- [\[0x0B\] Configuration](#0x0b-configuration)
- [\[0x0C\] Status](#0x0c-status)
- [\[0xFF\] License](#0xff-license)

---

## [0x00] OVERVIEW

- **Full desktop trading suite** — replaces TraderVue ($30/mo journal) + DayTradeDash ($187/mo Warrior Trading scanner) + StockInvest.us in one self-hosted binary. **$2,604/yr saved**, data stays on your machine.
- **One workspace, two binaries** — `traderview-desktop` (Tauri v2 + embedded Postgres) and `server` (axum + external Postgres) both depend on six shared library crates. No code is duplicated between desktop and web.
- **Executions are the atom** — every broker fill is one row in `executions`. Trades are FIFO-derived from those rows and materialized into `trades` for fast UI queries. Re-running the roll-up is always safe.
- **FIFO trade roll-up** — `traderview-core::rollup` matches buy/sell pairs in first-in-first-out order per `(account_id, symbol)`. Open positions stay in `status='open'`; fully-closed positions get `gross_pnl`, `exit_avg`, `closed_at`.
- **Embedded Postgres on the desktop** — `postgresql_embedded` downloads a portable PostgreSQL on first launch (~80 MB, cached in `~/.theseus`), stores data under `$APP_DATA_DIR/traderview/pg/`, and shuts it down cleanly on app exit. Stale-PID lockfile cleanup survives a hard-killed parent. Zero external dependencies for desktop users.
- **Multi-user web on the same crates** — the axum binary swaps the embedded pool for an external `DATABASE_URL`, layers in argon2 password hashing + JWT bearer auth, and serves the same vanilla-JS frontend.
- **Tile launcher (Cmd-K)** — categorized tile grid with live filter replaces the old 77-tab strip. Press `?` anywhere for the in-app tutorial. Topbar carries the most-used routes.
- **Right-click context menus everywhere** — every view registers a `data-context-scope` slug that the global ctxmenu handler resolves on right-click. Per-row scopes (`trade-row`, `symbol-row`, `journal-entry`, `tag-chip`, `webhook-row`, `api-token-row`, `account-row`, `plan-row`, `share-row`, `board-row`, `dashboard-sidebar-item`, `hotkey-row`, `custom-indicator-row`, `alert-rule-row`, `strategy-alert-row`, `watchlist-symbol-row`, `position-row`, `backtest-preset-row` — 18 scopes) plus 37 symbol-aware view scopes give 222 distinct symbol-nav paths to the active symbol (Charts / Options / Research / Earnings / News / Copy) without leaving the current view. Items are registered in a single `ALL_SCOPED_ITEMS` array; handler bodies are 3-4 LoC each via 6 shared helpers (`clipboardWrite` / `refreshView` / `dataFromTarget` / `toastErr` / `toastOk` / `symbolFromTarget`). Two regression tests pin (a) every emitting tag carries the data-* attrs its handler reads and (b) every registered scope is documented in the audit's required-attrs map.
- **48-shortcut keyboard surface** — `_shortcuts.js` registry: 8 nav globals (`Cmd+Opt+T/J/D/W/C/L/R/M` for Trades/Journal/Dashboard/Watchlists/Charts/Live/Reports/Scanner) + 11 view-scoped binds (`n` for new-trade in trades scope, `r` to refresh in dashboard/live/trades/journal/watchlists/webull/charts, etc.) + the pre-existing global set. Every view-scoped bind has a visible `⟳ Refresh` / `+ New` button stamped with `data-shortcut`, so the tooltip augmenter appends `(⌘⌥T)` / `(R)` chips on hover. The cheat-sheet view (`?` or Cmd+K → "Keyboard Shortcuts") lists all 48 globals + the 70+ scoped ctxmenu items in three searchable tables.
- **Non-blocking dialog primitive** — `tConfirm()` / `tPrompt()` (in `dialog.js`) replace every `alert()` / `confirm()` / `prompt()` call site (63 + 37 = 100 swept across 42 view files). Returns a Promise; themed by level (`info` / `warning` / `danger`), Enter/Esc keybound, required-empty input shake, i18n labels via `dialog.btn.*` keys.
- **Vanilla JS + uPlot frontend** — zero npm, zero bundler, zero framework. 257 view modules + pure helper modules + runtime modules across ~134k LOC JS + 2,512 LOC CSS. Per-view race-token machinery prevents post-await DOM crashes; window.onerror + console.error funnel to a Rust-side `/api/client-errors` sink.
- **Live data streams** — Nasdaq halts (3s RSS, TTS voice alerts), SEC EDGAR + 4 PR wires (catalyst radar with ticker NER), Finnhub WebSocket 6-panel intraday scanner, Webull read-only broker (paste session tokens, in-memory only), 16-symbol world markets snapshot (60s in-process cache). All live stores are bounded with oldest-first eviction.
- **12 broker importers + Generic CSV wizard** — Webull, Lightspeed, IBKR Flex, ThinkOrSwim, TD Ameritrade, Schwab, Fidelity, ETrade, Robinhood, TradeStation, DAS Trader, TradeZero, plus a column-mapping Generic parser for anything else.
- **17 reports + R-multiple + Monte Carlo forecast + fill-quality TCA + tax-lot tracker** with Schedule-D export.
- **stryke-JIT backtest engine + walk-forward sweeper + custom-indicator AST + strategy alerts (AND/OR/NOT) + webhooks** (Discord / Slack / generic).
- **Expense tracker with receipt OCR + Schedule-C report** — `traderview-expense` parses Amazon / BoA / Chase / Apple Card statements; `traderview-ocr` extracts merchant / amount / date from receipt images.
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

| Crate                        | Lines  | Purpose                                                                       |
|------------------------------|--------|-------------------------------------------------------------------------------|
| `traderview-core`            | 139,357 | Domain types, FIFO roll-up + tests, per-asset P&L, statistics (R-multiple / SQN / Sharpe / Sortino / expectancy), Kelly + correlation-aware position sizing, Monte Carlo equity forecaster, stryke-JIT backtest engine + walk-forward sweeper, sentiment scoring, custom-indicator AST. |
| `traderview-db`              | 18,966 | ~50 repo modules — trades / executions / tags / journal / screenshots / imports / mentorships / shares / forum / settings / plans / users / watchlists / alerts / hotkeys / paper / disclosures / catalysts / halts / live_ticks / markets / premarket / earnings / news / strategy alerts / rebalance / goals / reviews / custom indicators. sqlx pool + 31 migrations + embedded PG lifecycle (stale-PID cleanup, persisted password). Background pollers for Yahoo / FINRA / EDGAR / Nasdaq RSS / Finnhub WS / Reddit WSB / StockTwits / CoinGecko. Bounded in-memory stores. |
| `traderview-import`          | 1,860  | Generic ColumnMap CSV parser + 12 broker presets — Webull, Lightspeed, IBKR Flex, ThinkOrSwim, TD Ameritrade, Schwab, Fidelity, ETrade, Robinhood, TradeStation, DAS Trader, TradeZero. |
| `traderview-expense`         | 6,527  | Schedule C business-expense parsers (Amazon, BoA, Chase, Apple Card — CSV / XLSX / PDF via `calamine` + `lopdf`), merchant→category rule engine + seed, cross-account transfer dedup. |
| `traderview-ocr`             | 814    | Receipt OCR via the system `tesseract` binary + image preprocessing (binarize, deskew), PDF text-layer extraction + amount/date/merchant regex parsing + Jaccard match scoring. |
| `traderview-web`             | 25,101 | axum 0.7 router — **~1,000 routes** across ~83 route files (auth, accounts, trades, executions, tags, journal + AI, screenshots, imports + CSV wizard, 17 reports, mentorships, shares, comments, forum, charts/bars, settings, plans, hotkeys, watchlists, alerts + strategy alerts, paper, options, vol, breadth, fear-greed, sector rotation, sentiment, disclosures, catalysts WS, halts WS, live-ticks WS, webull WS, premarket, markets, news, earnings, custom indicators, backtest + walk-forward, rebalance, goals, reviews, expenses + receipts + Schedule C, dashboards, API tokens, webhooks, client-errors sink). Custom logging middleware (`log_mw`) records every request with elapsed_ms; 4xx/5xx attaches a 4KB body snippet. |
| `src-tauri` (`traderview-desktop`) | 370    | Tauri v2 shell — spawns embedded Postgres + axum on localhost. Worker-thread bring-up, native-dialog on failure, `tracing-appender` non-blocking file log + panic hook, `Embedded` held across `axum::serve` so Postgres can't be dropped mid-request. |

**Dependency direction** is one-way: `desktop` depends on `db + web`. `web` depends on `core + db + import + expense + ocr`. `import`, `expense`, `db` all depend on `core`. Nothing depends on `desktop`.

---

## [0x04] SCHEMA

31 migrations from `0001_initial.sql` through `0031_risk_fires.sql` define **69 tables, 91 indexes, 17 PostgreSQL enum types**. Each migration adds a self-contained feature; the schema grows by feature, not by big-bang. Money is `NUMERIC(20, 8)` everywhere — no floats. Grouped by domain:

| Domain                     | Tables                                                                 |
|----------------------------|------------------------------------------------------------------------|
| Identity & accounts        | `users`, `accounts`, `api_tokens`, `mentorships`                       |
| Executions & trades        | `executions`, `trades`, `trade_executions`, `trade_tags`, `tags`, `imports` |
| Journal                    | `journal_entries`, `note_templates`, `trade_reviews`, `chart_drawings`, `screenshots` |
| Plans / goals / discipline | `plans`, `trading_goals`, `goal_progress`, `discipline_violations`     |
| Price data & quotes        | `bars`, `quote_snapshots`, `news_items`, `earnings_events`, `dividends` |
| Live feeds                 | `halts`, `catalysts`, `mentions` (sentiment), `tick_snapshots`         |
| Watchlists & screening     | `watchlists`, `watchlist_symbols`, `filter_sets`                       |
| Alerts / webhooks / hotkeys | `alerts`, `strategy_alerts`, `strategy_alert_fires`, `hotkeys`, `webhooks`, `webhook_deliveries`, `disclosures_watchers` |
| Backtest & strategy        | `backtest_runs`, `backtest_presets`, `walk_forward_runs`, `custom_indicators` |
| Paper trading              | `paper_accounts`, `paper_orders`, `paper_positions`                    |
| Portfolio / risk           | `rebalance_targets`, `rebalance_runs`, `tax_lots`                      |
| Disclosures                | `disclosures` (Form 4, 13D/G, Senate / House STOCK Act)                |
| Institutional 13F          | `institutional_managers`, `institutional_13f_filings`, `institutional_holdings` + 2 views (`institutional_latest_filings`, `institutional_position_changes`) |
| Community                  | `shares`, `shared_comments`, `forum_categories`, `forum_threads`, `forum_posts`, `boards`, `board_items` |
| Settings & AI              | `user_settings`, `ai_settings`, `ai_journal_cache`, `dashboards`       |
| Expenses + OCR             | `expense_accounts`, `expense_categories`, `expense_transactions`, `expense_rules`, `expense_receipts` |

Sides are typed enums: `side_t = (buy, sell, short, cover)` for executions; `trade_side_t = (long, short)` and `trade_status_t = (open, closed)` for trades. Other enums cover order status, review status, asset class, alert triggers, sentiment sources, halt reason codes, etc.

---

## [0x05] HTTP API

**~990 axum routes** under `/api/` across 83 route files. Bearer-auth required on everything except `/health`, `/config`, `/auth/*`, and `/client-errors`. Four WebSocket endpoints expose live feeds. Frontend bindings live in `frontend/js/api.js`. Grouped:

| Group                      | Endpoints | Examples                                                          |
|----------------------------|-----------|-------------------------------------------------------------------|
| Auth + config              | ~6        | `GET /config`, `GET /auth/me`, `POST /auth/login`, `POST /auth/register` |
| Trades + executions        | ~20       | `GET/POST /trades`, `POST /trades/rollup`, `POST /trades/merge`, `POST /trades/bulk`, `GET/POST /executions` |
| Journal + AI + reviews     | ~15       | `GET /journal/day/{day}`, `POST /journal-ai/{id}/analyze`, `GET /trade-reviews/needed/{acct}` |
| Reports (17 cuts)          | ~20       | `/reports/{overview, by-symbol, by-day-of-week, by-hour, by-hold, r-distribution, comparison, exit-efficiency, liquidity, drawdown, risk-adjusted, calendar, …}` |
| **Live streams (WS)**      | 4         | `WS /ws/halts`, `WS /ws/catalysts`, `WS /ws/ticks`, `WS /ws/webull` |
| Research per-symbol        | ~10       | `/symbols/{sym}/{quote, signals, news, earnings, dividends, recommendations, insiders, fundamentals, holders}` |
| Chart transformations      | 13        | `GET /bars/{sym}`, `/bars/{sym}/heikin-ashi`, `/renko`, `/volume-profile`, `/ichimoku`, `/fibonacci`, `/supertrend`, `/swing-points`, `/candlestick-patterns`, `/pivots/{floor,camarilla,woodie,demark}` |
| Technical indicators       | 32        | `GET /bars/{sym}/{sma, ema, rsi, macd, bollinger, atr, roc, trix, dpo, coppock, schaff-trend, mass-index, adx, stochastic, williams-r, cci, mfi, donchian, parabolic-sar, anchored-vwap, aroon, awesome-oscillator, vortex, chaikin-volatility, obv, accumulation-distribution, force-index, keltner, vwap-bands, bb-squeeze, rsi-divergence, trend-channel}` — each takes `?interval&from&to&period[&...]` |
| Screener + scanners        | ~4        | `GET /screener/run`, `GET /screener/top`, `GET /scans/run` (24 presets) |
| Options + analytics        | 10        | `/options/{sym}`, `/options/{sym}/{max-pain, gex, iv-skew}`, `/greeks`, `/vol-surface/{sym}`, `/iv/scan`, `/iv/symbols/{sym}` |
| Stateless calculators      | 20        | `POST /calc/{kelly, dynamic-kelly, optimal-f, var-historical, var-gaussian, monte-carlo, risk-parity, risk-on-off, margin-call, margin-runway, buying-power, tax-loss-harvest, wash-sale, cost-basis, commission-optimizer, yield-curve, bond-duration, carry-score, currency-exposure, vix-term-structure}` |
| Trade analytics (POST)     | 35        | `POST /analytics/{tilt-detector, discipline-score, emotion-tags, overtrading, streaks, losing-streak-probability, winloss-asymmetry, pyramid-rules, cagr-simple, cagr-rolling, profit-factor, sortino, treynor, information-ratio, sharpe-by-window, high-water-mark, drawdown-duration, earnings-move-straddle, earnings-move-iv, pead, gap-analysis, calendar-bias, halt-risk, trade-quality, exit-timing, mae-stop-tuning, bracket-order, probability-of-touch, portfolio-greeks, concentration, sector-exposure, beta, beta-hedge, hedge-ratio, spread-payoff}` |
| Microstructure + regime    | 24        | `POST /microstructure/{order-book-imbalance, order-flow-classify, order-flow-aggregate, liquidity, market-impact, per-symbol-slippage, vwap-slippage, order-staleness}`, `/heatmaps/{intraday, dow-hour}`, `/regime/{equity, news-event}`, `/discipline/{time-in-force, open-type, trade-plan-checklist, stop-loss-backtest, stop-loss-best-of, pyramid-plan}`, `/options/calc/{iv-rank, iv-backtest, oi-change}`, `/clusters/{trade-features, correlation}`, `/setups/by-setup` |
| Discipline + calendar + extras | 29    | `POST /discipline/{daily-loss-limit, drawdown-throttle, goal-tracker, triple-screen, chandelier-stop, vol-stop-close}`, `/options/calc/{margin-naked-short, margin-vertical}`, `/portfolio/{position-aging, position-irr, mtm-reconciliation}`, `/sentiment/calc/put-call-ratio`, `/tax/reconcile-1099b`, `/calc/risk-reward`, `/analytics/{rolling-zscore, strategy-correlation, spread-attribution, pair-trade-signal}`, `/microstructure/twap`, `/charts/atr-cone`, `/bars/alligator`, `/calendar/{is-trading-day, next-trading-day, prior-trading-day, add-trading-days, trading-days-between, earnings-window, earnings-analysis}`, `/filter/symbols` |
| Markets + breadth          | ~7        | `/markets/snapshot` (60s cache), `/premarket/snapshot`, `/breadth/snapshot`, `/fear-greed`, `/sector-rotation`, `/heatmap` |
| Backtest + custom indicators | ~12     | `POST /backtest/run`, `POST /backtest/walk-forward`, `POST /custom-indicators/eval/{sym}` |
| Paper trading              | ~8        | `POST /paper/accounts`, `GET /paper/accounts/{id}/positions`, `POST /paper/accounts/{id}/orders` |
| Alerts + webhooks + hotkeys | ~15      | `GET/POST /alerts`, `GET/POST /strategy-alerts`, `GET/POST /hotkeys`, `GET/POST /webhooks`, `POST /webhooks/{id}/test` |
| Sentiment                  | ~5        | `/sentiment/{feed, ranked, symbol/{sym}, series/{sym}, poll}` |
| Crypto                     | 3         | `/crypto/markets`, `/crypto/global`, `/crypto/btc/chain` |
| Tax + analytics            | ~10       | `/tax-lots/{acct}`, `/r-distribution/{acct}`, `/discipline/{acct}`, `/mood-analytics/{acct}`, `/equity-forecast`, `/fill-quality/{acct}` |
| Webull (read-only)         | 2         | `POST /webull/connect` (tokens in memory only), `GET /webull/snapshot` |
| Expenses + OCR             | ~15       | `GET/POST /expense/transactions`, `POST /expense/import`, `POST /expense/receipts`, `GET /expense/report/schedule_c?year=` |
| Community                  | ~12       | `/shares`, `/shared/{slug}`, `/forum/threads`, `/mentorships`, `/boards` |
| Watchlists + filter-sets   | ~10       | `GET/POST /watchlists`, `GET /watchlists/{id}/{symbols,quotes}`, `GET/POST /filter-sets` |
| Custom dashboards          | ~5        | `GET/POST /dashboards`, `GET /dashboards/{id}` |
| Disclosures + earnings + news | ~10    | `GET /disclosures`, `GET /earnings/calendar`, `GET /news/recent`, `GET /news/search` |
| API tokens + import sources | ~7       | `GET/POST /api-tokens`, `PATCH /api-tokens/{id}/rate-limit`, `GET /imports/sources`, `POST /imports` |
| Client error sink          | 1         | `POST /client-errors` (no auth; browser-side error funnel) |

Desktop mode auto-logs in as the local user; the frontend talks to the embedded server on a random localhost port. Web mode requires `POST /api/auth/login` → returns JWT → `Authorization: Bearer …` on subsequent calls. A custom logging middleware (`log_mw.rs`) records every request with elapsed_ms; 4xx/5xx responses get a 4 KB body snippet attached to the log for offline debugging.

---

## [0x06] FRONTEND

`frontend/` is **vanilla JS + uPlot**. Zero npm, zero bundler, zero framework. **257 view modules + pure helper modules + runtime modules**, ~134k LOC JS + 2,512 LOC CSS. All views render into `<main id="app">` via hash-routed dispatch. **Tile launcher (Cmd-K)** is the primary entry point; topbar carries 11 shortcuts and the rest is the launcher. `?` opens the in-app tutorial.

| Category              | Tiles | Notable views                                                       |
|-----------------------|-------|---------------------------------------------------------------------|
| Live Markets          | 6     | Live Scanner (Finnhub WS), Halts, Catalysts, Pre-market, Tape, Heatmap |
| Trading               | 7     | Webull (read-only broker), Live Positions, Paper Trade, New Trade, Plans, Position Size (Kelly + correlation-aware), Hotkeys |
| Journal               | 9     | Journal (per-trade + daily + general), AI Journal, Trade Reviews, Trade Compare, Replay, Tape Replay, Discipline, Mood Analytics, Goals |
| Charts & Research     | 25    | Charts, Research, Watchlists, Screener, Scanners (24 presets), Top Signals, Compare, Pairs, Correlation, Sectors, Sector Rotation, Breadth, Fear/Greed, Sentiment, Dark Pool, Short Interest, Vol, Vol Surface, Options, Earnings Cal, Earnings IV, Disclosures, Economy, News, Crypto |
| Reports               | 11    | Dashboard, Reports (17 cuts), R-Multiple, Equity Forecast, Fill Quality, Risk, Rebalance, Tax Lots, Expenses, Calendar, Accounts Overview |
| Strategy & Automation | 7     | Backtest (stryke-JIT), Backtest Presets, Walk-forward, Custom Indicators, Strategy Alerts, Alerts, Webhooks |
| Community             | 4     | Shares, Community (forum), Mentorship, Boards |
| Admin & Data          | 9     | Import (12 brokers), CSV Wizard, Exports, Accounts, Tags, Search, Settings, Developer (API tokens), **Tutorial** (`?`) |

**Race-safe view dispatch** — `app.js` maintains a per-dispatch token (`currentViewToken()`) bumped on every navigation. Every view captures the token at render start and bails after each `await` if the token is stale, preventing the "`document.getElementById(...)` returns null after navigation" crash that hits naïve SPAs when slow async resolves into a replaced DOM. WebSocket reconnect loops and `setInterval` callbacks are also token-gated so leaving a view tears down its streams.

`js/api.js` wraps `fetch` with the JWT header, error reporting, JSON parsing, and an `ApiError` class. `js/error_reporter.js` funnels `window.onerror`, `unhandledrejection`, and overridden `console.error` to `POST /api/client-errors` (queue-capped at 200). `js/charts.js` owns all uPlot setup. `js/hud-theme.js` provides the cyberpunk chrome (5 color schemes, CRT scanlines, neon-border pulse). `js/alert_engine.js` polls alerts and fires sound + SpeechSynthesis voice + Notification (all SecurityError-guarded under Tauri's custom scheme).

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

## [0x0AB] LANDLORD / RENTAL PROPERTY (SCHEDULE E)

Same discipline as the Schedule C surface above, but for **Form 1040 Schedule E Part I** (Rental Real Estate). Migration `0032_rental_properties.sql` adds:

- **`rental_properties`** — one row per unit with `property_type` (IRS codes 1-8: single-family / multi-family / vacation-short-term / commercial / land / royalties / self-rental / other), purchase basis, land value (excluded from depreciation), `placed_in_service_at`, `recovery_period_years` (27.5 residential, 39 commercial), `fair_rental_days` + `personal_use_days` for line 2, QJV (spouse co-owner) and QBI safe-harbor (Rev. Proc. 2019-38, 250 hours/yr) flags.
- **`rental_tenants`** + **`rental_leases`** — tenancy term with `rent_amount` + `rent_frequency` + `rent_due_day` + `grace_days` + `late_fee_fixed` / `late_fee_pct` + `security_deposit` + `pet_deposit` + `deposit_held_by`. Status: `draft` / `active` / `expired` / `terminated_early`.
- **`rental_income`** — every receipt as an atom (`rent` / `late_fee` / `deposit_forfeit` / `reimbursement` / `royalty` / `parking` / `laundry` / `storage` / `other`) with `period_start`/`period_end`, optional `transaction_id` back-link into the bank-statement side. Dedupe index on `(property_id, posted_at, amount, payer_raw, kind)`.
- **`schedule_e_categories`** — 24 stable codes mapping to Schedule E lines 5-19 (Advertising / Auto-Travel / Cleaning-Maint / Commissions / Insurance / Legal-Prof / Mgmt-Fees / Mortgage-Interest / Other-Interest / Repairs / Supplies / Taxes / Utilities / Depreciation), plus 9 line-19 "Other" detail codes (HOA / landscaping / pest control / permits / appliance / property-mgmt software / bank fees / eviction / security).
- **`rental_expenses`** — categorized outflow with `is_capitalized` flag + `capital_useful_life` (improvement vs ordinary repair per Reg. §1.263(a)-3 — capitalized rows are excluded from line 14 and recovered via depreciation instead).
- **`rental_mileage`** — odometer log; `rate_per_mile` is persisted at log time so a 2024 trip stays at $0.67 even after the IRS publishes the 2025 rate. Folds into line 6 (Auto and Travel).
- **`rental_maintenance`** — work orders with `status` (open / in_progress / done / cancelled) + `priority` (low / normal / high / emergency) + back-link to the `rental_expenses` row that paid for it.
- **`rental_services_log`** — 250-hour QBI safe-harbor tracker. Each row is `hours` + `activity` + `performer` (self / employee / contractor). Section 199A allows a 20% deduction on rental income when the property qualifies as a trade or business; the safe harbor requires 250 hours of rental services per year per enterprise.
- **`receipts.rental_expense_id`** — additive column wiring the existing OCR receipt store into the rental side so a contractor invoice PDF can attach to a Schedule E line-14 expense the same way a receipt attaches to a Schedule C transaction.

`traderview-expense::schedule_e` is the **pure-compute roll-up** that takes the rows above + the year's depreciation number from `depreciation.rs` and emits a `ScheduleELine` per property (lines 3a / 3b / 5 / 6 / 7 / 8 / 9 / 10 / 11 / 12 / 13 / 14 / 15 / 16 / 17 / 18 / 19 / 20-total / 21-income-or-loss) and a `ScheduleEReport` totals block (lines 23a-e + 24 income / 25 loss / 26 total real-estate income). Capitalized improvements are excluded from line 14 ("Repairs") by design — a $15k roof goes through depreciation, not the repair line. Six tests in `schedule_e.rs` pin: capitalized excluded from line 14, mileage folds into line 6, winners and losers split correctly into lines 24/25, category-code round-trip matches the migration `code` column, and IRS property-type codes 1-8 match Schedule E instructions.

The landlord routes are mounted at **`/api/rental`** (`rental_routes.rs`, ~43 endpoints) and mirror the discipline of the `/api/expense` surface: ownership enforced at every endpoint via `ensure_property_owner` / `ensure_lease_owner` helpers, all reads/writes either filter `user_id = $1` directly or join through `rental_properties.user_id` with a `Forbidden` response when a row exists but belongs to a different user. Endpoints cover properties + tenants + leases + income + expenses + mileage + maintenance + services-log CRUD, plus five reports: `GET /report/schedule_e?year=YYYY` runs `schedule_e::roll_report` over a year's rows, `GET /properties/:id/qbi-hours?year=YYYY` returns logged vs 250-hour required for the QBI safe harbor, `GET /properties/:id/rent-roll?year=YYYY&month=M` per-lease expected/collected/balance with `paid` / `partial` / `due` / `late` status derived from `rent_due_day + grace_days`, `GET /properties/:id/depreciation?year=YYYY` per-property MACRS line-18 deduction, and `POST /deposit-interest` returns state-specific security-deposit interest accrual with citation. The router-builds-without-duplicate-routes smoke test in `routes.rs` covers the new mount.

`traderview-expense::rental_depreciation` is the **pure-compute MACRS engine** for rental real property: residential 27.5-year straight-line per IRS Pub 946 Table A-6, commercial 39-year straight-line per Table A-7a, both with the mid-month convention. Year-1 deduction depends on placed-in-service month (Jan = 3.485% residential, Dec = 0.152% residential, etc.), years 2 through the last full year use 1/27.5 = 3.636% (residential) or 1/39 = 2.564% (commercial), and the final partial year recovers whatever's left. Land is never depreciable — caller subtracts `land_value` from `purchase_price` before passing in the basis. Ten tests pin: Pub 946 Table A-6 January/December year-1 rates, Table A-7a July year-1 rate, year-2 full-year rate, pre-service and post-recovery edge cases, and a cumulative-recovery-never-exceeds-basis sweep across years 1-29.

`traderview-expense::contractor_1099` is the **Form 1099-NEC contractor $600 threshold tracker** — every landlord paying a non-corporate vendor (handyman, plumber, electrician, property manager) $600+ in a tax year must issue Form 1099-NEC by January 31. Missing the filing costs **$310 per form**, **$630 with intentional disregard** under §6721(e). The module aggregates the existing `rental_expenses` ledger by `vendor_normalized`, applies four exclusions per Reg. §1.6041, and flags vendors at risk.

Exclusions modeled:

- **Credit-card payments** (Reg. §1.6041-1(a)(1)(iv)) — excluded because the card processor files Form 1099-K. Detected via `method = "card"` on the entry (case-insensitive). Mixed card + check payments only count the check portion toward the qualifying total.
- **Corporation vendors** (Reg. §1.6041-3(p)) — C-corps and S-corps generally don't need 1099-NEC. Caller asserts via `vendor_is_corporation`; defaults to false (assume 1099-eligible — the safe default for landlords).
- **Attorney exception** (§6045(f)) — attorneys ALWAYS get 1099-NEC regardless of corporate status. The attorney corporation exception takes precedence over the general corporation exemption. Caller asserts via `vendor_is_attorney`.
- **Materials-only payments** — Form 1099-NEC reports payments for **services**. Lumber from a non-corporate sawmill doesn't trigger because no labor is involved. Caller asserts per-entry via `services_payment` (defaults to true).

Mounted at `POST /api/rental/1099-nec-report`. Eighteen tests pin: single vendor under $600 no 1099; **exactly $600 triggers** (≥, not >); $599.99 no 1099; multiple payments aggregate to threshold ($250 × 3 = $750 triggers); all-card payments excluded (note mentions 1099-K); mixed card + check counts only non-card portion ($400 card + $400 check = $400 qualifying, no trigger); over-threshold mixed ($400 card + $700 check = $700 qualifying, triggers); corporation vendor excluded; attorney corporation STILL triggers (§6045(f)); materials-only no 1099; mixed materials + services counts only services portion; year filter excludes other years; empty input no-op; multiple vendors aggregated separately; threshold override replaces $600 default; case-insensitive "CARD" method match; latest_payment date reflects max across the year; total_qualifying_payments aggregates across vendors requiring 1099.

`traderview-expense::bedbug_disclosure` is the **state bedbug disclosure + inspection-duty compliance table** — sibling to `heat_requirements`, `foreclosure_tenant_rights`, `lead_disclosure`, `detector_requirements`, `soi_protection`, `just_cause_eviction`, `dv_termination`, `lockout_penalties`, `application_fees`, `entry_notice`, `retaliation_windows`, `eviction_notices`, `late_fee_caps`, `deposit_interest`, `deposit_return_windows`, `lease_disclosures`, `habitability_remedies`, `rent_control`, `military_termination`, `security_deposit_caps`, and `contractor_1099`. Recent legislative wave (2009-2024) hit dense urban housing hardest after the post-2000 bedbug resurgence.

**Five regimes** across 51 jurisdictions:

| Regime                              | States                                                                  |
|-------------------------------------|-------------------------------------------------------------------------|
| **Pre-lease history disclosure**    | CA Civ. Code § 1954.603 (2017) / NJ N.J.A.C. § 5:10-3                  |
| **Informational pamphlet only**     | AZ § 33-1319 (SFH exempt) / AL / FL / GA / IL Chicago / MI / MN / NE / NV / OH / OR / PA / RI / SD / TX / WI / WV |
| **Post-discovery adjacent-unit notice** | NY RPL § 235-j (2010, amended 2024 — 72 hours)                      |
| **Inspection duty on tenant report** | ME 14 M.R.S. § 6021-A (5d) / KS K.S.A. § 58-2576a (5d) / IA Iowa Code § 562A.15 (7d) / NH RSA § 540-A (7d) |
| **No statewide statute**            | 24 other states — implied habitability + local ordinances              |

**NY's 72-hour adjacent-unit notice is unique** post-discovery requirement. Once landlord learns of an infestation, written notice to tenants in immediately adjacent units (above/below/side) is required within 72 hours per RPL § 235-j (amended 2024 — previous version required 24 hours to all tenants). Pinned by `ny_is_only_state_with_adjacent_notice_hours` (sweep verifying every other state has `adjacent_notice_hours = None`).

**NY 72-hour boundary is bright-line at exact 72**: complies AT 72h; fails at 73h. Pinned by `ny_72_hour_window_complies_at_exactly_72_hours` + `ny_72_hour_adjacent_notice_violation_at_73_hours`. The clock only starts when landlord LEARNS of infestation — `ny_no_violation_if_no_infestation_learned_of` (1000h without infestation = no violation).

**AZ § 33-1319 single-family home exemption is load-bearing.** AZ explicitly excludes single-family rentals from the bedbug pamphlet regime. CA does NOT have a similar exemption — pre-lease history disclosure required even for single-family. Pinned by `az_single_family_home_exempted` (SFH + missing pamphlet = complies) + `ca_sfh_not_exempted_from_disclosure` (CA SFH + missing disclosure = violation).

**ME 14 M.R.S. § 6021-A inspection duty 5-day boundary.** Tenant report triggers 5-day inspection clock. 6 days without inspection = violation; 5 days exact = within window. Pinned by `me_5_day_inspection_duty_violation_at_6_days` + `me_5_day_window_complies_at_exactly_5_days` + `me_complies_when_inspection_done_even_past_deadline` (30 days post-report but inspection done = complies).

**IA / NH 7-day inspection windows** are the next tier; KS matches ME at 5 days. Pinned by `inspection_duty_states_pinned` (4-state regime sweep with day count assertion).

**Multiple simultaneous violations stack.** CA missing both pre-lease history AND info pamphlet → 2 distinct violation entries. Pinned by `multiple_simultaneous_violations_stack`.

**No-statewide-statute states (24 jurisdictions) always comply** at the state level — implied habitability covenant and local ordinances govern instead. Pinned by `no_statute_states_always_comply` (24-state sweep with all flags missing → still complies).

Mounted at `POST /api/rental/bedbug-disclosure-check`. Twenty-five tests pin: 51-row coverage; CA pre-lease history violation; AZ pamphlet required + **single-family exemption** (load-bearing carve-out); CA SFH not exempted (distinguishes from AZ); **NY 72h boundary** (complies at 72h, fails at 73h, no violation pre-learn, complies if notified); ME 5-day boundary (complies at 5d, fails at 6d, complies if completed); KS 5-day mirror; IA 7-day boundary; NJ pre-lease required; no-statute 24-state sweep always complies; unknown state errors; case-insensitive; sorted all_states; non-empty citations; pre-lease history states (CA/NJ) pinned; **NY-only adjacent-notice sweep** (every other state has None); inspection-duty 4-state sweep (ME/KS = 5d, IA/NH = 7d); multiple violations stack.

`traderview-expense::heat_requirements` is the **state heat minimum temperature compliance table** — winter habitability obligation. Sibling to `foreclosure_tenant_rights`, `lead_disclosure`, `detector_requirements`, `soi_protection`, `just_cause_eviction`, `dv_termination`, `lockout_penalties`, `application_fees`, `entry_notice`, `retaliation_windows`, `eviction_notices`, `late_fee_caps`, `deposit_interest`, `deposit_return_windows`, `lease_disclosures`, `habitability_remedies`, `rent_control`, `military_termination`, `security_deposit_caps`, and `contractor_1099`. Failure to provide adequate heat is one of the most common habitability violations — leading to rent withholding, code enforcement, and M.G.L. c. 186 § 14 criminal felony exposure for willful interruption.

**Three regimes** across 51 jurisdictions:

| Regime                              | States                                                       |
|-------------------------------------|--------------------------------------------------------------|
| **Specific statute / code**         | NY / MA / IL / MN / CT / NJ / OR / DC / RI / MD / VA / WI / CA (13 with concrete temperatures) |
| **Implied habitability covenant**   | TX / FL / WA / OH / PA / NM / MT / NV / NH / IN / ND / NE / MI / DE / KS / KY / ME / NC / NH / SC / TN / UT / VT / WV / AZ / AK / CO / IA / OK / SD (~28 with no specific temp; caller applies local code) |
| **No statewide statute**            | AL / AR / GA / HI / ID / LA / MO / MS / SD / WY (10 states; HI listed because warm climate makes no heat needed) |

**Verified specific-statute rules:**

| State    | Day temp | Day window     | Night temp | Night window    | Heat season       | Outside trigger | Citation                              |
|----------|----------|----------------|------------|-----------------|--------------------|-----------------|---------------------------------------|
| NY (NYC) | 68°F     | 6am-10pm       | 62°F       | 10pm-6am        | Oct 1 - May 31    | **< 55°F**      | NYC Admin. Code § 27-2029             |
| MA       | 68°F     | 7am-11pm       | 64°F       | 11pm-7am        | Sept 16 - Jun 14  | none            | M.G.L. c. 105 § 410.201               |
| IL       | 68°F     | 8:30am-10:30pm | 66°F       | 10:30pm-8:30am  | Sept 15 - Jun 1   | none            | Chicago Mun. Code § 13-196-410        |
| MN       | 68°F     | 24/7 (no split) | 68°F      | n/a             | Oct 1 - Apr 30    | none            | Minn. Stat. § 504B.161                |
| CT       | 65°F     | 24/7           | 65°F       | n/a             | Oct 1 - May 31    | none            | Conn. Gen. Stat. § 47a-7              |
| NJ       | 68°F     | 6am-11pm       | 65°F       | 11pm-6am        | Oct 1 - May 1     | none            | N.J.A.C. § 5:10-14.3                  |
| OR       | 68°F     | 6am-10pm       | 60°F       | 10pm-6am        | Oct 1 - May 1     | none            | ORS § 90.320                          |
| DC       | 68°F     | 6am-11pm       | 65°F       | 11pm-6am        | Oct 1 - May 1     | none            | 14 DCMR § 503                         |
| RI       | 68°F     | 6am-11pm       | 64°F       | 11pm-6am        | Oct 1 - May 1     | none            | R.I. Gen. Laws § 45-24.3-8            |
| MD       | 68°F     | 7am-11pm       | 65°F       | 11pm-7am        | Oct 1 - May 1     | none            | Md. Code Real Prop. § 8-211           |
| VA       | 68°F     | 6am-11pm       | 65°F       | 11pm-6am        | Oct 15 - May 1    | none            | Va. Code § 36-105                     |
| WI       | 67°F     | 24/7           | 67°F       | n/a             | Oct 1 - Apr 30    | none            | Wis. Admin. Code § ATCP 134.04        |
| CA       | 70°F     | 24/7           | 70°F       | n/a             | Nov 1 - May 31    | none            | Cal. Code Regs. tit. 25 § 34          |

**NY's 55°F outside-temperature trigger is load-bearing and unique to NY.** The NYC Heat Law's daytime 68°F requirement only fires when outside temp drops below 55°F. Every other state requires continuous compliance during heat season regardless of outside temperature. Pinned by `ny_is_only_state_with_outside_trigger` (sweep verifying every other state has `outside_temp_trigger_f = None`) + `ny_day_outside_above_55_disengages_requirement` (60°F outside = no requirement, complies even at 60°F indoor) + `ma_no_outside_temp_trigger_continuous_requirement` (MA fails at 67°F even with 70°F outside).

**Night requirement is unconditional**. NY's nighttime 62°F applies regardless of outside temp — the 55°F trigger gates only the daytime window. Pinned by `night_window_outside_trigger_does_not_apply` (2am, indoor 60°F, outside 80°F → still fails the 62°F night requirement).

**Heat season wrap-around math is load-bearing.** Most heat seasons span the calendar boundary (Oct 1 - May 31 wraps past Jan 1). The `in_heat_season` helper handles `start_month > end_month` correctly. Pinned by `ny_heat_season_wraps_past_jan_1` (Jan 15 is in NY's Oct-May season) + `ny_heat_season_boundary_oct_1_in_season` + `ny_heat_season_boundary_sep_30_out_of_season` (one-day boundaries) + MA September boundary tests.

**Day vs night boundary** is a half-open interval `[day_start, night_start)`. NY at hour 22 (10pm) exact = night (start of night window); hour 21 (9pm) = day. Pinned by `ny_day_night_boundary_at_22_hour`.

**Hawaii is listed as no-statewide-statute deliberately** — warm climate makes heat moot; the no_statute classification is correct rather than a coverage gap. Pinned by `hawaii_no_statewide_statute`.

Mounted at `POST /api/rental/heat-requirements-check`. Twenty-six tests pin: 51-row coverage; **NY day 68°F when outside < 55°F** + **NY outside ≥ 55°F disengages** + **NY night 62°F regardless of outside**; NY heat-season boundary (Oct 1 in season, Sept 30 out of season, Jan 15 wraps); MA day 68 / night 64 with 7am-11pm split + MA continuous requirement (no outside trigger); MN single temp 68 no day/night split; CT 65°F with 24/7 application; **habitability-only states (7-state sweep) return complies=true with habitability_only flag**; **no-statute states (9-state sweep) return complies=true with no_statute flag**; Hawaii no-statute deliberately; MA Sept 16/15 season boundary; unknown state handled; case-insensitive; sorted all_states; non-empty citations; specific-statute states pinned (13-state regime sweep); **NY-only outside-trigger pin** (sweep verifying NoTrigger on all other 50 states); shortfall math correct; **NY day-night boundary at hour 22** (half-open interval pin); night window unconditional vs outside trigger.

`traderview-expense::foreclosure_tenant_rights` is the **federal PTFA + state foreclosure tenant rights compliance table** — sibling to `lead_disclosure`, `detector_requirements`, `soi_protection`, `just_cause_eviction`, `dv_termination`, `lockout_penalties`, `application_fees`, `entry_notice`, `retaliation_windows`, `eviction_notices`, `late_fee_caps`, `deposit_interest`, `deposit_return_windows`, `lease_disclosures`, `habitability_remedies`, `rent_control`, `military_termination`, `security_deposit_caps`, and `contractor_1099`.

**Federal floor (universal)**: Protecting Tenants at Foreclosure Act (PTFA) — enacted 2009 as Title VII of the Helping Families Save Their Homes Act, sunset in 2014, then **permanently reinstated in 2018** by § 304 of the Economic Growth, Regulatory Relief, and Consumer Protection Act (EGRRCPA). Two core protections:

1. **90-day notice minimum** — all bona fide tenants must receive ≥ 90 days written notice before vacating after foreclosure
2. **Lease honor through expiration** — tenant may stay through bona fide lease end if the lease has > 90 days remaining AND was entered into before notice of foreclosure

**Bona fide tenancy three-prong test** — ALL three required:

| Prong | Test                                                                          | Field                            |
|-------|-------------------------------------------------------------------------------|----------------------------------|
| (a)   | Tenant is NOT the mortgagor or spouse/parent/child of mortgagor               | `bona_fide_prong_not_mortgagor_family` |
| (b)   | Lease/tenancy was the result of an arm's length transaction                   | `bona_fide_prong_arm_length`     |
| (c)   | Rent is at or above FMR OR is reduced/subsidized by federal/state/local subsidy | `bona_fide_prong_fair_market_rent` |

Failing ANY prong → PTFA protections do NOT apply. Pinned by `all_three_bona_fide_prongs_required` (combinatorial 2×2×2 sweep — only the all-true cell returns bona_fide_tenant=true).

**Subsidized below-FMR rent satisfies prong (c).** Section 8 voucher tenants with below-market rent still qualify because subsidy explicitly counts under the statute. Pinned by `subsidized_rent_satisfies_prong_c`.

**State additions** layered atop the federal floor (federal is a floor, not a ceiling):

| Regime                            | States                                                       |
|-----------------------------------|--------------------------------------------------------------|
| **Federal floor only**            | 44 states (most jurisdictions; standard PTFA 90 days)        |
| **Extended notice period**        | DC (120 days § 42-3505.01a) / CT / IL / MD / NY / OR         |
| **Right of first refusal**        | CA (SB 1079 + AB 1837) — included in Comprehensive regime    |
| **Comprehensive state protections** | CA / MA c. 186A / NJ § 2A:50-69 — extended notice + ROFR + non-honor of owner-occupant exception |

**DC is the only state with a longer notice period than 90 days** at 120 days under § 42-3505.01a. Pinned by `dc_120_day_notice_is_the_only_extended_period_state` (sweep verifying every other state is 90 days).

**Purchaser-owner-occupant exception** is federal: if the foreclosure purchaser will occupy the unit as a primary residence, the 90-day notice replaces the lease-completion right. However, **MA and NJ do NOT honor this exception** — even with a primary-residence purchaser, the tenant may still complete the lease in those two pro-tenant comprehensive-regime states. Pinned by `massachusetts_does_not_honor_owner_occupant_exception` + `new_jersey_does_not_honor_owner_occupant_exception`. Most states honor the exception (federal default).

**California's right of first refusal under SB 1079 / AB 1837** lets a bona fide tenant purchase the property at the trustee's sale within 45 days post-auction. Pinned by `california_right_of_first_refusal_available` + `texas_no_right_of_first_refusal` (negative case).

**Tenant-may-complete-lease logic**: lease must extend beyond the notice vacate deadline AND no owner-occupant override. Lease already expired before notice → no completion path. Pinned by `tenant_may_complete_lease_when_extends_beyond_notice` + `lease_already_expired_no_completion_path`.

**Bright-line 90-day boundary** for federal floor: 90 days exact = complies; 89 days = fails. Pinned by `federal_floor_90_day_notice_complies_at_exact_boundary` + `federal_floor_89_day_notice_fails`.

Mounted at `POST /api/rental/foreclosure-tenant-check`. Twenty-five tests pin: 51-row coverage; **90-day exact boundary** (federal); 89-day federal short; **DC 120-day extended period** (exact + short); **CA right of first refusal**; TX no ROFR (negative); each prong's failure mode individually flagged; subsidized rent satisfies prong (c) [Section 8 case]; tenant-may-complete-lease when lease extends; owner-occupant exception overrides lease completion (federal default); **MA + NJ don't honor owner-occupant exception** (comprehensive-regime pin); lease already expired no completion path; unknown state → federal floor; case-insensitive; sorted all_states; non-empty citations; comprehensive-states 3-state regime sweep; **DC-only 120-day sweep** (every other state is 90); **all-three-prongs combinatorial sweep** (8 combinations, only all-true qualifies); note describes "MAY complete lease" path; note states shortfall when noncompliant.

`traderview-expense::lead_disclosure` is the **federal Title X + state lead-based paint compliance table** — sibling to `detector_requirements`, `soi_protection`, `just_cause_eviction`, `dv_termination`, `lockout_penalties`, `application_fees`, `entry_notice`, `retaliation_windows`, `eviction_notices`, `late_fee_caps`, `deposit_interest`, `deposit_return_windows`, `lease_disclosures`, `habitability_remedies`, `rent_control`, `military_termination`, `security_deposit_caps`, and `contractor_1099`.

**Federal floor (universal)** — Section 1018 of Title X of the Residential Lead-Based Paint Hazard Reduction Act of 1992 (40 CFR Part 745 / 24 CFR Part 35 Subpart A) applies to **all pre-1978 private rentals nationwide**. Four mandatory landlord disclosure elements:

1. EPA pamphlet "Protect Your Family From Lead in Your Home"
2. Federal Lead Warning Statement in the lease (specific federal language)
3. Disclosure of any known lead-based paint records / prior inspections
4. 10-day risk-assessment window for tenant before lease binds

**Federal penalty**: $10,000 per violation under 40 CFR § 745.118(c), plus tenant may sue for **treble damages**.

**Five state regime types** layered atop the federal floor:

| Regime                              | States                                                          | Trigger                              |
|-------------------------------------|-----------------------------------------------------------------|--------------------------------------|
| **Federal floor only**              | 41 states (AK / AL / AR / AZ / CA / CO / DE / FL / GA / HI / IA / ID / IN / KS / KY / LA / ME / MI / MO / MS / MT / NC / ND / NE / NH / NM / NV / OH / OK / OR / PA / SC / SD / TN / TX / UT / VA / WA / WV / WY) | No state additions |
| **Child-based deleading**           | MA / CT / MN / WI                                               | Child under 6 in household           |
| **Periodic inspection**             | NJ (2022) / MD / DC                                             | On 3-year or annual cycle            |
| **Inspection at occupancy change**  | IL / RI / VT                                                    | Each new tenant                      |
| **Comprehensive (combined)**        | NY (Pub. Health Law + NYC LL1)                                  | Child + periodic + investigation     |

**MA Lead Law (M.G.L. c. 111 § 197, 1971) is the strictest in the country.** When a child under 6 occupies a pre-1978 rental, the landlord must "delead" — remove or permanently cover all lead-paint hazards — regardless of blood-lead level. Federal compliance alone is NOT sufficient. Pinned by `ma_strict_law_requires_deleading_with_child_under_6` (federal-compliant + child + no deleading = NOT compliant) + `ma_no_child_under_6_no_state_violation` (no child = MA state rules don't trigger, federal floor alone suffices).

**NJ Lead-Safe Law (P.L. 2021, c.182, effective July 2022)** requires periodic inspections of ALL pre-1978 rentals on a 3-year cycle, regardless of whether a child occupies. Initial inspection deadline was July 22, 2024 (or first tenant turnover if earlier). Pinned by `nj_lead_safe_law_periodic_inspection_required` (no child needed; missing inspection alone is a violation).

**RI / VT inspection-at-occupancy-change** is the third major model. Each new tenant triggers a new inspection requirement; landlord cannot stack inspections by spacing turnovers. Pinned by `ri_inspection_at_change_of_occupancy_required`.

**NY (NYC Local Law 1 of 2004) is the comprehensive case.** Multiple regime flags fire simultaneously (child-trigger + periodic). When both miss, two distinct violations emit. Pinned by `ny_comprehensive_requires_both_periodic_and_child_action` (2 simultaneous violations).

**1978 cutoff is a hard `<` boundary, not `≤`**. Property built in 1978 exactly is post-1978 and gets no federal obligation. Property built 1977 gets the full Title X treatment. Pinned by `property_built_1978_exact_is_post_1978_no_obligation` + `property_built_1977_triggers_federal_floor`.

**Federal-floor-only states do not stack state-side violations.** When state regime = `FederalFloorOnly` (TX / FL / CO / AZ etc., 41 jurisdictions), even missing state-side inputs don't produce violations because the state has no statutory requirement. Pinned by `federal_floor_only_states_no_state_additions` (8-state sweep).

**Post-1978 property skips all state rules too.** Even in MA, a 1980-built rental is exempt from the Massachusetts Lead Law because the law applies only to pre-1978 housing. Pinned by `post_1978_property_state_lead_rules_dont_apply`.

Mounted at `POST /api/rental/lead-disclosure-check`. Twenty-three tests pin: 51-row coverage; **1977/1978/1980 cutoff boundary** (regulatory bright line); each of 4 federal disclosure elements individually triggers a violation; **MA child-under-6 deleading** trigger + no-child case; NJ periodic-inspection regime; RI occupancy-change regime; **NY comprehensive (two simultaneous violations)** when both rules fail; 8-state federal-floor-only sweep; unknown state handled; case-insensitive; sorted `all_states()`; non-empty citations; **child-based-deleading 9-state sweep** (CT/MA/MN/WI/NY/IL/RI/VT/DC); **periodic-inspection 4-state sweep** (NJ/MD/NY/DC); **occupancy-change 4-state sweep** (IL/RI/VT/MD); post-1978 in MA also skips state rules; federal penalty constant matches 40 CFR § 745.118 ($10k/violation); treble-damages flag pinned across 10 states; multiple violations stack (NJ regime: 2 federal + 2 NJ inspections = 4); note describes post-1978 no-obligation case.

`traderview-expense::detector_requirements` is the **state-by-state smoke + carbon monoxide detector compliance table** — life-safety obligation with massive liability exposure (tenant death from non-functional CO detector = wrongful death suit + criminal exposure in some states). Sibling to `soi_protection`, `just_cause_eviction`, `dv_termination`, `lockout_penalties`, `application_fees`, `entry_notice`, `retaliation_windows`, `eviction_notices`, `late_fee_caps`, `deposit_interest`, `deposit_return_windows`, `lease_disclosures`, `habitability_remedies`, `rent_control`, `military_termination`, `security_deposit_caps`, and `contractor_1099`.

**All 50 states require smoke detectors in residential rental units.** The variation lives in placement rules (every bedroom vs outside sleeping areas vs every level), power source (10-year sealed battery vs replaceable battery), and landlord-install obligations at occupancy.

**Three CO detector regimes:**

| Regime                              | States                                                                  |
|-------------------------------------|-------------------------------------------------------------------------|
| **All units (universal)**           | CO / CT / DC / IL / MA / MD / ME / MN / NH / NJ / NY / OR / RI / VT / WA (15 jurisdictions) |
| **Only with fuel source or garage** | CA / AZ / DE / FL / GA / HI / IA / KS / KY / LA / MI / MT / NC / NE / NM / NV / OH / PA / TN / TX / UT / VA / WI / AK + ~10 others |
| **No statewide CO (smoke only)**    | AL / AR / ID / IN / MO / MS / ND / OK / SC / SD / WV / WY                |

**CA model carve-out is load-bearing.** The CA rule under Health & Safety Code § 13113.7 / SB 745 requires CO detectors only when the property has a fossil-fuel-burning appliance OR an attached garage. Property without either (e.g., all-electric, detached garage) doesn't trigger the CO requirement at all. Pinned by `california_co_only_with_fuel_or_garage` (no fuel + no garage = no CO required, complies) + `california_co_required_with_garage_only` (garage alone triggers) + `california_co_required_with_fuel_only` (fuel alone triggers). The "OR" between the two conditions is the load-bearing logic.

**10-year sealed battery requirement** is the modern wave — CA SB 745 (2014, replacement triggers), NY NYC Local Law 111, MD, OR. Sealed unit prevents the "battery removed at 3am because it was chirping" failure mode that drives most tenant deaths from CO poisoning and fires. Pinned by `ten_year_battery_states_pinned` (4-state sweep) + the per-state violation tests for CA and NY.

**Smoke in every bedroom is rarer than outside-sleeping-area placement.** Only CA, TX, and WI require smoke in every bedroom in the table. All states require smoke outside each sleeping area. Pinned by `texas_smoke_in_every_bedroom_required` (TX-specific) + `smoke_outside_sleeping_area_universal_violation_check` (8-state sweep).

**Universal smoke + landlord-install** — every row has `smoke_in_every_bedroom_installed` or `outside_each_sleeping_area` checked AND `landlord_install_at_occupancy = true`. The landlord-install obligation means the landlord must verify detectors are present and working at occupancy; tenant takes over routine battery replacement after that.

**Violations list distinguishes missing-detector from wrong-placement.** Multiple violations stack — pinned by `multiple_violations_listed_in_result` (3 simultaneous violations all listed individually). Note text reports the count.

Mounted at `POST /api/rental/detector-check`. Twenty-four tests pin: 51-row coverage; CA fully compliant; **CA OnlyWithFuelOrGarage carve-out** (no fuel + no garage → CO not required); CA CO triggered by garage alone; CA CO triggered by fuel alone; NY AllUnits regime (CO always required regardless of fuel); MA AllUnits same; IL every-level-with-sleeping rule (CO on each level violation); TX every-bedroom smoke rule (TX-specific); **CA 10-year sealed battery violation**; **NY NYC Local Law 111 sealed battery** violation; MO smoke-only no-CO regime (NoStatewide); smoke-outside-sleeping universal sweep; landlord-install-at-occupancy violation; CO model with no-fuel/no-garage produces no violation; multiple violations stack in list; unknown state errors; case-insensitive lookup; sorted `all_states()`; non-empty citations; **CO-only states pinned** (8-state regime sweep); **CO AllUnits states pinned** (15-state regime sweep); **NoStatewide states pinned** (12-state sweep); 10-year sealed battery states pinned.

`traderview-expense::soi_protection` is the **state-by-state source-of-income (SOI) discrimination protection table** — sibling to `just_cause_eviction`, `dv_termination`, `lockout_penalties`, `application_fees`, `entry_notice`, `retaliation_windows`, `eviction_notices`, `late_fee_caps`, `deposit_interest`, `deposit_return_windows`, `lease_disclosures`, `habitability_remedies`, `rent_control`, `military_termination`, `security_deposit_caps`, and `contractor_1099`. The **federal Fair Housing Act does NOT cover source of income** — landlords may refuse Section 8 voucher holders nationwide unless a state or local statute provides protection. ~18-20 states + DC + many cities have enacted SOI protection since CT (1989) became the first.

**Three regimes:**

| Regime                              | States                                                                       |
|-------------------------------------|------------------------------------------------------------------------------|
| **Statewide full protection**       | CA / CO / CT / DC / DE / IL / MA / MD / ME / MN / NJ / NM / OR / RI / VA / VT / WA (17 jurisdictions) |
| **Statewide challenged / partial**  | NY (HSTPA 2019 struck down at appellate level March 2026 — NYC local survives) / UT (veterans only) |
| **No statewide (federal floor only)** | 33 other states; local ordinances in GA (Atlanta), PA (Philadelphia/Pittsburgh), TN (Memphis), TX (Austin), WI (Madison/Dane County) |

**Historical waves:**

| Wave            | States                                  |
|-----------------|-----------------------------------------|
| Pre-2000 pioneers | DC (1977), CT/MA/VT (1989), RI (1996) |
| 2002-2014       | NJ (2002), ME (2009), OR (2014)         |
| 2018 wave       | WA (2018)                               |
| 2019 wave       | CA SB 329 (2019), NY HSTPA (2019)       |
| 2020 wave       | CO / DE / MD / VA — racial-equity / fair-housing push |
| 2023 wave       | IL / MN / NM — most recent batch        |

**`verify_current_status_needed` flag** is load-bearing. NY's SOI provision was struck down at the appellate level in March 2026 — statewide enforcement is contested but NYC Human Rights Law still applies. UT § 13-21-302 covers veterans only, not full voucher population. Both flag `verify_current_status_needed: true` to tell the UI to surface "check current court status + locality" rather than reporting a definitive answer. Pinned by `new_york_challenged_regime_verify_needed` and `utah_partial_only_veterans` + the `challenged_states_set_verify_needed_flag` sweep.

**Voucher type doesn't change the answer.** Section 8 HCV, HUD-VASH, FUP, and other federal/state subsidy vouchers all get the same state-law treatment — the SOI statute classifies "source of income" without distinguishing voucher programs. The `voucher_type` enum is captured on input for future-proofing if a state someday differentiates. Pinned by `voucher_type_does_not_change_state_law_outcome` (CA returns identical results across all four voucher types).

**Local-only ordinances surface in the citation.** GA, PA, TN, TX, WI all have `NoProtection` regime statewide but mention local ordinances (Atlanta / Philadelphia / Memphis / Austin / Madison) in the citation field. Caller must check municipal law for these states. Pinned by `local_only_states_flagged_in_citation`.

**Unknown state defaults to federal floor (landlord may refuse).** Safest assumption — if we don't have the state in the table, federal FHA doesn't cover SOI so the landlord can refuse. Pinned by `unknown_state_defaults_to_federal_floor`.

Mounted at `POST /api/rental/soi-protection-check`. Eighteen tests pin: 51-row coverage; CA full protection (landlord may NOT refuse); TX no statewide (landlord may refuse, federal floor only); CT 1989 oldest statewide; OR 2014 first post-2010; **NY challenged regime sets verify flag**; UT partial-only-veterans sets verify flag; **17-state full-protection sweep** (catches regime-flag regression); voucher_type doesn't change outcome (all four types same); unknown state defaults to federal floor; case-insensitive lookup; sorted `all_states()`; non-empty citations; **2023 wave** (IL/MN/NM all enacted 2023); **2020 wave** (CO/DE/MD/VA); pre-2000 pioneers (CT/MA/VT/RI/DC); local-only-states flagged in citation (GA/PA/TN/TX/WI); `verify_current_status_needed` flag set for both NY and UT.

`traderview-expense::just_cause_eviction` is the **state-by-state just-cause eviction + relocation assistance table** — sibling to `dv_termination`, `lockout_penalties`, `application_fees`, `entry_notice`, `retaliation_windows`, `eviction_notices`, `late_fee_caps`, `deposit_interest`, `deposit_return_windows`, `lease_disclosures`, `habitability_remedies`, `rent_control`, `military_termination`, `security_deposit_caps`, and `contractor_1099`. Major statutory shift 2019-2021 — CA AB 1482 (Tenant Protection Act), OR SB 608, WA HB 1236 — changed whether a landlord can terminate at lease end at all, and required relocation assistance equal to one month's rent for no-fault grounds.

**Four regimes**:

| Regime                          | Coverage threshold | Relocation                          | States                                  |
|---------------------------------|--------------------|-------------------------------------|-----------------------------------------|
| **Statewide after N months**    | CA 12mo / OR 12mo  | 1 month current rent                | CA Civ. Code § 1946.2 (AB 1482) / OR ORS § 90.427 (SB 608) |
| **Statewide day-one**           | Immediate          | WA: 1 mo rent / DC: 0 / NJ: 0       | WA RCW § 59.18.650 (HB 1236) / DC § 42-3505.01 / NJ Anti-Eviction Act |
| **Partial by building**         | Rent-regulated / 4+ unit only | Statewide does NOT reach all rentals — caller checks rent-regulation status | NY HSTPA / IL local ordinances / ME 14 M.R.S. § 6002 |
| **No statewide regime**         | No coverage        | No relocation                       | 41 other states                         |

**OR small-landlord exemption is load-bearing.** SB 608 carves out landlords with ≤ 4 units from the relocation obligation even though the just-cause requirement still applies — tenant still gets protection, landlord still needs grounds, but no $$$ on no-fault terminations. Pinned by `oregon_small_landlord_4_units_exempt_from_relocation` (4 units exempt, $0 relocation) and `oregon_5_unit_landlord_owes_relocation` (5 units owes full month's rent). CA has no equivalent small-landlord carve-out.

**At-fault vs no-fault distinction.** All grounds satisfy just-cause once it applies, but only no-fault grounds (owner move-in, substantial remodel, withdrawal from market) trigger relocation assistance. At-fault grounds (non-payment, lease violation, nuisance, criminal activity) require no payment regardless of state. Pinned by `california_at_fault_grounds_no_relocation` and `oregon_at_fault_grounds_no_relocation_regardless_of_unit_count` (the OR small-landlord exempt flag stays false on at-fault path because the exemption only matters for no-fault).

**Relocation is current-rent-anchored.** The figure is the displaced tenant's monthly rent as of the notice date — not market rent. AB 1482 makes this explicit; OR SB 608 mirrors. Deliberate pro-tenant choice: the displaced tenant is compensated for what THEY were paying.

**Boundary at 12-month threshold matters for both CA and OR**: 11 months → no-cause termination still available; 12 months exactly → just-cause kicks in. Pinned by `california_under_12_months_no_just_cause_required` and `california_at_12_months_just_cause_required` + the OR mirror in `oregon_at_12_months_boundary`.

**PartialByBuilding flag (`partial_by_building_check_rent_regulation`) tells the caller to look further.** NY HSTPA covers rent-stabilized buildings, IL has Chicago RLTO + Evanston + Mt. Prospect local ordinances, ME applies just-cause to 4+ unit buildings only. The compute returns `just_cause_required: false` for these (because the statewide regime doesn't reach the building by default) but sets the flag so the downstream caller queries the appropriate per-building or per-municipality table.

**NJ Anti-Eviction Act 1974 is the strongest pro-tenant in the US** but has no statutory relocation (regime carries `relocation_months_rent: 0`). The regime still flags `just_cause_required: true` and `no_cause_termination_available: false`. Pinned by `new_jersey_anti_eviction_act_day_one_no_relocation`.

Mounted at `POST /api/rental/just-cause-check`. Twenty-five tests pin: 51-row coverage; **CA 12-month boundary** (under/at); at-fault grounds satisfy without relocation; no-fault owner move-in = 1 mo rent; **OR small-landlord 4-units exempt + 5-units owes** (load-bearing carve-out); OR 12mo boundary mirrors CA; WA day-one + 1mo relocation; NJ Anti-Eviction Act day-one no-relocation; DC day-one no-relocation; **NY/ME/IL partial-by-building flag set** (caller-must-check-further regression target); no-statewide states pass through; unknown state errors; case-insensitive lookup; sorted `all_states()`; non-empty citations on every row; substantial-remodel and withdrawal-from-market both trigger CA relocation; **OR at-fault grounds never trigger relocation** even at 100 units; zero rent → zero relocation no overflow; **i64::MAX/2 rent no overflow** (saturating_mul regression target).

`traderview-expense::dv_termination` is the **state-specific domestic violence early lease termination table** — sibling to `lockout_penalties`, `application_fees`, `entry_notice`, `retaliation_windows`, `eviction_notices`, `late_fee_caps`, `deposit_interest`, `deposit_return_windows`, `lease_disclosures`, `habitability_remedies`, `rent_control`, `military_termination`, `security_deposit_caps`, and `contractor_1099`. Federal **VAWA** (34 U.S.C. § 12491) provides a floor for federally-assisted housing (HUD, Section 8, LIHTC, public housing). State laws extend the protection to the private market and provide affirmative early-termination rights with notice + documentation requirements.

Four notice-period bands + three special cases:

| Band                    | States                                                          |
|-------------------------|-----------------------------------------------------------------|
| **3 days**              | IL Safe Homes Act (765 ILCS 750/15) — strictest pro-tenant      |
| **7 days**              | ME / NM                                                          |
| **14 days**             | CA § 1946.7 / DC § 42-3505.07 / HI § 521-80 / OR § 90.453 / TN / VT |
| **28-30 days**          | AZ / CT / DE / IA / IN / LA / MD / MI / MT / NC / ND / NE / NH / NJ / NV / NY § 227-c / PA / RI / SD / TX § 92.0161 / UT / VA / WI |
| **End of current month**| WA § 59.18.575 (calendar-anchored)                              |
| **Immediate (0 days)**  | MA § 24 / MN § 504B.206 (immediate with documentation)          |
| **No fixed period**     | CO § 38-12-402 (statute exists, "reasonable" notice required)   |
| **No state statute**    | 15 states (VAWA floor only)                                     |

**Documentation requirements** are nearly uniform across states with statutes: at least ONE of (a) protective order, (b) police report, or (c) qualified-third-party statement (medical provider, mental health professional, victim service provider, clergy). Compute returns `documentation_sufficient: true` if any of the three flags is true.

**Documentation freshness window** is state-specific and varies 30 days (CT, IA, NE) → 60 days (DC, IL, NJ, NV, WI) → 90 days (HI, MA, NC, OR, WA) → 180 days (CA, DE, RI) → no requirement (most 30-day-notice states). Notice date - incident date must be within the window. Pinned by `california_180_day_freshness_window` (179 days inside / 181 days outside).

**Three immediate-termination triggers** can override the notice requirement entirely:
- **Co-tenant violence**: TX § 92.0161(b), AZ, HI, IL, MA, MN, OR allow immediate termination when the abuser is a co-tenant on the lease
- **Landlord/agent violence**: WA § 59.18.575, AZ, DC, HI, IL, MA, MN, OR allow immediate termination when the abuser is the landlord
- Compute returns `immediate_termination_available: true` whenever the state's per-trigger flag is set AND the corresponding input flag is true

States WITHOUT a carve-out don't override: setting `violence_by_co_tenant = true` in CA has no effect because CA § 1946.7 still requires the 14-day notice. Pinned by `co_tenant_violence_does_not_trigger_immediate_in_states_without_carveout` (7-day actual still shortfalls by 7 against the 14-day requirement).

**Washington's EndOfCurrentMonth regime** is calendar-anchored, not day-counted — required_notice_days returns None. Notice on Jan 15 → must terminate ≥ Jan 31 (end of month). The `end_of_month()` helper handles year rollover (Dec → next Jan) and short-month February correctly. Pinned by `washington_short_month_february_handled` (Feb 5 notice → Feb 28 end) and `washington_december_notice_wraps_to_dec_31` (Dec 15 notice → Dec 31).

Mounted at `POST /api/rental/dv-termination-check`. Twenty-four tests pin: 51-row coverage; CA 14-day exact boundary + 13-day shortfall; TX 30-day boundary; **IL 3-day strictest band**; WA end-of-month with year-rollover and short-Feb handling (calendar-anchored math regression targets); WA Jan-31 boundary; TX co-tenant violence waives notice; WA landlord violence waives notice; **CA without co-tenant carve-out doesn't waive** (7-day shortfall pinned); documentation sufficient with only protective order; documentation insufficient when all three false; CA 180-day freshness window inside/outside; no-freshness states accept any-age documentation; 15 no-statute states flagged correctly with VAWA-only note; unknown state errors; case-insensitive lookup; sorted `all_states()`; non-empty citations; notice-before-incident negative freshness window; end-of-month boundary same-day; shortfall zero when compliant; shortfall reported only for Days regime not EndOfCurrentMonth (note text carries the explanation instead).

`traderview-expense::lockout_penalties` is the **state-specific self-help eviction penalty table** — sibling to `application_fees`, `entry_notice`, `retaliation_windows`, `eviction_notices`, `late_fee_caps`, `deposit_interest`, `deposit_return_windows`, `lease_disclosures`, `habitability_remedies`, `rent_control`, `military_termination`, `security_deposit_caps`, and `contractor_1099`. Self-help eviction (lockout, utility shutoff, removal of tenant property without court order) is universal landlord exposure — every state prohibits it — but the dollar consequences vary by 10× across jurisdictions.

**Seven distinct penalty regimes** are present in the table:

| Regime                          | States                                  | Formula                                                              |
|---------------------------------|-----------------------------------------|----------------------------------------------------------------------|
| **Additive per-day**            | CA § 789.3                              | `max($100/day × days, $250 minimum) + actual + fees`                |
| **Additive rent + flat**        | TX § 92.0081                            | `1 month rent + $1,000 + actual + fees`                              |
| **Greater-of rent-mult or actual** | FL § 83.67 / WA § 59.18.290 / MA c.186 §14 | `max(3× rent, actual) + fees`                                  |
| **Greater-of rent-mult or actual-mult** | AZ § 33-1367                    | `max(2× rent, 2× actual) + fees`                                     |
| **Greater-of rent-mult or flat, plus actual** | CO § 38-12-510            | `max(3× rent, $5,000) + actual + fees`                              |
| **Treble actual**               | NY RPL § 235 / DC § 42-3505.01 / NJ § 2A:39-1 | `3× actual + fees` (+ criminal exposure in NY/NJ)             |
| **Rent multiple + actual**      | IL Forcible Entry Act                   | `2× rent + actual + fees`                                            |
| **Actual damages only**         | ~35 other states                        | actual + fees (where statute provides)                               |
| **No statute**                  | AR / WV / WY / MS / SD / ID             | actual common-law trespass damages only                              |

**CO § 38-12-510 is the strictest state in the table** — the deliberate 2021 HB21-1121 reform set a `max(3× rent, $5,000)` floor PLUS actual damages, so even a tenant with zero actual damages on a $1,000/month unit recovers $5,000 (the flat floor wins over $3,000 = 3× rent). Pinned by `colorado_strictest_state_three_times_rent_or_5k_plus_actual` covering both low-rent floor-wins and high-rent multiplier-wins.

**Three states have statutory floors that survive all-zero input**: CA ($250 per-day minimum), TX ($1,000 flat), CO ($5,000 flat). Every other regime zeros out cleanly. Pinned by `zero_rent_zero_actual_zero_days_no_panic` which sweeps all three positive-floor states against the negative-control states (FL/WA/NY/AZ/IL/MA) — catches a future regression where someone moves the floor logic out of CA/TX/CO into a shared helper.

**Treble model has no statutory floor independent of actual damages**: 3× 0 = 0. Distinct from the per-day model. Pinned by `treble_with_zero_actual_yields_zero_total`. This matters at the UI layer — telling a landlord "you owe at least 3× actual" is misleading if actual is zero (e.g., immediate self-cure before any tenant outlay).

**Criminal exposure flag** is independent of civil regime. NY, NJ, MA, GA, MI, NH, CT, LA, MO, NV, PA all flag `criminal_exposure: true` — the landlord faces misdemeanor / criminal trespass charges separate from the civil suit. Pinned by `criminal_exposure_states_pinned`. The flag is a downstream UI signal, not used in the dollar compute.

**Arkansas is uniquely `NoStatute`** — the only US state with effectively no tenant protections on self-help. Pinned by `arkansas_uniquely_no_statute_landlord_friendly`.

**Integer cents + saturating_mul throughout** — large rent / large day count never overflow i64. Pinned by `large_rent_no_overflow_via_saturating_mul` with $10 billion rent against the CO 3× regime, which still clamps under i64::MAX.

Mounted at `POST /api/rental/lockout-penalty-check`. Twenty-five tests pin: 51-row coverage; **CA per-day floor cross-over math** (1 day / 2 days / 3 days / 5 days — floor wins until day 3); CA zero-days still returns $250 minimum; TX additive 1mo rent + $1k + actual; FL greater-of with both directions (actual wins, statutory wins); WA mirrors FL with distinct citation noting separate utility-shutoff statute; AZ both-sides-multiplied; **CO strictest with floor-wins and multiplier-wins both pinned**; treble states pinned at 3× actual; treble × 0 = 0; IL 2× rent additive; MA mirrors FL but criminal flag set; actual-damages-only states pass through; no-statute states (AR/WV/WY/MS/SD/ID) yield actual only with AR uniquely flagged; unknown state errors; case-insensitive lookup; sorted `all_states()`; non-empty citations; attorney_fees_recoverable flag pinned across 9 strong fee-shifting states; criminal_exposure flag pinned across 11 states; arkansas no-statute uniquely; all-zero input no panic; **large-rent no overflow**; TX zero-actual still has $1k+rent floor; greater-of equal amounts no double-count.

`traderview-expense::application_fees` is the **state-specific rental application / tenant-screening fee cap table** — sibling to `entry_notice`, `retaliation_windows`, `eviction_notices`, `late_fee_caps`, `deposit_interest`, `deposit_return_windows`, `lease_disclosures`, `habitability_remedies`, `rent_control`, `military_termination`, `security_deposit_caps`, and `contractor_1099`. Application fees are the highest-frequency landlord-tenant transaction and increasingly regulated — NJ AG put landlords on notice in 2024, and CA's CPI-adjusted cap moves every December.

Six regulatory regimes plus no-statute:

| Regime                                | States                                                                  | Citation example                   |
|---------------------------------------|-------------------------------------------------------------------------|------------------------------------|
| **Prohibited**                        | MA, VT                                                                  | M.G.L. c. 186 § 15B / 9 V.S.A. § 4456a |
| **Flat cap**                          | NY $20 / VA $50 / WI $20 / DC $50 / NJ $50                              | RPL § 238-a (HSTPA 2019)           |
| **CPI-adjusted cap + actual-cost limiter** | CA $65.86 (2026) — lesser-of cap-or-actual-cost                       | Cal. Civ. Code § 1950.6            |
| **Lesser-of-cap-or-actual-cost**      | MD $25 cap                                                              | Md. Code Real Prop. § 8-213        |
| **Greater-of-flat-or-pct-of-rent**    | DE: greater of $50 or 10% of monthly rent                               | 25 Del. C. § 5514(d)               |
| **Actual cost only (no markup)**      | WA / OR                                                                 | RCW § 59.18.257 / ORS § 90.297     |
| **No statute**                        | 38 other states                                                         | n/a                                |

**Integer cents arithmetic is load-bearing.** Money is `i64` cents throughout, not `Decimal` — CA's $65.86 is exactly `6586_i64` with no precision drift, and DE's `10% × monthly_rent_cents` uses `saturating_mul + 10_000` floor-divide. The `delaware_pct_rounds_down_at_uneven_rent` test pins this: rent of $1,234.56 (123_456¢) → 10% = 12,345.6¢ → integer floor 12,345¢. A future "fix" that converts to `Decimal` rounds halfway-banker's-round upward and would silently change the compute on uneven rents.

**Prohibited vs no-statute distinction matters.** MA tenant can sue for the prohibited fee; TX tenant cannot — landlord just charges what they want. Both come back as "non-compliant" if the proposed fee is nonzero, but `prohibited` flag and `no_statute` flag are mutually exclusive on the result. Downstream UI must distinguish them. Pinned by `massachusetts_prohibited_flag_distinct_from_no_statute`.

**California CPI cap moves annually.** The $65.86 value is the 2026 figure (per the California Apartment Association December 2025 announcement). Callers must refresh this annually — the citation field flags `cpi_adjusted: true` and the result note spells out "CPI-adjusted" so a stale value is visible at the UI layer.

**Actual-cost-only states require the actual cost.** WA and OR fail closed if `actual_screening_cost_cents` is `None` — compliance can't be determined without the third-party screening invoice. The `actual_cost_required_but_missing` flag on the result tells the caller to ask the user for the cost rather than silently approving the fee.

Mounted at `POST /api/rental/application-fee-check`. Twenty-three tests pin: 51-row coverage; MA prohibits any fee ($0 OK, any ¢ violates); VT prohibition with distinct citation; NY $20 exact-boundary + $20.01 over; **CA 2026 CPI cap with both actual-cost-provided and not-provided paths** (load-bearing dual-mode test); MD lesser-of $25-or-actual-cost; **DE greater-of $50-or-10%-rent at low rent / high rent / 1-cent-over boundary**; DE rounds-down at uneven rent (123_456¢ → 12_345 cap, not 12_346); WA actual-cost-only with cost provided; WA `actual_cost_required_but_missing` flag set when missing; OR mirrors WA with distinct citation; no-statute states (TX/FL/CO/AZ/GA) accept any fee; unknown state returns no-statute with error note; case-insensitive lookup; sorted `all_states()`; non-empty citations on every row; **flat-cap states (NY/VA/WI/DC/NJ) pinned at exact boundary AND boundary+1** (catches future cap typos); MA prohibited vs TX no-statute flags mutually exclusive; CA without actual-cost uses CPI cap with informative note; DE 0¢ rent falls back to $50 minimum (concession-period regression target); $0 proposed fee always complies in every Cap-rule state; CA note spells out CPI + actual-cost-limiter when both apply; negative proposed fee clamps excess to 0 (input-sanity stability).

`traderview-expense::retaliation_windows` is the **state-specific anti-retaliation rebuttable-presumption-window table** — sibling to `eviction_notices`, `entry_notice`, `late_fee_caps`, `deposit_interest`, `deposit_return_windows`, `lease_disclosures`, `habitability_remedies`, `rent_control`, `military_termination`, `security_deposit_caps`, and `contractor_1099`. Pins the burden-shift window: when a landlord initiates an adverse action (eviction filing, rent increase, service reduction, refusal to renew) within the statutory window after a tenant exercises a protected right, a rebuttable presumption arises that the action is retaliatory and the burden shifts to the landlord to show a legitimate non-retaliatory reason.

The presumption windows cluster into three bands plus three special categories:

| Band                          | States                                                       |
|-------------------------------|--------------------------------------------------------------|
| **90 days** (URLTA short)     | AK / DE / MI / MN / VT / WA                                  |
| **6 months / 180 days**       | AL / AZ / CA / CO / CT / DC / KS / MA / MD / ME / MT / NE / NH / NM / NV / OR / PA / RI / SC / TN / TX / WI |
| **1 year**                    | HI / IA / IL / KY / NC / NY / VA                             |
| Statute, **no presumption period** (burden stays on tenant) | FL § 83.64 / OH § 5321.02 / OK 41 O.S. § 101 |
| Statute, **case-by-case window** (NJ Anti-Reprisal Act)     | NJ                                            |
| **No statute** at all         | AR / GA / ID / IN / LA / MO / MS / ND / SD / UT / WV / WY    |

**Days vs Months matters.** "180 days" is exact-day math; "6 months" is calendar-anchored and varies 181-184 days depending on which months the window spans. The `PresumptionPeriod::Days(n)` vs `Months(n)` distinction is load-bearing: e.g. CA uses `Days(180)` while TX uses `Months(6)`. For a complaint on 2026-08-31, CA's window ends 2027-02-27 (exact day 180) but TX's ends 2027-02-28 (calendar 6 months, using `chrono::Months::checked_add` for short-month rollover). A regression that "simplifies" `Months(6)` to `Days(180)` would silently misclassify the TX 8/31 → 2/28 case as outside the window. The `months_path_uses_calendar_anchored_window_not_180_days` test pins this against future drift.

**Action-before-complaint short-circuits.** If the landlord's action precedes the tenant's complaint, no causal link exists and the presumption cannot apply regardless of state. The `landlord_action_before_complaint_cannot_be_retaliation` test pins this — `days_since_complaint` is negative and the result is non-presumption with a note explaining the missing causal link.

Mounted at `POST /api/rental/retaliation-check`. Twenty tests pin: 51-row coverage; case-insensitive lookup and unknown-state handling; CA 180-day exact boundary inside / 181 outside; WA 90-day window cluster boundary; TX 6-month calendar-anchored window with 8/31 → 2/28 short-month rollover (the load-bearing month-vs-day case); IL 12-month band 11-month inside / 13-month outside; FL statute-no-presumption-period burden-on-tenant flag; NJ case-by-case neither presumption nor burden-on-tenant; no-statute states (12 jurisdictions) all flagged correctly; landlord-action-before-complaint cannot be retaliation; day-zero same-day action within window (hostile-landlord scenario); unknown state code marked no-statute; **90-day band states uniformly reject day 91**; **6-month band states uniformly reject month 7**; **12-month band states uniformly hold at month 11** (three-way band consistency catches future cell-edit errors); `all_states()` returns sorted by code; every row has non-empty citation; calendar-anchored window math distinct from 180-day math; note describes burden-on-landlord vs days-past explicitly when inside vs outside window.

`traderview-expense::entry_notice` is the **state-specific landlord entry-notice hour-count table** — sibling to `eviction_notices`, `late_fee_caps`, `deposit_interest`, `deposit_return_windows`, `lease_disclosures`, `habitability_remedies`, `rent_control`, `military_termination`, `security_deposit_caps`, and `contractor_1099`. Closes the privacy-rights leg of the landlord-state-data set: every other module covers money or eviction, this one covers when the landlord is allowed in the door.

Each state's landlord-tenant code sets a minimum advance-notice period before non-emergency entry (repairs, inspection, showing to prospects). The hour count clusters into four bands:

| Band                   | Hours | States                                                       |
|------------------------|-------|--------------------------------------------------------------|
| URLTA-2015 default     | 24    | AK / CA / CO / IA / ME / MT / NE / NV / NM / OH / OK / OR / SC / SD / TN / UT / VA |
| URLTA-1974 default     | 48    | AL / AZ / DE / DC / HI / KY / RI / VT                        |
| Strictest (repairs)    | 48    | WA (with separate 24h column for showings to prospects)      |
| Aggressive carve-out   | 12    | FL (Fla. Stat. § 83.53) / WI (ATCP § 134.09)                |
| No statutory hours     | n/a   | AR / CT / GA / ID / IL / IN / KS / LA / MA / MD / MI / MN / MO / MS / NC / ND / NH / NJ / NY / PA / TX / WV / WY (common-law "reasonable" applies but is not measurable in hours) |

**The Washington split is load-bearing.** RCW 59.18.150 is the only statute in the table with a per-purpose carve-out: 48h required for repairs/inspection, 24h for showings to prospective buyers or replacement tenants. Every other state uses one column for all non-emergency purposes. The compute fn picks `showing_hours.or(standard_hours)` so the carve-out applies only when the showing column is explicitly set.

**Emergency, tenant-requested, and abandonment exceptions short-circuit to compliant in every state** — these are universal common-law exceptions, not per-state rules. Models them as `EntryPurpose` variants that bypass the hour-count check entirely with a labeled `exception` field on the result.

Mounted at `POST /api/rental/entry-notice-check`. Twenty-two tests pin: 51-row coverage (50 states + DC); case-insensitive state lookup; unknown state errors; emergency entry compliant at 0h notice in every state including WA-48h; tenant-requested entry compliant at 0h; abandoned-unit entry compliant at 0h; CA 24h exactly compliant; CA 23h one hour short (exact shortfall reporting); FL 12h minimum + 11h one-hour-short; **WA 48h repairs but 24h showings split** (the only per-purpose carve-out); URLTA-1974 states (AL/AZ/KY/HI/RI/VT) all default to 48h; no-statute states report compliant at 0h (no measurable standard); "reasonable" states (CT/MN/KS) treated like no-statute; unknown-state error reported; `all_states()` returns sorted by code (first AK, last WY); every row has non-empty citation; showings fall back to standard column when no carve-out (CA showing = 24h); inspections use the standard column (OH 24h); 0h notice fails in every hour-count state; excess notice (96h in CA) still compliant with 0 shortfall; FL 12h is distinct from 24h default (18h compliant in FL/WI, not in AL); emergency short-circuits even for unknown state code (lookup-first ordering).

`traderview-expense::eviction_notices` is the **state-specific eviction-notice period table** — sibling to `late_fee_caps` and `deposit_interest`. Each state's landlord-tenant statute sets a minimum notice period before the landlord can file for possession, varying dramatically by ground:

- **Pay or Quit** (nonpayment): TX 3 days, FL 3 days, CT 3 days vs MA 14 days, NY 14 days, WA 14 days (post-2019), CO 10 days (post-2021), DC 30 days, NJ — no pay-or-quit notice required (landlord files directly after 5 business days late).
- **Cure or Quit** (curable lease breach): typically 7-30 days; tenant has the right to fix the breach and avoid eviction.
- **Unconditional Quit** (non-curable, e.g. illegal activity): typically 3-14 days; no cure right.
- **No-Cause Termination** of month-to-month: many states **scale by tenancy length** — CA 30 days under 1 year / 60 days at 1+ year (CCP §1946.1); NY 30/60/90 days under HSTPA; OR 30 days under 1 year / 90 days at 1+ year with just cause; PA 15/30 days. NJ has effectively **eliminated** no-cause for residential tenants under the Anti-Eviction Act; WA largely eliminated under the 2021 Just Cause Act.

25 jurisdictions modeled (AL/AZ/CA/CO/CT/DC/FL/GA/IL/MA/MD/MI/MN/NC/NJ/NY/OH/OR/PA/SC/TN/TX/VA/WA/WI), each with statute citation + URL + notes calling out the load-bearing carve-outs (CA AB1482 just cause for >12-month tenancies in multi-unit; CO HB21-1121 pay-or-quit raise from 3 to 10 days; NY HSTPA 14-day pay-or-quit and 90-day no-cause for ≥ 2-year tenancies; WA 2021 just-cause statewide). Unknown states return `state_recognized: false` with a "consult state code directly" note rather than guessing.

Mounted at `POST /api/rental/eviction-notice-check`. Eighteen tests pin: TX 3-day pay-or-quit; NY 14-day pay-or-quit post-HSTPA; NY no-cause scales to 90 days at 2 years; NY no-cause 30 days at 6 months; CA no-cause 30→60 at 1 year boundary; CA 3-day pay-or-quit with just-cause flag; CO post-2021 10-day pay-or-quit; WA post-2019 14-day pay-or-quit + just-cause flag; NJ no pay-or-quit notice required; NJ no-cause unavailable (Anti-Eviction); GA no pre-filing pay-or-quit; DC 30 days for all four grounds; unknown state returns not-recognized; case-insensitive lookup; just-cause jurisdictions flagged correctly (CA/DC/NJ/OR/WA); non-just-cause states not flagged (TX/FL/AZ/AL); citation correctness for CA/TX/NY; OR no-cause scales to 90 days at 18 months.

`traderview-expense::late_fee_caps` is the **state-specific late-fee-cap + grace-period table** — sibling to `deposit_interest`. The existing `rental_leases.late_fee_fixed` and `late_fee_pct` columns on iter 1's migration accepted any value with no statute check; this module fills the gap. 17 jurisdictions with statutory caps or grace requirements:

| State | Cap formula                | Cap value          | Grace |
|-------|----------------------------|--------------------|-------|
| CA    | Reasonable (case law)      | ~6% bound          | 0     |
| CO    | Greater of $50 or 5%       | $50 / 5%           | 7     |
| CT    | Reasonable                 | none               | 9     |
| DC    | 5% of monthly rent         | 5%                 | 5     |
| DE    | 5% of monthly rent         | 5%                 | 5     |
| MA    | Reasonable (30-day delay)  | none               | 30    |
| MD    | 5% of monthly rent         | 5%                 | 0     |
| ME    | 4% of monthly rent         | 4%                 | 15    |
| MN    | 8% of overdue rent         | 8%                 | 0     |
| NC    | Greater of $15 or 5%       | $15 / 5%           | 5     |
| NJ    | Reasonable                 | none               | 5     |
| NV    | 5% of monthly rent         | 5%                 | 0     |
| NY    | **Lesser** of $50 or 5%    | $50 / 5%           | 5     |
| OR    | Reasonable (~5%)           | ~5% bound          | 4     |
| TX    | 12% safe harbor (1-4 unit) | 12%                | 2     |
| VA    | 10% of past-due rent       | 10%                | 0     |
| WA    | Reasonable                 | none               | 5     |

Each row carries `Citation { statute, source }` — the published statute reference + a URL to the canonical text. Caller-facing fields on `LateFeeCheckResult`: `state_has_specific_cap`, `max_fee_permitted`, `compliant`, `grace_days_required`, `grace_violation`, `reasonableness_review_required` (true for ReasonableOnly states that have no bright-line cap), plus the statute + source + notes for audit display. Local ordinances (Chicago, Seattle, NYC) frequently impose stricter caps than state law — caller's responsibility to layer those on top.

Mounted at `POST /api/rental/late-fee-check`. Eighteen tests pin: NY uses lesser of $50 or 5% (at $2k rent → $50 cap, not $100); NY proposed above $50 not compliant; NY grace violation at 3 days < 5; NC greater of $15 or 5% — $50 at $1k rent, $15 at $200 rent; CO $50 floor at $500 rent, 5% at $5k rent; MA 30-day grace blocks early fee; MN 8%; TX 12% safe harbor; CA reasonableness flag (compliant=true with `reasonableness_review_required=true`); unknown state defaults to "no statutory cap" + reasonableness review; case-insensitive lookup; citation correctness for MD/NY/TX; grace satisfaction at exactly the grace day; VA 10%; ME 4% with 15-day grace; NY at 3% with $1k rent under both caps.

`traderview-expense::military_termination` is the **federal SCRA + state military lease termination table** — the tenth state-data module after `deposit_interest`, `late_fee_caps`, `eviction_notices`, `contractor_1099`, `deposit_return_windows`, `lease_disclosures`, `rent_control`, `habitability_remedies`, and `security_deposit_caps`. Landlords near military bases (Fort Cavazos, Camp Pendleton, Fort Bragg, Norfolk) routinely encounter this.

**Federal Servicemembers Civil Relief Act** (50 USC §3955) applies in every state. An active-duty servicemember may terminate any residential lease for any of three **qualifying events**:

1. **Permanent change of station** orders (PCS).
2. **Deployment ≥ 90 days** with their unit.
3. **Active duty after lease signing** (entry from reserve or new enlistment).

Mechanics: written notice with copy of orders → termination effective on **the next rent-due date 30+ days after notice**. Landlord cannot charge an early-termination fee. **Civil penalty up to $55,000** for first violation, $110,000 thereafter (15 USC §15), plus tenant's actual damages and equitable relief.

State law layers ADDITIONAL protections on top of the federal floor. 11 states modeled (CA, NY, TX, FL, VA, WA, IL, CO, NJ, NC, PA) with the following extension dimensions:

- **Spouse termination right** — CA, NY, TX, FL, VA, IL, CO, NC. Civilian spouse may terminate when the servicemember PCSs or deploys.
- **Dependent termination right** — CA, TX, VA, NC. Extends to dependents living with the servicemember.
- **First-responder termination right** — TX only (Tex. Prop. Code §92.017 unique extension to peace officers, firefighters, EMS reassigned ≥ 50 miles).
- **Modified notice days** — most states match federal 30; future expansions may use shorter periods.

When neither federal SCRA nor state extension applies, `termination_right_available` returns false (e.g. spouse in WA, where state recognizes federal SCRA but doesn't extend to spouses, and federal doesn't extend to spouses). `controlling_authority` returns "50 U.S.C. §3955 (SCRA)" when federal qualifies, "state extension (statute)" otherwise.

Mounted at `POST /api/rental/military-termination-check`. Twenty-one tests pin: federal SCRA PCS servicemember qualifies; federal deployment ≥ 90 days qualifies; federal active duty after signing qualifies; CA spouse extension applies (federal doesn't extend); CA dependent extension applies; **TX first responder extension is unique** (only TX extends); CA first responder NOT extended (unlike TX); NJ codifies SCRA but no extra extensions (spouse doesn't qualify); NJ servicemember PCS federal still applies; unknown state federal SCRA still applies (returns `state_recognized: false`); unknown state + spouse role no termination right; case-insensitive state lookup; NC extends to spouse + dependent; NY extends to spouse only (not dependent); FL spouse extension; VA dependent extension; WA servicemember PCS qualifies under federal; WA spouse no state extension no federal no right; citation present for known states; controlling authority prefers federal when applicable; controlling authority falls to state extension for spouse.

`traderview-expense::security_deposit_caps` is the **state security deposit maximum amount table** — the ninth state-data module after `deposit_interest`, `late_fee_caps`, `eviction_notices`, `contractor_1099`, `deposit_return_windows`, `lease_disclosures`, `rent_control`, and `habitability_remedies`. Most states cap how much a landlord can require as a security deposit; collecting in excess voids the excess + may trigger statutory penalties (MD 3× excess + attorney's fees the strictest).

14 states modeled (CA, NY, MA, NJ, VA, DC, MD, NV, OR, MI, IA, DE, KS, IL). Cap range from **0 months (no statutory cap — IL state level)** to **3 months (NV — highest in country)**. The applicable cap is the maximum of three potential values:

- **Base months_rent cap** — standard cap (CA 1 month, NY 1 month, NJ 1.5 months, VA 2 months, etc.).
- **Furnished cap** — KS and OR permit a higher cap for furnished units (KS 1.5 vs 1 base; OR 1.5 vs 1 base).
- **Small landlord cap** — CA AB12 (effective July 2024) maintains the prior 2-month cap for natural-person or natural-person-owned-LLC landlords with ≤ 2 rental properties totaling ≤ 4 dwelling units. Lets small operators continue charging up to 2 months while large operators are limited to 1 month.

Applicable cap precedence: **small landlord > furnished > base**. A small-landlord-and-furnished property in CA gets the small-landlord 2-month cap (higher of the two carve-outs wins). The compliance check returns `compliant: true` when `proposed_deposit_amount ≤ max_permitted` (or no statutory cap exists in the state); `excess_amount` reports the dollar overage when above cap.

Mounted at `POST /api/rental/security-deposit-cap-check`. Twenty tests pin: CA AB12 1-month cap for standard landlords; CA proposed $4k (2 months) not compliant for standard landlord; **CA small-landlord 2-month cap allowed**; NY 1-month cap post-HSTPA; MA 1-month security deposit only (note mentions first + last separately allowed); NJ 1.5-month cap; NV 3-month cap (highest in country); OR furnished carve-out (1.5 furnished vs 1 unfurnished); KS furnished 1.5 vs unfurnished 1; **small-landlord priority over furnished in CA** (small landlord 2 months wins); IL no state cap returns `state_has_cap: false`; DE long-term 1-month cap with > 1 year note; unknown state not recognized; case-insensitive lookup; MD 2-month cap with 3× penalty note; proposed deposit at exactly cap compliant; excess amount calculated correctly ($5k − $2k = $3k); citation present for known states; `rule_for` helper returns citation.

`traderview-expense::habitability_remedies` is the **state habitability warranty + tenant-remedy table** — the eighth state-data module after `deposit_interest`, `late_fee_caps`, `eviction_notices`, `contractor_1099`, `deposit_return_windows`, `lease_disclosures`, and `rent_control`. When a landlord fails to maintain habitable conditions (broken heat in winter, leaking roof, code violations, vermin), state landlord-tenant codes grant tenants one or more remedies under the **implied warranty of habitability**:

- **Repair-and-deduct** — tenant fixes + deducts cost from rent up to statutory cap. CA = one month's rent (CC §1942); TX = one month's rent (Tex. Prop. Code §92.0561, 7-day notice + 7-day cure); WA = one month's rent (RCW 59.18.100, 10-day cure); IL Chicago RLTO = $500 fixed; OR = $300 fixed; VA = greater of $1,500 or one month's rent (Va. Code §55.1-1245).
- **Rent withholding into escrow** — tenant continues to pay rent into a court-supervised escrow account until landlord cures. Common in MA (G.L. c.111 §127L), WA (RCW 59.18.115), NJ (Marini v. Ireland), VA (§55.1-1244), FL (court registry per §83.60), NY (RPL §235-b). Withholding rent OUTRIGHT (rather than into escrow) is illegal in most states and grounds for nonpayment eviction.
- **Lease termination** — tenant breaks lease without penalty after notice + cure. Available in URLTA states (TX, OR).
- **Damages action** — tenant sues for actual + statutory damages. CA §1942.4 awards up to **$5,000 + attorney's fees** when landlord demands rent on a substandard property. MA c. 93A treble damages plus attorney's fees the highest in the country.
- **Eviction affirmative defense** — habitability raised in defense to landlord's nonpayment eviction. Universally available in URLTA states + jurisdictions adopting the *Pugh v. Holmes* / *Green v. Superior Court* / *Park West v. Mitchell* line.

10 states modeled (CA, TX, NY, IL, WA, FL, MA, NJ, VA, OR) with **18 distinct state×remedy combinations**. The compute function returns the full list of available remedies for the state with each one's `notice_days_required` + `cure_period_days` + `repair_deduct_cap_dollars` (computed from monthly_rent × months_cap OR greater-of-fixed-and-months formula) + `damages_multiplier` + `attorney_fees_to_prevailing_tenant` flag.

Mounted at `POST /api/rental/habitability-remedies`. Nineteen tests pin: CA has 3 remedies modeled (repair-and-deduct + damages action + eviction defense); CA repair-and-deduct caps at one month's rent ($2,500 at $2,500 rent); TX repair-and-deduct one month rent with 7-day notice + 7-day cure; IL Chicago RLTO repair-and-deduct $500 fixed cap; **VA greater-of-$1,500-or-month-rent** math ($1,000 rent → $1,500 cap; $2,000 rent → $2,000 cap); MA treble damages multiplier for c.93A action; NY has withholding + eviction defense but NO state-wide repair-and-deduct; FL rent into court registry after 7-day notice; OR has repair-and-deduct + termination with attorney fees; unknown state not recognized; case-insensitive lookup; WA repair-and-deduct + withholding both modeled with different cure periods (10 vs 30 days); CA damages action $5,000 cap + attorney fees; states with attorney fees correctly flagged (TX/IL/OR/WA); CA eviction defense no notice required; report total count matches list length; `remedies_for_state` helper returns all matching rules; citation present for known states.

`traderview-expense::rent_control` is the **state rent control / rent stabilization table** — the seventh state-data module after `deposit_interest`, `late_fee_caps`, `eviction_notices`, `contractor_1099`, `deposit_return_windows`, and `lease_disclosures`. Three classes of state law govern annual rent increases:

- **Statewide rent cap** — CA (AB1482, 2019), OR (SB608, 2019, first-in-nation), WA (HB1217, effective 2025). Cap formula = fixed percentage + local CPI, **absolute max 10%**. CA = 5% + CPI; OR = 7% + CPI; WA = 7% + CPI. All three require **just cause termination after 12 months** of tenancy. Common exemptions: **new construction under 15 years old** (all three states), **single-family non-corporate-owned** (CA only — OR/WA don't exempt), **owner-occupied 2-4 unit buildings** (CA only).
- **Local stabilization permitted** — NY (NYC Rent Guidelines Board + ETPA), NJ (Newark/Jersey City/Hoboken + 100+ municipalities), MD (Takoma Park), MN (Saint Paul 3% cap voted 2021), DC (Rental Housing Act of 1985 covers buildings 1975 and earlier at CPI + 2%). No statewide cap — caller responsible for the applicable local ordinance.
- **State preemption** — TX (Loc. Gov't Code §214.902), FL (§125.0103), AZ (§33-1329), GA (§44-7-19), TN (§66-35-102), IL (50 ILCS 825). Local rent control prohibited; rent fully market-rate subject only to lease terms + notice rules.

Exemption priority order: **single-family non-corporate** → **owner-occupied 2-4 unit** → **new construction** → **tenancy < 12 months**. First match wins and short-circuits the cap calculation. When no exemption applies, the cap = `MIN(fixed_pct + local_cpi_pct, absolute_max_pct)`. Maximum permitted new rent = `current_rent × (1 + max_permitted_pct)`. `just_cause_required` flag fires only when state has just-cause AND tenancy ≥ 12 months (handles year-1 carve-out correctly).

Mounted at `POST /api/rental/rent-increase-check`. Twenty-four tests pin: CA 5% + 3% CPI = 8% cap (5% proposed compliant); exactly-at-cap compliant; over-cap not compliant; high CPI caps at 10% absolute (5% + 8% = 13% raw → 10%); CA single-family non-corporate exempt; CA owner-occupied 2-4 unit exempt; CA new construction (year_built 2020, age 4) exempt; CA old construction (1990) not exempt; CA tenancy < 12 months exempt with explicit note; OR 7% + 3% CPI = 10% cap; **OR single-family NOT exempt** (unlike CA — pinned with explicit naming); WA 7% + CPI capped at 10%; TX preemption no cap (150% increase compliant); FL preemption no cap; NY local stabilization permitted state returns compliant without cap check; just-cause required for CA after 12mo; just-cause not required under 12mo; just-cause not required in preemption states; unknown state returns not_recognized; case-insensitive state lookup; proposed-increase-pct calculated correctly; CA exemption priority (single-family beats year-built check); DC local stabilization with just-cause flag; citation correctness for CA/OR/TX.

`traderview-expense::lease_disclosures` is the **lease disclosure requirements table** — the sixth state-data module after `deposit_interest`, `late_fee_caps`, `eviction_notices`, `contractor_1099`, and `deposit_return_windows`. Two layers of mandatory disclosures: **federal Title X lead-paint** (42 USC §4852d + 24 CFR 35) — required in EVERY state for target housing built before 1978, with civil penalty up to **$19,507/violation** (2024 inflation-adjusted) — plus **state-specific disclosures** (mold, bedbug, sex offender / Megan's Law, radon, asbestos, methamphetamine, truth-in-renting handbook, foreclosure proceedings, demolition permits, smoking policy).

12 `DisclosureType` variants modeled with state-by-state `DisclosureRule` rows. A property's required-disclosure list is computed from `PropertyFacts` (year_built, landlord_in_foreclosure, known_lead_hazard, known_mold_history, known_bedbug_history_12mo, known_meth_contamination, demolition_permit_pending) by intersecting the state's rules with the facts. Three categories of rule behavior:

- **Unconditional disclosures** — always required when the state has the rule (e.g. NJ Truth in Renting Act, FL radon §404.056, CA Megan's Law §2079.10a, WA mold info per RCW 59.18.060(13), AZ bedbug educational info per A.R.S. §33-1319). These appear in the result regardless of the property facts.
- **Year-gated disclosures** — required only when `year_built` is before a threshold (federal lead paint < 1978; CA asbestos §1102.6e < 1981; MD adds a state-specific lead paint cert on top of federal for pre-1978). Properties with unknown `year_built` skip year-gated disclosures (caller asserts if uncertain — safer to err to non-required than to false-positive a list of legal obligations).
- **Conditional disclosures** — required only when a corresponding fact is asserted (CA mold §26147 only when `known_mold_history: true`; CA/NY/ME bedbug only when `known_bedbug_history_12mo: true`; CA demolition permit §1940.6 only when `demolition_permit_pending: true`; CA/OR foreclosure notice only when `landlord_in_foreclosure: true`).

The `*` sentinel in the state column marks federal rules that fire for every state — currently used only for Title X. Stacking is intentional: a pre-1978 MD property hits the federal Title X lead-paint disclosure AND the state-specific MD lead-paint inspection certificate (Md. Real Prop. §8-208.2), producing TWO `LeadPaint` entries in the result so the user knows both compliance obligations exist.

Mounted at `POST /api/rental/lease-disclosures-required`. Twenty-one tests pin: pre-1978 property in any state requires federal lead paint with $19,507 penalty; **boundary at exactly 1977 = pre-1978** (required) vs 1978 (not required); post-1978 in unmodeled state returns empty list; CA post-1978 still has unconditional Megan's Law disclosure; CA with mold history adds mold rule; CA without mold history skips it; WA mold unconditional; FL radon unconditional; CA asbestos for pre-1981 / not for post-1981; CA foreclosure when landlord_in_foreclosure ($2,000 penalty); NJ Truth in Renting unconditional with $500 penalty; case-insensitive state lookup; **MD pre-1978 adds state lead paint atop federal** (2 LeadPaint entries); conditional bedbug only when history known; AZ bedbug unconditional; OR smoking policy unconditional; OR foreclosure conditional; CA demolition permit only when pending; unknown year_built skips year-gated disclosures.

`traderview-expense::deposit_return_windows` is the **state security deposit RETURN window table** — sibling to `deposit_interest`, `late_fee_caps`, `eviction_notices`, and `contractor_1099`. Every state has a statutory window (14-45 days) for returning the security deposit + itemized deduction statement after the tenancy ends. Missing the deadline frequently triggers automatic forfeiture of the right to withhold + a bad-faith damages multiplier (MA 3× plus attorney's fees and 5% interest is the strictest in the country; TX 3× + $100 + fees; CO/DC/GA/MD 3×; CA/CT/IL/MI/MN/NJ/NV/NY/OH/OR/PA/WA 2×; AZ 2× + fees; FL/NC/VA 1× / no statutory multiplier).

22 jurisdictions modeled (AZ, CA, CO, CT, DC, FL, GA, IL, MA, MD, MI, MN, NC, NJ, NV, NY, OH, OR, PA, TX, VA, WA). Each `StateReturnRule` carries: `return_window_days` (statutory), `business_days_basis` (true for AZ which counts business days only), `itemized_statement_required`, `bad_faith_damages_multiplier`, `attorney_fees_to_prevailing_tenant`, plus `Citation { statute, source URL }` to the published text. Window math: `compliant = days_elapsed ≤ required` (the statutory day itself is compliant — landlord returning on exactly day 21 in CA is fine, day 22 is one day late).

`max_penalty_exposure` is computed as `wrongful_withholding × multiplier` only when `bad_faith_alleged: true` — a good-faith dispute over actual damages doesn't trigger the multiplier even if the withholding is later found wrongful. `attorney_fees_at_risk` flags only when BOTH `bad_faith_alleged` is true AND the state statute awards fees to the prevailing tenant.

Mounted at `POST /api/rental/deposit-return-check`. Eighteen tests pin: CA 21-day window compliant at day 21 / one day late at day 22; TX 30-day + 3× multiplier ($1,500 wrongful × 3 = $4,500 exposure) + attorney fees flag; MA 30-day with 3× (strictest — note text contains "triple"); NY 14-day post-HSTPA compliant at day 13 / one day late at day 15; FL 15-day shortest window for no-deductions path; VA 45-day longest among modeled; no-bad-faith → zero penalty exposure regardless of lateness; unknown state returns `state_recognized: false`; case-insensitive state lookup; CO 3× + attorney fees; FL 1× (no statutory penalty escalation); citation correctness for MA/TX/CA; return-before-tenancy-end no panic + compliant; 10 attorney-fee states flagged correctly; 7 non-attorney-fee states flagged correctly; good-faith withholding in attorney-fee state → no attorney_fees_at_risk.

`traderview-expense::deposit_interest` is the **state-specific security-deposit-interest table** for the 13 jurisdictions (CT, DC, FL, IA, IL, MA, MD, MN, NH, NJ, NY, PA, RI) that have a security-deposit-interest statute. Each row carries the statutory citation + source URL, the published annual rate (where statute fixes one — MN 1%, MD 1.5% min, CT 1.45% as of 2024), the minimum holding period before interest accrues (PA 24mo, MA 12mo, NH 12mo, etc.), and a note covering carve-outs (IL requires interest only for buildings of 25+ units; NY/NJ/PA use the *actual* bank rate via the caller-supplied override). The 37 states without a requirement return `required: false` with empty citation. Nine tests pin: TX no-requirement, MN 1% full-year, MD 1.5% full-year, NY uses caller-supplied bank rate, PA's 24-month gate, case-insensitive state lookup, negative-window safety, citation correctness for CT/MD, and unknown-state-returns-None.

`traderview-expense::section_1045` is the **IRC §1045 QSBS rollover module** — direct companion to `section_1202`. §1202 caps the exclusion at 5 years of holding; §1045 plugs the gap for taxpayers who sell BEFORE the 5-year clock matures: a holder of QSBS held **more than 6 months** can **defer gain** by reinvesting proceeds into OTHER QSBS within **60 days** of the sale. The original's holding period **tacks onto** the replacement for the §1202 5-year clock — chaining sales through multiple rollovers eventually qualifies for full §1202 exclusion at $0 in deferred basis.

Mechanics: gain deferred = MIN(realized gain, replacement cost); boot received = sale proceeds net − replacement cost (when positive); replacement basis = replacement cost − gain deferred (carryover basis preserving the deferred gain); effective holding-period-start inherits the original's acquisition date. Disqualification routes the full gain to current-year recognition with a reason list (six tested paths: original not §1202-qualified, replacement not §1202-qualified, held ≤ 6 months, replacement after 60-day window, replacement before sale date, multiple-failure stack).

Mounted at `POST /api/calc/section-1045`. Seventeen tests pin: full-replacement no-boot full deferral; partial replacement triggers boot recognition; boot exceeds gain caps recognition at gain; held under 6 months disqualified; replacement after 60-day window disqualified; replacement before sale disqualified; original/replacement not QSBS-qualified disqualified; **boundary tests** — exactly 6 months (183 days) disqualified, just over 6 months (184 days) qualifies, exactly 60 days qualifies, 61 days disqualified; loss returns no-op; holding period tacks to original acquisition; replacement basis never negative under stress; replacement_value − replacement_basis == deferred_gain invariant; multi-disqualification lists all reasons.

`traderview-expense::section_1295` is the **IRC §1295 Qualified Electing Fund (QEF) election module** — the natural companion to `section_1296`. Both let a U.S. shareholder escape the punitive §1291 excess-distribution regime, but with different tradeoffs:

- **§1296 MTM** (iter 22) — annual mark-to-market reported as ordinary income/loss. Loss limited to "unreversed inclusions" (cumulative prior gain). Only available for marketable PFIC stock. Simpler; doesn't require the PFIC to cooperate.
- **§1295 QEF** (this module) — shareholder included as partner-equivalent: each year reports pro-rata share of the PFIC's ordinary earnings AND net capital gain per §1293(a), with **character PRESERVED** — capital gain stays LTCG. No deferred-interest charge. Better than §1296 when the PFIC generates LTCG-eligible gain (preferential rate), worse when it generates only ordinary income (same as §1296). Requires the PFIC to provide a **PFIC Annual Information Statement** — many PFICs don't.

The basis + previously-taxed-income (PTI) machinery is the non-obvious part. Each year-end:
- **Basis increases** by total inclusion per §1293(d)(1) — prevents double tax when the gain is eventually distributed.
- **PTI account** tracks cumulative prior inclusions minus prior PTI distributions. Year 1 PTI starts at 0.
- **Distributions consume PTI first** (excluded from gross income per §1293(c)), then excess becomes a regular dividend.
- **Basis decreases** by PTI distributions only (the dividend-tax bucket doesn't touch basis — it's already-taxed earnings flowing back to the shareholder).
- PTI and basis are both floored at zero.

Mounted at `POST /api/calc/section-1295`. Eighteen tests pin: year-1 inclusion preserves character (ordinary stays ordinary, LTCG stays LTCG); basis steps up by total inclusion; PTI account year-end equals total inclusion when no distribution; distribution fully absorbed by PTI no taxable dividend; distribution exceeds PTI excess taxable as dividend ($5k PTI pool + $8k dist = $5k PTI absorbed + $3k taxable); prior PTI carries into current year ($10k prior + $5k current pool absorbs $8k dist with $7k remaining); basis decreases by PTI distribution only; **basis doesn't decrease for taxable dividend portion** (the $3k dividend doesn't touch basis); multi-year chain basis + PTI evolve correctly; zero inclusion + zero distribution no-op; negative PFIC earnings treated as zero (§1293 includes only positives); character preserved (key advantage over §1296 ordinary-only); PTI never negative; basis never negative; note text distinguishes distribution vs no-distribution paths; ordinary-only PFIC still includes ordinary; LTCG-only PFIC still includes LTCG.

`traderview-expense::section_1296` is the **IRC §1296 PFIC mark-to-market election module** — every trader holding foreign ETFs (VWO, EWZ, EWJ-class international funds) or foreign ADRs without QEF status faces PFIC rules under §1297. The default §1291 regime is punitive: "excess distributions" are taxed at the HIGHEST historical marginal rate plus a deferred-interest charge computed back to acquisition. Most retail international ETF holders trip §1291 without realizing it.

§1296 offers an escape valve for **marketable PFIC stock**: elect mark-to-market and report unrealized appreciation as **ordinary income** each year. Gain is recognized at ordinary rates (no LTCG preference), but the punitive interest charge vanishes entirely. The non-obvious trap: MTM **losses** are deductible only up to the taxpayer's **unreversed inclusions** — the running cumulative MTM gain previously recognized. A first-year MTM loss with no prior inclusions is **suspended**: not deductible, doesn't carry forward as a future deduction, doesn't reduce basis. It just vanishes for tax purposes until future gains get clawed back.

Basis adjustments per §1296(b): increased by recognized MTM gain, decreased by deductible MTM loss only (suspended portion doesn't touch basis). Unreversed inclusions per §1296(d): increased by gain, decreased by deductible loss, never negative. Multi-year ledger: callers feed prior year's `adjusted_basis_year_end` and `unreversed_inclusions_year_end` into the next year's input.

Mounted at `POST /api/calc/section-1296`. Twelve tests pin: year-1 gain recognized as ordinary with basis step-up; year-1 loss with zero prior inclusions fully suspended (basis stays flat); year-2 loss absorbed by prior $2k inclusions; loss exceeding unreversed inclusions split into deductible + suspended (basis reduces only by deductible portion); no-MTM-change no-op; **multi-year chain** (gain → loss → gain) basis and unreversed-inclusions evolve correctly; first-year-loss-then-gain chain showing the suspended-loss-is-gone-forever economic reality (year-2 gain measured from the year-2-start basis, not the original cost basis); unreversed inclusions never negative under stress; exact-zero gain no-op; loss capped at full basis doesn't create negative basis; gain note describes inclusion with amounts; loss note distinguishes full-absorb vs partial-suspend.

`traderview-expense::section_1234` is the **IRC §1234 options character + holding-period rules module** — defines the taxation of every non-§1256 stock/securities option (covered calls, cash-secured puts, equity options that aren't broad-based-index). Trio with `section_1091` (wash sale) + `section_1233` (short sales) on the trader-anti-abuse side, and complement to `section_1256` for non-1256 equity options.

Three subsections govern:

| Subsection         | Applies to     | Result                                                                 |
|--------------------|----------------|------------------------------------------------------------------------|
| **§1234(a)**       | Holder         | Character mirrors underlying property; option holding period drives ST/LT (≤ 365 days = ST per §1222) |
| **§1234(a)(3)**    | Holder, §1221 ordinary underlying | Ordinary character regardless of option holding period   |
| **§1234(a)(4)**    | Holder exercises | NO realized event — premium adjusts basis of acquired/disposed underlying |
| **§1234(b)(1)**    | Writer         | **Fixed short-term capital** regardless of holding period — the "premium is always ST to the writer" rule |
| **§1234(b)(2)(A)** | Writer dealer in options | Ordinary character (carve-out from §1234(b)(1))                 |
| **Writer assignment** | Writer (counterparty exercises) | NO realized event — premium adjusts proceeds on sale of underlying |
| **§1234(c)**       | §1256 contracts | §1234 does not apply — route to `section_1256` 60/40 MTM            |

**§1234(b)(1) is the load-bearing bright-line.** Every covered-call and cash-secured-put writer relies on it: even if the writer holds the short side for 18 months, the close is ST. This is a deliberate override of §1234(a)'s mirror-character behavior. Pinned by `writer_held_over_one_year_still_short_term` and the explicit "regardless of option holding period" phrase in the note text (pinned separately so removing it would fail the test).

**Basis-adjustment events (Exercise / Assignment) return `is_basis_adjustment_event = true` with `gain_loss = 0`.** The option itself produces nothing — the premium becomes a price adjustment on the underlying. Caller's downstream code must NOT report these as Schedule D rows; they're folded into the underlying stock transaction. Pinned by `holder_exercised_no_realized_event` and `writer_assigned_is_basis_adjustment_event_no_gain_loss`.

**§1256 override fires before everything else.** A §1256 contract (broad-based index option, futures option, foreign currency option) bypasses §1234 entirely. The compute returns `TaxCharacter::Section1256` so the caller routes to `section_1256.rs` for 60/40 MTM. Pinned twice — once against the dealer-in-options carve-out, once against the holder-exercise basis-adjustment path — to catch any future branch-ordering regression that would let §1234 swallow §1256 cases.

**Sign conventions in the input:** `premium` is always positive (the absolute dollars; sign is implicit from role — Writer received it, Holder paid it). `close_proceeds_or_cost` is the close-side dollars (sale proceeds for Holder/Sold; buyback cost for Writer/BoughtBack). Lapsed is treated as 0 close-side. This keeps the validation logic simple in the route layer.

Mounted at `POST /api/calc/section-1234`. Twenty-one tests pin: writer lapsed → ST gain = premium; writer buyback below premium → ST gain; writer buyback above premium → ST loss; **writer held > 1 year still ST**; writer assigned → basis-adjustment event with zero gain/loss; writer dealer-in-options → ordinary character; holder sold short-term → ST capital; holder sold long-term LEAP → LT capital; holder lapsed → capital loss of premium; holder LEAP lapsed → LT capital loss (the patient-LEAP-holder trap); holder exercised → basis-adjustment event; **365-day boundary holder ST**; **366-day boundary holder LT**; holder ordinary underlying → ordinary regardless of holding period (§1234(a)(3) carve-out); §1256 underlying bypasses §1234; writer sold to another writer → still ST under §1234(b); zero premium writer lapsed → zero gain; **§1256 override priority over dealer flag** (branch-ordering regression target); **§1256 override priority over holder exercise** (similar); note describes actual holding-period days for ST/LT-boundary UX; writer note explicitly states "regardless of option holding period" (UX-text regression target).

`traderview-expense::section_1233` is the **IRC §1233 short-sale character + holding-period rules module** — paired with `section_1091` (wash sale) and `section_1259` (constructive sale) to close the trader anti-abuse trio for short positions. §1259 came in 1997 to shut down "short against the box" deferral; §1233 still applies wherever §1259's 30-day post-short window doesn't trigger.

Three pieces:

- **§1233(b)(1) — gain → short-term.** If on the date of the short open the taxpayer holds substantially identical property for ≤ 1 year, OR acquires substantially identical property between short open and short close, then any **gain** on closing the short is short-term — regardless of how long the property used to close the short was held. Prevents shorting against a short-term long position to lock in price while preserving capital-gain-rate optionality.
- **§1233(b)(2) — holding-period reset.** When §1233(b)(1) applies, the substantially identical property's holding period is **reset to begin on the date the short is closed**. FIFO across substantially identical lots (oldest acquisition first), capped at `short_shares` per Treas. Reg. §1.1233-1(c)(3). Only the short-held lots reset — long-held lots are unaffected (the rule penalizes new positions, not LTCG-qualified existing ones).
- **§1233(d) — loss → long-term.** If on the date of the short open the taxpayer holds substantially identical property for > 1 year, any **loss** on closing the short is long-term — regardless of how long the close property was held. Prevents using a short sale to convert a long-term loss into the short-term-loss bucket. STCL is cash-flow-preferable because §1212(b)(2) absorbs STCL first against the $3,000 ordinary-income offset; this rule forces the long-term character to stick.

**Both rules can trigger simultaneously** when the taxpayer holds BOTH short-held AND long-held substantially identical property at the time of the short. §1233(d) governs losses; §1233(b) governs gains; the sign of the close determines which rule fires. Pinned by `both_short_and_long_held_with_loss_subsection_d_wins` and `both_short_and_long_held_with_gain_subsection_b_wins`.

**Holding-period boundary is the §1222 standard**: ≤ 365 days = short-term (LTCG requires > 365). Day 365 exact triggers §1233(b); day 366 does not. Pinned by `one_year_boundary_held_exactly_365_days_is_short_term` + `one_year_boundary_held_366_days_is_long_term`.

Mounted at `POST /api/calc/section-1233`. Twenty-one tests pin: no substantially identical → default ST; short-held at open + gain → §1233(b) + reset; long-held at open + loss → §1233(d); long-held at open + gain → no rule (default ST); short-held at open + loss → no rule (default ST); **acquired-during-short triggers §1233(b)(1)(B)**; both short and long held with loss → §1233(d) wins; both short and long held with gain → §1233(b) wins (only short-held lots reset); FIFO resets capped at short_shares (150 candidates → first 100 reset); 365-day boundary short-held → §1233(b) triggers; 366-day boundary long-held → §1233(d) on loss path only; zero gain/loss → no rule; **reset date equals short-close date not short-open date** (regression target); during-short acquisition resets to close date; long-held lots never appear in reset list; combined (A)+(B) buckets in FIFO order by acquisition date; acquisition on short_sale_date is held_at_open not during_short (boundary classification); only-(A) bucket works; only-(B) bucket works; notes mention short close date when resets emit; notes mention loss amount when §1233(d) fires.

`traderview-expense::section_408m` is the **IRC §408(m) collectibles in IRA module** — the gold/silver/crypto IRA trap. Critical for any trader running a self-directed IRA. Companion to `section_408_d3` (IRA 60-day rollover) and `section_408A_d3` (Roth conversion 5-year aging).

**§408(m)(1) general rule**: acquisition of a "collectible" by an IRA is treated as a **deemed distribution** of the purchase price — taxable income to the beneficiary plus a 10% additional tax under §72(t) if under 59½. The IRA itself isn't disqualified; just the offending acquisition is recharacterized.

**§408(m)(2) definition of "collectible"** = artwork, antiques, gems, metals, stamps, coins, alcoholic beverages, rugs, and "any other tangible personal property" specified by IRS regulations. Cryptocurrency is treated as a collectible per IRS guidance.

**Two narrow exceptions under §408(m)(3):**

**(A) Statutory coin exception** — American Gold / Silver / Platinum / Palladium Eagles AND state-issued coins are permitted regardless of fineness. **The American Gold Eagle is the canonical edge case**: it's only 22-karat (.9167 fineness, BELOW the .995 bullion standard) but explicitly authorized by 31 U.S.C. § 5112 so the purity check is waived. Pinned by `american_gold_eagle_exempt_regardless_of_purity` and `eagle_coin_does_not_require_trustee_to_be_exempt` (Eagles don't even need trustee custody).

**(B) Bullion exception** — physical bullion meets the exception ONLY if BOTH (i) the fineness meets the metal-specific threshold AND (ii) it is in the physical possession of a qualified trustee (a bank or IRS-approved non-bank trustee):

| Metal     | Threshold | Bright-line (per 10,000) |
|-----------|-----------|--------------------------|
| Gold      | .995      | 9950                     |
| Silver    | .999      | 9990                     |
| Platinum  | .9995     | 9995                     |
| Palladium | .9995     | 9995                     |

**The "home storage gold IRA" trap is load-bearing.** Personal possession (gold in a home safe) fails the trustee-custody prong even with .9999 purity → deemed distribution fires. Pinned by `gold_bullion_995_without_trustee_home_storage_trap` ($100k purchase price becomes $100k taxable distribution, plus 10% §72(t) penalty if under 59½). The note explicitly mentions "home-storage trap" to surface this to the UI.

**Diagnostic flags** distinguish single-prong vs both-prong bullion failures so the caller can suggest the right fix:

| Path                          | `purity_meets_threshold` | `custody_qualifies` | Note phrasing                |
|-------------------------------|--------------------------|---------------------|------------------------------|
| Purity too low                | Some(false)              | true                | "purity below threshold"     |
| Custody fails (home storage)  | Some(true)               | false               | "home-storage trap"          |
| Both prongs fail              | Some(false)              | false               | "BOTH purity ... AND not ..." |
| Eagle/state coin              | None                     | (whatever)          | "statutory coin exception"   |

**Bright-line purity thresholds** are integer math (parts per ten thousand) — no Decimal precision drift. .995 = 9950 exactly; .9995 = 9995 exactly. Pinned by `gold_bullion_995_with_trustee_passes` + `gold_bullion_994_purity_fails` (9950 passes, 9949 fails). Distinct silver (.999 = 9990) vs gold (.995 = 9950) thresholds pinned by `silver_bullion_995_fails_too_low` — using gold's threshold for silver mis-classifies.

**Cryptocurrency in IRA → prohibited.** IRS has signaled in several rulings that crypto held in an IRA is a collectible. Compute returns deemed distribution = full purchase price. Pinned by `cryptocurrency_in_ira_is_prohibited_collectible`. Self-directed IRA promoters mis-sell crypto IRAs constantly; this is the load-bearing trader-facing answer.

**The non-Eagle non-state foreign coin trap** — South African Krugerrand and other foreign gold coins aren't on the Eagle whitelist and aren't state-issued → prohibited despite being mostly-pure gold. Compute calls this out by name in the note. Pinned by `other_coin_krugerrand_prohibited`.

Mounted at `POST /api/calc/section-408m`. Twenty-six tests pin: all four Eagle types exempt; state coin exempt; **Gold Eagle exempt regardless of .9167 purity** (load-bearing statutory exception); gold bullion .995/.9994 boundary; **home storage trap** ($100k → deemed distribution with diagnostic flags); both-prongs-fail with combined note; silver .999 boundary distinct from gold; platinum .9995 vs .9994 boundary; palladium same threshold as platinum; crypto / artwork / antique / gem / rug / stamp / alcoholic beverage all prohibited; Krugerrand foreign-coin trap; bullion missing purity treated as failing; diagnostic flag combinations; **Eagle doesn't require trustee** (no §3(A) custody prong, regression target); §72(t) penalty mentioned in every prohibited note (UX-text regression target); deemed distribution zero for all exempt paths; very large purchase ($100M) no precision loss; citation correct per path (§408(m)(3)(A) for Eagles, (3)(B) for bullion, (1) for prohibited).

`traderview-expense::section_83b` is the **IRC §83(b) restricted-stock election module** — every founder and early employee receiving restricted stock or restricted stock units needs to decide within 30 calendar days of grant whether to file this election. One of the most consequential tax-position choices in the code.

**Decision economics:**

- **§83(a) default**: ordinary income at VESTING = `FMV_vesting - amount_paid`. LTCG holding period begins at vesting.
- **§83(b) election**: ordinary income at GRANT = `FMV_grant - amount_paid`. LTCG holding period begins at grant.

When stock appreciates significantly between grant and vesting (the canonical founder case — grant FMV ≈ $0.001/share, vesting FMV $10+/share), the election converts what would be ordinary income at vesting into long-term capital gain at sale. For a founder with $10M of post-vesting appreciation, that's the gap between ~37% federal ordinary + state + FICA-Medicare and 20% federal LTCG + 3.8% NIIT + state — easily 20%+ savings on the appreciation.

**The 30-day deadline is bright-line and unforgivable.** No extension, no equitable exception, no judicial relief. Carta, Cooley, and the IRS agree: file by day 30 or the option is gone. Day 30 exact = valid; day 31 = invalid. Pinned by `election_filed_day_30_exact_boundary_is_valid` + `election_filed_day_31_invalid_per_bright_line` + `election_filed_day_zero_is_valid` (the three boundary cases).

**§83(b)(2) forfeiture trap** is load-bearing. If the property is later forfeited (employee leaves before vesting), the §83(b) election cannot be undone. Taxpayer paid ordinary income tax at grant on property never received. NO refund, NO deduction for the previously included amount. The only loss recognized is the **amount paid for the property** (usually zero for pure stock grants). Compute returns:

| Path                      | Forfeiture outcome                                                  |
|---------------------------|---------------------------------------------------------------------|
| Valid election + forfeit  | Grant-day ordinary recognized; NO refund; loss = amount paid only   |
| No election + forfeit     | Clean: §83(a) never triggered; no ordinary income; loss = amount paid |
| Late election + forfeit   | Same as no-election forfeit — late election was invalid anyway      |

Pinned by `forfeiture_with_valid_election_no_refund_per_83b2`, `forfeiture_without_election_clean_no_income`, and `forfeiture_with_late_election_clean_no_income`.

**Capital gain character driven by holding-period start.**

| Election path  | Holding start | Sale ≤ 365d from start → STCG | Sale > 365d → LTCG |
|----------------|---------------|-------------------------------|--------------------|
| Valid election | Grant date    | `election_sale_within_one_year_of_grant_is_stcg` | `election_sale_at_366_days_is_long_term` |
| No election    | Vesting date  | `no_election_sale_within_one_year_of_vesting_is_stcg` | `no_election_sale_one_year_after_vesting_is_ltcg` |

**Election savings field** compares ordinary income with election vs §83(a) baseline. Negative = good (election reduced ordinary income); positive = bad (election was wrong choice because FMV dropped between grant and vesting). Pinned by `founder_grant_election_saves_ordinary_income` (−$9.99 negative = good) and `election_wrong_when_fmv_drops_after_grant` (+$2 positive = bad).

**Ordinary income clamps at zero** when amount paid ≥ FMV at grant (i.e., underwater grant). Loss is recognized at sale, not as negative ordinary income. Pinned by `negative_grant_minus_paid_clamps_to_zero_ordinary`.

**Pathological inputs are bounded.** Election filed BEFORE grant date returns `days_grant_to_election = -1` and `election_timely = false`. Pinned by `election_before_grant_date_invalid`.

Mounted at `POST /api/calc/section-83b`. Twenty-four tests pin: timely election within 30 days valid; **day 0 / day 30 / day 31 boundaries** (the three bright-line cases); no election falls back to §83(a); founder grant election savings (−$9.99); election with full $50 appreciation → full LTCG; no-election same appreciation → ordinary $9.99 + LTCG $40 split; election sale within 1 year STCG; **365 vs 366-day boundary**; no-election sale at 17 months from vesting → LTCG; no-election STCG path; FMV grant above paid creates ordinary at grant; **election-was-wrong** (positive savings number); negative ordinary clamps to zero; not-yet-sold returns no gain; election before grant invalid (pathological pin); **very large grant precision** ($0.0001 → $5000); day-31 late election uses vesting basis (not grant); note describes 30-day deadline explicitly ("day 14/30"); forfeiture with valid election + §83(b)(2) call-out in note (UX-text regression target); forfeiture without election clean.

`traderview-expense::section_170e` is the **IRC §170(e) appreciated-property charitable contribution module** — the single highest-frequency tax-planning move for successful traders. Donate winners to charity, deduct FMV (or basis on specific paths), pay NO capital gain tax on the embedded appreciation. Independent of §1091 wash sale (gifts aren't sales, no replacement-period concern).

**Six rule paths** cover every combination of property kind × charity type × basis-election flag:

| Path                                            | Contribution = | AGI cap | Citation                          |
|-------------------------------------------------|----------------|---------|-----------------------------------|
| LTCG → public charity (no election)             | **FMV**        | 30%     | §170(b)(1)(C)(i)                  |
| LTCG → public charity (basis election)          | basis          | 50%     | §170(b)(1)(C)(iii)                |
| LTCG → private foundation (qualified appreciated stock) | **FMV** | 20%     | §170(e)(5)                        |
| LTCG → private foundation (not QAS)             | basis          | 20%     | §170(e)(1)(B)(ii)                 |
| STCG / Ordinary → public charity                | basis          | 50%     | §170(e)(1)(A)                     |
| STCG / Ordinary → private foundation            | basis          | 30%     | §170(e)(1)(A)                     |
| Tangible personal unrelated use → public        | basis          | 50%     | §170(e)(1)(B)(i)                  |
| Tangible personal unrelated use → private       | basis          | 30%     | §170(e)(1)(B)(i)                  |

**§170(e)(5) Qualified Appreciated Stock (QAS) carve-out is load-bearing for family foundations.** Publicly-traded stock not exceeding 10% of the corporation's outstanding shares qualifies for FMV deduction even to a private foundation, despite the general §170(e)(1)(B)(ii) reduction rule. This is why family foundations love receiving Apple / Microsoft / Berkshire-Hathaway shares but not closely-held LLC interests. Pinned by `ltcg_private_foundation_qas_fmv_deduction_at_20pct` ($100k FMV → $40k deductible at 20% AGI cap, $60k carries forward, $90k gain eliminated).

**§170(b)(1)(C)(iii) basis election trades deduction amount for AGI capacity.** Same $100k FMV / $10k basis LTCG stock to public charity:
- No election: contribution $100k, 30% cap = $60k deductible, $40k carryforward, $90k gain eliminated
- With election: contribution $10k, 50% cap = $100k, full $10k deductible, $0 carryforward, $0 gain eliminated

The election is rational when basis is close to FMV (gain elimination matters less) or when the donor needs the higher AGI limit. Pinned in `ltcg_public_basis_election_lower_deduction_higher_cap`.

**§170(e)(1)(A) STCG/ordinary reduction is the "don't donate winners you've held < 1 year" trap.** The reduction wipes out the embedded gain entirely, so the deduction is basis only. This is exactly the same answer as the basis election — but you didn't choose it, you got it by holding < 1 year. Pinned by `stcg_property_reduced_to_basis_under_170e1a` and `ordinary_income_property_same_reduction_as_stcg`.

**§170(e)(1)(B)(i) tangible personal unrelated use** hits the "donate art to a hospital" case — the donor must determine whether the donee's use of the property is RELATED to the donee's exempt purpose. Art to a museum that displays it: related, FMV deduction available. Art to a hospital that sells it at auction: unrelated, basis only. The compute does not adjudicate the related/unrelated determination — caller flips the `PropertyKind::TangiblePersonalUnrelatedUse` discriminator when the unrelated test fails.

**Branch ordering between flags is load-bearing.** When both `is_qualified_appreciated_stock = true` AND `elect_basis_for_higher_limit = true` on a public-charity LTCG donation, the basis election wins (QAS is irrelevant for public charity). Pinned by `qas_with_basis_election_election_wins`. Conversely, the QAS flag is silently ignored on public-charity donations (`qas_flag_ignored_for_public_charity_path`).

**§170(d) 5-year carryforward** with same-character bucketing. Prior carryover adds to the current contribution before the AGI cap check; excess rolls to next year. Caller must track buckets separately if straddling rule paths (e.g., LTCG and STCG carryforwards don't pool).

Mounted at `POST /api/calc/section-170e`. Twenty-three tests pin: canonical LTCG-public-FMV path with all numbers spelled out ($100k → $60k deduct + $40k CF + $90k gain eliminated); basis election trade-off; STCG and ordinary income same reduction; LTCG-private-foundation QAS at 20% cap; non-QAS reduces to basis; tangible unrelated use to both public (50%) and private (30%); prior carryover compounds against current cap; other-this-year contributions eat budget; **zero AGI → full carryforward**; contribution exactly at cap → 0 carryforward; **other contributions exceeding cap clamp remaining at 0** (negative-budget regression target); **FMV below basis no gain eliminated reports 0 not negative** (the underwater-stock no-bonus case); basis election flag ignored for STCG; QAS flag ignored for public-charity path; QAS+election combo → election wins (branch ordering pinned); note describes rule path citation + cap pct; QAS path note mentions §170(e)(5); very large donation no precision loss ($9.87B basis with $20B AGI); multi-year roll picks up prior carryforward only (zero new contribution case); **carryforward never negative under pathological negative input**; private-foundation STCG uses 30% cap not 20% (rule × charity-type interaction).

`traderview-expense::section_108` is the **IRC §108 cancellation of debt income module** — critical for distressed debt traders, underwater real estate investors, mortgage workouts, and credit card settlement scenarios. Default rule under §61(a)(12) is that cancelled debt is gross income; §108(a) provides five narrow exclusions with mandatory §108(b) attribute-reduction consequences.

**Five exclusions in priority order:**

| Priority | Citation         | Exclusion type                            | Conditions                                              |
|----------|------------------|-------------------------------------------|---------------------------------------------------------|
| 1        | §108(a)(1)(A)    | Title 11 bankruptcy                       | Discharge in bankruptcy case                            |
| 2        | §108(a)(1)(E)    | Qualified principal residence indebtedness | Pre-2026 written arrangement; election can swap to (B)  |
| 3        | §108(a)(1)(B)    | Insolvency                                | Liabilities > FMV of assets (§108(d)(3) test)           |
| 4        | §108(a)(1)(C)    | Qualified farm                            | Farmer (50%+ income from farming for 3 prior years)     |
| 5        | §108(a)(1)(D)    | Qualified real property business (QRPBI)  | Non-C-corp with qualifying real property biz debt        |

**Bankruptcy is highest priority and full exclusion** under §108(a)(1)(A). Pinned by `bankruptcy_full_exclusion_highest_priority` + `bankruptcy_overrides_qpri` (even with QPRI flags set, bankruptcy wins) + `priority_bankruptcy_over_insolvency`.

**§108(d)(3) Insolvency test**: insolvency_amount = `liabilities - assets_fmv`, clamped at zero (solvent → 0). Insolvency excludes ONLY up to the insolvency amount — solvent taxpayers must include the entire COD income. Pinned by `insolvency_full_exclusion_when_amount_ge_debt` ($30k insolvency, $50k debt? wait actually full exclusion when insolvency ≥ debt; let me re-check — yes, $100k insolvency ≥ $50k debt = full exclusion) + `insolvency_partial_exclusion_with_remainder_includible` ($30k insolvency, $50k debt → $30k excluded, $20k includible) + `insolvency_amount_at_exact_zero_solvent` (assets = liabilities → solvent, no exclusion via this path).

**QPRI sunset before January 1, 2026** under §108(a)(1)(E). Discharges in 2026+ on **pre-2026 written arrangements** still qualify; new arrangements made in 2026 do NOT. Pinned by `qpri_pre_2026_exclusion_overrides_insolvency` (full $50k excluded under (E), insolvency wouldn't have been needed) + `qpri_post_2026_arrangement_does_not_exclude` (flag set but post-2026 arrangement → falls through to no exclusion or insolvency).

**§108(a)(2)(C) election** lets taxpayer choose insolvency (B) instead of QPRI (E). Useful when attribute reduction under (B) is more favorable than basis reduction under (E). Pinned by `qpri_election_to_use_insolvency_routes_through_insolvency` (election routes through (B) path even with QPRI flags set).

**Qualifying-debt categories layer on top of insolvency** when insolvency is only partial. Insolvent by $30k + $50k debt + qualified farm + qualified farmer → $30k via (B), remaining $20k via (C). Pinned by `insolvency_partial_plus_farm_excludes_remainder` + `insolvency_partial_plus_qrpbi_excludes_remainder`.

**Farm exclusion requires BOTH the debt category AND the qualified-farmer status** (§108(g)(1)(B): 50%+ of income for 3 prior years from farming). Pinned by `qualified_farm_indebtedness_solvent_excludes` (both flags + solvent → full exclusion) + `farm_debt_without_qualified_farmer_status_no_exclusion` (farm debt + non-farmer → no exclusion).

**C-corporations cannot use §108(a)(1)(D) QRPBI exclusion** — statutorily blocked. Pinned by `qrpbi_for_non_c_corp_excludes` + `c_corp_cannot_use_qrpbi_exclusion`.

**§108(b) attribute reduction equals the excluded amount** — the "deferred tax" cost of the exclusion. Reduces NOL carryovers, GBC, minimum tax credit, capital loss carryovers, basis of property, PAL carryovers, FTC carryovers (in that order). This module reports the required total; ordering and per-bucket allocation is downstream Form 982 work. Pinned by `attribute_reduction_equals_excluded_amount`.

Mounted at `POST /api/calc/section-108`. Twenty-two tests pin: solvent taxpayer no exclusion full inclusion; **bankruptcy highest priority full exclusion**; insolvency full when amount ≥ debt; insolvency partial with remainder includible; QPRI pre-2026 overrides insolvency; **QPRI post-2026 does NOT exclude** (sunset regression target); QPRI election routes through insolvency; bankruptcy overrides QPRI; qualified farm + farmer excludes solvent; farm debt without farmer status no exclusion; QRPBI for non-C-corp excludes; C-corp cannot use QRPBI; insolvency partial + farm covers remainder; insolvency partial + QRPBI covers remainder; attribute reduction = excluded amount; insolvency clamps to zero when solvent; insolvency at exact zero (liabilities = assets) is solvent; **priority sweep** (bankruptcy > QPRI > insolvency); zero canceled debt no-op; very large precision ($10B bankruptcy discharge); note describes section per path (bankruptcy / QPRI / insolvency / §61(a)(12) no-exclusion).

`traderview-expense::section_7872` is the **IRC §7872 below-market loan module** — the family sweetheart loan trap. When a trader lends to family, child, or controlled entity at below-AFR rates, the IRS imputes the missing interest as if charged at the Applicable Federal Rate. Forgone interest becomes income to the lender AND deemed transferred back as gift / compensation / dividend depending on the relationship.

**AFR brackets by loan term** under §1274(d):

| Bracket    | Term                       | Boundary       |
|------------|----------------------------|----------------|
| Short-term | ≤ 3 years                  | day 0          |
| Mid-term   | > 3 and ≤ 9 years          | day 3yr + 1    |
| Long-term  | > 9 years                  | day 9yr + 1    |

Pinned by `short_term_loan_classified_at_3_year_boundary` (term = 3y exact = short) + `mid_term_loan_at_3_year_plus_boundary` (3.01y = mid) + `mid_term_loan_at_9_year_boundary` (9y exact = mid) + `long_term_loan_at_9_year_plus_boundary` (9.01y = long).

**Two narrow exceptions for GIFT loans ONLY** (no exceptions for compensation or corp-shareholder):

**§7872(c)(2)(A) — $10,000 de minimis.** If aggregate outstanding ≤ $10,000 AND proceeds NOT used for income-producing assets, NO imputation. Both conditions required; pinned by `gift_loan_10k_de_minimis_no_imputation` (passes) + `gift_loan_10001_exceeds_de_minimis` (boundary + 1 fails) + `gift_loan_10k_used_for_income_producing_disables_de_minimis` (income-asset flag kills exception even at $10k).

**§7872(d)(1) — $100,000 NII cap.** Gift loans with aggregate ≤ $100,000:
- Borrower NII ≤ $1,000 → NO imputation (separate de minimis floor on NII)
- Borrower NII > $1,000 → imputation CAPPED at borrower's NII (never more)

Pinned by `gift_loan_below_100k_nii_below_1k_no_imputation` + `gift_loan_below_100k_nii_exact_1k_no_imputation` (1k exact = ≤ → exception) + `gift_loan_below_100k_nii_above_1k_capped_at_nii` + `gift_loan_below_100k_nii_caps_higher_raw_forgone` (load-bearing: $3,000 raw forgone capped at $1,500 NII).

**Full AFR imputation** under §7872(a)(1) applies to:
- Gift loans with aggregate > $100,000 (no NII cap)
- All compensation-related loans (any size)
- All corporation/shareholder loans (any size)

Pinned by `gift_loan_above_100k_full_afr_imputation` ($200k × 3% gap = $6,000 imputed, no NII cap) + `compensation_loan_no_de_minimis_no_nii_cap` (even $5k compensation loan gets full imputation) + `corporation_shareholder_loan_no_exceptions` ($8k corp loan still imputed).

**Aggregate-outstanding is the threshold metric, not individual loan principal.** A small $5k loan can fall outside the $10k de minimis if the aggregate between the two parties already exceeds $10k. Pinned by `aggregate_outstanding_threshold_uses_aggregate_not_principal`.

**Boundary thresholds use ≤ on the high side**: $10k = de minimis applies; $10k + 1 = fails. $100k = NII cap applies; $100k + 1 = full imputation. NII $1,000 exact = exception applies. Pinned by individual boundary tests.

**Raw forgone interest is reported even when an exception applies** for diagnostic/documentation purposes. UI can show "would have been $X but exception applies". Pinned by `forgone_raw_reported_even_when_exception_applies` ($300 raw, $0 imputed, both fields present on result).

**Deemed transfer equals imputed income (mirror)**. The lender's imputed interest income and the borrower's deemed gift/comp/dividend are always equal — they're two sides of the same accounting entry. Pinned by `deemed_transfer_equals_imputed_income`.

**Rate ≥ AFR short-circuits with no imputation** regardless of loan type or size. The §7872 rule only fires on below-market loans. Pinned by `rate_meets_afr_no_imputation` (rate = AFR exact) + `rate_above_afr_no_imputation` (rate > AFR).

Mounted at `POST /api/calc/section-7872`. Twenty-four tests pin: rate at/above AFR no imputation; **$10k de minimis** with boundary cases (passes at 10k, fails at 10k+1, income-asset disables); **§7872(d)(1) NII cap** with three sub-paths (≤ $1k no imputation, > $1k capped at NII, cap binds when NII < raw forgone); gift loan > $100k full imputation; compensation + corp-shareholder loans always get full imputation (no exceptions); aggregate-outstanding is the metric not principal; **AFR term classification at 3-year and 9-year boundaries** (both edges per bracket); zero-interest loan = full forgone at AFR; forgone raw reported even on exception paths; **aggregate at $100k exact = NII cap path** (boundary regression target); aggregate > $100k = full imputation; **deemed transfer mirrors imputed income** (accounting consistency pin); very large precision ($10B HNW loan with $300M imputed); note describes rule path per branch (de minimis / NII cap / full imputation).

`traderview-expense::section_1041` is the **IRC §1041 transfers between spouses module** — completes the basis-transfer trio with `section_1014` (death) and `section_1015` (lifetime gift). Critical for HNW divorce property division.

**§1041(a)**: NO gain or loss recognized on transfer between current spouses or former spouses if incident to divorce. The transferor doesn't pay tax on embedded appreciation at transfer.

**§1041(b)**: Transferee takes the **transferor's adjusted basis** as carryover — regardless of whether basis is less than, equal to, or greater than FMV at transfer. **No dual-basis rule** like §1015 (spouses are economically a single unit; loss-shifting concern doesn't apply). Holding period TACKS uniformly under §1223(2) — both for gain and for loss.

**§1041(c) "incident to divorce" rules** per Treas. Reg. § 1.1041-1T(b) Q&A-7:

| Days from cessation | Rule                                             | Applies?                          |
|---------------------|--------------------------------------------------|-----------------------------------|
| 0-365 (within 1 yr) | **Automatic** — no test required                 | Always yes                        |
| 366-2190 (1-6 yrs)  | Must be pursuant to divorce/separation instrument | Yes IF instrument flag set       |
| 2191+ (>6 yrs)      | Presumption against; needs instrument language    | Yes IF instrument flag set (rebuts presumption) |

**§1041(d) NR alien exception**: §1041 does NOT apply when the transferee spouse (or former spouse) is a nonresident alien at the time of transfer. Highest precedence — fires BEFORE the timing-rule check. Transferor recognizes immediate gain/loss as though transferring to a third party. Pinned by `nonresident_alien_overrides_otherwise_applicable_path` (NR alien + currently married still disqualifies).

**Distinction from §1015 (lifetime gift carryover)** is load-bearing:

| Rule                | §1015 (lifetime gift)            | §1041 (spousal transfer)         |
|---------------------|----------------------------------|----------------------------------|
| Recognition at transfer | None for donor (gift exclusion) | None for transferor              |
| Basis to recipient  | Donor's basis                    | Transferor's basis               |
| Dual-basis on depreciated property | **YES — phantom zone** | **NO — single basis for gain or loss** |
| §1015(d) gift-tax basis increase | Applies                | Does NOT apply (no gift tax)     |
| Holding period tacking | Gain path only; loss path resets | Always tacks                  |

Pinned by `depreciated_property_no_dual_basis_unlike_section_1015` (transferor basis $100k, FMV $50k, sale $30k → loss $70k recognized via $100k carryover basis, not via $50k FMV).

**1-year boundary is bright-line**: day 365 = automatic; day 366 = requires instrument. Pinned by `within_one_year_exact_boundary_day_365_automatic` + `day_366_requires_divorce_instrument` + `day_366_with_instrument_applies`.

**6-year boundary is also bright-line at 2190 days** (6 × 365). Day 2190 = within 1-6 year window; day 2191 = beyond 6 years. Both require instrument; difference is the IncidentRule classification that surfaces to the UI. Pinned by `six_year_exact_boundary_with_instrument_applies` (day 2190 = OneToSix) + `day_2191_beyond_six_years_with_instrument_applies` (day 2191 = BeyondSixYears) + `beyond_six_years_without_instrument_disqualified`.

**Holding period only tacks when §1041 applies**. Disqualified paths (NR alien, not incident to divorce) start the holding period at the transfer date. Pinned by `holding_period_tacks_when_section_1041_applies` + `holding_period_starts_at_transfer_when_disqualified`.

Mounted at `POST /api/calc/section-1041`. Twenty-four tests pin: current spouses applies; within 1 year automatic; **day 365 exact boundary**; **day 366 requires instrument** (both with-instrument and without-instrument paths); **6-year boundary at day 2190** (calendar-leap-year-aware regression target); **day 2191 beyond 6 years**; **NR alien overrides everything** (precedence pin); carryover basis with zero recognition; FMV basis with full recognition when §1041 fails; sale uses carryover basis under §1041; sale uses FMV basis when disqualified; holding period tacks vs starts at transfer; **depreciated property NO dual basis** (distinguishes from §1015); loss recognized at carryover basis even on depreciated property; zero basis transferor → zero basis transferee; note describes each IncidentRule path; disqualification reason in note; very large basis ($1.234B) no precision loss; 366-day post-transfer LTCG via tacking; 365-day boundary STCG.

`traderview-expense::section_1015` is the **IRC §1015 carryover basis on gifts module** — sibling to `section_1014` (stepped-up basis at death). Where §1014 wipes out embedded gains at death, §1015 **carries them through to the donee** for eventual recognition. No step-up on lifetime gifts.

**§1015(a) general carryover** — donee's basis = donor's adjusted basis. Holding period TACKS from donor's acquisition date under §1223(2). A one-day-old gift of LTCG-eligible stock is immediately long-term in the donee's hands. Pinned by `appreciated_with_long_donor_holding_immediate_ltcg_via_tacking`.

**§1015(a) dual-basis rule (depreciated property)** is the famous loss-bifurcation trap. When FMV at gift is LESS than donor's adjusted basis, the donee takes a **split basis**:

| Sale price                      | Basis used   | Outcome                                |
|---------------------------------|--------------|----------------------------------------|
| Sale > donor's basis            | Donor's basis | Gain = sale - donor's basis           |
| Sale < FMV-at-gift              | FMV-at-gift   | Loss = sale - FMV-at-gift             |
| FMV-at-gift ≤ sale ≤ donor's basis | (neither)  | **PHANTOM ZONE — no gain, no loss**    |

The phantom zone is the load-bearing case. If donor's basis = $100k, FMV-at-gift = $50k, and donee sells at $75k, NEITHER gain nor loss is recognized — the donor's embedded $25k loss vanishes (loss bifurcation prevents donor from shifting loss to donee) AND the donee's $25k of economic appreciation since gift also disappears for tax purposes. Pinned by `depreciated_sale_in_phantom_zone_no_gain_no_loss` + both exact-boundary pins (`depreciated_phantom_zone_at_fmv_exact_boundary` at $50k = no loss; `depreciated_phantom_zone_at_donor_basis_exact_boundary` at $100k = no gain).

**Loss-side holding-period exception**. On the dual-basis loss path, the donee's holding period **starts at the gift date**, not the donor's acquisition date. Per Treas. Reg. § 1.1015-1, tacking applies only when the basis is determined "in whole or in part" by reference to donor's basis — using FMV-for-loss satisfies neither. Pinned by `loss_path_holding_period_starts_at_gift_not_donor` (donor held since 2020, gift 2026-01-01, sale 2026-07-01 on loss path → STCG, not LTCG).

**§1015(d) gift-tax basis increase** — when gift tax was paid on the transfer, the donor's basis is increased by the gift tax attributable to net appreciation:

```
increase = gift_tax_paid × (net_appreciation / gift_amount_for_tax_purposes)
```

where `net_appreciation = FMV - donor's basis`. Two ceilings:

1. **Cannot exceed net appreciation itself** — prevents the formula from over-correcting on unusual gift-tax-to-gift-amount ratios.
2. **Cannot exceed FMV at gift date** — prevents the adjustment from converting a gain asset into a loss asset.

Pinned by `gift_tax_basis_increase_applied_to_appreciated` (canonical case: $30k tax × ($90k / $84k) = $32.14k increase) + `gift_tax_basis_increase_capped_at_net_appreciation` (pathological large tax → cap fires) + `no_gift_tax_increase_on_depreciated_property` (depreciated → net_appreciation = 0 → no increase even if tax paid) + `zero_gift_amount_no_increase` (division-by-zero guard).

**Mode classification:**

| Mode                              | Sale outcome | Holding period start    | §1015(d) applies? |
|-----------------------------------|--------------|--------------------------|---------------------|
| Appreciated (single basis)        | Gain or Loss | Donor's acquisition      | Yes if tax paid     |
| Depreciated, sale > donor's basis | Gain         | Donor's acquisition      | Yes if tax paid (but net_app might be 0) |
| Depreciated, sale < FMV-at-gift   | Loss         | **Gift date** (no tacking) | No (net_app = 0)   |
| Depreciated, phantom zone         | Neither      | n/a                      | No                  |

**FMV exactly equals donor basis**: single basis (no dual). The dual-basis rule fires only when FMV is **strictly less than** donor basis. Pinned by `fmv_equals_donor_basis_single_basis_no_dual`.

**Appreciated property + sale below basis**: single-basis path → loss via donor's (single) basis. Dual basis does NOT fire. Pinned by `appreciated_sale_below_basis_single_basis_loss` (sale $5k, donor basis $10k → -$5k loss).

Mounted at `POST /api/calc/section-1015`. Twenty-four tests pin: appreciated carryover; **LTCG via §1223(2) tacking** (donor held 6.5 years, sale 6 months after gift → still LTCG); depreciated sale-above-donor-basis gain via donor basis; **depreciated sale-below-FMV loss via FMV** (not donor basis); **phantom zone with both exact boundaries** (loss-bifurcation regression target); **loss path holding period starts at gift** (Treas. Reg. § 1.1015-1 regression target); gain path tacks; **§1015(d) gift-tax increase canonical** ($30k × 90k/84k); §1015(d) net-appreciation ceiling fires; no §1015(d) on depreciated property; zero gift tax / zero gift amount / FMV equals donor basis single-basis; sale at donor basis exact → no-gain-no-loss; appreciated single-basis loss; dual-basis with gift-tax-increase math; note describes loss-path no-tacking exception; note describes loss-bifurcation in phantom zone; zero basis donor; **very large precision** ($1.234B basis); 366-day boundary gain LTCG; 365-day boundary loss-path STCG.

`traderview-expense::section_1014` is the **IRC §1014 stepped-up basis at death module** — the single most powerful rule in the Internal Revenue Code for buy-and-hold investors and the foundation of every "die with low basis" estate-planning strategy. When property passes from a decedent to an heir, the heir's basis is **stepped up (or down) to the fair market value on the date of death** under §1014(a)(1). All gain or loss that accrued during the decedent's lifetime is permanently eliminated.

Four exceptions / refinements:

- **§1014(c) — Income in respect of decedent (IRD)** — NO step-up. Applies to IRA distributions, accrued bond interest, deferred compensation, US savings bond interest. The decedent's adjusted basis carries over and the heir recognizes the embedded ordinary income on receipt. The "IRA can't escape tax at death" rule.
- **§1014(e) — One-year anti-abuse clawback** — if the decedent acquired the property by gift within 1 year of death AND the property passes back to the donor (or the donor's spouse), there is NO step-up. Heir's basis = decedent's adjusted basis. Blocks the "deathbed-gift-to-grandma" basis-laundering strategy where a family member transfers low-basis appreciated property to a dying relative just to harvest the step-up on death.
- **§1014(f) — Consistent basis with estate tax return** — heir's basis cannot exceed the FMV reported on Form 706. Closes the "report low for estate tax, claim high for income tax" arbitrage. Capped by Form 706 (ceiling, not a floor); higher Form 706 doesn't INCREASE heir's basis.
- **§2032 — Alternate valuation date election** — executor may elect to value the estate as of 6 months after DOD instead of DOD itself, BUT only if the election lowers BOTH (a) gross estate AND (b) federal estate tax. Election applies to ALL property in the estate. Canonical use: declining-value estate (stocks crashed in the 6 months after death).

**Priority ordering is load-bearing**: §1014(c) IRD → §1014(e) clawback → §2032 valuation pick → §1014(f) Form 706 cap. The IRD and clawback rules check BEFORE the valuation-date logic so a §2032 election cannot accidentally override the no-step-up rules. Pinned by `ird_takes_precedence_over_alternate_valuation_election` and `clawback_takes_precedence_over_alternate_valuation`.

**§2032 election validity is a both-must-decrease test**, not just one. The fallback flag distinguishes "election claimed but invalid (didn't lower both)" from "election not claimed at all" — both fall to DOD valuation but only the former is a tax-position issue the caller should surface. Pinned by `alternate_valuation_election_invalid_falls_back_to_dod` and `alternate_valuation_election_without_av_fmv_falls_back` (two distinct invalidity modes).

**§1014(e) requires BOTH conditions** — gift within 1 year AND property returns to donor/spouse. Either alone is insufficient. If the property goes to a different heir (a child rather than back to the original donor), the clawback doesn't trigger and standard step-up applies. Pinned by `one_year_clawback_does_not_trigger_when_property_goes_to_other_heir` and `one_year_clawback_does_not_trigger_outside_one_year_window` (the two negative cases).

**Step-up cuts both ways**. The rule says basis = FMV, period. If FMV < basis (declining asset), the basis steps DOWN and the embedded loss is also eliminated. Pinned by `step_down_works_basis_drops_to_lower_fmv` — this is why the alternate-valuation election can hurt the heir's income-tax position even as it lowers the gross estate.

Mounted at `POST /api/calc/section-1014`. Twenty-one tests pin: standard step-up ($10k → $100k, $90k eliminated); **step-down works** ($100k → $50k); §1014(c) IRD no step-up; §1014(e) clawback denies step-up; **clawback negative case 1** (property to other heir); **clawback negative case 2** (outside 1-year window); §2032 valid election uses AV FMV ($75k); §2032 invalid fallback to DOD; §2032 election without AV FMV fallback; §1014(f) cap applied when Form 706 below FMV ($80k vs $100k); §1014(f) does NOT cap when Form 706 above FMV (ceiling-not-floor); **IRD takes precedence over §2032** (priority ordering); **clawback takes precedence over §2032** (priority ordering); zero basis full step-up ($0 → $500k = $500k eliminated, the purest "die with $0 basis" play); basis equals FMV zero gain eliminated; **Form 706 cap with §2032 compounds correctly** (AV first, then cap); IRD + Form 706 no interaction (cap flag stays false); note describes DOD vs alternate-valuation phrasing distinctly; very large basis no Decimal precision loss ($1.23B basis); alternate-valuation step-down (canonical declining-estate use case).

`traderview-expense::section_1091` is the **IRC §1091 wash sale loss-disallowance module** — the trader rule. Fills the long-standing `wash_sale.rs` TODO referenced by `schedule_d.rs` since the module was first written ("does NOT model §1091 wash sales — caller pre-processes those"). Every non-§475(f) trader needs this; §475(f) electors are exempt per §475(f)(1)(C).

**Core rule (§1091(a)):** loss on the sale of stock or securities is disallowed if the taxpayer (or spouse, controlled corp, or IRA) acquires substantially identical stock within a **61-day window centered on the sale date** — 30 days before, the sale date itself, and 30 days after. The disallowed loss is added FIFO to the basis of the replacement shares under §1091(d); the holding period tacks under §1223(4).

**Disallowance ratio** = min(replacement_shares_in_window, sale_shares) / sale_shares. Per-share basis addition to each replacement lot equals the original per-share loss. Replacement shares exceeding the sale count are unaffected — only the first N where N=sale_shares get adjusted.

**Rev. Rul. 2008-5 IRA carve-out** is load-bearing: when the replacement is purchased in the taxpayer's IRA or Roth IRA, §1091(d) is overridden. The loss is still disallowed but the IRA basis is NOT increased — the loss is permanently lost. Most retail brokers do not warn on this. The module's `account_is_ira` flag drives this path: for IRA replacement lots, `basis_added_per_share = 0` and `ira_permanent_loss = true` on the result.

**§475(f) MTM elector exemption** short-circuits the entire rule per §475(f)(1)(C). The check happens BEFORE the window scan so a 475(f) elector with a same-day repurchase doesn't falsely trigger (test pinned: `mtm_election_short_circuits_before_window_check`).

Mounted at `POST /api/calc/section-1091`. Twenty-one tests pin: sale at gain skips rule (loss-only); loss with no replacement allowed; loss with full replacement 100% disallowed; loss with partial 40% replacement → 40% disallowed (conservation: `|allowed| + disallowed = |original|` swept across 10/25/50/75/100); **−30 day boundary in window**; **−31 day boundary outside**; +30/+31 boundaries pinned same way; **§475(f) elector completely exempt** even with same-day same-share replacement; **IRA replacement triggers permanent loss** with zero basis addition; FIFO allocation across multiple lots (30 + 50 + 80 with sale 100 → first lot 30, second 50, third 20); out-of-order purchases sorted to FIFO; outside-window purchases ignored even with in-window co-purchases; replacement exceeds sale capped at sale_shares (250 repl → only 100 adjusted); **mixed IRA + taxable splits correctly** (60 IRA + 80 tax with sale 100 → IRA 60 shares $0 basis, tax 40 shares $400 basis); sale-on-Friday + Monday repurchase in window; zero-share sale no-op; MTM election short-circuits before window check; same-day replacement is in window (T+0 trap); conservation invariant across all partial-disallowance ratios.

`traderview-expense::section_481` is the **IRC §481(a) accounting method change adjustment module** — the cumulative MTM hit that every trader making the §475(f) trader-in-securities election (covered by the existing `mtm_475f` module) faces on day 1 of the election year. All open positions are marked to market; the resulting cumulative delta from the prior cost-basis method is a §481(a) adjustment recognized as **ordinary income or loss**.

Recognition timing follows Rev. Proc. 2015-13:
- **Positive §481(a) (net unrealized gain)** — spread **ratably over 4 tax years** (25% per year). Avoids the one-time tax cliff that would otherwise crush traders with large unrealized gains at election.
- **Negative §481(a) (net unrealized loss)** — recognized **entirely in the year of change**. No spread on losses; the trader gets the deduction immediately, consistent with §475(f)'s general ordinary-loss character.

The 4-year spread is purely timing relief — it doesn't change character. Both gain and loss adjustments are ordinary, not capital, not LTCG-eligible. The final year of the spread absorbs any rounding residual so the cumulative recognized total ties out exactly to the original adjustment.

Mounted at `POST /api/calc/section-481`. Seventeen tests pin: positive adjustment 25% year 1 ($100k gain → $25k recognized + $75k remaining); year 4 full recognition ($100k cumulative + $0 remaining); negative adjustment recognized immediately no spread; negative adjustment in years after election shows already-fully-recognized cumulative; aggregates across multiple positions ($50k + $30k - $20k = $60k net spread); zero adjustment no-op; multi-year cumulative grows 25k/50k/75k/100k predictably; recognition before election year zero; schedule has exactly 4 entries for positive; **final year absorbs rounding residual** for both even ($100,001) and odd ($100,003) totals (schedule sums tie out to total); year-2 = $25k for $100k gain; `spread_years_override: Some(2)` distributes 50% each year (only 2 entries); per-position delta breakdown preserves symbol + basis + FMV + delta; empty positions zero-adjustment zero-recognition; negative adjustment + current year before election zero recognition; **mixed winners/losers netting to negative** recognized as immediate loss (no spread).

`traderview-expense::section_1092` is the **IRC §1092 straddle loss deferral module** — the rule every active trader doing options or futures hedges hits. A **straddle** under §1092(c)(1) is two or more offsetting positions that substantially diminish the taxpayer's risk of loss on holding any one of them. §1092(a)(1) defers loss on the closed leg to the extent of **unrecognized gain on the offsetting position(s) held at year-end**; the disallowed loss carries forward until the offsetting position is closed. §1092(b)(2) also **suspends the holding period** of every straddle position while the straddle remains open — preventing short-against-the-box-style conversion of short-term gain into long-term.

The **§1092(c)(4)(B) qualified covered call (QCC) exception** is the load-bearing carve-out. A covered call (long stock + written call) is NOT a straddle when ALL three conditions are met: (1) the underlying is a **publicly traded** stock; (2) the call has **more than 30 days to expiration** when written (strict `>`, not `≥` — exactly 30 days disqualifies); (3) the strike is **not deep in the money** (per Reg. §1.1092(c)-1, the strike must be at or above the "lowest qualified benchmark" — typically the first available strike less than the prior-day close). When QCC qualifies, the loss is fully recognized and the long-stock holding period is preserved.

Mounted at `POST /api/calc/section-1092`. Seventeen tests pin: loss fully deferred when gain on offset exceeds loss (5k gain + 2k loss → all 2k deferred, 0 recognized); loss partially deferred when gain less than loss (2k gain + 5k loss → 2k deferred, 3k recognized); no gain on offset full loss recognized (still flags holding-period suspension because it's still a straddle); loss-on-disposed-at-zero no-op; QCC exception fully qualified recognizes loss with holding period preserved; **QCC disqualified at exactly 30 days** (boundary — strict `>` boundary); QCC qualified at 31 days; QCC disqualified when underlying not publicly traded; QCC disqualified when strike deep ITM; multiple offsetting legs sum their unrecognized gains; **unrealized LOSS on offsetting leg doesn't count negative** (only positive unrecognized gains feed the deferral pool, losses ignored); loss exactly equal to gain fully deferred; no-offsetting-legs degenerate case handled; note distinguishes QCC path from normal straddle path; holding-period suspension only for non-QCC straddle; empty legs no-op; QCC short-circuit runs before offsetting-gain calculation (even a $90k gain doesn't change the QCC outcome).

`traderview-expense::section_1259` is the **IRC §1259 constructive sale of appreciated financial position module** — the "short against the box" anti-conversion rule that ended pre-1997 traders' ability to lock in gains tax-free by shorting the same stock they held long. An appreciated long position (FMV > basis) is **constructively sold** when the taxpayer enters a hedge that substantially eliminates risk of loss and opportunity for gain. The deemed sale triggers gain recognition at FMV as of the hedge entry date; basis steps up to FMV per §1259(b)(2) and holding period restarts on the hedge entry date.

`HedgeType` enumerates the §1259(c) covered-transaction list: `ShortSaleSubstantiallyIdentical` (the classic), `OffsettingNotionalPrincipalContract` (total-return swap), `FuturesContractSubstantiallyIdentical`, `ForwardContractSubstantiallyIdentical`, `CombinedPositionsSameEconomicEffect` (collar / synthetic short), plus two non-triggering categories: `Section1256Contract` (exempt under §1259(c)(3)(C) because §1256 already marks to market) and `NoCoveredTransaction` (standalone protective put at OTM strike, etc.).

The **§1259(c)(3)(A) safe harbor** lets a hedge escape if ALL three conditions hold: (1) hedge closed **before January 30** of the next year (= 30 days after Dec 31); (2) taxpayer holds the long position throughout the **60-day window** after closing the hedge; (3) **no risk reduction** on the long during that 60-day window (no replacement hedge, no protective put bought, no new offsetting position). Missing any one condition triggers the constructive sale retroactively to the hedge entry date — not the close date. Character is LTCG vs STCG based on whether the long position had been held > 1 year at the hedge entry date, computed using calendar-month arithmetic (`checked_add_months(12)`) so leap years don't shift the boundary.

Mounted at `POST /api/calc/section-1259`. Nineteen tests pin: classic short-against-box triggers + $30k LTCG + $80k basis step-up + new holding period from hedge date; short-term holding yields STCG; **exactly one year is short-term** (leap-year-safe calendar math); safe harbor with all three conditions = no trigger; safe harbor failures (missing 60-day window / risk reduction / late close) each trigger; loss position no trigger (§1259(b)(1) requires appreciation); break-even no trigger; §1256 contract exempt under §1259(c)(3)(C); NoCoveredTransaction no trigger; offsetting NPC triggers; forward / futures / combined positions each trigger; basis step-up equals gain recognized invariant; safe harbor preserves original basis + no new holding period; appreciation = FMV - basis; loss position with failed safe harbor still no trigger.

`traderview-expense::section_871m` is the **IRC §871(m) dividend-equivalent withholding module** — companion to iter 32's `section_864b2`. That module handles the non-US trader's own-account safe harbor (avoiding ECI classification); this module handles the **30% withholding** the broker imposes on dividend-equivalent payments from US-equity-linked derivatives, regardless of safe-harbor status. The two analyses are INDEPENDENT — a non-US person qualifying under §864(b)(2) still owes §871(m) withholding on their derivative dividend equivalents.

Pre-§871(m), non-US persons used total-return swaps and other equity derivatives to receive dividend-equivalent payments on US stocks WITHOUT triggering the §871(a) / §881 FDAP withholding that an actual dividend would trigger. Congress closed the loophole through §871(m) (enacted 2010, effective 2014 for swaps, 2017 for listed options).

**Specified Equity-Linked Instrument** (SELI) classification per Reg. §1.871-15(g) follows the **delta test**:

- **Short-term contracts** (original term ≤ 365 days) effective 2017+: SELI when **delta ≥ 0.80** at issuance. Classic near-the-money equity options pass; deep OTM doesn't.
- **Long-term contracts** (original term > 365 days): SELI only when **delta = 1.0** at issuance. Deep-ITM LEAPS that functionally hold the stock are caught; standard delta-0.6 LEAPS are not.

**Statutory rate** is **30%** under §871(a)(1)(A) / §881(a)(1). Tax treaties typically REDUCE the rate to **15%** (US-Canada, US-UK, US-Germany, US-Japan, US-Switzerland, US-Netherlands) when the recipient files Form W-8BEN with the broker. `treaty_rate_override` accepts 0.0 (full exemption — rare) to 1.0 (clamped) to handle the full treaty-rate spectrum.

`InstrumentType` enum covers `ListedEquityOption`, `SingleStockFuture`, `TotalReturnSwap` (subject from 2014), `StructuredNote`, `OtherEquityLinked`. The withholding agent per §1441 + Reg. §1.871-15(p) is the broker/counterparty paying the dividend-equivalent — the non-US recipient just sees the net amount. This module helps the recipient verify the withheld amount matches statute (or treaty).

Mounted at `POST /api/calc/section-871m`. Seventeen tests pin: short-term delta 0.85 subject to §871(m) at 30% ($200 × 30% = $60 withheld, $140 net); short-term delta exactly 0.80 subject (boundary); short-term delta 0.79 NOT subject; long-term delta 0.90 (2-year LEAPS) NOT subject (needs 1.0); long-term delta exactly 1.0 subject; US-person recipient not subject (short-circuit first); non-dividend-paying underlying skips §871(m); 15% treaty rate overrides statutory 30%; treaty rate zero full exemption ($200 net to recipient); treaty rate above 1.0 clamps; delta above 1.0 clamps; delta negative clamped to zero not subject; zero dividend equivalent no-withholding path; **short-term boundary at exactly 365 days uses short-term threshold**; **long-term boundary at 366 days uses long-term threshold**; note distinguishes subject vs inapplicable paths; US-person short-circuit runs first (even with other bad facts).

`traderview-expense::section_864b2` is the **IRC §864(b)(2) trader / investor safe harbor module** — the rule that lets non-US persons trade US securities through US brokers without being treated as engaged in a US trade or business. Without §864(b)(2), every gain would be **Effectively Connected Income** (ECI) under §871/§882, taxed on a net basis at graduated rates with US return filing required. The safe harbor pulls trader-style activity entirely outside the US tax net for non-US persons.

The classification follows a strict four-factor short-circuit chain:

1. **§864(b)(2) is for non-US persons only** — US persons are already subject to net US tax; the safe harbor doesn't apply (`non_us_person: false` → immediate ECI).
2. **Trading must be for own account** — proprietary trading only. Trading for customers (or a customer-facing book) doesn't qualify (`trades_for_own_account: false` → ECI).
3. **No US office** — §864(c)(5) attributes income to a US trade or business when a fixed place of business in the US is used for the activity, regardless of whether the §864(b)(2) safe harbor would otherwise apply (`has_us_office_for_trading: true` → ECI under §864(c)(5)).
4. **§864(b)(2)(B) dealer exclusion** — a "dealer in stocks or securities" (or commodity dealer) does NOT qualify per Reg. §1.864-2(c)(2). Applied separately per `InstrumentClass`: a securities dealer who trades commodities for own account still gets the commodities safe harbor under §864(b)(2)(A)(i); a commodities dealer who trades securities still gets §864(b)(2)(A)(ii).

`InstrumentClass` is `Securities` (§864(b)(2)(A)(ii)) / `Commodities` (§864(b)(2)(A)(i)) / `Both` (§864(b)(2)(A) generic). The result's `controlling_subsection` echoes the exact subsection that authorizes the classification, so callers can cite it on filings or in correspondence with the IRS.

Mounted at `POST /api/calc/section-864b2`. Seventeen tests pin: non-US individual securities trading qualifies; US person doesn't qualify (US-person check runs first); non-own-account doesn't qualify; securities dealer excluded under §864(b)(2)(B); commodities dealer excluded when trading commodities; **cross-class dealer status is irrelevant** (securities dealer trading commodities-only still qualifies under §864(b)(2)(A)(i); commodities dealer trading securities-only still qualifies); both classes + dealer in one disqualifies; both classes + no dealer qualifies under §864(b)(2)(A); US office kicks out under §864(c)(5); US office overrides even an otherwise-clean profile; note distinguishes safe-harbor path from disqualification; both-class dual-dealer lists both reasons; US-person check short-circuits other factors (first reason is non-US); commodities-only + commodities dealer disqualified; safe-harbor note cites applicable subsection (§864(b)(2)(A)(ii) for securities-only, (i) for commodities-only, (A) for both); own-account check short-circuits before dealer check.

`traderview-expense::section_988` is the **IRC §988 foreign currency transaction character module** — every forex trader, every crypto trader using non-USD pairs, every holder of FX-denominated debt instruments faces §988. The default rule: gains/losses are **ordinary** income/loss, not capital. Three interacting carve-outs make the routing non-obvious:

- **§988(c)(1)(D) personal-use exclusion** — gain (not loss) on a personal currency transaction is excluded if total gain is ≤ $200 per transaction. Travelers buying euros for vacation routinely qualify. The threshold doesn't graduate — $201 of gain is fully ordinary; the whole amount, not just the excess. Personal-use LOSSES are §165(c) nondeductible personal losses, NOT §988 ordinary losses.
- **§988(a)(1)(B) capital election** — taxpayer may elect to treat forward/futures/option contracts that are capital assets (and not part of a straddle) as CAPITAL gain/loss. Election requires clear identification on the books before close of trading on the trade date per Reg. §1.988-3(b). Does NOT apply to spot, FX-denominated debt, or accrued FX items — those stay ordinary regardless.
- **§1256(g) interaction** — regulated futures contracts that are "foreign currency contracts" within §1256(g)(2) default to §1256 **60% LTCG / 40% STCG**, NOT §988. The taxpayer can use the **§988(c)(1)(D)(i) kick-out election** to override and route them BACK to §988 ordinary treatment.

`TransactionKind` enumerates seven §988(c)(1)(B) categories: `ForexSpot`, `ForwardContract`, `NonRegulatedFuturesContract`, `ForexFuturesSection1256g`, `OptionContract`, `FxDenominatedDebt`, `AccruedFxItem`. `Character` covers five outcomes: `Ordinary`, `Capital`, `Section1256Sixty40`, `ExcludedPersonalUse`, `DisallowedPersonalLoss`.

Mounted at `POST /api/calc/section-988`. Seventeen tests pin: spot default ordinary; loss on spot also ordinary (bypasses the $3k §1212(b) cap); forward + election = capital; forward without election = ordinary; regulated currency futures default to §1256 60/40; kick-out election routes futures back to ordinary; personal-use gain under $200 excluded; exactly $200 excluded; $201 fully ordinary (no graduation); personal-use loss disallowed (not ordinary); FX-denominated debt always ordinary even with election asserted (election doesn't apply); forex spot election ignored; option + election = capital; personal-use zero gain no-op; personal-use route runs first (election flags ignored); accrued FX item ordinary even with election; non-regulated futures + election = capital.

`traderview-expense::section_263a` is the **IRC §263A UNICAP trader-vs-dealer classifier** — the module that pins the load-bearing distinction between traders and dealers for cost-capitalization purposes. A **dealer in securities** under §475(c)(1) (buys + sells to customers in the ordinary course) holds securities as **inventory** and must capitalize direct + indirect costs into basis. A **trader** (proprietary trading for own account; no customers) holds positions as **investment property** and is EXEMPT from §263A under §263A(c)(3) + §475(f) — costs remain currently deductible as §162 ordinary business expenses.

The trader exemption is the most-asked-about distinction in active-trader tax planning, and the module's short-circuit chain reflects the analysis order a CPA actually performs: (1) trader → exempt, costs currently deductible (most common path for proprietary traders); (2) investor → direct costs capitalized to basis per §1012, indirect costs §67(g)-limited (the TCJA 2018-2025 suspension makes them effectively nondeductible); (3) dealer + §263A(b)(2)(B) **small business exception** (avg 3-year gross receipts ≤ §448(c) threshold) → exempt, costs currently deductible; (4) dealer above threshold → subject to UNICAP, capitalize direct + indirect into inventory basis.

`TradingClassification` enum: `Dealer`, `Trader`, `Investor`. The dealer-vs-trader question turns on whether the taxpayer makes "regular and continuous sales to customers" — caller's responsibility to assert based on facts. Active securities dealers almost always blow past the small-business threshold (gross receipts include gross proceeds from every sale), so the exception rarely helps; but day-1 trading startups may briefly qualify.

The **§448(c) threshold table** (shared with iter 16's `section_163j`):

| Year | §448(c) threshold |
|------|-------------------|
| 2020 | $26M              |
| 2021 | $26M              |
| 2022 | $27M              |
| 2023 | $29M              |
| 2024 | $30M              |
| 2025 | $31M              |

Caller can override via `small_business_threshold_override` for 2026+.

Mounted at `POST /api/calc/section-263a`. Sixteen tests pin: dealer above threshold subject to UNICAP ($150k capitalized); dealer below threshold exempt currently deductible; dealer at threshold exactly still exempt (≤ not <); dealer $1 over loses exemption; **trader exempt regardless of receipts** (even $100M — short-circuits before threshold check); investor costs capitalized to basis not currently deductible (§1012 path); §448(c) threshold table 2020-2025 each year exact; caller override beats embedded table; zero costs dealer subject but nothing to capitalize; only direct costs only those capitalized; only indirect costs only those capitalized; trader note distinguishes from dealer path (§475(f) trader vs §263A applies); investor note describes basis-capitalization path (§1012); small-business exception with huge costs still currently deductible ($2.5M deductible at $5M gross receipts); dealer subject total = sum of buckets invariant; trader short-circuits threshold check.

`traderview-expense::section_267` is the **IRC §267 related-party loss disallowance module** — every trader has family members or controlled entities, and selling stock at a loss to a spouse, child, sibling, or one's own S-corp triggers §267(a)(1) which COMPLETELY disallows the loss. The §267(d) buyer-side adjustment is the non-obvious part: when the related-party buyer later sells the property at a gain, that gain is REDUCED (down to zero) by the previously-disallowed loss. If the buyer sells at a loss, the seller's disallowed amount is gone permanently. Buyer's initial basis is their actual cash purchase price (§267 does NOT transfer the seller's basis — it only preserves the gain-reduction right).

`RelationshipCategory` exposes the §267(b) ten categories explicitly so the API caller doesn't have to guess: `FamilyMember` (§267(b)(1) — siblings whole/half blood, spouse, ancestors, lineal descendants; explicitly NOT in-laws/cousins/aunts/uncles); `IndividualAndControlledCorp` (§267(b)(2) — >50% stock value); `TwoControlledCorps` (§267(b)(3) — §1563(a) common ownership); `GrantorAndTrustFiduciary` / `TwoTrustFiduciariesSameGrantor` / `TrustFiduciaryAndBeneficiary` / `TrustFiduciaryAndOtherBeneficiary` (§267(b)(4)-(7) trust pairs); `CorpAndPartnershipCommonOwner` (§267(b)(8) — frequent gotcha for trader LLC + S-corp combos); `TwoSCorps` (§267(b)(9)); `EstateExecutorAndBeneficiary` (§267(b)(10)); and `Unrelated` to short-circuit when §267 doesn't apply.

Mounted at `POST /api/calc/section-267`. Fourteen tests pin: unrelated full loss recognized; family member full loss disallowed; no-loss no-op; §267(d) buyer gain reduced by disallowed loss ($20k gain → $10k taxable after $10k reduction); §267(d) reduction capped at buyer gain ($3k gain → $7k permanently lost); buyer loss loses entire disallowance; buyer not yet sold preserves disallowance for future; all ten §267(b) categories treated as related; buyer initial basis is cash price (not seller basis); §267(d) zero gain leaves loss lost; unrelated with subsequent gain ignores §267(d); corp+partnership common owner is related (trader LLC ↔ S-corp); is-related helper returns false only for Unrelated; note text describes the partial-reduction split.

`traderview-expense::section_163d` is the **IRC §163(d) investment interest expense limitation** — the §163(j) equivalent for non-trader investors (anyone NOT making the §475(f) trader-in-securities election). Margin interest is deductible only up to **net investment income** under §163(d)(1); excess carries forward **indefinitely** under §163(d)(2). Pairs with iter 16's `section_163j` — together they cover the two §163 limitation paths a margin-debt taxpayer can hit.

Net investment income per §163(d)(4) sums: **interest income** (always counted), **ordinary dividends** (always), **net short-term capital gain** (always — STCG is ordinary regardless), and OPTIONALLY **qualified dividends** + **net long-term capital gain** if the taxpayer makes the **§1(h)(11)(D)(i)** / **§163(d)(4)(B)(iii)** election to treat them as investment income. Investment expenses other than interest (§163(d)(4)(C) — e.g. portion of management fees) reduce the NII figure.

The QD/LTCG election is the **non-obvious tradeoff**: a taxpayer with unused QDs/LTCG can elect them into NII to unlock the interest deduction NOW, but pays **ordinary-income rates** instead of preferential capital-gain rates on the elected portion. The module surfaces `qualified_dividends_lost_preferential_rate` + `long_term_capital_gain_lost_preferential_rate` so the caller can compare: ordinary rate × elected QD vs deferred interest deduction value. Worth it when the elected interest deduction × ordinary rate exceeds the QD/LTCG preferential-rate savings.

Mounted at `POST /api/calc/section-163d`. Seventeen tests pin: baseline without elections uses interest + ordinary dividends + STCG only ($3k + $1k + $2k = $6k NII); QD election boosts NII to $11k + $5k forfeits preferential rate; LTCG election adds $8k + $8k forfeits; both elections stack ($19k NII); other investment expenses reduce NII (gross $6k - expenses $2k = $4k); NII cannot go negative when other expenses exceed gross (clamped to zero, full carryforward); prior carryforward stacks with current expense; no expense no-op; expense fully under NII no carryforward; negative STCG treated as zero in NII (won't reduce other income); no-QD-election keeps QD at preferential rate (`qualified_dividends_lost_preferential_rate = 0`); multi-year chain absorbs carryforward when NII rises; note text reflects election tradeoff when applicable; zero income zero NII full carryforward; NII equals gross when no other expenses; carryforward never negative under stress; election with zero amount doesn't artificially increase NII.

`traderview-expense::section_163j` is the **IRC §163(j) business interest limitation** — caps deductible margin interest for traders who've elected §475(f) trader-in-securities status. Active traders pay 5-figure margin interest annually; without §163(j) modeling, they over-deduct and face audit/penalty exposure. The deduction limit is `business_interest_income + floor_plan_financing_interest + 0.30 × adjusted_taxable_income`, with anything above carrying forward **indefinitely** under §163(j)(2).

Adjusted Taxable Income (§163(j)(8)) is taxable income computed WITHOUT regard to business interest expense or income, NOL deduction, or §199A QBI. For tax years before 2022, depreciation/amortization/depletion were also added back; for 2022+, they're NOT, making the cap meaningfully tighter post-TCJA-transition.

**§163(j)(3) small-business exception**: the cap doesn't apply when the taxpayer's average annual gross receipts for the prior 3 years are at or below the §448(c) threshold. The threshold is annually indexed; the module embeds the published table:

| Year | §448(c) threshold |
|------|-------------------|
| 2020 | $26M              |
| 2021 | $26M              |
| 2022 | $27M              |
| 2023 | $29M              |
| 2024 | $30M              |
| 2025 | $31M              |

For 2026+, caller passes `small_business_threshold_override` with the current IRS-published figure. Note: active traders almost always blow past this — gross receipts = gross sale proceeds — so the exception rarely helps. Day-1 traders may briefly qualify.

Mounted at `POST /api/calc/section-163j`. Fifteen tests pin: standard 30% cap partial deduction ($50k expense, $100k ATI → $30k deducted, $20k carries); expense below cap fully deducted; business interest income raises cap dollar-for-dollar; prior carryforward stacks; small-business under threshold fully exempt; at threshold exactly still exempt (≤ not <); $1 over loses exemption; **threshold table** 2020-2025 each year exact; caller override beats embedded table; negative ATI caps 30% at zero (only BI income + floor plan in cap); no-expense no-op; floor plan financing adds to cap; multi-year chain absorbs carryforward when ATI rises; full-deduction note vs carries-forward note.

`traderview-expense::section_1202` is the **IRC §1202 Qualified Small Business Stock (QSBS) gain-exclusion module** — the most-missed tax break for founders, employees with exit stock, and active traders buying primary-issuance small-company stock. Up to **the GREATER of $10,000,000 OR 10× the taxpayer's adjusted basis** of gain on QSBS is excluded from federal income tax. Paired with `section_1244`: §1244 handles the LOSS side (ordinary-loss treatment up to $50k/$100k), §1202 handles the GAIN side (exclusion up to $10M / 10× basis). This is the mechanism behind the "Peter Thiel $5B Roth" — qualified stock acquired cheaply, held > 5 years, sold for 9-figures, federal tax = $0.

The exclusion percentage is a three-band step function on acquisition date:
- Acquired **before Feb 18, 2009** → 50% exclusion + 7% AMT preference on the excluded portion (§57(a)(7)).
- Acquired **Feb 18, 2009 – Sep 27, 2010** → 75% exclusion + 7% AMT preference.
- Acquired **after Sep 27, 2010** → 100% exclusion, no AMT preference.

The §1202(c) + §1202(e) qualification checklist surfaces as 8 booleans on `Qsbs1202Qualification`: domestic C-corporation; aggregate gross assets ≤ $50M at issuance; original issuance to taxpayer (no secondary market); non-corporate taxpayer (C-corps cannot use §1202); holding period > 5 years; ≥ 80% of corp assets in active qualified trade or business; not an §1202(e)(3) excluded business (health / law / engineering / accounting / consulting / financial services / brokerage / banking / insurance / farming / hotels / restaurants / mineral extraction / any business where principal asset is the reputation of one or more employees); not §1202(f) preferred-stock-as-debt disqualified. Per-issuer cap is per-taxpayer lifetime; the `prior_exclusion_used_this_issuer` input shrinks the remaining cap so multi-tranche sales of the same issuer stack correctly.

Mounted at `POST /api/calc/section-1202`. Eighteen tests pin: post-2010 full 100% exclusion zero AMT, pre-2009 50% band with 7% AMT, mid-band 75% with 7% AMT, all three band boundaries (Feb 17 vs Feb 18 2009 and Sep 27 vs Sep 28 2010), cap uses MAX(10M, 10× basis), over-cap portion taxable LTCG, prior exclusion reduces cap remaining, each disqualification path routes full gain to LTCG, excluded business listed in note, no-gain no-op, loss returns no-op (not negative exclusion), corporate taxpayer disqualified, multi-disqualification lists all failures, qualification helper returns true only when all 8 pass, full exclusion at cap with 100% band zero AMT. §1045 60-day rollover is out of scope — caller handles by reducing `realized_gain` upstream.

`traderview-expense::section_1244` is the **IRC §1244 small-business-stock-loss module** — bypasses the §1212(b) $3,000 capital-loss cap for losses on qualifying small-business stock. The first **$50,000 single / $100,000 MFJ** of such loss in any tax year is treated as **ordinary loss** (Schedule 1 Line 8z, not Schedule D), absorbing dollar-for-dollar against ordinary income with no cap. Anything above the threshold overflows back to capital loss treatment on Schedule D.

The §1244(c) qualification checklist surfaces explicitly as 5 booleans on the `Qualification` struct so the caller (and downstream UI) can show exactly which tests pass: stock from a **domestic** corporation; aggregate paid-in capital + paid-in surplus **≤ $1M at issuance**; for the 5 years before the loss **< 50% of gross receipts from passive sources** (royalties / rents / dividends / interest / annuities / sales of stock or securities); stock **issued for money or other property** (not services); taxpayer is the **original holder** (no inherited, gifted, or secondary-market stock). `Qualification::qualifies()` returns true only when all five pass; the result note enumerates the failing tests when one or more fail. Mounted at `POST /api/calc/section-1244`. Fourteen tests pin: single under-cap, single over-cap with capital overflow, MFJ $100k cap, MFS uses $50k (not 50% of MFJ), prior-claimed reduces remaining, each disqualification routes the full loss to capital, multi-disqualification listed in note, no-loss no-op, cap-stays-non-negative under stress, exact-cap edge, qualification helper returns true only when all five pass.

`traderview-expense::section_121` is the **IRC §121 principal-residence sale exclusion** — companion to `disposition.rs` (iter 7 covers rental sale; §121 covers personal-residence sale). Up to **$250k single / $500k MFJ** of gain on the sale of a principal residence is excluded from gross income when the 2-of-5-year **ownership test** and **use test** both pass, AND the taxpayer hasn't used §121 on another sale within the prior 2 years. Mounted at `POST /api/calc/section-121`.

Three carve-outs / haircuts the module handles end-to-end:

- **§121(b)(4) reduced maximum exclusion** — if the 2-year tests failed due to (a) change in employment, (b) health, or (c) unforeseen circumstances, the cap is pro-rated by `qualifying_months / 24` where qualifying months = `min(months_owned, months_used).min(24)`. A 12-month qualifier excludes $125k single / $250k MFJ. An 18-month job-move excludes $187,500.
- **§121(b)(5) non-qualified use** — applies to post-2008 dispositions. Any period AFTER 2008 during which the property was NOT the principal residence (e.g. rental years before conversion to primary residence) reduces eligible gain proportionally: `eligible = realized × (qualified_days / total_ownership_days_post_2008)`. The NQU portion is taxable LTCG even though the rest of the gain is excluded.
- **§121(d)(6) depreciation recapture** — any depreciation deducted after May 6, 1997 (home office, prior rental) is NOT excludable. It's recaptured as **§1250 unrecaptured gain** at the 25% rate. The module surfaces it as a separate `unrecaptured_section_1250` field, with `taxable_long_term_gain` for the LTCG bucket and `total_taxable_gain` for the sum across rate brackets.

Fifteen tests pin: single full qualifier under cap → full exclusion; MFJ $500k cap; over-cap portion taxable LTCG; failed 2-year tests with no §121(b)(4) reason → fully disqualified with reason list; health-reason pro-rates cap to $125k at 12 months; job-move pro-rates to $187,500 at 18 months; §121(b)(4) uses lesser of owned/used months; once-every-2-years blocks; §121(b)(5) NQU proportional reduction with exact day-count math; §121(d)(6) recapture before exclusion (LTCG=0 if cap absorbs, recapture still taxable); loss on sale not excludable but recognized; MFS uses $250k (not half of MFJ); combined recapture + NQU + over-cap all stack into `total_taxable_gain`; zero post-2008 ownership skips NQU; note text full-exclusion path.

`traderview-expense::section_174` is the **IRC §174 R&D capitalization module** — the post-TCJA amendment that hit every algorithmic trader hard starting in tax year 2022. Before TCJA, R&D expenditures could be either expensed currently OR capitalized + amortized over 5+ years at taxpayer's election under §174(a)(1). After TCJA, current expensing is **GONE**: all R&E expenditures MUST be capitalized and amortized over **5 years (domestic)** under §174(a)(2)(A) or **15 years (foreign)** under §174(a)(2)(B), with a **half-year convention** that spreads recovery across 6 calendar years (or 16 for foreign).

The cash impact: an algorithmic trader who spends **$100,000 on internal trading-software development in 2024** gets only **$10,000 deductible in 2024** (vs the full $100k pre-TCJA). The remaining $90,000 sits on the balance sheet as capitalized basis, deducted over the next 5 calendar years: $20k each in 2025-2028 plus a $10k stub in 2029. Software development is explicitly within §174 per Rev. Proc. 2000-50 + TCJA committee report — so any trader who writes algorithms in-house bears this cost.

Schedule structure per §174(a)(2) half-year convention:
- Year 1 (year of expenditure): half a year's amortization = `(amount / life) × 0.5`
- Years 2 through `life`: full year's amortization = `amount / life`
- Year `life + 1` (stub): the remaining half = `(amount / life) × 0.5`

For domestic 5-year: 6 calendar years touched. For foreign 15-year: 16 calendar years touched. The schedule sum invariant holds (sum of all entries = total R&D amount).

§174 covers research or experimental expenditures including software development. Excluded (still §162 ordinary expense): routine business operations, market research, advertising, sales promotion, ordinary testing of prototypes. The classification is fact-intensive; caller asserts via the input. **Pre-2022 expenditures** still get the pre-TCJA expensing option — the module flags `pre_tcja_expensing_available: true` on year-of-expenditure < 2022 so the user knows they had a choice.

Mounted at `POST /api/calc/section-174`. Sixteen tests pin: domestic 5-year year 1 half-year convention ($100k → $10k year 1); domestic year 2 full year ($20k); domestic year 6 stub half ($10k, total cumulative $100k, fully amortized); domestic year 7 post-recovery zero; schedule has 6 entries for 5-year recovery (years 2024-2029); schedule amounts correctly distributed (10/20/20/20/20/10 sum to 100); foreign 15-year year 1 half-year convention ($3,333.34); foreign schedule has 16 entries; pre-2022 flags expensing available; post-2022 no expensing option; zero amount no-op; before expenditure year zero deduction; cumulative grows predictably (10/30/50/70/90/100); RDLocation helper returns 5 / 15; **algorithmic trader software dev $100k year 1 only $10k deductible** (the load-bearing TCJA-hit scenario); 5-year recovery sum ties to full amount under arbitrary $250k input.

`traderview-expense::section_168_e6` is the **IRC §168(e)(6) Qualified Improvement Property module** — the 15-year MACRS class for interior improvements to nonresidential buildings made by the taxpayer after the building was originally placed in service. Critical for commercial landlords planning tenant build-out allowances.

**Drafting-error saga** worth modeling because it changed the recovery period mid-history:
- **TCJA 2017** intended QIP to be 15-year property eligible for §168(k) bonus depreciation. The bill accidentally omitted the recovery-period assignment, so QIP placed in service 2018-2019 defaulted to **39-year** real property under §168(c) — and was NOT bonus eligible.
- **CARES Act 2020** (P.L. 116-136 §2307) retroactively fixed the drafting error effective for property placed in service after Dec 31, 2017 — assigned **15-year** recovery and restored §168(k) bonus eligibility. Taxpayers could file Form 3115 (accounting method change, paired with iter 28's `section_481`) to recover missed bonus depreciation as a §481(a) adjustment.

§168(e)(6) **definition**: any improvement made by the taxpayer to an interior portion of a building which is nonresidential real property, IF such improvement is placed in service after the date such building was first placed in service. **Three categories are explicitly EXCLUDED** (revert to 39-year nonresidential recovery):

1. **§168(e)(6)(A)** — building enlargement / addition.
2. **§168(e)(6)(B)** — any elevator or escalator.
3. **§168(e)(6)(C)** — internal structural framework.

The `ImprovementType` enum exposes the exclusion categories explicitly so the caller doesn't have to guess: `InteriorNonresidential` (the QIP general case), `BuildingEnlargement`, `ElevatorOrEscalator`, `InternalStructuralFramework`, `ResidentialRental` (QIP is nonresidential-only by definition). The `placed_in_service_year > building_first_placed_in_service_year` test enforces the "after the date such building was first placed in service" requirement — a same-year improvement to a newly-constructed building doesn't qualify.

§168(k) bonus phase-down shared with iter 11's `cost_segregation` (2018-2022 = 100%, 2023 = 80%, 2024 = 60%, 2025 = 40%, 2026 = 20%, 2027+ = 0%). When elected, year 1 = `bonus_pct × cost + 5% × (cost − bonus)`. Without bonus or when QIP doesn't qualify, year 1 falls to 5% (15-year MACRS half-year) or 1.282% (39-year nonresidential mid-month half-year approximation) depending on which path applies.

Mounted at `POST /api/calc/section-168-e6`. Eighteen tests pin: interior nonresidential qualifies as QIP; QIP 2024 60% bonus year 1 ($60k + $2k = $62k); QIP 2022 100% bonus full year 1 ($100k); QIP 2023 80% bonus phase-down ($81k); QIP 2027 zero bonus phase-down ($5k year 1); building enlargement excluded → 39-year; elevator/escalator excluded; internal structural framework excluded; residential rental not QIP by definition; **improvement in same year as building placed in service not QIP** (must be AFTER); improvement before building placed in service not QIP; excluded category uses 39-year MACRS ($1,282 year 1); no bonus election → MACRS half-year only ($5k); bonus phase-down 2023-2027 each year exact; helper returns true only for interior nonresidential; CARES Act fix verified for years 2018-2021 (15-year recovery); note distinguishes QIP path vs excluded path; total deduction equals bonus + MACRS invariant.

`traderview-expense::section_168g` is the **IRC §168(g) Alternative Depreciation System (ADS) + §163(j)(7)(B) tradeoff analyzer** — the natural companion to iter 16's `section_163j`. A landlord with high mortgage interest hitting the §163(j) 30%-of-ATI cap can elect to be an **electing real property trade or business** under §163(j)(7)(B): full business interest deductibility (no §163(j) cap), BUT must use slower ADS depreciation on all real property forever — the election is **IRREVOCABLE**.

ADS recovery periods (§168(g)(2)): **30 years** residential (post-TCJA; 40 years pre-2018), **40 years** non-residential, **20 years** for Qualified Improvement Property at electing RPTBs, plus personal-property classes (5/7/15 years). Method is **straight-line** (no double-declining acceleration). Convention is **mid-month** for real property, **half-year** for personal property. Bonus depreciation per §168(k)(2)(D)(i) is NOT allowed on ADS property — another giveup beyond the longer recovery period.

The compute function returns both the ADS annual deduction AND a GDS comparison (straight-line, same convention) so callers can sum the per-year depreciation difference across all real property and feed it into the tradeoff analyzer. `analyze_tradeoff` takes `annual_depreciation_sacrificed` + `annual_interest_disallowed_under_163j` + `marginal_federal_rate`, converts each to after-tax dollars, and returns `net_annual_benefit` + `election_recommended` + a note reminding the user that the election is irrevocable and requires a multi-decade horizon model before committing.

Mounted at `POST /api/calc/section-168g` and `POST /api/calc/section-163j-tradeoff`. Eighteen tests pin: residential 30-year year-2 full-year at 1/30; mid-month January year-1 = 11.5/12 of full year; mid-month December year-1 smallest; year-31 stub recovers the leftover from year-1 partial; commercial 40-year longer recovery; QIP 20-year shortest real-property recovery; personal 5-year half-year convention; personal 5-year stub (year 6); residential 30y vs GDS 27.5y difference positive; pre-service year returns zero; zero basis no-op; residential 40-year legacy; recovery-period helper matches class; is_real_property helper; tradeoff election worth it when interest savings exceed depreciation loss; tradeoff not recommended when reversed; tradeoff zero interest disallowed → election pointless; tradeoff scales with marginal rate.

`traderview-expense::section_280f` is the **IRC §280F luxury auto depreciation cap module** — caps annual depreciation on passenger autos used in a trade or business. Highly practical for landlords driving to properties: without the §280F(a)(1) cap, MACRS 5-year on a $100k vehicle would generate $20,000 of year-1 depreciation; §280F limits 2024 year-1 to $12,400 (no bonus) or $20,400 (with §168(k) bonus) — a $7,600 cliff that goes to the depreciation "tail" past the recovery period per Reg. §1.280F-2T.

The published Rev. Proc. cap table is statically encoded for placed-in-service years **2020-2024** (Rev. Proc. 2020-37 / 2021-31 / 2022-17 / 2023-14 / 2024-13), each with year-1-with-bonus / year-1-no-bonus / year-2 / year-3 / year-4-plus values. For years outside the static table (2025+ at time of writing), caller passes `caller_override: Some(PassengerAutoCaps)` with the current IRS-published values; the module surfaces "no published §280F caps on file" without guessing when both the table miss and override are absent. The `rev_proc_citation` field is preserved through the result for audit display.

Three structural elements:
- **Year-by-year cap routing** — different caps for year-1-bonus / year-1-no-bonus / year-2 / year-3 / year-4-through-end-of-recovery. Year 5 and 6 use the year-4-plus cap.
- **§280F(d)(5) heavy-vehicle carve-out** — vehicles over **6,000 lbs GVWR** (large SUVs, trucks, commercial vans) escape the passenger-auto definition entirely. The module skips cap computation and returns the unmodified MACRS deduction.
- **Business-use percentage scaling** — caps apply proportionally per §280F(b)(1). 60% business use in 2024 → year-1 cap = $20,400 × 0.60 = $12,240. Values above 1.0 clamp to 1.0; below 0 clamp to 0.

Mounted at `POST /api/calc/section-280f`. Seventeen tests pin: year-1 no bonus 2024 caps at $12,400 (under MACRS $12,000 → no cap); year-1 with bonus 2024 caps at $20,400; year-2 caps at $19,800; year-3 caps at $11,900; year-4 caps at $7,160; expensive vehicle ($150k) shows $17,600 capped-amount-lost; **heavy vehicle (>6000 lb GVWR) skips cap entirely** ($100k SUV gets full $20,000 year-1); business-use 60% scales cap proportionally; zero business use no deduction; above-one business use clamps to one; published caps table 2020-2024 each year exact year-1-no-bonus; unknown year (2099) returns None + caller_override path takes precedence; **caller_override beats published table** for known years; pre-service year no deduction; MACRS rates match Pub 946 Table A-1 (20/32/19.20/11.52/11.52/5.76%); year 5 and 6 use year-4-plus cap; capped_amount_lost calculated correctly.

`traderview-expense::cost_segregation` is the **cost-seg-study + §168(k) bonus depreciation accelerator** — the strategy that converts a $500k STR purchase into a $100k+ first-year tax shield when paired with §280A short-term-rental + material participation from `section_280a`. A landlord who buys a $500k residential rental and depreciates it as a single 27.5-year asset gets ~$9k/year. Run cost seg + bonus and year-1 jumps to ~$150k.

The module breaks the depreciable basis into FIVE MACRS class buckets per the typical industry breakdown for the property type:

| Type           | 5y | 7y  | 15y | 27.5y | 39y |
|----------------|----|-----|-----|-------|-----|
| SingleFamily   | 5% | 0%  | 10% | 85%   | 0%  |
| MultiFamily    | 10%| 5%  | 15% | 70%   | 0%  |
| ShortTermRental| 25%| 10% | 10% | 55%   | 0%  |
| Commercial     | 5% | 5%  | 15% | 0%    | 75% |
| Restaurant     | 30%| 0%  | 15% | 0%    | 55% |

Callers can override via `allocation_override` (the study's actual percentages) — overrides outside ±0.005 of a sum-to-1.0 fall back to the default and surface a note. **§168(k) bonus depreciation** is applied to the 5/7/15-year buckets only (real property — 27.5 and 39 — is excluded per §168(k)(2)(A)(i)). Phase-down by tax year: 100% for 2018-2022, **80%** for 2023, **60%** for 2024, **40%** for 2025, **20%** for 2026, **0%** for 2027+. Pre-2018 stock is 50%. Year-1 deduction per bucket = `bonus × basis + (1/life × 0.5) × (basis − bonus)`. The report also returns `year_1_without_cost_seg` (straight-line baseline at 27.5y for residential or 39y for commercial) and `year_1_acceleration` so the user sees the actual boost.

Mounted at `POST /api/rental/properties/:id/cost-segregation`. Auto-fill: missing `depreciable_basis` = `purchase_price − land_value` from the property row; missing `cost_seg_type` inferred from the property's `property_type` (`vacation_short_term` → `ShortTermRental`, etc.). Seventeen tests pin: STR 2024 60% bonus pool, no-bonus-election zeroes all buckets, real-property buckets never get bonus, bonus phase-down 2023-2027 exact, pre-2018 50%, 2018-2022 100%, 2027+ zero, all 5 property-type defaults sum to 1.0, allocation override with bad sum falls back to default, override within tolerance used, commercial uses 39y baseline, residential uses 27.5y baseline, zero-basis edge, STR acceleration > 5× baseline, restaurant 30% 5y bucket largest, bucket_year_1_total = bonus + macrs identity, allocation sum helper round-trip.

`traderview-expense::section_280a_d2` is the **IRC §280A(d)(2) related-party rental personal-use classifier** — the §267 analog on the rental-income side. Renting your property to a family member at below-market rent is one of the most-common landlord tax mistakes: §280A(d)(2)(A) treats ANY use by the taxpayer or a related party as **personal use** regardless of rent paid, and those days flow into iter 9's `section_280a` which then flips the property into vacation-home classification under §280A(c)(5) — deductions capped at gross rental income, no net loss permitted.

Three carve-outs the module models:

- **§280A(d)(2)(C) flush-language exception** — related-party use does NOT count as personal use when BOTH (a) the related party uses it as their **principal residence** AND (b) rent paid is **≥ fair market rent**. Below-market rent to family kills the exception even if they live there full-time. Caller supplies `fair_market_rent_for_period` from comparables (Zillow Rent Estimate, Rentometer, etc.); when omitted (zero), the module errs to personal-use treatment.
- **§280A(d)(3) shared-equity-financing arrangement** — a co-owner residing in the property qualifies as paying fair rental even at $0 cash rent, provided the agreement meets the statutory requirements. Caller asserts via `shared_equity_arrangement: true`.
- **§280A(d)(4) repair days** — days spent performing maintenance on the property aren't personal use even when the taxpayer or family stayed there. Common save when an owner spends a week renovating between tenants.

`Occupant` enum has three variants: `Taxpayer` (always personal), `RelatedParty` (gated by §280A(d)(2)(C)/(d)(3)/(d)(4)), `Unrelated` (rental if any rent paid; personal if gratuitous). The compute function aggregates a list of `OccupancyPeriod` rows into `personal_use_days` + `rental_use_days` that feed directly into `section_280a::compute` from iter 9.

Mounted at `POST /api/rental/section-280a-d2`. Sixteen tests pin: taxpayer use always personal; unrelated paying tenant full rental; unrelated gratuitous = personal; related-party at FMV + principal residence = rental (§280A(d)(2)(C) exception); related-party below market = personal (the common trap); related-party not principal residence = personal even at FMV; shared-equity-arrangement qualifies as rental even at $0 rent; repair days don't count as personal; aggregate across multiple periods; zero-FMV reference defaults to personal (safe disposition); exactly at FMV qualifies; above FMV qualifies; one cent below FMV is personal (strict ≥, not approximate); empty input no-op; shared-equity overrides below-market; repair-day overrides taxpayer personal use.

`traderview-expense::section_280a` is the **IRC §280A vacation home / mixed-use classifier** — uses the `fair_rental_days` and `personal_use_days` fields on `rental_properties` to bucket each property into one of four classifications:

- **Rental** — `fair_rental_days ≥ 15` AND `personal_use_days ≤ MAX(14, 10% of fair_rental_days)`. Full Schedule E; §469 PAL applies separately.
- **VacationHome** — `fair_rental_days ≥ 15` AND personal use over the threshold. §280A(c)(5) caps deductions at gross rental income (no net loss); expenses allocated pro-rata between personal and rental days and tiered. **Tier 1** (mortgage interest, property tax — already deductible on Schedule A) always allowed at the rental allocation %; **Tier 2** (operating expenses: insurance, utilities, repairs, management, supplies, advertising) allowed up to remaining income after tier 1; **Tier 3** (depreciation) allowed up to remaining income after tier 1 + 2. Excess tier 2 + 3 carries forward to next year via the `prior_year_suspended` input.
- **AugustaRule** — `fair_rental_days` is 1–14. §280A(g) **tax-free rental income**: the gross income is excluded from gross income entirely (not reported), and no rental deductions are allowed. Famously used by homeowners renting to their own corporations for board meetings (corp deducts the rent, owner excludes it).
- **PersonalResidence** — `fair_rental_days = 0`. No rental activity reported.

The personal-use threshold uses the GREATER of 14 days OR 10% of fair rental days per IRS Pub 527 — so a property rented 200 days passes the rental classification if personal use ≤ 20 days, not the bare 14. Mounted at `POST /api/rental/properties/:id/section-280a` with auto-fill: missing `fair_rental_days` / `personal_use_days` are pulled from the `rental_properties` row. Sixteen tests pin: pure-rental no-personal-use; rental within threshold allocates proportionally; threshold uses max(14, 10%); rental boundary at 14 days exact stays rental; 15 personal days flips to vacation home; vacation home deductions capped at income (no loss); low-income suspends excess; Augusta Rule 14 days tax-free; Augusta boundary at 14 vs 15; personal residence zero rental days; prior suspended stacks with tier 2; 1-day rental routes to Augusta safely; allocation pct zero when both days zero; personal_use_ceiling math (10% of 100/140/200/365 days).

`traderview-expense::section_1031_f` is the **IRC §1031(f) related-party 2-year clawback module** — the anti-abuse rule that complements iter 7's `disposition` module. Plain §1031 like-kind exchange defers gain when rolling into replacement property; §1031(f) adds: if either party is related under §267(b) AND either party disposes of the property received within **2 years** of the exchange, the deferred gain is **recognized retroactively in the year of the disqualifying disposition** (not the original exchange year). The character is preserved — LTCG stays LTCG, §1250 recapture stays recapture.

§1031(f)(2) recognizes three exceptions that block retroactive recognition even within the 2-year window: **DeathOfParty** (§1031(f)(2)(A)), **InvoluntaryConversion** under §1033 (§1031(f)(2)(B)), and **LackOfTaxAvoidancePurpose** (§1031(f)(2)(C) — taxpayer establishes neither the exchange nor the disposition had tax avoidance as a principal purpose).

The 2-year window is computed via `checked_add_months(24)` for leap-year correctness — a Feb 29 2024 exchange produces a window ending Feb 28 2026 (chrono adjusts non-existent dates correctly). The window-end date is **exclusive**: a disposition exactly on the end date preserves the deferral; one day earlier triggers. When no disposition has occurred yet, the module reports `window_still_open` + `days_to_window_end` so the landlord can see their exposure runway. Past-window exchanges report "matured cleanly". `RelationshipCategory` is re-exported from iter 18's `section_267` module — all 10 §267(b) relationship classes route through the same code path.

Mounted at `POST /api/calc/section-1031-f`. Seventeen tests pin: unrelated parties — §1031(f) doesn't apply; family disposition within 2-year window triggers full retroactive recognition with disposition-year tax; disposition after window preserves deferral; **disposition exactly at window end preserves**; one day before window end triggers; each §1031(f)(2) exception blocks recognition (death / involuntary conversion / lack of tax avoidance); no-disposition open-window reports `window_still_open: true` + positive days remaining; no-disposition past-window reports "matured cleanly"; zero deferred gain no-op; all 10 §267(b) categories trigger when disposition within window; character preserved (LTCG stays LTCG, §1250 stays §1250); 2-year window uses calendar months not days (Feb 29 2024 exchange → Feb 28 2026 window-end); exception logged in note text; unrelated disposition within window no trigger (control); recognized year matches disposition year not exchange year (§1031(f)(1)(C) retroactive-in-disposition-year rule).

`traderview-expense::section_453` is the **IRC §453 installment sale gain deferral module** — landlord-relevant for seller-financed rental property sales. A $500k rental sold with 20% down + 80% seller-financed note recognizes the gain over the life of the note rather than all in year 1. Companion to iter 7's `disposition` (which handles the all-cash sale path) and iter 27's `section_1031_f` (which handles the §1031 exchange path) — together they cover the three exit strategies for rental property.

**Gross profit ratio method** per §453(c):
- `gross_profit = sale_price − selling_costs − adjusted_basis`
- `contract_price = sale_price − selling_costs − qualifying_indebtedness_capped_at_basis`
- `gross_profit_ratio = gross_profit / contract_price`, capped at 1.0
- Each year's gain = `principal_received × gross_profit_ratio`
- Interest portion is separately recognized as ordinary interest income (Form 1040 Schedule B), regardless of GPR.

Three disqualification paths:

- **§453(k) marketable securities** — installment treatment NOT available for sales of publicly traded stock or securities. This is why §453 doesn't help traders selling public stock — full recognition in the year of sale. Closely-held private company stock CAN use §453, so secondary-market private-share sales (employee buyback of pre-IPO shares, founder sell-down) do qualify.
- **§453(g) related-party 2-year resale anti-abuse** — when the buyer is a §267(b) related party (cross-references iter 18's `section_267::RelationshipCategory`) AND the buyer resells within 2 years of the original §453 sale, the ORIGINAL seller must recognize **all remaining unrecognized gain** in the year of the second sale. Pairs with iter 27's §1031(f) related-party clawback.
- **§453(d) elect out** — seller can affirmatively elect OUT of installment treatment and recognize the full gain in the year of sale. Useful when buyer creditworthiness is poor or when the seller has offsetting losses to absorb the gain.

Mounted at `POST /api/calc/section-453`. Eighteen tests pin: straight installment GPR applied correctly ($500k gain / $700k contract = 0.714286 GPR; $50k down × 0.714286 = $35,714.30 gain); marketable securities excluded with full recognition; opt-out election triggers full recognition; **§453(g) related-party 2-year resale triggers full clawback** of $350k remaining + current GPR gain; related-party without 2-year resale no clawback (selling to family is fine if they hold); loss on sale no-op (§453 only for gains); qualifying indebtedness reduces contract price; full-basis assumed-debt zero contract price; GPR capped at 1.0 when gross profit exceeds contract price; interest separately recognized as ordinary even in zero-principal years; multi-year chain eventually recognizes full gain; zero principal received zero recognition; **marketable security short-circuits other inputs** (even with §453(g) facts, §453(k) disqualifies first); both marketable + opt-out list both disqualification reasons; unrelated buyer resold no clawback (only related-party triggers); $1M business sale GPR math (0.7 GPR, $175k recognized year 1); note distinguishes §453(k) vs §453(d) paths; note distinguishes normal installment vs §453(g) clawback path.

`traderview-expense::disposition` is the **rental property disposition module** — the sale-time computation every landlord faces but generic tax software handles poorly. Realized gain decomposes into TWO buckets the IRS taxes at different rates: **§1250 unrecaptured gain** (the portion attributable to prior depreciation, capped at 25% federal) and **§1231 LTCG** (the remainder, at 0/15/20% LTCG rates). The split is `§1250 = min(accumulated_depreciation, realized_gain)`; depreciation can't recapture more gain than actually exists. Selling at a loss triggers §1231 ordinary-loss treatment with no §1250 component.

When the seller rolls into a replacement via **§1031 like-kind exchange**, gain is DEFERRED to the extent of replacement value. Boot — cash received or net debt relief — triggers recognition `MIN(realized_gain, boot_received + debt_relief_net)`. Replacement basis = `adjusted_basis + boot_paid − boot_received + gain_recognized`, carrying the deferred gain into the new property. Per §1031(c), losses are recognized in full — §1031 does not defer losses.

Wired at `POST /api/rental/properties/:id/dispose`. Caller supplies `sale_price + selling_costs + (optional) original_cost_basis + accumulated_depreciation + capital_improvements_added + like_kind_exchange`. Missing `original_cost_basis` is filled from `rental_properties.purchase_price`; missing `accumulated_depreciation` is summed from `rental_expenses` rows where `category_code = 'e_depreciation'`. Thirteen tests pin: straight-sale matches Form 4797, capital improvements raise basis lowering gain, §1250 capped at total gain (can't recapture phantom amounts), loss triggers §1231 ordinary, §1031 no-boot full deferral, §1031 boot recognized up to realized gain, §1031 boot exceeds gain caps recognition, §1031 debt-relief net counts as boot, §1031 replacement-basis carries deferred gain (`replacement_value - replacement_basis == deferred_gain`), §1031(c) losses recognized in full, max-§1250-tax estimate is 25% of unrecaptured, zero-gain edge case, no-depreciation → all §1231.

`traderview-expense::mlp_ubti` is the **MLP K-1 Unrelated Business Taxable Income tracker for IRAs and qualified plans** — the Form 990-T trap that catches traders rotating ET / KMI / MPLX / EPD / NGL through retirement accounts. Under IRC §§511-514, the IRA itself is taxable on its share of MLP operating income passed through on the K-1; the tax is paid by the IRA custodian via Form 990-T and the cash comes out of the IRA balance, eroding the retirement compounding. The broker doesn't flag this — most retail holders find out years later when a custodian deducts the tax from the account.

Routing:
- **K-1 Box 1 — ordinary business income** flows directly to UBTI (always).
- **§512(b) exclusions** — dividends (Box 6a), interest (Box 5), royalties, short-term and long-term capital gains (Boxes 8/9a) on the K-1 are **NOT UBTI** (passive investment income exempt from §511). The module surfaces these as `excluded_passive_income` per MLP so the user can verify the K-1 was read correctly.
- **§514 debt-financed UBTI** — if the MLP has acquisition indebtedness, even otherwise-excluded items become UBTI proportional to the debt-financed share. Caller supplies the dollar amount from Box 20V (the partner's footnote often spells it out separately).
- **K-1 Box 13 — deductions** allocable to UBTI activity reduce the inclusion.
- **§512(b)(12) specific deduction** — first $1,000 of total UBTI is exempt before tax applies. Caller can override for non-IRA tax-exempts with different statutory thresholds.
- **Trust-rate tax per §511(b)(2)** — IRAs and qualified plans use the **compressed trust brackets** (2024: 10% to $3,100, 24% to $11,150, 35% to $15,200, 37% over $15,200) — NOT corp 21%. A $20k UBTI year pays $5,435.50 in tax; $30k pays $8,765.50. The compressed brackets mean MLP UBTI bites hard above $15k.
- **Form 990-T threshold** — gross UBTI ≥ $1,000 triggers the filing requirement regardless of whether tax is owed after the §512(b)(12) deduction.

Mounted at `POST /api/calc/mlp-ubti`. Seventeen tests pin: single MLP below $1,000 no Form 990-T; exactly $1,000 triggers form but zero tax (specific deduction absorbs); aggregate across multiple MLPs; passive income (dividends + cap gains) excluded from UBTI; debt-financed Box 20V additive; Box 13 deductions reduce UBTI; negative UBTI doesn't create artificial deduction; 2024 trust brackets compressed correctly; trust tax at each bracket threshold; zero income zero tax; negative income clamped; corp 21% flat rate when `use_trust_brackets: false`; specific-deduction override; empty MLP list no-op; loss + passive income still zero UBTI; per-MLP breakdown preserves names; high-UBTI year ($30k) uses 37% top bracket correctly.

`traderview-expense::section_408A_d3` is the **IRC §408A(d)(3)(F) Roth conversion 5-year rule module** — the trap that catches early-retirees doing "Roth conversion ladders" (the FIRE-movement strategy). Completes the IRA-rules trio: `form_8606` (iter 12, basis + pro-rata on conversions) + `section_408_d3` (iter 40, 60-day rollover + Bobrow) + this module (5-year aging on conversions per §72(t)).

Each Roth conversion starts its own SEPARATE 5-year clock under §408A(d)(3)(F). Withdrawing converted principal before BOTH 5-year aging AND age 59½ triggers a **10% §72(t) penalty** on the converted amount (not the earnings). Distinct from the general §408A(d)(2)(B) 5-year rule for "qualified distributions" — that one applies to earnings and runs from the FIRST Roth funding (contribution or conversion).

**§408A(d)(4) ordering rules** for Roth IRA distributions — the load-bearing ordering that drives the module's bucket chain:

1. **Contributions** (regular annual contributions) come out FIRST — always tax-free + penalty-free regardless of age or holding period. The Roth's "always-accessible basis" feature.
2. **Conversions** come out next in **FIFO order** (oldest first), each subject to its OWN §408A(d)(3)(F) 5-year aging clock. Converted basis itself is always tax-free (it was taxed at conversion); the 10% penalty applies on UNAGED conversion withdrawals when under age 59½.
3. **Earnings** come out LAST — taxable + 10% penalty if before §408A(d)(2)(B) qualified-distribution threshold (5 years from first Roth funding AND age 59½).

**Age 59½ bypass**: once the taxpayer reaches age 59½ (modeled as ≥ 60 since the half-year doesn't have a clean integer representation), the §72(t) penalty disappears regardless of 5-year aging. Aged conversions and unaged conversions both become penalty-free.

Mounted at `POST /api/calc/section-408a-d3`. Eighteen tests pin: withdrawal from contributions only no tax + no penalty (always-accessible basis); aged conversion no penalty at age 45; unaged conversion triggers 10% penalty at age 45 ($10k × 10% = $1k); unaged conversion no penalty at age 60+ (age 59½ bypass); ordering contributions before conversions ($5k+$5k → contributions taken first); FIFO ordering oldest conversion first (2019 aged taken before 2022 unaged); earnings taxable + penalized when not qualified; qualified distribution age 60 + 5 years full tax-free; not qualified when under 5 years from first funding; **conversion 5-year boundary exactly 5 years aged**; conversion 4y-11m-29d not aged (1 day under); multiple conversions some aged some not (independent clocks); withdrawal exceeds all buckets caps at earnings; zero withdrawal no-op; empty account no-op; note distinguishes qualified vs ordering paths; **classic FIRE conversion ladder 5-year wait pays off** (load-bearing scenario); age 59½ + unaged conversion no penalty.

`traderview-expense::section_408_d3` is the **IRC §408(d)(3) IRA 60-day rollover module** — the timing trap that catches retail traders moving IRA money between brokerages. Companion to `form_8606` (which handles backdoor Roth basis + §408(d)(2) pro-rata) — that module is about *what's taxable on conversions*; this one is about *whether the rollover even qualified*.

Three rules in §408(d)(3) all apply concurrently:

- **60-day deposit window** per §408(d)(3)(A) — rollover must complete within 60 days of distribution. Day 60 inclusive (`0..=60`); day 61 disqualifies.
- **Once-per-12-months aggregated across ALL IRAs** per §408(d)(3)(B) + **Bobrow v. Commissioner (2014)** + IRS Ann. 2014-15. Was previously per-IRA; Bobrow changed it to aggregated. Trustee-to-trustee transfers don't count toward this limit. Roth conversions don't count toward this limit (§408(d)(3)(C)(ii)(II)).
- **§72(t) 10% early withdrawal penalty** — when rollover fails, the distribution amount is fully taxable AND a 10% additional tax applies if the taxpayer was under age 59½ at distribution.

**§408(d)(3)(I) hardship waiver** (Rev. Proc. 2020-46 self-certification) excuses the 60-day window for 12 specific hardships (financial institution error, postal error, severe damage to principal residence, family death, severe illness, incarceration, restrictions imposed by a foreign country, etc.). Does NOT excuse the Bobrow once-per-12-months violation.

**Trustee-to-trustee transfer short-circuit** — when `trustee_to_trustee_transfer: true`, the analysis bypasses §408(d)(3) entirely (always safer path; no time or count limit). The module returns this as the controlling path regardless of other facts.

Mounted at `POST /api/calc/section-408-d3`. Nineteen tests pin: within 60-day window no prior rollover qualifies; **day 60 exactly qualifies** (boundary); **day 61 misses window** (strict ≤ boundary); no rollover attempted full taxable + 10% penalty under 59½; over 59½ no early withdrawal penalty; trustee-to-trustee transfer bypasses ALL limits (even with 61-day delay + prior rollover); Bobrow violation when prior rollover 182 days ago; prior rollover 397 days ago no Bobrow violation; Roth conversion doesn't count toward Bobrow (current and prior); hardship waiver excuses 60-day violation; hardship waiver does NOT excuse Bobrow violation; taxable + penalty math at $75k distribution ($75k taxable + $7,500 penalty); day count = 44 days for base case; same-day rollover (0 days) qualifies; negative days (rollover before distribution) doesn't qualify; **Bobrow boundary at exactly 365 days** no violation (strict `<` 365); 364 days violation; note distinguishes each failure path.

`traderview-expense::form_8606` is the **IRS Form 8606 nondeductible IRA basis ledger + §408(d)(2) pro-rata rule** — the form most active traders get wrong. High-income traders above the Roth IRA contribution phase-out ($161k single / $240k MFJ for 2024 modified AGI) use the **backdoor Roth**: contribute to a traditional IRA nondeductibly, then convert to Roth. The conversion is *supposed* to be tax-free since basis equals the contribution. **Pro-rata rule blows this up** when the taxpayer has ANY pre-tax IRA balance — under §408(d)(2), every distribution AND every conversion is taxed pro-rata across the full IRA aggregate. A user with $10,000 of pre-tax SEP-IRA plus a $7,000 nondeductible contribution doing a $7,000 Roth conversion gets taxed on $4,117.68 — not zero.

The module implements Form 8606 line-by-line:
- Line 3 = prior basis + nondeductible contributions this year.
- Line 9 = year-end aggregate + distributions + conversions (the proration denominator).
- Line 10 = line 3 / line 9 (proration ratio, capped at 1.0).
- Line 11 = conversions × ratio (nontaxable conversion portion).
- Line 12 = distributions × ratio (nontaxable distribution).
- Line 13 = line 11 + line 12.
- Line 14 = line 3 − line 13 (basis carryover to next year).
- Line 15c = distributions − line 12 (taxable distribution).
- Line 18 = conversions − line 11 (taxable conversion).

Persistence lives in migration `0035_ira_basis.sql` (`ira_basis_history`, UNIQUE on `(user_id, tax_year)` so re-runs are idempotent), the `traderview-db::ira_basis` module (CRUD + `prior_year_basis(user_id, current_year)` helper that returns last year's line 14), and the routes:

- `GET    /api/tax/ira-basis` — list all ledger rows
- `POST   /api/tax/ira-basis` — run compute + persist; if `prior_basis` is omitted, the prior tax_year row is pulled from the ledger (zero if none)
- `GET    /api/tax/ira-basis/:year` — fetch a specific year
- `DELETE /api/tax/ira-basis/:year` — drop a year's row

Thirteen tests pin: clean backdoor (no pre-tax balance) → zero tax; pro-rata blows up backdoor with $10k pre-tax balance → exact $4,117.68 taxable; prior basis carries into current year; distribution-only no-conversion; mixed distribution + conversion both pro-rated; nondeductible-contribution-only no event (basis accumulates); ratio capped at 1.0 when basis exceeds denominator; empty year basis rolls forward; full conversion year-end-zero clean path; pro-rata 50/50 balance gives 50% taxable; multi-year chain preserves basis (3-year sequence); pro-rata taxable never negative under stress; note distinguishes clean vs pro-rated backdoor.

`traderview-expense::section_1212` is the **IRC §1212(b) capital loss carryover ledger** — the multi-year persistence layer the existing `schedule_d` module was missing. Active traders routinely lose more than $3,000 in a year; the excess carries forward indefinitely under §1212(b)(1) and *retains its character* — short-term stays short-term, long-term stays long-term. Per §1212(b)(2), when computing next year's carryover, the amount allowed against ordinary income is **treated as absorbing short-term loss first**, then long-term. The IRS Capital Loss Carryover Worksheet (Pub 550) is implemented step-for-step: combine prior ST/LT carryovers into current losses → net within character → cross-absorb ST loss vs LT gain (and vice versa) → if combined net is a loss, deductible = min(|loss|, $3,000 single / $1,500 MFS) → ST absorbed first against ordinary → ST carryover = net ST loss − ST absorbed, LT carryover = net LT loss − LT absorbed.

Persistence lives in migration `0034_capital_loss_carryover.sql` (table `capital_loss_carryovers`, UNIQUE on `(user_id, tax_year)` so re-running the compute is idempotent), the `traderview-db::carryover` module (CRUD + `prior_year_carryovers(user_id, current_year)` helper that returns last year's ST/LT carryover), and the routes:

- `GET    /api/tax/carryover` — list all ledger rows
- `POST   /api/tax/carryover` — run compute + persist; if `prior_st_carryover` / `prior_lt_carryover` are omitted, the prior year's row is pulled from the ledger (zero if none)
- `GET    /api/tax/carryover/:year` — fetch a specific year
- `DELETE /api/tax/carryover/:year` — drop a year's row

Thirteen tests pin: pure-ST-loss-deducts-3k-carries-rest-as-ST; pure-LT-loss-deducts-3k-carries-rest-as-LT; ST-absorbed-first-when-both-ST-and-LT-losses (§1212(b)(2)); ST-carryover-below-3k-lets-LT-absorb-remainder; MFS-caps-deduction-at-1500; prior-ST-carryover-absorbs-current-ST-gain-first; ST-loss-cross-absorbs-LT-gain-before-deduction; LT-loss-cross-absorbs-ST-gain-before-deduction; loss-exactly-3k-no-carryover; net-gain-clears-carryovers-no-deduction; exact-wash-returns-zero; multi-year-chain-ST-character-preserved (4 sequential years exhausting a $10k loss); carryover-stays-non-negative under stress.

`traderview-expense::reps_qualification` is the **§469(c)(7) Real Estate Professional Status checker** — fills the input gap in iter 5's `section_469`, which accepted a `reps_qualified: bool` from the caller but never computed it. REPS is the gate that flips rental losses from per-se passive to NON-PASSIVE, eliminating the §25k allowance cap entirely. The bar is high — most landlords don't qualify.

Three-prong test:

1. **750-hour test** — strictly more than 750 hours of services performed during the year in real-property trades or businesses where the taxpayer materially participates.
2. **>50% of personal services test** — more than half of the taxpayer's total personal services (across ALL work including W-2 employment) performed that year are in real-property trades or businesses. This is what kills most "active landlord with day job" claims: 2,000 W-2 software-job hours vs 800 landlord hours = 28.6% RPTB share = fail.
3. **Material participation** — per-activity (or aggregated if §469(c)(7)(A) grouping election is filed) under one of seven §1.469-5T tests. Caller asserts which test was satisfied; we accept any of the seven (`OverFiveHundredHours`, `SubstantiallyAll`, `OverHundredHoursAndMost`, `SpaTotalOverFiveHundred`, `PriorFiveOfTen`, `PersonalServicePriorThree`, `FactsAndCircumstances`).

§469(c)(7)(B) lists eleven qualifying RPTB activities: development, redevelopment, construction, reconstruction, acquisition, conversion, rental, operation, management, leasing, brokerage. Hours in NON-RPTB activities (W-2 software dev, retail clerk, etc.) are excluded from the numerator of both tests but counted in the denominator of the >50% test.

**MFJ rule** (§469(c)(7)(B) flush language): REPS is **per-spouse**. One spouse alone must meet the 750-hour AND >50% tests. Spouses CANNOT aggregate hours to qualify jointly. Once one spouse qualifies, both spouses' rental activities are tested for material participation, and material participation IS aggregated per §469(h)(5). The failure note explicitly calls this out for MFJ filers.

Mounted at `POST /api/calc/reps-qualification`. Fifteen tests pin: full-time landlord qualifies; W-2 software job (2000 other hours vs 800 RPTB) kills >50% test even with 750-hour test passing; boundary at exactly 750 hours fails (strict `>`, not `≥`); 751 passes; exactly 50% share fails; 50.05% passes; missing material participation kills qualification; hours sum across all eleven activity categories; MFJ failure note calls out per-spouse rule; grouping-election note when qualified; negative hours clamp to zero; zero-hours edge no divide-by-zero; all three failure reasons listed when all three miss; all seven §1.469-5T material participation tests accepted; 750 with zero other hours still fails strict `>` despite 100% RPTB share.

`traderview-expense::section_469` is the **IRC §469 passive-activity-loss limitation calculator** — the tax rule that most-often kills new landlords' refunds. Rental real estate is *per se* passive under §469(c)(2); losses can only offset passive income, with overflow suspended to next year. Three carve-outs let losses through, and we model all three:

- **§469(i) $25,000 active-participation allowance** — full $25k for MAGI ≤ $100k ($50k MFS), phases out 50¢ on the dollar between $100k–$150k MAGI, zero at $150k+. MFS halves limits and band ($12,500 max, $50k–$75k band).
- **§469(c)(7) Real-Estate-Professional Status** — > 750 hours material participation + > 50% of personal services in real-property trades. Once REPS, rental losses are non-passive and unlimited.
- **Short-term rental loophole** (Reg. §1.469-1T(e)(3)(ii)(A)) — when average customer stay ≤ 7 days and the taxpayer materially participates, the activity is NOT a rental for §469 purposes. Vacation rentals + material participation = unlimited loss deductibility.
- **§469(g) full disposition** — fully disposing of the activity releases all suspended losses immediately.

Computation order: full disposition → REPS → STR + MP → offset against other passive income → §469(i) allowance subject to phase-out (only if `active_participation = true`) → suspend remainder. Mounted at `POST /api/rental/section-469`. Fourteen tests pin the IRS rules: under-$100k full allowance, phase-out exact at MAGI $125k = $12,500 cap, $150k zeros, MFS half-band, REPS no-limit, STR no-limit, passive-income offset ordering, no-active-participation kills allowance, full-disposition releases carryover, no-loss no-op, REPS-priority-over-offset (no double-counting).

### [0x0AC] Institutional 13F holdings (smart-money tracker)

Migration `0033_institutional_13f.sql` adds the surface QuiverQuant / WhaleWisdom / 13F.info charge $30+/month for: `institutional_managers` (CIK + name + manager_type ∈ `hedge_fund` / `rita` / `pension` / `sovereign` / `insurance` / `bank` / `other` + aliases + `notable` flag), `institutional_13f_filings` (one row per accession, `quarter_end`, `filed_at`, total AUM, holdings count, dedupe on `(manager_id, accession_number)`), and `institutional_holdings` (per-position: CUSIP + symbol + issuer + shares + value + sole/shared/none voting power + `put_call` for option positions, unique on `(filing_id, cusip, COALESCE(put_call, ''))`).

Two SQL views ride on top so callers don't reinvent window functions:
- `institutional_latest_filings` — `DISTINCT ON (manager_id)` over filings, returns the most recent filing per CIK in one row.
- `institutional_position_changes` — `ROW_NUMBER() OVER (PARTITION BY manager_id, cusip ORDER BY quarter_end DESC)` joined on `rn = cur.rn + 1` to produce `change_type ∈ {new, increased, decreased, held}` plus `delta_shares` and `delta_value`. Used by `GET /api/institutional/managers/:id/changes`.

The `traderview-db::institutional` module exposes 8 read queries (`list_managers` with ILIKE search + notable filter, `manager_by_cik`, `manager_filings`, `holdings_for_manager_latest`, `holdings_for_filing`, `position_changes_for_manager` filterable by change_type, `top_owners_of_symbol`, `top_managers_by_aum`). Routes are mounted at `/api/institutional/*`:

- `GET /institutional/managers?search=&notable=&limit=`
- `GET /institutional/managers/by-cik/:cik`
- `GET /institutional/managers/:id/filings?limit=`
- `GET /institutional/managers/:id/holdings?limit=` — most recent quarter, ordered by value
- `GET /institutional/managers/:id/changes?change_type=new|increased|decreased|held&limit=`
- `GET /institutional/filings/:id/holdings?limit=`
- `GET /institutional/symbols/:symbol/owners?limit=` — who owns SYM
- `GET /institutional/top-managers?limit=` — by AUM

The EDGAR 13F-HR XML poller that populates these tables is **deferred to a follow-up iteration** — the read surface is built first so the UI can wire to stable types and the data side can land independently. EDGAR endpoints to plug in: `https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&type=13F-HR&dateb=&owner=include&count=40` for new-filing detection, accession-specific `/Archives/edgar/data/{cik}/{accession_no_dashes}/{accession_no}-index.html` for the `informationTable.xml` payload, and the SEC company-tickers JSON (`https://www.sec.gov/files/company_tickers.json`) for the CIK→ticker mapping table that fills `institutional_holdings.symbol`.

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
