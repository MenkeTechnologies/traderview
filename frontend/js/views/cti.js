// Chande Trend Index (CTI) view.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD, MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    strengthBadge, crossBadge, changeBadge, summarizeCloses,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtPrice, fmtInt,
} from '../_cti_inputs.js';

let state = { ...makeDemoInput('uptrend') };
let chart = null;

export async function renderCti(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.cti.h1.title" class="view-title">// CHANDE TREND INDEX</h1>

        <div class="chart-panel" data-context-scope="chande-trend-index">
            <h2 data-i18n="view.cti.h2.closes">Closes
                <small data-i18n="view.cti.h2.closes_hint" class="muted">(positive prices; ≥ period)</small></h2>
            <textarea id="ci-blob" rows="6"
                      data-tip="view.cti.tip.closes"
                      placeholder="100, 100.5, 101.2, ...">${esc(closesToBlob(state.closes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.cti.label.period">Period</span>
                    <input id="ci-period" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"></label>
                <button data-i18n="view.cti.btn.compute" id="ci-run" class="primary"
                        data-tip="view.cti.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.cti.btn.demo_up"     id="ci-d1" class="secondary" type="button">Demo: uptrend</button>
                <button data-i18n="view.cti.btn.demo_down"   id="ci-d2" class="secondary" type="button">Demo: downtrend</button>
                <button data-i18n="view.cti.btn.demo_flat"   id="ci-d3" class="secondary" type="button">Demo: flat</button>
                <button data-i18n="view.cti.btn.demo_noisy"  id="ci-d4" class="secondary" type="button">Demo: noisy trend</button>
                <button data-i18n="view.cti.btn.demo_osc"    id="ci-d5" class="secondary" type="button">Demo: oscillating</button>
                <button data-i18n="view.cti.btn.demo_rev"    id="ci-d6" class="secondary" type="button">Demo: reversal</button>
                <button data-i18n="view.cti.btn.demo_chop_t" id="ci-d7" class="secondary" type="button">Demo: chop → trend</button>
                <button data-i18n="view.cti.btn.demo_short"  id="ci-d8" class="secondary" type="button">Demo: short period (5)</button>
            </div>
            <p data-i18n="view.cti.hint.about" class="muted">CTI = Pearson correlation of closes vs perfect linear ramp 1..N. Range [−1, +1]: +1 = perfect uptrend, −1 = perfect downtrend, 0 = no trend / random walk. A TREND-STRENGTH metric, distinct from CMO (momentum). Default period=14.</p>
        </div>

        <div id="ci-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.cti.h2.chart">CTI series</h2>
            <div id="ci-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cti.h2.stats">Closes summary</h2>
            <div id="ci-stats"></div>
        </div>

        <div id="ci-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('ci-blob').value   = closesToBlob(state.closes);
        document.getElementById('ci-period').value = state.period;
    };
    document.getElementById('ci-d1').addEventListener('click', () => { loadDemo('uptrend');         void compute(tok); });
    document.getElementById('ci-d2').addEventListener('click', () => { loadDemo('downtrend');       void compute(tok); });
    document.getElementById('ci-d3').addEventListener('click', () => { loadDemo('flat');            void compute(tok); });
    document.getElementById('ci-d4').addEventListener('click', () => { loadDemo('noisy-trend');     void compute(tok); });
    document.getElementById('ci-d5').addEventListener('click', () => { loadDemo('oscillating');     void compute(tok); });
    document.getElementById('ci-d6').addEventListener('click', () => { loadDemo('reversal');        void compute(tok); });
    document.getElementById('ci-d7').addEventListener('click', () => { loadDemo('chop-then-trend'); void compute(tok); });
    document.getElementById('ci-d8').addEventListener('click', () => { loadDemo('short-period');    void compute(tok); });
    document.getElementById('ci-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseClosesBlob(document.getElementById('ci-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.cti.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.closes = p.closes;
    const periodV = parseInt(document.getElementById('ci-period').value, 10);
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
        resp = await api.anlyChandeTrendIndex(buildBody(state));
    } catch (e) {
        showErr(`${t('view.cti.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!Array.isArray(resp)) { showErr(t('view.cti.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderStats();
}

function renderSummary(cti, pending) {
    const local = localCompute(state.closes, state.period);
    let parityOk = Array.isArray(local) && Array.isArray(cti) && local.length === cti.length;
    if (parityOk) {
        for (let i = 0; i < local.length; i++) {
            const a = local[i], b = cti[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-9) { parityOk = false; break; }
        }
    }
    const last = lastDefined(cti);
    const sBadge = strengthBadge(last);
    const xBadge = crossBadge(cti);
    const cBadge = changeBadge(cti);
    const xValue = xBadge.barsAgo != null ? t('common.ago.bars_paren', { label: t(xBadge.key), n: xBadge.barsAgo }) : t(xBadge.key);
    const populated = countDefined(cti);
    const localTag = pending ? ` (${t('view.cti.tag.local')})` : '';
    document.getElementById('ci-summary').innerHTML = [
        card(t('view.cti.card.strength'), t(sBadge.key) + localTag, sBadge.cls),
        card(t('view.cti.card.cross'),    xValue, xBadge.cls),
        card(t('view.cti.card.change'),   t(cBadge.key), cBadge.cls),
        card(t('view.cti.card.last_cti'), fmtNumSigned(last),
             last == null ? '' : last > 0.2 ? 'pos' : last < -0.2 ? 'neg' : ''),
        card(t('view.cti.card.period'),   fmtInt(state.period)),
        card(t('view.cti.card.populated'), `${populated} / ${state.closes.length}`),
        card(t('view.cti.card.parity'),
             parityOk ? t('view.cti.tag.ok') : t('view.cti.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(cti) {
    const el = document.getElementById('ci-chart');
    if (!el || !window.uPlot) return;
    const xs = state.closes.map((_, i) => i);
    const arr = cti.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, arr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    chart = new window.uPlot({
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false }, y: { range: [-1.05, 1.05] } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.cti.series.cti'), stroke: '#1de9b6', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    }, data, el);
}

function renderStats() {
    const wrap = document.getElementById('ci-stats');
    if (!state.closes.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.cti.empty">${esc(t('view.cti.empty'))}</div>`;
        return;
    }
    const s = summarizeCloses(state.closes);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.cti.col.metric">Metric</th>
                <th data-i18n="view.cti.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.cti.row.count">Closes</td><td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.cti.row.last">Last</td>   <td>${esc(fmtPrice(s.last))}</td></tr>
                <tr><td data-i18n="view.cti.row.min">Min</td>     <td>${esc(fmtPrice(s.min))}</td></tr>
                <tr><td data-i18n="view.cti.row.max">Max</td>     <td>${esc(fmtPrice(s.max))}</td></tr>
                <tr><td data-i18n="view.cti.row.mean">Mean</td>   <td>${esc(fmtPrice(s.mean))}</td></tr>
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
    const el = document.getElementById('ci-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ci-err').style.display = 'none'; }
