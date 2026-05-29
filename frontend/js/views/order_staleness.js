// Order Staleness view — resting-order freshness gauge.
//
// Surfaces orders that have sat in the book past your warn/stale/forgotten
// thresholds. Modifying an order resets the clock (treated as
// re-confirming intent). Most-stale shown first.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseOrderBlob, validateInputs, buildBody,
    tierBadge, fmtHours, makeDemoData, nowIso,
} from '../_order_staleness_inputs.js';

const DEFAULT_THRESH = { warn_hours: 24, stale_hours: 72, forgotten_hours: 168 };

let state = {
    ordersText: '',
    now: '2024-06-15T15:00:00Z',
    thresholds: { ...DEFAULT_THRESH },
};

export async function renderOrderStaleness(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// ORDER STALENESS</h1>

        <div class="chart-panel">
            <h2>Resting orders</h2>
            <p class="muted">One line per order: <code>id symbol placed_at [last_modified_at] side</code>.
                Timestamps accept ISO 8601 (<code>2024-06-15T10:00:00Z</code>). Side is one of
                <code>buy / sell / buy_stop / sell_stop</code>. Modifying an order resets the
                staleness clock — treated as "re-confirming intent."</p>
            <textarea id="os-orders" rows="8" placeholder="A1 AAPL 2024-06-15T10:00:00Z buy&#10;B1 TSLA 2024-06-14T10:00:00Z 2024-06-15T08:00:00Z sell_stop"></textarea>
            <div class="inline-form">
                <button id="os-demo" class="secondary" type="button">Load demo (12 orders across all tiers)</button>
                <button id="os-clear" class="secondary" type="button">Clear</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2>Reference time + thresholds</h2>
            <div class="inline-form">
                <label>Now (ISO 8601)
                    <input id="os-now" type="text" value="${state.now}"></label>
                <button id="os-nownow" class="secondary" type="button">now = current time</button>
                <label>Warn (h)
                    <input id="os-warn" type="number" step="any" min="0" value="${state.thresholds.warn_hours}"></label>
                <label>Stale (h)
                    <input id="os-stale" type="number" step="any" min="0" value="${state.thresholds.stale_hours}"></label>
                <label>Forgotten (h)
                    <input id="os-forgot" type="number" step="any" min="0" value="${state.thresholds.forgotten_hours}"></label>
                <button id="os-run" class="primary" type="button">Evaluate</button>
            </div>
            <p class="muted">Industry defaults: 24h warn / 72h stale / 168h forgotten.
                Tight intraday traders use 1h/4h/24h. Position traders use 7d/30d/90d.</p>
        </div>

        <div id="os-errors" class="boot" style="display:none"></div>
        <div id="os-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Orders sorted most-stale first</h2>
            <div id="os-table"></div>
            <p class="muted">STALE/FORGOTTEN rows are candidates to cancel. Most accidental
                "what was that fill?" P&amp;L surprises come from forgotten resting orders.</p>
        </div>

        <div id="os-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('os-demo').addEventListener('click', () => {
        const { orders, now } = makeDemoData();
        document.getElementById('os-orders').value = orders.map(o => {
            const parts = [o.order_id, o.symbol, o.placed_at];
            if (o.last_modified_at) parts.push(o.last_modified_at);
            parts.push(o.side);
            return parts.join(' ');
        }).join('\n');
        document.getElementById('os-now').value = now;
    });
    document.getElementById('os-clear').addEventListener('click', () => {
        document.getElementById('os-orders').value = '';
    });
    document.getElementById('os-nownow').addEventListener('click', () => {
        document.getElementById('os-now').value = nowIso();
    });
    document.getElementById('os-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.ordersText = document.getElementById('os-orders').value;
    state.now = document.getElementById('os-now').value.trim();
    state.thresholds = {
        warn_hours:      Number(document.getElementById('os-warn').value),
        stale_hours:     Number(document.getElementById('os-stale').value),
        forgotten_hours: Number(document.getElementById('os-forgot').value),
    };
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('os-errors');
    errs.style.display = 'none';
    const { orders, errors } = parseOrderBlob(state.ordersText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (orders.length === 0) return;
    }
    const err = validateInputs(orders, state.now, state.thresholds);
    if (err) { showErr(err); return; }
    let report;
    try {
        report = await api.microOrderStaleness(buildBody(orders, state.now, state.thresholds));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report);
    renderTable(report);
}

function renderSummary(r) {
    const total = (r.fresh_count + r.aging_count + r.stale_count + r.forgotten_count) || 0;
    const liabilityPct = total > 0 ? (r.stale_count + r.forgotten_count) / total : 0;
    document.getElementById('os-summary').innerHTML = [
        card('Total orders',    String(total)),
        card('Fresh',           String(r.fresh_count),     r.fresh_count ? 'pos' : ''),
        card('Aging',           String(r.aging_count)),
        card('Stale',           String(r.stale_count),     r.stale_count ? 'neg' : ''),
        card('Forgotten',       String(r.forgotten_count), r.forgotten_count ? 'neg' : ''),
        card('Liability %',     (liabilityPct * 100).toFixed(0) + '%',
            liabilityPct > 0.25 ? 'neg' : liabilityPct === 0 ? 'pos' : ''),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderTable(report) {
    const wrap = document.getElementById('os-table');
    if (!report.rows || !report.rows.length) {
        wrap.innerHTML = '<div class="muted">No orders.</div>';
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>Order ID</th><th>Symbol</th><th>Age</th><th>Tier</th>
            </tr></thead>
            <tbody>
                ${report.rows.map(r => {
                    const t = tierBadge(r.tier);
                    return `<tr>
                        <td>${esc(r.order_id)}</td>
                        <td>${esc(r.symbol)}</td>
                        <td>${esc(fmtHours(r.age_hours))}</td>
                        <td class="${t.cls}">${esc(t.label)}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('os-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('os-err').style.display = 'none'; }
