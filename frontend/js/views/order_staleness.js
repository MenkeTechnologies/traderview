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
import { showToast } from '../toast.js';
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
            <textarea id="os-orders" rows="8" placeholder="A1 AAPL 2024-06-15T10:00:00Z buy&#10;B1 TSLA 2024-06-14T10:00:00Z 2024-06-15T08:00:00Z sell_stop" data-tip="view.order_staleness.tip.orders"></textarea>
            <div class="inline-form">
                <button data-i18n="view.order_staleness.btn.load_demo_12_orders_across_all_tiers" data-tip="view.order_staleness.tip.demo" data-shortcut="order_staleness_demo" id="os-demo" class="secondary" type="button">Load demo (12 orders across all tiers)</button>
                <button data-i18n="view.order_staleness.btn.clear" data-tip="view.order_staleness.tip.clear" id="os-clear" class="secondary" type="button">Clear</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.order_staleness.h2.reference_time_thresholds">Reference time + thresholds</h2>
            <div class="inline-form">
                <label><span data-i18n="view.order_staleness.label.now">Now (ISO 8601)</span>
                    <input id="os-now" type="text" value="${state.now}" data-tip="view.order_staleness.tip.now"></label>
                <button data-i18n="view.order_staleness.btn.now_current_time" data-tip="view.order_staleness.tip.now_now" id="os-nownow" class="secondary" type="button">now = current time</button>
                <label><span data-i18n="view.order_staleness.label.warn_h">Warn (h)</span>
                    <input id="os-warn" type="number" step="0.01" min="0" value="${state.thresholds.warn_hours}" data-tip="view.order_staleness.tip.warn"></label>
                <label><span data-i18n="view.order_staleness.label.stale_h">Stale (h)</span>
                    <input id="os-stale" type="number" step="0.01" min="0" value="${state.thresholds.stale_hours}" data-tip="view.order_staleness.tip.stale"></label>
                <label><span data-i18n="view.order_staleness.label.forgotten_h">Forgotten (h)</span>
                    <input id="os-forgot" type="number" step="0.01" min="0" value="${state.thresholds.forgotten_hours}" data-tip="view.order_staleness.tip.forgotten"></label>
                <button data-i18n="view.order_staleness.btn.evaluate" data-tip="view.order_staleness.tip.evaluate" data-shortcut="order_staleness_evaluate" id="os-run" class="primary" type="button">Evaluate</button>
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

        <div class="chart-panel">
            <h2 data-i18n="view.order_staleness.h2.tier_chart">Tier counts</h2>
            <div id="os-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.order_staleness.h2.age_dist_chart">Age distribution (hours)</h2>
            <div id="os-age-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.order_staleness.hint.age_dist" class="muted small">Histogram of resting-order ages. Reveals the shape — long tail = forgotten cluster, peak near zero = active scalping cluster. Yellow dashed = your warn threshold.</p>
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
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (orders.length === 0) return;
    }
    const err = validateInputs(orders, state.now, state.thresholds);
    if (err) { showErr(err); showToast(err, { level: 'warning' }); return; }
    let report;
    try {
        report = await api.microOrderStaleness(buildBody(orders, state.now, state.thresholds));
    } catch (e) {
        const m = t("common.error.api", { msg: e.message || e });
        showErr(m); showToast(m, { level: 'error' }); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report);
    renderTable(report);
    const rows = report.orders || [];
    const stale = rows.filter(r => r.tier === 'stale' || r.tier === 'forgotten').length;
    showToast(t('view.order_staleness.toast.done', {
        orders: rows.length,
        stale,
    }), { level: stale > 0 ? 'warning' : 'success' });
    renderTierChart(report);
    renderAgeDistChart(report);
}

function renderAgeDistChart(report) {
    const el = document.getElementById('os-age-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (report && report.rows) || [];
    const ages = rows.map(r => Number(r.age_hours)).filter(Number.isFinite);
    if (ages.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.order_staleness.empty_age_chart">${esc(t('view.order_staleness.empty_age_chart'))}</div>`;
        return;
    }
    const maxA = Math.max(...ages);
    const bins = Math.min(20, Math.max(4, Math.ceil(Math.sqrt(ages.length))));
    const span = maxA || 1;
    const counts = new Array(bins).fill(0);
    for (const a of ages) {
        const idx = Math.min(bins - 1, Math.max(0, Math.floor(a / span * bins)));
        counts[idx] += 1;
    }
    const xs = Array.from({ length: bins }, (_, i) => (i + 0.5) * (span / bins));
    const warn = Number(state.thresholds.warn_hours) || 0;
    const peak = Math.max(...counts);
    const warnLine = xs.map(x => Math.abs(x - warn) < (span / bins / 2) ? peak : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false, auto: true }, y: { auto: true } },
        series: [
            { label: t('view.order_staleness.chart.age_bin') },
            { label: t('view.order_staleness.chart.count'),
              stroke: '#b86bff', width: 1.5,
              fill: '#b86bff33', points: { show: true, size: 5 } },
            { label: t('view.order_staleness.chart.warn_marker'),
              stroke: '#ffd84a', width: 0,
              points: { show: true, size: 12, fill: '#ffd84a', stroke: '#ffd84a' } },
        ],
        axes: [{ stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => v.toFixed(0) + 'h') },
            { stroke: '#aab', size: 40 }],
        legend: { show: true },
    }, [xs, counts, warnLine], el);
}

function renderTierChart(report) {
    const el = document.getElementById('os-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const tiers = [
        { key: 'fresh',     count: Number(report.fresh_count) || 0,     color: '#7af0a8' },
        { key: 'aging',     count: Number(report.aging_count) || 0,     color: '#ffd84a' },
        { key: 'stale',     count: Number(report.stale_count) || 0,     color: '#ff7a1f' },
        { key: 'forgotten', count: Number(report.forgotten_count) || 0, color: '#ff3860' },
    ];
    if (!tiers.some(b => b.count > 0)) {
        el.innerHTML = `<div class="muted" data-i18n="view.order_staleness.empty_chart">${esc(t('view.order_staleness.empty_chart'))}</div>`;
        return;
    }
    const labels = tiers.map(b => t(`view.order_staleness.chart.${b.key}`));
    const xs = labels.map((_, i) => i + 1);
    const series = [{ label: t('view.order_staleness.chart.tier_idx') }];
    const data = [xs];
    tiers.forEach((b, i) => {
        const ys = xs.map((_, j) => j === i ? b.count : null);
        series.push({
            label: labels[i], stroke: b.color, width: 0,
            points: { show: true, size: 16, fill: b.color, stroke: b.color },
        });
        data.push(ys);
    });
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series,
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, data, el);
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
