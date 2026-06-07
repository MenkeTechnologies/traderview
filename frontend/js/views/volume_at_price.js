// Volume-at-price (Volume Profile) view — horizontal volume histogram
// per price bucket with POC + value-area overlay.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_NUM_BINS, DEFAULT_VA_PCT,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    valueAreaRangePct, profileBadge,
    makeDemoInput, fmtUSD, fmtVol, fmtInt, fmtPct,
} from '../_volume_at_price_inputs.js';

let state = { ...makeDemoInput('normal-session') };

export async function renderVolumeAtPrice(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.vap.h1.title" class="view-title">// VOLUME AT PRICE</h1>

        <div class="chart-panel" data-context-scope="vap">
            <h2 data-i18n="view.vap.h2.bars">Bars
                <small data-i18n="view.vap.h2.bars_hint" class="muted">(per line: high low volume)</small></h2>
            <textarea id="vap-blob" rows="6"
                      data-tip="view.vap.tip.bars"
                      placeholder="101 99 1000&#10;102 100 1200&#10;103 101 900">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.vap.label.num_bins">Bins</span>
                    <input id="vap-bins" type="number" step="1" min="2" value="${state.num_bins}" data-tip="view.vap.tip.num_bins"></label>
                <label><span data-i18n="view.vap.label.va_pct">Value-area %</span>
                    <input id="vap-va" type="number" step="0.01" min="1" max="99.9" value="${state.value_area_pct}" data-tip="view.vap.tip.va_pct"></label>
                <button data-i18n="view.vap.btn.compute" id="vap-run" class="primary"
                        data-tip="view.vap.tip.compute" data-shortcut="volume_at_price_run" type="button">Build profile</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.vap.btn.demo_normal"  id="vap-demo-norm"   class="secondary" type="button" data-tip="view.vap.tip.demo_norm">Demo: normal session</button>
                <button data-i18n="view.vap.btn.demo_tight"   id="vap-demo-tight"  class="secondary" type="button" data-tip="view.vap.tip.demo_tight">Demo: tight balanced</button>
                <button data-i18n="view.vap.btn.demo_trend"   id="vap-demo-trend"  class="secondary" type="button" data-tip="view.vap.tip.demo_trend">Demo: trending up</button>
                <button data-i18n="view.vap.btn.demo_double"  id="vap-demo-double" class="secondary" type="button" data-tip="view.vap.tip.demo_double">Demo: double-distribution</button>
                <button data-i18n="view.vap.btn.demo_spike"   id="vap-demo-spike"  class="secondary" type="button" data-tip="view.vap.tip.demo_spike">Demo: spike POC</button>
                <button data-i18n="view.vap.btn.demo_narrow"  id="vap-demo-narrow" class="secondary" type="button" data-tip="view.vap.tip.demo_narrow">Demo: narrow VA (90%)</button>
                <button data-i18n="view.vap.btn.demo_wide"    id="vap-demo-wide"   class="secondary" type="button" data-tip="view.vap.tip.demo_wide">Demo: wide VA (50%)</button>
                <button data-i18n="view.vap.btn.demo_fine"    id="vap-demo-fine"   class="secondary" type="button" data-tip="view.vap.tip.demo_fine">Demo: fine bins (100)</button>
            </div>
            <p data-i18n="view.vap.hint.about" class="muted">Each bar's volume is split across price buckets it overlaps, proportional to overlap length. POC = bin with the most volume (price the market accepted most). Value Area = bracket around POC covering N% of total volume.</p>
        </div>

        <div id="vap-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.vap.h2.profile">Volume profile (POC highlighted, VA shaded)</h2>
            <div id="vap-chart" style="width:100%;height:420px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.vap.h2.cum_chart">Cumulative volume share by price bin (low → high)</h2>
            <div id="vap-cum-chart" style="width:100%;height:240px"></div>
            <p data-i18n="view.vap.hint.cum_chart" class="muted small">Running cumulative volume fraction as price rises through the bins. Reveals concentration: a steep early rise = volume sits at the lows; a steep late rise = at the highs; a near-diagonal = uniform. Orthogonal to the per-bin histogram above. Yellow dashed = 50 % accumulation reference.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.vap.h2.table">Top 10 bins by volume</h2>
            <div id="vap-table"></div>
        </div>

        <div id="vap-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('vap-blob').value = barsToBlob(state.bars);
        document.getElementById('vap-bins').value = state.num_bins;
        document.getElementById('vap-va').value   = state.value_area_pct;
    };
    document.getElementById('vap-demo-norm').addEventListener('click',   () => { loadDemo('normal-session');       void compute(tok); });
    document.getElementById('vap-demo-tight').addEventListener('click',  () => { loadDemo('tight-balanced');       void compute(tok); });
    document.getElementById('vap-demo-trend').addEventListener('click',  () => { loadDemo('trending-up');          void compute(tok); });
    document.getElementById('vap-demo-double').addEventListener('click', () => { loadDemo('double-distribution');  void compute(tok); });
    document.getElementById('vap-demo-spike').addEventListener('click',  () => { loadDemo('spike-poc');            void compute(tok); });
    document.getElementById('vap-demo-narrow').addEventListener('click', () => { loadDemo('narrow-va');            void compute(tok); });
    document.getElementById('vap-demo-wide').addEventListener('click',   () => { loadDemo('wide-va');              void compute(tok); });
    document.getElementById('vap-demo-fine').addEventListener('click',   () => { loadDemo('fine-bins');            void compute(tok); });
    document.getElementById('vap-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('vap-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.vap.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.vap.toast.parse_error', { n: p.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.bars = p.bars;
    const b = parseInt(document.getElementById('vap-bins').value, 10);
    const v = Number(document.getElementById('vap-va').value);
    state.num_bins = Number.isInteger(b) && b >= 2 ? b : DEFAULT_NUM_BINS;
    state.value_area_pct = Number.isFinite(v) && v >= 1 && v <= 99.9 ? v : DEFAULT_VA_PCT;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.vap.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.bars, state.num_bins, state.value_area_pct);
    renderSummary(local, true);
    renderChart(local);
    renderCumChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.chartsVolumeAtPrice(buildBody(state));
    } catch (e) {
        showErr(`${t('view.vap.err.api')}: ${e.message || e}`);
        showToast(t('view.vap.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderCumChart(resp);
    renderTable(resp);
    const bins = (resp.bins || []).length;
    const poc = resp.poc_index != null && resp.bins && resp.bins[resp.poc_index]
        ? Number(resp.bins[resp.poc_index].price_mid).toFixed(2)
        : '—';
    const vaPct = (Number(resp.value_area_pct) || 0).toFixed(0);
    showToast(t('view.vap.toast.built', { bins, poc, va: vaPct }), { level: 'success' });
}

function renderSummary(report, pending) {
    const local = localCompute(state.bars, state.num_bins, state.value_area_pct);
    const parityOk = report.bins.length === local.bins.length
        && report.poc_index === local.poc_index
        && report.bins.every((b, i) => Math.abs(b.volume - local.bins[i].volume) < 1e-6);
    const badge = profileBadge(report);
    const localTag = pending ? ` (${t('view.vap.tag.local')})` : '';
    const poc = report.poc_index != null && report.bins[report.poc_index]
        ? report.bins[report.poc_index]
        : null;
    const vaRange = valueAreaRangePct(report);
    document.getElementById('vap-summary').innerHTML = [
        card(t('view.vap.card.verdict'),    t(badge.key) + localTag, badge.cls),
        card(t('view.vap.card.bars'),       fmtInt(state.bars.length)),
        card(t('view.vap.card.bins'),       fmtInt(report.bins.length)),
        card(t('view.vap.card.total_vol'),  fmtVol(report.total_volume)),
        card(t('view.vap.card.poc'),        poc ? fmtUSD(poc.center) : '—'),
        card(t('view.vap.card.poc_vol'),    poc ? fmtVol(poc.volume) : '—'),
        card(t('view.vap.card.va_high'),    report.value_area_high != null ? fmtUSD(report.value_area_high) : '—'),
        card(t('view.vap.card.va_low'),     report.value_area_low  != null ? fmtUSD(report.value_area_low)  : '—'),
        card(t('view.vap.card.va_range'),   fmtPct(vaRange)),
        card(t('view.vap.card.parity'),
             parityOk ? t('view.vap.tag.ok') : t('view.vap.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('vap-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!report.bins || report.bins.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.vap.empty">${esc(t('view.vap.empty'))}</div>`;
        return;
    }
    // x = bin index (rendered as price label), y = volume.
    const xs = report.bins.map((_, i) => i);
    const ys = report.bins.map(b => b.volume);
    // POC marker series: null everywhere except POC bin.
    const pocSeries = report.bins.map((_, i) => i === report.poc_index ? ys[i] : null);
    // VA shading series: between value_area_low and value_area_high, render volume; else null.
    const vaSeries = report.bins.map((b, i) => {
        if (report.value_area_low == null || report.value_area_high == null) return null;
        return (b.center >= report.value_area_low && b.center <= report.value_area_high) ? ys[i] : null;
    });
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 420,
        scales: { x: { time: false,}, y: {} },
        series: [
            { label: t('chart.series.bin') },
            { label: t('chart.series.vol'),     stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: t('chart.series.va'),      stroke: '#ffd84a', width: 2.0, points: { show: false } },
            { label: t('chart.series.poc'),     stroke: '#ff3860', width: 0,   points: { show: true, size: 8 } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => {
                  const i = Math.trunc(v);
                  return i >= 0 && i < report.bins.length ? fmtUSD(report.bins[i].center) : '';
              })
            },
            { stroke: '#aab', size: 70,
              values: (_u, splits) => splits.map(v => fmtVol(v)) },
        ],
        legend: { show: true },
    }, [xs, ys, vaSeries, pocSeries], el);
}

function renderCumChart(report) {
    const el = document.getElementById('vap-cum-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const bins = (report.bins || []).filter(b => Number.isFinite(Number(b.volume)));
    const total = Number(report.total_volume);
    if (bins.length < 1 || !(total > 0)) {
        el.innerHTML = `<div class="muted" data-i18n="view.vap.empty_cum_chart">${esc(t('view.vap.empty_cum_chart'))}</div>`;
        return;
    }
    const sorted = [...bins].sort((a, b) => Number(a.center) - Number(b.center));
    let acc = 0;
    const cum = sorted.map(b => (acc += Number(b.volume) / total) * 100);
    const xs = sorted.map(b => Number(b.center));
    const half = xs.map(() => 50);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { range: [0, 100] } },
        series: [
            { label: t('view.vap.chart.price') },
            { label: t('view.vap.chart.cum_pct'),
              stroke: '#b86bff', width: 1.6, points: { show: false } },
            { label: t('view.vap.chart.half'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => fmtUSD(v)) },
            { stroke: '#aab', size: 50,
              values: (_u, splits) => splits.map(v => v.toFixed(0) + '%') },
        ],
        legend: { show: true },
    }, [xs, cum, half], el);
}

function renderTable(report) {
    const wrap = document.getElementById('vap-table');
    if (!report.bins || report.bins.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.vap.empty">${esc(t('view.vap.empty'))}</div>`;
        return;
    }
    const indexed = report.bins.map((b, i) => ({ ...b, idx: i }));
    indexed.sort((a, b) => b.volume - a.volume);
    const top = indexed.slice(0, 10);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.vap.col.rank">#</th>
                <th data-i18n="view.vap.col.price">Price</th>
                <th data-i18n="view.vap.col.volume">Volume</th>
                <th data-i18n="view.vap.col.share">% of total</th>
                <th data-i18n="view.vap.col.tag">Tag</th>
            </tr></thead>
            <tbody>
                ${top.map((b, rank) => {
                    const share = report.total_volume > 0 ? b.volume / report.total_volume : 0;
                    const inVA = report.value_area_low != null && report.value_area_high != null
                        && b.center >= report.value_area_low && b.center <= report.value_area_high;
                    const isPoc = b.idx === report.poc_index;
                    const tagKey = isPoc ? 'view.vap.tag.poc' : inVA ? 'view.vap.tag.va' : 'view.vap.tag.tail';
                    const cls = isPoc ? 'neg' : inVA ? 'pos' : '';
                    return `<tr>
                        <td>${rank + 1}</td>
                        <td><strong>${esc(fmtUSD(b.center))}</strong></td>
                        <td>${esc(fmtVol(b.volume))}</td>
                        <td>${esc(fmtPct(share))}</td>
                        <td data-i18n="${esc(tagKey)}" class="${cls}">${esc(t(tagKey))}</td>
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
    const el = document.getElementById('vap-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('vap-err').style.display = 'none'; }
