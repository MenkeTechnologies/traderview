// Equivolume bars view — Richard Arms classification + volume-proportional
// width allocation.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    summarize, convictionBadge, lastBadge,
    makeDemoInput,
    fmtUSD, fmtNum, fmtInt, fmtVol, kindLabelKey, kindCls,
} from '../_equivolume_inputs.js';

let state = { ...makeDemoInput('mixed-kinds') };

export async function renderEquivolume(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.equivol.h1.title" class="view-title">// EQUIVOLUME BARS</h1>

        <div class="chart-panel" data-context-scope="equivolume">
            <h2 data-i18n="view.equivol.h2.bars">Bars
                <small data-i18n="view.equivol.h2.bars_hint" class="muted">(per line: high low volume)</small></h2>
            <textarea id="ev-blob" rows="6"
                      data-tip="view.equivol.tip.bars"
                      placeholder="101 99 1000&#10;102 100 1200&#10;115 95 5000">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.equivol.label.total_width">Total chart width</span>
                    <input id="ev-width" type="number" step="any" min="0" value="${state.total_width}"></label>
                <button data-i18n="view.equivol.btn.compute" id="ev-run" class="primary"
                        data-tip="view.equivol.tip.compute" type="button">Compute widths</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.equivol.btn.demo_mix"     id="ev-demo-mix"     class="secondary" type="button">Demo: mixed kinds (all 4)</button>
                <button data-i18n="view.equivol.btn.demo_normal"  id="ev-demo-normal"  class="secondary" type="button">Demo: normal mix</button>
                <button data-i18n="view.equivol.btn.demo_power"   id="ev-demo-power"   class="secondary" type="button">Demo: power spike</button>
                <button data-i18n="view.equivol.btn.demo_wide"    id="ev-demo-wide"    class="secondary" type="button">Demo: wide only (no range)</button>
                <button data-i18n="view.equivol.btn.demo_narrow"  id="ev-demo-narrow"  class="secondary" type="button">Demo: narrow spike</button>
                <button data-i18n="view.equivol.btn.demo_flat"    id="ev-demo-flat"    class="secondary" type="button">Demo: flat volume</button>
                <button data-i18n="view.equivol.btn.demo_climax"  id="ev-demo-climax"  class="secondary" type="button">Demo: climax day</button>
                <button data-i18n="view.equivol.btn.demo_noisy"   id="ev-demo-noisy"   class="secondary" type="button">Demo: noisy walk (50 bars)</button>
            </div>
            <p data-i18n="view.equivol.hint.about" class="muted">width_i = volume_i / Σvol × total_width. Tags: Narrow (≤0.5× avg vol) · Wide (&gt;1.5× avg vol) · Power (wide AND range &gt;1.5× avg range = strong conviction). Σ widths = total_width.</p>
        </div>

        <div id="ev-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.equivol.h2.chart">Cumulative bar widths (each bar plotted at its center vs price midpoint)</h2>
            <div id="ev-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.equivol.h2.table">Per-bar classification</h2>
            <div id="ev-table"></div>
        </div>

        <div id="ev-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('ev-blob').value  = barsToBlob(state.bars);
        document.getElementById('ev-width').value = state.total_width;
    };
    document.getElementById('ev-demo-mix').addEventListener('click',    () => { loadDemo('mixed-kinds');  void compute(tok); });
    document.getElementById('ev-demo-normal').addEventListener('click', () => { loadDemo('normal-mix');   void compute(tok); });
    document.getElementById('ev-demo-power').addEventListener('click',  () => { loadDemo('power-spike');  void compute(tok); });
    document.getElementById('ev-demo-wide').addEventListener('click',   () => { loadDemo('wide-only');    void compute(tok); });
    document.getElementById('ev-demo-narrow').addEventListener('click', () => { loadDemo('narrow-spike'); void compute(tok); });
    document.getElementById('ev-demo-flat').addEventListener('click',   () => { loadDemo('flat-volume');  void compute(tok); });
    document.getElementById('ev-demo-climax').addEventListener('click', () => { loadDemo('climax-day');   void compute(tok); });
    document.getElementById('ev-demo-noisy').addEventListener('click',  () => { loadDemo('noisy-walk');   void compute(tok); });
    document.getElementById('ev-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('ev-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.equivol.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.bars = p.bars;
    const w = Number(document.getElementById('ev-width').value);
    state.total_width = Number.isFinite(w) && w > 0 ? w : 1000;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.bars, state.total_width);
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.chartsEquivolumeBars(buildBody(state));
    } catch (e) {
        showErr(`${t('view.equivol.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
}

function renderSummary(report, pending) {
    const local = localCompute(state.bars, state.total_width);
    const parityOk = report.widths.length === local.widths.length
        && report.widths.every((w, i) => Math.abs(w - local.widths[i]) < 1e-6)
        && report.kinds.every((k, i) => k === local.kinds[i]);
    const s = summarize(report);
    const cBadge = convictionBadge(s);
    const lBadge = lastBadge(report.kinds.length > 0 ? report.kinds[report.kinds.length - 1] : null);
    const localTag = pending ? ` (${t('view.equivol.tag.local')})` : '';
    const widthSum = report.widths.reduce((s, w) => s + w, 0);
    const sumOk = state.bars.length === 0 || Math.abs(widthSum - state.total_width) < 1e-6;
    document.getElementById('ev-summary').innerHTML = [
        card(t('view.equivol.card.verdict'),     t(cBadge.key) + localTag, cBadge.cls),
        card(t('view.equivol.card.last_kind'),   t(lBadge.key), lBadge.cls),
        card(t('view.equivol.card.bars'),        fmtInt(s.count)),
        card(t('view.equivol.card.narrow'),      fmtInt(s.narrow)),
        card(t('view.equivol.card.normal'),      fmtInt(s.normal)),
        card(t('view.equivol.card.wide'),        fmtInt(s.wide), s.wide > 0 ? 'neg' : ''),
        card(t('view.equivol.card.power'),       fmtInt(s.power), s.power > 0 ? 'neg' : ''),
        card(t('view.equivol.card.avg_vol'),     fmtVol(report.avg_volume)),
        card(t('view.equivol.card.avg_range'),   fmtUSD(report.avg_range)),
        card(t('view.equivol.card.max_width'),   fmtNum(s.max_width, 2)),
        card(t('view.equivol.card.min_width'),   fmtNum(s.min_width, 2)),
        card(t('view.equivol.card.sum_check'),
             sumOk ? t('view.equivol.tag.ok') : t('view.equivol.tag.diverged'),
             sumOk ? 'pos' : 'neg'),
        card(t('view.equivol.card.parity'),
             parityOk ? t('view.equivol.tag.ok') : t('view.equivol.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('ev-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!state.bars || state.bars.length === 0 || !report.widths || report.widths.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.equivol.empty">${esc(t('view.equivol.empty'))}</div>`;
        return;
    }
    // X = cumulative bar-center on the width axis; Y = bar midprice.
    let cursor = 0;
    const xs = [];
    const mids = [];
    const highs = [];
    const lows = [];
    for (let i = 0; i < state.bars.length; i++) {
        const w = report.widths[i];
        const center = cursor + w / 2;
        xs.push(center);
        const b = state.bars[i];
        mids.push((b.high + b.low) / 2);
        highs.push(b.high);
        lows.push(b.low);
        cursor += w;
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: 'cum width' },
            { label: 'mid',  stroke: '#00e5ff', width: 1.5, points: { show: true, size: 5 } },
            { label: t('chart.series.high'), stroke: '#3ad96b', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('chart.series.low'),  stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => v.toFixed(0)) },
            { stroke: '#aab', size: 70 },
        ],
        legend: { show: true },
    }, [xs, mids, highs, lows], el);
}

function renderTable(report) {
    const wrap = document.getElementById('ev-table');
    if (!state.bars || state.bars.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.equivol.empty">${esc(t('view.equivol.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.equivol.col.idx">#</th>
                <th data-i18n="view.equivol.col.high">High</th>
                <th data-i18n="view.equivol.col.low">Low</th>
                <th data-i18n="view.equivol.col.range">Range</th>
                <th data-i18n="view.equivol.col.volume">Volume</th>
                <th data-i18n="view.equivol.col.width">Width</th>
                <th data-i18n="view.equivol.col.kind">Kind</th>
            </tr></thead>
            <tbody>
                ${state.bars.map((b, i) => {
                    const k = report.kinds[i];
                    const cls = kindCls(k);
                    return `<tr>
                        <td>${i + 1}</td>
                        <td>${esc(fmtUSD(b.high))}</td>
                        <td>${esc(fmtUSD(b.low))}</td>
                        <td>${esc(fmtUSD(b.high - b.low))}</td>
                        <td>${esc(fmtVol(b.volume))}</td>
                        <td>${esc(fmtNum(report.widths[i], 2))}</td>
                        <td data-i18n="${esc(kindLabelKey(k))}" class="${cls}">${esc(t(kindLabelKey(k)))}</td>
                    </tr>`;
                }).join('')}
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
    const el = document.getElementById('ev-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ev-err').style.display = 'none'; }
