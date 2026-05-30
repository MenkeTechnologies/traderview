// Centered Smoothed Momentum (Ehlers) view.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_MOMENTUM, DEFAULT_SMOOTH, MIN_MOMENTUM, MAX_MOMENTUM, MIN_SMOOTH, MAX_SMOOTH,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    momentumBadge, trendBadge, crossBadge, summarizeCloses,
    makeDemoInput,
    fmtNumSigned, fmtPrice, fmtInt,
} from '../_csm_inputs.js';

let state = { ...makeDemoInput('uptrend') };
let chart = null;

export async function renderCsm(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.csm.h1.title" class="view-title">// CENTERED SMOOTHED MOMENTUM</h1>

        <div class="chart-panel" data-context-scope="centered-smoothed-momentum">
            <h2 data-i18n="view.csm.h2.closes">Closes
                <small data-i18n="view.csm.h2.closes_hint" class="muted">(positive prices; ≥ momentum_period + 3)</small></h2>
            <textarea id="cm-blob" rows="6"
                      data-tip="view.csm.tip.closes"
                      placeholder="100, 100.5, 101.2, ...">${esc(closesToBlob(state.closes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.csm.label.momentum">Momentum period</span>
                    <input id="cm-mom" type="number" step="1" min="${MIN_MOMENTUM}" max="${MAX_MOMENTUM}" value="${state.momentum_period}"></label>
                <label><span data-i18n="view.csm.label.smooth">Smooth period</span>
                    <input id="cm-smooth" type="number" step="1" min="${MIN_SMOOTH}" max="${MAX_SMOOTH}" value="${state.smooth_period}"></label>
                <button data-i18n="view.csm.btn.compute" id="cm-run" class="primary"
                        data-tip="view.csm.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.csm.btn.demo_up"     id="cm-d1" class="secondary" type="button">Demo: uptrend</button>
                <button data-i18n="view.csm.btn.demo_down"   id="cm-d2" class="secondary" type="button">Demo: downtrend</button>
                <button data-i18n="view.csm.btn.demo_side"   id="cm-d3" class="secondary" type="button">Demo: sideways</button>
                <button data-i18n="view.csm.btn.demo_rev_up" id="cm-d4" class="secondary" type="button">Demo: reversal up</button>
                <button data-i18n="view.csm.btn.demo_rev_dn" id="cm-d5" class="secondary" type="button">Demo: reversal down</button>
                <button data-i18n="view.csm.btn.demo_osc"    id="cm-d6" class="secondary" type="button">Demo: oscillating</button>
                <button data-i18n="view.csm.btn.demo_short"  id="cm-d7" class="secondary" type="button">Demo: short smooth (4)</button>
                <button data-i18n="view.csm.btn.demo_long"   id="cm-d8" class="secondary" type="button">Demo: long momentum (25)</button>
            </div>
            <p data-i18n="view.csm.hint.about" class="muted">Ehlers' SuperSmoother-filtered momentum: mom_t = close_t − close_{t−momentum_period}; csm_t = SuperSmoother(mom). Centered at zero. Positive = up-momentum, zero-cross signals trend turn. Defaults: momentum=10, smooth=8.</p>
        </div>

        <div id="cm-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.csm.h2.chart">CSM series</h2>
            <div id="cm-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.csm.h2.stats">Closes summary</h2>
            <div id="cm-stats"></div>
        </div>

        <div id="cm-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('cm-blob').value   = closesToBlob(state.closes);
        document.getElementById('cm-mom').value    = state.momentum_period;
        document.getElementById('cm-smooth').value = state.smooth_period;
    };
    document.getElementById('cm-d1').addEventListener('click', () => { loadDemo('uptrend');       void compute(tok); });
    document.getElementById('cm-d2').addEventListener('click', () => { loadDemo('downtrend');     void compute(tok); });
    document.getElementById('cm-d3').addEventListener('click', () => { loadDemo('sideways');      void compute(tok); });
    document.getElementById('cm-d4').addEventListener('click', () => { loadDemo('reversal-up');   void compute(tok); });
    document.getElementById('cm-d5').addEventListener('click', () => { loadDemo('reversal-down'); void compute(tok); });
    document.getElementById('cm-d6').addEventListener('click', () => { loadDemo('oscillating');   void compute(tok); });
    document.getElementById('cm-d7').addEventListener('click', () => { loadDemo('short-smooth');  void compute(tok); });
    document.getElementById('cm-d8').addEventListener('click', () => { loadDemo('long-momentum'); void compute(tok); });
    document.getElementById('cm-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseClosesBlob(document.getElementById('cm-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.csm.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.closes = p.closes;
    const momV = parseInt(document.getElementById('cm-mom').value, 10);
    const smV  = parseInt(document.getElementById('cm-smooth').value, 10);
    state.momentum_period = Number.isInteger(momV) && momV >= MIN_MOMENTUM && momV <= MAX_MOMENTUM ? momV : DEFAULT_MOMENTUM;
    state.smooth_period   = Number.isInteger(smV)  && smV  >= MIN_SMOOTH   && smV  <= MAX_SMOOTH   ? smV  : DEFAULT_SMOOTH;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.closes, state.momentum_period, state.smooth_period);
    renderSummary(local, true);
    renderChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyCenteredSmoothedMomentum(buildBody(state));
    } catch (e) {
        showErr(`${t('view.csm.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!Array.isArray(resp)) { showErr(t('view.csm.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderStats();
}

function renderSummary(csm, pending) {
    const local = localCompute(state.closes, state.momentum_period, state.smooth_period);
    let parityOk = Array.isArray(local) && Array.isArray(csm) && local.length === csm.length;
    if (parityOk) {
        for (let i = 0; i < local.length; i++) {
            const a = local[i], b = csm[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-9) { parityOk = false; break; }
        }
    }
    const last = lastDefined(csm);
    const mBadge = momentumBadge(last);
    const tBadge = trendBadge(csm);
    const xBadge = crossBadge(csm);
    const xValue = xBadge.barsAgo != null ? t('common.ago.bars_paren', { label: t(xBadge.key), n: xBadge.barsAgo }) : t(xBadge.key);
    const populated = countDefined(csm);
    const localTag = pending ? ` (${t('view.csm.tag.local')})` : '';
    document.getElementById('cm-summary').innerHTML = [
        card(t('view.csm.card.momentum'), t(mBadge.key) + localTag, mBadge.cls),
        card(t('view.csm.card.trend'),    t(tBadge.key), tBadge.cls),
        card(t('view.csm.card.cross'),    xValue, xBadge.cls),
        card(t('view.csm.card.last_csm'), fmtNumSigned(last),
             last == null ? '' : last > 1 ? 'pos' : last < -1 ? 'neg' : ''),
        card(t('view.csm.card.momentum_p'), fmtInt(state.momentum_period)),
        card(t('view.csm.card.smooth_p'),   fmtInt(state.smooth_period)),
        card(t('view.csm.card.populated'), `${populated} / ${state.closes.length}`),
        card(t('view.csm.card.parity'),
             parityOk ? t('view.csm.tag.ok') : t('view.csm.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(csm) {
    const el = document.getElementById('cm-chart');
    if (!el || !window.uPlot) return;
    const xs = state.closes.map((_, i) => i);
    const arr = csm.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, arr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    chart = new window.uPlot({
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.csm.series.csm'), stroke: '#1de9b6', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    }, data, el);
}

function renderStats() {
    const wrap = document.getElementById('cm-stats');
    if (!state.closes.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.csm.empty">${esc(t('view.csm.empty'))}</div>`;
        return;
    }
    const s = summarizeCloses(state.closes);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.csm.col.metric">Metric</th>
                <th data-i18n="view.csm.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.csm.row.count">Closes</td><td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.csm.row.last">Last</td>   <td>${esc(fmtPrice(s.last))}</td></tr>
                <tr><td data-i18n="view.csm.row.min">Min</td>     <td>${esc(fmtPrice(s.min))}</td></tr>
                <tr><td data-i18n="view.csm.row.max">Max</td>     <td>${esc(fmtPrice(s.max))}</td></tr>
                <tr><td data-i18n="view.csm.row.mean">Mean</td>   <td>${esc(fmtPrice(s.mean))}</td></tr>
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
    const el = document.getElementById('cm-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cm-err').style.display = 'none'; }
