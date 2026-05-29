// Option Payoff Diagram view — multi-leg strategy builder + visual P/L.
//
// Visualizes two curves over a spot-price grid:
//   * At-expiry P/L (intrinsic - premium per leg, summed)        ← solid
//   * Current MTM P/L (Black-Scholes per leg at given T, σ, r)   ← dashed
//
// Backed by `/analytics/option-payoff-diagram` (at-expiry, breakevens,
// max profit/loss) and `/analytics/multi-leg-option-pricer` (MTM at
// every spot point in the grid, called once for the current spot to
// show the live strategy value).
//
// Presets: long call/put, straddle, strangle, vertical spreads, iron
// condor, iron butterfly, covered call. Custom legs editable inline.

import { api } from '../api.js';
import { esc, fmt, fmtMoney } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    PRESETS, validateLegs,
    buildPayoffBody, buildPricerBody, defaultSpotRange,
} from '../_option_strategy_presets.js';

const DEFAULTS = {
    spot: 100,
    t_to_expiry: 30 / 365.0,    // 30 calendar days
    rate: 0.05,
    div_yield: 0.0,
    sigma: 0.25,
    steps: 121,
    legs: PRESETS.long_call(100),
};

// View-local state (single instance — the user is on one payoff view at a time).
let state = clone(DEFAULTS);

function clone(o) { return JSON.parse(JSON.stringify(o)); }

export async function renderOptionPayoff(mount, _appState) {
    const tok = currentViewToken();
    state = state || clone(DEFAULTS);

    mount.innerHTML = `
        <h1 data-i18n="view.option_payoff.h1.option_payoff" class="view-title">// OPTION PAYOFF</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.option_payoff.h2.strategy">Strategy</h2>
            <div class="inline-form">
                <label>Preset
                    <select id="op-preset">
                        <option data-i18n="view.option_payoff.opt.custom" value="">— custom —</option>
                        <option data-i18n="view.option_payoff.opt.long_call" value="long_call">Long call</option>
                        <option data-i18n="view.option_payoff.opt.long_put" value="long_put">Long put</option>
                        <option data-i18n="view.option_payoff.opt.long_straddle" value="long_straddle">Long straddle</option>
                        <option data-i18n="view.option_payoff.opt.long_strangle" value="long_strangle">Long strangle</option>
                        <option data-i18n="view.option_payoff.opt.bull_call_spread" value="bull_call_spread">Bull call spread</option>
                        <option data-i18n="view.option_payoff.opt.bear_put_spread" value="bear_put_spread">Bear put spread</option>
                        <option data-i18n="view.option_payoff.opt.iron_condor" value="iron_condor">Iron condor</option>
                        <option data-i18n="view.option_payoff.opt.iron_butterfly" value="iron_butterfly">Iron butterfly</option>
                        <option data-i18n="view.option_payoff.opt.covered_call" value="covered_call">Covered call</option>
                    </select>
                </label>
                <label>Spot <input id="op-spot" type="number" step="any" value="${state.spot}"></label>
                <label>T (years) <input id="op-t" type="number" step="any" value="${state.t_to_expiry}"></label>
                <label>Rate <input id="op-rate" type="number" step="any" value="${state.rate}"></label>
                <label>Div yield <input id="op-q" type="number" step="any" value="${state.div_yield}"></label>
                <label>IV <input id="op-sigma" type="number" step="any" value="${state.sigma}"></label>
                <button data-i18n="view.option_payoff.btn.recalculate" id="op-recalc" class="primary" type="button">Recalculate</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.option_payoff.h2.legs">Legs</h2>
            <table class="op-legs">
                <thead><tr>
                    <th>#</th><th data-i18n="view.option_payoff.th.kind">Kind</th><th data-i18n="view.option_payoff.th.strike">Strike</th>
                    <th data-i18n="view.option_payoff.th.premium">Premium</th><th data-i18n="view.option_payoff.th.qty_short">Qty (− short)</th><th></th>
                </tr></thead>
                <tbody id="op-legs-body"></tbody>
            </table>
            <button data-i18n="view.option_payoff.btn.add_leg" id="op-add-leg" class="secondary" type="button">+ Add leg</button>
        </div>

        <div id="op-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.option_payoff.h2.payoff_curve">Payoff curve</h2>
            <div id="op-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.option_payoff.hint.solid_p_l_at_expiry_dashed_current_mtm_black_schol" class="muted">Solid = P/L at expiry · Dashed = current MTM (Black-Scholes per leg)</p>
        </div>

        <div id="op-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    renderLegsTable();
    wireForm(mount, tok);
    await recalc(mount, tok);
    void fmt;
}

function renderLegsTable() {
    const tbody = document.getElementById('op-legs-body');
    tbody.innerHTML = state.legs.map((l, i) => `
        <tr>
            <td>${i + 1}</td>
            <td>
                <select data-leg="${i}" data-field="kind">
                    <option data-i18n="view.option_payoff.opt.call" value="call"       ${l.kind === 'call' ? 'selected' : ''}>Call</option>
                    <option data-i18n="view.option_payoff.opt.put" value="put"        ${l.kind === 'put' ? 'selected' : ''}>Put</option>
                    <option data-i18n="view.option_payoff.opt.stock" value="underlying" ${l.kind === 'underlying' ? 'selected' : ''}>Stock</option>
                </select>
            </td>
            <td><input type="number" step="any" value="${l.strike}"  data-leg="${i}" data-field="strike"></td>
            <td><input type="number" step="any" value="${l.premium}" data-leg="${i}" data-field="premium"></td>
            <td><input type="number" step="any" value="${l.qty}"     data-leg="${i}" data-field="qty"></td>
            <td><button type="button" class="secondary" data-leg-delete="${i}">×</button></td>
        </tr>
    `).join('');

    tbody.querySelectorAll('select[data-leg], input[data-leg]').forEach(el => {
        el.addEventListener('change', e => {
            const i = Number(e.target.dataset.leg);
            const f = e.target.dataset.field;
            const v = e.target.tagName === 'SELECT' ? e.target.value : Number(e.target.value);
            state.legs[i][f] = v;
        });
    });
    tbody.querySelectorAll('button[data-leg-delete]').forEach(btn => {
        btn.addEventListener('click', e => {
            const i = Number(e.target.dataset.legDelete);
            state.legs.splice(i, 1);
            renderLegsTable();
        });
    });
}

function wireForm(mount, tok) {
    document.getElementById('op-preset').addEventListener('change', (e) => {
        const id = e.target.value;
        if (id && PRESETS[id]) {
            state.legs = PRESETS[id](state.spot);
            renderLegsTable();
        }
    });
    document.getElementById('op-add-leg').addEventListener('click', () => {
        state.legs.push({
            kind: 'call',
            strike: Math.round(state.spot),
            premium: Math.round(state.spot * 0.03),
            qty: 1,
        });
        renderLegsTable();
    });
    document.getElementById('op-recalc').addEventListener('click', () => {
        state.spot       = Number(document.getElementById('op-spot').value);
        state.t_to_expiry = Number(document.getElementById('op-t').value);
        state.rate       = Number(document.getElementById('op-rate').value);
        state.div_yield  = Number(document.getElementById('op-q').value);
        state.sigma      = Number(document.getElementById('op-sigma').value);
        void recalc(mount, tok);
    });
}

async function recalc(mount, tok) {
    const err = document.getElementById('op-err');
    err.style.display = 'none';

    const validation = validateLegs(state.legs);
    if (validation) { showError(validation); return; }

    const { min: smin, max: smax } = defaultSpotRange(state.spot);
    let payoff, mtmAtSpot;
    try {
        payoff = await api.anlyOptionPayoffDiagram(
            buildPayoffBody(state.legs, smin, smax, state.steps),
        );
        if (!payoff) throw new Error('payoff returned null');
        mtmAtSpot = await api.anlyMultiLegOptionPricer(
            buildPricerBody(state.legs, state.spot, state.t_to_expiry,
                            state.rate, state.div_yield, state.sigma),
        );
    } catch (e) {
        showError(`API error: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(payoff, mtmAtSpot);
    await renderChart(payoff);

    function showError(msg) {
        err.textContent = msg;
        err.style.display = 'block';
    }
}

function renderSummary(payoff, mtm) {
    const breakevensCell = (payoff.breakevens && payoff.breakevens.length)
        ? payoff.breakevens.map(b => b.toFixed(2)).join(' · ')
        : '—';
    const mtmPnl = mtm ? mtm.strategy_pnl : null;
    const mtmValue = mtm ? mtm.strategy_value : null;
    document.getElementById('op-summary').innerHTML = `
        <div class="card"><div class="label">Max profit (@ expiry)</div>
            <div class="value pos">${fmtMoney(payoff.max_profit)}</div></div>
        <div class="card"><div class="label">Max loss (@ expiry)</div>
            <div class="value neg">${fmtMoney(payoff.max_loss)}</div></div>
        <div class="card"><div class="label">Breakevens (@ expiry)</div>
            <div class="value">${esc(breakevensCell)}</div></div>
        <div class="card"><div class="label">Strategy value (now)</div>
            <div class="value">${mtmValue == null ? '—' : fmtMoney(mtmValue)}</div></div>
        <div class="card"><div class="label">Strategy P/L (now)</div>
            <div class="value ${mtmPnl >= 0 ? 'pos' : 'neg'}">
                ${mtmPnl == null ? '—' : fmtMoney(mtmPnl)}
            </div></div>
    `;
}

async function renderChart(payoff) {
    const el = document.getElementById('op-chart');
    if (!window.uPlot) {
        el.textContent = 'uPlot not loaded — run scripts/vendor-uplot.sh';
        return;
    }
    el.innerHTML = '';

    // X-axis = spot price; series 1 = at-expiry P/L; series 2 = current MTM curve.
    const xs = payoff.spots;
    const atExpiry = payoff.pnls;
    const mtmCurve = await mtmAcrossSpots(xs);

    const w = el.clientWidth || 800;
    const h = 340;

    // Add a zero-line marker via an axis split.
    new window.uPlot({
        title: '',
        width: w,
        height: h,
        scales: { x: {}, y: {} },
        series: [
            { label: 'spot' },
            {
                label: 'P/L @ expiry',
                stroke: '#00e5ff',
                width: 2,
                fill: 'rgba(0,229,255,0.05)',
            },
            {
                label: 'MTM now',
                stroke: '#ff9f1a',
                width: 1,
                dash: [6, 4],
            },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, atExpiry, mtmCurve], el);
}

// Call the multi-leg pricer once per spot point for the dashed MTM curve.
// This is O(N) backend calls; the analytics endpoints are pure CPU and
// fast enough at 121 steps that the round-trip is dominated by network.
// Caller can throttle by lowering `state.steps` if it gets sluggish.
async function mtmAcrossSpots(xs) {
    const requests = xs.map(s => api.anlyMultiLegOptionPricer(
        buildPricerBody(state.legs, s, state.t_to_expiry,
                        state.rate, state.div_yield, state.sigma),
    ));
    const results = await Promise.all(requests);
    return results.map(r => (r && Number.isFinite(r.strategy_pnl)) ? r.strategy_pnl : null);
}
