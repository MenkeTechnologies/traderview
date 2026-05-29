// Volatility Smile fitter — paste strike/IV pairs, fit a Gatheral SVI
// curve, plot raw points + fitted curve, surface SVI params + ATM skew
// + arbitrage-violation flag.
//
// Backend: `/analytics/svi-volatility-smile` (gradient-free coordinate
// descent on the 5 raw SVI params: a, b, ρ, m, σ).
//
// Workflow:
//   1. Paste 5+ rows of `strike  iv` (whitespace OR comma separated).
//   2. Enter spot, expiry (years), rate, dividend yield.
//   3. Click Fit. The view computes log-moneyness + total-variance,
//      sends to backend, plots both raw and fitted IV vs strike.

import { api } from '../api.js';
import { esc, fmt, fmtPct } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { applyUiI18n } from '../i18n.js';
import {
    parseStrikeIvText, buildSviBody, validateSmileInputs,
    sortRowsByStrike, atmSkewSlope,
} from '../_vol_smile_inputs.js';

const DEFAULT_BLOB = `# Paste your chain here. One row per quote:
#   strike  iv     (whitespace or comma separated; iv as 0.25 OR 25%)
# Example: an SPY 30-day chain at spot ≈ 100, with a typical equity skew.
90   30%
95   27%
100  25%
105  24%
110  24.5%
115  26%
120  29%
`;

let state = {
    text: DEFAULT_BLOB,
    spot: 100,
    t_years: 30 / 365.0,
    rate: 0.05,
    div_yield: 0.0,
};

export async function renderVolSmile(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.vol_smile.h1.vol_smile" class="view-title">// VOL SMILE</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.vol_smile.h2.inputs">Inputs</h2>
            <div class="inline-form">
                <label><span data-i18n="view.vol_smile.label.spot">Spot</span>
                    <input id="vs-spot" type="number" step="any" value="${state.spot}"></label>
                <label><span data-i18n="view.vol_smile.label.t_years">Expiry (years)</span>
                    <input id="vs-t" type="number" step="any" value="${state.t_years}"></label>
                <label><span data-i18n="view.vol_smile.label.rate">Rate</span>
                    <input id="vs-rate" type="number" step="any" value="${state.rate}"></label>
                <label><span data-i18n="view.vol_smile.label.div_yield">Div yield</span>
                    <input id="vs-q" type="number" step="any" value="${state.div_yield}"></label>
                <button data-i18n="view.vol_smile.btn.fit" id="vs-fit" class="primary" type="button">Fit</button>
            </div>
            <p class="muted">
                Paste quotes below — one per line, two whitespace-OR-comma
                separated fields: <code>strike  iv</code>. IV accepts
                <code>0.25</code> or <code>25%</code>. Lines starting
                with <code>#</code> are skipped.
            </p>
            <textarea id="vs-text" rows="10" style="width:100%;font-family:monospace;font-size:13px">${esc(state.text)}</textarea>
        </div>

        <div id="vs-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="vs-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.vol_smile.h2.smile">Smile</h2>
            <div id="vs-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.vol_smile.hint.solid_svi_fitted_curve_markers_raw_paste" class="muted">Solid = SVI fitted curve · Markers = raw paste</p>
        </div>

        <div id="vs-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    wireForm(mount, tok);
    await fit(mount, tok);
    void fmt; void fmtPct;
}

function wireForm(mount, tok) {
    document.getElementById('vs-fit').addEventListener('click', () => {
        state.spot = Number(document.getElementById('vs-spot').value);
        state.t_years = Number(document.getElementById('vs-t').value);
        state.rate = Number(document.getElementById('vs-rate').value);
        state.div_yield = Number(document.getElementById('vs-q').value);
        state.text = document.getElementById('vs-text').value;
        void fit(mount, tok);
    });
}

async function fit(mount, tok) {
    hideErr();
    const { rows, errors } = parseStrikeIvText(state.text);
    if (errors.length) showParseErrors(errors);
    const sorted = sortRowsByStrike(rows);

    const validation = validateSmileInputs(sorted, state.spot, state.t_years);
    if (validation) { showErr(validation); return; }

    const body = buildSviBody(sorted, state.spot, state.rate, state.div_yield, state.t_years);
    let res;
    try {
        res = await api.anlySviVolatilitySmile(body);
        if (!res) throw new Error('SVI fit returned null (input violated constraints)');
    } catch (e) {
        showErr(`API error: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(res, state.t_years);
    renderChart(sorted, res);
}

function renderSummary(res, tYears) {
    const skew = atmSkewSlope(
        { a: res.a, b: res.b, rho: res.rho, m: res.m, sigma: res.sigma },
        tYears,
    );
    const arbCell = res.arbitrage_ok
        ? '<span class="pos">no-arb OK</span>'
        : '<span class="neg">violated</span>';
    const vsSummary = document.getElementById('vs-summary');
    vsSummary.innerHTML = `
        <div class="card"><div class="label"><span data-i18n="view.vol_smile.card.a">a</span> (level)</div>
            <div class="value">${formatN(res.a, 6)}</div></div>
        <div class="card"><div class="label"><span data-i18n="view.vol_smile.card.b">b</span> (scale)</div>
            <div class="value">${formatN(res.b, 4)}</div></div>
        <div class="card"><div class="label">ρ (<span data-i18n="view.vol_smile.card.skew">skew</span>)</div>
            <div class="value ${res.rho < 0 ? 'neg' : 'pos'}">${formatN(res.rho, 4)}</div></div>
        <div class="card"><div class="label"><span data-i18n="view.vol_smile.card.m">m</span> (center)</div>
            <div class="value">${formatN(res.m, 4)}</div></div>
        <div class="card"><div class="label">σ (<span data-i18n="view.vol_smile.card.smoothness">smoothness</span>)</div>
            <div class="value">${formatN(res.sigma, 4)}</div></div>
        <div class="card"><div class="label" data-i18n="view.vol_smile.card.rmse_total_var">RMSE (total var)</div>
            <div class="value">${formatN(res.rmse_total_var, 6)}</div></div>
        <div class="card"><div class="label">∂σ/∂k <span data-i18n="view.vol_smile.card.at_atm">at ATM</span></div>
            <div class="value ${skew < 0 ? 'neg' : 'pos'}">${formatN(skew, 4)}</div></div>
        <div class="card"><div class="label" data-i18n="view.vol_smile.card.arbitrage">Arbitrage</div>
            <div class="value">${arbCell}</div></div>
    `;
    try { applyUiI18n(vsSummary); } catch (_) {}
}

function renderChart(rows, res) {
    const el = document.getElementById('vs-chart');
    if (!window.uPlot) {
        el.textContent = 'uPlot not loaded — run scripts/vendor-uplot.sh';
        return;
    }
    el.innerHTML = '';

    // X-axis = strike. Raw rows + fitted IVs share the same x order
    // because both the rows and `res.fitted_iv` are aligned: the backend
    // returns one fitted IV per input strike in submission order.
    const xs = rows.map(r => r.strike);
    const rawY = rows.map(r => r.iv);
    const fitY = res.fitted_iv;

    const w = el.clientWidth || 800;
    const h = 340;

    // Markers for raw (drawn as a transparent stroke with point markers).
    new window.uPlot({
        title: '',
        width: w,
        height: h,
        scales: { x: {}, y: {} },
        series: [
            { label: 'strike' },
            {
                label: 'IV (raw)',
                stroke: '#ff9f1a',
                width: 0,
                points: { show: true, size: 8, stroke: '#ff9f1a', fill: '#ff9f1a' },
            },
            {
                label: 'IV (SVI fit)',
                stroke: '#00e5ff',
                width: 2,
            },
        ],
        axes: [{ stroke: '#aab' }, {
            stroke: '#aab',
            values: (_, ticks) => ticks.map(t => `${(t * 100).toFixed(1)}%`),
        }],
    }, [xs, rawY, fitY], el);
}

function showParseErrors(errors) {
    const el = document.getElementById('vs-parse-errors');
    el.innerHTML = errors.map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw)}</code></span></div>`
    ).join('');
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('vs-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() {
    document.getElementById('vs-parse-errors').style.display = 'none';
    document.getElementById('vs-err').style.display = 'none';
}

function formatN(x, digits) {
    if (!Number.isFinite(x)) return '—';
    return x.toFixed(digits);
}
