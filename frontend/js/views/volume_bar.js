// Volume-bar chart view — aggregates trade prints into fixed-volume
// OHLC bars.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parsePrintsBlob, printsToBlob, validateInputs, buildBody, localCompute,
    trendBadge, coverageBadge, summarize,
    makeDemoInput,
    fmtUSD, fmtMove, fmtNum, fmtInt, fmtVol,
} from '../_volume_bar_inputs.js';

let state = { ...makeDemoInput('uptrend-large') };

export async function renderVolumeBar(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.vol_bar.h1.title" class="view-title">// VOLUME BAR CHART</h1>

        <div class="chart-panel" data-context-scope="vol-bar">
            <h2 data-i18n="view.vol_bar.h2.prints">Trade prints
                <small data-i18n="view.vol_bar.h2.prints_hint" class="muted">(per line: price size)</small></h2>
            <textarea id="vb-blob" rows="6"
                      data-tip="view.vol_bar.tip.prints"
                      placeholder="100.05 200&#10;100.06 250&#10;100.10 300">${esc(printsToBlob(state.prints))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.vol_bar.label.volume">Volume per bar</span>
                    <input id="vb-target" type="number" step="any" min="0" value="${state.volume_per_bar}"></label>
                <button data-i18n="view.vol_bar.btn.compute" id="vb-run" class="primary"
                        data-tip="view.vol_bar.tip.compute" type="button">Build bars</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.vol_bar.btn.demo_up"      id="vb-demo-up"      class="secondary" type="button">Demo: uptrend (large vol)</button>
                <button data-i18n="view.vol_bar.btn.demo_down"    id="vb-demo-down"    class="secondary" type="button">Demo: downtrend (large vol)</button>
                <button data-i18n="view.vol_bar.btn.demo_flat"    id="vb-demo-flat"    class="secondary" type="button">Demo: flat volume</button>
                <button data-i18n="view.vol_bar.btn.demo_spiky"   id="vb-demo-spiky"   class="secondary" type="button">Demo: spiky volume</button>
                <button data-i18n="view.vol_bar.btn.demo_tiny"    id="vb-demo-tiny"    class="secondary" type="button">Demo: tiny target (50)</button>
                <button data-i18n="view.vol_bar.btn.demo_huge"    id="vb-demo-huge"    class="secondary" type="button">Demo: huge target (5000)</button>
                <button data-i18n="view.vol_bar.btn.demo_partial" id="vb-demo-partial" class="secondary" type="button">Demo: partial trail (dropped)</button>
                <button data-i18n="view.vol_bar.btn.demo_noisy"   id="vb-demo-noisy"   class="secondary" type="button">Demo: noisy walk (200 prints)</button>
            </div>
            <p data-i18n="view.vol_bar.hint.about" class="muted">Each bar accumulates prints until cumulative volume ≥ volume_per_bar, then closes. Time is ignored. Useful in futures where calendar bars contain wildly different volumes (open vs lunch lull). Trailing partial bars are not emitted.</p>
        </div>

        <div id="vb-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.vol_bar.h2.chart">OHLC closes by bar #</h2>
            <div id="vb-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.vol_bar.h2.table">Bars (tail — last 30)</h2>
            <div id="vb-table"></div>
        </div>

        <div id="vb-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('vb-blob').value   = printsToBlob(state.prints);
        document.getElementById('vb-target').value = state.volume_per_bar;
    };
    document.getElementById('vb-demo-up').addEventListener('click',      () => { loadDemo('uptrend-large');   void compute(tok); });
    document.getElementById('vb-demo-down').addEventListener('click',    () => { loadDemo('downtrend-large'); void compute(tok); });
    document.getElementById('vb-demo-flat').addEventListener('click',    () => { loadDemo('flat-volume');     void compute(tok); });
    document.getElementById('vb-demo-spiky').addEventListener('click',   () => { loadDemo('spiky-volume');    void compute(tok); });
    document.getElementById('vb-demo-tiny').addEventListener('click',    () => { loadDemo('tiny-target');     void compute(tok); });
    document.getElementById('vb-demo-huge').addEventListener('click',    () => { loadDemo('huge-target');     void compute(tok); });
    document.getElementById('vb-demo-partial').addEventListener('click', () => { loadDemo('partial-trail');   void compute(tok); });
    document.getElementById('vb-demo-noisy').addEventListener('click',   () => { loadDemo('noisy-walk');      void compute(tok); });
    document.getElementById('vb-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parsePrintsBlob(document.getElementById('vb-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.vol_bar.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.prints = p.prints;
    const v = Number(document.getElementById('vb-target').value);
    state.volume_per_bar = Number.isFinite(v) && v > 0 ? v : 1000;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.prints, state.volume_per_bar);
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.chartsVolumeBar(buildBody(state));
    } catch (e) {
        showErr(`${t('view.vol_bar.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
}

function renderSummary(bars, pending) {
    const local = localCompute(state.prints, state.volume_per_bar);
    const parityOk = bars.length === local.length
        && bars.every((b, i) => Math.abs(b.close - local[i].close) < 1e-9
            && Math.abs(b.volume - local[i].volume) < 1e-9
            && b.tick_count === local[i].tick_count);
    const totalVol = state.prints.reduce((s, p) => s + p.size, 0);
    const tBadge = trendBadge(bars);
    const cBadge = coverageBadge(bars, totalVol, state.volume_per_bar);
    const s = summarize(bars);
    const localTag = pending ? ` (${t('view.vol_bar.tag.local')})` : '';
    const droppedVol = totalVol - s.total_volume;
    document.getElementById('vb-summary').innerHTML = [
        card(t('view.vol_bar.card.verdict'),     t(tBadge.key) + localTag, tBadge.cls),
        card(t('view.vol_bar.card.coverage'),    t(cBadge.key), cBadge.cls),
        card(t('view.vol_bar.card.target'),      fmtVol(state.volume_per_bar)),
        card(t('view.vol_bar.card.prints'),      fmtInt(state.prints.length)),
        card(t('view.vol_bar.card.bars'),        fmtInt(s.count)),
        card(t('view.vol_bar.card.total_vol'),   fmtVol(totalVol)),
        card(t('view.vol_bar.card.dropped_vol'), fmtVol(droppedVol),
             droppedVol > 0 ? 'neg' : ''),
        card(t('view.vol_bar.card.ups'),         fmtInt(s.ups),
             s.ups > s.downs ? 'pos' : ''),
        card(t('view.vol_bar.card.downs'),       fmtInt(s.downs),
             s.downs > s.ups ? 'neg' : ''),
        card(t('view.vol_bar.card.doji'),        fmtInt(s.doji)),
        card(t('view.vol_bar.card.avg_range'),   fmtUSD(s.avg_range)),
        card(t('view.vol_bar.card.avg_ticks'),   fmtNum(s.avg_ticks, 1)),
        card(t('view.vol_bar.card.last_close'),  fmtUSD(s.last_close)),
        card(t('view.vol_bar.card.parity'),
             parityOk ? t('view.vol_bar.tag.ok') : t('view.vol_bar.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(bars) {
    if (!window.uPlot) return;
    const el = document.getElementById('vb-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!bars || bars.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.vol_bar.empty">${esc(t('view.vol_bar.empty'))}</div>`;
        return;
    }
    const xs = bars.map((_, i) => i);
    const closes = bars.map(b => b.close);
    const highs = bars.map(b => b.high);
    const lows = bars.map(b => b.low);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.bar') },
            { label: t('chart.series.close'), stroke: '#00e5ff', width: 1.5, points: { show: true, size: 4 } },
            { label: t('chart.series.high'),  stroke: '#3ad96b', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('chart.series.low'),   stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => '#' + Math.trunc(v)) },
            { stroke: '#aab', size: 70 },
        ],
        legend: { show: true },
    }, [xs, closes, highs, lows], el);
}

function renderTable(bars) {
    const wrap = document.getElementById('vb-table');
    if (!bars || bars.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.vol_bar.empty">${esc(t('view.vol_bar.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, bars.length - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.vol_bar.col.idx">#</th>
                <th data-i18n="view.vol_bar.col.open">Open</th>
                <th data-i18n="view.vol_bar.col.high">High</th>
                <th data-i18n="view.vol_bar.col.low">Low</th>
                <th data-i18n="view.vol_bar.col.close">Close</th>
                <th data-i18n="view.vol_bar.col.move">Move</th>
                <th data-i18n="view.vol_bar.col.range">Range</th>
                <th data-i18n="view.vol_bar.col.volume">Volume</th>
                <th data-i18n="view.vol_bar.col.ticks">Ticks</th>
            </tr></thead>
            <tbody>
                ${bars.slice(start).map((b, k) => {
                    const i = start + k;
                    const move = b.close - b.open;
                    const cls = move > 0 ? 'pos' : move < 0 ? 'neg' : '';
                    return `<tr>
                        <td>${i + 1}</td>
                        <td>${esc(fmtUSD(b.open))}</td>
                        <td>${esc(fmtUSD(b.high))}</td>
                        <td>${esc(fmtUSD(b.low))}</td>
                        <td><strong>${esc(fmtUSD(b.close))}</strong></td>
                        <td class="${cls}">${esc(fmtMove(move))}</td>
                        <td>${esc(fmtUSD(b.high - b.low))}</td>
                        <td>${esc(fmtVol(b.volume))}</td>
                        <td>${esc(fmtInt(b.tick_count))}</td>
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
    const el = document.getElementById('vb-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('vb-err').style.display = 'none'; }
