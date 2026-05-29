// VPIN view — toxic order-flow detector (Easley/López de Prado/O'Hara 2012).
//
// Pipeline:
//   1. Paste tick stream (price volume per line).
//   2. Configure volume_per_bucket / window_buckets / return_window.
//   3. Backend BVC-classifies each tick into buy/sell volume, accumulates
//      into equal-volume buckets, computes per-bucket VPIN.
//   4. View charts VPIN with a red toxic-threshold line at 0.5 +
//      bucket-level buy/sell volume bars.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseTickBlob, validateInputs, buildBody,
    extractFinishedVpin, summarize, makeDemoTicks,
    fmtN, fmtPct,
} from '../_vpin_inputs.js';

import { t } from '../i18n.js';
const DEFAULT_CONFIG = {
    volume_per_bucket: 50_000,
    window_buckets: 50,
    return_window: 100,
};

const TOXIC_THRESHOLD = 0.5;

let state = {
    config: { ...DEFAULT_CONFIG },
    tickText: '',
    demoLoaded: false,
};

export async function renderVpin(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.vpin.h1.vpin_toxic_order_flow_detector" class="view-title">// VPIN · TOXIC ORDER-FLOW DETECTOR</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.vpin.h2.tick_stream">Tick stream</h2>
            <p class="muted" data-i18n-html="view.vpin.tick_stream.help">Paste <code>price volume</code> per line (whitespace- or comma-separated).
                Lines starting with <code>#</code> are ignored. Demo data
                injects a benign random-walk regime followed by a toxic burst
                so the VPIN line clearly crosses the 0.5 threshold.</p>
            <textarea id="vp-ticks" rows="8" placeholder="100.05 250&#10;100.06 1200&#10;100.04 500&#10;..."></textarea>
            <div class="inline-form">
                <button data-i18n="view.vpin.btn.load_demo_1500_ticks" id="vp-demo" class="secondary" type="button">Load demo (1500 ticks)</button>
                <button data-i18n="view.vpin.btn.clear" id="vp-clear" class="secondary" type="button">Clear</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.vpin.h2.config">Config</h2>
            <div class="inline-form">
                <label><span data-i18n="view.vpin.label.volume_per_bucket">Volume per bucket</span>
                    <input id="vp-vpb" type="number" step="any" min="1" value="${state.config.volume_per_bucket}"></label>
                <label><span data-i18n="view.vpin.label.window_buckets">Window buckets</span>
                    <input id="vp-wb"  type="number" step="1" min="1" max="2000" value="${state.config.window_buckets}"></label>
                <label><span data-i18n="view.vpin.label.return_window">Return window (ticks)</span>
                    <input id="vp-rw"  type="number" step="1" min="2" max="10000" value="${state.config.return_window}"></label>
                <button data-i18n="view.vpin.btn.compute_vpin" id="vp-run" class="primary" type="button">Compute VPIN</button>
            </div>
            <p class="muted">VPIN ≥ ${TOXIC_THRESHOLD} marks a "toxic" bucket — informed
                traders running through the book. Was used to flag the 2010 flash crash
                minutes ahead of the cascade.</p>
        </div>

        <div id="vp-errors" class="boot" style="display:none"></div>
        <div id="vp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.vpin.h2.vpin_time_series">VPIN time series</h2>
            <div id="vp-chart-vpin" style="height:260px"></div>
            <p class="muted">Red dashed = ${TOXIC_THRESHOLD} toxic threshold. Spikes above the line are flow-toxicity alerts.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.vpin.h2.bucket_buy_sell_volume">Bucket buy / sell volume</h2>
            <div id="vp-chart-vol" style="height:240px"></div>
            <p data-i18n="view.vpin.hint.cyan_buy_classified_volume_magenta_sell_classified" class="muted">Cyan = buy-classified volume. Magenta = sell-classified. Vertical asymmetry per bucket = imbalance = VPIN's numerator.</p>
        </div>

        <div id="vp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('vp-demo').addEventListener('click', () => {
        const ticks = makeDemoTicks(1500, 42);
        const text = ticks.map(t => `${t.price} ${t.volume}`).join('\n');
        document.getElementById('vp-ticks').value = text;
        state.demoLoaded = true;
    });
    document.getElementById('vp-clear').addEventListener('click', () => {
        document.getElementById('vp-ticks').value = '';
    });
    document.getElementById('vp-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.tickText = document.getElementById('vp-ticks').value;
    state.config = {
        volume_per_bucket: Number(document.getElementById('vp-vpb').value),
        window_buckets:    parseInt(document.getElementById('vp-wb').value, 10),
        return_window:     parseInt(document.getElementById('vp-rw').value, 10),
    };
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('vp-errors');
    errs.style.display = 'none';
    const { ticks, errors } = parseTickBlob(state.tickText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (ticks.length < 10) return;
    }
    const err = validateInputs(ticks, state.config);
    if (err) { showErr(err); return; }
    let res;
    try {
        res = await api.microVpin(buildBody(ticks, state.config));
    } catch (e) {
        showErr(`API error: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res, ticks);
    renderVpinChart(res);
    renderVolChart(res);
}

function renderSummary(report, ticks) {
    const s = summarize(report, TOXIC_THRESHOLD) || {
        nBuckets: 0, maxVpin: NaN, avgVpin: NaN,
        toxicCount: 0, toxicPct: 0, totalBuy: 0, totalSell: 0, buySellSkew: 0,
    };
    document.getElementById('vp-summary').innerHTML = [
        card(t('view.vpin.card.ticks'),         String(ticks.length)),
        card(t('view.vpin.card.buckets'),       String(s.nBuckets)),
        card(t('view.vpin.card.max_vpin'),      fmtN(s.maxVpin), s.maxVpin >= TOXIC_THRESHOLD ? 'neg' : 'pos'),
        card(t('view.vpin.card.avg_vpin'),      fmtN(s.avgVpin), s.avgVpin >= TOXIC_THRESHOLD ? 'neg' : 'pos'),
        card(t('view.vpin.card.toxic_buckets'), `${s.toxicCount} (${fmtPct(s.toxicPct)})`,
            s.toxicCount > 0 ? 'neg' : 'pos'),
        card(t('view.vpin.card.buy_vol'),       formatVolume(s.totalBuy)),
        card(t('view.vpin.card.sell_vol'),      formatVolume(s.totalSell)),
        card(t('view.vpin.card.b_s_skew'),      fmtPct(s.buySellSkew), s.buySellSkew > 0 ? 'pos' : 'neg'),
    ].join('');
}

function formatVolume(v) {
    if (!Number.isFinite(v)) return '—';
    const a = Math.abs(v);
    if (a >= 1e9) return (v / 1e9).toFixed(2) + 'B';
    if (a >= 1e6) return (v / 1e6).toFixed(2) + 'M';
    if (a >= 1e3) return (v / 1e3).toFixed(2) + 'k';
    return v.toFixed(0);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderVpinChart(report) {
    if (!window.uPlot) return;
    const { xs, ys } = extractFinishedVpin(report);
    const thrYs = xs.map(() => TOXIC_THRESHOLD);
    const el = document.getElementById('vp-chart-vpin');
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 260,
        scales: { x: {}, y: { range: [0, 1] } },
        series: [
            { label: 'bucket #' },
            { label: 'VPIN', stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff1A', points: { show: false } },
            { label: 'toxic threshold', stroke: '#ff3860', width: 1.0,
              dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 40 }],
        legend: { show: true },
    }, [xs, ys, thrYs], el);
}

function renderVolChart(report) {
    if (!window.uPlot) return;
    const buys  = report.bucket_buy_volume  || [];
    const sells = (report.bucket_sell_volume || []).map(v => -v);  // negate for divergent display
    const xs = buys.map((_, i) => i);
    const el = document.getElementById('vp-chart-vol');
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 240,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bucket #' },
            { label: 'buy',  stroke: '#00e5ff', width: 1.0, fill: '#00e5ff66',
              points: { show: false } },
            { label: 'sell', stroke: '#ff3860', width: 1.0, fill: '#ff386066',
              points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, buys, sells], el);
}

function showErr(msg) {
    const el = document.getElementById('vp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('vp-err').style.display = 'none'; }
