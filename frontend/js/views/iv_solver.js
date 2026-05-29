// Implied Volatility Solver view.
//
// Input: option market price + contract terms. Output: the σ such that
// BS(σ) == market_price, via Newton-Raphson on the backend.
//
// Visualization:
//   * Summary cards: solved IV, iteration count, residual, BS price at
//     solved IV (sanity), no-arb bounds.
//   * Sensitivity chart: BS price across a σ range with a horizontal
//     line at market_price + a vertical marker at solved σ — confirms
//     visually that the solver hit the right intersection.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    buildBody, validateParams, arbBounds,
    priceVsSigmaSweep, fmtVolPct, fmtPrice,
} from '../_iv_solver_inputs.js';
import { blackScholesEuropean } from '../_american_option_inputs.js';

import { t } from '../i18n.js';
const DEFAULT_PARAMS = {
    kind: 'call',
    spot: 100,
    strike: 105,
    time_to_expiry: 0.25,
    risk_free: 0.05,
    dividend_yield: 0.0,
    market_price: 2.50,
};

let state = { params: { ...DEFAULT_PARAMS } };

export async function renderIvSolver(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.iv_solver.h1.iv_solver" class="view-title">// IV SOLVER</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.iv_solver.h2.contract">Contract</h2>
            <div class="inline-form">
                <label><span data-i18n="view.iv_solver.label.kind">Kind</span>
                    <select id="iv-kind">
                        <option data-i18n="view.iv_solver.opt.call" value="call" ${state.params.kind === 'call' ? 'selected' : ''}>Call</option>
                        <option data-i18n="view.iv_solver.opt.put" value="put"  ${state.params.kind === 'put'  ? 'selected' : ''}>Put</option>
                    </select></label>
                <label><span data-i18n="view.iv_solver.label.spot">Spot</span>
                    <input id="iv-spot"   type="number" step="any" min="0" value="${state.params.spot}"></label>
                <label><span data-i18n="view.iv_solver.label.strike">Strike</span>
                    <input id="iv-strike" type="number" step="any" min="0" value="${state.params.strike}"></label>
                <label><span data-i18n="view.iv_solver.label.t">T (years)</span>
                    <input id="iv-t"   type="number" step="any" min="0" value="${state.params.time_to_expiry}"></label>
                <label><span data-i18n="view.iv_solver.label.rate">Rate r</span>
                    <input id="iv-r"    type="number" step="any" value="${state.params.risk_free}"></label>
                <label><span data-i18n="view.iv_solver.label.div">Dividend q</span>
                    <input id="iv-q" type="number" step="any" min="0" value="${state.params.dividend_yield}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.iv_solver.h2.market_price">Market price</h2>
            <div class="inline-form">
                <label><span data-i18n="view.iv_solver.label.market_price">Observed market price</span>
                    <input id="iv-mkt" type="number" step="any" min="0" value="${state.params.market_price}"></label>
                <button data-i18n="view.iv_solver.btn.solve_iv" id="iv-run" class="primary" type="button">Solve IV</button>
            </div>
            <p data-i18n="view.iv_solver.hint.newton_raphson_on_black_scholes_finds_such_that_bs" class="muted">
                Newton-Raphson on Black-Scholes: finds σ such that BS(σ) matches the
                observed market price. Pre-flight rejects market prices outside the no-arb
                band — no IV exists there, would be a free arb if real.
            </p>
        </div>

        <div id="iv-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.iv_solver.h2.bs_price_vs">BS price vs σ</h2>
            <div id="iv-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.iv_solver.hint.cyan_bs_price_curve_as_sweeps_from_near_zero_orang" class="muted">
                Cyan: BS price curve as σ sweeps from near-zero. Orange dashed line: your
                observed market price. Green marker: the solved σ — should sit exactly at
                the intersection. If the curve never crosses the line, no IV exists
                (out of no-arb bounds).
            </p>
        </div>

        <div id="iv-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('iv-run').addEventListener('click', () => {
        readInputs();
        void solve(mount, tok);
    });
    void fmt;
}

function readInputs() {
    const get = id => document.getElementById(id).value;
    state.params = {
        kind:            get('iv-kind'),
        spot:            Number(get('iv-spot')),
        strike:          Number(get('iv-strike')),
        time_to_expiry:  Number(get('iv-t')),
        risk_free:       Number(get('iv-r')),
        dividend_yield:  Number(get('iv-q')),
        market_price:    Number(get('iv-mkt')),
    };
}

async function solve(mount, tok) {
    hideErr();
    const err = validateParams(state.params);
    if (err) { showErr(err); return; }

    let res;
    try {
        res = await api.optsIvSolver(buildBody(state.params));
        if (!res) throw new Error('IV solver returned null (root-finding failed)');
    } catch (e) {
        showErr(`API error: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(res);
    renderChart(res);
}

function renderSummary(res) {
    const p = state.params;
    const bounds = arbBounds(p);
    const bsAtSolved = blackScholesEuropean(
        p.kind, p.spot, p.strike, p.time_to_expiry,
        p.risk_free, p.dividend_yield, res.implied_vol,
    );
    const cards = [];
    cards.push(card(t('view.iv_solver.card.implied_volatility'), fmtVolPct(res.implied_vol), 'pos',
        `<div class="vc-row"><span class="muted">absolute σ</span>
            <strong>${res.implied_vol.toFixed(6)}</strong></div>`));
    cards.push(card(t('view.iv_solver.card.newton_iterations'), String(res.iterations), '',
        `<div class="vc-row"><span class="muted">residual</span>
            <strong>${res.residual.toExponential(3)}</strong></div>`));
    cards.push(card(t('view.iv_solver.card.bs_at_solved'), fmtPrice(bsAtSolved), '',
        `<div class="vc-row"><span class="muted">market price</span>
            <strong>${fmtPrice(p.market_price)}</strong></div>`));
    cards.push(card(t('view.iv_solver.card.no_arb_band'),
        `${fmtPrice(bounds.lower)} – ${fmtPrice(bounds.upper)}`, '',
        `<div class="vc-row"><span class="muted">market is</span>
            <strong>${p.market_price >= bounds.lower && p.market_price <= bounds.upper ? 'inside ✓' : 'outside ✗'}</strong></div>`));
    document.getElementById('iv-summary').innerHTML = cards.join('');
}

function card(label, value, cls = '', body = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
        ${body ? `<div class="value iv-summary-value">${body}</div>` : ''}
    </div>`;
}

function renderChart(res) {
    const el = document.getElementById('iv-chart');
    if (!window.uPlot) { el.textContent = 'uPlot not loaded'; return; }
    el.innerHTML = '';
    const p = state.params;
    // Sweep σ from near-zero up to max(2.0, 1.5 × solved IV) so the
    // user always sees both the intersection AND some curve past it.
    const maxSigma = Math.max(2.0, res.implied_vol * 1.5);
    const { xs, ys } = priceVsSigmaSweep(p, maxSigma, 161);
    if (xs.length === 0) { el.innerHTML = '<div class="boot">Bad σ range.</div>'; return; }
    const marketLine = xs.map(() => p.market_price);
    // Solved-σ marker: single point at (impliedVol, marketPrice).
    const halfWidth = (xs[1] - xs[0]) / 2;
    const markerYs = xs.map(x =>
        Math.abs(x - res.implied_vol) < halfWidth ? p.market_price : null);

    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: 'σ' },
            { label: 'BS price',     stroke: '#00e5ff', width: 2, points: { show: false } },
            { label: 'market price', stroke: '#ff9f1a', width: 1, dash: [4, 4],
              points: { show: false } },
            { label: 'solved σ',     stroke: '#39ff14', width: 0,
              points: { show: true, size: 12, stroke: '#39ff14', fill: '#39ff14' } },
        ],
        axes: [
            { stroke: '#aab',
              values: (_, ticks) => ticks.map(t => `${(t * 100).toFixed(0)}%`) },
            { stroke: '#aab' },
        ],
    }, [xs, ys, marketLine, markerYs], el);
}

function showErr(msg) {
    const el = document.getElementById('iv-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('iv-err').style.display = 'none'; }
