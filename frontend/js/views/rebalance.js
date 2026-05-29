// Portfolio rebalancing tool — target weights, current-vs-target diff,
// exact trade list with CSV download for manual broker entry.

import { api, apiFetchBlob } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const PRESETS = {
    '60/40 stocks/bonds': [
        { symbol: 'VTI', weight: 0.60 },
        { symbol: 'BND', weight: 0.40 },
    ],
    'Three-fund (Bogle)': [
        { symbol: 'VTI', weight: 0.60 },
        { symbol: 'VXUS', weight: 0.20 },
        { symbol: 'BND',  weight: 0.20 },
    ],
    'Permanent Portfolio (Browne)': [
        { symbol: 'VTI', weight: 0.25 },
        { symbol: 'TLT', weight: 0.25 },
        { symbol: 'GLD', weight: 0.25 },
        { symbol: 'BIL', weight: 0.25 },
    ],
    'Risk parity (lite)': [
        { symbol: 'VTI', weight: 0.30 },
        { symbol: 'TLT', weight: 0.40 },
        { symbol: 'GLD', weight: 0.15 },
        { symbol: 'IEF', weight: 0.15 },
    ],
};

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
                <label>Preset
                    <select name="preset">
                        <option data-i18n="view.rebalance.opt.custom" value="">(custom)</option>
                        ${Object.keys(PRESETS).map(k => `<option>${esc(k)}</option>`).join('')}
                    </select>
                </label>
                <label>Cash on hand
                    <input name="cash" type="number" min="0" step="any" value="0" style="width:120px;">
                </label>
                <label>Max trades
                    <input name="max_trades" type="number" min="1" max="200" value="20" style="width:80px;">
                </label>
                <button data-i18n="view.rebalance.btn.compute_plan" class="primary" id="rb-go" type="button">Compute plan</button>
                <button data-i18n="view.rebalance.btn.download_trades_csv" class="btn" id="rb-csv" type="button">Download trades CSV</button>
                <span id="rb-status" class="muted small"></span>
            </form>
            <textarea id="rb-targets" rows="8"
                style="width:100%;font-family:'Share Tech Mono',monospace;font-size:11px;background:#070714;color:#cfd2e8;border:1px solid var(--border);padding:8px;margin-top:8px;"
                placeholder='Targets JSON: [{"symbol":"SPY","weight":0.6},{"symbol":"BND","weight":0.4}]'>[
  {"symbol":"SPY","weight":0.6},
  {"symbol":"BND","weight":0.4}
]</textarea>
        </div>

        <div id="rb-out"></div>
    `;

    mount.querySelector('#rb-form [name=preset]').addEventListener('change', (e) => {
        const k = e.target.value;
        if (k && PRESETS[k]) {
            const ta = mount.querySelector('#rb-targets');
            if (ta) ta.value = JSON.stringify(PRESETS[k], null, 2);
        }
    });

    mount.querySelector('#rb-go').addEventListener('click', () => run(acct.id, false, mount, tok));
    mount.querySelector('#rb-csv').addEventListener('click', () => run(acct.id, true, mount, tok));
}

function bodyFromForm(accountId, mount) {
    const f = mount.querySelector('#rb-form');
    const ta = mount.querySelector('#rb-targets');
    if (!f || !ta) throw new Error('form gone');
    let targets;
    try { targets = JSON.parse(ta.value); }
    catch (e) { throw new Error('targets JSON invalid: ' + e.message); }
    if (!Array.isArray(targets)) throw new Error('targets must be a JSON array');
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
            if (s2) s2.textContent = 'downloaded';
            return;
        }
        const r = await api.rebalanceRun(body);
        if (!viewIsCurrent(tok)) return;
        render(r, mount);
        const s2 = mount.querySelector('#rb-status');
        if (s2) s2.textContent = `${r.plan.trade_count} trades · $${fmt(r.plan.total_trade_value)} traded · $${fmt(r.plan.total_value)} portfolio`;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const s2 = mount.querySelector('#rb-status');
        if (s2) s2.textContent = 'error: ' + e.message;
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
            <div class="card"><div class="label">Portfolio value</div>
                <div class="value">$${fmt(p.total_value)}</div></div>
            <div class="card"><div class="label">Trade count</div>
                <div class="value">${p.trade_count}</div></div>
            <div class="card"><div class="label">Total $ traded</div>
                <div class="value">$${fmt(p.total_trade_value)}</div></div>
            <div class="card"><div class="label">Cash now → target</div>
                <div class="value">$${fmt(p.cash_current)} → $${fmt(p.cash_target)}</div></div>
        </div>

        ${p.warnings.length === 0 ? '' : `<div class="chart-panel">
            <ul class="muted small">${p.warnings.map(w => `<li class="neg">${esc(w)}</li>`).join('')}</ul>
        </div>`}

        <div class="chart-panel">
            <h2>Trade list (${p.trades.length})</h2>
            ${tradeTable(p.trades)}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.rebalance.h2.all_positions_current_vs_target">All positions — current vs target</h2>
            ${positionsTable(p.rows)}
        </div>
    `;
}

function tradeTable(trades) {
    if (!trades.length) return '<p data-i18n="view.rebalance.hint.no_trades_already_balanced" class="muted small">No trades — already balanced.</p>';
    return `<table class="trades">
        <thead><tr>
            <th data-i18n="view.rebalance.th.symbol">Symbol</th><th data-i18n="view.rebalance.th.side">Side</th><th data-i18n="view.rebalance.th.qty">Qty</th><th data-i18n="view.rebalance.th.price">Price</th><th>$ value</th>
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
