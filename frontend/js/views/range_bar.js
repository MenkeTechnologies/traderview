// Range-bar chart view — aggregates trade prints into fixed-range OHLC
// bars (time ignored, only price range matters).
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parsePrintsBlob, printsToBlob, validateInputs, buildBody, localCompute,
    trendBadge, activityBadge, summarize,
    makeDemoInput,
    fmtUSD, fmtMove, fmtNum, fmtInt, fmtVol,
} from '../_range_bar_inputs.js';

let state = { ...makeDemoInput('uptrend') };

export async function renderRangeBar(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.range_bar.h1.title" class="view-title">// RANGE BAR CHART</h1>

        <div class="chart-panel" data-context-scope="range-bar">
            <h2 data-i18n="view.range_bar.h2.prints">Trade prints
                <small data-i18n="view.range_bar.h2.prints_hint" class="muted">(per line: price size)</small></h2>
            <textarea id="rb-blob" rows="6"
                      data-tip="view.range_bar.tip.prints"
                      placeholder="100.05 10&#10;100.06 25&#10;100.10 50">${esc(printsToBlob(state.prints))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.range_bar.label.target">Target range ($)</span>
                    <input id="rb-target" type="number" step="any" min="0" value="${state.target_range}" data-tip="view.range_bar.tip.target"></label>
                <button data-i18n="view.range_bar.btn.compute" id="rb-run" class="primary"
                        data-tip="view.range_bar.tip.compute" data-shortcut="range_bar_run" type="button">Build bars</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.range_bar.btn.demo_up"     id="rb-demo-up"      class="secondary" type="button" data-tip="view.range_bar.tip.demo_up">Demo: pure uptrend</button>
                <button data-i18n="view.range_bar.btn.demo_down"   id="rb-demo-down"    class="secondary" type="button" data-tip="view.range_bar.tip.demo_down">Demo: pure downtrend</button>
                <button data-i18n="view.range_bar.btn.demo_chop"   id="rb-demo-chop"    class="secondary" type="button" data-tip="view.range_bar.tip.demo_chop">Demo: choppy oscillation</button>
                <button data-i18n="view.range_bar.btn.demo_flat"   id="rb-demo-flat"    class="secondary" type="button" data-tip="view.range_bar.tip.demo_flat">Demo: flat (no bars)</button>
                <button data-i18n="view.range_bar.btn.demo_big"    id="rb-demo-big"     class="secondary" type="button" data-tip="view.range_bar.tip.demo_big">Demo: big-volume prints</button>
                <button data-i18n="view.range_bar.btn.demo_small"  id="rb-demo-small"   class="secondary" type="button" data-tip="view.range_bar.tip.demo_small">Demo: range too small</button>
                <button data-i18n="view.range_bar.btn.demo_wide"   id="rb-demo-wide"    class="secondary" type="button" data-tip="view.range_bar.tip.demo_wide">Demo: wide range (10$)</button>
                <button data-i18n="view.range_bar.btn.demo_noisy"  id="rb-demo-noisy"   class="secondary" type="button" data-tip="view.range_bar.tip.demo_noisy">Demo: noisy walk (200 prints)</button>
            </div>
            <p data-i18n="view.range_bar.hint.about" class="muted">Each bar accumulates prints until high − low ≥ target_range, then closes and starts a new bar. Time is ignored. Flat markets emit nothing; volatile markets emit many bars. Trailing partial bars are not emitted.</p>
        </div>

        <div id="rb-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.range_bar.h2.chart">OHLC closes by bar #</h2>
            <div id="rb-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.range_bar.h2.table">Bars (tail — last 30)</h2>
            <div id="rb-table"></div>
        </div>

        <div id="rb-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('rb-blob').value   = printsToBlob(state.prints);
        document.getElementById('rb-target').value = state.target_range;
    };
    document.getElementById('rb-demo-up').addEventListener('click',    () => { loadDemo('uptrend');     void compute(tok); });
    document.getElementById('rb-demo-down').addEventListener('click',  () => { loadDemo('downtrend');   void compute(tok); });
    document.getElementById('rb-demo-chop').addEventListener('click',  () => { loadDemo('chop');        void compute(tok); });
    document.getElementById('rb-demo-flat').addEventListener('click',  () => { loadDemo('flat');        void compute(tok); });
    document.getElementById('rb-demo-big').addEventListener('click',   () => { loadDemo('big-prints');  void compute(tok); });
    document.getElementById('rb-demo-small').addEventListener('click', () => { loadDemo('small-range'); void compute(tok); });
    document.getElementById('rb-demo-wide').addEventListener('click',  () => { loadDemo('wide-range');  void compute(tok); });
    document.getElementById('rb-demo-noisy').addEventListener('click', () => { loadDemo('noisy-walk');  void compute(tok); });
    document.getElementById('rb-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parsePrintsBlob(document.getElementById('rb-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.range_bar.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.range_bar.toast.parse_error', { n: p.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.prints = p.prints;
    const r = Number(document.getElementById('rb-target').value);
    state.target_range = Number.isFinite(r) && r > 0 ? r : 1;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.range_bar.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.prints, state.target_range);
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.chartsRangeBar(buildBody(state));
    } catch (e) {
        showErr(`${t('view.range_bar.err.api')}: ${e.message || e}`);
        showToast(t('view.range_bar.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
    const bars = Array.isArray(resp) ? resp.length : 0;
    const s = summarize(resp);
    const dir = s.ups > s.downs ? 'up' : s.downs > s.ups ? 'down' : 'flat';
    const level = dir === 'up' ? 'success' : dir === 'down' ? 'warning' : 'info';
    showToast(t('view.range_bar.toast.built', { bars, prints: state.prints.length, dir }), { level });
}

function renderSummary(bars, pending) {
    const local = localCompute(state.prints, state.target_range);
    const parityOk = bars.length === local.length
        && bars.every((b, i) => Math.abs(b.close - local[i].close) < 1e-9
            && Math.abs(b.volume - local[i].volume) < 1e-9
            && b.tick_count === local[i].tick_count);
    const tBadge = trendBadge(bars);
    const aBadge = activityBadge(bars, state.prints.length);
    const s = summarize(bars);
    const localTag = pending ? ` (${t('view.range_bar.tag.local')})` : '';
    document.getElementById('rb-summary').innerHTML = [
        card(t('view.range_bar.card.verdict'),     t(tBadge.key) + localTag, tBadge.cls),
        card(t('view.range_bar.card.activity'),    t(aBadge.key), aBadge.cls),
        card(t('view.range_bar.card.target'),      fmtUSD(state.target_range)),
        card(t('view.range_bar.card.prints'),      fmtInt(state.prints.length)),
        card(t('view.range_bar.card.bars'),        fmtInt(s.count)),
        card(t('view.range_bar.card.ups'),         fmtInt(s.ups),
             s.ups > s.downs ? 'pos' : ''),
        card(t('view.range_bar.card.downs'),       fmtInt(s.downs),
             s.downs > s.ups ? 'neg' : ''),
        card(t('view.range_bar.card.doji'),        fmtInt(s.doji)),
        card(t('view.range_bar.card.avg_ticks'),   fmtNum(s.avg_ticks, 1)),
        card(t('view.range_bar.card.total_vol'),   fmtVol(s.total_volume)),
        card(t('view.range_bar.card.last_close'),  fmtUSD(s.last_close)),
        card(t('view.range_bar.card.parity'),
             parityOk ? t('view.range_bar.tag.ok') : t('view.range_bar.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(bars) {
    if (!window.uPlot) return;
    const el = document.getElementById('rb-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!bars || bars.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.range_bar.empty">${esc(t('view.range_bar.empty'))}</div>`;
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
    const wrap = document.getElementById('rb-table');
    if (!bars || bars.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.range_bar.empty">${esc(t('view.range_bar.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, bars.length - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.range_bar.col.idx">#</th>
                <th data-i18n="view.range_bar.col.open">Open</th>
                <th data-i18n="view.range_bar.col.high">High</th>
                <th data-i18n="view.range_bar.col.low">Low</th>
                <th data-i18n="view.range_bar.col.close">Close</th>
                <th data-i18n="view.range_bar.col.move">Move</th>
                <th data-i18n="view.range_bar.col.range">Range</th>
                <th data-i18n="view.range_bar.col.volume">Volume</th>
                <th data-i18n="view.range_bar.col.ticks">Ticks</th>
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
    const el = document.getElementById('rb-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('rb-err').style.display = 'none'; }
