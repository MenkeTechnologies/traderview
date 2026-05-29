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

import { t } from '../i18n.js';
const DEFAULT_THRESH = { warn_hours: 24, stale_hours: 72, forgotten_hours: 168 };

let state = {
    ordersText: '',
    now: '2024-06-15T15:00:00Z',
    thresholds: { ...DEFAULT_THRESH },
};

export async function renderOrderStaleness(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.order_staleness.h1.order_staleness" class="view-title">// ORDER STALENESS</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.order_staleness.h2.resting_orders">Resting orders</h2>
            <p class="muted" data-i18n="view.order_staleness.hint.format">One line per order: id symbol placed_at [last_modified_at] side. Timestamps accept ISO 8601 (2024-06-15T10:00:00Z). Side is one of buy / sell / buy_stop / sell_stop. Modifying an order resets the staleness clock — treated as "re-confirming intent."</p>
            <textarea id="os-orders" rows="8" placeholder="A1 AAPL 2024-06-15T10:00:00Z buy&#10;B1 TSLA 2024-06-14T10:00:00Z 2024-06-15T08:00:00Z sell_stop"></textarea>
            <div class="inline-form">
                <button data-i18n="view.order_staleness.btn.load_demo_12_orders_across_all_tiers" id="os-demo" class="secondary" type="button">Load demo (12 orders across all tiers)</button>
                <button data-i18n="view.order_staleness.btn.clear" id="os-clear" class="secondary" type="button">Clear</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.order_staleness.h2.reference_time_thresholds">Reference time + thresholds</h2>
            <div class="inline-form">
                <label><span data-i18n="view.order_staleness.label.now">Now (ISO 8601)</span>
                    <input id="os-now" type="text" value="${state.now}"></label>
                <button data-i18n="view.order_staleness.btn.now_current_time" id="os-nownow" class="secondary" type="button">now = current time</button>
                <label><span data-i18n="view.order_staleness.label.warn_h">Warn (h)</span>
                    <input id="os-warn" type="number" step="any" min="0" value="${state.thresholds.warn_hours}"></label>
                <label><span data-i18n="view.order_staleness.label.stale_h">Stale (h)</span>
                    <input id="os-stale" type="number" step="any" min="0" value="${state.thresholds.stale_hours}"></label>
                <label><span data-i18n="view.order_staleness.label.forgotten_h">Forgotten (h)</span>
                    <input id="os-forgot" type="number" step="any" min="0" value="${state.thresholds.forgotten_hours}"></label>
                <button data-i18n="view.order_staleness.btn.evaluate" id="os-run" class="primary" type="button">Evaluate</button>
            </div>
            <p data-i18n="view.order_staleness.hint.industry_defaults_24h_warn_72h_stale_168h_forgotte" class="muted">Industry defaults: 24h warn / 72h stale / 168h forgotten.
                Tight intraday traders use 1h/4h/24h. Position traders use 7d/30d/90d.</p>
        </div>

        <div id="os-errors" class="boot" style="display:none"></div>
        <div id="os-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.order_staleness.h2.orders_sorted_most_stale_first">Orders sorted most-stale first</h2>
            <div id="os-table"></div>
            <p data-i18n="view.order_staleness.hint.stale_forgotten_rows_are_candidates_to_cancel_most" class="muted">STALE/FORGOTTEN rows are candidates to cancel. Most accidental
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
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report);
    renderTable(report);
}

function renderSummary(r) {
    const total = (r.fresh_count + r.aging_count + r.stale_count + r.forgotten_count) || 0;
    const liabilityPct = total > 0 ? (r.stale_count + r.forgotten_count) / total : 0;
    document.getElementById('os-summary').innerHTML = [
        card(t('view.order_staleness.card.total_orders'),    String(total)),
        card(t('view.order_staleness.card.fresh'),           String(r.fresh_count),     r.fresh_count ? 'pos' : ''),
        card(t('view.order_staleness.card.aging'),           String(r.aging_count)),
        card(t('view.order_staleness.card.stale'),           String(r.stale_count),     r.stale_count ? 'neg' : ''),
        card(t('view.order_staleness.card.forgotten'),       String(r.forgotten_count), r.forgotten_count ? 'neg' : ''),
        card(t('view.order_staleness.card.liability'),     (liabilityPct * 100).toFixed(0) + '%',
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
        wrap.innerHTML = `<div class="muted" data-i18n="view.order_staleness.empty.orders">No orders.</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.order_staleness.th.order_id">Order ID</th><th data-i18n="view.order_staleness.th.symbol">Symbol</th><th data-i18n="view.order_staleness.th.age">Age</th><th data-i18n="view.order_staleness.th.tier">Tier</th>
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
