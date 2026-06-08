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

// Bounded ring buffer for live stdout — keeps the last N events so a
// long-running session doesn't grow the DOM unbounded. 500 events at
// ~80 chars each ≈ 40KB of text, well inside what a <pre> renders fast.
const STDOUT_MAX_LINES = 500;
const stdoutBuffer = [];
let stdoutNameMap = new Map(); // strategy_id → display name (filled on refresh)

function fmtStdoutTs(d = new Date()) {
    return d.toISOString().slice(11, 23); // HH:MM:SS.mmm
}

function appendStdout(mount, line) {
    stdoutBuffer.push(line);
    if (stdoutBuffer.length > STDOUT_MAX_LINES) stdoutBuffer.shift();
    renderStdout(mount);
}

function renderStdout(mount) {
    const pre = mount.querySelector('#algo-stdout');
    if (!pre) return;
    const filterEl = mount.querySelector('#algo-stdout-filter');
    const autoEl = mount.querySelector('#algo-autoscroll');
    const filter = (filterEl?.value || '').trim().toLowerCase();
    const lines = filter
        ? stdoutBuffer.filter(l => l.toLowerCase().includes(filter))
        : stdoutBuffer;
    pre.textContent = lines.join('\n');
    if (autoEl?.checked) pre.scrollTop = pre.scrollHeight;
}

function strategyLabel(id) {
    const name = stdoutNameMap.get(id);
    return name ? `${name} (${id.slice(0, 8)})` : id.slice(0, 8);
}

function logSignal(mount, msg) {
    appendStdout(mount, `${fmtStdoutTs()} [${strategyLabel(msg.strategy_id)}] SIGNAL ${msg.side.toUpperCase()} ${msg.symbol} @ ${Number(msg.entry_price).toFixed(2)} (${msg.kind})`);
}
function logOrder(mount, msg) {
    appendStdout(mount, `${fmtStdoutTs()} [${strategyLabel(msg.strategy_id)}] ORDER ${msg.side.toUpperCase()} ${msg.symbol} qty=${msg.qty} broker=${msg.broker_order_id.slice(0, 12)}`);
}
function logFill(mount, msg) {
    appendStdout(mount, `${fmtStdoutTs()} [${strategyLabel(msg.strategy_id)}] FILL ${msg.symbol} qty=${msg.qty} @ ${Number(msg.price).toFixed(4)}`);
}

function fmtDateTime(iso) {
    if (!iso) return '—';
    try { return new Date(iso).toLocaleString(); } catch (_) { return iso; }
}

function brokerBadge(mode) {
    if (mode === 'alpaca_live') return '<span class="badge badge-danger">live</span>';
    if (mode === 'alpaca_paper') return '<span class="badge">paper</span>';
    return '<span class="badge muted">sim</span>';
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
];

const STRATEGY_HINTS = {
    momentum:          'EMA(9)/EMA(21) crossover + RSI(14) ∈ [50,70] + ROC(10) > 2% + RVOL ≥ 1.5×. ATR-based bracket. Trend follower.',
    mean_reversion:    'Connors RSI(3) < 10 + close < session VWAP − 2σ. Long the oversold pierce, target = VWAP. ATR stop.',
    orb:               'First close that breaks the opening-range high (15 bars default) with RVOL ≥ 1.5×. ATR trailing stop. Day-trade setup.',
    donchian_trend:    'Close > Donchian(20).upper + ADX(14) > 20 (chop filter). Exit on Donchian(10).low break or ATR trail. Long trends.',
    bb_squeeze:        'BBW(20,2) in the bottom 10th percentile over 100 bars + close breaks BB.upper. Target = BB.middle. Volatility expansion.',
    ttm_squeeze:       'BB inside KC = squeeze coil. After release (within 5 bars) + momentum histogram positive and accelerating = long entry.',
    vwap_scalp:        'Pure z-score reversion: close ≤ session VWAP − 2σ + recovery tick. Tight 1×ATR stop, target = VWAP. Intraday scalp.',
    supertrend:        'ATR(10)×3 banded reversal. Entry on trend flip (−1 → +1), exit on opposite flip. Simple trend follower.',
    heikin_ashi_trend: '3 consecutive green HA candles + close > EMA(21). Noise-filtered trend follower; use on 5m+ bars for best signal-to-noise.',
};

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

        <div class="chart-panel" id="algo-stdout-panel">
            <div class="row" style="justify-content:space-between;align-items:center">
                <h2 data-i18n="view.algo.h2.stdout">Live stdout</h2>
                <div class="row" style="gap:8px">
                    <label class="row small" style="gap:4px;align-items:center">
                        <input type="checkbox" id="algo-autoscroll" checked>
                        <span data-i18n="view.algo.label.autoscroll">auto-scroll</span>
                    </label>
                    <input type="text" id="algo-stdout-filter" placeholder="filter by strategy id…"
                           data-i18n-placeholder="view.algo.placeholder.stdout_filter" style="min-width:200px">
                    <button id="algo-stdout-clear" class="link" data-i18n="view.algo.btn.clear">clear</button>
                </div>
            </div>
            <pre id="algo-stdout" class="algo-stdout"></pre>
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
    mount.querySelector('#algo-stdout-clear').addEventListener('click', () => {
        stdoutBuffer.length = 0;
        renderStdout(mount);
    });
    mount.querySelector('#algo-stdout-filter').addEventListener('input', () => renderStdout(mount));
    await refreshStrategies(mount);
    renderStdout(mount);

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
    const accountOptions = accounts.length
        ? accounts.map(a => {
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
                    ${accounts.length ? '' : `<p class="muted small" style="margin:-4px 0 0;color:var(--red)">
                        ${esc(t('view.algo.hint.no_accounts'))}
                    </p>`}
                    <label><span data-i18n="view.algo.label.strategy_type">Strategy</span>
                        <select name="strategy_type" id="algo-strategy-type">
                            ${stratOptions}
                        </select>
                    </label>
                    <p class="muted small" id="algo-strategy-hint" style="margin:0">
                        ${esc(STRATEGY_HINTS[s.strategy_type || 'momentum'])}
                    </p>
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
                    <label><span data-i18n="view.algo.label.broker_mode">Broker</span>
                        <select name="broker_mode">
                            <option value="internal_sim"  ${s.broker_mode === 'internal_sim'  ? 'selected' : ''} data-i18n="view.algo.opt.broker_sim">Internal simulator</option>
                            <option value="alpaca_paper" ${s.broker_mode === 'alpaca_paper' ? 'selected' : ''} data-i18n="view.algo.opt.broker_paper">Alpaca paper</option>
                            <option value="alpaca_live"  ${s.broker_mode === 'alpaca_live'  ? 'selected' : ''} data-i18n="view.algo.opt.broker_live">Alpaca LIVE (after paper-lock)</option>
                        </select>
                    </label>
                    <div class="algo-form-actions">
                        <button type="button" id="algo-cancel" data-i18n="view.algo.btn.cancel">Cancel</button>
                        <button type="submit" id="algo-save" class="primary"
                                ${accounts.length ? '' : 'disabled'}
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
        host.querySelector('#algo-strategy-hint').textContent = STRATEGY_HINTS[v] || '';
    });
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
