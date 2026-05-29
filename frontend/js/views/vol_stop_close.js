// Vol-Stop (close-based) view — Chandelier exit but referenced to
// highest CLOSE not highest HIGH. Compare side-by-side to vanilla
// Chandelier so the user can see how wick-protection plays out.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarBlob, computeAtr, validateInputs, buildBody,
    splitStops, triggerMarkers, summarize, fmtN, fmtPct,
    localVolStopClose, localChandelier, compareStops, makeDemoBars,
} from '../_vol_stop_close_inputs.js';

import { t } from '../i18n.js';
let state = {
    bars: makeDemoBars('wicks'),
    side: 'long',
    cfg: { lookback: 22, atr_multiplier: 3.0 },
    atrPeriod: 14,
};

export async function renderVolStopClose(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.vol_stop_close.h1.vol_stop_close_based" class="view-title">// VOL-STOP (CLOSE-BASED)</h1>

        <div class="chart-panel">
            <h2><span data-i18n="view.vol_stop_close.h2.paste">Paste HLC bars (one per line:</span> <code>high low close</code>)</h2>
            <textarea id="vsc-blob" rows="6" placeholder="100.5 99.5 100.0&#10;101.0 100.0 100.6&#10;...">${esc(barsToBlob(state.bars))}</textarea>
            <div class="inline-form">
                <label><span data-i18n="view.vol_stop_close.label.side">Side</span>
                    <select id="vsc-side">
                        <option data-i18n="view.vol_stop_close.opt.long" value="long"  ${state.side === 'long'  ? 'selected' : ''}>Long</option>
                        <option data-i18n="view.vol_stop_close.opt.short" value="short" ${state.side === 'short' ? 'selected' : ''}>Short</option>
                    </select></label>
                <label><span data-i18n="view.vol_stop_close.label.lookback">Lookback (bars)</span>
                    <input id="vsc-lb" type="number" step="1" min="1" value="${state.cfg.lookback}"></label>
                <label><span data-i18n="view.vol_stop_close.label.atr_multiplier">ATR multiplier</span>
                    <input id="vsc-mult" type="number" step="any" min="0" value="${state.cfg.atr_multiplier}"></label>
                <label><span data-i18n="view.vol_stop_close.label.atr_period">ATR period</span>
                    <input id="vsc-atr" type="number" step="1" min="1" value="${state.atrPeriod}"></label>
                <button data-i18n="view.vol_stop_close.btn.compute" id="vsc-run" class="primary" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.vol_stop_close.btn.demo_wick_spike_long" id="vsc-demo-wicks"    class="secondary" type="button">Demo: wick-spike (long)</button>
                <button data-i18n="view.vol_stop_close.btn.demo_uptrend_reversal" id="vsc-demo-rev"      class="secondary" type="button">Demo: uptrend → reversal</button>
                <button data-i18n="view.vol_stop_close.btn.demo_downtrend_bounce" id="vsc-demo-down"     class="secondary" type="button">Demo: downtrend → bounce</button>
                <button data-i18n="view.vol_stop_close.btn.demo_chop_with_wicks" id="vsc-demo-chop"     class="secondary" type="button">Demo: chop with wicks</button>
            </div>
            <p data-i18n="view.vol_stop_close.hint.both_stops_use_the_same_atr_multiplier_close_based" class="muted">Both stops use the same ATR + multiplier. Close-based references highest CLOSE in window; Chandelier references highest HIGH. On the wick-spike demo, watch Chandelier ratchet up to the wick while close-based ignores it.</p>
        </div>

        <div id="vsc-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.vol_stop_close.h2.close_vs_both_stops">Close vs both stops</h2>
            <div id="vsc-chart" style="height:340px"></div>
            <p data-i18n="view.vol_stop_close.hint.cyan_close_yellow_close_based_vol_stop_red_dashed_" class="muted">Cyan = close. Yellow = close-based vol-stop. Red dashed = chandelier (high-based). Red dots = triggers (filled = close-based, hollow = chandelier).</p>
        </div>

        <div id="vsc-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state.bars = makeDemoBars(k);
        state.side = k === 'downtrend' ? 'short' : 'long';
        document.getElementById('vsc-blob').value = barsToBlob(state.bars);
        document.getElementById('vsc-side').value = state.side;
    };
    document.getElementById('vsc-demo-wicks').addEventListener('click', () => loadDemo('wicks'));
    document.getElementById('vsc-demo-rev').addEventListener('click',   () => loadDemo('uptrend-reverse'));
    document.getElementById('vsc-demo-down').addEventListener('click',  () => loadDemo('downtrend'));
    document.getElementById('vsc-demo-chop').addEventListener('click',  () => loadDemo('chop'));
    document.getElementById('vsc-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function barsToBlob(bars) {
    return bars.map(b => `${b.high} ${b.low} ${b.close}`).join('\n');
}

function readInputs() {
    const parsed = parseBarBlob(document.getElementById('vsc-blob').value);
    if (parsed.errors.length) {
        showErr(`Parse errors: ${parsed.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; ')}`);
    } else {
        hideErr();
        state.bars = parsed.bars;
    }
    state.side = document.getElementById('vsc-side').value;
    state.cfg = {
        lookback: parseInt(document.getElementById('vsc-lb').value, 10),
        atr_multiplier: Number(document.getElementById('vsc-mult').value),
    };
    state.atrPeriod = parseInt(document.getElementById('vsc-atr').value, 10);
}

async function compute(tok) {
    if (!state.bars.length) return;
    const atr = computeAtr(state.bars, state.atrPeriod).map(v => Number.isFinite(v) ? v : 0);
    const err = validateInputs(state.bars, atr, state.side, state.cfg);
    if (err) { showErr(err); return; }
    hideErr();
    const localClose = localVolStopClose(state.bars, atr, state.side, state.cfg);
    const localChand = localChandelier(state.bars, atr, state.side, state.cfg);
    renderSummary(localClose, localChand, true);
    renderChart(state.bars, localClose, localChand);
    let resp;
    try {
        resp = await api.discVolStopClose(buildBody(state.bars, atr, state.side, state.cfg));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, localChand, false);
    renderChart(state.bars, resp, localChand);
}

function renderSummary(closeStops, chandStops, pending) {
    const summClose = summarize(closeStops, state.bars, state.side);
    const summChand = summarize(chandStops, state.bars, state.side);
    const cmp = compareStops(chandStops, closeStops);
    const parity = closeStops.length === chandStops.length;
    document.getElementById('vsc-summary').innerHTML = [
        card(t('view.vol_stop_close.card.latest_close_stop'),  fmtN(summClose.latestStop, 2) + (pending ? ' (local)' : ''), 'pos'),
        card(t('view.vol_stop_close.card.latest_chand_stop'),  fmtN(summChand.latestStop, 2), 'neg'),
        card(t('view.vol_stop_close.card.stop_spread'),        fmtN(cmp.diff, 2) + ' (' + fmtPct(cmp.diffPct) + ')',
            Math.abs(cmp.diffPct) > 0.005 ? 'neg' : 'pos'),
        card(t('view.vol_stop_close.card.latest_close'),       fmtN(summClose.latestClose, 2)),
        card(t('view.vol_stop_close.card.distance_close_stop_close'), fmtPct(summClose.distancePct), 'pos'),
        card(t('view.vol_stop_close.card.distance_chand_stop_close'), fmtPct(summChand.distancePct)),
        card(t('view.vol_stop_close.card.triggers_close'),   String(summClose.triggerCount),
            summClose.triggerCount > 0 ? 'neg' : 'pos'),
        card(t('view.vol_stop_close.card.triggers_chand'),   String(summChand.triggerCount),
            summChand.triggerCount > 0 ? 'neg' : 'pos'),
        card(t('view.vol_stop_close.card.agreement_bars'),     String(cmp.agreement)),
        card(t('view.vol_stop_close.card.disagreement_bars'),  String(cmp.disagreement),
            cmp.disagreement === 0 ? 'pos' : 'neg'),
        card(t('view.vol_stop_close.card.side'),               state.side.toUpperCase(),
            state.side === 'long' ? 'pos' : 'neg'),
        card(t('view.vol_stop_close.card.sample_size'),        String(state.bars.length)),
        card(t('view.vol_stop_close.card.parity_lengths'),   parity ? 'OK' : 'DIVERGED', parity ? 'pos' : 'neg'),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(bars, closeStops, chandStops) {
    if (!window.uPlot) return;
    const el = document.getElementById('vsc-chart');
    if (!el) return;
    el.innerHTML = '';
    const xs = bars.map((_, i) => i);
    const closes = bars.map(b => b.close);
    const closeStopLine = splitStops(closeStops).stopPrice;
    const chandStopLine = splitStops(chandStops).stopPrice;
    const closeTrigs = triggerMarkers(closeStops, bars);
    const chandTrigs = triggerMarkers(chandStops, bars);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bar #' },
            { label: 'close',          stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: 'close-vol-stop', stroke: '#ffd84a', width: 1.5, points: { show: false } },
            { label: 'chandelier',     stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: 'close trig',     stroke: '#ff3860', width: 0,
              points: { show: true, size: 9, fill: '#ff3860', stroke: '#ff3860' } },
            { label: 'chand trig',     stroke: '#ff3860', width: 0,
              points: { show: true, size: 9, fill: 'transparent', stroke: '#ff3860' } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, closes, closeStopLine, chandStopLine, closeTrigs, chandTrigs], el);
}

function showErr(msg) {
    const el = document.getElementById('vsc-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('vsc-err').style.display = 'none'; }
