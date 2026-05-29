// Absorption Detector view — heavy-volume tight-range bars (one side absorbing the other).
//
// i18n throughout.

import uPlot from '../vendor/uPlot.esm.js';
import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { uplotTheme } from '../uplot_theme.js';
import {
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    lastSignalBadge, biasBadge, intensityBadge, summarizeBars,
    makeDemoInput,
    fmtPrice, fmtInt, fmtPct, fmtRatio,
} from '../_absorption_inputs.js';

let state = { ...makeDemoInput('bullish-absorb') };
let chart = null;

export async function renderAbsorption(mount, _appState) {
    const tok = currentViewToken();
    if (chart) { chart.destroy(); chart = null; }
    mount.innerHTML = `
        <h1 data-i18n="view.abs.h1.title" class="view-title">// ABSORPTION DETECTOR</h1>

        <div class="chart-panel" data-context-scope="absorption">
            <h2 data-i18n="view.abs.h2.bars">Bars
                <small data-i18n="view.abs.h2.bars_hint" class="muted">(4 tokens per line: high low close volume)</small></h2>
            <textarea id="abs-blob" rows="7"
                      data-tip="view.abs.tip.bars"
                      placeholder="101 99 100 1000\n100.9 99.9 100.9 10000">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label data-i18n="view.abs.label.period">Period</label>
                <input id="abs-period" type="number" step="1" min="2" max="500" value="${state.period}"
                       data-tip="view.abs.tip.period">
                <label data-i18n="view.abs.label.threshold">Range threshold</label>
                <input id="abs-threshold" type="number" step="0.05" min="0.01" value="${state.threshold}"
                       data-tip="view.abs.tip.threshold">
                <label data-i18n="view.abs.label.vol_mul">Volume multiplier</label>
                <input id="abs-volmul" type="number" step="0.1" min="0.1" value="${state.vol_multiplier}"
                       data-tip="view.abs.tip.vol_mul">
                <button data-i18n="view.abs.btn.compute" id="abs-run" class="primary"
                        data-tip="view.abs.tip.compute" type="button">Detect</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.abs.btn.demo_flat"   id="abs-d1" class="secondary" type="button">Demo: flat</button>
                <button data-i18n="view.abs.btn.demo_bull"   id="abs-d2" class="secondary" type="button">Demo: bullish</button>
                <button data-i18n="view.abs.btn.demo_bear"   id="abs-d3" class="secondary" type="button">Demo: bearish</button>
                <button data-i18n="view.abs.btn.demo_normal" id="abs-d4" class="secondary" type="button">Demo: normal vol</button>
                <button data-i18n="view.abs.btn.demo_multi"  id="abs-d5" class="secondary" type="button">Demo: multi</button>
                <button data-i18n="view.abs.btn.demo_noisy"  id="abs-d6" class="secondary" type="button">Demo: noisy</button>
                <button data-i18n="view.abs.btn.demo_short"  id="abs-d7" class="secondary" type="button">Demo: short period</button>
                <button data-i18n="view.abs.btn.demo_tight"  id="abs-d8" class="secondary" type="button">Demo: tight thresh</button>
            </div>
            <p data-i18n="view.abs.hint.about" class="muted">Flags bars where range-per-volume is much smaller than recent baseline AND volume is much larger — i.e. one side absorbed the other's flow. Direction inferred from close vs midpoint and prior close.</p>
        </div>

        <div id="abs-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.abs.h2.chart">Close + absorption markers</h2>
            <div id="abs-chart" style="width:100%;height:320px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.abs.h2.events">Absorption events</h2>
            <div id="abs-events"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.abs.h2.stats">Bar summary</h2>
            <div id="abs-stats"></div>
        </div>

        <div id="abs-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('abs-blob').value      = barsToBlob(state.bars);
        document.getElementById('abs-period').value    = state.period;
        document.getElementById('abs-threshold').value = state.threshold;
        document.getElementById('abs-volmul').value    = state.vol_multiplier;
    };
    document.getElementById('abs-d1').addEventListener('click', () => { loadDemo('flat');            void compute(tok); });
    document.getElementById('abs-d2').addEventListener('click', () => { loadDemo('bullish-absorb'); void compute(tok); });
    document.getElementById('abs-d3').addEventListener('click', () => { loadDemo('bearish-absorb'); void compute(tok); });
    document.getElementById('abs-d4').addEventListener('click', () => { loadDemo('normal-volume');  void compute(tok); });
    document.getElementById('abs-d5').addEventListener('click', () => { loadDemo('multi-absorb');   void compute(tok); });
    document.getElementById('abs-d6').addEventListener('click', () => { loadDemo('noisy');           void compute(tok); });
    document.getElementById('abs-d7').addEventListener('click', () => { loadDemo('short-period');    void compute(tok); });
    document.getElementById('abs-d8').addEventListener('click', () => { loadDemo('tight-thresh');    void compute(tok); });
    document.getElementById('abs-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('abs-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.abs.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    state.bars           = p.bars;
    state.period         = parseInt(document.getElementById('abs-period').value, 10);
    state.threshold      = Number(document.getElementById('abs-threshold').value);
    state.vol_multiplier = Number(document.getElementById('abs-volmul').value);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.bars, state.period, state.threshold, state.vol_multiplier);
    renderSummary(local, true);
    renderChart(local);
    renderEvents(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyAbsorptionDetector(buildBody(state));
    } catch (e) {
        showErr(`${t('view.abs.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderEvents(resp);
    renderStats();
}

function renderSummary(report, pending) {
    const local = localCompute(state.bars, state.period, state.threshold, state.vol_multiplier);
    const parityOk = report
        && report.bullish.length === local.bullish.length
        && report.bullish.every((v, i) => v === local.bullish[i])
        && report.bearish.every((v, i) => v === local.bearish[i]);
    const last = lastSignalBadge(report);
    const bias = biasBadge(report);
    const intens = intensityBadge(report);
    const bullCount = report ? report.bullish.filter(Boolean).length : 0;
    const bearCount = report ? report.bearish.filter(Boolean).length : 0;
    const localTag = pending ? ` (${t('view.abs.tag.local')})` : '';
    document.getElementById('abs-summary').innerHTML = [
        card(t('view.abs.card.last'),       t(last.key) + localTag, last.cls),
        card(t('view.abs.card.bias'),       t(bias.key), bias.cls),
        card(t('view.abs.card.intensity'),  t(intens.key), intens.cls),
        card(t('view.abs.card.bull'),       fmtInt(bullCount), 'pos'),
        card(t('view.abs.card.bear'),       fmtInt(bearCount), 'neg'),
        card(t('view.abs.card.bars_ago'),   last.barsAgo != null ? String(last.barsAgo) : '—'),
        card(t('view.abs.card.rate'),       fmtPct((bullCount + bearCount) / Math.max(1, state.bars.length))),
        card(t('view.abs.card.parity'),
             parityOk ? t('view.abs.tag.ok') : t('view.abs.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    const wrap = document.getElementById('abs-chart');
    if (!report || state.bars.length === 0) { wrap.innerHTML = ''; return; }
    const xs = state.bars.map((_, i) => i);
    const closes = state.bars.map(b => b.close);
    const bullMark = state.bars.map((b, i) => report.bullish[i] ? b.close : null);
    const bearMark = state.bars.map((b, i) => report.bearish[i] ? b.close : null);
    const opts = uplotTheme({
        width: wrap.clientWidth || 800,
        height: 320,
        scales: { x: { time: false } },
        series: [
            { label: 'idx' },
            { label: t('view.abs.series.close'), stroke: 'var(--fg)', width: 1 },
            { label: t('view.abs.series.bullish'), stroke: 'var(--green)', width: 0,
              points: { show: true, size: 9, fill: 'var(--green)' } },
            { label: t('view.abs.series.bearish'), stroke: 'var(--red)', width: 0,
              points: { show: true, size: 9, fill: 'var(--red)' } },
        ],
    });
    if (chart) chart.destroy();
    chart = new uPlot(opts, [xs, closes, bullMark, bearMark], wrap);
}

function renderEvents(report) {
    const wrap = document.getElementById('abs-events');
    if (!report) { wrap.innerHTML = ''; return; }
    const rows = [];
    for (let i = 0; i < report.bullish.length; i++) {
        if (!report.bullish[i] && !report.bearish[i]) continue;
        const dir = report.bullish[i] ? 'bull' : 'bear';
        const b = state.bars[i];
        const range = b.high - b.low;
        const mid = b.low + range / 2;
        const rpv = range / b.volume;
        rows.push(`<tr>
            <td>${i}</td>
            <td class="${dir === 'bull' ? 'pos' : 'neg'}">${esc(t(dir === 'bull' ? 'view.abs.bias.bullish' : 'view.abs.bias.bearish'))}</td>
            <td>${esc(fmtPrice(b.high))}</td>
            <td>${esc(fmtPrice(b.low))}</td>
            <td>${esc(fmtPrice(b.close))}</td>
            <td>${esc(fmtPrice(mid))}</td>
            <td>${fmtInt(b.volume)}</td>
            <td>${esc(fmtRatio(rpv, 6))}</td>
        </tr>`);
    }
    if (rows.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.abs.no_events">${esc(t('view.abs.no_events'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th>
                <th data-i18n="view.abs.col.bias">Bias</th>
                <th data-i18n="view.abs.col.high">High</th>
                <th data-i18n="view.abs.col.low">Low</th>
                <th data-i18n="view.abs.col.close">Close</th>
                <th data-i18n="view.abs.col.mid">Mid</th>
                <th data-i18n="view.abs.col.volume">Volume</th>
                <th data-i18n="view.abs.col.rpv">Range/Vol</th>
            </tr></thead>
            <tbody>${rows.join('')}</tbody>
        </table>
    `;
}

function renderStats() {
    const wrap = document.getElementById('abs-stats');
    const s = summarizeBars(state.bars);
    if (s.count === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.abs.empty">${esc(t('view.abs.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.abs.col.metric">Metric</th>
                <th data-i18n="view.abs.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.abs.row.count">Bars</td>             <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.abs.row.last_close">Last close</td>  <td>${esc(fmtPrice(s.last_close))}</td></tr>
                <tr><td data-i18n="view.abs.row.min_low">Min low</td>        <td>${esc(fmtPrice(s.min_low))}</td></tr>
                <tr><td data-i18n="view.abs.row.max_high">Max high</td>      <td>${esc(fmtPrice(s.max_high))}</td></tr>
                <tr><td data-i18n="view.abs.row.vol_min">Vol min</td>        <td>${fmtInt(s.vol_min)}</td></tr>
                <tr><td data-i18n="view.abs.row.vol_max">Vol max</td>        <td>${fmtInt(s.vol_max)}</td></tr>
                <tr><td data-i18n="view.abs.row.vol_avg">Vol avg</td>        <td>${esc(fmtPrice(s.vol_avg, 0))}</td></tr>
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
    const el = document.getElementById('abs-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('abs-err').style.display = 'none'; }
