// Cup-and-Handle view — IBD-style pattern detector.
//
// Pipeline:
//   1. Paste OHLC bars (high low close per line).
//   2. Configure cup / handle bar windows + depth tolerances.
//   3. Backend walks every candidate handle/cup window; returns the most
//      recent qualifying pattern (or null).
//   4. View charts the close series with markers at the left rim,
//      trough, right rim, handle low, and the IBD pivot.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarBlob, validateInputs, buildBody,
    makeDemoBars, fmtN, fmtPct, depthQuality,
} from '../_cup_and_handle_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
const DEFAULT_CONFIG = {
    cup_min_bars: 30,
    cup_max_bars: 250,
    min_depth_pct: 0.10,
    max_depth_pct: 0.33,
    rim_tolerance_pct: 0.05,
    handle_min_bars: 5,
    handle_max_bars: 25,
    max_handle_depth_pct: 0.15,
};

let state = { config: { ...DEFAULT_CONFIG }, barText: '' };

export async function renderCupAndHandle(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.cup_and_handle.h1.cup_and_handle_ibd_pattern_detector" class="view-title">// CUP &amp; HANDLE · IBD PATTERN DETECTOR</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.cup_and_handle.h2.ohlc_bars">OHLC bars</h2>
            <p class="muted" data-i18n-html="view.cup_and_handle.help">Paste <code>high low close</code> per line.
                Demo loads a synthetic 122-bar cup with an 8% handle that
                triggers the canonical IBD pivot buy-point.</p>
            <textarea id="ch-bars" rows="8" placeholder="100.50 99.20 100.10&#10;101.30 100.00 100.85&#10;..." data-tip="view.cup_and_handle.tip.bars"></textarea>
            <div class="inline-form">
                <button data-i18n="view.cup_and_handle.btn.load_demo_122_bars" data-tip="view.cup_and_handle.tip.demo" data-shortcut="cup_and_handle_demo" id="ch-demo" class="secondary" type="button">Load demo (122 bars)</button>
                <button data-i18n="view.cup_and_handle.btn.clear" data-tip="view.cup_and_handle.tip.clear" id="ch-clear" class="secondary" type="button">Clear</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cup_and_handle.h2.config">Config</h2>
            <div class="inline-form">
                <label><span data-i18n="view.cup_and_handle.label.cup_min">Cup bars (min)</span>
                    <input id="ch-cmin" type="number" step="1" min="4" value="${state.config.cup_min_bars}"></label>
                <label><span data-i18n="view.cup_and_handle.label.cup_max">Cup bars (max)</span>
                    <input id="ch-cmax" type="number" step="1" min="5" value="${state.config.cup_max_bars}"></label>
                <label><span data-i18n="view.cup_and_handle.label.min_depth">Min depth %</span>
                    <input id="ch-dmin" type="number" step="0.01" min="0" max="1" value="${state.config.min_depth_pct}"></label>
                <label><span data-i18n="view.cup_and_handle.label.max_depth">Max depth %</span>
                    <input id="ch-dmax" type="number" step="0.01" min="0" max="1" value="${state.config.max_depth_pct}"></label>
                <label><span data-i18n="view.cup_and_handle.label.rim_tolerance">Rim tolerance %</span>
                    <input id="ch-rim" type="number" step="0.01" min="0" max="1" value="${state.config.rim_tolerance_pct}"></label>
                <label><span data-i18n="view.cup_and_handle.label.handle_min">Handle bars (min)</span>
                    <input id="ch-hmin" type="number" step="1" min="1" value="${state.config.handle_min_bars}"></label>
                <label><span data-i18n="view.cup_and_handle.label.handle_max">Handle bars (max)</span>
                    <input id="ch-hmax" type="number" step="1" min="1" value="${state.config.handle_max_bars}"></label>
                <label><span data-i18n="view.cup_and_handle.label.handle_depth">Max handle depth %</span>
                    <input id="ch-hdep" type="number" step="0.01" min="0" max="1" value="${state.config.max_handle_depth_pct}"></label>
                <button data-i18n="view.cup_and_handle.btn.detect" data-tip="view.cup_and_handle.tip.detect" data-shortcut="cup_and_handle_detect" id="ch-run" class="primary" type="button">Detect</button>
            </div>
            <p data-i18n="view.cup_and_handle.hint.canonical_ibd_defaults_30_250_cup_bars_10_33_depth" class="muted">Canonical IBD defaults: 30-250 cup bars, 10-33% depth, ≤5% rim asymmetry,
                5-25 handle bars, ≤15% handle dip. Loosen depth/handle to surface near-misses
                during base-building.</p>
        </div>

        <div id="ch-errors" class="boot" style="display:none"></div>
        <div id="ch-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.cup_and_handle.h2.close_series_pattern_markers">Close series + pattern markers</h2>
            <div id="ch-chart" style="height:300px"></div>
            <p data-i18n="view.cup_and_handle.hint.yellow_left_rim_magenta_trough_cyan_right_rim_oran" class="muted">Yellow = left rim. Magenta = trough. Cyan = right rim. Orange =
                handle low. Green dashed = pivot (IBD buy-point = right-rim high).</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cup_and_handle.h2.cup_profile_chart">Cup profile — closes normalized within the cup span (curvature symmetry)</h2>
            <div id="ch-profile-chart" style="height:220px"></div>
        </div>

        <div id="ch-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('ch-demo').addEventListener('click', () => {
        const bars = makeDemoBars(7);
        document.getElementById('ch-bars').value =
            bars.map(b => `${b.high} ${b.low} ${b.close}`).join('\n');
    });
    document.getElementById('ch-clear').addEventListener('click', () => {
        document.getElementById('ch-bars').value = '';
    });
    document.getElementById('ch-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.barText = document.getElementById('ch-bars').value;
    state.config = {
        cup_min_bars:         parseInt(document.getElementById('ch-cmin').value, 10),
        cup_max_bars:         parseInt(document.getElementById('ch-cmax').value, 10),
        min_depth_pct:        Number(document.getElementById('ch-dmin').value),
        max_depth_pct:        Number(document.getElementById('ch-dmax').value),
        rim_tolerance_pct:    Number(document.getElementById('ch-rim').value),
        handle_min_bars:      parseInt(document.getElementById('ch-hmin').value, 10),
        handle_max_bars:      parseInt(document.getElementById('ch-hmax').value, 10),
        max_handle_depth_pct: Number(document.getElementById('ch-hdep').value),
    };
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('ch-errors');
    errs.style.display = 'none';

    const { bars, errors } = parseBarBlob(state.barText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
    }
    const err = validateInputs(bars, state.config);
    if (err) { showErr(err); showToast(err, { level: 'warning' }); return; }

    let res;
    try {
        res = await api.anlyCupAndHandle(buildBody(bars, state.config));
    } catch (e) {
        const m = t("common.error.api", { msg: e.message || e });
        showErr(m); showToast(m, { level: 'error' }); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res, bars);
    renderChart(bars, res);
    renderProfileChart(bars, res);
    showToast(t('view.cup_and_handle.toast.done', {
        bars: bars.length,
        verdict: res ? t('view.cup_and_handle.verdict.found') : t('view.cup_and_handle.verdict.none'),
    }), { level: res ? 'success' : 'info' });
}

function renderSummary(cand, bars) {
    if (!cand) {
        document.getElementById('ch-summary').innerHTML = [
            card(t('view.cup_and_handle.card.pattern'), 'NONE', 'neg'),
            card(t('view.cup_and_handle.card.bars_scanned'), String(bars.length)),
            card(t('view.cup_and_handle.card.hint'), 'try loosening depth / handle bounds'),
        ].join('');
        return;
    }
    const q = depthQuality(cand.depth_pct);
    const handleQ = cand.handle_depth_pct < 0.05
        ? { label: t('chart.series.tight_5'), cls: 'pos' }
        : cand.handle_depth_pct <= 0.15
            ? { label: t('chart.series.normal_515'), cls: '' }
            : { label: t('chart.series.wide_15'), cls: 'neg' };
    document.getElementById('ch-summary').innerHTML = [
        card(t('view.cup_and_handle.card.pattern_2'),        'DETECTED', 'pos'),
        card(t('view.cup_and_handle.card.cup_depth'),      fmtPct(cand.depth_pct) + ' · ' + q.label, q.cls),
        card(t('view.cup_and_handle.card.handle_depth'),   fmtPct(cand.handle_depth_pct) + ' · ' + handleQ.label, handleQ.cls),
        card(t('view.cup_and_handle.card.cup_length'),     String(cand.right_rim_index - cand.left_rim_index) + ' bars'),
        card(t('view.cup_and_handle.card.handle_length'),  String(cand.last_index - cand.right_rim_index) + ' bars'),
        card(t('view.cup_and_handle.card.left_rim_d'),   fmtN(cand.left_rim_price)),
        card(t('view.cup_and_handle.card.right_rim_d'),  fmtN(cand.right_rim_price)),
        card(t('view.cup_and_handle.card.trough_d'),     fmtN(cand.trough_price)),
        card(t('view.cup_and_handle.card.handle_low_d'), fmtN(cand.handle_low_price)),
        card(t('view.cup_and_handle.card.pivot_d'),      fmtN(cand.pivot_price), 'pos'),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(bars, cand) {
    if (!window.uPlot) return;
    const el = document.getElementById('ch-chart');
    const xs = bars.map((_, i) => i);
    const closes = bars.map(b => b.close);

    // Marker series: one value at the pattern index, null elsewhere.
    const marker = (idx, price) => xs.map((_, i) => i === idx ? price : null);

    const series = [
        { label: t('chart.series.bar_num') },
        { label: t('chart.series.close'), stroke: '#aab', width: 1.0,
          fill: '#aab1A', points: { show: false } },
    ];
    const data = [xs, closes];

    if (cand) {
        series.push(
            { label: t('chart.series.left_rim'),    stroke: '#ffd84a', width: 0, points: { show: true, size: 10, stroke: '#ffd84a', fill: '#ffd84a' } },
            { label: t('chart.series.trough'),      stroke: '#ff3860', width: 0, points: { show: true, size: 10, stroke: '#ff3860', fill: '#ff3860' } },
            { label: t('chart.series.right_rim'),   stroke: '#00e5ff', width: 0, points: { show: true, size: 10, stroke: '#00e5ff', fill: '#00e5ff' } },
            { label: t('chart.series.handle_low'),  stroke: '#ff9f1a', width: 0, points: { show: true, size: 10, stroke: '#ff9f1a', fill: '#ff9f1a' } },
            { label: t('chart.series.pivot'),       stroke: '#39ff14', width: 1.0, dash: [4, 4], points: { show: false } },
        );
        data.push(
            marker(cand.left_rim_index,   cand.left_rim_price),
            marker(cand.trough_index,     cand.trough_price),
            marker(cand.right_rim_index,  cand.right_rim_price),
            marker(cand.handle_low_index, cand.handle_low_price),
            xs.map(() => cand.pivot_price),
        );
    }

    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 300,
        scales: { x: {}, y: {} },
        series,
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, data, el);
}

function renderProfileChart(bars, cand) {
    if (!window.uPlot) return;
    const el = document.getElementById('ch-profile-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!cand) {
        el.innerHTML = `<div class="muted" data-i18n="view.cup_and_handle.empty_profile_chart">${esc(t('view.cup_and_handle.empty_profile_chart'))}</div>`;
        return;
    }
    const lo = Math.max(0, Math.min(bars.length - 1, cand.left_rim_index));
    const hi = Math.max(lo, Math.min(bars.length - 1, cand.right_rim_index));
    if (hi - lo < 2) {
        el.innerHTML = `<div class="muted" data-i18n="view.cup_and_handle.empty_profile_chart">${esc(t('view.cup_and_handle.empty_profile_chart'))}</div>`;
        return;
    }
    const slice = bars.slice(lo, hi + 1).map(b => b.close);
    const minV = Math.min(...slice);
    const maxV = Math.max(...slice);
    const range = maxV - minV;
    if (!(range > 0)) {
        el.innerHTML = `<div class="muted" data-i18n="view.cup_and_handle.empty_profile_chart">${esc(t('view.cup_and_handle.empty_profile_chart'))}</div>`;
        return;
    }
    const len = slice.length;
    const xs = slice.map((_, i) => i / (len - 1));
    const ys = slice.map(v => (v - minV) / range);
    const mid = xs.map(() => 0.5);
    const symmetryAxis = xs.map(() => 0.5);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { range: [0, 1] }, y: { range: [-0.05, 1.05] } },
        series: [
            { label: t('view.cup_and_handle.chart.position') },
            { label: t('view.cup_and_handle.chart.norm_close'),
              stroke: '#00e5ff', width: 1.5,
              fill: 'rgba(0,229,255,0.10)',
              points: { show: false } },
            { label: t('view.cup_and_handle.chart.mid_y'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('view.cup_and_handle.chart.symmetry_x'),
              stroke: '#7af0a8', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, ys, mid, symmetryAxis], el);
}

function showErr(msg) {
    const el = document.getElementById('ch-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ch-err').style.display = 'none'; }
