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

`traderview-expense::deposit_interest` is the **state-specific security-deposit-interest table** for the 13 jurisdictions (CT, DC, FL, IA, IL, MA, MD, MN, NH, NJ, NY, PA, RI) that have a security-deposit-interest statute. Each row carries the statutory citation + source URL, the published annual rate (where statute fixes one — MN 1%, MD 1.5% min, CT 1.45% as of 2024), the minimum holding period before interest accrues (PA 24mo, MA 12mo, NH 12mo, etc.), and a note covering carve-outs (IL requires interest only for buildings of 25+ units; NY/NJ/PA use the *actual* bank rate via the caller-supplied override). The 37 states without a requirement return `required: false` with empty citation. Nine tests pin: TX no-requirement, MN 1% full-year, MD 1.5% full-year, NY uses caller-supplied bank rate, PA's 24-month gate, case-insensitive state lookup, negative-window safety, citation correctness for CT/MD, and unknown-state-returns-None.

`traderview-expense::section_1045` is the **IRC §1045 QSBS rollover module** — direct companion to `section_1202`. §1202 caps the exclusion at 5 years of holding; §1045 plugs the gap for taxpayers who sell BEFORE the 5-year clock matures: a holder of QSBS held **more than 6 months** can **defer gain** by reinvesting proceeds into OTHER QSBS within **60 days** of the sale. The original's holding period **tacks onto** the replacement for the §1202 5-year clock — chaining sales through multiple rollovers eventually qualifies for full §1202 exclusion at $0 in deferred basis.

Mechanics: gain deferred = MIN(realized gain, replacement cost); boot received = sale proceeds net − replacement cost (when positive); replacement basis = replacement cost − gain deferred (carryover basis preserving the deferred gain); effective holding-period-start inherits the original's acquisition date. Disqualification routes the full gain to current-year recognition with a reason list (six tested paths: original not §1202-qualified, replacement not §1202-qualified, held ≤ 6 months, replacement after 60-day window, replacement before sale date, multiple-failure stack).

Mounted at `POST /api/calc/section-1045`. Seventeen tests pin: full-replacement no-boot full deferral; partial replacement triggers boot recognition; boot exceeds gain caps recognition at gain; held under 6 months disqualified; replacement after 60-day window disqualified; replacement before sale disqualified; original/replacement not QSBS-qualified disqualified; **boundary tests** — exactly 6 months (183 days) disqualified, just over 6 months (184 days) qualifies, exactly 60 days qualifies, 61 days disqualified; loss returns no-op; holding period tacks to original acquisition; replacement basis never negative under stress; replacement_value − replacement_basis == deferred_gain invariant; multi-disqualification lists all reasons.

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

`traderview-expense::section_280a` is the **IRC §280A vacation home / mixed-use classifier** — uses the `fair_rental_days` and `personal_use_days` fields on `rental_properties` to bucket each property into one of four classifications:

- **Rental** — `fair_rental_days ≥ 15` AND `personal_use_days ≤ MAX(14, 10% of fair_rental_days)`. Full Schedule E; §469 PAL applies separately.
- **VacationHome** — `fair_rental_days ≥ 15` AND personal use over the threshold. §280A(c)(5) caps deductions at gross rental income (no net loss); expenses allocated pro-rata between personal and rental days and tiered. **Tier 1** (mortgage interest, property tax — already deductible on Schedule A) always allowed at the rental allocation %; **Tier 2** (operating expenses: insurance, utilities, repairs, management, supplies, advertising) allowed up to remaining income after tier 1; **Tier 3** (depreciation) allowed up to remaining income after tier 1 + 2. Excess tier 2 + 3 carries forward to next year via the `prior_year_suspended` input.
- **AugustaRule** — `fair_rental_days` is 1–14. §280A(g) **tax-free rental income**: the gross income is excluded from gross income entirely (not reported), and no rental deductions are allowed. Famously used by homeowners renting to their own corporations for board meetings (corp deducts the rent, owner excludes it).
- **PersonalResidence** — `fair_rental_days = 0`. No rental activity reported.

The personal-use threshold uses the GREATER of 14 days OR 10% of fair rental days per IRS Pub 527 — so a property rented 200 days passes the rental classification if personal use ≤ 20 days, not the bare 14. Mounted at `POST /api/rental/properties/:id/section-280a` with auto-fill: missing `fair_rental_days` / `personal_use_days` are pulled from the `rental_properties` row. Sixteen tests pin: pure-rental no-personal-use; rental within threshold allocates proportionally; threshold uses max(14, 10%); rental boundary at 14 days exact stays rental; 15 personal days flips to vacation home; vacation home deductions capped at income (no loss); low-income suspends excess; Augusta Rule 14 days tax-free; Augusta boundary at 14 vs 15; personal residence zero rental days; prior suspended stacks with tier 2; 1-day rental routes to Augusta safely; allocation pct zero when both days zero; personal_use_ceiling math (10% of 100/140/200/365 days).

`traderview-expense::disposition` is the **rental property disposition module** — the sale-time computation every landlord faces but generic tax software handles poorly. Realized gain decomposes into TWO buckets the IRS taxes at different rates: **§1250 unrecaptured gain** (the portion attributable to prior depreciation, capped at 25% federal) and **§1231 LTCG** (the remainder, at 0/15/20% LTCG rates). The split is `§1250 = min(accumulated_depreciation, realized_gain)`; depreciation can't recapture more gain than actually exists. Selling at a loss triggers §1231 ordinary-loss treatment with no §1250 component.

When the seller rolls into a replacement via **§1031 like-kind exchange**, gain is DEFERRED to the extent of replacement value. Boot — cash received or net debt relief — triggers recognition `MIN(realized_gain, boot_received + debt_relief_net)`. Replacement basis = `adjusted_basis + boot_paid − boot_received + gain_recognized`, carrying the deferred gain into the new property. Per §1031(c), losses are recognized in full — §1031 does not defer losses.

Wired at `POST /api/rental/properties/:id/dispose`. Caller supplies `sale_price + selling_costs + (optional) original_cost_basis + accumulated_depreciation + capital_improvements_added + like_kind_exchange`. Missing `original_cost_basis` is filled from `rental_properties.purchase_price`; missing `accumulated_depreciation` is summed from `rental_expenses` rows where `category_code = 'e_depreciation'`. Thirteen tests pin: straight-sale matches Form 4797, capital improvements raise basis lowering gain, §1250 capped at total gain (can't recapture phantom amounts), loss triggers §1231 ordinary, §1031 no-boot full deferral, §1031 boot recognized up to realized gain, §1031 boot exceeds gain caps recognition, §1031 debt-relief net counts as boot, §1031 replacement-basis carries deferred gain (`replacement_value - replacement_basis == deferred_gain`), §1031(c) losses recognized in full, max-§1250-tax estimate is 25% of unrecaptured, zero-gain edge case, no-depreciation → all §1231.

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
