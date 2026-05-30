// Dollar-bar chart view — aggregates trade prints into fixed-notional
// OHLC bars (López de Prado's "AFML" preferred sampling).
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
    fmtUSD, fmtMove, fmtNum, fmtInt, fmtVol, fmtNotional,
} from '../_dollar_bar_inputs.js';

let state = { ...makeDemoInput('mid-cap-uptrend') };

export async function renderDollarBar(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.dollar_bar.h1.title" class="view-title">// DOLLAR BAR CHART</h1>

        <div class="chart-panel" data-context-scope="dollar-bar">
            <h2 data-i18n="view.dollar_bar.h2.prints">Trade prints
                <small data-i18n="view.dollar_bar.h2.prints_hint" class="muted">(per line: price size)</small></h2>
            <textarea id="db-blob" rows="6"
                      data-tip="view.dollar_bar.tip.prints"
                      placeholder="100.05 200&#10;100.06 250&#10;100.10 300">${esc(printsToBlob(state.prints))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.dollar_bar.label.dollars">Dollars per bar</span>
                    <input id="db-target" type="number" step="any" min="0" value="${state.dollars_per_bar}"></label>
                <button data-i18n="view.dollar_bar.btn.compute" id="db-run" class="primary"
                        data-tip="view.dollar_bar.tip.compute" type="button">Build bars</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.dollar_bar.btn.demo_up"      id="db-demo-up"      class="secondary" type="button">Demo: mid-cap uptrend</button>
                <button data-i18n="view.dollar_bar.btn.demo_down"    id="db-demo-down"    class="secondary" type="button">Demo: mid-cap downtrend</button>
                <button data-i18n="view.dollar_bar.btn.demo_flat"    id="db-demo-flat"    class="secondary" type="button">Demo: flat market</button>
                <button data-i18n="view.dollar_bar.btn.demo_penny"   id="db-demo-penny"   class="secondary" type="button">Demo: penny stock ($3, $50k target)</button>
                <button data-i18n="view.dollar_bar.btn.demo_large"   id="db-demo-large"   class="secondary" type="button">Demo: large-cap ($450, $200k)</button>
                <button data-i18n="view.dollar_bar.btn.demo_partial" id="db-demo-partial" class="secondary" type="button">Demo: partial trail (dropped)</button>
                <button data-i18n="view.dollar_bar.btn.demo_spiky"   id="db-demo-spiky"   class="secondary" type="button">Demo: spiky notional</button>
                <button data-i18n="view.dollar_bar.btn.demo_noisy"   id="db-demo-noisy"   class="secondary" type="button">Demo: noisy walk (200 prints)</button>
            </div>
            <p data-i18n="view.dollar_bar.hint.about" class="muted">Each bar accumulates prints until Σ(price × size) ≥ dollars_per_bar. López de Prado (AFML) showed dollar bars best approximate i.i.d. returns for ML — they normalize for both activity AND price-level shifts. Trailing partial bars dropped.</p>
        </div>

        <div id="db-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.dollar_bar.h2.chart">OHLC closes by bar #</h2>
            <div id="db-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.dollar_bar.h2.density_chart">Tick density per bar — how many prints aggregated into each bar</h2>
            <div id="db-density-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.dollar_bar.h2.table">Bars (tail — last 30)</h2>
            <div id="db-table"></div>
        </div>

        <div id="db-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('db-blob').value   = printsToBlob(state.prints);
        document.getElementById('db-target').value = state.dollars_per_bar;
    };
    document.getElementById('db-demo-up').addEventListener('click',      () => { loadDemo('mid-cap-uptrend');   void compute(tok); });
    document.getElementById('db-demo-down').addEventListener('click',    () => { loadDemo('mid-cap-downtrend'); void compute(tok); });
    document.getElementById('db-demo-flat').addEventListener('click',    () => { loadDemo('flat-market');       void compute(tok); });
    document.getElementById('db-demo-penny').addEventListener('click',   () => { loadDemo('penny-stock');       void compute(tok); });
    document.getElementById('db-demo-large').addEventListener('click',   () => { loadDemo('large-cap');         void compute(tok); });
    document.getElementById('db-demo-partial').addEventListener('click', () => { loadDemo('partial-trail');     void compute(tok); });
    document.getElementById('db-demo-spiky').addEventListener('click',   () => { loadDemo('spiky-notional');    void compute(tok); });
    document.getElementById('db-demo-noisy').addEventListener('click',   () => { loadDemo('noisy-walk');        void compute(tok); });
    document.getElementById('db-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parsePrintsBlob(document.getElementById('db-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.dollar_bar.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.prints = p.prints;
    const d = Number(document.getElementById('db-target').value);
    state.dollars_per_bar = Number.isFinite(d) && d > 0 ? d : 100_000;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.prints, state.dollars_per_bar);
    renderSummary(local, true);
    renderChart(local);
    renderDensityChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.chartsDollarBar(buildBody(state));
    } catch (e) {
        showErr(`${t('view.dollar_bar.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderDensityChart(resp);
    renderTable(resp);
}

function renderSummary(bars, pending) {
    const local = localCompute(state.prints, state.dollars_per_bar);
    const parityOk = bars.length === local.length
        && bars.every((b, i) => Math.abs(b.close - local[i].close) < 1e-9
            && Math.abs(b.notional - local[i].notional) < 1e-6
            && b.tick_count === local[i].tick_count);
    const totalNotional = state.prints.reduce((s, p) => s + p.price * p.size, 0);
    const tBadge = trendBadge(bars);
    const cBadge = coverageBadge(bars, totalNotional, state.dollars_per_bar);
    const s = summarize(bars);
    const localTag = pending ? ` (${t('view.dollar_bar.tag.local')})` : '';
    const dropped = totalNotional - s.total_notional;
    document.getElementById('db-summary').innerHTML = [
        card(t('view.dollar_bar.card.verdict'),     t(tBadge.key) + localTag, tBadge.cls),
        card(t('view.dollar_bar.card.coverage'),    t(cBadge.key), cBadge.cls),
        card(t('view.dollar_bar.card.target'),      fmtNotional(state.dollars_per_bar)),
        card(t('view.dollar_bar.card.prints'),      fmtInt(state.prints.length)),
        card(t('view.dollar_bar.card.bars'),        fmtInt(s.count)),
        card(t('view.dollar_bar.card.total_not'),   fmtNotional(totalNotional)),
        card(t('view.dollar_bar.card.dropped_not'), fmtNotional(dropped),
             dropped > 0 ? 'neg' : ''),
        card(t('view.dollar_bar.card.avg_not'),     fmtNotional(s.avg_notional)),
        card(t('view.dollar_bar.card.ups'),         fmtInt(s.ups),
             s.ups > s.downs ? 'pos' : ''),
        card(t('view.dollar_bar.card.downs'),       fmtInt(s.downs),
             s.downs > s.ups ? 'neg' : ''),
        card(t('view.dollar_bar.card.doji'),        fmtInt(s.doji)),
        card(t('view.dollar_bar.card.avg_range'),   fmtUSD(s.avg_range)),
        card(t('view.dollar_bar.card.avg_ticks'),   fmtNum(s.avg_ticks, 1)),
        card(t('view.dollar_bar.card.last_close'),  fmtUSD(s.last_close)),
        card(t('view.dollar_bar.card.parity'),
             parityOk ? t('view.dollar_bar.tag.ok') : t('view.dollar_bar.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(bars) {
    if (!window.uPlot) return;
    const el = document.getElementById('db-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!bars || bars.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.dollar_bar.empty">${esc(t('view.dollar_bar.empty'))}</div>`;
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

function renderDensityChart(bars) {
    if (!window.uPlot) return;
    const el = document.getElementById('db-density-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!bars || bars.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.dollar_bar.empty_density_chart">${esc(t('view.dollar_bar.empty_density_chart'))}</div>`;
        return;
    }
    const xs = bars.map((_, i) => i);
    const ticks = bars.map(b => Number.isFinite(b.tick_count) ? b.tick_count : null);
    const meanTicks = ticks.reduce((s, v) => s + (v || 0), 0) / Math.max(1, ticks.length);
    const mean = xs.map(() => meanTicks);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('chart.series.bar') },
            { label: t('view.dollar_bar.chart.tick_count'),
              stroke: '#b86bff', width: 1.2,
              fill: 'rgba(184,107,255,0.10)',
              points: { show: false } },
            { label: t('view.dollar_bar.chart.mean_ticks'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => '#' + Math.trunc(v)) },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ticks, mean], el);
}

function renderTable(bars) {
    const wrap = document.getElementById('db-table');
    if (!bars || bars.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.dollar_bar.empty">${esc(t('view.dollar_bar.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, bars.length - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.dollar_bar.col.idx">#</th>
                <th data-i18n="view.dollar_bar.col.open">Open</th>
                <th data-i18n="view.dollar_bar.col.high">High</th>
                <th data-i18n="view.dollar_bar.col.low">Low</th>
                <th data-i18n="view.dollar_bar.col.close">Close</th>
                <th data-i18n="view.dollar_bar.col.move">Move</th>
                <th data-i18n="view.dollar_bar.col.range">Range</th>
                <th data-i18n="view.dollar_bar.col.volume">Volume</th>
                <th data-i18n="view.dollar_bar.col.notional">Notional</th>
                <th data-i18n="view.dollar_bar.col.ticks">Ticks</th>
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
                        <td>${esc(fmtNotional(b.notional))}</td>
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
    const el = document.getElementById('db-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('db-err').style.display = 'none'; }
