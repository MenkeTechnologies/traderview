// Chande Dynamic Momentum Index view — volatility-adapted RSI.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_TD_CONST, DEFAULT_STD_PERIOD, DEFAULT_TD_MIN, DEFAULT_TD_MAX,
    MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    zoneBadge, crossBadge, trendBadge, currentTdInfo, summarizeCloses,
    makeDemoInput,
    fmtNum, fmtPrice, fmtInt,
} from '../_cdmi_inputs.js';

let state = { ...makeDemoInput('uptrend') };
let chart = null;

export async function renderCdmi(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.cdmi.h1.title" class="view-title">// CHANDE DYNAMIC MOMENTUM INDEX</h1>

        <div class="chart-panel" data-context-scope="chande-dynamic-momentum">
            <h2 data-i18n="view.cdmi.h2.closes">Closes
                <small data-i18n="view.cdmi.h2.closes_hint" class="muted">(positive prices; ≥ 2·std_period + td_max bars)</small></h2>
            <textarea id="cd-blob" rows="6"
                      data-tip="view.cdmi.tip.closes"
                      placeholder="100, 100.5, 101.2, ...">${esc(closesToBlob(state.closes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.cdmi.label.td_const">td_const</span>
                    <input id="cd-tdc" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.td_const}"></label>
                <label><span data-i18n="view.cdmi.label.std_period">std_period</span>
                    <input id="cd-sp"  type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.std_period}"></label>
                <label><span data-i18n="view.cdmi.label.td_min">td_min</span>
                    <input id="cd-min" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.td_min}"></label>
                <label><span data-i18n="view.cdmi.label.td_max">td_max</span>
                    <input id="cd-max" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.td_max}"></label>
                <button data-i18n="view.cdmi.btn.compute" id="cd-run" class="primary"
                        data-tip="view.cdmi.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.cdmi.btn.demo_up"     id="cd-d1" class="secondary" type="button">Demo: uptrend</button>
                <button data-i18n="view.cdmi.btn.demo_down"   id="cd-d2" class="secondary" type="button">Demo: downtrend</button>
                <button data-i18n="view.cdmi.btn.demo_quiet"  id="cd-d3" class="secondary" type="button">Demo: quiet market</button>
                <button data-i18n="view.cdmi.btn.demo_vol"    id="cd-d4" class="secondary" type="button">Demo: volatile market</button>
                <button data-i18n="view.cdmi.btn.demo_chop"   id="cd-d5" class="secondary" type="button">Demo: choppy range</button>
                <button data-i18n="view.cdmi.btn.demo_rev_up" id="cd-d6" class="secondary" type="button">Demo: reversal up</button>
                <button data-i18n="view.cdmi.btn.demo_rev_dn" id="cd-d7" class="secondary" type="button">Demo: reversal down</button>
                <button data-i18n="view.cdmi.btn.demo_fixed"  id="cd-d8" class="secondary" type="button">Demo: fixed period (14)</button>
            </div>
            <p data-i18n="view.cdmi.hint.about" class="muted">Volatility-adapted RSI: vi = stdev / avg_stdev; td = round(td_const / vi) clamped to [td_min, td_max]. Quiet markets stretch the RSI period (more smoothing); volatile markets shrink it (more responsiveness). 0..100 range like classic RSI. Defaults: td_const=14, std_period=5, td_min=5, td_max=30.</p>
        </div>

        <div id="cd-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.cdmi.h2.chart">DMI series</h2>
            <div id="cd-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cdmi.h2.stats">Closes summary</h2>
            <div id="cd-stats"></div>
        </div>

        <div id="cd-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('cd-blob').value = closesToBlob(state.closes);
        document.getElementById('cd-tdc').value  = state.td_const;
        document.getElementById('cd-sp').value   = state.std_period;
        document.getElementById('cd-min').value  = state.td_min;
        document.getElementById('cd-max').value  = state.td_max;
    };
    document.getElementById('cd-d1').addEventListener('click', () => { loadDemo('uptrend');         void compute(tok); });
    document.getElementById('cd-d2').addEventListener('click', () => { loadDemo('downtrend');       void compute(tok); });
    document.getElementById('cd-d3').addEventListener('click', () => { loadDemo('quiet-market');    void compute(tok); });
    document.getElementById('cd-d4').addEventListener('click', () => { loadDemo('volatile-market'); void compute(tok); });
    document.getElementById('cd-d5').addEventListener('click', () => { loadDemo('choppy-range');    void compute(tok); });
    document.getElementById('cd-d6').addEventListener('click', () => { loadDemo('reversal-up');     void compute(tok); });
    document.getElementById('cd-d7').addEventListener('click', () => { loadDemo('reversal-down');   void compute(tok); });
    document.getElementById('cd-d8').addEventListener('click', () => { loadDemo('short-bounds');    void compute(tok); });
    document.getElementById('cd-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseClosesBlob(document.getElementById('cd-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.cdmi.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.closes = p.closes;
    const intish = (id, def) => {
        const v = parseInt(document.getElementById(id).value, 10);
        return Number.isInteger(v) && v >= MIN_PERIOD && v <= MAX_PERIOD ? v : def;
    };
    state.td_const   = intish('cd-tdc', DEFAULT_TD_CONST);
    state.std_period = intish('cd-sp',  DEFAULT_STD_PERIOD);
    state.td_min     = intish('cd-min', DEFAULT_TD_MIN);
    state.td_max     = intish('cd-max', DEFAULT_TD_MAX);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.closes, state.td_const, state.std_period, state.td_min, state.td_max);
    renderSummary(local, true);
    renderChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyChandeDynamicMomentum(buildBody(state));
    } catch (e) {
        showErr(`${t('view.cdmi.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!Array.isArray(resp)) { showErr(t('view.cdmi.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderStats();
}

function renderSummary(dmi, pending) {
    const local = localCompute(state.closes, state.td_const, state.std_period, state.td_min, state.td_max);
    let parityOk = Array.isArray(local) && Array.isArray(dmi) && local.length === dmi.length;
    if (parityOk) {
        for (let i = 0; i < local.length; i++) {
            const a = local[i], b = dmi[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-6) { parityOk = false; break; }
        }
    }
    const last = lastDefined(dmi);
    const zBadge = zoneBadge(last);
    const xBadge = crossBadge(dmi);
    const tBadge = trendBadge(dmi);
    const tdInfo = currentTdInfo(state.closes, state.td_const, state.std_period, state.td_min, state.td_max);
    const xValue = xBadge.barsAgo != null ? `${t(xBadge.key)} (${xBadge.barsAgo} bars ago)` : t(xBadge.key);
    const populated = countDefined(dmi);
    const localTag = pending ? ` (${t('view.cdmi.tag.local')})` : '';
    document.getElementById('cd-summary').innerHTML = [
        card(t('view.cdmi.card.zone'),   t(zBadge.key) + localTag, zBadge.cls),
        card(t('view.cdmi.card.cross'),  xValue, xBadge.cls),
        card(t('view.cdmi.card.trend'),  t(tBadge.key), tBadge.cls),
        card(t('view.cdmi.card.last_dmi'), fmtNum(last),
             last == null ? '' : last >= 70 ? 'neg' : last <= 30 ? 'pos' : ''),
        card(t('view.cdmi.card.cur_td'),  fmtInt(tdInfo.td)),
        card(t('view.cdmi.card.cur_vi'),  fmtNum(tdInfo.vi, 3)),
        card(t('view.cdmi.card.td_const'), fmtInt(state.td_const)),
        card(t('view.cdmi.card.td_bounds'), `${state.td_min} – ${state.td_max}`),
        card(t('view.cdmi.card.populated'), `${populated} / ${state.closes.length}`),
        card(t('view.cdmi.card.parity'),
             parityOk ? t('view.cdmi.tag.ok') : t('view.cdmi.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(dmi) {
    const el = document.getElementById('cd-chart');
    if (!el || !window.uPlot) return;
    const xs = state.closes.map((_, i) => i);
    const arr = dmi.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, arr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    chart = new window.uPlot({
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false }, y: { range: [0, 100] } },
        series: [
            { label: 'i' },
            { label: t('view.cdmi.series.dmi'), stroke: '#1de9b6', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    }, data, el);
}

function renderStats() {
    const wrap = document.getElementById('cd-stats');
    if (!state.closes.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.cdmi.empty">${esc(t('view.cdmi.empty'))}</div>`;
        return;
    }
    const s = summarizeCloses(state.closes);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.cdmi.col.metric">Metric</th>
                <th data-i18n="view.cdmi.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.cdmi.row.count">Closes</td><td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.cdmi.row.last">Last</td>   <td>${esc(fmtPrice(s.last))}</td></tr>
                <tr><td data-i18n="view.cdmi.row.min">Min</td>     <td>${esc(fmtPrice(s.min))}</td></tr>
                <tr><td data-i18n="view.cdmi.row.max">Max</td>     <td>${esc(fmtPrice(s.max))}</td></tr>
                <tr><td data-i18n="view.cdmi.row.mean">Mean</td>   <td>${esc(fmtPrice(s.mean))}</td></tr>
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
    const el = document.getElementById('cd-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cd-err').style.display = 'none'; }
