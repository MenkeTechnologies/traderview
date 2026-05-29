// Choppiness Index view — E.W. Dreiss's trend-vs-consolidation oscillator.
// Range 0–100. > 61.8 = choppy, < 38.2 = trending.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarBlob, validateInputs, buildBody, localCompute,
    regimeBadge, regimeBuckets, lastRegimeSwitch, makeDemoBars,
    fmtN, fmtPct,
} from '../_choppiness_inputs.js';

let state = {
    bars: makeDemoBars('trend-then-chop'),
    period: 14,
};

export async function renderChoppiness(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// CHOPPINESS INDEX</h1>

        <div class="chart-panel">
            <h2>Paste OHLC bars (per line: <code>high low close</code>)</h2>
            <textarea id="cp-blob" rows="6" placeholder="100.5 99.5 100.0&#10;100.6 99.4 100.1&#10;...">${esc(barsToBlob(state.bars))}</textarea>
            <div class="inline-form">
                <label>Lookback period
                    <input id="cp-per" type="number" step="1" min="2" max="200" value="${state.period}"></label>
                <button id="cp-run" class="primary" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button id="cp-demo-trend-up"   class="secondary" type="button">Demo: trending up</button>
                <button id="cp-demo-trend-dn"   class="secondary" type="button">Demo: trending down</button>
                <button id="cp-demo-choppy"     class="secondary" type="button">Demo: choppy</button>
                <button id="cp-demo-mixed"      class="secondary" type="button">Demo: mixed drift</button>
                <button id="cp-demo-switch"     class="secondary" type="button">Demo: trend → chop switch</button>
            </div>
            <p class="muted">Formula: CI = 100 × log10(ΣTR / (max H − min L)) / log10(period). Default period 14. Reference bands: 61.8 (choppy line), 38.2 (trending line).</p>
        </div>

        <div id="cp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Close + Choppiness Index</h2>
            <div id="cp-chart" style="height:380px"></div>
            <p class="muted">Cyan = close (left axis). Yellow = CI (right axis 0–100). Red dashed = 61.8 (chop), green dashed = 38.2 (trend).</p>
        </div>

        <div id="cp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.bars = makeDemoBars(kind);
        document.getElementById('cp-blob').value = barsToBlob(state.bars);
    };
    document.getElementById('cp-demo-trend-up').addEventListener('click', () => loadDemo('trending-up'));
    document.getElementById('cp-demo-trend-dn').addEventListener('click', () => loadDemo('trending-down'));
    document.getElementById('cp-demo-choppy').addEventListener('click',   () => loadDemo('choppy'));
    document.getElementById('cp-demo-mixed').addEventListener('click',    () => loadDemo('mixed'));
    document.getElementById('cp-demo-switch').addEventListener('click',   () => loadDemo('trend-then-chop'));
    document.getElementById('cp-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function barsToBlob(bars) {
    return bars.map(b => `${b.high} ${b.low} ${b.close}`).join('\n');
}

function readInputs() {
    const parsed = parseBarBlob(document.getElementById('cp-blob').value);
    if (parsed.errors.length) {
        showErr(`Parse errors: ${parsed.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; ')}`);
        return;
    }
    hideErr();
    state.bars   = parsed.bars;
    state.period = parseInt(document.getElementById('cp-per').value, 10);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.bars, state.period);
    if (err) { showErr(err); return; }
    const local = localCompute(state.bars, state.period);
    renderSummary(local, true);
    renderChart(state.bars, local);
    let resp;
    try {
        resp = await api.anlyChoppiness(buildBody(state.bars, state.period));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(state.bars, resp);
}

function renderSummary(report, pending) {
    const local = localCompute(state.bars, state.period);
    const parity = reportEq(report, local);
    const badge = regimeBadge(report.regime);
    const buckets = regimeBuckets(report.series);
    const totalEvaluated = buckets.trending + buckets.mixed + buckets.choppy;
    const switchEvt = lastRegimeSwitch(report.series);
    document.getElementById('cp-summary').innerHTML = [
        card('Regime',         badge.label + (pending ? ' (local)' : ''), badge.cls),
        card('Action',         badge.hint),
        card('Latest CI',      report.latest == null ? '—' : fmtN(report.latest, 2),
            badge.cls),
        card('Note',           report.note),
        card('% bars trending', totalEvaluated > 0 ? fmtPct(buckets.trending / totalEvaluated) : '—',
            buckets.trending > buckets.choppy ? 'pos' : ''),
        card('% bars mixed',    totalEvaluated > 0 ? fmtPct(buckets.mixed / totalEvaluated) : '—'),
        card('% bars choppy',   totalEvaluated > 0 ? fmtPct(buckets.choppy / totalEvaluated) : '—',
            buckets.choppy > buckets.trending ? 'neg' : ''),
        card('Warmup bars',    String(buckets.warmup)),
        card('Last switch',    switchEvt
            ? `bar ${switchEvt.switchedAt}: ${switchEvt.fromRegime} → ${switchEvt.toRegime}`
            : 'no switch in window'),
        card('Local parity',   parity ? 'OK' : 'DIVERGED', parity ? 'pos' : 'neg'),
    ].join('');
}

function reportEq(a, b) {
    if (!a || !b) return false;
    if (a.regime !== b.regime) return false;
    const al = a.latest, bl = b.latest;
    if (al == null && bl == null) return true;
    if (al == null || bl == null) return false;
    return Math.abs(al - bl) < 1e-6;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(bars, report) {
    if (!window.uPlot) return;
    const el = document.getElementById('cp-chart');
    if (!el) return;
    el.innerHTML = '';
    const xs = bars.map((_, i) => i);
    const closes = bars.map(b => b.close);
    const ci = report.series.slice();
    const chopBand  = new Array(bars.length).fill(61.8);
    const trendBand = new Array(bars.length).fill(38.2);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 360,
        scales: { x: {}, y: {}, y_ci: { range: [0, 100] } },
        series: [
            { label: 'bar #' },
            { label: 'close',         stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: 'CI',            stroke: '#ffd84a', width: 1.5, scale: 'y_ci',
              points: { show: false } },
            { label: '61.8 (choppy)', stroke: '#ff3860', width: 1.0, dash: [4, 4],
              scale: 'y_ci', points: { show: false } },
            { label: '38.2 (trend)',  stroke: '#23d18b', width: 1.0, dash: [4, 4],
              scale: 'y_ci', points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
            { stroke: '#ffd84a', size: 50, scale: 'y_ci', side: 1 },
        ],
        legend: { show: true },
    }, [xs, closes, ci, chopBand, trendBand], el);
}

function showErr(msg) {
    const el = document.getElementById('cp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cp-err').style.display = 'none'; }
