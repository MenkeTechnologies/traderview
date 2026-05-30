// CUSUM view — Page-Hinkley change-point detector.
//
// "When did this series quietly shift regime?" Maintains a running
// cumulative-deviation sum that resets on threshold hits. Each event
// marks a bar where the series demonstrated a sustained shift away
// from the reference mean.
//
// Use cases:
//   * Regime alert on returns: when did drift flip from + to −?
//   * Risk alert: when did vol-scaled returns enter a fat-tail regime?
//   * Execution: when did the order-flow imbalance series shift?

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseSeries, validateInputs, buildBody,
    meanStdev, eventMarkers, makeDemoSeries,
    fmtN, dirCss,
} from '../_cusum_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
const DEFAULT_CFG = {
    reference_mean: 0.0,
    reference_stdev: 1.0,
    threshold_stdevs: 5.0,
    slack: 0.5,
};

let state = { seriesText: '', config: { ...DEFAULT_CFG } };

export async function renderCusum(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.cusum.h1.cusum_change_point_detector" class="view-title">// CUSUM · CHANGE-POINT DETECTOR</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.cusum.h2.series">Series</h2>
            <p data-i18n="view.cusum.hint.paste_one_signed_value_per_line_typically_log_retu" class="muted">Paste one signed value per line — typically log-returns or
                vol-normalized returns. Demo loads 200 bars where mean flips at bar 100.</p>
            <textarea id="cu-series" rows="6" placeholder="0.0025&#10;-0.001&#10;0.004&#10;..." data-tip="view.cusum.tip.series"></textarea>
            <div class="inline-form">
                <button data-i18n="view.cusum.btn.load_demo_200_bars_regime_flip_100" data-tip="view.cusum.tip.demo" id="cu-demo" class="secondary" type="button">Load demo (200 bars, regime flip @ 100)</button>
                <button data-i18n="view.cusum.btn.clear" data-tip="view.cusum.tip.clear" id="cu-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.cusum.btn.auto_fit_mean_stdev_from_series" data-tip="view.cusum.tip.autofit" data-shortcut="cusum_autofit" id="cu-autofit" class="secondary" type="button">Auto-fit mean / stdev from series</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cusum.h2.config">Config</h2>
            <div class="inline-form">
                <label><span data-i18n="view.cusum.label.ref_mean">Reference mean</span>
                    <input id="cu-mean" type="number" step="any" value="${state.config.reference_mean}" data-tip="view.cusum.tip.mean"></label>
                <label><span data-i18n="view.cusum.label.ref_stdev">Reference stdev</span>
                    <input id="cu-sd" type="number" step="any" min="0" value="${state.config.reference_stdev}" data-tip="view.cusum.tip.sd"></label>
                <label><span data-i18n="view.cusum.label.threshold">Threshold (stdevs)</span>
                    <input id="cu-thr" type="number" step="any" min="0" value="${state.config.threshold_stdevs}" data-tip="view.cusum.tip.thr"></label>
                <label><span data-i18n="view.cusum.label.slack">Slack</span>
                    <input id="cu-slk" type="number" step="any" min="0" value="${state.config.slack}" data-tip="view.cusum.tip.slk"></label>
                <button data-i18n="view.cusum.btn.detect" data-tip="view.cusum.tip.detect" data-shortcut="cusum_detect" id="cu-run" class="primary" type="button">Detect</button>
            </div>
            <p data-i18n="view.cusum.hint.threshold_stdevs_reference_stdev_trigger_level_sla" class="muted">
                threshold_stdevs × reference_stdev = trigger level. Slack subtracts a
                small amount each bar to suppress noise (typically 0.5 × stdev).
                A high threshold + zero slack = chatty detector. Low threshold + large
                slack = laggy but stable.</p>
        </div>

        <div id="cu-errors" class="boot" style="display:none"></div>
        <div id="cu-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.cusum.h2.series_event_markers">Series + event markers</h2>
            <div id="cu-chart" style="height:280px"></div>
            <p data-i18n="view.cusum.hint.cyan_the_series_itself_green_dots_up_change_points" class="muted">Cyan = the series itself. Green dots = UP change-points
                (regime mean shifted higher). Red dots = DOWN change-points (mean shifted lower).
                Marker height = CUSUM value at the firing bar (how hard the threshold was busted).</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cusum.h2.detector_state_chart">CUSUM detector state — G+ and G− vs threshold band</h2>
            <div id="cu-state-chart" style="height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cusum.h2.event_log">Event log</h2>
            <div id="cu-events"></div>
        </div>

        <div id="cu-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('cu-demo').addEventListener('click', () => {
        const xs = makeDemoSeries(42);
        document.getElementById('cu-series').value = xs.join('\n');
    });
    document.getElementById('cu-clear').addEventListener('click', () => {
        document.getElementById('cu-series').value = '';
    });
    document.getElementById('cu-autofit').addEventListener('click', () => {
        const { value: xs } = parseSeries(document.getElementById('cu-series').value);
        const { mean, stdev } = meanStdev(xs);
        if (Number.isFinite(mean))  document.getElementById('cu-mean').value = mean.toFixed(6);
        if (Number.isFinite(stdev)) document.getElementById('cu-sd').value   = stdev.toFixed(6);
    });
    document.getElementById('cu-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.seriesText = document.getElementById('cu-series').value;
    state.config = {
        reference_mean:   Number(document.getElementById('cu-mean').value),
        reference_stdev:  Number(document.getElementById('cu-sd').value),
        threshold_stdevs: Number(document.getElementById('cu-thr').value),
        slack:            Number(document.getElementById('cu-slk').value),
    };
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('cu-errors');
    errs.style.display = 'none';
    const { value: series, errors } = parseSeries(state.seriesText);
    if (errors.length) {
        const head = errors.slice(0, 6).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 6 ? `<br>… and ${errors.length - 6} more.` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
    }
    const err = validateInputs(series, state.config);
    if (err) { showErr(err); showToast(err, { level: 'warning' }); return; }
    let res;
    try {
        res = await api.anlyCusum(buildBody(series, state.config));
    } catch (e) {
        const m = t("common.error.api", { msg: e.message || e });
        showErr(m); showToast(m, { level: 'error' }); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res, series);
    renderChart(series, res);
    renderStateChart(series);
    renderEvents(res);
    const events = res.events || [];
    showToast(t('view.cusum.toast.done', {
        bars: series.length,
        events: events.length,
    }), { level: events.length > 0 ? 'success' : 'info' });
}

function renderSummary(report, series) {
    const events = report.events || [];
    const upCount = events.filter(e => e.direction === 'up').length;
    const dnCount = events.filter(e => e.direction === 'down').length;
    const lastEvent = events[events.length - 1];
    document.getElementById('cu-summary').innerHTML = [
        card(t('view.cusum.card.bars'),          String(series.length)),
        card(t('view.cusum.card.events'),        String(report.n_events || 0),
            (report.n_events || 0) > 0 ? '' : 'pos'),
        card(t('view.cusum.card.up_events'),     String(upCount), upCount > 0 ? 'pos' : ''),
        card(t('view.cusum.card.down_events'),   String(dnCount), dnCount > 0 ? 'neg' : ''),
        card(t('view.cusum.card.final_g'),      fmtN(report.final_g_pos)),
        card(t('view.cusum.card.final_g_2'),      fmtN(report.final_g_neg)),
        card(t('view.cusum.card.last_event'),    lastEvent
            ? `bar ${lastEvent.bar_index} ${lastEvent.direction.toUpperCase()}`
            : '—',
            lastEvent ? dirCss(lastEvent.direction) : ''),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(series, report) {
    if (!window.uPlot) return;
    const el = document.getElementById('cu-chart');
    const xs = series.map((_, i) => i);
    const { up, dn } = eventMarkers(report.events, series.length);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 280,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.bar_num') },
            { label: t('chart.series.series'), stroke: '#00e5ff', width: 1.2,
              fill: '#00e5ff14', points: { show: false } },
            { label: t('chart.series.up_event'), stroke: '#39ff14', width: 0,
              points: { show: true, size: 10, stroke: '#39ff14', fill: '#39ff14' } },
            { label: t('chart.series.down_event'), stroke: '#ff3860', width: 0,
              points: { show: true, size: 10, stroke: '#ff3860', fill: '#ff3860' } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, series, up, dn], el);
}

function renderStateChart(series) {
    if (!window.uPlot) return;
    const el = document.getElementById('cu-state-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!Array.isArray(series) || series.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.cusum.empty_state_chart">${esc(t('view.cusum.empty_state_chart'))}</div>`;
        return;
    }
    const cfg = state.config;
    const xs = series.map((_, i) => i);
    const slack = cfg.slack * cfg.reference_stdev;
    const trig = cfg.threshold_stdevs * cfg.reference_stdev;
    const gp = new Array(series.length).fill(0);
    const gn = new Array(series.length).fill(0);
    let pos = 0, neg = 0;
    for (let i = 0; i < series.length; i++) {
        const dev = series[i] - cfg.reference_mean;
        pos = Math.max(0, pos + dev - slack);
        neg = Math.min(0, neg + dev + slack);
        if (pos >= trig) pos = 0;
        if (neg <= -trig) neg = 0;
        gp[i] = pos;
        gn[i] = neg;
    }
    const upBand = xs.map(() => trig);
    const dnBand = xs.map(() => -trig);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('chart.series.bar_num') },
            { label: t('view.cusum.series.g_pos'),
              stroke: '#7af0a8', width: 1.4, points: { show: false } },
            { label: t('view.cusum.series.g_neg'),
              stroke: '#ff3860', width: 1.4, points: { show: false } },
            { label: t('view.cusum.series.up_threshold'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('view.cusum.series.dn_threshold'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, gp, gn, upBand, dnBand], el);
}

function renderEvents(report) {
    const wrap = document.getElementById('cu-events');
    const events = report.events || [];
    if (!events.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.cusum.empty.events">No change-point events at current threshold.</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr><th>#</th><th data-i18n="view.cusum.th.bar_index">Bar index</th><th data-i18n="view.cusum.th.direction">Direction</th><th data-i18n="view.cusum.th.cusum_value">CUSUM value</th></tr></thead>
            <tbody>
                ${events.map((e, i) => `<tr>
                    <td>${i + 1}</td>
                    <td>${esc(String(e.bar_index))}</td>
                    <td class="${dirCss(e.direction)}">${esc(String(e.direction || '').toUpperCase())}</td>
                    <td>${esc(fmtN(e.cusum_value))}</td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('cu-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cu-err').style.display = 'none'; }
