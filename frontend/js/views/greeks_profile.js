// Greeks Profile view — plots price + delta + gamma + vega + theta +
// rho as functions of spot for a single option contract.
//
// Use cases:
//   * "Where does my long gamma peak?" → check the gamma curve.
//   * "How much theta do I bleed if spot drifts 5%?" → read theta at
//     that spot from the curve.
//   * "What's my delta exposure on a 1-point move from here?" → trace
//     delta to see slope (= gamma).

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    buildBody, validateParams, splitMetricSeries,
    METRICS, fmtN, defaultSpotGrid,
} from '../_greeks_profile_inputs.js';

import { t } from '../i18n.js';
const DEFAULT_PARAMS = {
    kind: 'call',
    strike: 100,
    time_to_expiry: 0.25,
    risk_free: 0.05,
    dividend_yield: 0.0,
    sigma: 0.25,
    spot_grid_low: 50,
    spot_grid_high: 150,
    n_points: 41,
};

const METRIC_COLORS = {
    price: '#00e5ff', delta: '#ff9f1a', gamma: '#a06bff',
    vega:  '#39ff14', theta: '#ff3860', rho:   '#ffd84a',
};

let state = { params: { ...DEFAULT_PARAMS } };

export async function renderGreeksProfile(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.greeks_profile.h1.greeks_profile" class="view-title">// GREEKS PROFILE</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.greeks_profile.h2.contract">Contract</h2>
            <div class="inline-form">
                <label><span data-i18n="view.greeks_profile.label.kind">Kind</span>
                    <select id="gp-kind">
                        <option data-i18n="view.greeks_profile.opt.call" value="call" ${state.params.kind === 'call' ? 'selected' : ''}>Call</option>
                        <option data-i18n="view.greeks_profile.opt.put" value="put"  ${state.params.kind === 'put'  ? 'selected' : ''}>Put</option>
                    </select></label>
                <label><span data-i18n="view.greeks_profile.label.strike">Strike</span> <input id="gp-strike" type="number" step="any" min="0" value="${state.params.strike}"></label>
                <label><span data-i18n="view.greeks_profile.label.t">T (years)</span> <input id="gp-t" type="number" step="any" min="0" value="${state.params.time_to_expiry}"></label>
                <label><span data-i18n="view.greeks_profile.label.r">Rate r</span> <input id="gp-r" type="number" step="any" value="${state.params.risk_free}"></label>
                <label><span data-i18n="view.greeks_profile.label.q">Dividend q</span> <input id="gp-q" type="number" step="any" min="0" value="${state.params.dividend_yield}"></label>
                <label><span data-i18n="view.greeks_profile.label.sigma">σ</span> <input id="gp-sigma" type="number" step="any" min="0" value="${state.params.sigma}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.greeks_profile.h2.spot_grid">Spot grid</h2>
            <div class="inline-form">
                <label><span data-i18n="view.greeks_profile.label.low">Low</span> <input id="gp-low" type="number" step="any" min="0" value="${state.params.spot_grid_low}"></label>
                <label><span data-i18n="view.greeks_profile.label.high">High</span> <input id="gp-high" type="number" step="any" min="0" value="${state.params.spot_grid_high}"></label>
                <label><span data-i18n="view.greeks_profile.label.points">Points</span> <input id="gp-n" type="number" step="1" min="5" max="501" value="${state.params.n_points}"></label>
                <button data-i18n="view.greeks_profile.btn.50_from_strike" id="gp-defaults" class="secondary" type="button">±50% from strike</button>
                <button data-i18n="view.greeks_profile.btn.compute" id="gp-run" class="primary" type="button">Compute</button>
            </div>
            <p data-i18n="view.greeks_profile.hint.each_greek_plotted_as_a_function_of_spot_across_th" class="muted">
                Each greek plotted as a function of spot across the chosen grid. ATM marker
                shows where spot = strike. Quick sanity: ATM call delta ≈ 0.5, gamma peaks
                near ATM, vega peaks near ATM, theta most negative near ATM (max time-decay).
            </p>
        </div>

        <div id="gp-summary" class="cards"></div>

        <div id="gp-grid" class="gp-grid"></div>

        <div id="gp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('gp-defaults').addEventListener('click', () => {
        const strike = Number(document.getElementById('gp-strike').value);
        const grid = defaultSpotGrid(strike);
        document.getElementById('gp-low').value  = grid.low;
        document.getElementById('gp-high').value = grid.high;
    });
    document.getElementById('gp-run').addEventListener('click', () => {
        readInputs();
        void compute(mount, tok);
    });
    void fmt;
}

function readInputs() {
    const get = id => document.getElementById(id).value;
    state.params = {
        kind:           get('gp-kind'),
        strike:         Number(get('gp-strike')),
        time_to_expiry: Number(get('gp-t')),
        risk_free:      Number(get('gp-r')),
        dividend_yield: Number(get('gp-q')),
        sigma:          Number(get('gp-sigma')),
        spot_grid_low:  Number(get('gp-low')),
        spot_grid_high: Number(get('gp-high')),
        n_points:       parseInt(get('gp-n'), 10),
    };
}

async function compute(mount, tok) {
    hideErr();
    const err = validateParams(state.params);
    if (err) { showErr(err); return; }

    let res;
    try {
        res = await api.optsGreeksProfile(buildBody(state.params));
        if (!res) throw new Error(t('view.greeks_profile.error.null'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    const series = splitMetricSeries(res.points);
    renderSummary(res, series);
    renderGrid(series, res);
}

function renderSummary(res, series) {
    const i = res.atm_index;
    const atm = res.points[i] || {};
    document.getElementById('gp-summary').innerHTML = [
        card(t('view.greeks_profile.card.atm_spot'), fmtN(atm.spot, 2)),
        card(t('view.greeks_profile.card.price_atm'), fmtN(atm.price)),
        card(t('view.greeks_profile.card.atm'), fmtN(atm.delta), atm.delta >= 0 ? 'pos' : 'neg'),
        card(t('view.greeks_profile.card.atm_2'), fmtN(atm.gamma, 6)),
        card(t('view.greeks_profile.card.vega_atm'), fmtN(atm.vega)),
        card(t('view.greeks_profile.card.atm_3'), fmtN(atm.theta), atm.theta < 0 ? 'neg' : ''),
        card(t('view.greeks_profile.card.atm_4'), fmtN(atm.rho)),
        card(t('view.greeks_profile.card.grid_points'), String(series.spots.length)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderGrid(series, res) {
    const wrap = document.getElementById('gp-grid');
    wrap.innerHTML = METRICS.map(m =>
        `<div class="gp-cell">
            <div class="gp-cell-label">${esc(m)}</div>
            <div id="gp-chart-${esc(m)}" class="gp-chart-area"></div>
        </div>`
    ).join('');
    if (!window.uPlot) return;
    const atmSpot = (res.points[res.atm_index] || {}).spot;
    for (const m of METRICS) {
        drawMini(`gp-chart-${m}`, series.spots, series[m], METRIC_COLORS[m], atmSpot);
    }
}

function drawMini(elId, xs, ys, stroke, atmSpot) {
    const el = document.getElementById(elId);
    if (!el || xs.length === 0) return;
    // ATM-reference series: a single point that sits on the curve at
    // the ATM spot. uPlot draws it as a marker, visually anchoring the
    // reader at strike.
    const halfWidth = xs.length > 1 ? (xs[1] - xs[0]) / 2 : 0;
    const atmYs = xs.map((s, i) =>
        Number.isFinite(atmSpot) && Math.abs(s - atmSpot) <= halfWidth ? ys[i] : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 400, height: 140,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.spot') },
            { label: t('chart.series.value'), stroke, width: 1.5,
              fill: `${stroke}1A`, points: { show: false } },
            { label: 'ATM', stroke: '#fff', width: 0,
              points: { show: true, size: 8, stroke: '#fff', fill: '#fff' } },
        ],
        axes: [
            { stroke: '#aab', size: 24 },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: false },
    }, [xs, ys, atmYs], el);
}

function showErr(msg) {
    const el = document.getElementById('gp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('gp-err').style.display = 'none'; }
