// Bollinger Bandwidth Percentile (BBWP) view.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_BB_PERIOD, DEFAULT_N_STDEV, DEFAULT_LOOKBACK,
    MIN_BB_PERIOD, MAX_BB_PERIOD, MIN_LOOKBACK, MAX_LOOKBACK,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    regimeBadge, trendBadge, triggerBadge, summarizeCloses,
    makeDemoInput,
    fmtPrice, fmtPct, fmtNum, fmtInt,
} from '../_bbwp_inputs.js';

let state = { ...makeDemoInput('rising-vol') };
let chart = null;

export async function renderBbwp(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.bbwp.h1.title" class="view-title">// BOLLINGER BANDWIDTH PERCENTILE</h1>

        <div class="chart-panel" data-context-scope="bollinger-bandwidth-percentile">
            <h2 data-i18n="view.bbwp.h2.closes">Closes
                <small data-i18n="view.bbwp.h2.closes_hint" class="muted">(positive prices; ≥ lookback bars; whitespace/comma)</small></h2>
            <textarea id="bp-blob" rows="6"
                      data-tip="view.bbwp.tip.closes"
                      placeholder="100.5, 100.8, 101.2, ...">${esc(closesToBlob(state.closes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.bbwp.label.bb_period">BB period</span>
                    <input id="bp-period" type="number" step="1" min="${MIN_BB_PERIOD}" max="${MAX_BB_PERIOD}" value="${state.bb_period}"></label>
                <label><span data-i18n="view.bbwp.label.n_stdev">n_stdev</span>
                    <input id="bp-stdev" type="number" step="0.1" min="0.1" value="${state.n_stdev}"></label>
                <label><span data-i18n="view.bbwp.label.lookback">Lookback</span>
                    <input id="bp-lookback" type="number" step="1" min="${MIN_LOOKBACK}" max="${MAX_LOOKBACK}" value="${state.lookback}"></label>
                <button data-i18n="view.bbwp.btn.compute" id="bp-run" class="primary"
                        data-tip="view.bbwp.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.bbwp.btn.demo_rise"  id="bp-d1" class="secondary" type="button">Demo: rising vol</button>
                <button data-i18n="view.bbwp.btn.demo_sqz"   id="bp-d2" class="secondary" type="button">Demo: squeeze at end</button>
                <button data-i18n="view.bbwp.btn.demo_osc"   id="bp-d3" class="secondary" type="button">Demo: vol cycles</button>
                <button data-i18n="view.bbwp.btn.demo_steady" id="bp-d4" class="secondary" type="button">Demo: steady walk</button>
                <button data-i18n="view.bbwp.btn.demo_flat"  id="bp-d5" class="secondary" type="button">Demo: flat market</button>
                <button data-i18n="view.bbwp.btn.demo_short" id="bp-d6" class="secondary" type="button">Demo: short lookback (60)</button>
                <button data-i18n="view.bbwp.btn.demo_kshigh" id="bp-d7" class="secondary" type="button">Demo: n_stdev = 3</button>
                <button data-i18n="view.bbwp.btn.demo_spike" id="bp-d8" class="secondary" type="button">Demo: spike → mean revert</button>
            </div>
            <p data-i18n="view.bbwp.hint.about" class="muted">BBWP = percent rank of BBW within the rolling lookback window. Range [0, 100]. < 10 = compression/squeeze (energy building); > 90 = expansion (vol top). Defaults: BB period=20, n_stdev=2.0, lookback=252.</p>
        </div>

        <div id="bp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbwp.h2.chart">BBWP series</h2>
            <div id="bp-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbwp.h2.stats">Closes summary</h2>
            <div id="bp-stats"></div>
        </div>

        <div id="bp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bp-blob').value     = closesToBlob(state.closes);
        document.getElementById('bp-period').value   = state.bb_period;
        document.getElementById('bp-stdev').value    = state.n_stdev;
        document.getElementById('bp-lookback').value = state.lookback;
    };
    document.getElementById('bp-d1').addEventListener('click', () => { loadDemo('rising-vol');             void compute(tok); });
    document.getElementById('bp-d2').addEventListener('click', () => { loadDemo('squeeze-end');            void compute(tok); });
    document.getElementById('bp-d3').addEventListener('click', () => { loadDemo('oscillating');            void compute(tok); });
    document.getElementById('bp-d4').addEventListener('click', () => { loadDemo('steady');                 void compute(tok); });
    document.getElementById('bp-d5').addEventListener('click', () => { loadDemo('flat');                   void compute(tok); });
    document.getElementById('bp-d6').addEventListener('click', () => { loadDemo('short-lookback');         void compute(tok); });
    document.getElementById('bp-d7').addEventListener('click', () => { loadDemo('high-stdev');             void compute(tok); });
    document.getElementById('bp-d8').addEventListener('click', () => { loadDemo('spike-and-mean-revert'); void compute(tok); });
    document.getElementById('bp-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseClosesBlob(document.getElementById('bp-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.bbwp.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.closes = p.closes;
    const periodV   = parseInt(document.getElementById('bp-period').value, 10);
    const stdevV    = parseFloat(document.getElementById('bp-stdev').value);
    const lookbackV = parseInt(document.getElementById('bp-lookback').value, 10);
    state.bb_period = Number.isInteger(periodV) && periodV >= MIN_BB_PERIOD && periodV <= MAX_BB_PERIOD ? periodV : DEFAULT_BB_PERIOD;
    state.n_stdev   = Number.isFinite(stdevV) && stdevV > 0 ? stdevV : DEFAULT_N_STDEV;
    state.lookback  = Number.isInteger(lookbackV) && lookbackV >= MIN_LOOKBACK && lookbackV <= MAX_LOOKBACK ? lookbackV : DEFAULT_LOOKBACK;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.closes, state.bb_period, state.n_stdev, state.lookback);
    renderSummary(local, true);
    renderChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyBollingerBandwidthPercentile(buildBody(state));
    } catch (e) {
        showErr(`${t('view.bbwp.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!Array.isArray(resp)) { showErr(t('view.bbwp.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderStats();
}

function renderSummary(bbwp, pending) {
    const local = localCompute(state.closes, state.bb_period, state.n_stdev, state.lookback);
    let parityOk = Array.isArray(local) && Array.isArray(bbwp) && local.length === bbwp.length;
    if (parityOk) {
        for (let i = 0; i < local.length; i++) {
            const a = local[i], b = bbwp[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-6) { parityOk = false; break; }
        }
    }
    const last = lastDefined(bbwp);
    const populated = countDefined(bbwp);
    const rBadge = regimeBadge(last);
    const tBadge = trendBadge(bbwp);
    const trBadge = triggerBadge(bbwp);
    const localTag = pending ? ` (${t('view.bbwp.tag.local')})` : '';
    document.getElementById('bp-summary').innerHTML = [
        card(t('view.bbwp.card.regime'),   t(rBadge.key) + localTag, rBadge.cls),
        card(t('view.bbwp.card.trend'),    t(tBadge.key), tBadge.cls),
        card(t('view.bbwp.card.trigger'),  t(trBadge.key), trBadge.cls),
        card(t('view.bbwp.card.last_bbwp'), fmtPct(last),
             last == null ? '' : last >= 90 ? 'neg' : last <= 10 ? 'pos' : ''),
        card(t('view.bbwp.card.bb_period'), fmtInt(state.bb_period)),
        card(t('view.bbwp.card.n_stdev'),   fmtNum(state.n_stdev, 2)),
        card(t('view.bbwp.card.lookback'),  fmtInt(state.lookback)),
        card(t('view.bbwp.card.populated'), `${populated} / ${state.closes.length}`),
        card(t('view.bbwp.card.parity'),
             parityOk ? t('view.bbwp.tag.ok') : t('view.bbwp.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(bbwp) {
    const el = document.getElementById('bp-chart');
    if (!el || !window.uPlot) return;
    const xs = state.closes.map((_, i) => i);
    const arr = bbwp.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, arr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    const opts = {
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false }, y: { range: [0, 100] } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.bbwp.series.bbwp'), stroke: '#1de9b6', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    chart = new window.uPlot(opts, data, el);
}

function renderStats() {
    const wrap = document.getElementById('bp-stats');
    if (!state.closes.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.bbwp.empty">${esc(t('view.bbwp.empty'))}</div>`;
        return;
    }
    const s = summarizeCloses(state.closes);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.bbwp.col.metric">Metric</th>
                <th data-i18n="view.bbwp.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.bbwp.row.count">Closes</td><td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.bbwp.row.last">Last</td>   <td>${esc(fmtPrice(s.last))}</td></tr>
                <tr><td data-i18n="view.bbwp.row.min">Min</td>     <td>${esc(fmtPrice(s.min))}</td></tr>
                <tr><td data-i18n="view.bbwp.row.max">Max</td>     <td>${esc(fmtPrice(s.max))}</td></tr>
                <tr><td data-i18n="view.bbwp.row.mean">Mean</td>   <td>${esc(fmtPrice(s.mean))}</td></tr>
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

function lastDefined(arr) {
    if (!Array.isArray(arr)) return NaN;
    for (let i = arr.length - 1; i >= 0; i--) {
        if (arr[i] != null && Number.isFinite(arr[i])) return arr[i];
    }
    return NaN;
}

function countDefined(arr) {
    if (!Array.isArray(arr)) return 0;
    let n = 0;
    for (const v of arr) if (v != null && Number.isFinite(v)) n++;
    return n;
}

function showErr(msg) {
    const el = document.getElementById('bp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bp-err').style.display = 'none'; }
