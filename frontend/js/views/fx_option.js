// FX Option Calculator — Garman-Kohlhagen pricer with all 7 outputs
// the backend returns: price, delta, gamma, vega, theta, ρ_domestic,
// ρ_foreign.
//
// Why the chart uses local computation: greeks come from the backend
// for the user's input point, but rendering price vs spot needs ~100
// price evaluations across a spot range. Doing those server-side would
// be 100 round-trips. The closed-form GK is short enough to evaluate
// locally — see `_fx_option_inputs.js`.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t, applyUiI18n } from '../i18n.js';
import {
    buildGkBody, validateGkParams,
    garmanKohlhagenPrice, fmtRate, fmtGreek,
} from '../_fx_option_inputs.js';

const DEFAULT_PARAMS = {
    kind: 'call',
    spot: 1.0850,       // EURUSD-like
    strike: 1.1000,
    t_years: 0.25,      // 3 months
    rate_dom: 0.045,    // USD short rate
    rate_for: 0.035,    // EUR short rate
    sigma: 0.08,        // 8% FX vol
};

let state = { params: { ...DEFAULT_PARAMS } };

export async function renderFxOption(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.fx_option.h1.fx_option_garman_kohlhagen" class="view-title">// FX OPTION (GARMAN-KOHLHAGEN)</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.fx_option.h2.contract">Contract</h2>
            <div class="inline-form">
                <label><span data-i18n="view.fx_option.label.kind">Kind</span>
                    <select id="fx-kind">
                        <option data-i18n="view.fx_option.opt.call_right_to_buy_foreign" value="call" ${state.params.kind === 'call' ? 'selected' : ''}>Call (right to buy foreign)</option>
                        <option data-i18n="view.fx_option.opt.put_right_to_sell_foreign" value="put"  ${state.params.kind === 'put' ? 'selected' : ''}>Put (right to sell foreign)</option>
                    </select></label>
                <label><span data-i18n="view.fx_option.label.spot">Spot (dom/for)</span>
                    <input id="fx-spot" type="number" step="any" min="0" value="${state.params.spot}"></label>
                <label><span data-i18n="view.fx_option.label.strike">Strike</span>
                    <input id="fx-strike" type="number" step="any" min="0" value="${state.params.strike}"></label>
                <label><span data-i18n="view.fx_option.label.t_years">T (years)</span>
                    <input id="fx-t" type="number" step="any" min="0" value="${state.params.t_years}"></label>
                <label><span data-i18n="view.fx_option.label.rate_dom">Rate domestic</span>
                    <input id="fx-rd" type="number" step="any" value="${state.params.rate_dom}"></label>
                <label><span data-i18n="view.fx_option.label.rate_for">Rate foreign</span>
                    <input id="fx-rf" type="number" step="any" value="${state.params.rate_for}"></label>
                <label><span data-i18n="view.fx_option.label.sigma">σ (FX vol)</span>
                    <input id="fx-sigma" type="number" step="any" min="0" value="${state.params.sigma}"></label>
                <button data-i18n="view.fx_option.btn.price" id="fx-run" class="primary" type="button">Price</button>
            </div>
            <p class="muted">
                Spot is quoted as <em>domestic per 1 unit of foreign</em> — e.g. EURUSD 1.0850
                means 1 EUR = 1.0850 USD (USD is domestic, EUR is foreign). Rate-domestic
                discounts the strike; rate-foreign discounts the spot.
            </p>
        </div>

        <div id="fx-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.fx_option.h2.price_vs_spot">Price vs spot</h2>
            <div id="fx-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.fx_option.hint.garman_kohlhagen_price_across_20_of_the_input_spot" class="muted">
                Garman-Kohlhagen price across ±20% of the input spot. The orange marker is
                the backend's response at the user's spot — should sit exactly on the curve
                (sanity check that the local closed-form matches the Rust module).
            </p>
        </div>

        <div id="fx-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('fx-run').addEventListener('click', () => {
        readParams();
        void price(mount, tok);
    });
    void fmt;
}

function readParams() {
    const get = id => document.getElementById(id).value;
    state.params = {
        kind: get('fx-kind'),
        spot: Number(get('fx-spot')),
        strike: Number(get('fx-strike')),
        t_years: Number(get('fx-t')),
        rate_dom: Number(get('fx-rd')),
        rate_for: Number(get('fx-rf')),
        sigma: Number(get('fx-sigma')),
    };
}

async function price(mount, tok) {
    hideErr();
    const err = validateGkParams(state.params);
    if (err) { showErr(err); return; }

    let res;
    try {
        res = await api.anlyGarmanKohlhagenFxOption(buildGkBody(state.params));
        if (!res) throw new Error(t('view.fx_option.error.null_result'));
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
    const intrinsic = p.kind === 'call'
        ? Math.max(p.spot - p.strike, 0)
        : Math.max(p.strike - p.spot, 0);
    const timeValue = res.price - intrinsic;

    const fxSummary = document.getElementById('fx-summary');
    fxSummary.innerHTML = [
        bigCard('Price', fmtRate(res.price), `
            <div class="vc-row"><span class="muted" data-i18n="view.fx_option.row.intrinsic">Intrinsic</span> <strong>${fmtRate(intrinsic)}</strong></div>
            <div class="vc-row"><span class="muted" data-i18n="view.fx_option.row.time_value">Time value</span> <strong>${fmtRate(timeValue)}</strong></div>
        `),
        bigCard('Delta', fmtGreek(res.delta, 4), `
            <div class="vc-row"><span class="muted" data-i18n="view.fx_option.row.hedge_ratio">Hedge ratio</span> <strong>${fmtGreek(res.delta, 4)}</strong></div>
            <div class="vc-row"><span class="muted" data-i18n="view.fx_option.row.atm_target">ATM target</span>
                <strong>${p.kind === 'call' ? '~ +0.5' : '~ −0.5'}</strong></div>
        `),
        bigCard('Gamma', fmtGreek(res.gamma), `
            <div class="vc-row"><span class="muted">d²V/dS²</span> <strong>${fmtGreek(res.gamma)}</strong></div>
            <div class="vc-row"><span class="muted" data-i18n="view.fx_option.row.peak">Peak</span> <strong data-i18n="view.fx_option.row.at_atm">at ATM</strong></div>
        `),
        bigCard('Vega', fmtGreek(res.vega), `
            <div class="vc-row"><span class="muted">per 1.00 σ</span> <strong>${fmtGreek(res.vega)}</strong></div>
            <div class="vc-row"><span class="muted">per 1% σ</span> <strong>${fmtGreek(res.vega / 100)}</strong></div>
        `),
        bigCard('Theta', fmtGreek(res.theta), `
            <div class="vc-row"><span class="muted">per year</span> <strong>${fmtGreek(res.theta)}</strong></div>
            <div class="vc-row"><span class="muted">per day (365)</span> <strong>${fmtGreek(res.theta / 365)}</strong></div>
        `),
        bigCard('ρ domestic', fmtGreek(res.rho_domestic), `
            <div class="vc-row"><span class="muted">dV/dr_d</span> <strong>${fmtGreek(res.rho_domestic)}</strong></div>
        `),
        bigCard('ρ foreign', fmtGreek(res.rho_foreign), `
            <div class="vc-row"><span class="muted">dV/dr_f</span> <strong>${fmtGreek(res.rho_foreign)}</strong></div>
        `),
    ].join('');
    try { applyUiI18n(fxSummary); } catch (_) {}
}

function bigCard(label, value, body) {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value">${esc(value)}</div>
        <div class="value fx-summary-value">${body}</div>
    </div>`;
}

function renderChart(res) {
    const el = document.getElementById('fx-chart');
    if (!window.uPlot) {
        el.textContent = t('common.error.uplot_not_loaded');
        return;
    }
    el.innerHTML = '';
    const p = state.params;
    // ±20% of input spot, 121 points (FX vols are usually low and the
    // sensitivity is gentle, so a tighter range than the BS view).
    const lo = Math.max(p.spot * 0.8, 1e-9);
    const hi = p.spot * 1.2;
    const n = 121;
    const xs = new Array(n);
    const ys = new Array(n);
    for (let i = 0; i < n; i++) {
        const s = lo + (hi - lo) * i / (n - 1);
        xs[i] = s;
        ys[i] = garmanKohlhagenPrice(p.kind, s, p.strike, p.t_years, p.rate_dom, p.rate_for, p.sigma);
    }
    const half = (hi - lo) / (2 * (n - 1));
    const mcPoint = xs.map(s => Math.abs(s - p.spot) < half ? res.price : null);

    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: 'spot' },
            { label: 'GK (local)', stroke: '#00e5ff', width: 2,
              points: { show: false } },
            { label: 'backend response', stroke: '#ff9f1a', width: 0,
              points: { show: true, size: 10, stroke: '#ff9f1a', fill: '#ff9f1a' } },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, ys, mcPoint], el);
}

function showErr(msg) {
    const el = document.getElementById('fx-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('fx-err').style.display = 'none'; }
