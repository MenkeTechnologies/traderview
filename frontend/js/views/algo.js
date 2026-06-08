// Momentum algo trading — strategy CRUD, kill switch, run lifecycle,
// order/fill viewer. All new strategies start internal_sim and the
// backend refuses alpaca_live until paper_locked_until has expired
// (30 days from create). Live trading itself is free on Alpaca; the
// $99/mo Algo Trader Plus subscription only unlocks SIP market data.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm, tPrompt } from '../dialog.js';
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
// Throttle TickSkipped / BarEvaluated rendering — they can fire many
// times per tick across N symbols and would otherwise drown the
// stdout pane in repeated lines.
const lastEmitAt = new Map(); // `${strategyId}:${reason}` → epoch ms
function shouldEmit(key, intervalMs = 5000) {
    const now = Date.now();
    const prev = lastEmitAt.get(key) || 0;
    if (now - prev < intervalMs) return false;
    lastEmitAt.set(key, now);
    return true;
}
function logTickSkipped(mount, msg) {
    const key = `${msg.strategy_id}:skip:${msg.reason}`;
    if (!shouldEmit(key)) return;
    appendStdout(mount, msg.strategy_id,
        `${fmtStdoutTs()} [${strategyLabel(msg.strategy_id)}] SKIP ${msg.reason}`);
}
function logBarEvaluated(mount, msg) {
    // One heartbeat per (strategy, symbol) every 30s — proves the
    // engine is alive without flooding when nothing's firing.
    const key = `${msg.strategy_id}:eval:${msg.symbol}`;
    if (!shouldEmit(key, 30000)) return;
    appendStdout(mount, msg.strategy_id,
        `${fmtStdoutTs()} [${strategyLabel(msg.strategy_id)}] EVAL ${msg.symbol} bars=${msg.bars} (no signal)`);
}
function logHeartbeat(mount, msg) {
    // Fires every 10s from the runner. Confirms the engine is alive
    // and shows what it knows right now — live-tick subs, bars
    // processed lifetime, signals emitted lifetime, time to the
    // next bar boundary eval.
    appendStdout(mount, msg.strategy_id,
        `${fmtStdoutTs()} [${strategyLabel(msg.strategy_id)}] HB live=${msg.subscribed_live} bars=${msg.bars_processed} sigs=${msg.signals_emitted} next_eval_in=${msg.seconds_to_next_eval}s`);
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
    { value: 'ma_cross_adx',      label_key: 'view.algo.opt.strat_ma_cross_adx',      label: 'MA Cross + ADX filter (EMA cross gated on trend strength)' },
    { value: 'keltner_breakout',  label_key: 'view.algo.opt.strat_keltner_breakout',  label: 'Keltner Channel Breakout (EMA ± ATR band)' },
    { value: 'ichimoku_cloud',    label_key: 'view.algo.opt.strat_ichimoku_cloud',    label: 'Ichimoku Cloud (TK cross + cloud break + Chikou clear)' },
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
    ma_cross_adx: {
        title: 'MA Cross + ADX filter',
        family: 'Trend-following · long or short · gated on trend strength',
        entry: [
            'Compute EMA(fast_period) and EMA(slow_period) over closes',
            'Compute ADX(adx_period) + ±DI',
            'Long: fast EMA crosses above slow EMA AND ADX ≥ adx_min AND +DI > −DI',
            'Short: fast EMA crosses below slow EMA AND ADX ≥ adx_min AND −DI > +DI',
        ],
        exit: [
            'ADX drops below adx_trend_lost → exit (trend has weakened)',
            'Opposite-direction EMA crossover',
            'ATR-multiple stop / take-profit (atr_stop_mult / atr_take_profit_mult)',
        ],
        params: [
            ['fast_period', 9, 'Fast EMA window'],
            ['slow_period', 21, 'Slow EMA window'],
            ['adx_period', 14, 'ADX/DI window'],
            ['adx_min', 25, 'Minimum ADX to allow entry (25 = canonical "trending" threshold)'],
            ['adx_trend_lost', 18, 'Force exit when ADX falls below this'],
            ['atr_period', 14, 'ATR window for stop/target sizing'],
            ['atr_stop_mult', 1.5, 'Stop distance in ATRs'],
            ['atr_take_profit_mult', 3.0, 'Target distance in ATRs'],
        ],
        scope_note: 'The ADX gate is the entire point — naked MA crosses get chopped to death in sideways markets. ADX ≥ 25 ensures the directional component is genuinely in force.',
        when_to_use: 'Multi-bar trends. Strong daily/hourly trends in liquid names. Useless intraday in tight ranges.',
    },
    keltner_breakout: {
        title: 'Keltner Channel Breakout',
        family: 'Volatility breakout · long or short',
        entry: [
            'upper = EMA(period) + multiplier × ATR(atr_period)',
            'lower = EMA(period) − multiplier × ATR(atr_period)',
            'Long: prior close ≤ upper AND current close > upper (fresh breakout)',
            'Short: prior close ≥ lower AND current close < lower',
        ],
        exit: [
            'Close back through the EMA midline (channel break failed)',
            'Touch of the opposite band',
        ],
        params: [
            ['period', 20, 'EMA window'],
            ['atr_period', 20, 'ATR window'],
            ['multiplier', 1.5, 'Band width in ATR multiples'],
            ['take_profit_mult', 2.0, 'Target = multiplier × channel half-width × this'],
        ],
        scope_note: 'Channel half-width = multiplier × ATR. Wider channels = stronger breakouts but bigger initial risk. Stop sits at the EMA midline.',
        when_to_use: 'Range-then-breakout setups. Pairs well with squeeze detectors as a confirmation gate.',
    },
    ichimoku_cloud: {
        title: 'Ichimoku Cloud (Kinkō Hyō)',
        family: 'Trend + momentum + structure · long or short · 5-line confluence',
        entry: [
            'Compute Tenkan (9), Kijun (26), Senkou A/B (displaced 26 forward), Chikou (close shifted 26 back)',
            'Long: close > cloud AND TK bullish cross AND Chikou clear of prior price AND Senkou A > Senkou B (green cloud)',
            'Short: mirror all four conditions',
        ],
        exit: [
            'Close crosses back through Kijun (baseline broken)',
            'Opposite-direction TK cross',
        ],
        params: [
            ['tenkan_period', 9, 'Conversion line period'],
            ['kijun_period', 26, 'Base line period'],
            ['senkou_b_period', 52, 'Leading span B period'],
            ['displacement', 26, 'Senkou/Chikou shift (rarely changed)'],
            ['take_profit_cloud_mult', 2.0, 'Target distance as cloud-thickness multiple'],
        ],
        scope_note: 'Requires 4 conditions in confluence — far fewer false positives than naive moving-average crossovers but you wait longer between trades. Stop is at the Kijun line; target scales with cloud thickness so quiet markets get small targets and volatile markets get large ones.',
        when_to_use: 'Daily / 4H charts on liquid names. Don\'t expect intraday signals on slow stocks — the 52-bar Senkou B requires a meaningful window.',
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

        <details class="chart-panel" id="algo-tape-panel" open>
            <summary><h2 style="display:inline" data-i18n="view.algo.h2.tape">Raw tape — live trades off the Alpaca WS</h2></summary>
            <p class="muted small" data-i18n="view.algo.hint.tape">
                Every trade frame the live-tick worker parses off the WS. Per-symbol coalesce of 250ms so a chatty pair doesn't flood the pane.
                Empty pane = the WS isn't producing trades for any subscribed symbol right now.
            </p>
            <div class="row" style="gap:8px;margin-bottom:6px">
                <input type="text" id="algo-tape-filter" placeholder="filter symbol (e.g. BTC)" style="flex:1 1 auto">
                <label style="white-space:nowrap"><input type="checkbox" id="algo-tape-autoscroll" checked> autoscroll</label>
                <button class="link" id="algo-tape-clear" data-i18n="view.algo.btn.clear">clear</button>
            </div>
            <pre id="algo-tape" class="algo-stdout" style="max-height:280px;overflow:auto"></pre>
        </details>

        <div id="algo-runs" class="chart-panel" style="display:none">
            <h2><span data-i18n="view.algo.h2.runs">Recent runs</span> — <span id="algo-runs-strategy-name" class="muted">…</span></h2>
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
    // Diagnostic events — no refresh, no flash, just stdout visibility.
    wsUnsubs.push(onWsEvent('algo_tick_skipped', (msg) => {
        logTickSkipped(mount, msg);
    }));
    wsUnsubs.push(onWsEvent('algo_bar_evaluated', (msg) => {
        logBarEvaluated(mount, msg);
    }));
    wsUnsubs.push(onWsEvent('algo_heartbeat', (msg) => {
        logHeartbeat(mount, msg);
    }));
    wsUnsubs.push(onWsEvent('tick', (msg) => {
        logTape(mount, msg);
    }));
    const clearBtn = mount.querySelector('#algo-tape-clear');
    if (clearBtn) {
        clearBtn.addEventListener('click', () => {
            const pane = mount.querySelector('#algo-tape');
            if (pane) pane.textContent = '';
            tapeLastEmit.clear();
        });
    }
}

// Raw tape pane — coalesce per (symbol) at 250ms so a chatty pair
// (BTC/USD doing 100 trades/sec at peak) doesn't drown other symbols.
const tapeLastEmit = new Map();
const TAPE_MAX_LINES = 500;
function logTape(mount, msg) {
    const last = tapeLastEmit.get(msg.symbol) || 0;
    const now = performance.now();
    if (now - last < 250) return;
    tapeLastEmit.set(msg.symbol, now);
    const pane = mount.querySelector('#algo-tape');
    if (!pane) return;
    const filter = (mount.querySelector('#algo-tape-filter')?.value || '').trim().toLowerCase();
    if (filter && !msg.symbol.toLowerCase().includes(filter)) return;
    const ts = new Date(msg.ts_ms).toISOString().slice(11, 23);
    const line = `${ts}  ${msg.symbol.padEnd(10)} @ ${Number(msg.price).toFixed(4).padStart(14)}  vol=${Number(msg.volume).toFixed(4)}`;
    pane.textContent += (pane.textContent ? '\n' : '') + line;
    // Trim to last TAPE_MAX_LINES
    const lines = pane.textContent.split('\n');
    if (lines.length > TAPE_MAX_LINES) {
        pane.textContent = lines.slice(-TAPE_MAX_LINES).join('\n');
    }
    const autoscroll = mount.querySelector('#algo-tape-autoscroll');
    if (autoscroll?.checked !== false) pane.scrollTop = pane.scrollHeight;
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
                <button class="link" data-act="metrics" data-i18n="view.algo.btn.metrics">metrics</button>
                <button class="link" data-act="backtest" data-i18n="view.algo.btn.backtest">backtest</button>
                <button class="link" data-act="optimize" data-i18n="view.algo.btn.optimize">optimize</button>
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
            if (act === 'metrics') return openMetricsModal(mount, s);
            if (act === 'backtest') return openBacktestModal(mount, s);
            if (act === 'optimize') return openOptimizeModal(mount, s);
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
        const reason = (await tPrompt('view.algo.prompt.kill_reason')) || null;
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

function fmtUsd(n) {
    if (!Number.isFinite(n)) return '—';
    return (n < 0 ? '−$' : '$') + Math.abs(n).toLocaleString(undefined, { maximumFractionDigits: 2 });
}

function fmtPct(n) {
    if (!Number.isFinite(n)) return '—';
    return n.toFixed(2) + '%';
}

// Curated default grids — keep combos under ~120 so a sweep finishes
// in a few seconds on a desktop. Users can edit the JSON before
// submitting if they want a tighter or wider sweep.
const OPTIMIZE_DEFAULT_GRIDS = {
    momentum: {
        ema_fast: [5, 9, 13],
        ema_slow: [21, 34, 50],
        rsi_min: [50, 55, 60],
    },
    mean_reversion: {
        connors_rsi_max: [10, 15, 20],
        vwap_z_min: [1.5, 2.0, 2.5],
    },
    orb: {
        or_minutes: [5, 15, 30],
        rvol_min: [1.5, 2.0, 3.0],
    },
    donchian_trend: {
        period: [20, 40, 55],
        adx_min: [20, 25, 30],
    },
    bb_squeeze: {
        period: [15, 20, 30],
        k: [1.5, 2.0, 2.5],
    },
    ttm_squeeze: {
        bb_period: [15, 20, 25],
        kc_period: [15, 20, 25],
    },
    vwap_scalp: {
        z_entry: [1.5, 2.0, 2.5],
        atr_stop_mult: [1.0, 1.5, 2.0],
    },
    supertrend: {
        atr_period: [7, 10, 14],
        multiplier: [2.0, 3.0, 4.0],
    },
    heikin_ashi_trend: {
        run_min: [3, 4, 5],
    },
    connors_rsi2: {
        rsi_period: [2, 3, 4],
        rsi_max: [5, 10, 15],
    },
    order_block_sweep: {
        lookback: [20, 30, 50],
    },
    pead: {
        eps_surprise_min: [0.05, 0.1, 0.15],
    },
    pairs: {
        lookback: [40, 60, 90],
        z_entry: [1.5, 2.0, 2.5],
    },
    ma_cross_adx: {
        fast_period: [5, 9, 13],
        slow_period: [21, 34, 50],
        adx_min: [20, 25, 30],
    },
    keltner_breakout: {
        period: [15, 20, 30],
        multiplier: [1.0, 1.5, 2.0],
    },
    ichimoku_cloud: {
        tenkan_period: [7, 9, 11],
        kijun_period: [22, 26, 30],
    },
};

async function openOptimizeModal(mount, s) {
    const defaultGrid = OPTIMIZE_DEFAULT_GRIDS[s.strategy_type] || { period: [10, 20, 30] };
    const gridText = JSON.stringify(defaultGrid, null, 2);
    const symbol = (s.entry_rules && s.entry_rules.symbol_a) || s.entry_rules?.symbol || 'SPY';
    const interval = s.timeframe || '5m';

    const wrap = document.createElement('div');
    wrap.className = 'modal';
    wrap.innerHTML = `
        <div class="modal-inner" style="max-width:1040px">
            <h2>Optimize: ${esc(s.name)}</h2>
            <p class="muted small">Sweeps the strategy's <code>entry_rules</code> across the grid below, runs the backtester for each combination, and ranks results by the chosen metric. Cap: 1024 combinations per sweep.</p>
            <form id="opt-form" class="algo-form">
                <label>Symbol
                    <input name="symbol" value="${esc(symbol)}" required>
                </label>
                <label>Interval
                    <select name="interval">
                        <option value="1m" ${interval === '1m' || interval === 'min1' ? 'selected' : ''}>1m</option>
                        <option value="5m" ${interval === '5m' || interval === 'min5' ? 'selected' : ''}>5m</option>
                        <option value="15m" ${interval === '15m' || interval === 'min15' ? 'selected' : ''}>15m</option>
                        <option value="1h" ${interval === '1h' || interval === 'hour1' ? 'selected' : ''}>1h</option>
                        <option value="1d" ${interval === '1d' || interval === 'day1' ? 'selected' : ''}>1d</option>
                    </select>
                </label>
                <label>Days back
                    <input name="days_back" type="number" value="60" min="5" max="730" step="1">
                </label>
                <label>Initial equity
                    <input name="initial_equity" type="number" value="100000" min="100" step="100">
                </label>
                <label>Metric
                    <select name="metric">
                        <option value="sharpe">Sharpe (bar)</option>
                        <option value="total_return">Total return %</option>
                        <option value="profit_factor">Profit factor</option>
                        <option value="avg_r">Avg R-multiple</option>
                        <option value="return_minus_dd">Return − max DD</option>
                    </select>
                </label>
                <label>Top N
                    <input name="top_n" type="number" value="10" min="1" max="50" step="1">
                </label>
                <label style="flex:1 1 100%">Grid (JSON, key → array of candidate values)
                    <textarea name="grid" rows="9" style="font-family:monospace;width:100%">${esc(gridText)}</textarea>
                </label>
                <div class="row" style="gap:8px;margin-top:8px">
                    <button type="submit" class="primary">Run sweep</button>
                    <button type="button" id="opt-close">Close</button>
                </div>
            </form>
            <div id="opt-results" style="margin-top:12px"></div>
        </div>`;
    document.body.appendChild(wrap);
    const close = () => wrap.remove();
    wrap.querySelector('#opt-close').addEventListener('click', close);
    wrap.addEventListener('click', e => { if (e.target === wrap) close(); });

    wrap.querySelector('#opt-form').addEventListener('submit', async (ev) => {
        ev.preventDefault();
        const fd = new FormData(ev.target);
        let grid;
        try {
            grid = JSON.parse(fd.get('grid'));
            if (!grid || typeof grid !== 'object' || Array.isArray(grid)) {
                throw new Error('grid must be a JSON object');
            }
        } catch (e) {
            const out = wrap.querySelector('#opt-results');
            out.innerHTML = `<p class="error">Bad grid JSON: ${esc(e.message || String(e))}</p>`;
            return;
        }
        const body = {
            symbol: fd.get('symbol'),
            interval: fd.get('interval'),
            days_back: Number(fd.get('days_back')),
            initial_equity: Number(fd.get('initial_equity')),
            metric: fd.get('metric'),
            top_n: Number(fd.get('top_n')),
            grid,
        };
        const out = wrap.querySelector('#opt-results');
        out.innerHTML = '<p class="muted">Sweeping … this can take a moment for large grids.</p>';
        try {
            const r = await api.optimizeAlgoStrategy(s.id, body);
            renderOptimizeResult(out, r, s);
        } catch (e) {
            out.innerHTML = `<p class="error">Optimize failed: ${esc(e.message || String(e))}</p>`;
        }
    });
}

function renderOptimizeResult(host, r, strategy) {
    const rows = r.top.map((row, i) => {
        const overrides = Object.entries(row.overrides)
            .map(([k, v]) => `<code>${esc(k)}=${esc(JSON.stringify(v))}</code>`)
            .join(' · ');
        return `
            <tr>
                <td>${i + 1}</td>
                <td>${row.metric_score.toFixed(4)}</td>
                <td>${row.summary.trades}</td>
                <td>${(row.summary.win_rate * 100).toFixed(1)}%</td>
                <td>${Number.isFinite(row.summary.profit_factor) ? row.summary.profit_factor.toFixed(2) : '∞'}</td>
                <td>${row.summary.total_return_pct.toFixed(2)}%</td>
                <td>${row.summary.max_drawdown_pct.toFixed(2)}%</td>
                <td>${row.summary.avg_r.toFixed(2)}</td>
                <td>${overrides}</td>
                <td><button class="link" data-apply="${esc(JSON.stringify(row.entry_rules))}">apply</button></td>
            </tr>`;
    }).join('');
    host.innerHTML = `
        <p class="muted small">Evaluated ${r.combinations_evaluated} combinations, ranked by <strong>${esc(r.metric)}</strong> (descending).</p>
        <table class="trades">
            <thead><tr><th>#</th><th>Score</th><th>Trades</th><th>Win %</th><th>PF</th><th>Return</th><th>Max DD</th><th>Avg R</th><th>Overrides</th><th></th></tr></thead>
            <tbody>${rows || '<tr><td colspan="10" class="muted">No combinations produced trades.</td></tr>'}</tbody>
        </table>`;
    host.querySelectorAll('button[data-apply]').forEach(btn => {
        btn.addEventListener('click', async () => {
            const newRules = JSON.parse(btn.dataset.apply);
            try {
                await api.updateAlgoStrategy(strategy.id, { ...strategy, entry_rules: newRules });
                showToast('entry_rules updated', { level: 'success' });
            } catch (e) {
                showToast(`update failed: ${e.message || String(e)}`, { level: 'error' });
            }
        });
    });
}

async function openMetricsModal(mount, s) {
    const wrap = document.createElement('div');
    wrap.className = 'modal';
    wrap.innerHTML = `
        <div class="modal-inner" style="max-width:960px">
            <h2>Metrics: ${esc(s.name)}</h2>
            <p class="muted small">Live aggregate over every algo_runs row for this strategy. Equity curve uses settled runs only (in-flight runs are excluded until stopped).</p>
            <div id="mt-body"><p class="muted">Loading…</p></div>
            <div class="row" style="gap:8px;margin-top:8px">
                <button type="button" id="mt-refresh">Refresh</button>
                <button type="button" id="mt-close">Close</button>
            </div>
        </div>`;
    document.body.appendChild(wrap);
    const close = () => wrap.remove();
    wrap.querySelector('#mt-close').addEventListener('click', close);
    wrap.addEventListener('click', e => { if (e.target === wrap) close(); });

    const body = wrap.querySelector('#mt-body');
    const load = async () => {
        body.innerHTML = '<p class="muted">Loading…</p>';
        try {
            const m = await api.algoStrategyMetrics(s.id);
            renderMetrics(body, m);
        } catch (e) {
            body.innerHTML = `<p class="error">Metrics failed: ${esc(e.message || String(e))}</p>`;
        }
    };
    wrap.querySelector('#mt-refresh').addEventListener('click', load);
    load();
}

function renderMetrics(host, m) {
    const totalPnl = Number(m.total_realized_pnl || 0);
    const fillRate = m.orders_submitted > 0
        ? (m.fills_received / m.orders_submitted * 100)
        : 0;
    const rejectRate = (m.orders_submitted + m.orders_rejected) > 0
        ? (m.orders_rejected / (m.orders_submitted + m.orders_rejected) * 100)
        : 0;
    const lastOrders = m.recent_orders.slice(0, 25);

    // Equity curve as SVG sparkline — no chart-lib dep.
    const eq = m.equity_curve;
    let curve = '<p class="muted">No settled runs yet — start a run and let it complete to see the equity curve.</p>';
    if (eq.length >= 2) {
        const vals = eq.map(p => Number(p.cumulative_pnl));
        const min = Math.min(0, ...vals);
        const max = Math.max(0, ...vals);
        const range = (max - min) || 1;
        const w = 760, h = 160, pad = 8;
        const xs = (i) => pad + i * ((w - 2 * pad) / (eq.length - 1));
        const ys = (v) => h - pad - ((v - min) / range) * (h - 2 * pad);
        const points = vals.map((v, i) => `${xs(i).toFixed(1)},${ys(v).toFixed(1)}`).join(' ');
        const zeroY = ys(0).toFixed(1);
        curve = `
            <svg viewBox="0 0 ${w} ${h}" width="100%" style="max-width:100%;background:#0a0a0a;border:1px solid #222">
                <line x1="${pad}" x2="${w - pad}" y1="${zeroY}" y2="${zeroY}" stroke="#444" stroke-dasharray="3,3"/>
                <polyline fill="none" stroke="${totalPnl >= 0 ? '#39ff14' : '#ff5a5a'}" stroke-width="1.6" points="${points}"/>
            </svg>`;
    }

    host.innerHTML = `
        <div class="row" style="gap:24px;flex-wrap:wrap;margin-bottom:12px">
            <div><strong>Total runs:</strong> ${m.runs}</div>
            <div><strong>Bars processed:</strong> ${m.bars_processed.toLocaleString()}</div>
            <div><strong>Signals:</strong> ${m.signals_emitted}</div>
            <div><strong>Orders submitted:</strong> ${m.orders_submitted}</div>
            <div><strong>Fills:</strong> ${m.fills_received}</div>
            <div><strong>Fill rate:</strong> ${fillRate.toFixed(1)}%</div>
            <div><strong>Rejected:</strong> ${m.orders_rejected} (${rejectRate.toFixed(1)}%)</div>
            <div style="color:${totalPnl >= 0 ? '#39ff14' : '#ff5a5a'}"><strong>Total realized P&amp;L:</strong> ${fmtUsd(totalPnl)}</div>
        </div>
        <h3 style="margin-top:4px">Equity curve (settled runs)</h3>
        ${curve}
        <h3 style="margin-top:12px">Recent orders (last ${lastOrders.length})</h3>
        <table class="trades">
            <thead><tr><th>Submitted</th><th>Symbol</th><th>Side</th><th>Type</th><th>Qty</th><th>Status</th><th>Broker order</th></tr></thead>
            <tbody>${lastOrders.map(o => `
                <tr>
                    <td>${esc((o.submitted_at || '').replace('T', ' ').slice(0, 19))}</td>
                    <td>${esc(o.symbol)}</td>
                    <td>${esc(o.side)}</td>
                    <td>${esc(o.order_type)}</td>
                    <td>${o.qty}</td>
                    <td>${esc(o.status)}</td>
                    <td><code class="small">${esc(o.broker_order_id || '—')}</code></td>
                </tr>`).join('') || '<tr><td colspan="7" class="muted">No orders yet.</td></tr>'}
            </tbody>
        </table>`;
}

async function openBacktestModal(mount, s) {
    const symbol = (s.entry_rules && s.entry_rules.symbol_a) || s.entry_rules?.symbol || 'SPY';
    const interval = s.timeframe || '5m';
    const wrap = document.createElement('div');
    wrap.className = 'modal';
    wrap.innerHTML = `
        <div class="modal-inner" style="max-width:920px">
            <h2>Backtest: ${esc(s.name)}</h2>
            <p class="muted small">Replays this strategy through cached historical bars using the same Sizing config the live engine would. Fills land at the next bar's open + slippage; SL/TP intra-bar resolution is pessimistic (SL wins ties).</p>
            <form id="bt-form" class="algo-form">
                <label>Symbol
                    <input name="symbol" value="${esc(symbol)}" required>
                </label>
                <label>Interval
                    <select name="interval">
                        <option value="1m" ${interval === '1m' || interval === 'min1' ? 'selected' : ''}>1m</option>
                        <option value="5m" ${interval === '5m' || interval === 'min5' ? 'selected' : ''}>5m</option>
                        <option value="15m" ${interval === '15m' || interval === 'min15' ? 'selected' : ''}>15m</option>
                        <option value="1h" ${interval === '1h' || interval === 'hour1' ? 'selected' : ''}>1h</option>
                        <option value="1d" ${interval === '1d' || interval === 'day1' ? 'selected' : ''}>1d</option>
                    </select>
                </label>
                <label>Days back
                    <input name="days_back" type="number" value="60" min="5" max="730" step="1">
                </label>
                <label>Initial equity
                    <input name="initial_equity" type="number" value="100000" min="100" step="100">
                </label>
                <label>Fee / trade
                    <input name="fee_per_trade" type="number" value="1" min="0" step="0.1">
                </label>
                <label>Slippage (bps)
                    <input name="slippage_bps" type="number" value="5" min="0" step="0.5">
                </label>
                <div class="row" style="gap:8px;margin-top:8px">
                    <button type="submit" class="primary">Run backtest</button>
                    <button type="button" id="bt-close">Close</button>
                </div>
            </form>
            <div id="bt-results" style="margin-top:12px"></div>
            <h3 style="margin-top:12px">History</h3>
            <div id="bt-history" class="muted small">Loading past runs…</div>
        </div>`;
    document.body.appendChild(wrap);
    const close = () => wrap.remove();
    wrap.querySelector('#bt-close').addEventListener('click', close);
    wrap.addEventListener('click', e => { if (e.target === wrap) close(); });

    const refreshHistory = async () => {
        const host = wrap.querySelector('#bt-history');
        try {
            const rows = await api.listAlgoBacktests(s.id, 25);
            renderBacktestHistory(host, rows, refreshHistory);
        } catch (e) {
            host.innerHTML = `<p class="error">History fetch failed: ${esc(e.message || String(e))}</p>`;
        }
    };

    wrap.querySelector('#bt-form').addEventListener('submit', async (ev) => {
        ev.preventDefault();
        const fd = new FormData(ev.target);
        const body = {
            symbol: fd.get('symbol'),
            interval: fd.get('interval'),
            days_back: Number(fd.get('days_back')),
            initial_equity: Number(fd.get('initial_equity')),
            fee_per_trade: Number(fd.get('fee_per_trade')),
            slippage_bps: Number(fd.get('slippage_bps')),
        };
        const out = wrap.querySelector('#bt-results');
        out.innerHTML = '<p class="muted">Running backtest …</p>';
        try {
            const r = await api.backtestAlgoStrategy(s.id, body);
            renderBacktestResult(out, r);
            await refreshHistory();
        } catch (e) {
            out.innerHTML = `<p class="error">Backtest failed: ${esc(e.message || String(e))}</p>`;
        }
    });
    refreshHistory();
}

function renderBacktestHistory(host, rows, onChange) {
    if (!rows.length) {
        host.innerHTML = '<p class="muted small">No past backtests for this strategy yet.</p>';
        return;
    }
    const fmtTime = (s) => esc(String(s).replace('T', ' ').slice(0, 16));
    host.innerHTML = `
        <table class="trades small">
            <thead><tr>
                <th>When</th><th>Symbol</th><th>Interval</th><th>Trades</th>
                <th>Win %</th><th>PF</th><th>Return</th><th>Max DD</th>
                <th>Sharpe</th><th></th>
            </tr></thead>
            <tbody>${rows.map(r => `
                <tr>
                    <td>${fmtTime(r.created_at)}</td>
                    <td>${esc(r.symbol)}</td>
                    <td>${esc(r.interval)}</td>
                    <td>${r.trades}</td>
                    <td>${(Number(r.win_rate) * 100).toFixed(1)}%</td>
                    <td>${Number(r.profit_factor).toFixed(2)}</td>
                    <td style="color:${Number(r.total_return_pct) >= 0 ? '#39ff14' : '#ff5a5a'}">${Number(r.total_return_pct).toFixed(2)}%</td>
                    <td>${Number(r.max_drawdown_pct).toFixed(2)}%</td>
                    <td>${Number(r.sharpe).toFixed(3)}</td>
                    <td><button class="link" data-del="${esc(r.id)}">×</button></td>
                </tr>`).join('')}
            </tbody>
        </table>`;
    host.querySelectorAll('button[data-del]').forEach(btn => {
        btn.addEventListener('click', async () => {
            try {
                await api.deleteAlgoBacktest(btn.dataset.del);
                if (onChange) await onChange();
            } catch (e) {
                showToast(`delete failed: ${e.message || String(e)}`, { level: 'error' });
            }
        });
    });
}

function renderBacktestResult(host, r) {
    const sm = r.summary;
    const finalRow = (() => {
        if (!r.equity || !r.equity.length) return '—';
        return fmtUsd(sm.final_equity);
    })();
    const trades = r.trades.slice(-30).reverse();
    host.innerHTML = `
        <div class="row" style="gap:24px;flex-wrap:wrap;margin-bottom:12px">
            <div><strong>Trades:</strong> ${sm.trades}</div>
            <div><strong>Win rate:</strong> ${fmtPct(sm.win_rate * 100)}</div>
            <div><strong>Profit factor:</strong> ${Number.isFinite(sm.profit_factor) ? sm.profit_factor.toFixed(2) : '∞'}</div>
            <div><strong>Total return:</strong> ${fmtPct(sm.total_return_pct)}</div>
            <div><strong>Max DD:</strong> ${fmtPct(sm.max_drawdown_pct)}</div>
            <div><strong>Avg R:</strong> ${sm.avg_r.toFixed(2)}</div>
            <div><strong>Sharpe (bar):</strong> ${sm.sharpe.toFixed(3)}</div>
            <div><strong>Final equity:</strong> ${finalRow}</div>
            <div><strong>Exits:</strong> SL ${sm.exits_by_stop} / TP ${sm.exits_by_tp} / Sig ${sm.exits_by_signal} / EOD ${sm.exits_by_eod}</div>
        </div>
        <details ${trades.length ? 'open' : ''}>
            <summary>Last ${trades.length} trades</summary>
            <table class="trades" style="margin-top:8px">
                <thead><tr><th>Entry</th><th>Side</th><th>Qty</th><th>Entry $</th><th>Exit $</th><th>PnL</th><th>R</th><th>Bars</th><th>Reason</th></tr></thead>
                <tbody>${trades.map(t => `
                    <tr>
                        <td>${esc(t.entry_time.replace('T', ' ').slice(0, 16))}</td>
                        <td>${t.side === 'Buy' ? 'long' : 'short'}</td>
                        <td>${t.qty}</td>
                        <td>${t.entry_price.toFixed(2)}</td>
                        <td>${t.exit_price.toFixed(2)}</td>
                        <td style="color:${t.pnl >= 0 ? '#39ff14' : '#ff5a5a'}">${fmtUsd(t.pnl)}</td>
                        <td>${t.r_multiple.toFixed(2)}</td>
                        <td>${t.bars_held}</td>
                        <td>${esc(t.exit_reason)}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
        </details>`;
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
    const nameSlot = mount.querySelector('#algo-runs-strategy-name');
    if (nameSlot) {
        nameSlot.textContent = `${s.name} (${s.strategy_type}, ${s.timeframe})`;
        nameSlot.classList.remove('muted');
    }
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
        ['alpaca', 'tradier', 'ibkr', 'td', 'tdameritrade', 'schwab', 'tastytrade']
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
                    <label><span data-i18n="view.algo.label.asset_class">Asset class</span>
                        <select name="asset_class">
                            <option value="equity" ${(s.entry_rules?.asset_class || 'equity') === 'equity' ? 'selected' : ''} data-i18n="view.algo.opt.asset_equity">Equity (US RTH + extended hours)</option>
                            <option value="crypto" ${s.entry_rules?.asset_class === 'crypto' ? 'selected' : ''} data-i18n="view.algo.opt.asset_crypto">Crypto (24/7, BASE/USD universe)</option>
                        </select>
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
            entry_rules: {
                ...(s.entry_rules || {}),
                // Asset class selector — merged into entry_rules so
                // the backend's AssetClass::from_entry_rules picks it
                // up. Default "equity" preserves prior behaviour.
                asset_class: f.get('asset_class') || 'equity',
            },
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
