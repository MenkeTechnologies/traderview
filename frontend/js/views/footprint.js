// Footprint chart view — Sierra Chart-class per-bar bid/ask volume +
// delta visualization. The Bookmap / ATAS / Jigsaw display: each
// price-time bar renders as a stacked column of cells (one per price
// level) showing `bid_vol × ask_vol`, with the delta color-coded.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { api } from '../api.js';
import {
    parseTickBlob, validateInputs, buildBody,
    deltaCls, summarize, imbalanceHotspots,
    makeDemoTicks, fmtN, fmtPrice, fmtSigned,
} from '../_footprint_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = { tickText: '', tickSize: 0.05 };

export async function renderFootprint(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.footprint.h1.footprint_bid_ask_per_price_level" class="view-title">// FOOTPRINT · BID/ASK PER PRICE LEVEL</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.footprint.h2.classified_tick_stream">Classified tick stream</h2>
            <p class="muted" data-i18n="view.footprint.hint.ticks">One tick per line: bar_id price volume side where side ∈ {buy, sell, uncertain}. Demo loads 4 bars with engineered patterns: balanced churn → absorption at low → drive up → rejection at high.</p>
            <textarea id="fp-ticks" rows="8" placeholder="0 100.00 50 buy&#10;0 100.00 50 sell&#10;..." data-tip="view.footprint.tip.ticks"></textarea>
            <div class="inline-form">
                <label><span data-i18n="view.footprint.label.tick_size">Tick size (price quantization)</span>
                    <input id="fp-ts" type="number" step="any" min="0" value="${state.tickSize}" data-tip="view.footprint.tip.tick_size"></label>
                <button data-i18n="view.footprint.btn.load_demo_4_bars_4_patterns" id="fp-demo" class="secondary" type="button" data-tip="view.footprint.tip.demo" data-shortcut="footprint_demo">Load demo (4 bars, 4 patterns)</button>
                <button data-i18n="view.footprint.btn.clear" id="fp-clear" class="secondary" type="button" data-tip="view.footprint.tip.clear">Clear</button>
                <button data-i18n="view.footprint.btn.build_footprint" id="fp-run" class="primary" type="button" data-tip="view.footprint.tip.run" data-shortcut="footprint_run">Build footprint</button>
            </div>
        </div>

        <div id="fp-errors" class="boot" style="display:none"></div>
        <div id="fp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.footprint.h2.footprint_bars">Footprint bars</h2>
            <div id="fp-grid" class="fp-grid"></div>
            <p class="muted" data-i18n="view.footprint.hint.grid">Each column = one bar. Each row = one price level. Cells show bid × ask; row color is the per-cell delta (green = ask won, red = bid won). Gold-highlighted row = bar POC (most-traded level).</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.footprint.h2.imbalance_hotspots_largest_abs_delta_cells">Imbalance hotspots (largest abs(delta) cells)</h2>
            <div id="fp-hotspots"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.footprint.h2.bar_delta_chart">Per-bar net delta</h2>
            <div id="fp-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.footprint.h2.cum_delta_chart">Cumulative delta across bars — running order-flow imbalance</h2>
            <div id="fp-cum-chart" style="width:100%;height:220px"></div>
        </div>

        <div id="fp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('fp-demo').addEventListener('click', () => {
        const demoTicks = makeDemoTicks();
        document.getElementById('fp-ticks').value =
            demoTicks.map(x => `${x.bar_id} ${x.price} ${x.classified.volume} ${x.classified.side}`).join('\n');
        showToast(t('view.footprint.toast.demo_loaded', { n: demoTicks.length }), { level: 'info' });
    });
    document.getElementById('fp-clear').addEventListener('click', () => {
        document.getElementById('fp-ticks').value = '';
        showToast(t('view.footprint.toast.cleared'), { level: 'info' });
    });
    document.getElementById('fp-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.tickText = document.getElementById('fp-ticks').value;
    state.tickSize = Number(document.getElementById('fp-ts').value);
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('fp-errors');
    errs.style.display = 'none';
    const { ticks, errors } = parseTickBlob(state.tickText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        showToast(t('view.footprint.toast.parse_error', { n: errors.length }), { level: 'warning' });
        if (ticks.length === 0) return;
    }
    const err = validateInputs(ticks, state.tickSize);
    if (err) { showErr(err); showToast(t('view.footprint.toast.invalid'), { level: 'warning' }); return; }
    let report;
    try {
        report = await api.microFootprint(buildBody(ticks, state.tickSize));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        showToast(t('view.footprint.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report);
    renderGrid(report);
    renderHotspots(report);
    renderDeltaChart(report);
    renderCumDeltaChart(report);
    const bars = (report && report.bars) ? report.bars.length : 0;
    showToast(t('view.footprint.toast.built', { bars, ticks: ticks.length }), { level: 'success' });
}

function renderDeltaChart(report) {
    const el = document.getElementById('fp-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const bars = (report && report.bars) || [];
    const valid = bars.filter(b => Number.isFinite(Number(b.total_delta)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.footprint.empty_chart">${esc(t('view.footprint.empty_chart'))}</div>`;
        return;
    }
    const labels = valid.map(b => String(b.bar_id));
    const ys = valid.map(b => Number(b.total_delta));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.footprint.chart.bar_idx') },
            { label: t('view.footprint.chart.delta'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.footprint.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function renderCumDeltaChart(report) {
    const el = document.getElementById('fp-cum-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const bars = (report && report.bars) || [];
    const valid = bars.filter(b => Number.isFinite(Number(b.total_delta)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.footprint.empty_cum_chart">${esc(t('view.footprint.empty_cum_chart'))}</div>`;
        return;
    }
    const labels = valid.map(b => String(b.bar_id));
    let acc = 0;
    const cum = valid.map(b => { acc += Number(b.total_delta); return acc; });
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.footprint.chart.bar_idx') },
            { label: t('view.footprint.chart.cum_delta'),
              stroke: '#7af0a8', width: 1.5,
              fill: 'rgba(122,240,168,0.10)',
              points: { show: false } },
            { label: t('view.footprint.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, cum, zero], el);
}

function renderSummary(r) {
    const s = summarize(r);
    document.getElementById('fp-summary').innerHTML = [
        card(t('view.footprint.card.bars'),         String(s.barCount)),
        card(t('view.footprint.card.total_volume'), fmtN(s.totalVolume)),
        card(t('view.footprint.card.net_delta'),    fmtSigned(s.totalDelta), s.totalDelta >= 0 ? 'pos' : 'neg'),
        card(t('view.footprint.card.max_bar_delta'), fmtN(s.maxAbsDelta)),
        card(t('view.footprint.card.last_poc'),     s.lastPoc != null ? fmtPrice(s.lastPoc, state.tickSize) : '—'),
        card(t('view.footprint.card.tick_size'),    fmtN(state.tickSize, 4)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderGrid(report) {
    const wrap = document.getElementById('fp-grid');
    const bars = (report && report.bars) || [];
    if (!bars.length) { wrap.innerHTML = `<div class="muted" data-i18n="view.footprint.empty.bars">No bars.</div>`; return; }
    // Build a UNION price-axis spanning every level seen across all bars
    // so each bar column aligns on the same vertical row grid.
    const priceSet = new Set();
    for (const b of bars) for (const c of (b.cells || [])) priceSet.add(c.price);
    const allPrices = [...priceSet].sort((a, b) => b - a);  // top-down (highest first)

    const headerCells = bars.map(b => `<th>${esc(t('view.footprint.th.bar_n', { id: b.bar_id }))}</th>`).join('');
    const rowsHtml = allPrices.map(p => {
        const cells = bars.map(b => {
            const cell = (b.cells || []).find(c => c.price === p);
            if (!cell) return `<td class="fp-cell fp-empty"></td>`;
            const isPoc = b.poc_price === p;
            const cls = `fp-cell ${deltaCls(cell.delta)} ${isPoc ? 'fp-poc' : ''}`;
            return `<td class="${cls}">
                <span class="fp-bid">${esc(fmtN(cell.bid_volume))}</span>
                <span class="fp-ask">${esc(fmtN(cell.ask_volume))}</span>
                <span class="fp-delta">${esc(fmtSigned(cell.delta))}</span>
            </td>`;
        }).join('');
        return `<tr><th>${esc(fmtPrice(p, state.tickSize))}</th>${cells}</tr>`;
    }).join('');
    const footerCells = bars.map(b => `<th>
        <div class="fp-foot-vol">${esc(t('view.footprint.foot.vol', { vol: fmtN(b.total_volume) }))}</div>
        <div class="fp-foot-delta ${deltaCls(b.total_delta)}">${esc(t('view.footprint.foot.delta', { delta: fmtSigned(b.total_delta) }))}</div>
        <div class="fp-foot-poc">${esc(t('view.footprint.foot.poc', { price: fmtPrice(b.poc_price, state.tickSize) }))}</div>
    </th>`).join('');
    wrap.innerHTML = `
        <table class="fp-table">
            <thead><tr><th data-i18n="view.footprint.th.price">Price</th>${headerCells}</tr></thead>
            <tbody>${rowsHtml}</tbody>
            <tfoot><tr><th></th>${footerCells}</tr></tfoot>
        </table>
    `;
}

function renderHotspots(report) {
    const wrap = document.getElementById('fp-hotspots');
    const hots = imbalanceHotspots(report, 8);
    if (!hots.length) { wrap.innerHTML = `<div class="muted" data-i18n="view.footprint.empty.cells">No cells.</div>`; return; }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th data-i18n="view.footprint.th.bar">Bar</th><th data-i18n="view.footprint.th.price_2">Price</th><th data-i18n="view.footprint.th.bid_vol">Bid vol</th>
                <th data-i18n="view.footprint.th.ask_vol">Ask vol</th><th>Δ</th>
            </tr></thead>
            <tbody>
                ${hots.map((h, i) => `<tr>
                    <td>${i + 1}</td>
                    <td>${h.bar_id}</td>
                    <td>${esc(fmtPrice(h.price, state.tickSize))}</td>
                    <td>${esc(fmtN(h.bid))}</td>
                    <td>${esc(fmtN(h.ask))}</td>
                    <td class="${deltaCls(h.delta)}">${esc(fmtSigned(h.delta))}</td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('fp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('fp-err').style.display = 'none'; }
