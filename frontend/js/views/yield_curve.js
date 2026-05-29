// Yield-curve shape classifier view. UST 3M / 2Y / 5Y / 10Y / 30Y →
// Normal / Flat / Inverted / Humped.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    TENORS, TENOR_YEARS, TENOR_LABELS, DEFAULT_INPUTS,
    validateInputs, buildBody, localClassify,
    shapeBadge, consecutiveSpreads, makeDemoCurve,
    fmtPct, fmtBpsSigned, fmtSpreadPct,
} from '../_yield_curve_inputs.js';

let state = { ...DEFAULT_INPUTS };

export async function renderYieldCurve(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.yield_curve.h1.title" class="view-title">// YIELD CURVE</h1>

        <div class="chart-panel" data-context-scope="yield-curve">
            <h2 data-i18n="view.yield_curve.h2.tenors">UST yields by tenor (decimal — 0.045 = 4.5%)</h2>
            <div class="inline-form">
                ${TENORS.map(k => `
                    <label><span data-i18n="view.yield_curve.label.${k}">${TENOR_LABELS[k]}</span>
                        <input id="yc-${k}" type="number" step="any" value="${state[k]}"></label>
                `).join('')}
                <button data-i18n="view.yield_curve.btn.classify" id="yc-run" class="primary"
                        data-tip="view.yield_curve.tip.classify" type="button">Classify</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.yield_curve.btn.demo_normal"   id="yc-demo-normal"  class="secondary" type="button">Demo: normal</button>
                <button data-i18n="view.yield_curve.btn.demo_inverted" id="yc-demo-inv"     class="secondary" type="button">Demo: inverted</button>
                <button data-i18n="view.yield_curve.btn.demo_flat"     id="yc-demo-flat"    class="secondary" type="button">Demo: flat</button>
                <button data-i18n="view.yield_curve.btn.demo_humped"   id="yc-demo-humped"  class="secondary" type="button">Demo: humped</button>
                <button data-i18n="view.yield_curve.btn.demo_noisy"    id="yc-demo-noisy"   class="secondary" type="button">Demo: noisy-normal (30Y dip)</button>
                <button data-i18n="view.yield_curve.btn.demo_2024"     id="yc-demo-2024"    class="secondary" type="button">Demo: UST end-2024 (inverted)</button>
                <button data-i18n="view.yield_curve.btn.demo_2020"     id="yc-demo-2020"    class="secondary" type="button">Demo: COVID ZIRP (2020)</button>
                <button data-i18n="view.yield_curve.btn.demo_gfc"      id="yc-demo-gfc"     class="secondary" type="button">Demo: late 2008 (post-GFC)</button>
            </div>
            <p data-i18n="view.yield_curve.hint.about" class="muted">Priority: 2Y/10Y inversion → INVERTED (recession). Else humped (5Y peak) → HUMPED. Else all-spreads &lt; 25 bps → FLAT. Else mostly non-decreasing → NORMAL. Otherwise → FLAT (mixed).</p>
        </div>

        <div id="yc-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.yield_curve.h2.chart">Curve shape</h2>
            <div id="yc-chart" style="height:320px"></div>
            <p data-i18n="view.yield_curve.hint.chart" class="muted">X axis: tenor in years. Y axis: yield %. Upward slope = normal expansion. Downward = inversion = recession signal.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.yield_curve.h2.spreads">Consecutive-tenor spreads</h2>
            <div id="yc-spreads"></div>
        </div>

        <div id="yc-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoCurve(k);
        for (const tnr of TENORS) document.getElementById(`yc-${tnr}`).value = state[tnr];
    };
    document.getElementById('yc-demo-normal').addEventListener('click', () => loadDemo('normal'));
    document.getElementById('yc-demo-inv').addEventListener('click',    () => loadDemo('inverted'));
    document.getElementById('yc-demo-flat').addEventListener('click',   () => loadDemo('flat'));
    document.getElementById('yc-demo-humped').addEventListener('click', () => loadDemo('humped'));
    document.getElementById('yc-demo-noisy').addEventListener('click',  () => loadDemo('noisy-normal'));
    document.getElementById('yc-demo-2024').addEventListener('click',   () => loadDemo('ust-2024-inverted'));
    document.getElementById('yc-demo-2020').addEventListener('click',   () => loadDemo('ust-2020-zirp'));
    document.getElementById('yc-demo-gfc').addEventListener('click',    () => loadDemo('gfc-2008-flat'));
    document.getElementById('yc-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function readInputs() {
    const out = {};
    for (const k of TENORS) out[k] = Number(document.getElementById(`yc-${k}`).value);
    state = out;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localClassify(state);
    renderSummary(local, true);
    renderChart();
    renderSpreads();
    let resp;
    try {
        resp = await api.calcYieldCurve(buildBody(state));
    } catch (e) {
        showErr(`${t('view.yield_curve.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart();
    renderSpreads();
}

function renderSummary(report, pending) {
    const badge = shapeBadge(report.shape);
    const local = localClassify(state);
    const parityOk = report.shape === local.shape
                  && Math.abs(report.spread_10y_2y_bps - local.spread_10y_2y_bps) < 1e-6;
    const localTag = pending ? ` (${t('view.yield_curve.tag.local')})` : '';
    // Translate the inverted-note magnitude via local.note_params; falls back
    // to backend's raw `note` string if local didn't supply params.
    const noteText = local.note_params
        ? t(local.note_key, local.note_params)
        : (report.note || t(local.note_key));
    document.getElementById('yc-summary').innerHTML = [
        card(t('view.yield_curve.card.shape'),         t(badge.key) + localTag, badge.cls),
        card(t('view.yield_curve.card.spread_10_2'),   fmtBpsSigned(report.spread_10y_2y_bps),
             report.spread_10y_2y_bps < 0 ? 'neg' : 'pos'),
        card(t('view.yield_curve.card.spread_10_3m'),  fmtBpsSigned(report.spread_10y_3m_bps),
             report.spread_10y_3m_bps < 0 ? 'neg' : 'pos'),
        card(t('view.yield_curve.card.note'),          noteText),
        card(t('view.yield_curve.card.short_end'),     fmtPct(state.y3m)),
        card(t('view.yield_curve.card.long_end'),      fmtPct(state.y30y)),
        card(t('view.yield_curve.card.bull_steepener'),
             state.y10y - state.y3m > 0.015 ? t('view.yield_curve.tag.yes') : t('view.yield_curve.tag.no'),
             state.y10y - state.y3m > 0.015 ? 'pos' : ''),
        card(t('view.yield_curve.card.parity'),
             parityOk ? t('view.yield_curve.tag.ok') : t('view.yield_curve.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart() {
    if (!window.uPlot) return;
    const el = document.getElementById('yc-chart');
    if (!el) return;
    el.innerHTML = '';
    const xs = TENORS.map(k => TENOR_YEARS[k]);
    const ys = TENORS.map(k => state[k]);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 300,
        scales: { x: {}, y: {} },
        series: [
            { label: 'years' },
            { label: 'yield', stroke: '#00e5ff', width: 2,
              fill: '#00e5ff14',
              points: { show: true, size: 8, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => v < 1 ? '3M' : v + 'y') },
            { stroke: '#aab', size: 50,
              values: (_u, splits) => splits.map(v => (v * 100).toFixed(2) + '%') },
        ],
        legend: { show: false },
    }, [xs, ys], el);
}

function renderSpreads() {
    const wrap = document.getElementById('yc-spreads');
    const rows = consecutiveSpreads(state);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.yield_curve.col.from">From</th>
                <th data-i18n="view.yield_curve.col.to">To</th>
                <th data-i18n="view.yield_curve.col.delta">Δ yield</th>
                <th data-i18n="view.yield_curve.col.dir">Direction</th>
            </tr></thead>
            <tbody>
                ${rows.map(r => `<tr>
                    <td data-i18n="view.yield_curve.label.${esc(r.from)}"><strong>${esc(TENOR_LABELS[r.from] || r.from)}</strong></td>
                    <td data-i18n="view.yield_curve.label.${esc(r.to)}"><strong>${esc(TENOR_LABELS[r.to] || r.to)}</strong></td>
                    <td class="${r.delta >= 0 ? 'pos' : 'neg'}">${esc(fmtSpreadPct(r.delta))}</td>
                    <td class="${r.delta > 0 ? 'pos' : r.delta < 0 ? 'neg' : ''}">${r.delta > 0 ? '▲' : r.delta < 0 ? '▼' : '·'}</td>
                </tr>`).join('')}
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
    const el = document.getElementById('yc-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('yc-err').style.display = 'none'; }
