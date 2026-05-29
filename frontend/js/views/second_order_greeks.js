// Second-Order Greeks view — plots vanna / charm / vomma / veta as
// functions of spot. Computed client-side from the same closed-form the
// backend uses; backend is hit once at ATM as a canonical-value check.
//
// Why traders care:
//   * vanna shifts your delta as IV moves — short vanna is what blows up
//     long calls when realized vol drops below implied.
//   * charm is your overnight delta-drift; large negative charm on a
//     long OTM call means waking up shorter than you went to bed.
//   * vomma (volga) tells you whether your vega is convex in σ; high
//     vomma → strangles benefit nonlinearly from vol shocks.
//   * veta is vega's time-decay — vega isn't free even when σ is flat.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    buildBody, validateParams, computeGrid, computePoint,
    METRICS, fmtN, defaultSpotGrid, nearestAtmIndex,
} from '../_second_order_greeks_inputs.js';

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
    vanna: '#ff9f1a', charm: '#a06bff', vomma: '#39ff14', veta: '#ff3860',
};

const METRIC_LABELS = {
    vanna: 'vanna  ∂Δ/∂σ',
    charm: 'charm  ∂Δ/∂t',
    vomma: 'vomma  ∂vega/∂σ',
    veta:  'veta   ∂vega/∂t',
};

let state = { params: { ...DEFAULT_PARAMS } };

export async function renderSecondOrderGreeks(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 class="view-title">// SECOND-ORDER GREEKS</h1>

        <div class="chart-panel">
            <h2>Contract</h2>
            <div class="inline-form">
                <label>Kind
                    <select id="sg-kind">
                        <option value="call" ${state.params.kind === 'call' ? 'selected' : ''}>Call</option>
                        <option value="put"  ${state.params.kind === 'put'  ? 'selected' : ''}>Put</option>
                    </select></label>
                <label>Strike <input id="sg-strike" type="number" step="any" min="0" value="${state.params.strike}"></label>
                <label>T (years) <input id="sg-t" type="number" step="any" min="0" value="${state.params.time_to_expiry}"></label>
                <label>Rate r <input id="sg-r" type="number" step="any" value="${state.params.risk_free}"></label>
                <label>Dividend q <input id="sg-q" type="number" step="any" min="0" value="${state.params.dividend_yield}"></label>
                <label>σ <input id="sg-sigma" type="number" step="any" min="0" value="${state.params.sigma}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2>Spot grid</h2>
            <div class="inline-form">
                <label>Low <input id="sg-low" type="number" step="any" min="0" value="${state.params.spot_grid_low}"></label>
                <label>High <input id="sg-high" type="number" step="any" min="0" value="${state.params.spot_grid_high}"></label>
                <label>Points <input id="sg-n" type="number" step="1" min="5" max="501" value="${state.params.n_points}"></label>
                <button id="sg-defaults" class="secondary" type="button">±50% from strike</button>
                <button id="sg-run" class="primary" type="button">Compute</button>
            </div>
            <p class="muted">
                Grid values computed client-side (BS closed-form). The "backend ATM"
                card calls /options/calc/second-order-greeks once at spot = strike for a
                canonical reference — local + backend should agree to ~7 dp.
            </p>
        </div>

        <div id="sg-summary" class="cards"></div>

        <div id="sg-grid" class="gp-grid"></div>

        <div id="sg-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('sg-defaults').addEventListener('click', () => {
        const strike = Number(document.getElementById('sg-strike').value);
        const grid = defaultSpotGrid(strike);
        document.getElementById('sg-low').value  = grid.low;
        document.getElementById('sg-high').value = grid.high;
    });
    document.getElementById('sg-run').addEventListener('click', () => {
        readInputs();
        void compute(mount, tok);
    });

    readInputs();
    void compute(mount, tok);
}

function readInputs() {
    const get = id => document.getElementById(id).value;
    state.params = {
        kind:           get('sg-kind'),
        strike:         Number(get('sg-strike')),
        time_to_expiry: Number(get('sg-t')),
        risk_free:      Number(get('sg-r')),
        dividend_yield: Number(get('sg-q')),
        sigma:          Number(get('sg-sigma')),
        spot_grid_low:  Number(get('sg-low')),
        spot_grid_high: Number(get('sg-high')),
        n_points:       parseInt(get('sg-n'), 10),
    };
}

async function compute(_mount, tok) {
    hideErr();
    const err = validateParams(state.params);
    if (err) { showErr(err); return; }

    // Local grid is instant — no backend round-trips per spot.
    const grid = computeGrid(state.params);
    const atmI = nearestAtmIndex(grid.spots, state.params.strike);
    const atmLocal = atmI >= 0
        ? METRICS.reduce((o, m) => (o[m] = grid[m][atmI], o), { spot: grid.spots[atmI] })
        : null;

    // Backend single-point at strike: canonical reference, runs in parallel with rendering.
    let atmBackend = null;
    try {
        const body = buildBody({ ...state.params, spot: state.params.strike });
        atmBackend = await api.anlySecondOrderGreeks(body);
    } catch (e) {
        // Non-fatal — local grid still renders; show as backend "—" in card.
        // eslint-disable-next-line no-console
        console.warn('second-order-greeks backend call failed', e);
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(atmLocal, atmBackend);
    renderGrid(grid, atmLocal);
}

function renderSummary(atmLocal, atmBackend) {
    const has = (atmLocal && Number.isFinite(atmLocal.spot));
    document.getElementById('sg-summary').innerHTML = [
        card('ATM spot',         has ? fmtN(atmLocal.spot, 2) : '—'),
        card('vanna (local)',    fmtN(atmLocal?.vanna)),
        card('vanna (backend)',  fmtN(atmBackend?.vanna)),
        card('charm (local)',    fmtN(atmLocal?.charm), atmLocal?.charm < 0 ? 'neg' : 'pos'),
        card('charm (backend)',  fmtN(atmBackend?.charm)),
        card('vomma (local)',    fmtN(atmLocal?.vomma)),
        card('veta (local)',     fmtN(atmLocal?.veta)),
        card('veta (backend)',   fmtN(atmBackend?.veta)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderGrid(grid, atmLocal) {
    const wrap = document.getElementById('sg-grid');
    wrap.innerHTML = METRICS.map(m =>
        `<div class="gp-cell">
            <div class="gp-cell-label">${esc(METRIC_LABELS[m])}</div>
            <div id="sg-chart-${esc(m)}" class="gp-chart-area"></div>
        </div>`
    ).join('');
    if (!window.uPlot) return;
    const atmSpot = atmLocal?.spot;
    for (const m of METRICS) {
        drawMini(`sg-chart-${m}`, grid.spots, grid[m], METRIC_COLORS[m], atmSpot);
    }
}

function drawMini(elId, xs, ys, stroke, atmSpot) {
    const el = document.getElementById(elId);
    if (!el || xs.length === 0) return;
    const halfWidth = xs.length > 1 ? (xs[1] - xs[0]) / 2 : 0;
    const atmYs = xs.map((s, i) =>
        Number.isFinite(atmSpot) && Math.abs(s - atmSpot) <= halfWidth ? ys[i] : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 400, height: 140,
        scales: { x: {}, y: {} },
        series: [
            { label: 'spot' },
            { label: 'value', stroke, width: 1.5,
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
    const el = document.getElementById('sg-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('sg-err').style.display = 'none'; }
