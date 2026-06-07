// Portfolio rebalancing tool — target weights, current-vs-target diff,
// exact trade list with CSV download for manual broker entry.

import { api, apiFetchBlob } from '../api.js';
import { esc, fmt } from '../util.js';
import { t, applyUiI18n } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const PRESETS = [
    { id: '60_40', weights: [
        { symbol: 'VTI', weight: 0.60 },
        { symbol: 'BND', weight: 0.40 },
    ]},
    { id: 'three_fund_bogle', weights: [
        { symbol: 'VTI', weight: 0.60 },
        { symbol: 'VXUS', weight: 0.20 },
        { symbol: 'BND',  weight: 0.20 },
    ]},
    { id: 'permanent_browne', weights: [
        { symbol: 'VTI', weight: 0.25 },
        { symbol: 'TLT', weight: 0.25 },
        { symbol: 'GLD', weight: 0.25 },
        { symbol: 'BIL', weight: 0.25 },
    ]},
    { id: 'risk_parity_lite', weights: [
        { symbol: 'VTI', weight: 0.30 },
        { symbol: 'TLT', weight: 0.40 },
        { symbol: 'GLD', weight: 0.15 },
        { symbol: 'IEF', weight: 0.15 },
    ]},
];

export async function renderRebalance(mount, state) {
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) { mount.innerHTML = `<p data-i18n="view.rebalance.hint.no_account_selected" class="boot">No account selected.</p>`; return; }
    mount.innerHTML = `
        <h1 class="view-title">// REBALANCE — ${esc(acct.broker)} · ${esc(acct.name)}</h1>
        <p data-i18n="view.rebalance.hint.set_target_weights_per_symbol_the_engine_snapshots" class="muted small">Set target weights per symbol; the engine snapshots your open
            positions, fetches fresh quotes, and computes the exact whole-share trade list
            to drift back to target. Max-trades cap keeps a rebalance from blowing into 30+
            orders. Export trades as CSV for manual broker entry — TraderView never sends orders.</p>

        <div class="chart-panel">
            <form id="rb-form" class="inline-form">
                <label><span data-i18n="view.rebalance.label.preset">Preset</span>
                    <select name="preset" data-tip="view.rebalance.tip.preset">
                        <option data-i18n="view.rebalance.opt.custom" value="">(custom)</option>
                        ${PRESETS.map(p => `<option value="${p.id}" data-i18n="view.rebalance.preset.${p.id}">${esc(t(`view.rebalance.preset.${p.id}`))}</option>`).join('')}
                    </select>
                </label>
                <label><span data-i18n="view.rebalance.label.cash">Cash on hand</span>
                    <input name="cash" type="number" min="0" step="0.01" value="0" style="width:120px;" data-tip="view.rebalance.tip.cash">
                </label>
                <label><span data-i18n="view.rebalance.label.max_trades">Max trades</span>
                    <input name="max_trades" type="number" min="1" max="200" value="20" style="width:80px;" data-tip="view.rebalance.tip.max_trades">
                </label>
                <button data-i18n="view.rebalance.btn.compute_plan" data-tip="view.rebalance.tip.compute_plan" data-shortcut="rebalance_compute" class="primary" id="rb-go" type="button">Compute plan</button>
                <button data-i18n="view.rebalance.btn.download_trades_csv" data-tip="view.rebalance.tip.download_csv" class="btn" id="rb-csv" type="button">Download trades CSV</button>
                <span id="rb-status" class="muted small"></span>
            </form>
            <textarea id="rb-targets" rows="8"
                style="width:100%;font-family:'Share Tech Mono',monospace;font-size:11px;background:#070714;color:#cfd2e8;border:1px solid var(--border);padding:8px;margin-top:8px;"
                data-i18n-placeholder="view.rebalance.placeholder.targets"
                data-tip="view.rebalance.tip.targets"
                data-shortcut="rebalance_focus_targets"
                placeholder='Targets JSON: [{"symbol":"SPY","weight":0.6},{"symbol":"BND","weight":0.4}]'>[
  {"symbol":"SPY","weight":0.6},
  {"symbol":"BND","weight":0.4}
]</textarea>
        </div>

        <div id="rb-out"></div>
    `;

    mount.querySelector('#rb-form [name=preset]').addEventListener('change', (e) => {
        const preset = PRESETS.find(p => p.id === e.target.value);
        if (preset) {
            const ta = mount.querySelector('#rb-targets');
            if (ta) ta.value = JSON.stringify(preset.weights, null, 2);
        }
    });

    mount.querySelector('#rb-go').addEventListener('click', () => run(acct.id, false, mount, tok));
    mount.querySelector('#rb-csv').addEventListener('click', () => run(acct.id, true, mount, tok));
}

function bodyFromForm(accountId, mount) {
    const f = mount.querySelector('#rb-form');
    const ta = mount.querySelector('#rb-targets');
    if (!f || !ta) throw new Error(t('view.rebalance.error.form_gone'));
    let targets;
    try { targets = JSON.parse(ta.value); }
    catch (e) { throw new Error(t('view.rebalance.error.targets_json', { msg: e.message })); }
    if (!Array.isArray(targets)) throw new Error(t('view.rebalance.error.targets_array'));
    return {
        account_id: accountId,
        targets,
        cash: Number(f.cash.value) || 0,
        max_trades: Number(f.max_trades.value) || 20,
    };
}

async function run(accountId, asCsv, mount, tok) {
    const status = mount.querySelector('#rb-status');
    if (status) status.textContent = asCsv ? 'building CSV…' : 'computing…';
    try {
        const body = bodyFromForm(accountId, mount);
        if (asCsv) {
            // CSV path → blob → synthetic download.
            const r = await apiFetchBlobJson('/rebalance/run/trades.csv', body);
            if (!viewIsCurrent(tok)) return;
            const url = URL.createObjectURL(r);
            const a = document.createElement('a');
            a.href = url; a.download = `rebalance-${accountId}.csv`;
            document.body.appendChild(a); a.click(); a.remove();
            setTimeout(() => URL.revokeObjectURL(url), 60_000);
            const s2 = mount.querySelector('#rb-status');
            if (s2) s2.textContent = t('common.status.downloaded');
            showToast(t('common.status.downloaded'), { level: 'success' });
            return;
        }
        const r = await api.rebalanceRun(body);
        if (!viewIsCurrent(tok)) return;
        render(r, mount);
        const s2 = mount.querySelector('#rb-status');
        const msg = t('view.rebalance.status.result', { trades: r.plan.trade_count, traded: fmt(r.plan.total_trade_value), portfolio: fmt(r.plan.total_value) });
        if (s2) s2.textContent = msg;
        showToast(msg, { level: r.plan.trade_count > 0 ? 'success' : 'info' });
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const s2 = mount.querySelector('#rb-status');
        if (s2) s2.textContent = t('common.error', { err: e.message });
        showToast(t('toast.error.api', { err: e.message }), { level: 'error' });
    }
}

// Helper because apiFetchBlob is GET-only; we need a POST-with-blob path.
async function apiFetchBlobJson(path, body) {
    const token = localStorage.getItem('tv-token') || '';
    const headers = { 'Content-Type': 'application/json' };
    if (token) headers['Authorization'] = `Bearer ${token}`;
    const res = await fetch(`/api${path}`, {
        method: 'POST', headers, body: JSON.stringify(body),
    });
    if (!res.ok) {
        let msg = res.statusText;
        try { msg = (await res.json()).error || msg; } catch (_) {}
        throw new Error(msg);
    }
    return res.blob();
}

function render(r, mount) {
    const out = mount.querySelector('#rb-out');
    if (!out) return;
    const p = r.plan;
    out.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.rebalance.card.portfolio_value">Portfolio value</div>
                <div class="value">$${fmt(p.total_value)}</div></div>
            <div class="card"><div class="label" data-i18n="view.rebalance.card.trade_count">Trade count</div>
                <div class="value">${p.trade_count}</div></div>
            <div class="card"><div class="label" data-i18n="view.rebalance.card.total_traded">Total $ traded</div>
                <div class="value">$${fmt(p.total_trade_value)}</div></div>
            <div class="card"><div class="label" data-i18n="view.rebalance.card.cash_target">Cash now → target</div>
                <div class="value">$${fmt(p.cash_current)} → $${fmt(p.cash_target)}</div></div>
        </div>

        ${p.warnings.length === 0 ? '' : `<div class="chart-panel">
            <ul class="muted small">${p.warnings.map(w => `<li class="neg">${esc(w)}</li>`).join('')}</ul>
        </div>`}

        <div class="chart-panel">
            <h2>${esc(t('view.rebalance.h2.trade_list', { count: p.trades.length }))}</h2>
            ${tradeTable(p.trades)}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.rebalance.h2.all_positions_current_vs_target">All positions — current vs target</h2>
            ${positionsTable(p.rows)}
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.rebalance.h2.drift_chart">Drift % per position</h2>
            <div id="rb-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.rebalance.h2.trade_value_chart">Absolute trade $ value per position (signed)</h2>
            <div id="rb-trade-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.rebalance.hint.trade_value" class="muted small">Signed dollar amount to trade per symbol. Positive = buy, negative = sell. Reveals absolute dollar magnitude of the rebalance independent of drift %.</p>
        </div>
    `;
    try { applyUiI18n(out); } catch (_) {}
    renderDriftChart(p.rows);
    renderTradeValueChart(p.rows);
}

function renderTradeValueChart(rows) {
    const el = document.getElementById('rb-trade-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (rows || []).filter(r => Number.isFinite(Number(r.target_value)) && Number.isFinite(Number(r.current_value)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.rebalance.empty_trade_chart">${esc(t('view.rebalance.empty_trade_chart'))}</div>`;
        return;
    }
    const sorted = valid.slice().sort((a, b) => Math.abs(Number(b.target_value) - Number(b.current_value)) - Math.abs(Number(a.target_value) - Number(a.current_value)));
    const labels = sorted.map(r => r.symbol);
    const ys = sorted.map(r => Number(r.target_value) - Number(r.current_value));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.rebalance.chart.symbol_idx') },
            { label: t('view.rebalance.chart.trade_value'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 14, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.rebalance.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function renderDriftChart(rows) {
    const el = document.getElementById('rb-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (rows || []).filter(r => Number.isFinite(r.drift_pct));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.rebalance.empty_chart">${esc(t('view.rebalance.empty_chart'))}</div>`;
        return;
    }
    const labels = valid.map(r => r.symbol);
    const drift = valid.map(r => r.drift_pct * 100);
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.rebalance.chart.symbol_idx') },
            { label: t('view.rebalance.chart.drift_pct'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.rebalance.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, drift, zero], el);
}

function tradeTable(trades) {
    if (!trades.length) return '<p data-i18n="view.rebalance.hint.no_trades_already_balanced" class="muted small">No trades — already balanced.</p>';
    return `<table class="trades">
        <thead><tr>
            <th data-i18n="view.rebalance.th.symbol">Symbol</th><th data-i18n="view.rebalance.th.side">Side</th><th data-i18n="view.rebalance.th.qty">Qty</th><th data-i18n="view.rebalance.th.price">Price</th><th data-i18n="view.rebalance.th.dollar_value">$ value</th>
            <th data-i18n="view.rebalance.th.current_target_qty">Current → Target qty</th>
        </tr></thead>
        <tbody>
        ${trades.map(t => `<tr>
            <td>${esc(t.symbol)}</td>
            <td class="${t.side === 'buy' ? 'pos' : 'neg'}">${t.side.toUpperCase()}</td>
            <td>${Math.abs(t.trade_qty)}</td>
            <td>$${fmt(t.price, t.price < 10 ? 4 : 2)}</td>
            <td>$${fmt(Math.abs(t.trade_value))}</td>
            <td class="small muted">${fmt(t.current_qty, 0)} → ${t.target_qty}</td>
        </tr>`).join('')}
        </tbody></table>`;
}

function positionsTable(rows) {
    if (!rows.length) return '<p data-i18n="view.rebalance.hint.no_rows" class="muted small">no rows</p>';
    return `<table class="trades">
        <thead><tr>
            <th data-i18n="view.rebalance.th.symbol_2">Symbol</th><th data-i18n="view.rebalance.th.current">Current %</th><th data-i18n="view.rebalance.th.target">Target %</th><th data-i18n="view.rebalance.th.drift">Drift %</th>
            <th data-i18n="view.rebalance.th.current_2">Current $</th><th data-i18n="view.rebalance.th.target_2">Target $</th><th data-i18n="view.rebalance.th.qty_2">Δ qty</th>
        </tr></thead>
        <tbody>
        ${rows.map(r => `<tr>
            <td>${esc(r.symbol)}</td>
            <td>${(r.current_pct * 100).toFixed(2)}%</td>
            <td>${(r.target_pct * 100).toFixed(2)}%</td>
            <td class="${Math.abs(r.drift_pct) < 0.005 ? 'muted' : r.drift_pct >= 0 ? 'pos' : 'neg'}">
                ${(r.drift_pct >= 0 ? '+' : '') + (r.drift_pct * 100).toFixed(2)}%
            </td>
            <td>$${fmt(r.current_value)}</td>
            <td>$${fmt(r.target_value)}</td>
            <td class="${r.trade_qty > 0 ? 'pos' : r.trade_qty < 0 ? 'neg' : 'muted'}">
                ${r.trade_qty >= 0 ? '+' : ''}${r.trade_qty}
            </td>
        </tr>`).join('')}
        </tbody></table>`;
}
