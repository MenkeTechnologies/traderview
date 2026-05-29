// Breusch-Godfrey Serial Correlation LM Test view.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_LAG, MIN_LAG, MAX_LAG,
    parsePairsBlob, pairsToBlob, validateInputs, buildBody, localTest,
    verdictBadge, r2Badge, sampleBadge, summarizeData,
    makeDemoInput,
    fmtNum, fmtPVal, fmtPct, fmtInt,
} from '../_bg_inputs.js';

let state = { ...makeDemoInput('iid-residuals') };
let chart = null;

export async function renderBgTest(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.bg.h1.title" class="view-title">// BREUSCH-GODFREY SERIAL CORRELATION</h1>

        <div class="chart-panel" data-context-scope="breusch-godfrey">
            <h2 data-i18n="view.bg.h2.pairs">Paired (x, y) observations
                <small data-i18n="view.bg.h2.pairs_hint" class="muted">(≥ lag_order + 8 pairs)</small></h2>
            <textarea id="bg-blob" rows="8"
                      data-tip="view.bg.tip.pairs"
                      placeholder="1.0 2.1\n2.0 4.3\n...">${esc(pairsToBlob(state.x, state.y))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.bg.label.lag">Lag order (p)</span>
                    <input id="bg-lag" type="number" step="1" min="${MIN_LAG}" max="${MAX_LAG}" value="${state.lag_order}"></label>
                <button data-i18n="view.bg.btn.compute" id="bg-run" class="primary"
                        data-tip="view.bg.tip.compute" type="button">Test</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.bg.btn.demo_iid"      id="bg-d1" class="secondary" type="button">Demo: iid residuals</button>
                <button data-i18n="view.bg.btn.demo_ar1"      id="bg-d2" class="secondary" type="button">Demo: AR(1) residuals</button>
                <button data-i18n="view.bg.btn.demo_ar2"      id="bg-d3" class="secondary" type="button">Demo: AR(2) residuals</button>
                <button data-i18n="view.bg.btn.demo_mild"     id="bg-d4" class="secondary" type="button">Demo: mild AR(1)</button>
                <button data-i18n="view.bg.btn.demo_cycle"    id="bg-d5" class="secondary" type="button">Demo: cyclical residuals</button>
                <button data-i18n="view.bg.btn.demo_highlag"  id="bg-d6" class="secondary" type="button">Demo: high lag (p=10)</button>
                <button data-i18n="view.bg.btn.demo_short"    id="bg-d7" class="secondary" type="button">Demo: short series</button>
                <button data-i18n="view.bg.btn.demo_pricetn"  id="bg-d8" class="secondary" type="button">Demo: price vs return</button>
            </div>
            <p data-i18n="view.bg.hint.about" class="muted">Tests H₀: OLS residuals have no serial correlation up to lag p. Fits y = α + β·x, regresses ε̂ on (1, x, ε̂_{t-1}…ε̂_{t-p}); LM = n_aux · R² ~ χ²(p). Allows lagged regressors (unlike Durbin-Watson). Reject → use HAC / Newey-West SEs.</p>
        </div>

        <div id="bg-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.bg.h2.chart">x vs y scatter</h2>
            <div id="bg-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bg.h2.stats">Series summary</h2>
            <div id="bg-stats"></div>
        </div>

        <div id="bg-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bg-blob').value = pairsToBlob(state.x, state.y);
        document.getElementById('bg-lag').value  = state.lag_order;
    };
    document.getElementById('bg-d1').addEventListener('click', () => { loadDemo('iid-residuals');     void compute(tok); });
    document.getElementById('bg-d2').addEventListener('click', () => { loadDemo('ar1-residuals');     void compute(tok); });
    document.getElementById('bg-d3').addEventListener('click', () => { loadDemo('ar2-residuals');     void compute(tok); });
    document.getElementById('bg-d4').addEventListener('click', () => { loadDemo('mild-ar1');          void compute(tok); });
    document.getElementById('bg-d5').addEventListener('click', () => { loadDemo('cyclical-residuals'); void compute(tok); });
    document.getElementById('bg-d6').addEventListener('click', () => { loadDemo('high-lag');          void compute(tok); });
    document.getElementById('bg-d7').addEventListener('click', () => { loadDemo('short-series');      void compute(tok); });
    document.getElementById('bg-d8').addEventListener('click', () => { loadDemo('price-vs-return');   void compute(tok); });
    document.getElementById('bg-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parsePairsBlob(document.getElementById('bg-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.bg.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.x = p.x;
    state.y = p.y;
    const lagV = parseInt(document.getElementById('bg-lag').value, 10);
    state.lag_order = Number.isInteger(lagV) && lagV >= MIN_LAG && lagV <= MAX_LAG ? lagV : DEFAULT_LAG;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localTest(state.x, state.y, state.lag_order);
    if (!local) { showErr(t('view.bg.err.degenerate')); return; }
    renderSummary(local, true);
    renderChart();
    renderStats();
    let resp;
    try {
        resp = await api.anlyBreuschGodfrey(buildBody(state));
    } catch (e) {
        showErr(`${t('view.bg.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.bg.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart();
    renderStats();
}

function renderSummary(report, pending) {
    const local = localTest(state.x, state.y, state.lag_order);
    const parityOk = !!local
        && Math.abs(local.lm_statistic - report.lm_statistic) < 1e-6
        && Math.abs(local.p_value - report.p_value) < 1e-6
        && Math.abs(local.r_squared_auxiliary - report.r_squared_auxiliary) < 1e-9
        && local.lag_order === report.lag_order
        && local.n_observations === report.n_observations
        && local.reject_at_5pct === report.reject_at_5pct;
    const vBadge = verdictBadge(report);
    const rBadge = r2Badge(report.r_squared_auxiliary);
    const sBadge = sampleBadge(report);
    const localTag = pending ? ` (${t('view.bg.tag.local')})` : '';
    document.getElementById('bg-summary').innerHTML = [
        card(t('view.bg.card.verdict'),  t(vBadge.key) + localTag, vBadge.cls),
        card(t('view.bg.card.r2'),       t(rBadge.key), rBadge.cls),
        card(t('view.bg.card.sample'),   t(sBadge.key), sBadge.cls),
        card(t('view.bg.card.lm'),       fmtNum(report.lm_statistic)),
        card(t('view.bg.card.p_value'),  fmtPVal(report.p_value),
             report.p_value < 0.05 ? 'neg' : 'pos'),
        card(t('view.bg.card.r2_aux'),   fmtPct(report.r_squared_auxiliary)),
        card(t('view.bg.card.lag_order'), fmtInt(report.lag_order)),
        card(t('view.bg.card.n'),        fmtInt(report.n_observations)),
        card(t('view.bg.card.reject_5'),
             report.reject_at_5pct ? t('view.bg.tag.yes') : t('view.bg.tag.no'),
             report.reject_at_5pct ? 'neg' : 'pos'),
        card(t('view.bg.card.parity'),
             parityOk ? t('view.bg.tag.ok') : t('view.bg.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart() {
    const el = document.getElementById('bg-chart');
    if (!el || !window.uPlot) return;
    const pairs = state.x.map((xi, i) => [xi, state.y[i]]).sort((a, b) => a[0] - b[0]);
    const xs = pairs.map(p => p[0]);
    const ys = pairs.map(p => p[1]);
    const data = [xs, ys];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    chart = new window.uPlot({
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: 'x' },
            { label: t('view.bg.series.y'), stroke: '#1de9b6',
              width: 0, points: { show: true, size: 4, stroke: '#1de9b6', fill: '#1de9b6' } },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    }, data, el);
}

function renderStats() {
    const wrap = document.getElementById('bg-stats');
    if (!state.x.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.bg.empty">${esc(t('view.bg.empty'))}</div>`;
        return;
    }
    const s = summarizeData(state.x, state.y);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.bg.col.metric">Metric</th>
                <th data-i18n="view.bg.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.bg.row.n">Pairs</td>      <td>${fmtInt(s.n)}</td></tr>
                <tr><td data-i18n="view.bg.row.x_mean">x mean</td><td>${esc(fmtNum(s.x_mean))}</td></tr>
                <tr><td data-i18n="view.bg.row.y_mean">y mean</td><td>${esc(fmtNum(s.y_mean))}</td></tr>
                <tr><td data-i18n="view.bg.row.x_sd">x sd</td>    <td>${esc(fmtNum(s.x_sd))}</td></tr>
                <tr><td data-i18n="view.bg.row.y_sd">y sd</td>    <td>${esc(fmtNum(s.y_sd))}</td></tr>
            </tbody>
        </table>
    `;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('bg-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bg-err').style.display = 'none'; }
