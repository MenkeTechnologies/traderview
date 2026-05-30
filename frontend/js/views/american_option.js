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

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
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
        <h1 data-i18n="view.american_option.h1.american_option_lsmc" class="view-title">// AMERICAN OPTION (LSMC)</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.american_option.h2.contract">Contract</h2>
            <div class="inline-form">
                <label><span data-i18n="view.american_option.label.kind">Kind</span>
                    <select id="ao-kind" data-tip="view.american_option.tip.kind">
                        <option data-i18n="view.american_option.opt.call" value="call" ${state.params.kind === 'call' ? 'selected' : ''}>Call</option>
                        <option data-i18n="view.american_option.opt.put" value="put" ${state.params.kind === 'put' ? 'selected' : ''}>Put</option>
                    </select></label>
                <label><span data-i18n="view.american_option.label.spot">Spot</span>
                    <input id="ao-spot" type="number" step="any" min="0" value="${state.params.spot}" data-tip="view.american_option.tip.spot"></label>
                <label><span data-i18n="view.american_option.label.strike">Strike</span>
                    <input id="ao-strike" type="number" step="any" min="0" value="${state.params.strike}" data-tip="view.american_option.tip.strike"></label>
                <label><span data-i18n="view.american_option.label.t_years">T (years)</span>
                    <input id="ao-t" type="number" step="any" min="0" value="${state.params.t_years}" data-tip="view.american_option.tip.t"></label>
                <label><span data-i18n="view.american_option.label.rate">Rate r</span>
                    <input id="ao-rate" type="number" step="any" value="${state.params.rate}" data-tip="view.american_option.tip.rate"></label>
                <label><span data-i18n="view.american_option.label.dividend">Dividend q</span>
                    <input id="ao-div" type="number" step="any" min="0" value="${state.params.dividend}" data-tip="view.american_option.tip.div"></label>
                <label><span data-i18n="view.american_option.label.sigma">σ</span>
                    <input id="ao-sigma" type="number" step="any" min="0" value="${state.params.sigma}" data-tip="view.american_option.tip.sigma"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.american_option.h2.monte_carlo_settings">Monte Carlo settings</h2>
            <div class="inline-form">
                <label><span data-i18n="view.american_option.label.steps">Steps</span>
                    <input id="ao-steps" type="number" step="1" min="2" value="${state.params.steps}" data-tip="view.american_option.tip.steps"></label>
                <label><span data-i18n="view.american_option.label.paths">Paths</span>
                    <input id="ao-paths" type="number" step="100" min="10" value="${state.params.paths}" data-tip="view.american_option.tip.paths"></label>
                <label><span data-i18n="view.american_option.label.seed">Seed (0 = auto)</span>
                    <input id="ao-seed" type="number" step="1" min="0" value="${state.params.seed}" data-tip="view.american_option.tip.seed"></label>
                <button data-i18n="view.american_option.btn.price" data-tip="view.american_option.tip.price" data-shortcut="american_option_price" id="ao-run" class="primary" type="button">Price</button>
            </div>
            <p data-i18n="view.american_option.hint.longstaff_schwartz_2001_regression_monte_carlo_lar" class="muted">
                Longstaff-Schwartz 2001 regression-Monte-Carlo. Larger paths shrink the standard
                error as 1/√N. Default 5,000 paths gives ~1% SE for at-the-money options.
            </p>
        </div>

        <div id="ao-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.american_option.h2.sensitivity_to_spot">Sensitivity to spot</h2>
            <div id="ao-chart" style="width:100%;height:340px"></div>
            <p class="muted" id="ao-chart-caption" data-i18n-html="view.american_option.chart_caption">
                Click <em>Price</em> first. The chart plots the European Black-Scholes price
                across a spot range (instant, no MC) and overlays a single MC point at your
                input spot to show where the LSMC estimate sits relative to the parametric
                European curve.
            </p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.american_option.h2.sensitivity_to_sigma">Sensitivity to volatility (σ)</h2>
            <div id="ao-vol-chart" style="width:100%;height:240px"></div>
            <p class="muted" data-i18n="view.american_option.vol_chart_caption">European BS price across σ ∈ [0.05, 1.0] at fixed input spot, plus a single LSMC point at the input σ.</p>
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
    if (err) { showErr(err); showToast(err, { level: 'warning' }); return; }

    let res;
    try {
        res = await api.anlyAmericanOptionLsmc(buildLsmcBody(state.params));
        if (!res) throw new Error(t('view.american_option.error.null_result'));
    } catch (e) {
        const m = t("common.error.api", { msg: e.message || e });
        showErr(m); showToast(m, { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    state.lastResult = res;

    renderSummary(res);
    renderChart(res);
    renderVolChart(res);
    showToast(t('view.american_option.toast.done', {
        kind: state.params.kind,
        price: fmtMoney(res.price),
        se: res.standard_error != null ? res.standard_error.toFixed(4) : '—',
    }), { level: 'success' });
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
    cards.push(card(t('view.american_option.card.american_lsmc'), fmtMoney(res.price), '', `
        <div class="vc-row"><span class="muted">95% CI</span> <strong>±${fmtMoney(halfWidth)}</strong></div>
        <div class="vc-row"><span class="muted">SE</span> <strong>${fmtMoney(res.standard_error, 5)}</strong></div>
        <div class="vc-row"><span class="muted" data-i18n="view.american_option.row.paths">Paths</span> <strong>${res.paths_run}</strong></div>
    `));
    cards.push(card(t('view.american_option.card.european_bs_reference'), fmtMoney(european), '', `
        <div class="vc-row"><span class="muted" data-i18n="view.american_option.row.computed">Computed</span> <strong data-i18n="view.american_option.row.closed_form">closed-form</strong></div>
        <div class="vc-row"><span class="muted" data-i18n="view.american_option.row.no_mc_error">No MC error</span> <strong>—</strong></div>
    `));
    cards.push(card(t('view.american_option.card.early_exercise_premium'),
        eep == null ? '—' : fmtMoney(eep),
        eep != null && eep > 0 ? 'pos' : (eep != null && eep < 0 ? 'neg' : ''),
        `<div class="vc-row"><span class="muted" data-i18n="view.american_option.row.pct_of_american">% of American price</span>
             <strong>${eepPct == null ? '—' : eepPct.toFixed(2) + '%'}</strong></div>
         <div class="vc-row"><span class="muted" data-i18n="view.american_option.row.sign">Sign</span>
             <strong>${eep == null ? '—' : esc(t(eep > 0 ? 'view.american_option.sign.ee_valuable' : 'view.american_option.sign.mc_noise'))}</strong></div>`));
    cards.push(card(t('view.american_option.card.intrinsic_value_parity_floor'), fmtMoney(intrinsic), '', `
        <div class="vc-row"><span class="muted">${p.kind === 'call' ? 'max(S - K, 0)' : 'max(K - S, 0)'}</span>
            <strong>${fmtMoney(intrinsic)}</strong></div>
        <div class="vc-row"><span class="muted" data-i18n="view.american_option.row.lsmc_ge_intrinsic">LSMC ≥ intrinsic</span>
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
        el.textContent = t('common.error.uplot_not_loaded');
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
            { label: t('chart.series.spot') },
            { label: t('chart.series.european_bs'), stroke: '#00e5ff', width: 2,
              points: { show: false } },
            { label: t('chart.series.american_lsmc'), stroke: '#ff9f1a', width: 0,
              points: { show: true, size: 10, stroke: '#ff9f1a', fill: '#ff9f1a' } },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, ys, mcPoint], el);
}

function renderVolChart(res) {
    const el = document.getElementById('ao-vol-chart');
    if (!window.uPlot) {
        el.textContent = t('common.error.uplot_not_loaded');
        return;
    }
    el.innerHTML = '';
    const p = state.params;
    const lo = 0.05, hi = 1.0;
    const n = 96;
    const xs = new Array(n);
    const ys = new Array(n);
    for (let i = 0; i < n; i++) {
        const s = lo + (hi - lo) * i / (n - 1);
        xs[i] = s;
        ys[i] = blackScholesEuropean(p.kind, p.spot, p.strike, p.t_years, p.rate, p.dividend, s);
    }
    const mcPoint = xs.map(s => Math.abs(s - p.sigma) < (hi - lo) / (2 * (n - 1)) ? res.price : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 220,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.sigma') },
            { label: t('chart.series.european_bs'), stroke: '#7af0a8', width: 2,
              points: { show: false } },
            { label: t('chart.series.american_lsmc'), stroke: '#ff9f1a', width: 0,
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
