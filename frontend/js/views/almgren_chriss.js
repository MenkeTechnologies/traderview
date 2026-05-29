// Almgren-Chriss optimal-execution view — single trajectory + efficient
// frontier (λ sweep).
//
// What it shows:
//   1. Trade schedule v_k per slice (bar shape).
//   2. Inventory path x_k over time (sinh-shaped, front-loaded for high λ).
//   3. Efficient frontier (variance, expected-cost) for a geometric λ
//      ladder — with a "you are here" marker at the user's chosen λ.
//
// The whole point of AC vs TWAP: the trader picks a point on the frontier
// that matches her risk appetite. Low λ → patient (TWAP). High λ → urgent
// (front-load to dump risk, eat impact cost).

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    validateParams, buildBody, timeAxis,
    lambdaSweep, nearestLambdaIndex,
    fmtBig, fmtSeconds, fmtPct,
} from '../_almgren_chriss_inputs.js';

const DEFAULT_PARAMS = {
    total_shares:    1_000_000,
    horizon_seconds: 23_400,   // 6.5h trading day
    n_intervals:     30,
    eta:             2.5e-6,   // canonical AC reference
    gamma:           2.5e-7,
    lambda:          1e-6,
    sigma:           0.95 / Math.sqrt(86_400),  // daily 95¢ → per-√s
};

let state = { params: { ...DEFAULT_PARAMS } };

export async function renderAlmgrenChriss(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// ALMGREN-CHRISS · OPTIMAL EXECUTION</h1>

        <div class="chart-panel">
            <h2>Parent order</h2>
            <div class="inline-form">
                <label>Total shares (signed; − for sell)
                    <input id="ac-X" type="number" step="any" value="${state.params.total_shares}"></label>
                <label>Horizon (seconds)
                    <input id="ac-T" type="number" step="any" min="0" value="${state.params.horizon_seconds}"></label>
                <label>Slices
                    <input id="ac-n" type="number" step="1" min="1" max="2000" value="${state.params.n_intervals}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2>Impact &amp; risk parameters</h2>
            <div class="inline-form">
                <label>η (temp impact)
                    <input id="ac-eta" type="number" step="any" min="0" value="${state.params.eta}"></label>
                <label>γ (perm impact)
                    <input id="ac-gamma" type="number" step="any" min="0" value="${state.params.gamma}"></label>
                <label>λ (risk aversion)
                    <input id="ac-lambda" type="number" step="any" min="0" value="${state.params.lambda}"></label>
                <label>σ (per-√s vol)
                    <input id="ac-sigma" type="number" step="any" min="0" value="${state.params.sigma}"></label>
                <button id="ac-run" class="primary" type="button">Solve</button>
                <button id="ac-frontier" class="secondary" type="button">+ Plot frontier (λ sweep)</button>
            </div>
            <p class="muted">
                κ = √(λσ²/η) controls how front-loaded the trajectory is. λ→0 collapses to TWAP
                (uniform liquidation, minimum impact, maximum timing risk). λ→∞ front-loads
                aggressively, eating impact cost to dump timing risk. The frontier shows the
                cost/variance tradeoff over a geometric λ ladder centred on your chosen λ.
            </p>
        </div>

        <div id="ac-summary" class="cards"></div>

        <div class="chart-panel"><h2>Inventory path x<sub>k</sub></h2>
            <div id="ac-chart-inv" style="height:260px"></div></div>

        <div class="chart-panel"><h2>Trade schedule v<sub>k</sub></h2>
            <div id="ac-chart-sched" style="height:240px"></div></div>

        <div class="chart-panel"><h2>Efficient frontier (λ sweep)</h2>
            <div id="ac-chart-frontier" style="height:280px"></div>
            <p class="muted">Each dot is a solve at one λ. The dashed white marker is
                your currently selected λ. Move down-right = more patient. Up-left =
                more urgent.</p>
        </div>

        <div id="ac-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('ac-run').addEventListener('click', () => {
        readInputs(); void compute(tok);
    });
    document.getElementById('ac-frontier').addEventListener('click', () => {
        readInputs(); void computeFrontier(tok);
    });

    readInputs(); void compute(tok);
}

function readInputs() {
    const get = id => document.getElementById(id).value;
    state.params = {
        total_shares:    Number(get('ac-X')),
        horizon_seconds: Number(get('ac-T')),
        n_intervals:     parseInt(get('ac-n'), 10),
        eta:             Number(get('ac-eta')),
        gamma:           Number(get('ac-gamma')),
        lambda:          Number(get('ac-lambda')),
        sigma:           Number(get('ac-sigma')),
    };
}

async function compute(tok) {
    hideErr();
    const err = validateParams(state.params);
    if (err) { showErr(err); return; }
    let res;
    try {
        res = await api.microAlmgrenChriss(buildBody(state.params));
        if (!res) throw new Error('almgren-chriss returned null (check inputs)');
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res);
    renderTrajectory(res);
}

async function computeFrontier(tok) {
    hideErr();
    const err = validateParams(state.params);
    if (err) { showErr(err); return; }
    const lambdas = lambdaSweep(state.params.lambda, 7);
    let reports;
    try {
        reports = await Promise.all(
            lambdas.map(l => api.microAlmgrenChriss(buildBody({ ...state.params, lambda: l }))),
        );
    } catch (e) {
        showErr(`Frontier sweep failed: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    const points = reports.map((r, i) => r ? {
        lambda:   lambdas[i],
        variance: r.risk_variance,
        cost:     r.expected_impact_cost,
    } : null).filter(Boolean);
    renderFrontier(points, state.params.lambda);
}

function renderSummary(r) {
    const halfLife = r.kappa > 1e-12 ? Math.log(2) / r.kappa : Infinity;
    const trades = (r.trade_schedule || []).map(Math.abs);
    const maxSlice = trades.length ? Math.max(...trades) : 0;
    const avgSlice = trades.length ? trades.reduce((a, b) => a + b, 0) / trades.length : 0;
    const concentration = avgSlice > 0 ? maxSlice / avgSlice : 0;
    document.getElementById('ac-summary').innerHTML = [
        card('κ',                   r.kappa.toExponential(3)),
        card('Half-life (κ)',       fmtSeconds(halfLife)),
        card('Expected impact $',   fmtBig(r.expected_impact_cost)),
        card('Risk variance',       fmtBig(r.risk_variance)),
        card('Slices',              String(r.trade_schedule.length)),
        card('Max-slice / avg',     concentration.toFixed(2) + '×'),
        card('Total traded',        fmtBig(trades.reduce((a, b) => a + b, 0)),
            Math.sign(state.params.total_shares) >= 0 ? 'pos' : 'neg'),
        card('First-slice share',   fmtPct(trades.length ? trades[0] / trades.reduce((a, b) => a + b, 0) : 0)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderTrajectory(r) {
    if (!window.uPlot) return;
    const xsInv  = timeAxis(state.params.horizon_seconds, state.params.n_intervals, 'inventory');
    const xsSch  = timeAxis(state.params.horizon_seconds, state.params.n_intervals, 'schedule');
    const elInv = document.getElementById('ac-chart-inv');
    new window.uPlot({
        title: '', width: elInv.clientWidth || 600, height: 260,
        scales: { x: {}, y: {} },
        series: [
            { label: 't (s)' },
            { label: 'inventory x_k', stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff1A', points: { show: true, size: 5 } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 60 }],
        legend: { show: true },
    }, [xsInv, r.inventory_path], elInv);

    const elSch = document.getElementById('ac-chart-sched');
    new window.uPlot({
        title: '', width: elSch.clientWidth || 600, height: 240,
        scales: { x: {}, y: {} },
        series: [
            { label: 't (s)' },
            { label: 'slice v_k', stroke: '#ff9f1a', width: 1.2,
              fill: '#ff9f1a33', points: { show: true, size: 4 } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 60 }],
        legend: { show: true },
    }, [xsSch, r.trade_schedule], elSch);
}

function renderFrontier(points, currentLambda) {
    if (!window.uPlot || points.length === 0) return;
    const el = document.getElementById('ac-chart-frontier');
    const variances = points.map(p => p.variance);
    const costs     = points.map(p => p.cost);
    const lambdas   = points.map(p => p.lambda);
    const youIdx    = nearestLambdaIndex(lambdas, currentLambda);
    const youXs = variances.map((_, i) => i === youIdx ? variances[i] : null);
    const youYs = costs.map((_, i) => i === youIdx ? costs[i] : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 280,
        scales: { x: {}, y: {} },
        series: [
            { label: 'variance' },
            { label: 'expected cost ($)', stroke: '#a06bff', width: 1.5,
              fill: '#a06bff1A', points: { show: true, size: 6 } },
            { label: 'you are here', stroke: '#fff', width: 0,
              points: { show: true, size: 12, stroke: '#fff', fill: 'transparent' } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 60 }],
        legend: { show: true },
    }, [variances, costs, youYs], el);
    void youXs;
}

function showErr(msg) {
    const el = document.getElementById('ac-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ac-err').style.display = 'none'; }
