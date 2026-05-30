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
- **256-tile launcher (Cmd-K)** — categorized tile grid with live filter replaces the old 77-tab strip. Press `?` anywhere for the in-app tutorial. Topbar carries the most-used routes.
- **Right-click context menus everywhere** — every view registers a `data-context-scope` slug that the global ctxmenu handler resolves on right-click. Per-row scopes (`trade-row`, `symbol-row`, `journal-entry`, `tag-chip`, `webhook-row`, `api-token-row`, `account-row`, `plan-row`, `share-row`, `board-row`, `dashboard-sidebar-item`, `hotkey-row`, `custom-indicator-row`, `alert-rule-row`, `strategy-alert-row`, `watchlist-symbol-row`, `position-row`, `backtest-preset-row` — 18 scopes) plus 37 symbol-aware view scopes give 222 distinct symbol-nav paths to the active symbol (Charts / Options / Research / Earnings / News / Copy) without leaving the current view. Items are registered in a single `ALL_SCOPED_ITEMS` array; handler bodies are 3-4 LoC each via 6 shared helpers (`clipboardWrite` / `refreshView` / `dataFromTarget` / `toastErr` / `toastOk` / `symbolFromTarget`). Two regression tests pin (a) every emitting tag carries the data-* attrs its handler reads and (b) every registered scope is documented in the audit's required-attrs map.
- **48-shortcut keyboard surface** — `_shortcuts.js` registry: 8 nav globals (`Cmd+Opt+T/J/D/W/C/L/R/M` for Trades/Journal/Dashboard/Watchlists/Charts/Live/Reports/Scanner) + 11 view-scoped binds (`n` for new-trade in trades scope, `r` to refresh in dashboard/live/trades/journal/watchlists/webull/charts, etc.) + the pre-existing global set. Every view-scoped bind has a visible `⟳ Refresh` / `+ New` button stamped with `data-shortcut`, so the tooltip augmenter appends `(⌘⌥T)` / `(R)` chips on hover. The cheat-sheet view (`?` or Cmd+K → "Keyboard Shortcuts") lists all 48 globals + the 70+ scoped ctxmenu items in three searchable tables.
- **Non-blocking dialog primitive** — `tConfirm()` / `tPrompt()` (in `dialog.js`) replace every `alert()` / `confirm()` / `prompt()` call site (63 + 37 = 100 swept across 42 view files). Returns a Promise; themed by level (`info` / `warning` / `danger`), Enter/Esc keybound, required-empty input shake, i18n labels via `dialog.btn.*` keys.
- **Vanilla JS + uPlot frontend** — zero npm, zero bundler, zero framework. 257 view modules + 184 pure helper modules + ~21 runtime modules across 93,692 LOC JS + 2,512 LOC CSS. Per-view race-token machinery prevents post-await DOM crashes; window.onerror + console.error funnel to a Rust-side `/api/client-errors` sink.
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
| Community                  | `shares`, `shared_comments`, `forum_categories`, `forum_threads`, `forum_posts`, `boards`, `board_items` |
| Settings & AI              | `user_settings`, `ai_settings`, `ai_journal_cache`, `dashboards`       |
| Expenses + OCR             | `expense_accounts`, `expense_categories`, `expense_transactions`, `expense_rules`, `expense_receipts` |

Sides are typed enums: `side_t = (buy, sell, short, cover)` for executions; `trade_side_t = (long, short)` and `trade_status_t = (open, closed)` for trades. Other enums cover order status, review status, asset class, alert triggers, sentiment sources, halt reason codes, etc.

---

## [0x05] HTTP API

**413 axum routes** under `/api/` across ~83 route files. Bearer-auth required on everything except `/health`, `/config`, `/auth/*`, and `/client-errors`. Four WebSocket endpoints expose live feeds. Frontend bindings live in `frontend/js/api.js` (441 method-bound helpers). Grouped:

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

`frontend/` is **vanilla JS + uPlot**. Zero npm, zero bundler, zero framework. **257 view modules + 184 pure helper modules + ~21 runtime modules**, 93,692 LOC JS + 2,512 LOC CSS. All views render into `<main id="app">` via hash-routed dispatch. **256-tile launcher (Cmd-K)** is the primary entry point; topbar carries 11 shortcuts and the rest is the launcher. `?` opens the in-app tutorial.

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
