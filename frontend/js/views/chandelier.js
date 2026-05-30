// Chandelier Exit view — ATR trailing stop hanging from HH/LL with direction flip.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD, DEFAULT_MULTIPLIER, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    dirBadge, flipBadge, distanceBadge, flipStats, summarizeBars,
    makeDemoInput,
    fmtPrice, fmtPriceSigned, fmtPct, fmtInt,
} from '../_chandelier_inputs.js';

let state = { ...makeDemoInput('uptrend') };
let chart = null;
let cushionChart = null;

export async function renderChandelier(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.chx.h1.title" class="view-title">// CHANDELIER EXIT</h1>

        <div class="chart-panel" data-context-scope="chandelier-exit">
            <h2 data-i18n="view.chx.h2.bars">Bars
                <small data-i18n="view.chx.h2.bars_hint" class="muted">(3 tokens per line: high low close; ≥ period + 1 bars)</small></h2>
            <textarea id="cx-blob" rows="6"
                      data-tip="view.chx.tip.bars"
                      placeholder="101 99 100\n102 100 101\n...">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.chx.label.period">Period (ATR/HH/LL)</span>
                    <input id="cx-p" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"></label>
                <label><span data-i18n="view.chx.label.mult">Multiplier (× ATR)</span>
                    <input id="cx-m" type="number" step="0.1" min="0.1" value="${state.multiplier}"></label>
                <button data-i18n="view.chx.btn.compute" id="cx-run" class="primary"
                        data-tip="view.chx.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.chx.btn.demo_up"     id="cx-d1" class="secondary" type="button">Demo: uptrend</button>
                <button data-i18n="view.chx.btn.demo_down"   id="cx-d2" class="secondary" type="button">Demo: downtrend</button>
                <button data-i18n="view.chx.btn.demo_flat"   id="cx-d3" class="secondary" type="button">Demo: flat</button>
                <button data-i18n="view.chx.btn.demo_rev_up" id="cx-d4" class="secondary" type="button">Demo: reversal up</button>
                <button data-i18n="view.chx.btn.demo_rev_dn" id="cx-d5" class="secondary" type="button">Demo: reversal down</button>
                <button data-i18n="view.chx.btn.demo_whip"   id="cx-d6" class="secondary" type="button">Demo: whipsaw</button>
                <button data-i18n="view.chx.btn.demo_tight"  id="cx-d7" class="secondary" type="button">Demo: tight mult (1)</button>
                <button data-i18n="view.chx.btn.demo_wide"   id="cx-d8" class="secondary" type="button">Demo: wide mult (5)</button>
            </div>
            <p data-i18n="view.chx.hint.about" class="muted">ATR trailing stop "hanging" from HH (longs) or LL (shorts) over lookback. Direction flips when close crosses opposite stop; ratchets in favorable direction only. Defaults: period=22, mult=3.0. Companion to Parabolic SAR, Chande-Kroll, Volatility Stop.</p>
        </div>

        <div id="cx-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.chx.h2.chart">Stops overlay</h2>
            <div id="cx-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.chx.h2.cushion_chart">Close-to-stop cushion (% of close) — negative = flip imminent</h2>
            <div id="cx-cushion-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.chx.h2.stats">Bar series summary</h2>
            <div id="cx-stats"></div>
        </div>

        <div id="cx-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('cx-blob').value = barsToBlob(state.bars);
        document.getElementById('cx-p').value    = state.period;
        document.getElementById('cx-m').value    = state.multiplier;
    };
    document.getElementById('cx-d1').addEventListener('click', () => { loadDemo('uptrend');       void compute(tok); });
    document.getElementById('cx-d2').addEventListener('click', () => { loadDemo('downtrend');     void compute(tok); });
    document.getElementById('cx-d3').addEventListener('click', () => { loadDemo('flat');          void compute(tok); });
    document.getElementById('cx-d4').addEventListener('click', () => { loadDemo('reversal-up');   void compute(tok); });
    document.getElementById('cx-d5').addEventListener('click', () => { loadDemo('reversal-down'); void compute(tok); });
    document.getElementById('cx-d6').addEventListener('click', () => { loadDemo('whipsaw');       void compute(tok); });
    document.getElementById('cx-d7').addEventListener('click', () => { loadDemo('tight-mult');    void compute(tok); });
    document.getElementById('cx-d8').addEventListener('click', () => { loadDemo('wide-mult');     void compute(tok); });
    document.getElementById('cx-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('cx-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.chx.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.bars = p.bars;
    const pv = parseInt(document.getElementById('cx-p').value, 10);
    const mv = parseFloat(document.getElementById('cx-m').value);
    state.period     = Number.isInteger(pv) && pv >= MIN_PERIOD && pv <= MAX_PERIOD ? pv : DEFAULT_PERIOD;
    state.multiplier = Number.isFinite(mv) && mv > 0 ? mv : DEFAULT_MULTIPLIER;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.bars, state.period, state.multiplier);
    renderSummary(local, true);
    renderChart(local);
    renderCushionChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyChandelierExit(buildBody(state));
    } catch (e) {
        showErr(`${t('view.chx.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp || !Array.isArray(resp.stop)) { showErr(t('view.chx.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderCushionChart(resp);
    renderStats();
}

function renderSummary(report, pending) {
    const local = localCompute(state.bars, state.period, state.multiplier);
    let parityOk = Array.isArray(local.stop) && Array.isArray(report.stop)
        && local.stop.length === report.stop.length
        && local.direction.length === report.direction.length;
    if (parityOk) {
        for (let i = 0; i < local.stop.length; i++) {
            for (const key of ['stop', 'long_stop', 'short_stop']) {
                const a = local[key][i], b = report[key][i];
                if (a == null && b == null) continue;
                if (a == null || b == null || Math.abs(a - b) > 1e-9) { parityOk = false; break; }
            }
            if (!parityOk) break;
            if (local.direction[i] !== report.direction[i]) { parityOk = false; break; }
        }
    }
    const lastClose = state.bars.length ? state.bars[state.bars.length - 1].close : NaN;
    const lastStop = lastDefined(report.stop);
    const lastDir  = lastDefinedStr(report.direction);
    const dBadge = dirBadge(lastDir);
    const fBadge = flipBadge(report.direction);
    const distInfo = distanceBadge(lastStop, lastClose);
    const stats = flipStats(report.direction);
    const fValue = fBadge.barsAgo != null ? t('common.ago.bars_paren', { label: t(fBadge.key), n: fBadge.barsAgo }) : t(fBadge.key);
    const localTag = pending ? ` (${t('view.chx.tag.local')})` : '';
    document.getElementById('cx-summary').innerHTML = [
        card(t('view.chx.card.dir'),       t(dBadge.key) + localTag, dBadge.cls),
        card(t('view.chx.card.flip'),      fValue, fBadge.cls),
        card(t('view.chx.card.distance'),  t(distInfo.key), distInfo.cls),
        card(t('view.chx.card.last_close'), fmtPrice(lastClose)),
        card(t('view.chx.card.last_stop'), fmtPrice(lastStop)),
        card(t('view.chx.card.gap'),       fmtPriceSigned(distInfo.distance),
             distInfo.distance > 0 ? 'pos' : distInfo.distance < 0 ? 'neg' : ''),
        card(t('view.chx.card.gap_pct'),   fmtPct(distInfo.distance_pct)),
        card(t('view.chx.card.flip_count'), fmtInt(stats.flips)),
        card(t('view.chx.card.long_bars'), fmtInt(stats.long_bars),  'pos'),
        card(t('view.chx.card.short_bars'), fmtInt(stats.short_bars), 'neg'),
        card(t('view.chx.card.period'),    fmtInt(state.period)),
        card(t('view.chx.card.mult'),      fmtPrice(state.multiplier, 2)),
        card(t('view.chx.card.parity'),
             parityOk ? t('view.chx.tag.ok') : t('view.chx.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    const el = document.getElementById('cx-chart');
    if (!el || !window.uPlot) return;
    const xs = state.bars.map((_, i) => i);
    const closes = state.bars.map(b => b.close);
    const stopArr = report.stop.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const longArr = report.stop.map((v, i) =>
        (v == null || !Number.isFinite(v)) ? null
        : (report.direction[i] === 'long'  ? v : null));
    const shortArr = report.stop.map((v, i) =>
        (v == null || !Number.isFinite(v)) ? null
        : (report.direction[i] === 'short' ? v : null));
    const data = [xs, closes, longArr, shortArr, stopArr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    chart = new window.uPlot({
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.chx.series.close'),       stroke: '#888',     width: 1 },
            { label: t('view.chx.series.long_stop'),   stroke: '#1de9b6', width: 1.8 },
            { label: t('view.chx.series.short_stop'),  stroke: '#ff5252', width: 1.8 },
            { label: t('view.chx.series.active_stop'), stroke: '#ffd54f', width: 1, dash: [4, 4] },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    }, data, el);
}

function renderCushionChart(report) {
    const el = document.getElementById('cx-cushion-chart');
    if (!el || !window.uPlot) return;
    if (!state.bars || state.bars.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.chx.empty_cushion_chart">${esc(t('view.chx.empty_cushion_chart'))}</div>`;
        return;
    }
    const xs = state.bars.map((_, i) => i);
    const longCushion = state.bars.map((b, i) => {
        const s = report.stop[i];
        if (s == null || !Number.isFinite(s) || !b.close || report.direction[i] !== 'long') return null;
        return ((b.close - s) / b.close) * 100;
    });
    const shortCushion = state.bars.map((b, i) => {
        const s = report.stop[i];
        if (s == null || !Number.isFinite(s) || !b.close || report.direction[i] !== 'short') return null;
        return ((s - b.close) / b.close) * 100;
    });
    const zero = xs.map(() => 0);
    if (cushionChart) { try { cushionChart.destroy(); } catch {} cushionChart = null; }
    cushionChart = new window.uPlot({
        width: el.clientWidth || 800, height: 220,
        scales: { x: { time: false }, y: { auto: true } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.chx.series.long_cushion'),
              stroke: '#7af0a8', width: 1.5,
              points: { show: false } },
            { label: t('view.chx.series.short_cushion'),
              stroke: '#ff3860', width: 1.5,
              points: { show: false } },
            { label: t('view.chx.series.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    }, [xs, longCushion, shortCushion, zero], el);
}

function renderStats() {
    const wrap = document.getElementById('cx-stats');
    if (!state.bars.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.chx.empty">${esc(t('view.chx.empty'))}</div>`;
        return;
    }
    const s = summarizeBars(state.bars);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.chx.col.metric">Metric</th>
                <th data-i18n="view.chx.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.chx.row.count">Bars</td>      <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.chx.row.last">Last close</td> <td>${esc(fmtPrice(s.last_close))}</td></tr>
                <tr><td data-i18n="view.chx.row.minl">Min low</td>    <td>${esc(fmtPrice(s.min_low))}</td></tr>
                <tr><td data-i18n="view.chx.row.maxh">Max high</td>   <td>${esc(fmtPrice(s.max_high))}</td></tr>
                <tr><td data-i18n="view.chx.row.mean">Mean close</td> <td>${esc(fmtPrice(s.mean_close))}</td></tr>
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

function lastDefinedStr(arr) {
    if (!Array.isArray(arr)) return null;
    for (let i = arr.length - 1; i >= 0; i--) {
        if (arr[i] != null) return arr[i];
    }
    return null;
}

function showErr(msg) {
    const el = document.getElementById('cx-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cx-err').style.display = 'none'; }
