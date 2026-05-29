// American Option Pricer view — Longstaff-Schwartz Monte Carlo vs
// Black-Scholes European reference.
//
// Outputs:
//   * LSMC price + standard error + 95% CI.
//   * European reference (computed locally in JS — no extra round-trip).
//   * Early-exercise premium = American − European.
//   * Intrinsic value (parity floor).
//
// Famous results to sanity-check against:
//   * American call on a non-dividend stock ≈ European call (no early
//     exercise premium). Our LSMC should land within MC error of BS.
//   * American put on a non-dividend stock should be strictly ≥
//     European put for ITM strikes (early-exercise premium > 0).

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    buildLsmcBody, validateLsmcParams,
    blackScholesEuropean, earlyExercisePremium,
    fmtMoney, ciHalfWidth,
} from '../_american_option_inputs.js';

const DEFAULT_PARAMS = {
    kind: 'put',     // puts are where early exercise matters most
    spot: 90,
    strike: 100,
    t_years: 0.5,
    rate: 0.05,
    dividend: 0.0,
    sigma: 0.25,
    steps: 50,
    paths: 5000,
    seed: 42,
};

let state = { params: { ...DEFAULT_PARAMS }, lastResult: null };

export async function renderAmericanOption(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 class="view-title">// AMERICAN OPTION (LSMC)</h1>

        <div class="chart-panel">
            <h2>Contract</h2>
            <div class="inline-form">
                <label>Kind
                    <select id="ao-kind">
                        <option value="call" ${state.params.kind === 'call' ? 'selected' : ''}>Call</option>
                        <option value="put" ${state.params.kind === 'put' ? 'selected' : ''}>Put</option>
                    </select></label>
                <label>Spot <input id="ao-spot" type="number" step="any" min="0" value="${state.params.spot}"></label>
                <label>Strike <input id="ao-strike" type="number" step="any" min="0" value="${state.params.strike}"></label>
                <label>T (years) <input id="ao-t" type="number" step="any" min="0" value="${state.params.t_years}"></label>
                <label>Rate r <input id="ao-rate" type="number" step="any" value="${state.params.rate}"></label>
                <label>Dividend q <input id="ao-div" type="number" step="any" min="0" value="${state.params.dividend}"></label>
                <label>σ <input id="ao-sigma" type="number" step="any" min="0" value="${state.params.sigma}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2>Monte Carlo settings</h2>
            <div class="inline-form">
                <label>Steps <input id="ao-steps" type="number" step="1" min="2" value="${state.params.steps}"></label>
                <label>Paths <input id="ao-paths" type="number" step="100" min="10" value="${state.params.paths}"></label>
                <label>Seed (0 = auto) <input id="ao-seed" type="number" step="1" min="0" value="${state.params.seed}"></label>
                <button id="ao-run" class="primary" type="button">Price</button>
            </div>
            <p class="muted">
                Longstaff-Schwartz 2001 regression-Monte-Carlo. Larger paths shrink the standard
                error as 1/√N. Default 5,000 paths gives ~1% SE for at-the-money options.
            </p>
        </div>

        <div id="ao-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Sensitivity to spot</h2>
            <div id="ao-chart" style="width:100%;height:340px"></div>
            <p class="muted" id="ao-chart-caption">
                Click <em>Price</em> first. The chart plots the European Black-Scholes price
                across a spot range (instant, no MC) and overlays a single MC point at your
                input spot to show where the LSMC estimate sits relative to the parametric
                European curve.
            </p>
        </div>

        <div id="ao-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('ao-run').addEventListener('click', () => {
        readParams();
        void price(mount, tok);
    });
    void fmt;
}

function readParams() {
    const get = id => document.getElementById(id).value;
    state.params = {
        kind: get('ao-kind'),
        spot: Number(get('ao-spot')),
        strike: Number(get('ao-strike')),
        t_years: Number(get('ao-t')),
        rate: Number(get('ao-rate')),
        dividend: Number(get('ao-div')),
        sigma: Number(get('ao-sigma')),
        steps: parseInt(get('ao-steps'), 10),
        paths: parseInt(get('ao-paths'), 10),
        seed: parseInt(get('ao-seed'), 10),
    };
}

async function price(mount, tok) {
    hideErr();
    const err = validateLsmcParams(state.params);
    if (err) { showErr(err); return; }

    let res;
    try {
        res = await api.anlyAmericanOptionLsmc(buildLsmcBody(state.params));
        if (!res) throw new Error('LSMC returned null (parameters out of domain)');
    } catch (e) {
        showErr(`API error: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    state.lastResult = res;

    renderSummary(res);
    renderChart(res);
}

function renderSummary(res) {
    const p = state.params;
    const european = blackScholesEuropean(
        p.kind, p.spot, p.strike, p.t_years, p.rate, p.dividend, p.sigma,
    );
    const intrinsic = p.kind === 'call'
        ? Math.max(p.spot - p.strike, 0)
        : Math.max(p.strike - p.spot, 0);
    const eep = earlyExercisePremium(res.price, european);
    const eepPct = (res.price > 0 && eep != null) ? (eep / res.price * 100) : null;
    const halfWidth = ciHalfWidth(res.standard_error);

    const cards = [];
    cards.push(card('American (LSMC)', fmtMoney(res.price), '', `
        <div class="vc-row"><span class="muted">95% CI</span> <strong>±${fmtMoney(halfWidth)}</strong></div>
        <div class="vc-row"><span class="muted">SE</span> <strong>${fmtMoney(res.standard_error, 5)}</strong></div>
        <div class="vc-row"><span class="muted">Paths</span> <strong>${res.paths_run}</strong></div>
    `));
    cards.push(card('European (BS reference)', fmtMoney(european), '', `
        <div class="vc-row"><span class="muted">Computed</span> <strong>closed-form</strong></div>
        <div class="vc-row"><span class="muted">No MC error</span> <strong>—</strong></div>
    `));
    cards.push(card('Early-exercise premium',
        eep == null ? '—' : fmtMoney(eep),
        eep != null && eep > 0 ? 'pos' : (eep != null && eep < 0 ? 'neg' : ''),
        `<div class="vc-row"><span class="muted">% of American price</span>
             <strong>${eepPct == null ? '—' : eepPct.toFixed(2) + '%'}</strong></div>
         <div class="vc-row"><span class="muted">Sign</span>
             <strong>${eep == null ? '—' : (eep > 0 ? 'EE valuable' : 'within MC noise')}</strong></div>`));
    cards.push(card('Intrinsic value (parity floor)', fmtMoney(intrinsic), '', `
        <div class="vc-row"><span class="muted">${p.kind === 'call' ? 'max(S - K, 0)' : 'max(K - S, 0)'}</span>
            <strong>${fmtMoney(intrinsic)}</strong></div>
        <div class="vc-row"><span class="muted">LSMC ≥ intrinsic</span>
            <strong>${res.price + 1.96 * res.standard_error >= intrinsic - 1e-9 ? '✓' : '✗'}</strong></div>
    `));
    document.getElementById('ao-summary').innerHTML = cards.join('');
}

function card(label, value, valueCls, body) {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${valueCls}">${esc(value)}</div>
        <div class="value ao-summary-value">${body}</div>
    </div>`;
}

function renderChart(res) {
    const el = document.getElementById('ao-chart');
    if (!window.uPlot) {
        el.textContent = 'uPlot not loaded';
        return;
    }
    el.innerHTML = '';
    const p = state.params;
    // Spot range = ±50% of input spot, 121 points. European curve only.
    const lo = Math.max(p.spot * 0.5, 1e-6);
    const hi = p.spot * 1.5;
    const n = 121;
    const xs = new Array(n);
    const ys = new Array(n);
    for (let i = 0; i < n; i++) {
        const s = lo + (hi - lo) * i / (n - 1);
        xs[i] = s;
        ys[i] = blackScholesEuropean(p.kind, s, p.strike, p.t_years, p.rate, p.dividend, p.sigma);
    }
    // Single MC point at user's spot.
    const mcPoint = xs.map(s => Math.abs(s - p.spot) < (hi - lo) / (2 * (n - 1)) ? res.price : null);

    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: 'spot' },
            { label: 'European (BS)', stroke: '#00e5ff', width: 2,
              points: { show: false } },
            { label: 'American (LSMC)', stroke: '#ff9f1a', width: 0,
              points: { show: true, size: 10, stroke: '#ff9f1a', fill: '#ff9f1a' } },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, ys, mcPoint], el);
}

function showErr(msg) {
    const el = document.getElementById('ao-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ao-err').style.display = 'none'; }
