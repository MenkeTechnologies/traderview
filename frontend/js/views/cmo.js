// Chande Momentum Oscillator (CMO) view.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD, MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    zoneBadge, crossBadge, trendBadge, summarizeCloses,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtPrice, fmtInt,
} from '../_cmo_inputs.js';

let state = { ...makeDemoInput('uptrend') };
let chart = null;

export async function renderCmo(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.cmo.h1.title" class="view-title">// CHANDE MOMENTUM OSCILLATOR</h1>

        <div class="chart-panel" data-context-scope="chande-momentum-oscillator">
            <h2 data-i18n="view.cmo.h2.closes">Closes
                <small data-i18n="view.cmo.h2.closes_hint" class="muted">(positive prices; ≥ period + 1)</small></h2>
            <textarea id="cm-blob" rows="6"
                      data-tip="view.cmo.tip.closes"
                      placeholder="100, 100.5, 101.2, ...">${esc(closesToBlob(state.closes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.cmo.label.period">Period</span>
                    <input id="cm-period" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"></label>
                <button data-i18n="view.cmo.btn.compute" id="cm-run" class="primary"
                        data-tip="view.cmo.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.cmo.btn.demo_up"     id="cm-d1" class="secondary" type="button">Demo: uptrend</button>
                <button data-i18n="view.cmo.btn.demo_down"   id="cm-d2" class="secondary" type="button">Demo: downtrend</button>
                <button data-i18n="view.cmo.btn.demo_flat"   id="cm-d3" class="secondary" type="button">Demo: flat</button>
                <button data-i18n="view.cmo.btn.demo_alt"    id="cm-d4" class="secondary" type="button">Demo: alternating</button>
                <button data-i18n="view.cmo.btn.demo_osc"    id="cm-d5" class="secondary" type="button">Demo: oscillating</button>
                <button data-i18n="view.cmo.btn.demo_rev_up" id="cm-d6" class="secondary" type="button">Demo: reversal up</button>
                <button data-i18n="view.cmo.btn.demo_rev_dn" id="cm-d7" class="secondary" type="button">Demo: reversal down</button>
                <button data-i18n="view.cmo.btn.demo_short"  id="cm-d8" class="secondary" type="button">Demo: short period (5)</button>
            </div>
            <p data-i18n="view.cmo.hint.about" class="muted">Unsmoothed RSI variant: CMO = 100 × (Σ ups − Σ downs) / (Σ ups + Σ downs). Range [−100, +100]. > +50 = strong upside (overbought); < −50 = strong downside (oversold). Responds faster than RSI to single large bars. Default period=14.</p>
        </div>

        <div id="cm-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.cmo.h2.chart">CMO series</h2>
            <div id="cm-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cmo.h2.stats">Closes summary</h2>
            <div id="cm-stats"></div>
        </div>

        <div id="cm-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('cm-blob').value   = closesToBlob(state.closes);
        document.getElementById('cm-period').value = state.period;
    };
    document.getElementById('cm-d1').addEventListener('click', () => { loadDemo('uptrend');       void compute(tok); });
    document.getElementById('cm-d2').addEventListener('click', () => { loadDemo('downtrend');     void compute(tok); });
    document.getElementById('cm-d3').addEventListener('click', () => { loadDemo('flat');          void compute(tok); });
    document.getElementById('cm-d4').addEventListener('click', () => { loadDemo('alternating');   void compute(tok); });
    document.getElementById('cm-d5').addEventListener('click', () => { loadDemo('oscillating');   void compute(tok); });
    document.getElementById('cm-d6').addEventListener('click', () => { loadDemo('reversal-up');   void compute(tok); });
    document.getElementById('cm-d7').addEventListener('click', () => { loadDemo('reversal-down'); void compute(tok); });
    document.getElementById('cm-d8').addEventListener('click', () => { loadDemo('short-period');  void compute(tok); });
    document.getElementById('cm-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseClosesBlob(document.getElementById('cm-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.cmo.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.closes = p.closes;
    const periodV = parseInt(document.getElementById('cm-period').value, 10);
    state.period = Number.isInteger(periodV) && periodV >= MIN_PERIOD && periodV <= MAX_PERIOD ? periodV : DEFAULT_PERIOD;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.closes, state.period);
    renderSummary(local, true);
    renderChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyChandeMomentumOscillator(buildBody(state));
    } catch (e) {
        showErr(`${t('view.cmo.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!Array.isArray(resp)) { showErr(t('view.cmo.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderStats();
}

function renderSummary(cmo, pending) {
    const local = localCompute(state.closes, state.period);
    let parityOk = Array.isArray(local) && Array.isArray(cmo) && local.length === cmo.length;
    if (parityOk) {
        for (let i = 0; i < local.length; i++) {
            const a = local[i], b = cmo[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-9) { parityOk = false; break; }
        }
    }
    const last = lastDefined(cmo);
    const zBadge = zoneBadge(last);
    const xBadge = crossBadge(cmo);
    const tBadge = trendBadge(cmo);
    const xValue = xBadge.barsAgo != null ? t('common.ago.bars_paren', { label: t(xBadge.key), n: xBadge.barsAgo }) : t(xBadge.key);
    const populated = countDefined(cmo);
    const localTag = pending ? ` (${t('view.cmo.tag.local')})` : '';
    document.getElementById('cm-summary').innerHTML = [
        card(t('view.cmo.card.zone'),   t(zBadge.key) + localTag, zBadge.cls),
        card(t('view.cmo.card.cross'),  xValue, xBadge.cls),
        card(t('view.cmo.card.trend'),  t(tBadge.key), tBadge.cls),
        card(t('view.cmo.card.last_cmo'), fmtNumSigned(last),
             last == null ? '' : last > 20 ? 'pos' : last < -20 ? 'neg' : ''),
        card(t('view.cmo.card.period'), fmtInt(state.period)),
        card(t('view.cmo.card.populated'), `${populated} / ${state.closes.length}`),
        card(t('view.cmo.card.parity'),
             parityOk ? t('view.cmo.tag.ok') : t('view.cmo.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(cmo) {
    const el = document.getElementById('cm-chart');
    if (!el || !window.uPlot) return;
    const xs = state.closes.map((_, i) => i);
    const arr = cmo.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, arr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    chart = new window.uPlot({
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false }, y: { range: [-100, 100] } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.cmo.series.cmo'), stroke: '#1de9b6', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    }, data, el);
}

function renderStats() {
    const wrap = document.getElementById('cm-stats');
    if (!state.closes.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.cmo.empty">${esc(t('view.cmo.empty'))}</div>`;
        return;
    }
    const s = summarizeCloses(state.closes);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.cmo.col.metric">Metric</th>
                <th data-i18n="view.cmo.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.cmo.row.count">Closes</td><td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.cmo.row.last">Last</td>   <td>${esc(fmtPrice(s.last))}</td></tr>
                <tr><td data-i18n="view.cmo.row.min">Min</td>     <td>${esc(fmtPrice(s.min))}</td></tr>
                <tr><td data-i18n="view.cmo.row.max">Max</td>     <td>${esc(fmtPrice(s.max))}</td></tr>
                <tr><td data-i18n="view.cmo.row.mean">Mean</td>   <td>${esc(fmtPrice(s.mean))}</td></tr>
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
