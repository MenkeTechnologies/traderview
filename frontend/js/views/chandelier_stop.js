// Chandelier Stop view — Chuck LeBeau's ATR-based trailing stop.
//
// Stop hangs from the highest high (longs) or lowest low (shorts) over
// the lookback window minus N × ATR. Ratchets in the favorable direction
// only; never widens. Triggers when price crosses through.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarBlob, computeAtr, validateInputs, buildBody,
    splitStops, triggerMarkers, summarize,
    makeDemoBars, fmtN, fmtPct,
} from '../_chandelier_stop_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
const DEFAULT_CFG = { lookback: 22, atr_multiplier: 3.0 };
const DEFAULT_ATR_PERIOD = 14;

let state = { barText: '', side: 'long', atrPeriod: DEFAULT_ATR_PERIOD, config: { ...DEFAULT_CFG } };

export async function renderChandelierStop(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.chandelier_stop.h1.chandelier_stop" class="view-title">// CHANDELIER STOP</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.chandelier_stop.h2.hlc_bars">HLC bars</h2>
            <p class="muted" data-i18n-html="view.chandelier_stop.help">Paste <code>high low close</code> per line. ATR computed
                locally (Wilder smoothing). Demo loads 60 bars: 40-bar uptrend → 20-bar
                reversal — long stop should trigger on the way down.</p>
            <textarea id="cs-bars" rows="6" placeholder="100.5 99.5 100.0&#10;101.0 100.0 100.5&#10;..." data-tip="view.chandelier_stop.tip.bars"></textarea>
            <div class="inline-form">
                <button data-i18n="view.chandelier_stop.btn.load_demo_60_bars_uptrend_reversal" id="cs-demo" class="secondary" type="button" data-tip="view.chandelier_stop.tip.demo" data-shortcut="chandelier_stop_demo">Load demo (60 bars, uptrend → reversal)</button>
                <button data-i18n="view.chandelier_stop.btn.clear" id="cs-clear" class="secondary" type="button" data-tip="view.chandelier_stop.tip.clear">Clear</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.chandelier_stop.h2.config">Config</h2>
            <div class="inline-form">
                <label><span data-i18n="view.chandelier_stop.label.side">Side</span>
                    <select id="cs-side" data-tip="view.chandelier_stop.tip.side">
                        <option data-i18n="view.chandelier_stop.opt.long" value="long"  ${state.side === 'long'  ? 'selected' : ''}>Long</option>
                        <option data-i18n="view.chandelier_stop.opt.short" value="short" ${state.side === 'short' ? 'selected' : ''}>Short</option>
                    </select></label>
                <label><span data-i18n="view.chandelier_stop.label.atr_period">ATR period</span>
                    <input id="cs-atrp" type="number" step="1" min="1" value="${state.atrPeriod}" data-tip="view.chandelier_stop.tip.atr_period"></label>
                <label><span data-i18n="view.chandelier_stop.label.lookback">Lookback bars</span>
                    <input id="cs-lb"  type="number" step="1" min="1" value="${state.config.lookback}" data-tip="view.chandelier_stop.tip.lookback"></label>
                <label><span data-i18n="view.chandelier_stop.label.multiplier">ATR multiplier</span>
                    <input id="cs-mul" type="number" step="0.1" min="0" value="${state.config.atr_multiplier}" data-tip="view.chandelier_stop.tip.multiplier"></label>
                <button data-i18n="view.chandelier_stop.btn.compute" id="cs-run" class="primary" type="button" data-tip="view.chandelier_stop.tip.run" data-shortcut="chandelier_stop_run">Compute</button>
            </div>
            <p data-i18n="view.chandelier_stop.hint.lebeau_defaults_atr_22_lookback_3_0_multiplier_tig" class="muted">LeBeau defaults: ATR-22 lookback, 3.0× multiplier. Tighter
                multipliers (2.0×) trail closer but trigger on noise; looser (4.0×) catch
                only structural reversals.</p>
        </div>

        <div id="cs-errors" class="boot" style="display:none"></div>
        <div id="cs-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.chandelier_stop.h2.close_trailing_stop_trigger_markers">Close + trailing stop + trigger markers</h2>
            <div id="cs-chart" style="height:320px"></div>
            <p data-i18n="view.chandelier_stop.hint.cyan_close_yellow_chandelier_stop_ratcheted_red_do" class="muted">Cyan = close. Yellow = chandelier stop (ratcheted). Red dot =
                trigger fired on this bar. Long stop trails BELOW price; short stop trails ABOVE.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.chandelier_stop.h2.atr_chart">ATR (volatility regime driving the stop width)</h2>
            <div id="cs-atr-chart" style="height:220px"></div>
        </div>

        <div id="cs-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('cs-demo').addEventListener('click', () => {
        const b = makeDemoBars();
        document.getElementById('cs-bars').value =
            b.map(x => `${x.high} ${x.low} ${x.close}`).join('\n');
        showToast(t('view.chandelier_stop.toast.demo_loaded', { n: b.length }), { level: 'info' });
    });
    document.getElementById('cs-clear').addEventListener('click', () => {
        document.getElementById('cs-bars').value = '';
        showToast(t('view.chandelier_stop.toast.cleared'), { level: 'info' });
    });
    document.getElementById('cs-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.barText = document.getElementById('cs-bars').value;
    state.side = document.getElementById('cs-side').value;
    state.atrPeriod = parseInt(document.getElementById('cs-atrp').value, 10);
    state.config = {
        lookback: parseInt(document.getElementById('cs-lb').value, 10),
        atr_multiplier: Number(document.getElementById('cs-mul').value),
    };
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('cs-errors');
    errs.style.display = 'none';
    const { bars, errors } = parseBarBlob(state.barText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        showToast(t('view.chandelier_stop.toast.parse_error', { n: errors.length }), { level: 'warning' });
        if (bars.length === 0) return;
    }
    // Compute ATR locally so the backend gets a parallel-length array.
    const atr = computeAtr(bars, state.atrPeriod);
    // Replace NaN warmup entries with 0 so the backend accepts the array
    // (it length-checks but not for NaN; backend would then output NaN
    // stops which we already null-pad in splitStops).
    const safeAtr = atr.map(v => Number.isFinite(v) ? v : 0);
    const err = validateInputs(bars, safeAtr, state.side, state.config);
    if (err) { showErr(err); showToast(t('view.chandelier_stop.toast.invalid'), { level: 'warning' }); return; }

    let stops;
    try {
        stops = await api.discChandelierStop(buildBody(bars, safeAtr, state.side, state.config));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        showToast(t('view.chandelier_stop.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(stops || [], bars);
    renderChart(bars, stops || []);
    renderAtrChart(bars, atr);
    const arr = stops || [];
    const triggers = arr.filter(s => s && s.triggered).length;
    showToast(t('view.chandelier_stop.toast.computed', { bars: bars.length, triggers }), { level: triggers > 0 ? 'warning' : 'success' });
}

function renderSummary(stops, bars) {
    const s = summarize(stops, bars, state.side);
    document.getElementById('cs-summary').innerHTML = [
        card(t('view.chandelier_stop.card.bars'),          String(bars.length)),
        card(t('view.chandelier_stop.card.latest_close'),  fmtN(s.latestClose)),
        card(t('view.chandelier_stop.card.latest_stop'),   fmtN(s.latestStop)),
        card(t('view.chandelier_stop.card.distance'),      fmtPct(s.distancePct),
            s.distancePct >= 0 ? 'pos' : 'neg'),
        card(t('view.chandelier_stop.card.triggers'),      String(s.triggerCount), s.triggerCount ? 'neg' : 'pos'),
        card(t('view.chandelier_stop.card.first_trigger'), s.firstTriggerIdx >= 0 ? `bar ${s.firstTriggerIdx}` : '—',
            s.firstTriggerIdx >= 0 ? 'neg' : ''),
        card(t('view.chandelier_stop.card.side'),          state.side.toUpperCase(), state.side === 'long' ? 'pos' : 'neg'),
        card(t('view.chandelier_stop.card.stop_math'),     `${state.config.lookback}-bar HH/LL ± ${state.config.atr_multiplier}× ATR`),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(bars, stops) {
    if (!window.uPlot) return;
    const el = document.getElementById('cs-chart');
    const xs = bars.map((_, i) => i);
    const closes = bars.map(b => b.close);
    const { stopPrice } = splitStops(stops);
    const triggers = triggerMarkers(stops, bars);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: { time: false,}, y: {} },
        series: [
            { label: t('chart.series.bar_num') },
            { label: t('chart.series.close'), stroke: '#00e5ff', width: 1.2,
              fill: '#00e5ff14', points: { show: false } },
            { label: t('chart.series.stop'),  stroke: '#ffd84a', width: 1.5,
              points: { show: false } },
            { label: t('chart.series.trigger'), stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, stroke: '#ff3860', fill: '#ff3860' } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, closes, stopPrice, triggers], el);
}

function renderAtrChart(bars, atr) {
    if (!window.uPlot) return;
    const el = document.getElementById('cs-atr-chart');
    if (!el) return;
    if (!bars || bars.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.chandelier_stop.empty_atr_chart">${esc(t('view.chandelier_stop.empty_atr_chart'))}</div>`;
        return;
    }
    const xs = bars.map((_, i) => i);
    const atrSeries = atr.map(v => Number.isFinite(v) ? v : null);
    const mult = atrSeries.map(v => v == null ? null : v * state.config.atr_multiplier);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('chart.series.bar_num') },
            { label: t('view.chandelier_stop.series.atr'),
              stroke: '#00e5ff', width: 1.5,
              points: { show: false } },
            { label: t('view.chandelier_stop.series.atr_mult'),
              stroke: '#ffd84a', width: 1.2, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, atrSeries, mult], el);
}

function showErr(msg) {
    const el = document.getElementById('cs-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cs-err').style.display = 'none'; }
