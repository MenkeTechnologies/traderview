// Breusch-Pagan Heteroskedasticity Test view.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parsePairsBlob, pairsToBlob,
    validateInputs, buildBody, localTest,
    verdictBadge, r2Badge, sampleBadge, summarizeData,
    makeDemoInput,
    fmtNum, fmtPVal, fmtPct, fmtInt,
} from '../_bp_inputs.js';

let state = { ...makeDemoInput('homoskedastic') };
let chart = null;

export async function renderBpTest(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.bp.h1.title" class="view-title">// BREUSCH-PAGAN HETEROSKEDASTICITY</h1>

        <div class="chart-panel" data-context-scope="breusch-pagan">
            <h2 data-i18n="view.bp.h2.pairs">Paired (x, y) observations
                <small data-i18n="view.bp.h2.pairs_hint" class="muted">(2 tokens per line; ≥ 10 pairs)</small></h2>
            <textarea id="bp-blob" rows="8"
                      data-tip="view.bp.tip.pairs"
                      placeholder="1.0 2.1\n2.0 4.3\n...">${esc(pairsToBlob(state.x, state.y))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.bp.btn.compute" id="bp-run" class="primary"
                        data-tip="view.bp.tip.compute" type="button">Test</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.bp.btn.demo_hom"   id="bp-d1" class="secondary" type="button">Demo: homoskedastic</button>
                <button data-i18n="view.bp.btn.demo_var_up" id="bp-d2" class="secondary" type="button">Demo: variance ↑ x</button>
                <button data-i18n="view.bp.btn.demo_var_dn" id="bp-d3" class="secondary" type="button">Demo: variance ↓ x</button>
                <button data-i18n="view.bp.btn.demo_v"     id="bp-d4" class="secondary" type="button">Demo: V-shape variance</button>
                <button data-i18n="view.bp.btn.demo_n_w"   id="bp-d5" class="secondary" type="button">Demo: narrow → wide</button>
                <button data-i18n="view.bp.btn.demo_small" id="bp-d6" class="secondary" type="button">Demo: small sample (n=12)</button>
                <button data-i18n="view.bp.btn.demo_returns" id="bp-d7" class="secondary" type="button">Demo: returns vs price</button>
                <button data-i18n="view.bp.btn.demo_spike" id="bp-d8" class="secondary" type="button">Demo: spike residuals</button>
            </div>
            <p data-i18n="view.bp.hint.about" class="muted">Tests H₀: OLS residual variance is independent of x. Fits y = α + β·x, then regresses ê² on x. LM = n · R²_aux ~ χ²(1) under H₀. p &lt; 0.05 → reject homoskedasticity → use White / HC standard errors. Univariate predictor only.</p>
        </div>

        <div id="bp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.bp.h2.chart">x vs y scatter</h2>
            <div id="bp-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bp.h2.stats">Series summary</h2>
            <div id="bp-stats"></div>
        </div>

        <div id="bp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bp-blob').value = pairsToBlob(state.x, state.y);
    };
    document.getElementById('bp-d1').addEventListener('click', () => { loadDemo('homoskedastic');             void compute(tok); });
    document.getElementById('bp-d2').addEventListener('click', () => { loadDemo('variance-increasing-in-x'); void compute(tok); });
    document.getElementById('bp-d3').addEventListener('click', () => { loadDemo('variance-decreasing-in-x'); void compute(tok); });
    document.getElementById('bp-d4').addEventListener('click', () => { loadDemo('v-shape-variance');         void compute(tok); });
    document.getElementById('bp-d5').addEventListener('click', () => { loadDemo('narrow-then-wide');         void compute(tok); });
    document.getElementById('bp-d6').addEventListener('click', () => { loadDemo('small-sample');             void compute(tok); });
    document.getElementById('bp-d7').addEventListener('click', () => { loadDemo('returns-vs-vol');           void compute(tok); });
    document.getElementById('bp-d8').addEventListener('click', () => { loadDemo('extreme-spike-residuals'); void compute(tok); });
    document.getElementById('bp-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parsePairsBlob(document.getElementById('bp-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.bp.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.x = p.x;
    state.y = p.y;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localTest(state.x, state.y);
    if (!local) { showErr(t('view.bp.err.degenerate')); return; }
    renderSummary(local, true);
    renderChart();
    renderStats();
    let resp;
    try {
        resp = await api.anlyBreuschPagan(buildBody(state));
    } catch (e) {
        showErr(`${t('view.bp.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.bp.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart();
    renderStats();
}

function renderSummary(report, pending) {
    const local = localTest(state.x, state.y);
    const parityOk = !!local
        && Math.abs(local.lm_statistic - report.lm_statistic) < 1e-6
        && Math.abs(local.p_value - report.p_value) < 1e-6
        && Math.abs(local.r_squared_auxiliary - report.r_squared_auxiliary) < 1e-9
        && local.n_observations === report.n_observations
        && local.reject_at_5pct === report.reject_at_5pct
        && local.reject_at_1pct === report.reject_at_1pct;
    const vBadge = verdictBadge(report);
    const rBadge = r2Badge(report.r_squared_auxiliary);
    const sBadge = sampleBadge(report.n_observations);
    const localTag = pending ? ` (${t('view.bp.tag.local')})` : '';
    document.getElementById('bp-summary').innerHTML = [
        card(t('view.bp.card.verdict'),  t(vBadge.key) + localTag, vBadge.cls),
        card(t('view.bp.card.r2'),       t(rBadge.key), rBadge.cls),
        card(t('view.bp.card.sample'),   t(sBadge.key), sBadge.cls),
        card(t('view.bp.card.lm'),       fmtNum(report.lm_statistic)),
        card(t('view.bp.card.p_value'),  fmtPVal(report.p_value),
             report.p_value < 0.05 ? 'neg' : 'pos'),
        card(t('view.bp.card.r2_aux'),   fmtPct(report.r_squared_auxiliary)),
        card(t('view.bp.card.reject_5'),
             report.reject_at_5pct ? t('view.bp.tag.yes') : t('view.bp.tag.no'),
             report.reject_at_5pct ? 'neg' : 'pos'),
        card(t('view.bp.card.reject_1'),
             report.reject_at_1pct ? t('view.bp.tag.yes') : t('view.bp.tag.no'),
             report.reject_at_1pct ? 'neg' : 'pos'),
        card(t('view.bp.card.n'),        fmtInt(report.n_observations)),
        card(t('view.bp.card.parity'),
             parityOk ? t('view.bp.tag.ok') : t('view.bp.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart() {
    const el = document.getElementById('bp-chart');
    if (!el || !window.uPlot) return;
    // Sort pairs by x for stable line + scatter overlay.
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
            { label: t('view.bp.series.y'), stroke: '#1de9b6',
              width: 0, points: { show: true, size: 4, stroke: '#1de9b6', fill: '#1de9b6' } },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    }, data, el);
}

function renderStats() {
    const wrap = document.getElementById('bp-stats');
    if (!state.x.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.bp.empty">${esc(t('view.bp.empty'))}</div>`;
        return;
    }
    const s = summarizeData(state.x, state.y);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.bp.col.metric">Metric</th>
                <th data-i18n="view.bp.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.bp.row.n">Pairs</td>      <td>${fmtInt(s.n)}</td></tr>
                <tr><td data-i18n="view.bp.row.x_mean">x mean</td><td>${esc(fmtNum(s.x_mean))}</td></tr>
                <tr><td data-i18n="view.bp.row.y_mean">y mean</td><td>${esc(fmtNum(s.y_mean))}</td></tr>
                <tr><td data-i18n="view.bp.row.x_sd">x sd</td>    <td>${esc(fmtNum(s.x_sd))}</td></tr>
                <tr><td data-i18n="view.bp.row.y_sd">y sd</td>    <td>${esc(fmtNum(s.y_sd))}</td></tr>
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
    const el = document.getElementById('bp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bp-err').style.display = 'none'; }
