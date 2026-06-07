// TWAP view — time-weighted execution analyzer.
//
// Complements VWAP Slippage: VWAP weights by volume (right tool for
// active aggressive orders), TWAP weights equally by time (right tool
// for passive limit working orders where time-in-market matters more
// than volume-participation).

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseTypicals, validateInputs, buildBody,
    localTwap, rollingTwap, decToNum, unwrapResponse,
    makeDemoData, fmtN, fmtBps,
} from '../_twap_inputs.js';

import { t } from '../i18n.js';
let state = { side: 'long', fillPrice: 100, typicalsText: '' };

export async function renderTwap(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.twap.h1.twap_time_weighted_execution" class="view-title">// TWAP · TIME-WEIGHTED EXECUTION</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.twap.h2.trade">Trade</h2>
            <div class="inline-form">
                <label><span data-i18n="view.twap.label.side">Side</span>
                    <select id="tw-side">
                        <option data-i18n="view.twap.opt.long_buy_entry" value="long"  ${state.side === 'long'  ? 'selected' : ''}>Long (buy entry)</option>
                        <option data-i18n="view.twap.opt.short_sell_entry" value="short" ${state.side === 'short' ? 'selected' : ''}>Short (sell entry)</option>
                    </select></label>
                <label><span data-i18n="view.twap.label.fill_price">Fill price</span>
                    <input id="tw-fill" type="number" step="0.01" min="0" value="${state.fillPrice}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.twap.h2.typical_prices_over_the_exposure_window">Typical prices over the exposure window</h2>
            <p data-i18n="view.twap.hint.one_value_per_line_typical_h_l_c_3_from_each_bar_d" class="muted">One value per line. Typical = (H+L+C)/3 from each bar.
                Demo loads 200 typicals with a long fill engineered to beat the mean.</p>
            <textarea id="tw-typ" rows="6" placeholder="100.05&#10;100.08&#10;..."></textarea>
            <div class="inline-form">
                <button data-i18n="view.twap.btn.load_demo_200_prices_fill_beats_twap" id="tw-demo" class="secondary" type="button">Load demo (200 prices, fill beats TWAP)</button>
                <button data-i18n="view.twap.btn.clear" id="tw-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.twap.btn.analyze" id="tw-run" class="primary" type="button">Analyze</button>
            </div>
        </div>

        <div id="tw-errors" class="boot" style="display:none"></div>
        <div id="tw-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.twap.h2.typical_price_rolling_twap_fill_reference">Typical price + rolling TWAP + fill reference</h2>
            <div id="tw-chart" style="height:280px"></div>
            <p data-i18n="view.twap.hint.cyan_typical_yellow_rolling_twap_arithmetic_mean_t" class="muted">Cyan = typical. Yellow = rolling TWAP (arithmetic mean to bar i).
                Magenta dashed = fill. Long entries want magenta below yellow; shorts the inverse.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.twap.h2.advantage_chart">Cumulative fill advantage vs typical (signed for the trader)</h2>
            <div id="tw-adv-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.twap.hint.advantage_chart" class="muted small">Running sum of per-bar (typical − fill) for longs, (fill − typical) for shorts. Above zero = the fill was favorable vs the trajectory; below zero = the fill underperformed bars after it. Orthogonal to the price overlay above.</p>
        </div>

        <div id="tw-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('tw-demo').addEventListener('click', () => {
        const { side, fill_price, typicals } = makeDemoData(42);
        document.getElementById('tw-side').value = side;
        document.getElementById('tw-fill').value = fill_price;
        document.getElementById('tw-typ').value = typicals.join('\n');
    });
    document.getElementById('tw-clear').addEventListener('click', () => {
        document.getElementById('tw-typ').value = '';
    });
    document.getElementById('tw-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.side = document.getElementById('tw-side').value;
    state.fillPrice = Number(document.getElementById('tw-fill').value);
    state.typicalsText = document.getElementById('tw-typ').value;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('tw-errors');
    errs.style.display = 'none';
    const { value: typicals, errors } = parseTypicals(state.typicalsText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (typicals.length === 0) return;
    }
    const err = validateInputs(state.side, state.fillPrice, typicals);
    if (err) { showErr(err); return; }
    let resp;
    try {
        resp = await api.microTwap(buildBody(state.side, state.fillPrice, typicals));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    const unwrapped = unwrapResponse(resp);
    if (!unwrapped.ok) { showErr(t('common.error.backend', { reason: unwrapped.reason })); return; }
    renderSummary(unwrapped.result, typicals);
    renderChart(typicals, decToNum(unwrapped.result.twap), state.fillPrice);
    renderAdvantageChart(typicals, state.fillPrice, state.side);
}

function renderAdvantageChart(typicals, fillPrice, side) {
    const el = document.getElementById('tw-adv-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const arr = (typicals || []).filter(v => Number.isFinite(Number(v)));
    if (arr.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.twap.empty_advantage_chart">${esc(t('view.twap.empty_advantage_chart'))}</div>`;
        return;
    }
    const sign = side === 'short' ? -1 : 1;
    let acc = 0;
    const cum = arr.map(v => (acc += sign * (Number(v) - Number(fillPrice))));
    const xs = cum.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.twap.chart.bar_idx') },
            { label: t('view.twap.chart.cum_advantage'),
              stroke: '#b86bff', width: 1.6, points: { show: false } },
            { label: t('view.twap.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 60 }],
        legend: { show: true },
    }, [xs, cum, zero], el);
}

function renderSummary(r, typicals) {
    const twap = decToNum(r.twap);
    const fill = decToNum(r.fill_price);
    const slipDollars = decToNum(r.slippage_dollars);
    const localChk = localTwap(typicals);
    document.getElementById('tw-summary').innerHTML = [
        card(t('view.twap.card.twap_backend'), fmtN(twap)),
        card(t('view.twap.card.twap_local'),   fmtN(localChk),
            Math.abs(twap - localChk) < 1e-6 ? 'pos' : 'neg'),
        card(t('view.twap.card.fill_price'),     fmtN(fill)),
        card(t('common.card.slippage_dollars'),     fmtN(slipDollars),
            slipDollars > 0 ? 'pos' : slipDollars < 0 ? 'neg' : ''),
        card(t('view.twap.card.slippage_bps'),   fmtBps(r.slippage_bps),
            r.slippage_bps > 0 ? 'pos' : r.slippage_bps < 0 ? 'neg' : ''),
        card(t('view.twap.card.beat_twap'),     r.beat_twap ? t('common.yes') : t('common.no'),
            r.beat_twap ? 'pos' : 'neg'),
        card(t('view.twap.card.bars'),           String(typicals.length)),
        card(t('view.twap.card.mean_fill'),    fmtN(localChk - state.fillPrice)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(typicals, twap, fillPrice) {
    if (!window.uPlot) return;
    const el = document.getElementById('tw-chart');
    const xs = typicals.map((_, i) => i);
    const roll = rollingTwap(typicals);
    const twapYs = xs.map(() => twap);
    const fillYs = xs.map(() => fillPrice);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 280,
        scales: { x: { time: false,}, y: {} },
        series: [
            { label: t('chart.series.bar_num') },
            { label: t('chart.series.typical'),      stroke: '#00e5ff', width: 1.0,
              fill: '#00e5ff14', points: { show: false } },
            { label: t('chart.series.rolling_twap'), stroke: '#ffd84a', width: 1.2,
              points: { show: false } },
            { label: t('chart.series.final_twap'),   stroke: '#ff9f1a', width: 1.0,
              dash: [4, 4], points: { show: false } },
            { label: t('chart.series.fill'),         stroke: '#ff3860', width: 1.0,
              dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, typicals, roll, twapYs, fillYs], el);
}

function showErr(msg) {
    const el = document.getElementById('tw-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('tw-err').style.display = 'none'; }
