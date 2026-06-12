// Live Feed — raw firehose of every WebSocket event the backend emits.
// Subscribes to every Hub event type (tick / news / sentiment / disclosure
// / squeeze_fired / alert_fired / algo_* / ping) and renders them in a
// time-ordered scrolling log. The unaggregated tick stream covers Alpaca
// IEX / SIP, Polygon, Finnhub WS as they arrive — proves the upstream
// feeds are live, not synthesized.
//
// UI: per-type filter checkboxes, free-text grep, pause toggle, clear,
// auto-scroll. Capped ring buffer keeps the view responsive even when
// the tape lights up.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { on as onWsEvent, isConnected } from '../ws.js';

// Event types the backend publishes (mirror of crates/traderview-web/src/realtime.rs::Event).
// Order here is the order they show in the filter row.
const EVENT_TYPES = [
    'tick',
    'news',
    'sentiment',
    'disclosure',
    'squeeze_fired',
    'alert_fired',
    'paper_fill',
    'strategy_drift',
    'rebalance_drift',
    'daily_digest',
    'algo_signal_fired',
    'algo_order_submitted',
    'algo_fill_received',
    'algo_bar_evaluated',
    'algo_tick_skipped',
    'algo_heartbeat',
    'ping',
];

// Per-type color class (matches existing CSS pos/neg/muted/accent palette).
const TYPE_COLOR = {
    tick:                'lf-tick',
    news:                'lf-news',
    sentiment:           'lf-sentiment',
    disclosure:          'lf-disclosure',
    squeeze_fired:       'lf-squeeze',
    alert_fired:         'lf-alert',
    paper_fill:          'lf-fill',
    strategy_drift:      'lf-alert',
    rebalance_drift:     'lf-alert',
    daily_digest:        'lf-news',
    algo_signal_fired:   'lf-signal',
    algo_order_submitted:'lf-order',
    algo_fill_received:  'lf-fill',
    algo_bar_evaluated:  'lf-bar',
    algo_tick_skipped:   'lf-skip',
    algo_heartbeat:      'lf-heartbeat',
    ping:                'lf-ping',
};

// Bigger than the visible row count so the user can scroll back; small
// enough that 1000+ events/sec doesn't eat the heap.
const BUFFER_CAP = 800;

const STATE = {
    enabled: new Set(EVENT_TYPES),
    grep: '',
    paused: false,
    autoscroll: true,
    buf: [],         // [{ ts, type, data }, …]  newest at end
    receivedTotal: 0,
    perType: Object.fromEntries(EVENT_TYPES.map(t => [t, 0])),
};

let unsubs = [];

export async function renderLiveFeed(mount, _state) {
    const checkboxes = EVENT_TYPES.map(type => `
        <label class="lf-chip ${TYPE_COLOR[type]}">
            <input type="checkbox" data-type="${type}" ${STATE.enabled.has(type) ? 'checked' : ''}>
            <span>${esc(type)}</span>
            <span class="muted small" data-type-count="${type}">0</span>
        </label>
    `).join('');

    mount.innerHTML = `
        <h1 class="view-title">
            <span data-i18n="view.live_feed.title">// LIVE FEED · WS FIREHOSE</span>
            <span class="status-dot" id="lf-status" title="ws status">●</span>
        </h1>
        <p class="muted small" data-i18n="view.live_feed.intro">
            Every event the realtime hub publishes — raw ticks (Alpaca IEX/SIP, Polygon, Finnhub),
            news poller batches, sentiment ticks, disclosures, squeeze/alert fires, algo
            engine signals/orders/fills/heartbeats. Capped ring buffer (${BUFFER_CAP}).
            Subscribe to a watchlist symbol or configure a provider in Settings → Data Sources
            to populate the stream.
        </p>

        <div class="chart-panel">
            <div class="lf-toolbar" style="display:flex;gap:8px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <input id="lf-grep" type="text" placeholder="grep — symbol, message, …"
                       data-i18n-placeholder="view.live_feed.placeholder.grep"
                       style="min-width:240px;flex:1">
                <button class="btn btn-secondary" id="lf-pause"
                        data-tip="view.live_feed.tip.pause"
                        data-i18n="${STATE.paused ? 'view.live_feed.btn.resume' : 'view.live_feed.btn.pause'}">
                    ${STATE.paused ? '▶ Resume' : '⏸ Pause'}
                </button>
                <button class="btn btn-secondary" id="lf-clear"
                        data-tip="view.live_feed.tip.clear"
                        data-i18n="view.live_feed.btn.clear">⌫ Clear</button>
                <label style="display:flex;align-items:center;gap:4px">
                    <input type="checkbox" id="lf-autoscroll" ${STATE.autoscroll ? 'checked' : ''}>
                    <span class="muted small" data-i18n="view.live_feed.label.autoscroll">auto-scroll</span>
                </label>
                <span class="muted small" id="lf-meta" style="margin-left:auto"></span>
            </div>
            <div class="lf-types" style="display:flex;gap:6px;flex-wrap:wrap;margin-bottom:8px">
                ${checkboxes}
            </div>
            <div id="lf-out" class="lf-out"></div>
        </div>
    `;

    paintStatus(mount);

    // Wire toolbar
    const grepEl = mount.querySelector('#lf-grep');
    grepEl.addEventListener('input', () => {
        STATE.grep = grepEl.value.trim().toLowerCase();
        paintFeed(mount);
    });
    const pauseBtn = mount.querySelector('#lf-pause');
    pauseBtn.addEventListener('click', () => {
        STATE.paused = !STATE.paused;
        pauseBtn.textContent = STATE.paused ? '▶ Resume' : '⏸ Pause';
        paintMeta(mount);
    });
    mount.querySelector('#lf-clear').addEventListener('click', () => {
        STATE.buf = [];
        for (const k of EVENT_TYPES) STATE.perType[k] = 0;
        STATE.receivedTotal = 0;
        paintFeed(mount);
        paintTypeCounts(mount);
        paintMeta(mount);
    });
    const autoEl = mount.querySelector('#lf-autoscroll');
    autoEl.addEventListener('change', () => { STATE.autoscroll = autoEl.checked; });
    mount.querySelectorAll('[data-type]').forEach(cb => {
        cb.addEventListener('change', () => {
            const type = cb.dataset.type;
            if (cb.checked) STATE.enabled.add(type);
            else STATE.enabled.delete(type);
            paintFeed(mount);
        });
    });

    // Subscribe to every event type. Each callback pushes to the ring
    // buffer + repaints (when not paused).
    teardownPrior();
    for (const type of EVENT_TYPES) {
        const u = onWsEvent(type, (msg) => {
            STATE.receivedTotal++;
            STATE.perType[type] = (STATE.perType[type] || 0) + 1;
            STATE.buf.push({ ts: Date.now(), type, data: msg });
            if (STATE.buf.length > BUFFER_CAP) STATE.buf.shift();
            if (!STATE.paused) {
                paintFeed(mount);
                paintTypeCounts(mount);
                paintMeta(mount);
            }
        });
        if (typeof u === 'function') unsubs.push(u);
    }

    paintFeed(mount);
    paintTypeCounts(mount);
    paintMeta(mount);

    // Teardown for dashboards-as-tile callers — unhook every subscription
    // when the tile remounts so we don't pile up listeners.
    return () => teardownPrior();
}

function teardownPrior() {
    for (const u of unsubs) {
        try { u(); } catch (_) {}
    }
    unsubs = [];
}

function paintStatus(mount) {
    const dot = mount.querySelector('#lf-status');
    if (!dot) return;
    if (isConnected()) {
        dot.style.color = 'var(--green)';
        dot.title = 'connected';
    } else {
        dot.style.color = 'var(--text-muted)';
        dot.title = 'reconnecting…';
    }
}

function paintMeta(mount) {
    const meta = mount.querySelector('#lf-meta');
    if (!meta) return;
    const shown = STATE.buf.filter(passesFilter).length;
    meta.textContent = `${shown.toLocaleString()} shown · ${STATE.buf.length.toLocaleString()} buffered · ${STATE.receivedTotal.toLocaleString()} received total${STATE.paused ? ' · PAUSED' : ''}`;
}

function paintTypeCounts(mount) {
    EVENT_TYPES.forEach(type => {
        const el = mount.querySelector(`[data-type-count="${type}"]`);
        if (el) el.textContent = (STATE.perType[type] || 0).toLocaleString();
    });
}

function passesFilter(entry) {
    if (!STATE.enabled.has(entry.type)) return false;
    if (!STATE.grep) return true;
    const payload = `${entry.type} ${JSON.stringify(entry.data)}`.toLowerCase();
    return payload.includes(STATE.grep);
}

function paintFeed(mount) {
    const out = mount.querySelector('#lf-out');
    if (!out) return;
    const items = STATE.buf.filter(passesFilter);
    // Render newest at TOP so the user reads top-to-bottom without
    // scrolling. Auto-scroll keeps the latest visible.
    out.innerHTML = items.slice().reverse().map(renderRow).join('');
    if (STATE.autoscroll) out.scrollTop = 0;
}

function renderRow(entry) {
    const cls = TYPE_COLOR[entry.type] || '';
    const ts = new Date(entry.ts).toISOString().slice(11, 23);
    const summary = summarize(entry.type, entry.data);
    return `<div class="lf-row ${cls}">
        <span class="lf-ts muted small">${esc(ts)}</span>
        <span class="lf-type">${esc(entry.type)}</span>
        <span class="lf-body">${summary}</span>
    </div>`;
}

// Per-type one-liner — pulls the high-signal fields out of each event
// shape so the row is scannable without expanding JSON.
function summarize(type, d) {
    if (!d || typeof d !== 'object') return esc(String(d));
    switch (type) {
        case 'tick':
            return `<strong>${esc(d.symbol)}</strong> @ <strong>${num(d.price, 4)}</strong> vol=${num(d.volume, 6)}`;
        case 'news':
            return `${num(d.inserted, 0)} stories across ${num(d.symbols, 0)} symbols`;
        case 'sentiment':
            return `wsb=${num(d.wsb, 0)} stocktwits=${num(d.stocktwits, 0)}`;
        case 'disclosure':
            return `${esc(d.source || '?')} +${num(d.inserted, 0)}`;
        case 'squeeze_fired':
            return `<strong>${esc(d.symbol)}</strong> @ ${num(d.price, 2)} pct=${num(d.pct_change, 2)}% burst=${num(d.burst_ratio, 1)}×`;
        case 'alert_fired':
            return `<strong>${esc(d.symbol)}</strong> rule=${esc(d.rule_id)} — ${esc(d.message)}`;
        case 'paper_fill':
            return `<strong>${esc(d.symbol)}</strong> ${esc(d.side)} ${num(d.qty, 2)} @ ${num(d.price, 4)} (${esc(d.order_type)} filled in background)`;
        case 'strategy_drift':
            return `<strong>${esc(d.name)}</strong> ${esc(d.verdict)} · z=${d.win_rate_z != null ? num(d.win_rate_z, 2) : '—'} over ${num(d.live_trades, 0)} live trades — live record diverging from backtest`;
        case 'rebalance_drift':
            return `<strong>${esc(d.name)}</strong> max drift ${num(d.max_drift_pct, 1)}% > ${num(d.threshold_pct, 1)}% tolerance — portfolio needs rebalancing`;
        case 'daily_digest':
            return `<strong>digest</strong> ${esc(d.summary)}`;
        case 'algo_signal_fired':
            return `<strong>${esc(d.symbol)}</strong> ${esc(d.side)} @ ${num(d.entry_price, 4)} (${esc(d.kind)}) strat=${esc(d.strategy_id?.slice(0, 8))}`;
        case 'algo_order_submitted':
            return `<strong>${esc(d.symbol)}</strong> ${esc(d.side)} ${num(d.qty, 2)} order=${esc(d.order_id?.slice(0, 8))} broker=${esc(d.broker_order_id?.slice(0, 10))}`;
        case 'algo_fill_received':
            return `<strong>${esc(d.symbol)}</strong> ${num(d.qty, 2)} @ ${num(d.price, 4)} order=${esc(d.order_id?.slice(0, 8))}`;
        case 'algo_bar_evaluated':
            return `<strong>${esc(d.symbol)}</strong> bars=${num(d.bars, 0)} strat=${esc(d.strategy_id?.slice(0, 8))}`;
        case 'algo_tick_skipped':
            return `strat=${esc(d.strategy_id?.slice(0, 8))} reason=${esc(d.reason)}`;
        case 'algo_heartbeat':
            return `strat=${esc(d.strategy_id?.slice(0, 8))} universe=${num(d.universe_size, 0)} live=${num(d.subscribed_live, 0)} bars=${num(d.bars_processed, 0)} sigs=${num(d.signals_emitted, 0)}`;
        case 'ping':
            return `ts=${num(d.ts, 0)}`;
        default:
            return esc(JSON.stringify(d).slice(0, 240));
    }
}

function num(v, d) {
    if (v == null || !Number.isFinite(Number(v))) return '—';
    return Number(v).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
