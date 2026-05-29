// In-app tutorial. Linked from the launcher tile + the `?` hotkey.
// Content is grouped to match the launcher's category structure so the user
// can keep this open in one tab and the launcher (Cmd-K) in another.

import { go } from '../app.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const SECTIONS = [
    {
        id: 'quickstart',
        title: '// QUICKSTART',
        body: `
            <ol class="tut-steps">
                <li data-i18n-html="view.tutorial.quickstart.step1">Open <code>Accounts</code> (or tile <strong>🏦 Accounts</strong>) and add at least one broker — the dashboard P/L stays empty until something is bound.</li>
                <li data-i18n-html="view.tutorial.quickstart.step2">Open <code>Import</code> (or tile <strong>⤴ Import</strong>) and upload a broker CSV (12 importers supported). Trades roll up automatically.</li>
                <li data-i18n-html="view.tutorial.quickstart.step3">Hit <kbd>Cmd</kbd>+<kbd>K</kbd> to open the launcher. Type to filter all 74 tiles. Press Enter on the first match to navigate.</li>
                <li data-i18n-html="view.tutorial.quickstart.step4">Hit <kbd>?</kbd> anywhere outside a text field to re-open this tutorial.</li>
            </ol>`,
    },
    {
        id: 'keymap',
        title: '// KEYBINDINGS',
        body: `
            <table class="tut-kbd">
                <thead><tr><th data-i18n="view.tutorial.th.combo">Combo</th><th data-i18n="view.tutorial.th.action">Action</th></tr></thead>
                <tbody>
                    <tr><td><kbd>Cmd</kbd>+<kbd>K</kbd></td><td data-i18n="view.tutorial.keymap.cmd_k">Open the launcher (tile grid + filter)</td></tr>
                    <tr><td><kbd>?</kbd></td><td data-i18n="view.tutorial.keymap.question_mark">Open this tutorial</td></tr>
                    <tr><td data-i18n-html="view.tutorial.keymap.esc_combo"><kbd>Esc</kbd> (in launcher filter)</td><td data-i18n="view.tutorial.keymap.esc_action">Clear the filter</td></tr>
                    <tr><td data-i18n-html="view.tutorial.keymap.enter_combo"><kbd>Enter</kbd> (in launcher filter)</td><td data-i18n="view.tutorial.keymap.enter_action">Open the first matching tile</td></tr>
                </tbody>
            </table>
            <p class="muted small">More custom hotkeys can be bound on the <button data-i18n="view.tutorial.btn.hotkeys" class="link" data-go="hotkeys">Hotkeys</button> page. Bindings persist to the local database.</p>`,
    },
    {
        id: 'live',
        title: '// LIVE MARKETS',
        body: `
            <p data-i18n="view.tutorial.hint.always_on_streaming_tiles_each_opens_its_own_webso">Always-on streaming tiles. Each opens its own WebSocket; voice alerts use the browser SpeechSynthesis API.</p>
            <dl class="tut-defs">
                <dt><button data-i18n="view.tutorial.btn.live_scanner" class="link" data-go="live-scanner">⚡ Live Scanner</button></dt>
                <dd>6-panel real-time scanner (Gappers, Gainers, Losers, HOD, Volume, Ross 5-Pillar). Needs a free <a href="https://finnhub.io" target="_blank">Finnhub</a> API key + a comma-sep symbol universe (25 syms / WS connection, chunks automatically).</dd>
                <dt><button data-i18n="view.tutorial.btn.halts" class="link" data-go="halts">⏸ Halts</button></dt>
                <dd>Polls Nasdaq trade-halt RSS every 3s. Voice alerts spell tickers letter-by-letter so TTS pronounces SPCE, not "space".</dd>
                <dt><button data-i18n="view.tutorial.btn.catalysts" class="link" data-go="catalysts">📰 Catalysts</button></dt>
                <dd>SEC EDGAR (6s) + Business Wire / PR Newswire / GlobeNewswire / AccessWire (30s). Ticker NER extracts <code>$SYM</code>, <code>(SYM)</code>, and <code>NYSE:</code>/<code>NASDAQ:</code> prefixes. Voice fires when a watchlist symbol appears.</dd>
                <dt><button data-i18n="view.tutorial.btn.pre_market" class="link" data-go="premarket">🌅 Pre-market</button></dt>
                <dd>Cross-asset opening drive — index futures, commodities, crypto, FX, plus today's high-importance economic events. ATR-multiple shows how outsized the overnight move is.</dd>
            </dl>`,
    },
    {
        id: 'trading',
        title: '// TRADING',
        body: `
            <dl class="tut-defs">
                <dt><button data-i18n="view.tutorial.btn.webull" class="link" data-go="webull">🪙 Webull</button></dt>
                <dd><strong>Read-only.</strong> Order entry is intentionally not implemented. Open webull.com → DevTools → Network → copy <code>did</code>, <code>access_token</code>, <code>t_token</code> from any <code>tradeapi.webullbroker.com</code> request. Paste into the form. Tokens live in process memory only — never written to disk. Positions/orders/account poll every 5s.</dd>
                <dt><button data-i18n="view.tutorial.btn.live_positions" class="link" data-go="live">💰 Live Positions</button></dt>
                <dd>Open positions from imported trades, re-priced with fresh quotes. Shows unrealized P/L, day P/L, and total cost basis.</dd>
                <dt><button data-i18n="view.tutorial.btn.paper_trade" class="link" data-go="paper">📝 Paper Trade</button></dt>
                <dd>Simulated execution against the same quote pipeline as live. Order types: market, limit, stop, stop-limit. Reset starting cash anytime.</dd>
                <dt><button data-i18n="view.tutorial.btn.pre_trade_plans" class="link" data-go="plans">🎯 Pre-trade Plans</button></dt>
                <dd>Write the setup before you take the trade. Link a plan to an execution post-fill — the plan vs. outcome lands in your discipline scorecard.</dd>
                <dt><button data-i18n="view.tutorial.btn.risk_gate" class="link" data-go="risk-gate">🛡 Risk Gate</button></dt>
                <dd>Pre-trade rules that BLOCK bad trades before they reach the broker. Define max loss per trade / per day, max consecutive losses, cool-down after a loss, max open positions, blocked symbols, require-plan, require-stop. Every new trade form runs these rules first and refuses to submit if any Block-severity rule fires (warnings get a confirm-then-proceed prompt).</dd>
                <dt><button data-i18n="view.tutorial.btn.position_size" class="link" data-go="sizing">🧮 Position Size</button></dt>
                <dd>Three modes: Kelly fraction (from your historical win-rate), fixed-fractional (% of equity per trade), R-based (max loss in dollars / stop distance). Correlation-aware: caps concentration when symbols co-move.</dd>
            </dl>`,
    },
    {
        id: 'journal',
        title: '// JOURNAL',
        body: `
            <dl class="tut-defs">
                <dt><button data-i18n="view.tutorial.btn.journal" class="link" data-go="journal">📓 Journal</button></dt>
                <dd>Per-trade, daily, and general notes. Each scope has its own templates editable under <button data-i18n="view.tutorial.btn.settings" class="link" data-go="settings">Settings</button> → Notes Templates.</dd>
                <dt><button data-i18n="view.tutorial.btn.ai_journal" class="link" data-go="ai">🧠 AI Journal</button></dt>
                <dd>GPT-assisted post-mortem on individual trades. Requires an OpenAI key in <button data-i18n="view.tutorial.btn.ai_settings" class="link" data-go="ai">AI settings</button>. Output is cached so re-opening a trade is instant.</dd>
                <dt><button data-i18n="view.tutorial.btn.trade_reviews" class="link" data-go="reviews">🔁 Trade Reviews</button></dt>
                <dd>Forced reflection on |R| ≥ 2 trades. Tracks plan-quality, execution-quality, emotion. Surfaces "reviews needed" count on the dashboard.</dd>
                <dt><button data-i18n="view.tutorial.btn.discipline" class="link" data-go="discipline">🛡 Discipline</button></dt>
                <dd>Rule-violation tracker. Defines hard limits (max R per trade, max losses per day, no FOMO entries) and counts breaches.</dd>
                <dt><button data-i18n="view.tutorial.btn.mood_analytics" class="link" data-go="mood">🌡 Mood Analytics</button></dt>
                <dd>Self-rated mood entered with each daily journal correlates against same-day P/L. Surfaces "you trade worse when…" patterns.</dd>
            </dl>`,
    },
    {
        id: 'research',
        title: '// CHARTS & RESEARCH',
        body: `
            <dl class="tut-defs">
                <dt><button data-i18n="view.tutorial.btn.charts" class="link" data-go="charts">📈 Charts</button></dt>
                <dd>OHLC + indicator overlays. Draw lines/arrows/rectangles; drawings persist per symbol per user.</dd>
                <dt><button data-i18n="view.tutorial.btn.research" class="link" data-go="research">🔎 Research</button></dt>
                <dd>Per-symbol dossier: quote, fundamentals, news, earnings history, recommendations, insider trades, dividends, holders.</dd>
                <dt><button class="link" data-go="screener">🧪 Screener / <button data-i18n="view.tutorial.btn.scanners" class="link" data-go="scanners">🛰 Scanners</button> / <button data-i18n="view.tutorial.btn.top_signals" class="link" data-go="top-signals">📡 Top Signals</button></button></dt>
                <dd>Three layers of discovery. Screener = manual criteria. Scanners = 24 Warrior/Zendoo presets (gap-and-go, opening drive, low-float surge…). Top Signals = ranked live signal leaderboard.</dd>
                <dt><button data-i18n="view.tutorial.btn.dark_pool" class="link" data-go="darkpool">🕳 Dark Pool</button> / <button data-i18n="view.tutorial.btn.short_interest" class="link" data-go="short-interest">🩳 Short Interest</button></dt>
                <dd>Off-exchange % print rankings and FINRA Reg-SHO daily short volume. Use to flag squeezable setups.</dd>
                <dt><button data-i18n="view.tutorial.btn.options" class="link" data-go="options">⛓ Options</button> / <button data-i18n="view.tutorial.btn.vol_surface" class="link" data-go="vol-surface">🌋 Vol Surface</button> / <button data-i18n="view.tutorial.btn.earnings_iv" class="link" data-go="earnings-iv">💥 Earnings IV</button></dt>
                <dd>Full options chain with Greeks. Vol surface plots the IV grid across strikes × expiries. Earnings IV scans pre-event IV crush and computes straddle EV.</dd>
            </dl>`,
    },
    {
        id: 'reports',
        title: '// REPORTS',
        body: `
            <p><button data-i18n="view.tutorial.btn.reports" class="link" data-go="reports">📊 Reports</button> has 17 TraderVue-style cuts: by symbol, side, asset class, day-of-week, hour, hold duration, month, R-distribution, streaks, comparison, exit efficiency, commissions, liquidity, risk, drawdown, risk-adjusted, calendar.</p>
            <p><button data-i18n="view.tutorial.btn.equity_forecast" class="link" data-go="forecast">🔮 Equity Forecast</button> runs a Monte Carlo against your historical R-distribution to project the next N trades.</p>
            <p><button data-i18n="view.tutorial.btn.fill_quality" class="link" data-go="fill-quality">🎯 Fill Quality</button> approximates slippage vs. the bar's NBBO — useful to compare brokers.</p>
            <p><button data-i18n="view.tutorial.btn.tax_lots" class="link" data-go="tax-lots">💸 Tax Lots</button> reproduces lot-by-lot cost basis (FIFO/LIFO) with wash-sale detection. Exports a Schedule D friendly CSV from <button data-i18n="view.tutorial.btn.exports" class="link" data-go="exports">Exports</button>.</p>`,
    },
    {
        id: 'strategy',
        title: '// STRATEGY & AUTOMATION',
        body: `
            <dl class="tut-defs">
                <dt><button data-i18n="view.tutorial.btn.backtest" class="link" data-go="backtest">🧷 Backtest</button> / <button data-i18n="view.tutorial.btn.presets" class="link" data-go="backtest-presets">📦 Presets</button></dt>
                <dd>stryke-JIT backtester. Define entry/exit on any indicator combination. Save presets, fork community presets, share via slug.</dd>
                <dt><button data-i18n="view.tutorial.btn.walk_forward" class="link" data-go="walk-forward">🧱 Walk-forward</button></dt>
                <dd>Rolling in-sample / out-of-sample sweep. Reveals overfit parameters that look great in-sample but fall apart OOS.</dd>
                <dt><button data-i18n="view.tutorial.btn.indicators" class="link" data-go="custom-indicators">∇ Indicators</button></dt>
                <dd>Custom indicator editor. Define your own MA/RSI/ATR variants. Evaluable against any symbol from the Research view.</dd>
                <dt><button data-i18n="view.tutorial.btn.alerts" class="link" data-go="alerts">🚨 Alerts</button> / <button data-i18n="view.tutorial.btn.strategy_alerts" class="link" data-go="strategy-alerts">🔔 Strategy Alerts</button></dt>
                <dd>Price/threshold alerts vs. compound AND/OR/NOT strategy rules. Both fire voice + Notification + optional webhook.</dd>
                <dt><button data-i18n="view.tutorial.btn.webhooks" class="link" data-go="webhooks">🪝 Webhooks</button></dt>
                <dd>Outbound to Discord / Slack / generic URL. Use as a kill-switch trigger, mobile push, journal mirror, etc.</dd>
            </dl>`,
    },
    {
        id: 'data',
        title: '// DATA SOURCES',
        body: `
            <p data-i18n="view.tutorial.hint.most_feeds_are_free_public_endpoints_no_auth_requi">Most feeds are free public endpoints — no auth required:</p>
            <ul class="tut-list">
                <li><strong>Quotes / fundamentals / charts:</strong> Yahoo Finance v8/v10. Cached 60s per symbol in Postgres.</li>
                <li><strong>Live ticks:</strong> Finnhub WebSocket. Free tier = 25 syms / connection; chunked automatically.</li>
                <li><strong>Halts:</strong> nasdaqtrader.com RSS, 3s poll.</li>
                <li><strong>Catalysts:</strong> SEC EDGAR Atom (6s), BusinessWire/PRNewswire/GlobeNewswire/AccessWire RSS (30s).</li>
                <li><strong>Short interest:</strong> FINRA Reg-SHO daily files.</li>
                <li><strong>Sentiment:</strong> Reddit r/wallstreetbets JSON, StockTwits public stream.</li>
                <li><strong>Crypto:</strong> CoinGecko + blockchain.com.</li>
                <li><strong>Webull:</strong> session tokens you paste from your own browser. Read-only.</li>
            </ul>
            <p class="muted small" data-i18n-html="view.tutorial.data_sources.paths">All data lands in embedded Postgres at <code>~/Library/Application Support/com.menketechnologies.traderview/traderview/pg-data</code>. Logs at <code>~/Library/Application Support/traderview/traderview.log</code>.</p>`,
    },
    {
        id: 'risk-gate',
        title: '// RISK GATE — PRE-TRADE RULES',
        body: `
            <p>Discipline that <em>enforces</em> instead of just reporting. The gate sits between the new-trade form and the broker — every submission runs the rules first.</p>
            <ol class="tut-steps">
                <li><strong>Install a preset</strong> from the <button data-i18n="view.tutorial.btn.risk_gate_2" class="link" data-go="risk-gate">Risk Gate</button> view. Three packs:
                    <ul class="tut-list">
                        <li><strong>Beginner</strong> — 7 rules. 1% max per trade, 3% per day, 3 consec losses stop you, 15-min cool-down, 25% max position, plan + stop required. Strictest.</li>
                        <li><strong>Intermediate</strong> — 5 rules. Same per-trade cap, 5% daily, 4 streak, 5-min cool-down, stop required (plan optional).</li>
                        <li><strong>Aggressive</strong> — 2 rules. Daily-loss cap + cool-down only. Assumes you manage per-trade risk yourself.</li>
                    </ul>
                </li>
                <li><strong>Tune</strong> — toggle individual rules off via the checkbox column. Delete with the link. Add custom rules with the form below the list.</li>
                <li><strong>Dry-run</strong> a hypothetical trade in the bottom panel to see exactly what the gate would say. Same call the new-trade form makes on submit.</li>
                <li><strong>Compliance snapshot</strong> at top — synthetic probe that surfaces which rules would fire on ANY entry right now (max daily loss already hit, in cool-down after the last loss, etc).</li>
                <li><strong>Paper trades also gate.</strong> Paper trading is where you build the habit; if rules only fired live, you'd practice rule-breaking.</li>
                <li><strong>Webhook on Block.</strong> Every Block-severity veto fires to every enabled webhook (Discord / Slack / generic) so you get a public record of every rule save.</li>
            </ol>
            <p class="muted small" data-i18n-html="view.tutorial.risk_gate.engine">Engine lives in <code>traderview_core::risk_gate</code> — pure compute, 21 unit tests pin the rule semantics + serde compat + preset rule sets.</p>`,
    },
    {
        id: 'workflow',
        title: '// TYPICAL DAY-TRADE WORKFLOW',
        body: `
            <ol class="tut-steps">
                <li><strong>Pre-market (06:00–09:25 ET).</strong> Open <button data-i18n="view.tutorial.btn.pre_market_2" class="link" data-go="premarket">Pre-market</button> to scan overnight movers. Open <button data-i18n="view.tutorial.btn.catalysts_2" class="link" data-go="catalysts">Catalysts</button> for fresh 8-K / press releases on watchlist symbols. Check <button data-i18n="view.tutorial.btn.economy" class="link" data-go="economy">Economy</button> for today's data drops.</li>
                <li><strong>Open (09:30–09:45 ET).</strong> Open <button data-i18n="view.tutorial.btn.live_scanner_2" class="link" data-go="live-scanner">Live Scanner</button> with your gap watchlist. Open <button data-i18n="view.tutorial.btn.halts_2" class="link" data-go="halts">Halts</button> in a side window — T1 / T12 halts on small-caps are squeeze/dump candidates.</li>
                <li><strong>Intraday.</strong> Plan trades in <button data-i18n="view.tutorial.btn.plans" class="link" data-go="plans">Plans</button> before entry. Use <button data-i18n="view.tutorial.btn.position_size_2" class="link" data-go="sizing">Position Size</button> to compute share count from your max-loss-R. Watch the entry on <button data-i18n="view.tutorial.btn.charts_2" class="link" data-go="charts">Charts</button>. Track open P/L on <button data-i18n="view.tutorial.btn.live_positions_2" class="link" data-go="live">Live Positions</button>.</li>
                <li><strong>Post-market (16:00+).</strong> Run <button data-i18n="view.tutorial.btn.trades" class="link" data-go="trades">Trades</button> rollup. Write per-trade notes in <button data-i18n="view.tutorial.btn.journal_2" class="link" data-go="journal">Journal</button>. Any trade with |R| ≥ 2 gets a <button data-i18n="view.tutorial.btn.trade_review" class="link" data-go="reviews">Trade Review</button>. Daily journal includes mood rating for <button data-i18n="view.tutorial.btn.mood_analytics_2" class="link" data-go="mood">Mood Analytics</button>.</li>
                <li><strong>Weekly.</strong> Run <button data-i18n="view.tutorial.btn.reports_2" class="link" data-go="reports">Reports</button> → exit efficiency + R-distribution. Re-evaluate which setups are paying. Update <button data-i18n="view.tutorial.btn.goals" class="link" data-go="goals">Goals</button>.</li>
            </ol>`,
    },
    {
        id: 'troubleshoot',
        title: '// TROUBLESHOOTING',
        body: `
            <ul class="tut-list">
                <li><strong>App fails to launch with "pg_ctl: another server might be running":</strong> An orphan Postgres from a hard kill. Cleanup runs automatically on next launch — just re-open. If it persists, kill any stray <code>postgres</code> process owned by you.</li>
                <li><strong>Widgets show "loading…" forever:</strong> Check <code>~/Library/Application Support/traderview/traderview.log</code> — look for <code>pool timed out</code> (DB pool exhausted) or <code>ERROR</code> lines.</li>
                <li><strong>WebSocket-driven views (Halts / Live Scanner / Catalysts / Webull) silent:</strong> The connect status dot (next to the view title) turns green on connect, red on error, gray on disconnected. Disconnected auto-reconnects in 4s.</li>
                <li><strong>Voice alerts silent:</strong> Voice toggle is per-view. Browser SpeechSynthesis is throttled by macOS; if no voice fires for several events in a row, switch the system voice in System Settings → Accessibility → Spoken Content.</li>
                <li><strong>Premarket / markets data 60s stale:</strong> That's the in-process cache TTL. Yahoo throttles aggressive polling; restart to force-fresh.</li>
            </ul>`,
    },
];

export async function renderTutorial(mount, _state) {
    if (!mount) return;
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;

    mount.innerHTML = `
        <h1 class="view-title">// TUTORIAL
            <input id="tut-q" type="search" placeholder="search…" autocomplete="off">
        </h1>
        <p class="muted small">
            Press <kbd>?</kbd> any time to re-open this. Press <kbd>Cmd</kbd>+<kbd>K</kbd> for the launcher.
        </p>
        <nav class="tut-toc">
            ${SECTIONS.map(s => `<button class="tut-toc-btn" data-jump="${esc(s.id)}">${esc(s.title)}</button>`).join('')}
        </nav>
        <div class="tut-body">
            ${SECTIONS.map(s => `
                <section class="tut-section chart-panel" id="tut-${esc(s.id)}">
                    <h2>${esc(s.title)}</h2>
                    ${s.body}
                </section>
            `).join('')}
        </div>
    `;

    // Tile-jump buttons inside the body — every <button data-go="X"> navigates.
    mount.querySelectorAll('button[data-go]').forEach(b => {
        b.addEventListener('click', () => go(b.dataset.go));
    });

    // TOC jump buttons scroll to section.
    mount.querySelectorAll('button[data-jump]').forEach(b => {
        b.addEventListener('click', () => {
            const target = mount.querySelector('#tut-' + b.dataset.jump);
            if (target) target.scrollIntoView({ behavior: 'smooth', block: 'start' });
        });
    });

    // Filter — shows only sections whose text matches the query.
    const q = mount.querySelector('#tut-q');
    if (q) {
        q.addEventListener('input', () => {
            const needle = q.value.trim().toLowerCase();
            mount.querySelectorAll('.tut-section').forEach(sec => {
                if (!needle) { sec.style.display = ''; return; }
                sec.style.display = sec.textContent.toLowerCase().includes(needle) ? '' : 'none';
            });
        });
        q.focus();
    }
}
