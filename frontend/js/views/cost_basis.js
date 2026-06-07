// Cost-basis lot accounting view. FIFO / LIFO / HIFO / LOFO lot
// selection on a partial close.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    METHODS, parseLotBlob, validateInputs, buildBody, localClose, dec,
    realizedBadge, suggestMethod, methodLabelKey,
    makeDemoLots, makeDemoQtyPrice,
    fmtUSD, fmtUSDSigned, fmtNum,
} from '../_cost_basis_inputs.js';

let state = {
    lots: makeDemoLots('classic'),
    qty_to_close: 100,
    price_per_share: 200,
    method: 'fifo',
};

export async function renderCostBasis(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.cost_basis.h1.title" class="view-title">// COST BASIS</h1>

        <div class="chart-panel" data-context-scope="cost-basis">
            <h2 data-i18n="view.cost_basis.h2.lots">Open cost lots
                <small data-i18n="view.cost_basis.h2.lots_hint" class="muted">(per line: lot_id YYYY-MM-DD qty cost_per_share)</small></h2>
            <textarea id="cb-lots" rows="6"
                      data-tip="view.cost_basis.tip.lots"
                      placeholder="A 2024-01-15 100 100&#10;B 2024-06-10 100 150">${esc(lotsToBlob(state.lots))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.cost_basis.label.qty">Qty to close</span>
                    <input id="cb-qty" type="number" step="0.01" min="0" value="${state.qty_to_close}" data-tip="view.cost_basis.tip.qty"></label>
                <label><span data-i18n="view.cost_basis.label.price">Price per share ($)</span>
                    <input id="cb-px" type="number" step="0.01" min="0" value="${state.price_per_share}" data-tip="view.cost_basis.tip.price"></label>
                <label><span data-i18n="view.cost_basis.label.method">Method</span>
                    <select id="cb-method" data-tip="view.cost_basis.tip.method">
                        ${METHODS.map(m => `<option value="${m}" ${state.method === m ? 'selected' : ''} data-i18n="${methodLabelKey(m)}">${m.toUpperCase()}</option>`).join('')}
                    </select></label>
                <button data-i18n="view.cost_basis.btn.compute" id="cb-run" class="primary"
                        data-tip="view.cost_basis.tip.compute" data-shortcut="cost_basis_run" type="button">Compute</button>
                <button data-i18n="view.cost_basis.btn.optimize" id="cb-opt" class="secondary"
                        data-tip="view.cost_basis.tip.optimize" data-shortcut="cost_basis_opt" type="button">Use tax-optimal method</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.cost_basis.btn.demo_classic" id="cb-demo-classic" class="secondary" type="button" data-tip="view.cost_basis.tip.demo_classic">Demo: 3 lots @ different prices</button>
                <button data-i18n="view.cost_basis.btn.demo_gain"    id="cb-demo-gain"    class="secondary" type="button" data-tip="view.cost_basis.tip.demo_gain">Demo: all gains (HIFO wins)</button>
                <button data-i18n="view.cost_basis.btn.demo_loss"    id="cb-demo-loss"    class="secondary" type="button" data-tip="view.cost_basis.tip.demo_loss">Demo: all losses (LOFO wins)</button>
                <button data-i18n="view.cost_basis.btn.demo_many"    id="cb-demo-many"    class="secondary" type="button" data-tip="view.cost_basis.tip.demo_many">Demo: 6 lots ladder</button>
                <button data-i18n="view.cost_basis.btn.demo_single"  id="cb-demo-single"  class="secondary" type="button" data-tip="view.cost_basis.tip.demo_single">Demo: single lot partial</button>
                <button data-i18n="view.cost_basis.btn.demo_over"    id="cb-demo-over"    class="secondary" type="button" data-tip="view.cost_basis.tip.demo_over">Demo: over-close (200/100)</button>
            </div>
            <p data-i18n="view.cost_basis.hint.about" class="muted">FIFO = oldest first (IRS default). LIFO = newest. HIFO = highest cost (minimize gain). LOFO = lowest cost (maximize gain — useful for tax-loss carryforward). Realized = (price − cost) × qty per lot.</p>
        </div>

        <div id="cb-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.cost_basis.h2.closes">Closing schedule</h2>
            <div id="cb-table"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cost_basis.h2.realized_chart">Realized P&amp;L per lot</h2>
            <div id="cb-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cost_basis.h2.method_compare_chart">Realized total across all 4 methods — tax-optimization landscape</h2>
            <div id="cb-method-chart" style="width:100%;height:220px"></div>
        </div>

        <div id="cb-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state.lots = makeDemoLots(k);
        const qp = makeDemoQtyPrice(k);
        state.qty_to_close = qp.qty_to_close;
        state.price_per_share = qp.price_per_share;
        document.getElementById('cb-lots').value = lotsToBlob(state.lots);
        document.getElementById('cb-qty').value  = state.qty_to_close;
        document.getElementById('cb-px').value   = state.price_per_share;
    };
    document.getElementById('cb-demo-classic').addEventListener('click', () => loadDemo('classic'));
    document.getElementById('cb-demo-gain').addEventListener('click',    () => loadDemo('gain-only'));
    document.getElementById('cb-demo-loss').addEventListener('click',    () => loadDemo('loss-only'));
    document.getElementById('cb-demo-many').addEventListener('click',    () => loadDemo('many-lots'));
    document.getElementById('cb-demo-single').addEventListener('click',  () => loadDemo('single-lot'));
    document.getElementById('cb-demo-over').addEventListener('click',    () => loadDemo('overclose'));
    document.getElementById('cb-opt').addEventListener('click', () => {
        readInputs();
        const opt = suggestMethod(state.lots, state.qty_to_close, state.price_per_share);
        state.method = opt;
        document.getElementById('cb-method').value = opt;
        showToast(t('view.cost_basis.toast.optimized', { method: opt.toUpperCase() }), { level: 'info' });
        void compute(tok);
    });
    document.getElementById('cb-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function lotsToBlob(lots) {
    return lots.map(l => `${l.lot_id} ${l.acquired} ${l.qty} ${l.cost_per_share}`).join('\n');
}

function readInputs() {
    const p = parseLotBlob(document.getElementById('cb-lots').value);
    if (p.errors.length) {
        showErr(`${t('view.cost_basis.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.cost_basis.toast.parse_error', { n: p.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.lots            = p.lots;
    state.qty_to_close    = Number(document.getElementById('cb-qty').value);
    state.price_per_share = Number(document.getElementById('cb-px').value);
    state.method          = document.getElementById('cb-method').value;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.lots, state.qty_to_close, state.price_per_share, state.method);
    if (err) { showErr(err); showToast(t('view.cost_basis.toast.invalid'), { level: 'warning' }); return; }
    const local = localClose(state.lots, state.qty_to_close, state.price_per_share, state.method);
    renderSummary(local, true);
    renderTable(local);
    renderRealizedChart(local);
    renderMethodChart();
    let resp;
    try {
        resp = await api.calcCostBasis(buildBody(
            state.lots, state.qty_to_close, state.price_per_share, state.method));
    } catch (e) {
        showErr(`${t('view.cost_basis.err.api')}: ${e.message || e}`);
        showToast(t('view.cost_basis.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    const normalized = {
        closes: (resp.closes || []).map(c => ({
            ...c,
            qty_closed:        dec(c.qty_closed),
            cost_per_share:    dec(c.cost_per_share),
            realized_per_share: dec(c.realized_per_share),
            realized_total:    dec(c.realized_total),
        })),
        total_realized:         dec(resp.total_realized),
        qty_remaining_to_close: dec(resp.qty_remaining_to_close),
    };
    renderSummary(normalized, false);
    renderTable(normalized);
    renderRealizedChart(normalized);
    renderMethodChart();
    const realized = Number(normalized.total_realized) || 0;
    const remaining = Number(normalized.qty_remaining_to_close) || 0;
    const level = remaining > 0 ? 'warning' : realized >= 0 ? 'success' : 'info';
    showToast(t('view.cost_basis.toast.computed', {
        method: state.method.toUpperCase(),
        realized: Math.round(realized).toLocaleString(),
        remaining,
    }), { level });
}

function renderSummary(report, pending) {
    const badge = realizedBadge(report.total_realized);
    const local = localClose(state.lots, state.qty_to_close, state.price_per_share, state.method);
    const parityOk = Math.abs(report.total_realized - local.total_realized) < 1e-6
                  && report.closes.length === local.closes.length;
    const localTag = pending ? ` (${t('view.cost_basis.tag.local')})` : '';
    const optimal = suggestMethod(state.lots, state.qty_to_close, state.price_per_share);
    document.getElementById('cb-summary').innerHTML = [
        card(t('view.cost_basis.card.verdict'),
             t(badge.key) + localTag, badge.cls),
        card(t('view.cost_basis.card.realized'),
             fmtUSDSigned(report.total_realized),
             report.total_realized >= 0 ? 'neg' : 'pos'),  // gain = taxable (neg), loss = harvestable (pos)
        card(t('view.cost_basis.card.qty_closed'),
             fmtNum(state.qty_to_close - report.qty_remaining_to_close, 4)),
        card(t('view.cost_basis.card.qty_remaining'),
             fmtNum(report.qty_remaining_to_close, 4),
             report.qty_remaining_to_close > 0 ? 'neg' : 'pos'),
        card(t('view.cost_basis.card.lots_touched'),
             String(report.closes.length)),
        card(t('view.cost_basis.card.method'),
             t(methodLabelKey(state.method))),
        card(t('view.cost_basis.card.optimal_method'),
             t(methodLabelKey(optimal)),
             optimal === state.method ? 'pos' : 'neg'),
        card(t('view.cost_basis.card.parity'),
             parityOk ? t('view.cost_basis.tag.ok') : t('view.cost_basis.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderTable(report) {
    const wrap = document.getElementById('cb-table');
    if (!report.closes || report.closes.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.cost_basis.empty">${esc(t('view.cost_basis.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.cost_basis.col.lot_id">Lot</th>
                <th data-i18n="view.cost_basis.col.qty_closed">Qty closed</th>
                <th data-i18n="view.cost_basis.col.cost">Cost / sh</th>
                <th data-i18n="view.cost_basis.col.price">Price / sh</th>
                <th data-i18n="view.cost_basis.col.realized_ps">Realized / sh</th>
                <th data-i18n="view.cost_basis.col.realized_total">Realized total</th>
            </tr></thead>
            <tbody>
                ${report.closes.map(c => `<tr>
                    <td><strong>${esc(c.lot_id)}</strong></td>
                    <td>${esc(fmtNum(c.qty_closed, 4))}</td>
                    <td>${esc(fmtUSD(c.cost_per_share))}</td>
                    <td>${esc(fmtUSD(state.price_per_share))}</td>
                    <td class="${c.realized_per_share >= 0 ? 'neg' : 'pos'}">${esc(fmtUSDSigned(c.realized_per_share))}</td>
                    <td class="${c.realized_total >= 0 ? 'neg' : 'pos'}">${esc(fmtUSDSigned(c.realized_total))}</td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
}

function renderRealizedChart(report) {
    const el = document.getElementById('cb-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!report || !Array.isArray(report.closes) || report.closes.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.cost_basis.empty_chart">${esc(t('view.cost_basis.empty_chart'))}</div>`;
        return;
    }
    const labels = report.closes.map(c => c.lot_id);
    const realized = report.closes.map(c => Number.isFinite(c.realized_total) ? c.realized_total : null);
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.cost_basis.chart.lot_idx') },
            { label: t('view.cost_basis.chart.realized'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.cost_basis.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, realized, zero], el);
}

function renderMethodChart() {
    const el = document.getElementById('cb-method-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!state.lots.length || !(state.qty_to_close > 0) || !(state.price_per_share > 0)) {
        el.innerHTML = `<div class="muted" data-i18n="view.cost_basis.empty_method_chart">${esc(t('view.cost_basis.empty_method_chart'))}</div>`;
        return;
    }
    const labels = METHODS.map(m => m.toUpperCase());
    const realized = METHODS.map(m => {
        const r = localClose(state.lots, state.qty_to_close, state.price_per_share, m);
        return Number.isFinite(r.total_realized) ? r.total_realized : null;
    });
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.cost_basis.chart.method_idx') },
            { label: t('view.cost_basis.chart.realized_total'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 14, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.cost_basis.chart.zero'),
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
    }, [xs, realized, zero], el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('cb-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cb-err').style.display = 'none'; }
