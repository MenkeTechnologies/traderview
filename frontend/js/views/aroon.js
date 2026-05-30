// Aroon Indicator view (Chande 1995) — measures time since high/low extremes.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    regimeBadge, lastCrossover, summarize,
    makeDemoInput,
    fmtNum, fmtPct, fmtOsc, fmtInt, fmtUSD,
} from '../_aroon_inputs.js';

let state = { ...makeDemoInput('strong-uptrend') };

export async function renderAroon(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.aroon.h1.title" class="view-title">// AROON INDICATOR</h1>

        <div class="chart-panel" data-context-scope="aroon">
            <h2 data-i18n="view.aroon.h2.bars">Bars
                <small data-i18n="view.aroon.h2.bars_hint" class="muted">(per line: high low)</small></h2>
            <textarea id="ar-blob" rows="6"
                      data-tip="view.aroon.tip.bars"
                      placeholder="101 99&#10;102 100&#10;103 101">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.aroon.label.period">Period</span>
                    <input id="ar-period" type="number" step="1" min="2" value="${state.period}"
                           data-tip="view.aroon.tip.period"></label>
                <button data-i18n="view.aroon.btn.compute" id="ar-run" class="primary"
                        data-tip="view.aroon.tip.compute" data-shortcut="aroon_run" type="button">Compute Aroon</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.aroon.btn.demo_up"     id="ar-demo-up"     class="secondary" data-tip="view.aroon.tip.demo_up"    type="button">Demo: strong uptrend</button>
                <button data-i18n="view.aroon.btn.demo_down"   id="ar-demo-down"   class="secondary" data-tip="view.aroon.tip.demo_down"  type="button">Demo: strong downtrend</button>
                <button data-i18n="view.aroon.btn.demo_flat"   id="ar-demo-flat"   class="secondary" data-tip="view.aroon.tip.demo_flat"  type="button">Demo: flat (consolidation)</button>
                <button data-i18n="view.aroon.btn.demo_chop"   id="ar-demo-chop"   class="secondary" data-tip="view.aroon.tip.demo_chop"  type="button">Demo: consolidation oscillator</button>
                <button data-i18n="view.aroon.btn.demo_bull"   id="ar-demo-bull"   class="secondary" data-tip="view.aroon.tip.demo_bull"  type="button">Demo: bullish crossover</button>
                <button data-i18n="view.aroon.btn.demo_bear"   id="ar-demo-bear"   class="secondary" data-tip="view.aroon.tip.demo_bear"  type="button">Demo: bearish crossover</button>
                <button data-i18n="view.aroon.btn.demo_noisy"  id="ar-demo-noisy"  class="secondary" data-tip="view.aroon.tip.demo_noisy" type="button">Demo: noisy walk (200 bars)</button>
                <button data-i18n="view.aroon.btn.demo_short"  id="ar-demo-short"  class="secondary" data-tip="view.aroon.tip.demo_short" type="button">Demo: short period (10)</button>
            </div>
            <p data-i18n="view.aroon.hint.about" class="muted">AroonUp = 100·(period − bars_since_high)/period. AroonDown = 100·(period − bars_since_low)/period. Oscillator = Up − Down ∈ [−100, +100]. ≥ 80 = strong trend. Crossovers signal reversals.</p>
        </div>

        <div id="ar-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.aroon.h2.chart">Aroon Up + Down + Oscillator</h2>
            <div id="ar-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.aroon.h2.strength_chart">Trend strength (|oscillator|) vs thresholds</h2>
            <div id="ar-strength-chart" style="width:100%;height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.aroon.h2.table">Per-bar Aroon (tail — last 30)</h2>
            <div id="ar-table"></div>
        </div>

        <div id="ar-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('ar-blob').value   = barsToBlob(state.bars);
        document.getElementById('ar-period').value = state.period;
    };
    document.getElementById('ar-demo-up').addEventListener('click',    () => { loadDemo('strong-uptrend');   void compute(tok); });
    document.getElementById('ar-demo-down').addEventListener('click',  () => { loadDemo('strong-downtrend'); void compute(tok); });
    document.getElementById('ar-demo-flat').addEventListener('click',  () => { loadDemo('flat');             void compute(tok); });
    document.getElementById('ar-demo-chop').addEventListener('click',  () => { loadDemo('consolidation');    void compute(tok); });
    document.getElementById('ar-demo-bull').addEventListener('click',  () => { loadDemo('bull-cross');       void compute(tok); });
    document.getElementById('ar-demo-bear').addEventListener('click',  () => { loadDemo('bear-cross');       void compute(tok); });
    document.getElementById('ar-demo-noisy').addEventListener('click', () => { loadDemo('noisy');            void compute(tok); });
    document.getElementById('ar-demo-short').addEventListener('click', () => { loadDemo('short-period');     void compute(tok); });
    document.getElementById('ar-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('ar-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.aroon.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.aroon.toast.parse_error'), { level: 'error' });
        return;
    }
    hideErr();
    state.bars = p.bars;
    const period = parseInt(document.getElementById('ar-period').value, 10);
    state.period = Number.isInteger(period) && period >= 2 ? period : DEFAULT_PERIOD;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.aroon.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.bars, state.period);
    renderSummary(local, true);
    renderChart(local);
    renderStrengthChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.anlyAroonIndicator(buildBody(state));
    } catch (e) {
        showErr(`${t('view.aroon.err.api')}: ${e.message || e}`);
        showToast(t('view.aroon.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderStrengthChart(resp);
    renderTable(resp);
    showToast(t('view.aroon.toast.computed'), { level: 'success' });
}

function renderSummary(report, pending) {
    const local = localCompute(state.bars, state.period);
    const parityOk = report.aroon_up.length === local.aroon_up.length
        && report.aroon_up.every((v, i) => {
            if (v == null && local.aroon_up[i] == null) return true;
            if (v == null || local.aroon_up[i] == null) return false;
            return Math.abs(v - local.aroon_up[i]) < 1e-9;
        });
    const s = summarize(report);
    const badge = regimeBadge(s.last_osc);
    const xover = lastCrossover(report);
    const localTag = pending ? ` (${t('view.aroon.tag.local')})` : '';
    const xoverLabel = xover
        ? `${xover.kind === 'bull' ? '▲' : '▼'} ${t(xover.kind === 'bull' ? 'view.aroon.xover.bull' : 'view.aroon.xover.bear')} @ ${xover.idx}`
        : t('view.aroon.tag.no_cross');
    const xoverCls = !xover ? '' : xover.kind === 'bull' ? 'pos' : 'neg';
    document.getElementById('ar-summary').innerHTML = [
        card(t('view.aroon.card.verdict'),    t(badge.key) + localTag, badge.cls),
        card(t('view.aroon.card.period'),     fmtInt(state.period)),
        card(t('view.aroon.card.bars'),       fmtInt(s.count)),
        card(t('view.aroon.card.populated'),  fmtInt(s.populated)),
        card(t('view.aroon.card.last_up'),    fmtPct(s.last_up),
             s.last_up >= 80 ? 'pos' : s.last_up <= 20 ? '' : ''),
        card(t('view.aroon.card.last_down'),  fmtPct(s.last_down),
             s.last_down >= 80 ? 'neg' : s.last_down <= 20 ? '' : ''),
        card(t('view.aroon.card.last_osc'),   fmtOsc(s.last_osc), badge.cls),
        card(t('view.aroon.card.last_xover'), xoverLabel, xoverCls),
        card(t('view.aroon.card.parity'),
             parityOk ? t('view.aroon.tag.ok') : t('view.aroon.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('ar-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!report.aroon_up || report.aroon_up.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.aroon.empty">${esc(t('view.aroon.empty'))}</div>`;
        return;
    }
    const xs = report.aroon_up.map((_, i) => i);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: t('view.aroon.series.bar') },
            { label: t('view.aroon.series.aroon_up'),    stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: t('view.aroon.series.aroon_down'),  stroke: '#ff3860', width: 1.5, points: { show: false } },
            { label: t('view.aroon.series.oscillator'),  stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, report.aroon_up, report.aroon_down, report.aroon_oscillator], el);
}

function renderStrengthChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('ar-strength-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!report.aroon_oscillator || report.aroon_oscillator.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.aroon.empty_strength">${esc(t('view.aroon.empty_strength'))}</div>`;
        return;
    }
    const xs = report.aroon_oscillator.map((_, i) => i);
    const strength = report.aroon_oscillator.map(v => (v == null || !Number.isFinite(v) ? null : Math.abs(v)));
    const med = xs.map(() => 30);
    const strong = xs.map(() => 50);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { range: [0, 100] } },
        series: [
            { label: t('view.aroon.series.bar') },
            { label: t('view.aroon.chart.strength'),
              stroke: '#7af0a8', width: 1.5,
              points: { show: false } },
            { label: t('view.aroon.chart.medium_30'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('view.aroon.chart.strong_50'),
              stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, strength, med, strong], el);
}

function renderTable(report) {
    const wrap = document.getElementById('ar-table');
    const n = report.aroon_up?.length || 0;
    if (n === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.aroon.empty">${esc(t('view.aroon.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, n - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.aroon.col.idx">#</th>
                <th data-i18n="view.aroon.col.high">High</th>
                <th data-i18n="view.aroon.col.low">Low</th>
                <th data-i18n="view.aroon.col.up">Up</th>
                <th data-i18n="view.aroon.col.down">Down</th>
                <th data-i18n="view.aroon.col.osc">Oscillator</th>
                <th data-i18n="view.aroon.col.regime">Regime</th>
            </tr></thead>
            <tbody>
                ${Array.from({ length: n - start }, (_, k) => {
                    const i = start + k;
                    const b = state.bars[i];
                    const u = report.aroon_up[i];
                    const d = report.aroon_down[i];
                    const o = report.aroon_oscillator[i];
                    const rBadge = regimeBadge(o);
                    return `<tr>
                        <td>${i + 1}</td>
                        <td>${esc(fmtUSD(b?.high))}</td>
                        <td>${esc(fmtUSD(b?.low))}</td>
                        <td>${esc(fmtPct(u))}</td>
                        <td>${esc(fmtPct(d))}</td>
                        <td class="${rBadge.cls}">${esc(fmtOsc(o))}</td>
                        <td data-i18n="${esc(rBadge.key)}" class="${rBadge.cls}">${esc(t(rBadge.key))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
    void fmtNum;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('ar-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ar-err').style.display = 'none'; }
