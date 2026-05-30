// VWAP Slippage view — institutional TCA "did I beat VWAP?" benchmark.
//
// Long fill BELOW VWAP = positive slippage (you got a discount on entry).
// Short fill ABOVE VWAP = positive (you got premium on entry).

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarBlob, validateInputs, buildBody,
    localVwap, rollingVwap, unwrapResponse, decToNum,
    makeDemoData, fmtN, fmtBps, fmtVol,
} from '../_vwap_slippage_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = { side: 'long', fillPrice: 100, barText: '' };

export async function renderVwapSlippage(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.vwap_slippage.h1.vwap_slippage_tca_benchmark" class="view-title">// VWAP SLIPPAGE · TCA BENCHMARK</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.vwap_slippage.h2.trade">Trade</h2>
            <div class="inline-form">
                <label><span data-i18n="view.vwap_slippage.label.side">Side</span>
                    <select id="vw-side" data-tip="view.vwap_slippage.tip.side">
                        <option data-i18n="view.vwap_slippage.opt.long_buy_entry" value="long"  ${state.side === 'long'  ? 'selected' : ''}>Long (buy entry)</option>
                        <option data-i18n="view.vwap_slippage.opt.short_sell_entry" value="short" ${state.side === 'short' ? 'selected' : ''}>Short (sell entry)</option>
                    </select></label>
                <label><span data-i18n="view.vwap_slippage.label.fill_price">Fill price</span>
                    <input id="vw-fill" type="number" step="any" min="0" value="${state.fillPrice}" data-tip="view.vwap_slippage.tip.fill"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.vwap_slippage.h2.bars_during_the_trade_s_open_window">Bars during the trade's open window</h2>
            <p class="muted" data-i18n="view.vwap_slippage.hint.format">One line per bar: typical_price volume. Typical = (high+low+close)/3. Pre-computed by the caller so this view stays agnostic about whether you're feeding 1-second, 1-minute, or 1-hour bars. Demo loads 200 bars with an intentional below-VWAP long fill.</p>
            <textarea id="vw-bars" rows="6" placeholder="100.05 1200&#10;100.08 850&#10;..." data-tip="view.vwap_slippage.tip.bars"></textarea>
            <div class="inline-form">
                <button data-i18n="view.vwap_slippage.btn.load_demo_200_bars_fill_beats_vwap" data-tip="view.vwap_slippage.tip.demo" data-shortcut="vwap_slippage_demo" id="vw-demo" class="secondary" type="button">Load demo (200 bars, fill beats VWAP)</button>
                <button data-i18n="view.vwap_slippage.btn.clear" data-tip="view.vwap_slippage.tip.clear" id="vw-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.vwap_slippage.btn.analyze" data-tip="view.vwap_slippage.tip.analyze" data-shortcut="vwap_slippage_analyze" id="vw-run" class="primary" type="button">Analyze</button>
            </div>
        </div>

        <div id="vw-errors" class="boot" style="display:none"></div>
        <div id="vw-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.vwap_slippage.h2.typical_price_rolling_vwap_fill_reference">Typical price + rolling VWAP + fill reference</h2>
            <div id="vw-chart" style="height:280px"></div>
            <p data-i18n="view.vwap_slippage.hint.cyan_typical_price_per_bar_yellow_rolling_vwap_the" class="muted">Cyan = typical price per bar. Yellow = rolling VWAP (the
                benchmark). Magenta dashed = your fill price. For LONG entries you want
                magenta BELOW yellow at trade close; for SHORT entries you want it ABOVE.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.vwap_slippage.h2.volume_chart">Per-bar volume (the weight behind every VWAP point)</h2>
            <div id="vw-vol-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.vwap_slippage.hint.volume_chart" class="muted small">Trading size per bar across the trade window. Heavy bars dominate the VWAP benchmark — reveals which bars set the price you're being graded against. Orthogonal to the price overlay above.</p>
        </div>

        <div id="vw-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('vw-demo').addEventListener('click', () => {
        const { side, fill_price, bars } = makeDemoData(42);
        document.getElementById('vw-side').value = side;
        document.getElementById('vw-fill').value = fill_price;
        document.getElementById('vw-bars').value =
            bars.map(b => `${b.typical} ${b.volume}`).join('\n');
    });
    document.getElementById('vw-clear').addEventListener('click', () => {
        document.getElementById('vw-bars').value = '';
    });
    document.getElementById('vw-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.side = document.getElementById('vw-side').value;
    state.fillPrice = Number(document.getElementById('vw-fill').value);
    state.barText = document.getElementById('vw-bars').value;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('vw-errors');
    errs.style.display = 'none';
    const { bars, errors } = parseBarBlob(state.barText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (bars.length === 0) return;
    }
    const err = validateInputs(state.side, state.fillPrice, bars);
    if (err) { showErr(err); showToast(err, { level: 'warning' }); return; }
    let resp;
    try {
        resp = await api.microVwapSlippage(buildBody(state.side, state.fillPrice, bars));
    } catch (e) {
        const m = t("common.error.api", { msg: e.message || e });
        showErr(m); showToast(m, { level: 'error' }); return;
    }
    if (!viewIsCurrent(tok)) return;
    const unwrapped = unwrapResponse(resp);
    if (!unwrapped.ok) {
        const m = t('common.error.backend', { reason: unwrapped.reason });
        showErr(m); showToast(m, { level: 'error' }); return;
    }
    renderSummary(unwrapped.result, bars);
    renderChart(bars, decToNum(unwrapped.result.vwap), state.fillPrice);
    renderVolChart(bars);
    showToast(t('view.vwap_slippage.toast.done', {
        bps: fmtBps(unwrapped.result.slippage_bps),
        beat: unwrapped.result.beat_vwap ? t('common.yes') : t('common.no'),
    }), { level: unwrapped.result.beat_vwap ? 'success' : 'warning' });
}

function renderSummary(r, bars) {
    const vwap = decToNum(r.vwap);
    const fill = decToNum(r.fill_price);
    const slipDollars = decToNum(r.slippage_dollars);
    const localChk = localVwap(bars);
    const totalVol = bars.reduce((a, b) => a + (b.volume || 0), 0);
    document.getElementById('vw-summary').innerHTML = [
        card(t('view.vwap_slippage.card.vwap_backend'), fmtN(vwap)),
        card(t('view.vwap_slippage.card.vwap_local'),   fmtN(localChk),
            Math.abs(vwap - localChk) < 1e-6 ? 'pos' : 'neg'),
        card(t('view.vwap_slippage.card.fill_price'),     fmtN(fill)),
        card(t('common.card.slippage_dollars'),     fmtN(slipDollars),
            slipDollars > 0 ? 'pos' : slipDollars < 0 ? 'neg' : ''),
        card(t('view.vwap_slippage.card.slippage_bps'),   fmtBps(r.slippage_bps),
            r.slippage_bps > 0 ? 'pos' : r.slippage_bps < 0 ? 'neg' : ''),
        card(t('view.vwap_slippage.card.beat_vwap'),     r.beat_vwap ? t('common.yes') : t('common.no'),
            r.beat_vwap ? 'pos' : 'neg'),
        card(t('view.vwap_slippage.card.bars'),           String(bars.length)),
        card(t('view.vwap_slippage.card.total_volume'),   fmtVol(totalVol)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(bars, vwap, fillPrice) {
    if (!window.uPlot) return;
    const el = document.getElementById('vw-chart');
    const xs = bars.map((_, i) => i);
    const typ = bars.map(b => b.typical);
    const roll = rollingVwap(bars);
    const vwapYs = xs.map(() => vwap);
    const fillYs = xs.map(() => fillPrice);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 280,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.bar_num') },
            { label: t('chart.series.typical'), stroke: '#00e5ff', width: 1.0,
              fill: '#00e5ff14', points: { show: false } },
            { label: t('chart.series.rolling_vwap'), stroke: '#ffd84a', width: 1.2,
              points: { show: false } },
            { label: t('chart.series.final_vwap'), stroke: '#ff9f1a', width: 1.0,
              dash: [4, 4], points: { show: false } },
            { label: t('chart.series.fill'), stroke: '#ff3860', width: 1.0,
              dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, typ, roll, vwapYs, fillYs], el);
}

function renderVolChart(bars) {
    const el = document.getElementById('vw-vol-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (bars || []).filter(b => Number.isFinite(Number(b.volume)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.vwap_slippage.empty_vol_chart">${esc(t('view.vwap_slippage.empty_vol_chart'))}</div>`;
        return;
    }
    const ys = valid.map(b => Number(b.volume));
    const xs = ys.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.vwap_slippage.chart.bar_idx') },
            { label: t('view.vwap_slippage.chart.volume'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 10, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => '#' + Math.trunc(v)) },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function showErr(msg) {
    const el = document.getElementById('vw-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('vw-err').style.display = 'none'; }
