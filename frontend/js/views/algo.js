// Momentum algo trading — strategy CRUD, kill switch, run lifecycle,
// order/fill viewer. All new strategies start internal_sim and the
// backend refuses alpaca_live until paper_locked_until has expired
// (30 days from create). Live trading itself is free on Alpaca; the
// $99/mo Algo Trader Plus subscription only unlocks SIP market data.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm } from '../dialog.js';
import { on as onWsEvent } from '../ws.js';

// Single coalesced refresh on a short debounce — a burst of WS events
// (multi-leg bracket order: parent + take_profit + stop_loss accepts)
// otherwise re-renders the strategies table 3+ times in rapid
// succession. 250ms is short enough that the UI feels live but long
// enough to soak up the burst.
const REFRESH_DEBOUNCE_MS = 250;
let refreshTimer = null;
let wsUnsubs = [];

function scheduleRefresh(mount) {
    if (refreshTimer) clearTimeout(refreshTimer);
    refreshTimer = setTimeout(() => {
        refreshTimer = null;
        refreshStrategies(mount).catch(e => console.warn('algo: refresh failed', e));
    }, REFRESH_DEBOUNCE_MS);
}

function flashRow(mount, strategyId) {
    const tr = mount.querySelector(`tr[data-strat="${strategyId}"]`);
    if (!tr) return;
    tr.style.transition = 'background-color 0.6s';
    tr.style.backgroundColor = 'rgba(0, 229, 255, 0.25)';
    setTimeout(() => { tr.style.backgroundColor = ''; }, 600);
}

// Per-strategy bounded ring buffers. Keys: strategy_id ('all' = global
// firehose). 200 lines each so a 10-strategy account uses ~2k DOM
// lines worst case, still fast.
const STDOUT_MAX_LINES_PER_PANE = 200;
const stdoutBuffers = new Map(); // strategy_id (or 'all') → string[]
let stdoutNameMap = new Map();   // strategy_id → display name

function fmtStdoutTs(d = new Date()) {
    return d.toISOString().slice(11, 23); // HH:MM:SS.mmm
}

function bufferFor(key) {
    if (!stdoutBuffers.has(key)) stdoutBuffers.set(key, []);
    return stdoutBuffers.get(key);
}

function appendStdout(mount, strategyId, line) {
    // Dual-write: per-strategy pane + global firehose pane.
    for (const key of [strategyId, 'all']) {
        const buf = bufferFor(key);
        buf.push(line);
        if (buf.length > STDOUT_MAX_LINES_PER_PANE) buf.shift();
        renderPane(mount, key);
    }
}

function renderPane(mount, key) {
    const pre = mount.querySelector(`[data-stdout-pane="${key}"]`);
    if (!pre) return;
    const filterEl = mount.querySelector(`[data-stdout-filter="${key}"]`);
    const autoEl = mount.querySelector(`[data-stdout-autoscroll="${key}"]`);
    const buf = bufferFor(key);
    const filter = (filterEl?.value || '').trim().toLowerCase();
    const lines = filter ? buf.filter(l => l.toLowerCase().includes(filter)) : buf;
    pre.textContent = lines.join('\n');
    if (autoEl?.checked !== false) pre.scrollTop = pre.scrollHeight;
}

function strategyLabel(id) {
    const name = stdoutNameMap.get(id);
    return name ? `${name} (${id.slice(0, 8)})` : id.slice(0, 8);
}

function logSignal(mount, msg) {
    appendStdout(mount, msg.strategy_id,
        `${fmtStdoutTs()} [${strategyLabel(msg.strategy_id)}] SIGNAL ${msg.side.toUpperCase()} ${msg.symbol} @ ${Number(msg.entry_price).toFixed(2)} (${msg.kind})`);
}
function logOrder(mount, msg) {
    appendStdout(mount, msg.strategy_id,
        `${fmtStdoutTs()} [${strategyLabel(msg.strategy_id)}] ORDER ${msg.side.toUpperCase()} ${msg.symbol} qty=${msg.qty} broker=${msg.broker_order_id.slice(0, 12)}`);
}
function logFill(mount, msg) {
    appendStdout(mount, msg.strategy_id,
        `${fmtStdoutTs()} [${strategyLabel(msg.strategy_id)}] FILL ${msg.symbol} qty=${msg.qty} @ ${Number(msg.price).toFixed(4)}`);
}

function renderStdoutPanes(mount, strategies) {
    const host = mount.querySelector('#algo-stdout-panes');
    if (!host) return;
    const panes = [];
    // Global firehose pane always present at the top.
    panes.push(stdoutPane('all', 'All strategies (firehose)'));
    for (const s of strategies) {
        panes.push(stdoutPane(s.id, s.name));
    }
    host.innerHTML = panes.join('');
    // Re-render every pane's contents after the DOM rebuild.
    for (const key of [...stdoutBuffers.keys()]) renderPane(mount, key);
    // Wire per-pane controls.
    host.querySelectorAll('[data-stdout-clear]').forEach(btn => {
        btn.addEventListener('click', () => {
            const k = btn.dataset.stdoutClear;
            bufferFor(k).length = 0;
            renderPane(mount, k);
        });
    });
    host.querySelectorAll('[data-stdout-filter]').forEach(inp => {
        inp.addEventListener('input', () => renderPane(mount, inp.dataset.stdoutFilter));
    });
}

function stdoutPane(key, title) {
    const titleSafe = esc(title);
    return `
        <details class="algo-stdout-pane" ${key === 'all' ? 'open' : ''}>
            <summary>
                <span>${titleSafe}</span>
                <span class="muted small" data-stdout-pane-count="${esc(key)}"></span>
            </summary>
            <div class="row" style="gap:8px;margin:6px 0">
                <label class="row small" style="gap:4px;align-items:center">
                    <input type="checkbox" data-stdout-autoscroll="${esc(key)}" checked>
                    <span data-i18n="view.algo.label.autoscroll">auto-scroll</span>
                </label>
                <input type="text" data-stdout-filter="${esc(key)}"
                       placeholder="filter…" style="min-width:160px;flex:1">
                <button class="link" data-stdout-clear="${esc(key)}" data-i18n="view.algo.btn.clear">clear</button>
            </div>
            <pre data-stdout-pane="${esc(key)}" class="algo-stdout"></pre>
        </details>
    `;
}

function fmtDateTime(iso) {
    if (!iso) return '—';
    try { return new Date(iso).toLocaleString(); } catch (_) { return iso; }
}

function brokerBadge(mode) {
    if (mode === 'alpaca_live') return '<span class="badge badge-danger" title="Real money via Alpaca live trading">LIVE</span>';
    if (mode === 'alpaca_paper') return '<span class="badge badge-paper" title="Alpaca paper-trading sandbox — no real money">PAPER</span>';
    return '<span class="badge badge-paper" title="In-app simulator — no real money or broker connection">PAPER</span>';
}

function paperLockBadge(iso) {
    if (!iso) return '';
    const ts = new Date(iso).getTime();
    if (Number.isFinite(ts) && ts > Date.now()) {
        const days = Math.ceil((ts - Date.now()) / 86_400_000);
        return `<span class="badge muted" title="paper-locked ${fmtDateTime(iso)}">🔒 ${days}d</span>`;
    }
    return '';
}

function killBadge(s) {
    return s.kill_switch
        ? `<span class="badge badge-danger" title="${esc(s.kill_reason || '')}">KILLED</span>`
        : '';
}

const STRATEGY_KINDS = [
    { value: 'momentum',          label_key: 'view.algo.opt.strat_momentum',          label: 'Momentum (EMA cross + RSI + ROC + RVOL)' },
    { value: 'mean_reversion',    label_key: 'view.algo.opt.strat_mean_reversion',    label: 'Mean Reversion (Connors RSI + VWAP z-score)' },
    { value: 'orb',               label_key: 'view.algo.opt.strat_orb',               label: 'Opening Range Breakout (OR high break + RVOL)' },
    { value: 'donchian_trend',    label_key: 'view.algo.opt.strat_donchian_trend',    label: 'Donchian Trend / Turtle (Donchian + ADX filter)' },
    { value: 'bb_squeeze',        label_key: 'view.algo.opt.strat_bb_squeeze',        label: 'Bollinger Squeeze (BBW bottom decile + band break)' },
    { value: 'ttm_squeeze',       label_key: 'view.algo.opt.strat_ttm_squeeze',       label: 'TTM Squeeze Momentum (BB-in-KC release + histogram)' },
    { value: 'vwap_scalp',        label_key: 'view.algo.opt.strat_vwap_scalp',        label: 'VWAP Scalp (z-score reversion, 1×ATR stop)' },
    { value: 'supertrend',        label_key: 'view.algo.opt.strat_supertrend',        label: 'Supertrend Cross (ATR-banded trend flip)' },
    { value: 'heikin_ashi_trend', label_key: 'view.algo.opt.strat_heikin_ashi_trend', label: 'Heikin-Ashi Trend (HA run + EMA confirm)' },
    { value: 'connors_rsi2',      label_key: 'view.algo.opt.strat_connors_rsi2',      label: 'Connors RSI-2 + 200 SMA (mean-rev classic)' },
    { value: 'order_block_sweep', label_key: 'view.algo.opt.strat_order_block_sweep', label: 'Order Block + Liquidity Sweep (SMC)' },
    { value: 'pead',              label_key: 'view.algo.opt.strat_pead',              label: 'PEAD (post-earnings drift)' },
    { value: 'pairs',             label_key: 'view.algo.opt.strat_pairs',             label: 'Pairs Trading (spread z-score)' },
];

// Rich per-strategy documentation rendered in the modal docs panel
// and in the main-view "Strategy reference" expandable section. Each
// entry: { title, family, entry[], exit[], params[][name,default,desc],
// scope_note, when_to_use }.
const STRATEGY_DOCS = {
    momentum: {
        title: 'Momentum (EMA cross + RSI + ROC + RVOL)',
        family: 'Trend following · long or short',
        entry: [
            'EMA(9) crosses above EMA(21) on the latest bar',
            'RSI(14) is in the [50, 70] band (uptrend but not exhausted)',
            'ROC(10) > 2% (price has accelerated)',
            'Relative Volume (20-bar) ≥ 1.5× (confirming participation)',
        ],
        exit: [
            'Close breaks below EMA(21)',
            'ATR(14) × 2 trailing stop anchored to the high-water mark',
            'RSI(14) falls below 50',
            'MACD bearish crossover',
        ],
        params: [
            ['ema_fast', 9, 'Fast EMA period'],
            ['ema_slow', 21, 'Slow EMA period'],
            ['rsi_period', 14, 'RSI period'],
            ['roc_period', 10, 'Rate-of-Change period'],
            ['rvol_lookback', 20, 'RVOL trailing-average lookback'],
            ['rvol_min', 1.5, 'Minimum relative volume multiple'],
            ['atr_stop_mult', 2.0, 'ATR multiple for the trailing stop'],
        ],
        scope_note: 'Native bracket order: market entry + take-profit limit (+3×ATR) + stop-loss stop (−2×ATR).',
        when_to_use: 'Trending intraday markets with persistent volume. Avoid on news-frozen / range-bound days.',
    },
    mean_reversion: {
        title: 'Mean Reversion (Connors RSI + VWAP z-score)',
        family: 'Counter-trend · long or short',
        entry: [
            'Connors RSI (3, 2, 100) < 10 (extreme oversold composite)',
            'Close < session VWAP − 2σ (deep z-score deviation)',
        ],
        exit: [
            'Price crosses back through session VWAP (mean-reversion target)',
            'ATR(14) × 2 trailing stop anchored to low-water mark',
        ],
        params: [
            ['crsi_oversold', 10, 'CRSI threshold for long entry'],
            ['crsi_overbought', 90, 'CRSI threshold for short entry'],
            ['vwap_z_min', 2.0, 'Required σ deviation from VWAP'],
            ['atr_stop_mult', 2.0, 'ATR multiple for the stop'],
        ],
        scope_note: 'Stop-managed counter-trend bet — risk is invalidation, not "no follow-through". CRSI was invented to catch falling knives; don\'t wait for confirmation that the bounce already happened.',
        when_to_use: 'Range-bound intraday markets, especially after panic selling on no-news. Disable during sustained trending sessions.',
    },
    orb: {
        title: 'Opening Range Breakout (OR high + RVOL)',
        family: 'Breakout · day-trade setup · long or short',
        entry: [
            'First N bars define the opening range (default 15 bars = 15 minutes)',
            'A bar\'s close breaks the OR high (long) or OR low (short)',
            'RVOL(20) ≥ 1.5× on the breakout bar',
            'Only the FIRST breaking bar qualifies — subsequent in-range bars do not',
        ],
        exit: [
            'ATR(14) × 2 trailing stop anchored to high/low-water mark',
        ],
        params: [
            ['opening_bars', 15, 'Bars defining the opening range'],
            ['close_only', true, 'Reject wick-only pierces (close-based only)'],
            ['rvol_min', 1.5, 'Minimum relative-volume multiple'],
            ['atr_stop_mult', 2.0, 'ATR multiple for the trailing stop'],
        ],
        scope_note: 'Crabel\'s canonical day-trade setup. close_only=true filters out liquidity-sweep wicks that hit OR boundaries then reverse.',
        when_to_use: 'Standard equity index futures + high-volume stocks on regular session opens. Avoid earnings days.',
    },
    donchian_trend: {
        title: 'Donchian Trend / Turtle (Donchian + ADX filter)',
        family: 'Trend following · long or short',
        entry: [
            'Close > Donchian(20).upper (long) or close < Donchian(20).lower (short)',
            'ADX(14) > 20 (chop filter — trend strength confirmation)',
        ],
        exit: [
            'Close < Donchian(10).lower (long, the looser turtle exit)',
            'ATR(14) × 2 trailing stop anchored to high-water mark',
            'Whichever fires first',
        ],
        params: [
            ['entry_period', 20, 'Donchian entry-channel lookback'],
            ['exit_period', 10, 'Donchian exit-channel lookback (tighter)'],
            ['adx_period', 14, 'ADX period'],
            ['adx_min', 20.0, 'Minimum ADX to clear chop filter'],
            ['atr_stop_mult', 2.0, 'ATR multiple for the trailing stop'],
        ],
        scope_note: 'Classic Turtle Trader rule set. ADX gate is what separates this from a noise-following whipsaw machine.',
        when_to_use: 'Persistent trending markets. Particularly good on commodity / FX trends; equity uses with 1h+ bars.',
    },
    bb_squeeze: {
        title: 'Bollinger Squeeze Breakout (BBW + band break)',
        family: 'Volatility expansion · long or short',
        entry: [
            'BBW(20, 2) percentile-rank over 100 bars ≤ 10th (squeeze on the PRIOR bar)',
            'Close > BB.upper on the breakout bar (mirror for short)',
        ],
        exit: [
            'Price re-crosses BB.middle (target hit)',
            'ATR(14) × 2 trailing stop',
        ],
        params: [
            ['bb_period', 20, 'Bollinger period'],
            ['bb_k', 2.0, 'Bollinger σ multiple'],
            ['squeeze_lookback', 100, 'BBW percentile-rank window'],
            ['squeeze_pct', 0.10, 'Bottom-decile threshold (0.10 = bottom 10%)'],
            ['atr_stop_mult', 2.0, 'ATR multiple for the trailing stop'],
        ],
        scope_note: 'Squeeze check is evaluated on bar i-1 (BEFORE the breakout). The breakout bar\'s own wide range lifts its own BBW out of the bottom decile, masking the setup.',
        when_to_use: 'Coiled stocks pre-catalyst (earnings, FDA, FOMC). Set timeframe ≥ 5m so the squeeze metric isn\'t dominated by tick noise.',
    },
    ttm_squeeze: {
        title: 'TTM Squeeze Momentum (BB-in-KC release + histogram)',
        family: 'Volatility expansion · long or short',
        entry: [
            'BB inside KC = squeeze ON (volatility coiled)',
            'In the last 5 bars the squeeze RELEASED (BB expanded outside KC again)',
            'Linear-regression momentum histogram > 0 AND > prior bar (long)',
            'Mirror for short with momentum < 0 and falling',
        ],
        exit: [
            'Momentum histogram crosses zero in the unfavorable direction',
            'ATR(14) × 2 trailing stop',
        ],
        params: [
            ['period', 20, 'BB + KC + momentum window'],
            ['bb_mult', 2.0, 'Bollinger σ multiple'],
            ['kc_mult', 1.5, 'Keltner ATR multiple'],
            ['release_lookback', 5, 'Bars after release in which entry stays valid'],
            ['atr_stop_mult', 2.0, 'ATR multiple for the trailing stop'],
        ],
        scope_note: 'release_lookback=5 (not 1) because TTM\'s momentum oscillator lags 1–3 bars behind the BB/KC release. Requiring the release on the current bar misses every practical entry.',
        when_to_use: 'Intraday breakout candidates after a quiet morning consolidation. John Carter\'s canonical setup.',
    },
    vwap_scalp: {
        title: 'VWAP Scalp (z-score reversion · 1× ATR stop)',
        family: 'Intraday scalp · long or short',
        entry: [
            'Close ≤ session VWAP − 2σ (long) or ≥ +2σ (short)',
            'Recovery tick: close > close_prev + 10% of ATR (long; filters falling knives)',
        ],
        exit: [
            'Price crosses back through session VWAP (target)',
            'ATR(14) × 1.0 trailing stop (DEFINING trait of a scalp)',
        ],
        params: [
            ['z_min', 2.0, 'Required σ deviation from VWAP'],
            ['recovery_buffer', 0.10, 'Min recovery as ATR fraction'],
            ['atr_stop_mult', 1.0, 'ATR multiple for the (tight) stop'],
        ],
        scope_note: 'Distinct from Mean Reversion: no CRSI gate. Just pure z-score + the recovery filter. Tight 1×ATR stop is the scalp signature.',
        when_to_use: 'High-volume, mean-reverting tickers (SPY, QQQ intraday). Stop drift / panic candles set up the entry.',
    },
    supertrend: {
        title: 'Supertrend Cross (ATR-banded trend flip)',
        family: 'Trend reversal · long or short',
        entry: [
            'Supertrend trend flag flips: −1 → 1 (long) or 1 → −1 (short)',
        ],
        exit: [
            'Opposite Supertrend flip',
            'ATR-bounded stop: max(supertrend_value, close − 2×ATR) for long',
        ],
        params: [
            ['atr_period', 10, 'ATR period (Seban default)'],
            ['multiplier', 3.0, 'ATR multiple for the bands (Seban default)'],
            ['atr_take_profit_mult', 3.0, 'TP multiple'],
            ['atr_stop_mult', 2.0, 'ATR multiple for the protective stop'],
        ],
        scope_note: 'Simpler than Donchian — fewer rules, no chop filter. Whipsaws in tight ranges; pair with a higher-timeframe regime filter for production use.',
        when_to_use: 'Cleanly trending sessions. Avoid on overnight gap-and-fade days.',
    },
    heikin_ashi_trend: {
        title: 'Heikin-Ashi Trend (HA run + EMA confirm)',
        family: 'Noise-filtered trend follower · long or short',
        entry: [
            '3 consecutive same-color HA candles (green for long, red for short)',
            'Close > EMA(21) (long) or < EMA(21) (short)',
        ],
        exit: [
            'First opposing HA candle',
            'Close < EMA(21) (long)',
            'ATR(14) × 2 trailing stop',
        ],
        params: [
            ['ema_slow', 21, 'Trend confirmation EMA period'],
            ['green_run', 3, 'Consecutive same-color HA candles required'],
            ['atr_stop_mult', 2.0, 'ATR multiple for the trailing stop'],
        ],
        scope_note: 'HA candles smooth wicks — slower turnover than raw-candle strategies. Best on 5m+ bars where the noise filter pays for itself.',
        when_to_use: 'Multi-hour swing trends. Slower than momentum / Supertrend but with fewer false signals.',
    },
    connors_rsi2: {
        title: 'Connors RSI-2 + 200 SMA (mean-rev classic)',
        family: 'Mean reversion · LONG-ONLY',
        entry: [
            'Close > SMA(200) — only buy stocks above the long-term trend',
            'RSI(2) < 5 — extreme oversold on a tight period',
        ],
        exit: [
            'Close > SMA(5) (Connors\'s 5-day touch exit)',
            'RSI(2) > 70',
            'ATR(14) × 2 trailing stop',
        ],
        params: [
            ['sma_trend', 200, 'Long-term trend SMA period'],
            ['sma_exit', 5, 'Short-term exit SMA period'],
            ['rsi_period', 2, 'RSI period (tight)'],
            ['rsi_oversold', 5.0, 'RSI oversold threshold'],
            ['rsi_overbought', 70.0, 'RSI exit threshold'],
        ],
        scope_note: 'Long-only by design. The published edge in stocks BELOW SMA(200) was historically negative; the strategy refuses short side even under SideMode::Short. Larry Connors\'s canonical published edge.',
        when_to_use: 'End-of-day stock screens. Buy pullbacks in established uptrends. Best on individual stocks, not indices.',
    },
    order_block_sweep: {
        title: 'Order Block + Liquidity Sweep (Smart Money Concepts)',
        family: 'Pattern · long or short',
        entry: [
            'In the last 30 bars: bullish order block detected (down-candle before sharp up-expansion)',
            'In the last 30 bars: confirmed liquidity sweep of a recent swing low',
            'Latest bar overlaps the OB zone (price returned to support)',
            'Latest bar closes bullish (close > open)',
            'Mirror for short (bearish OB + high-side sweep)',
        ],
        exit: [
            'ATR(14) × 1.5 trailing stop',
        ],
        params: [
            ['lookback', 30, 'Bars to look back for OB + sweep'],
            ['ob_expansion_window', 3, 'Bars for expansion confirmation'],
            ['ob_expansion_multiple', 2.0, 'Required expansion size (× OB range)'],
            ['grab_min_sweep_atrs', 0.1, 'Min sweep distance in ATRs'],
            ['atr_stop_mult', 1.5, 'ATR multiple for the trailing stop'],
        ],
        scope_note: 'SMC pattern combining institutional-level (OB) with stop-hunt confirmation (liquidity sweep). The dual confirmation is what separates this from a generic support-bounce.',
        when_to_use: 'Stocks with clear structural levels and institutional interest. Tighter stops than momentum — sized to OB-zone breach.',
    },
    pead: {
        title: 'PEAD — Post-Earnings Announcement Drift',
        family: 'Event-driven · LONG-ONLY',
        entry: [
            'Symbol\'s most recent earnings event had surprise_pct ≥ 5%',
            'Within the last 5 days post-announcement',
            '(Both above gated by the runner querying earnings_events)',
            'Latest bar makes a new high above the last 10 bars',
            'Close > SMA(20) (still drifting up)',
        ],
        exit: [
            'ATR(14) × 2 trailing stop',
            'Close < SMA(20) (trend invalidation)',
        ],
        params: [
            ['min_surprise_pct', 5.0, 'Required EPS surprise % (runner gate)'],
            ['max_days_since_earnings', 5, 'Eligibility window post-announcement'],
            ['recent_high_lookback', 10, 'New-high confirmation window'],
            ['short_trend_period', 20, 'Short-term trend SMA'],
            ['atr_stop_mult', 2.0, 'ATR multiple for the trailing stop'],
        ],
        scope_note: 'Two-layer architecture: the runner queries earnings_events to filter the universe; the strategy confirms with technical alignment. Without the fundamental gate this would be just a momentum re-implementation. Ball & Brown / Bernard-Thomas anomaly — long-only.',
        when_to_use: 'After-earnings-season hold strategy. Combine with a watchlist of recently-reported tickers.',
    },
    pairs: {
        title: 'Pairs Trading (spread z-score)',
        family: 'Multi-symbol · relative value · long or short',
        entry: [
            'Set symbol_a, symbol_b, hedge_ratio in entry_rules',
            'spread = ln(price_a) − hedge_ratio × ln(price_b)',
            'Spread\'s rolling z-score ≤ −2σ → long symbol_a (underperformer)',
            'Spread\'s rolling z-score ≥ +2σ → short symbol_a (overperformer, if SideMode allows)',
        ],
        exit: [
            'ATR(14) × 2 trailing stop on the executed leg',
            '(Full z-revert exit is a follow-up — current build relies on the stop)',
        ],
        params: [
            ['symbol_a', '(required)', 'Primary leg (the side that gets traded)'],
            ['symbol_b', '(required)', 'Hedge leg (used only for spread calc)'],
            ['hedge_ratio', 1.0, 'Multiplier for leg B in the spread'],
            ['lookback', 60, 'Bars for z-score rolling window'],
            ['z_entry', 2.0, 'Required deviation σ for entry'],
            ['z_exit', 0.5, 'Mean-revert exit threshold (reserved)'],
        ],
        scope_note: 'True pairs is a simultaneous dual-leg position; this engine emits ONE order per signal, so the strategy fires on the underperforming leg only. For dollar-neutral pairs run two coupled strategies (one long-only on A, one short-only on B) tied to a shared account.',
        when_to_use: 'Closely correlated names (KO/PEP, GLD/SLV, sector ETFs). Set hedge_ratio = 1.0 unless you\'ve done explicit cointegration regression.',
    },
};

// One-line summary fallback — used by anything that still wants the
// terse preview. Computed from the rich docs above.
const STRATEGY_HINTS = Object.fromEntries(
    Object.entries(STRATEGY_DOCS).map(([k, d]) => [k, d.title + ' — ' + d.family])
);

function renderStrategyDoc(kind) {
    const d = STRATEGY_DOCS[kind];
    if (!d) return `<p class="muted small">No docs for ${esc(kind)}.</p>`;
    const ul = (arr) => arr.map(li => `<li>${esc(li)}</li>`).join('');
    const paramRow = ([name, def, desc]) =>
        `<tr><td><code>${esc(name)}</code></td><td>${esc(String(def))}</td><td>${esc(desc)}</td></tr>`;
    return `
        <div class="algo-doc">
            <h3>${esc(d.title)}</h3>
            <p class="muted small algo-doc-family">${esc(d.family)}</p>
            <div class="algo-doc-cols">
                <div>
                    <strong data-i18n="view.algo.doc.entry">Entry rules</strong>
                    <ul>${ul(d.entry)}</ul>
                </div>
                <div>
                    <strong data-i18n="view.algo.doc.exit">Exit rules</strong>
                    <ul>${ul(d.exit)}</ul>
                </div>
            </div>
            <strong data-i18n="view.algo.doc.params">Parameters</strong>
            <table class="trades algo-doc-params">
                <thead><tr><th>Name</th><th>Default</th><th>Description</th></tr></thead>
                <tbody>${d.params.map(paramRow).join('')}</tbody>
            </table>
            <p class="muted small"><strong data-i18n="view.algo.doc.scope_note">Note:</strong> ${esc(d.scope_note)}</p>
            <p class="muted small"><strong data-i18n="view.algo.doc.when_to_use">When to use:</strong> ${esc(d.when_to_use)}</p>
        </div>
    `;
}

export async function renderAlgo(mount) {
    mount.innerHTML = `
        <h1 data-i18n="view.algo.h1.algo_trading" class="view-title">// ALGO TRADING</h1>
        <p class="muted small" data-i18n="view.algo.hint.intro">
            Five strategy families: momentum, mean reversion, opening range breakout, Donchian trend, Bollinger squeeze.
            Native bracket orders with strategy-specific stops / take-profits. 1% risk per trade, capped by max position % of equity.
            New strategies are paper-locked for 30 days.
        </p>

        <div class="chart-panel">
            <div class="row" style="justify-content:space-between;align-items:center">
                <h2 data-i18n="view.algo.h2.strategies">Strategies</h2>
                <button id="algo-new" class="primary" data-i18n="view.algo.btn.new_strategy">New strategy</button>
            </div>
            <table class="trades" id="algo-strats-table">
                <thead><tr>
                    <th data-i18n="view.algo.th.name">Name</th>
                    <th data-i18n="view.algo.th.strategy_type">Strategy</th>
                    <th data-i18n="view.algo.th.timeframe">TF</th>
                    <th data-i18n="view.algo.th.universe">Universe</th>
                    <th data-i18n="view.algo.th.side">Side</th>
                    <th data-i18n="view.algo.th.broker">Broker</th>
                    <th data-i18n="view.algo.th.status">Status</th>
                    <th></th>
                </tr></thead>
                <tbody><tr><td colspan="8" class="muted">${esc(t('view.algo.loading'))}</td></tr></tbody>
            </table>
        </div>

        <details class="chart-panel" id="algo-docs-panel">
            <summary>
                <span data-i18n="view.algo.h2.docs">Strategy reference</span>
                <span class="muted small" data-i18n="view.algo.hint.docs">— full rules for all 13 strategies</span>
            </summary>
            <div class="row" style="gap:6px;flex-wrap:wrap;margin:8px 0">
                ${STRATEGY_KINDS.map(k => `
                    <button class="btn btn-secondary algo-docs-tab" data-kind="${k.value}">${esc(k.value)}</button>
                `).join('')}
            </div>
            <div id="algo-docs-body">${renderStrategyDoc('momentum')}</div>
        </details>

        <div class="chart-panel" id="algo-stdout-panel">
            <div class="row" style="justify-content:space-between;align-items:center">
                <h2 data-i18n="view.algo.h2.stdout">Live stdout</h2>
                <span class="muted small" data-i18n="view.algo.hint.stdout_multi">
                    One pane per strategy + a global firehose. Each pane keeps its own 200-line buffer.
                </span>
            </div>
            <div id="algo-stdout-panes"></div>
        </div>

        <div id="algo-runs" class="chart-panel" style="display:none">
            <h2 data-i18n="view.algo.h2.runs">Recent runs</h2>
            <table class="trades" id="algo-runs-table">
                <thead><tr>
                    <th data-i18n="view.algo.th.started">Started</th>
                    <th data-i18n="view.algo.th.stopped">Stopped</th>
                    <th data-i18n="view.algo.th.reason">Reason</th>
                    <th data-i18n="view.algo.th.bars">Bars</th>
                    <th data-i18n="view.algo.th.signals">Signals</th>
                    <th data-i18n="view.algo.th.orders">Orders</th>
                    <th data-i18n="view.algo.th.fills">Fills</th>
                    <th data-i18n="view.algo.th.pnl">P&amp;L</th>
                    <th></th>
                </tr></thead>
                <tbody></tbody>
            </table>
        </div>

        <div id="algo-orders" class="chart-panel" style="display:none">
            <h2 data-i18n="view.algo.h2.orders">Orders</h2>
            <table class="trades" id="algo-orders-table">
                <thead><tr>
                    <th data-i18n="view.algo.th.submitted">Submitted</th>
                    <th data-i18n="view.algo.th.symbol">Symbol</th>
                    <th data-i18n="view.algo.th.side">Side</th>
                    <th data-i18n="view.algo.th.qty">Qty</th>
                    <th data-i18n="view.algo.th.type">Type</th>
                    <th data-i18n="view.algo.th.class">Class</th>
                    <th data-i18n="view.algo.th.stop">Stop</th>
                    <th data-i18n="view.algo.th.status">Status</th>
                    <th data-i18n="view.algo.th.broker_order_id">Broker id</th>
                </tr></thead>
                <tbody></tbody>
            </table>
        </div>

        <div id="algo-modal-host"></div>
    `;

    mount.querySelector('#algo-new').addEventListener('click', () => openStrategyModal(mount));
    // Strategy reference tabs — clicking a button swaps the rendered doc.
    mount.querySelectorAll('.algo-docs-tab').forEach(btn => {
        btn.addEventListener('click', () => {
            mount.querySelector('#algo-docs-body').innerHTML = renderStrategyDoc(btn.dataset.kind);
        });
    });
    await refreshStrategies(mount);

    // Tear down any prior subscriptions (view re-mount on tab switch)
    // before wiring fresh ones — duplicates would multiply the refresh
    // calls per event AND duplicate stdout lines.
    wsUnsubs.forEach(unsub => { try { unsub(); } catch (_) {} });
    wsUnsubs = [];
    wsUnsubs.push(onWsEvent('algo_signal_fired', (msg) => {
        flashRow(mount, msg.strategy_id);
        logSignal(mount, msg);
        scheduleRefresh(mount);
    }));
    wsUnsubs.push(onWsEvent('algo_order_submitted', (msg) => {
        flashRow(mount, msg.strategy_id);
        logOrder(mount, msg);
        scheduleRefresh(mount);
    }));
    wsUnsubs.push(onWsEvent('algo_fill_received', (msg) => {
        flashRow(mount, msg.strategy_id);
        logFill(mount, msg);
        scheduleRefresh(mount);
    }));
}

async function refreshStrategies(mount) {
    const table = mount.querySelector('#algo-strats-table tbody');
    let strategies;
    try {
        strategies = await api.listAlgoStrategies();
    } catch (e) {
        table.innerHTML = `<tr><td colspan="7" class="muted">${esc(t('view.algo.load_error'))}: ${esc(e.message || e)}</td></tr>`;
        return;
    }
    // Keep the strategy_id → name map in sync so stdout shows readable
    // labels instead of bare UUID prefixes.
    stdoutNameMap = new Map(strategies.map(s => [s.id, s.name]));
    // Rebuild stdout panes — one per strategy + the global firehose.
    // Existing buffers preserve their contents (Map persists across the
    // call). Newly-created strategies get a fresh pane; deleted ones
    // disappear from the UI but their buffer lingers until view re-mount.
    renderStdoutPanes(mount, strategies);
    if (!strategies.length) {
        table.innerHTML = `<tr><td colspan="8" class="muted">${esc(t('view.algo.empty'))}</td></tr>`;
        return;
    }
    table.innerHTML = strategies.map(s => `
        <tr data-strat="${s.id}">
            <td>${esc(s.name)} ${paperLockBadge(s.paper_locked_until)} ${killBadge(s)}</td>
            <td>${esc(s.strategy_type || 'momentum')}</td>
            <td>${esc(s.timeframe)}</td>
            <td>${esc(s.universe_mode)}${s.universe_mode === 'autoscan' ? ` (top ${s.autoscan_top_n})` : ''}</td>
            <td>${esc(s.side_mode)}</td>
            <td>${brokerBadge(s.broker_mode)}</td>
            <td>${s.enabled ? '<span class="badge">enabled</span>' : '<span class="badge muted">disabled</span>'}</td>
            <td class="row" style="gap:4px">
                <button class="link" data-act="runs" data-i18n="view.algo.btn.runs">runs</button>
                <button class="link" data-act="start" data-i18n="view.algo.btn.start">start</button>
                <button class="link" data-act="stop" data-i18n="view.algo.btn.stop">stop</button>
                <button class="link" data-act="kill" data-i18n="view.algo.btn.kill">${s.kill_switch ? 'release' : 'kill'}</button>
                <button class="link" data-act="edit" data-i18n="view.algo.btn.edit">edit</button>
                <button class="link" data-act="del" data-i18n="view.algo.btn.delete">delete</button>
            </td>
        </tr>
    `).join('');

    table.querySelectorAll('button[data-act]').forEach(btn => {
        btn.addEventListener('click', async (e) => {
            const tr = e.target.closest('tr');
            const id = tr.dataset.strat;
            const s = strategies.find(x => x.id === id);
            const act = btn.dataset.act;
            if (act === 'runs') return showRuns(mount, s);
            if (act === 'start') return startRun(mount, s);
            if (act === 'stop') return stopRun(mount, s);
            if (act === 'kill') return toggleKill(mount, s);
            if (act === 'edit') return openStrategyModal(mount, s);
            if (act === 'del') return deleteStrategy(mount, s);
        });
    });
}

async function startRun(mount, s) {
    try {
        const r = await api.startAlgoRun(s.id);
        showToast(t('view.algo.toast.run_started', { id: r.id.slice(0, 8) }), { level: 'success' });
        await refreshStrategies(mount);
        await showRuns(mount, s);
    } catch (e) {
        showToast(`${t('view.algo.toast.run_start_failed')}: ${e.message || e}`, { level: 'error' });
    }
}

async function stopRun(mount, s) {
    try {
        await api.stopAlgoRun(s.id, 'user');
        showToast(t('view.algo.toast.run_stopped'), { level: 'success' });
        await showRuns(mount, s);
    } catch (e) {
        showToast(`${t('view.algo.toast.run_stop_failed')}: ${e.message || e}`, { level: 'error' });
    }
}

async function toggleKill(mount, s) {
    if (!s.kill_switch) {
        const ok = await tConfirm(t('view.algo.confirm.engage_kill'));
        if (!ok) return;
        const reason = prompt(t('view.algo.prompt.kill_reason')) || null;
        try {
            await api.setAlgoKillSwitch(s.id, true, reason);
            showToast(t('view.algo.toast.kill_engaged'), { level: 'warning' });
        } catch (e) {
            showToast(`${t('view.algo.toast.kill_failed')}: ${e.message || e}`, { level: 'error' });
        }
    } else {
        const ok = await tConfirm(t('view.algo.confirm.release_kill'));
        if (!ok) return;
        try {
            await api.setAlgoKillSwitch(s.id, false, null);
            showToast(t('view.algo.toast.kill_released'), { level: 'success' });
        } catch (e) {
            showToast(`${t('view.algo.toast.kill_failed')}: ${e.message || e}`, { level: 'error' });
        }
    }
    await refreshStrategies(mount);
}

async function deleteStrategy(mount, s) {
    const ok = await tConfirm(t('view.algo.confirm.delete', { name: s.name }));
    if (!ok) return;
    try {
        await api.deleteAlgoStrategy(s.id);
        showToast(t('view.algo.toast.deleted'), { level: 'success' });
        await refreshStrategies(mount);
    } catch (e) {
        showToast(`${t('view.algo.toast.delete_failed')}: ${e.message || e}`, { level: 'error' });
    }
}

async function showRuns(mount, s) {
    const panel = mount.querySelector('#algo-runs');
    const tbody = mount.querySelector('#algo-runs-table tbody');
    panel.style.display = 'block';
    tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.algo.loading'))}</td></tr>`;
    let runs;
    try { runs = await api.listAlgoRuns(s.id, 25); }
    catch (e) { tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(e.message)}</td></tr>`; return; }
    if (!runs.length) {
        tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.algo.empty_runs'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = runs.map(r => `
        <tr data-run="${r.id}">
            <td>${fmtDateTime(r.started_at)}</td>
            <td>${fmtDateTime(r.stopped_at)}</td>
            <td>${esc(r.stopped_reason || '—')}</td>
            <td>${r.bars_processed}</td>
            <td>${r.signals_emitted}</td>
            <td>${r.orders_submitted}</td>
            <td>${r.fills_received}</td>
            <td>${esc(String(r.pnl_realized || 0))}</td>
            <td><button class="link" data-act="orders" data-i18n="view.algo.btn.orders">orders</button></td>
        </tr>
    `).join('');
    tbody.querySelectorAll('button[data-act="orders"]').forEach(btn => {
        btn.addEventListener('click', (e) => {
            const tr = e.target.closest('tr');
            showOrders(mount, tr.dataset.run);
        });
    });
}

async function showOrders(mount, runId) {
    const panel = mount.querySelector('#algo-orders');
    const tbody = mount.querySelector('#algo-orders-table tbody');
    panel.style.display = 'block';
    tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.algo.loading'))}</td></tr>`;
    let orders;
    try { orders = await api.listAlgoOrders(runId, 100); }
    catch (e) { tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(e.message)}</td></tr>`; return; }
    if (!orders.length) {
        tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.algo.empty_orders'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = orders.map(o => `
        <tr>
            <td>${fmtDateTime(o.submitted_at)}</td>
            <td>${esc(o.symbol)}</td>
            <td>${esc(o.side)}</td>
            <td>${esc(String(o.qty))}</td>
            <td>${esc(o.order_type)}</td>
            <td>${esc(o.order_class)}</td>
            <td>${esc(String(o.stop_price || '—'))}</td>
            <td>${esc(o.status)}</td>
            <td class="small muted">${esc(o.broker_order_id || '—')}</td>
        </tr>
    `).join('');
}

async function openStrategyModal(mount, existing = null) {
    const host = mount.querySelector('#algo-modal-host');
    const s = existing || {
        name: '',
        enabled: true,
        timeframe: 'min1',
        universe_mode: 'watchlist',
        watchlist_id: null,
        autoscan_top_n: 25,
        side_mode: 'long',
        strategy_type: 'momentum',
        account_id: null,
        entry_rules: {},
        exit_rules: {},
        sizing: { risk_pct_per_trade: 0.01, max_pos_pct: 0.20 },
        risk_gates: { max_concurrent_positions: 5, daily_loss_limit_pct: 0.03, max_drawdown_pct: 0.10 },
        broker_mode: 'internal_sim',
    };
    let accounts = [];
    try { accounts = await api.accounts(); }
    catch (e) { console.warn('algo modal: api.accounts() failed', e); }
    // Only brokers with an API integration we can drive an algo against.
    // Today: Alpaca (REST + WS, fully wired). Extend this list as more
    // broker integrations land; the backend route enforces the same
    // constraint independently (defense-in-depth).
    // Scaffolded set; real implementation status differs (see
    // ALGO_BROKER_STATUS). UI lets users pick any of them — backend's
    // broker_dispatcher errors at submit time for the not-yet-real
    // adapters so the strategy can still be saved + scheduled.
    const algoBroker = (b) =>
        ['alpaca', 'tradier', 'ibkr', 'td', 'schwab', 'tastytrade']
            .includes(String(b || '').toLowerCase());
    // broker_mode is now broker-agnostic — internal_sim / paper / live.
    // The right paper/live endpoint is picked by the dispatcher based
    // on account.broker; the UI just needs to gate the account list on
    // the algoBroker set.
    const eligibleAccounts = accounts.filter(a => algoBroker(a.broker));
    const accountOptions = eligibleAccounts.length
        ? eligibleAccounts.map(a => {
            const sel = s.account_id === a.id ? 'selected' : '';
            const label = a.broker ? `${a.name} · ${a.broker}` : a.name;
            return `<option value="${esc(a.id)}" ${sel}>${esc(label)}</option>`;
        }).join('')
        : '';
    const stratOptions = STRATEGY_KINDS.map(k => {
        const sel = (s.strategy_type || 'momentum') === k.value ? 'selected' : '';
        return `<option value="${k.value}" ${sel} data-i18n="${k.label_key}">${esc(k.label)}</option>`;
    }).join('');
    host.innerHTML = `
        <div class="modal">
            <div class="modal-inner">
                <h2 data-i18n="view.algo.h2.${existing ? 'edit_strategy' : 'new_strategy'}">${existing ? 'Edit' : 'New'} strategy</h2>
                <form id="algo-form" class="algo-form">
                    <label><span data-i18n="view.algo.label.name">Name</span>
                        <input name="name" value="${esc(s.name)}" required>
                    </label>
                    <label><span data-i18n="view.algo.label.account">Broker account</span>
                        <select name="account_id" required>
                            ${accountOptions || `<option value="">${esc(t('view.algo.label.no_accounts'))}</option>`}
                        </select>
                    </label>
                    ${eligibleAccounts.length ? '' : `<p class="muted small" style="margin:-4px 0 0;color:var(--red)">
                        ${esc(t(accounts.length ? 'view.algo.hint.no_algo_accounts' : 'view.algo.hint.no_accounts'))}
                    </p>`}
                    <label><span data-i18n="view.algo.label.strategy_type">Strategy</span>
                        <select name="strategy_type" id="algo-strategy-type">
                            ${stratOptions}
                        </select>
                    </label>
                    <div id="algo-strategy-doc" class="algo-doc-modal">
                        ${renderStrategyDoc(s.strategy_type || 'momentum')}
                    </div>
                    <label class="checkbox-row">
                        <input type="checkbox" name="enabled" ${s.enabled ? 'checked' : ''}>
                        <span data-i18n="view.algo.label.enabled">Enabled</span>
                    </label>
                    <label><span data-i18n="view.algo.label.timeframe">Timeframe</span>
                        <select name="timeframe">
                            <option value="sec10" ${s.timeframe === 'sec10' ? 'selected' : ''} data-i18n="view.algo.opt.tf_10s">10 seconds</option>
                            <option value="min1"  ${s.timeframe === 'min1'  ? 'selected' : ''} data-i18n="view.algo.opt.tf_1m">1 minute</option>
                        </select>
                    </label>
                    <label><span data-i18n="view.algo.label.universe">Universe</span>
                        <select name="universe_mode">
                            <option value="watchlist" ${s.universe_mode === 'watchlist' ? 'selected' : ''} data-i18n="view.algo.opt.uni_watchlist">User watchlist</option>
                            <option value="autoscan"  ${s.universe_mode === 'autoscan'  ? 'selected' : ''} data-i18n="view.algo.opt.uni_autoscan">Autoscan (top by RVOL)</option>
                        </select>
                    </label>
                    <label><span data-i18n="view.algo.label.autoscan_top_n">Autoscan top N</span>
                        <input type="number" name="autoscan_top_n" min="1" max="500" value="${Number(s.autoscan_top_n) || 25}">
                    </label>
                    <label><span data-i18n="view.algo.label.side">Side</span>
                        <select name="side_mode">
                            <option value="long"  ${s.side_mode === 'long'  ? 'selected' : ''} data-i18n="view.algo.opt.side_long">Long only</option>
                            <option value="short" ${s.side_mode === 'short' ? 'selected' : ''} data-i18n="view.algo.opt.side_short">Short only</option>
                            <option value="both"  ${s.side_mode === 'both'  ? 'selected' : ''} data-i18n="view.algo.opt.side_both">Long + short</option>
                        </select>
                    </label>
                    <label><span data-i18n="view.algo.label.risk_pct">Risk per trade (fraction of equity)</span>
                        <input type="number" name="risk_pct_per_trade" step="0.001" min="0.001" max="0.10" value="${Number(s.sizing?.risk_pct_per_trade ?? 0.01)}">
                    </label>
                    <label><span data-i18n="view.algo.label.max_pos_pct">Max position % of equity</span>
                        <input type="number" name="max_pos_pct" step="0.01" min="0.01" max="1.0" value="${Number(s.sizing?.max_pos_pct ?? 0.20)}">
                    </label>
                    <label><span data-i18n="view.algo.label.max_concurrent">Max concurrent positions</span>
                        <input type="number" name="max_concurrent_positions" min="1" max="50" value="${Number(s.risk_gates?.max_concurrent_positions ?? 5)}">
                    </label>
                    <label><span data-i18n="view.algo.label.broker_mode">Execution mode</span>
                        <select name="broker_mode">
                            <option value="internal_sim"  ${s.broker_mode === 'internal_sim'  ? 'selected' : ''} data-i18n="view.algo.opt.broker_sim">Paper — In-app simulator (no broker call)</option>
                            <option value="paper"         ${s.broker_mode === 'paper'         ? 'selected' : ''} data-i18n="view.algo.opt.broker_paper">Paper — Broker sandbox (uses selected account)</option>
                            <option value="live"          ${s.broker_mode === 'live'          ? 'selected' : ''} data-i18n="view.algo.opt.broker_live">LIVE — Real money (after 30-day paper-lock)</option>
                        </select>
                    </label>
                    <p class="muted small" style="margin:0" data-i18n="view.algo.hint.execution_safety">
                        in-app sim fills at last known price (zero-risk). Broker paper uses the broker's sandbox (Alpaca: paper-api; Tradier: sandbox; etc.). Live trades real money and requires the 30-day paper-lock to expire.
                    </p>
                    <p class="muted small" style="margin:0" data-i18n="view.algo.hint.broker_status">
                        Adapter status: <strong>alpaca</strong> wired; <strong>tradier</strong> scaffolded (coming next); <strong>ibkr</strong> / <strong>td</strong> / <strong>tastytrade</strong> scaffolded — strategy saves and runs, but orders return integration_pending until the per-broker adapter ships.
                    </p>
                    <div class="algo-form-actions">
                        <button type="button" id="algo-cancel" data-i18n="view.algo.btn.cancel">Cancel</button>
                        <button type="submit" id="algo-save" class="primary"
                                ${eligibleAccounts.length ? '' : 'disabled'}
                                data-i18n="view.algo.btn.save">Save</button>
                    </div>
                </form>
            </div>
        </div>
        <style>
            .algo-form { display: flex; flex-direction: column; gap: 12px; min-width: 420px; }
            .algo-form label { display: flex; flex-direction: column; gap: 4px; }
            .algo-form label > span { color: var(--text-dim); font-size: 11px; text-transform: uppercase; letter-spacing: 1px; }
            .algo-form input, .algo-form select { width: 100%; box-sizing: border-box; }
            .algo-form label.checkbox-row { flex-direction: row; align-items: center; gap: 8px; }
            .algo-form label.checkbox-row > span { text-transform: none; letter-spacing: normal; font-size: 13px; color: var(--text); }
            .algo-form label.checkbox-row > input { width: auto; }
            .algo-form-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 8px; }
        </style>
    `;
    host.querySelector('#algo-cancel').addEventListener('click', () => { host.innerHTML = ''; });
    host.querySelector('#algo-strategy-type').addEventListener('change', (e) => {
        const v = e.target.value;
        host.querySelector('#algo-strategy-doc').innerHTML = renderStrategyDoc(v);
    });
    // No per-mode account filtering needed anymore — broker_mode is
    // broker-agnostic (internal_sim / paper / live) and the account's
    // broker is what selects the adapter at submit time.
    host.querySelector('#algo-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const f = new FormData(e.target);
        const body = {
            name: f.get('name').trim(),
            enabled: f.get('enabled') === 'on',
            timeframe: f.get('timeframe'),
            universe_mode: f.get('universe_mode'),
            watchlist_id: s.watchlist_id || null,
            autoscan_top_n: Number(f.get('autoscan_top_n')) || 25,
            side_mode: f.get('side_mode'),
            strategy_type: f.get('strategy_type') || 'momentum',
            account_id: f.get('account_id') || null,
            entry_rules: s.entry_rules || {},
            exit_rules: s.exit_rules || {},
            sizing: {
                risk_pct_per_trade: Number(f.get('risk_pct_per_trade')) || 0.01,
                max_pos_pct: Number(f.get('max_pos_pct')) || 0.20,
            },
            risk_gates: Object.assign({}, s.risk_gates || {}, {
                max_concurrent_positions: Number(f.get('max_concurrent_positions')) || 5,
            }),
            broker_mode: f.get('broker_mode'),
        };
        try {
            if (existing) {
                await api.updateAlgoStrategy(existing.id, body);
                showToast(t('view.algo.toast.updated'), { level: 'success' });
            } else {
                await api.createAlgoStrategy(body);
                showToast(t('view.algo.toast.created'), { level: 'success' });
            }
            host.innerHTML = '';
            await refreshStrategies(mount);
        } catch (err) {
            showToast(`${t('view.algo.toast.save_failed')}: ${err.message || err}`, { level: 'error' });
        }
    });
}
