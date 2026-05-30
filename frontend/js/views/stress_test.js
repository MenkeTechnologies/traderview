// Portfolio Stress-Test view — tastytrade Risk Analysis class.
//
// Re-prices an option portfolio under a grid of (price-shock × IV-shock)
// shocks, with optional time-decay advancement. Renders the resulting
// P&L matrix as a heatmap so the trader sees their tail exposure at a
// glance — including the "delta-neutral now but down -$X if IV crushes
// -20%" cases a single-greeks snapshot hides.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseLegBlob, validateInputs, buildBody,
    defaultPriceShocks, defaultIvShocks,
    pivotGrid, heatStyleClass, makeDemoLegs,
    fmtUSD, fmtUSDSigned, fmtPct,
} from '../_stress_test_inputs.js';

import { t } from '../i18n.js';
let state = {
    legText: '',
    priceShocks: defaultPriceShocks(),
    ivShocks: defaultIvShocks(),
    timeDecay: 0,
    rate: 0.045,
    div: 0.0,
};

export async function renderStressTest(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.stress_test.h1.portfolio_stress_test" class="view-title">// PORTFOLIO STRESS TEST</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.stress_test.h2.option_legs">Option legs</h2>
            <p class="muted" data-i18n="view.stress_test.hint.legs">One leg per line: symbol kind spot strike dte iv contracts entry_price. Multiplier is fixed at 100 (equity options). Demo loads a short SPY iron condor.</p>
            <textarea id="st-legs" rows="6" placeholder="SPY put 100 95 30 0.30 -1 1.20&#10;SPY put 100 90 30 0.30 1 0.40&#10;..."></textarea>
            <div class="inline-form">
                <button data-i18n="view.stress_test.btn.load_demo_iron_condor" id="st-demo" class="secondary" type="button">Load demo (iron condor)</button>
                <button data-i18n="view.stress_test.btn.clear" id="st-clear" class="secondary" type="button">Clear</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.stress_test.h2.shock_grid_market_params">Shock grid + market params</h2>
            <div class="inline-form">
                <label><span data-i18n="view.stress_test.label.price_shocks">Price shocks % (comma-sep, fractions e.g. -0.10 = -10%)</span>
                    <input id="st-ps" type="text" value="${state.priceShocks.join(',')}" style="min-width:300px"></label>
                <label><span data-i18n="view.stress_test.label.iv_shocks">IV shocks % (relative to current IV)</span>
                    <input id="st-iv" type="text" value="${state.ivShocks.join(',')}" style="min-width:240px"></label>
            </div>
            <div class="inline-form">
                <label><span data-i18n="view.stress_test.label.time_decay">Time-decay days</span>
                    <input id="st-td" type="number" step="1" min="0" value="${state.timeDecay}"></label>
                <label><span data-i18n="view.stress_test.label.rate">Risk-free rate</span>
                    <input id="st-rate" type="number" step="any" value="${state.rate}"></label>
                <label><span data-i18n="view.stress_test.label.div">Dividend yield</span>
                    <input id="st-div" type="number" step="any" min="0" value="${state.div}"></label>
                <button data-i18n="view.stress_test.btn.run_stress_test" id="st-run" class="primary" type="button">Run stress test</button>
            </div>
            <p data-i18n="view.stress_test.hint.negative_price_shock_downside_negative_iv_shock_vo" class="muted">Negative price-shock = downside; negative IV-shock = vol crush.
                Worst-case + best-case cells highlighted in the heatmap below.</p>
        </div>

        <div id="st-errors" class="boot" style="display:none"></div>
        <div id="st-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.stress_test.h2.pandl_heatmap_price_iv_shocks">P&amp;L heatmap (price × IV shocks)</h2>
            <div id="st-grid" class="st-grid"></div>
            <p data-i18n="view.stress_test.hint.each_cell_portfolio_pandl_under_that_shock_pair_wi" class="muted">Each cell = portfolio P&amp;L under that shock pair (with time-decay
                applied). Gold border = worst-case cell. Cyan border = best-case cell.
                Hover any cell for full greeks under that shock.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.stress_test.h2.curve_chart">P&L vs price shock (per IV shock)</h2>
            <div id="st-chart" style="width:100%;height:240px"></div>
        </div>

        <div id="st-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('st-demo').addEventListener('click', () => {
        const legs = makeDemoLegs();
        document.getElementById('st-legs').value =
            legs.map(l => `${l.symbol} ${l.kind} ${l.spot} ${l.strike} ${l.days_to_expiry} ${l.implied_vol} ${l.contracts} ${l.entry_price}`).join('\n');
    });
    document.getElementById('st-clear').addEventListener('click', () => {
        document.getElementById('st-legs').value = '';
    });
    document.getElementById('st-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.legText = document.getElementById('st-legs').value;
    state.priceShocks = parseFloatList(document.getElementById('st-ps').value);
    state.ivShocks = parseFloatList(document.getElementById('st-iv').value);
    state.timeDecay = Number(document.getElementById('st-td').value);
    state.rate = Number(document.getElementById('st-rate').value);
    state.div = Number(document.getElementById('st-div').value);
}

function parseFloatList(s) {
    return String(s || '').split(/[\s,]+/).filter(Boolean).map(Number).filter(Number.isFinite);
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('st-errors');
    errs.style.display = 'none';
    const { legs, errors } = parseLegBlob(state.legText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (legs.length === 0) return;
    }
    const err = validateInputs(legs, state.priceShocks, state.ivShocks,
        state.timeDecay, state.rate, state.div);
    if (err) { showErr(err); return; }

    let report;
    try {
        report = await api.microStressTest(buildBody(
            legs, state.priceShocks, state.ivShocks,
            state.timeDecay, state.rate, state.div,
        ));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report, legs);
    renderGrid(report);
    renderCurveChart(report);
}

function renderCurveChart(report) {
    const el = document.getElementById('st-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const grid = (report && report.grid) || [];
    if (grid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.stress_test.empty_chart">${esc(t('view.stress_test.empty_chart'))}</div>`;
        return;
    }
    const matrix = pivotGrid(grid, state.priceShocks, state.ivShocks);
    const sortedIdx = state.priceShocks
        .map((v, i) => [v, i])
        .sort((a, b) => a[0] - b[0]);
    const xs = sortedIdx.map(([v]) => v * 100);
    const palette = ['#ff3860', '#ff7a1f', '#ffd84a', '#7af0a8', '#00e5ff', '#b86bff'];
    const series = [{ label: t('view.stress_test.chart.price_shock') }];
    const data = [xs];
    state.ivShocks.forEach((iv, ii) => {
        const ys = sortedIdx.map(([, pi]) => {
            const c = matrix[pi] && matrix[pi][ii];
            return c && Number.isFinite(Number(c.pnl_dollars)) ? Number(c.pnl_dollars) : null;
        });
        series.push({
            label: t('view.stress_test.chart.iv_label', { iv: (iv * 100).toFixed(0) + '%' }),
            stroke: palette[ii % palette.length],
            width: 1.6,
            points: { show: true, size: 6, fill: palette[ii % palette.length], stroke: palette[ii % palette.length] },
        });
        data.push(ys);
    });
    const zero = xs.map(() => 0);
    series.push({ label: t('view.stress_test.chart.zero'), stroke: '#888', width: 1.0, dash: [4, 4], points: { show: false } });
    data.push(zero);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { auto: true }, y: { auto: true } },
        series,
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 60 } ],
        legend: { show: true },
    }, data, el);
}

function renderSummary(report, legs) {
    const w = report.worst_case || {};
    const b = report.best_case || {};
    document.getElementById('st-summary').innerHTML = [
        card(t('view.stress_test.card.legs'),         String(legs.length)),
        card(t('view.stress_test.card.grid_size'),    `${state.priceShocks.length} × ${state.ivShocks.length}`),
        card(t('view.stress_test.card.cells'),        String((report.grid || []).length)),
        card(t('view.stress_test.card.worst_case'),   fmtUSDSigned(w.pnl_dollars),
            (w.pnl_dollars || 0) < 0 ? 'neg' : 'pos'),
        card(t('view.stress_test.card.worst_shock'),  t('view.stress_test.card.shock_value', { px: fmtPct(w.price_shock_pct), iv: fmtPct(w.iv_shock_pct) })),
        card(t('view.stress_test.card.best_case'),    fmtUSDSigned(b.pnl_dollars),
            (b.pnl_dollars || 0) >= 0 ? 'pos' : 'neg'),
        card(t('view.stress_test.card.best_shock'),   t('view.stress_test.card.shock_value', { px: fmtPct(b.price_shock_pct), iv: fmtPct(b.iv_shock_pct) })),
        card(t('view.stress_test.card.time_decay'),   t('view.stress_test.card.time_decay_value', { n: state.timeDecay })),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderGrid(report) {
    const wrap = document.getElementById('st-grid');
    const grid = report.grid || [];
    if (!grid.length) { wrap.innerHTML = `<div class="muted" data-i18n="view.stress_test.empty.cells">No cells.</div>`; return; }
    const matrix = pivotGrid(grid, state.priceShocks, state.ivShocks);
    const maxAbs = Math.max(...grid.map(c => Math.abs(c.pnl_dollars || 0)), 1);
    const worstKey = keyOf(report.worst_case);
    const bestKey  = keyOf(report.best_case);

    // Build header row: blank corner + iv-shock columns.
    let html = `<table class="st-table"><thead><tr><th data-i18n="view.stress_test.th.price_iv">price ↓ / iv →</th>`;
    for (const ivS of state.ivShocks) html += `<th>${esc(fmtPct(ivS))}</th>`;
    html += `</tr></thead><tbody>`;
    // Iterate price shocks top-down (most negative first = worst-case downside row at top).
    const sortedPrice = [...state.priceShocks].sort((a, b) => b - a);
    for (const pS of sortedPrice) {
        const pi = state.priceShocks.indexOf(pS);
        html += `<tr><th>${esc(fmtPct(pS))}</th>`;
        for (let ii = 0; ii < state.ivShocks.length; ii++) {
            const cell = matrix[pi][ii];
            if (!cell) { html += `<td class="st-cell heat-empty"></td>`; continue; }
            const heatCls = heatStyleClass(cell.pnl_dollars, maxAbs);
            const ck = keyOf(cell);
            const flag = ck === worstKey ? 'st-worst' : ck === bestKey ? 'st-best' : '';
            const tip = `price ${fmtPct(cell.price_shock_pct)} · iv ${fmtPct(cell.iv_shock_pct)} · P&L ${fmtUSDSigned(cell.pnl_dollars)} · Δ ${fmtUSDSigned(cell.portfolio_delta_dollars)} · vega ${fmtUSDSigned(cell.portfolio_vega_dollars)} · θ ${fmtUSDSigned(cell.portfolio_theta_dollars)}`;
            html += `<td class="st-cell ${heatCls} ${flag}" title="${esc(tip)}">${esc(fmtUSD(cell.pnl_dollars))}</td>`;
        }
        html += `</tr>`;
    }
    html += `</tbody></table>`;
    wrap.innerHTML = html;
}

function keyOf(cell) {
    if (!cell) return null;
    return `${cell.price_shock_pct}|${cell.iv_shock_pct}`;
}

function showErr(msg) {
    const el = document.getElementById('st-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('st-err').style.display = 'none'; }
