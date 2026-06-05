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
import { applyUiI18n, t } from '../i18n.js';
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
                <label><span data-i18n="view.option_payoff.label.preset">Preset</span>
                    <select id="op-preset" data-tip="view.option_payoff.tip.preset">
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
                <label><span data-i18n="view.option_payoff.label.spot">Spot</span>
                    <input id="op-spot" type="number" step="0.01" value="${state.spot}" data-tip="view.option_payoff.tip.spot"></label>
                <label><span data-i18n="view.option_payoff.label.t">T (years)</span>
                    <input id="op-t" type="number" step="0.01" value="${state.t_to_expiry}" data-tip="view.option_payoff.tip.t"></label>
                <label><span data-i18n="view.option_payoff.label.rate">Rate</span>
                    <input id="op-rate" type="number" step="0.01" value="${state.rate}" data-tip="view.option_payoff.tip.rate"></label>
                <label><span data-i18n="view.option_payoff.label.div_yield">Div yield</span>
                    <input id="op-q" type="number" step="0.01" value="${state.div_yield}" data-tip="view.option_payoff.tip.div_yield"></label>
                <label><span data-i18n="view.option_payoff.label.iv">IV</span>
                    <input id="op-sigma" type="number" step="0.01" value="${state.sigma}" data-tip="view.option_payoff.tip.iv"></label>
                <button data-i18n="view.option_payoff.btn.recalculate" data-tip="view.option_payoff.tip.recalc" data-shortcut="option_payoff_recalc" id="op-recalc" class="primary" type="button">Recalculate</button>
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
            <button data-i18n="view.option_payoff.btn.add_leg" data-tip="view.option_payoff.tip.add_leg" id="op-add-leg" class="secondary" type="button">+ Add leg</button>
        </div>

        <div id="op-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.option_payoff.h2.payoff_curve">Payoff curve</h2>
            <div id="op-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.option_payoff.hint.solid_p_l_at_expiry_dashed_current_mtm_black_schol" class="muted">Solid = P/L at expiry · Dashed = current MTM (Black-Scholes per leg)</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.option_payoff.h2.delta_chart">At-expiry delta (slope of payoff curve)</h2>
            <div id="op-delta-chart" style="width:100%;height:240px"></div>
            <p data-i18n="view.option_payoff.hint.delta" class="muted small">First difference of the at-expiry payoff at each spot — that's the strategy's terminal delta. Reveals where the position becomes net long, short, or flat across the spot range.</p>
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
            <td><input type="number" step="0.01" value="${l.strike}"  data-leg="${i}" data-field="strike"></td>
            <td><input type="number" step="0.01" value="${l.premium}" data-leg="${i}" data-field="premium"></td>
            <td><input type="number" step="0.01" value="${l.qty}"     data-leg="${i}" data-field="qty"></td>
            <td><button type="button" class="secondary" data-leg-delete="${i}" data-i18n-aria-label="common.aria.remove" aria-label="Remove">×</button></td>
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
        if (!payoff) throw new Error(t('view.option_payoff.error.null'));
        mtmAtSpot = await api.anlyMultiLegOptionPricer(
            buildPricerBody(state.legs, state.spot, state.t_to_expiry,
                            state.rate, state.div_yield, state.sigma),
        );
    } catch (e) {
        showError(t('view.option_payoff.error.api', { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(payoff, mtmAtSpot);
    await renderChart(payoff);
    renderDeltaChart(payoff);

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
    const opSummary = document.getElementById('op-summary');
    opSummary.innerHTML = `
        <div class="card"><div class="label" data-i18n="view.option_payoff.card.max_profit">Max profit (@ expiry)</div>
            <div class="value pos">${fmtMoney(payoff.max_profit)}</div></div>
        <div class="card"><div class="label" data-i18n="view.option_payoff.card.max_loss">Max loss (@ expiry)</div>
            <div class="value neg">${fmtMoney(payoff.max_loss)}</div></div>
        <div class="card"><div class="label" data-i18n="view.option_payoff.card.breakevens">Breakevens (@ expiry)</div>
            <div class="value">${esc(breakevensCell)}</div></div>
        <div class="card"><div class="label" data-i18n="view.option_payoff.card.strategy_value">Strategy value (now)</div>
            <div class="value">${mtmValue == null ? '—' : fmtMoney(mtmValue)}</div></div>
        <div class="card"><div class="label" data-i18n="view.option_payoff.card.strategy_pnl">Strategy P/L (now)</div>
            <div class="value ${mtmPnl >= 0 ? 'pos' : 'neg'}">
                ${mtmPnl == null ? '—' : fmtMoney(mtmPnl)}
            </div></div>
    `;
    try { applyUiI18n(opSummary); } catch (_) {}
}

async function renderChart(payoff) {
    const el = document.getElementById('op-chart');
    if (!window.uPlot) {
        el.textContent = t('common.error.uplot_not_loaded_install');
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
            { label: t('chart.series.spot') },
            {
                label: t('chart.series.pl_at_expiry'),
                stroke: '#00e5ff',
                width: 2,
                fill: 'rgba(0,229,255,0.05)',
            },
            {
                label: t('chart.series.mtm_now'),
                stroke: '#ff9f1a',
                width: 1,
                dash: [6, 4],
            },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, atExpiry, mtmCurve], el);
}

function renderDeltaChart(payoff) {
    const el = document.getElementById('op-delta-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const xs = payoff.spots || [];
    const ys = payoff.pnls || [];
    if (xs.length < 2 || ys.length !== xs.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.option_payoff.empty_delta_chart">${esc(t('view.option_payoff.empty_delta_chart'))}</div>`;
        return;
    }
    const delta = new Array(xs.length).fill(0);
    for (let i = 1; i < xs.length - 1; i++) {
        delta[i] = (ys[i + 1] - ys[i - 1]) / (xs[i + 1] - xs[i - 1]);
    }
    delta[0] = delta[1];
    delta[xs.length - 1] = delta[xs.length - 2];
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('chart.series.spot') },
            { label: t('view.option_payoff.chart.delta'),
              stroke: '#b86bff', width: 1.5, points: { show: false } },
            { label: t('view.option_payoff.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
        legend: { show: true },
    }, [xs, delta, zero], el);
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
