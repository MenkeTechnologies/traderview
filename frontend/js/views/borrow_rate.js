// Borrow Rate Indicator view — annualized hard-to-borrow stress + change.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD, MIN_PERIOD, MAX_PERIOD, STRESS_LEVELS,
    parseRatesBlob, ratesToBlob, validateInputs, buildBody, localCompute,
    stressBadge, trendBadge, escalationBadge, stressDistribution, summarizeRates,
    makeDemoInput,
    fmtPct, fmtPctSigned, fmtInt,
} from '../_borrow_rate_inputs.js';

let state = { ...makeDemoInput('normal') };
let chart = null;

export async function renderBorrowRate(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.borrow.h1.title" class="view-title">// BORROW RATE INDICATOR</h1>

        <div class="chart-panel" data-context-scope="borrow-rate-indicator">
            <h2 data-i18n="view.borrow.h2.rates">Borrow rates (% annualized)
                <small data-i18n="view.borrow.h2.rates_hint" class="muted">(≥ 0; ≥ period + 1 values)</small></h2>
            <textarea id="br-blob" rows="6"
                      data-tip="view.borrow.tip.rates"
                      placeholder="2.5, 3.0, 2.8, ...">${esc(ratesToBlob(state.rates_pct))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.borrow.label.period">Lookback period</span>
                    <input id="br-period" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"></label>
                <button data-i18n="view.borrow.btn.compute" id="br-run" class="primary"
                        data-tip="view.borrow.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.borrow.btn.demo_norm"  id="br-d1" class="secondary" type="button">Demo: normal</button>
                <button data-i18n="view.borrow.btn.demo_grad"  id="br-d2" class="secondary" type="button">Demo: gradually escalating</button>
                <button data-i18n="view.borrow.btn.demo_spike" id="br-d3" class="secondary" type="button">Demo: sudden spike</button>
                <button data-i18n="view.borrow.btn.demo_ext"   id="br-d4" class="secondary" type="button">Demo: extreme squeeze</button>
                <button data-i18n="view.borrow.btn.demo_easy"  id="br-d5" class="secondary" type="button">Demo: easy borrow</button>
                <button data-i18n="view.borrow.btn.demo_osc"   id="br-d6" class="secondary" type="button">Demo: oscillating</button>
                <button data-i18n="view.borrow.btn.demo_relax" id="br-d7" class="secondary" type="button">Demo: spike → relax</button>
                <button data-i18n="view.borrow.btn.demo_short" id="br-d8" class="secondary" type="button">Demo: short period (2)</button>
            </div>
            <p data-i18n="view.borrow.hint.about" class="muted">Tracks per-bar annualized securities-lending fee + N-bar rate of change. Classification: < 1% Low-Available · 1–10% Normal · 10–50% Tight · 50–200% Hard-to-Borrow · ≥ 200% OR change ≥ 100% Extreme Squeeze. Rising borrow cost flags increasing short demand → squeeze risk.</p>
        </div>

        <div id="br-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.borrow.h2.chart">Rate + change %</h2>
            <div id="br-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.borrow.h2.dist">Stress distribution</h2>
            <div id="br-dist"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.borrow.h2.stats">Series summary</h2>
            <div id="br-stats"></div>
        </div>

        <div id="br-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('br-blob').value   = ratesToBlob(state.rates_pct);
        document.getElementById('br-period').value = state.period;
    };
    document.getElementById('br-d1').addEventListener('click', () => { loadDemo('normal');                void compute(tok); });
    document.getElementById('br-d2').addEventListener('click', () => { loadDemo('gradually-escalating'); void compute(tok); });
    document.getElementById('br-d3').addEventListener('click', () => { loadDemo('sudden-spike');         void compute(tok); });
    document.getElementById('br-d4').addEventListener('click', () => { loadDemo('extreme-squeeze');      void compute(tok); });
    document.getElementById('br-d5').addEventListener('click', () => { loadDemo('easy-borrow');          void compute(tok); });
    document.getElementById('br-d6').addEventListener('click', () => { loadDemo('oscillating');          void compute(tok); });
    document.getElementById('br-d7').addEventListener('click', () => { loadDemo('spike-and-relax');      void compute(tok); });
    document.getElementById('br-d8').addEventListener('click', () => { loadDemo('short-period');         void compute(tok); });
    document.getElementById('br-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseRatesBlob(document.getElementById('br-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.borrow.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.rates_pct = p.rates_pct;
    const periodV = parseInt(document.getElementById('br-period').value, 10);
    state.period = Number.isInteger(periodV) && periodV >= MIN_PERIOD && periodV <= MAX_PERIOD ? periodV : DEFAULT_PERIOD;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.rates_pct, state.period);
    renderSummary(local, true);
    renderChart(local);
    renderDist(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyBorrowRateIndicator(buildBody(state));
    } catch (e) {
        showErr(`${t('view.borrow.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp || !Array.isArray(resp.stress)) { showErr(t('view.borrow.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderDist(resp);
    renderStats();
}

function renderSummary(report, pending) {
    const local = localCompute(state.rates_pct, state.period);
    let parityOk = arraysEqualNum(local.change_pct, report.change_pct)
        && arraysEqualStr(local.stress, report.stress)
        && local.period === report.period;
    const lastRate = state.rates_pct.length ? state.rates_pct[state.rates_pct.length - 1] : NaN;
    const lastChange = lastDefined(report.change_pct);
    const lastStress = lastDefinedStr(report.stress);
    const sBadge = stressBadge(lastStress);
    const tBadge = trendBadge(report.change_pct);
    const eBadge = escalationBadge(report.stress);
    const localTag = pending ? ` (${t('view.borrow.tag.local')})` : '';
    document.getElementById('br-summary').innerHTML = [
        card(t('view.borrow.card.stress'),     t(sBadge.key) + localTag, sBadge.cls),
        card(t('view.borrow.card.trend'),      t(tBadge.key), tBadge.cls),
        card(t('view.borrow.card.escalation'), t(eBadge.key), eBadge.cls),
        card(t('view.borrow.card.last_rate'),  fmtPct(lastRate)),
        card(t('view.borrow.card.last_change'), fmtPctSigned(lastChange),
             lastChange == null ? '' : lastChange >= 10 ? 'neg' : lastChange <= -10 ? 'pos' : ''),
        card(t('view.borrow.card.period'),     fmtInt(state.period)),
        card(t('view.borrow.card.n_obs'),      fmtInt(state.rates_pct.length)),
        card(t('view.borrow.card.parity'),
             parityOk ? t('view.borrow.tag.ok') : t('view.borrow.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    const el = document.getElementById('br-chart');
    if (!el || !window.uPlot) return;
    const xs = state.rates_pct.map((_, i) => i);
    const rates = state.rates_pct;
    const ch = report.change_pct.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, rates, ch];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    chart = new window.uPlot({
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false }, y: {}, yCh: {} },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.borrow.series.rate'),       stroke: '#1de9b6', width: 1.5, scale: 'y' },
            { label: t('view.borrow.series.change_pct'), stroke: '#ff5252', width: 1.2, scale: 'yCh' },
        ],
        axes: [
            { stroke: '#aaa' },
            { stroke: '#1de9b6', scale: 'y' },
            { stroke: '#ff5252', scale: 'yCh', side: 1 },
        ],
        legend: { show: true },
    }, data, el);
}

function renderDist(report) {
    const wrap = document.getElementById('br-dist');
    const dist = stressDistribution(report.stress);
    const total = STRESS_LEVELS.reduce((s, k) => s + dist[k], 0);
    if (total === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.borrow.empty">${esc(t('view.borrow.empty'))}</div>`;
        return;
    }
    const colorFor = (k) => {
        if (k === 'low_available') return 'pos';
        if (k === 'normal') return '';
        return 'neg';
    };
    const rows = STRESS_LEVELS.map(k => `
        <tr>
            <td data-i18n="view.borrow.stress.${k}" class="${colorFor(k)}">${esc(t('view.borrow.stress.' + k))}</td>
            <td>${fmtInt(dist[k])}</td>
            <td>${fmtPct((dist[k] / total) * 100, 1)}</td>
        </tr>`).join('');
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.borrow.col.level">Stress level</th>
                <th data-i18n="view.borrow.col.count">Bars</th>
                <th data-i18n="view.borrow.col.share">Share</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
    `;
}

function renderStats() {
    const wrap = document.getElementById('br-stats');
    if (!state.rates_pct.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.borrow.empty">${esc(t('view.borrow.empty'))}</div>`;
        return;
    }
    const s = summarizeRates(state.rates_pct);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.borrow.col.metric">Metric</th>
                <th data-i18n="view.borrow.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.borrow.row.count">Observations</td><td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.borrow.row.last">Last rate</td>    <td>${esc(fmtPct(s.last))}</td></tr>
                <tr><td data-i18n="view.borrow.row.min">Min rate</td>      <td>${esc(fmtPct(s.min))}</td></tr>
                <tr><td data-i18n="view.borrow.row.max">Max rate</td>      <td>${esc(fmtPct(s.max))}</td></tr>
                <tr><td data-i18n="view.borrow.row.mean">Mean rate</td>    <td>${esc(fmtPct(s.mean))}</td></tr>
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

function arraysEqualNum(a, b) {
    if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
        if (a[i] == null && b[i] == null) continue;
        if (a[i] == null || b[i] == null || Math.abs(a[i] - b[i]) > 1e-6) return false;
    }
    return true;
}

function arraysEqualStr(a, b) {
    if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
        if (a[i] == null && b[i] == null) continue;
        if (a[i] !== b[i]) return false;
    }
    return true;
}

function lastDefined(arr) {
    if (!Array.isArray(arr)) return NaN;
    for (let i = arr.length - 1; i >= 0; i--) {
        if (arr[i] != null && Number.isFinite(arr[i])) return arr[i];
    }
    return NaN;
}

function lastDefinedStr(arr) {
    if (!Array.isArray(arr)) return null;
    for (let i = arr.length - 1; i >= 0; i--) {
        if (arr[i] != null) return arr[i];
    }
    return null;
}

function showErr(msg) {
    const el = document.getElementById('br-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('br-err').style.display = 'none'; }
