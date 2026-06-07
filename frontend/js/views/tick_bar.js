// Tick-bar chart view — aggregates trade prints into fixed-tick-count
// OHLC bars (time ignored).
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parsePrintsBlob, printsToBlob, validateInputs, buildBody, localCompute,
    trendBadge, coverageBadge, summarize,
    makeDemoInput,
    fmtUSD, fmtMove, fmtNum, fmtInt, fmtVol,
} from '../_tick_bar_inputs.js';

let state = { ...makeDemoInput('uptrend') };

export async function renderTickBar(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.tick_bar.h1.title" class="view-title">// TICK BAR CHART</h1>

        <div class="chart-panel" data-context-scope="tick-bar">
            <h2 data-i18n="view.tick_bar.h2.prints">Trade prints
                <small data-i18n="view.tick_bar.h2.prints_hint" class="muted">(per line: price size)</small></h2>
            <textarea id="tb-blob" rows="6"
                      data-tip="view.tick_bar.tip.prints"
                      placeholder="100.05 10&#10;100.06 25&#10;100.10 50">${esc(printsToBlob(state.prints))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.tick_bar.label.ticks">Ticks per bar</span>
                    <input id="tb-ticks" type="number" step="1" min="1" value="${state.ticks_per_bar}" data-tip="view.tick_bar.tip.ticks"></label>
                <button data-i18n="view.tick_bar.btn.compute" id="tb-run" class="primary"
                        data-tip="view.tick_bar.tip.compute" data-shortcut="tick_bar_run" type="button">Build bars</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.tick_bar.btn.demo_up"      id="tb-demo-up"      class="secondary" type="button" data-tip="view.tick_bar.tip.demo_up">Demo: pure uptrend</button>
                <button data-i18n="view.tick_bar.btn.demo_down"    id="tb-demo-down"    class="secondary" type="button" data-tip="view.tick_bar.tip.demo_down">Demo: pure downtrend</button>
                <button data-i18n="view.tick_bar.btn.demo_flat"    id="tb-demo-flat"    class="secondary" type="button" data-tip="view.tick_bar.tip.demo_flat">Demo: flat (doji bars)</button>
                <button data-i18n="view.tick_bar.btn.demo_noisy"   id="tb-demo-noisy"   class="secondary" type="button" data-tip="view.tick_bar.tip.demo_noisy">Demo: noisy walk (200 prints)</button>
                <button data-i18n="view.tick_bar.btn.demo_small"   id="tb-demo-small"   class="secondary" type="button" data-tip="view.tick_bar.tip.demo_small">Demo: small bars (N=5)</button>
                <button data-i18n="view.tick_bar.btn.demo_large"   id="tb-demo-large"   class="secondary" type="button" data-tip="view.tick_bar.tip.demo_large">Demo: large bars (N=30)</button>
                <button data-i18n="view.tick_bar.btn.demo_partial" id="tb-demo-partial" class="secondary" type="button" data-tip="view.tick_bar.tip.demo_partial">Demo: partial trailing bar (dropped)</button>
                <button data-i18n="view.tick_bar.btn.demo_one"     id="tb-demo-one"     class="secondary" type="button" data-tip="view.tick_bar.tip.demo_one">Demo: 1 tick/bar (every print)</button>
            </div>
            <p data-i18n="view.tick_bar.hint.about" class="muted">One OHLC bar per N prints. Time is ignored — useful in liquid markets where 1-minute bars contain wildly different trade counts. Trailing partial bars are not emitted.</p>
        </div>

        <div id="tb-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.tick_bar.h2.chart">OHLC closes by bar #</h2>
            <div id="tb-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.tick_bar.h2.vol_chart">Per-bar volume (size aggregated within each tick window)</h2>
            <div id="tb-vol-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.tick_bar.hint.vol_chart" class="muted small">Each bar's total trade size. Since each bar spans the same number of ticks (not the same wall-clock time), high-volume bars are heavy-print-size moments. Orthogonal to the OHLC chart above.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.tick_bar.h2.table">Bars (tail — last 30)</h2>
            <div id="tb-table"></div>
        </div>

        <div id="tb-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('tb-blob').value  = printsToBlob(state.prints);
        document.getElementById('tb-ticks').value = state.ticks_per_bar;
    };
    document.getElementById('tb-demo-up').addEventListener('click',     () => { loadDemo('uptrend');     void compute(tok); });
    document.getElementById('tb-demo-down').addEventListener('click',   () => { loadDemo('downtrend');   void compute(tok); });
    document.getElementById('tb-demo-flat').addEventListener('click',   () => { loadDemo('flat');        void compute(tok); });
    document.getElementById('tb-demo-noisy').addEventListener('click',  () => { loadDemo('noisy');       void compute(tok); });
    document.getElementById('tb-demo-small').addEventListener('click',  () => { loadDemo('small-bars');  void compute(tok); });
    document.getElementById('tb-demo-large').addEventListener('click',  () => { loadDemo('large-bars');  void compute(tok); });
    document.getElementById('tb-demo-partial').addEventListener('click', () => { loadDemo('partial');    void compute(tok); });
    document.getElementById('tb-demo-one').addEventListener('click',    () => { loadDemo('one-tick');    void compute(tok); });
    document.getElementById('tb-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parsePrintsBlob(document.getElementById('tb-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.tick_bar.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.tick_bar.toast.parse_error', { n: p.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.prints = p.prints;
    const n = parseInt(document.getElementById('tb-ticks').value, 10);
    state.ticks_per_bar = Number.isInteger(n) && n >= 1 ? n : 10;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.tick_bar.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.prints, state.ticks_per_bar);
    renderSummary(local, true);
    renderChart(local);
    renderVolChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.chartsTickBar(buildBody(state));
    } catch (e) {
        showErr(`${t('view.tick_bar.err.api')}: ${e.message || e}`);
        showToast(t('view.tick_bar.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderVolChart(resp);
    renderTable(resp);
    const bars = Array.isArray(resp) ? resp.length : 0;
    const covered = bars * state.ticks_per_bar;
    const dropped = state.prints.length - covered;
    const level = dropped > 0 ? 'warning' : 'success';
    showToast(t('view.tick_bar.toast.built', { bars, prints: state.prints.length, dropped }), { level });
}

function renderSummary(bars, pending) {
    const local = localCompute(state.prints, state.ticks_per_bar);
    const parityOk = bars.length === local.length
        && bars.every((b, i) => Math.abs(b.close - local[i].close) < 1e-9
            && Math.abs(b.volume - local[i].volume) < 1e-9
            && b.tick_count === local[i].tick_count);
    const tBadge = trendBadge(bars);
    const cBadge = coverageBadge(bars, state.prints.length, state.ticks_per_bar);
    const s = summarize(bars);
    const localTag = pending ? ` (${t('view.tick_bar.tag.local')})` : '';
    const covered = bars.length * state.ticks_per_bar;
    const dropped = state.prints.length - covered;
    document.getElementById('tb-summary').innerHTML = [
        card(t('view.tick_bar.card.verdict'),     t(tBadge.key) + localTag, tBadge.cls),
        card(t('view.tick_bar.card.coverage'),    t(cBadge.key), cBadge.cls),
        card(t('view.tick_bar.card.ticks_per'),   fmtInt(state.ticks_per_bar)),
        card(t('view.tick_bar.card.prints'),      fmtInt(state.prints.length)),
        card(t('view.tick_bar.card.bars'),        fmtInt(s.count)),
        card(t('view.tick_bar.card.dropped'),     fmtInt(dropped),
             dropped > 0 ? 'neg' : ''),
        card(t('view.tick_bar.card.ups'),         fmtInt(s.ups),
             s.ups > s.downs ? 'pos' : ''),
        card(t('view.tick_bar.card.downs'),       fmtInt(s.downs),
             s.downs > s.ups ? 'neg' : ''),
        card(t('view.tick_bar.card.doji'),        fmtInt(s.doji)),
        card(t('view.tick_bar.card.avg_range'),   fmtUSD(s.avg_range)),
        card(t('view.tick_bar.card.total_vol'),   fmtVol(s.total_volume)),
        card(t('view.tick_bar.card.last_close'),  fmtUSD(s.last_close)),
        card(t('view.tick_bar.card.parity'),
             parityOk ? t('view.tick_bar.tag.ok') : t('view.tick_bar.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
    void fmtNum;
}

function renderChart(bars) {
    if (!window.uPlot) return;
    const el = document.getElementById('tb-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!bars || bars.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.tick_bar.empty">${esc(t('view.tick_bar.empty'))}</div>`;
        return;
    }
    const xs = bars.map((_, i) => i);
    const closes = bars.map(b => b.close);
    const highs = bars.map(b => b.high);
    const lows = bars.map(b => b.low);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: { time: false,}, y: {} },
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

function renderVolChart(bars) {
    const el = document.getElementById('tb-vol-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (bars || []).filter(b => Number.isFinite(Number(b.volume)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.tick_bar.empty_vol_chart">${esc(t('view.tick_bar.empty_vol_chart'))}</div>`;
        return;
    }
    const ys = valid.map(b => Number(b.volume));
    const xs = ys.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.tick_bar.chart.bar_idx') },
            { label: t('view.tick_bar.chart.volume'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => '#' + Math.trunc(v)) },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderTable(bars) {
    const wrap = document.getElementById('tb-table');
    if (!bars || bars.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.tick_bar.empty">${esc(t('view.tick_bar.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, bars.length - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.tick_bar.col.idx">#</th>
                <th data-i18n="view.tick_bar.col.open">Open</th>
                <th data-i18n="view.tick_bar.col.high">High</th>
                <th data-i18n="view.tick_bar.col.low">Low</th>
                <th data-i18n="view.tick_bar.col.close">Close</th>
                <th data-i18n="view.tick_bar.col.move">Move</th>
                <th data-i18n="view.tick_bar.col.range">Range</th>
                <th data-i18n="view.tick_bar.col.volume">Volume</th>
                <th data-i18n="view.tick_bar.col.ticks">Ticks</th>
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
    const el = document.getElementById('tb-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('tb-err').style.display = 'none'; }
