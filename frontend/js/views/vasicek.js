// Vasicek Short-Rate Simulator view.
//
// Backend runs a Monte Carlo on the Vasicek OU process:
//   dr = a · (b − r) · dt + σ · dW
// and returns terminal-distribution stats (mean, stdev, min, max,
// fraction of paths that visited a negative rate at least once).
//
// The view adds derived characterizations the backend doesn't return:
//   * Mean-reversion half-life = ln(2)/a (in years).
//   * Long-run terminal stdev = σ / √(2a) — the asymptotic stationary
//     stdev as t → ∞. Lets the user sanity-check the simulated stdev
//     against the closed-form limit.
//   * Normal-approximation density of the terminal rate (asymptotically
//     exact for Vasicek).

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    buildBody, validateParams,
    halfLifeYears, longRunStdev, horizonYears,
    normalDensityCurve, fmtRatePct, fmtYears,
} from '../_vasicek_inputs.js';

import { t } from '../i18n.js';
// Realistic defaults: start at 5%, mean-revert to 3% with ~1.4yr half-life,
// 1% annualized vol, weekly steps over 10 years.
const DEFAULT_PARAMS = {
    r0: 0.05,
    a: 0.5,
    b: 0.03,
    sigma: 0.01,
    dt: 1 / 52,
    steps: 520,
    paths: 5000,
    seed: 42,
};

let state = { params: { ...DEFAULT_PARAMS } };

export async function renderVasicek(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.vasicek.h1.vasicek_short_rate" class="view-title">// VASICEK SHORT-RATE</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.vasicek.h2.sde_parameters">SDE parameters</h2>
            <div class="inline-form">
                <label>r₀ (initial rate)
                    <input id="va-r0"    type="number" step="any" value="${state.params.r0}"></label>
                <label>a (mean-reversion speed)
                    <input id="va-a"     type="number" step="any" min="0" value="${state.params.a}"></label>
                <label>b (long-run mean)
                    <input id="va-b"     type="number" step="any" value="${state.params.b}"></label>
                <label>σ (vol)
                    <input id="va-sigma" type="number" step="any" min="0" value="${state.params.sigma}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.vasicek.h2.simulation_grid">Simulation grid</h2>
            <div class="inline-form">
                <label><span data-i18n="view.vasicek.label.dt">dt (years)</span>
                    <input id="va-dt"    type="number" step="any" min="0" value="${state.params.dt}"></label>
                <label><span data-i18n="view.vasicek.label.steps">Steps</span>
                    <input id="va-steps" type="number" step="1"   min="1" value="${state.params.steps}"></label>
                <label><span data-i18n="view.vasicek.label.paths">Paths</span>
                    <input id="va-paths" type="number" step="100" min="10" value="${state.params.paths}"></label>
                <label><span data-i18n="view.vasicek.label.seed">Seed (0 = auto)</span>
                    <input id="va-seed"  type="number" step="1"   min="0" value="${state.params.seed}"></label>
                <button data-i18n="view.vasicek.btn.simulate" id="va-run" class="primary" type="button">Simulate</button>
            </div>
            <p data-i18n="view.vasicek.hint.half_life_of_mean_reversion_ln_2_a_default_a_0_5_h" class="muted">
                Half-life of mean reversion = ln(2)/a. Default a=0.5 → half-life ≈ 1.39
                years. Vasicek allows negative rates — the report tells you what fraction
                of paths visited a negative value, which is real risk for European-rate
                modeling post-2014.
            </p>
        </div>

        <div id="va-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.vasicek.h2.terminal_rate_distribution_normal_approximation">Terminal-rate distribution (normal approximation)</h2>
            <div id="va-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.vasicek.hint.density_centered_on_the_simulated_mean_stdev_vasic" class="muted">
                Density centered on the simulated (mean, stdev). Vasicek's terminal rate
                is asymptotically normal, so the curve is exact in the long-run limit and
                a good approximation for any reasonably-long horizon.
            </p>
        </div>

        <div id="va-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('va-run').addEventListener('click', () => {
        readInputs();
        void simulate(mount, tok);
    });
    void fmt;
}

function readInputs() {
    const get = id => document.getElementById(id).value;
    state.params = {
        r0:    Number(get('va-r0')),
        a:     Number(get('va-a')),
        b:     Number(get('va-b')),
        sigma: Number(get('va-sigma')),
        dt:    Number(get('va-dt')),
        steps: parseInt(get('va-steps'), 10),
        paths: parseInt(get('va-paths'), 10),
        seed:  parseInt(get('va-seed'),  10),
    };
}

async function simulate(mount, tok) {
    hideErr();
    const err = validateParams(state.params);
    if (err) { showErr(err); return; }

    let res;
    try {
        res = await api.anlyVasicekShortRateSimulator(buildBody(state.params));
        if (!res) throw new Error(t('view.vasicek.error.null_result'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(res);
    renderChart(res);
}

function renderSummary(res) {
    const p = state.params;
    const hl = halfLifeYears(p.a);
    const lrs = longRunStdev(p.a, p.sigma);
    const horizon = horizonYears(p.steps, p.dt);
    const negPct = res.negative_path_fraction * 100;
    document.getElementById('va-summary').innerHTML = [
        card(t('view.vasicek.card.terminal_mean'), fmtRatePct(res.mean_terminal_rate), '',
            `<div class="vc-row"><span class="muted" data-i18n="view.vasicek.row.long_run_target">long-run target b</span>
                <strong>${fmtRatePct(p.b)}</strong></div>`),
        card(t('view.vasicek.card.terminal_stdev'), fmtRatePct(res.stdev_terminal_rate, 4), '',
            `<div class="vc-row"><span class="muted" data-i18n="view.vasicek.row.long_run_sigma">long-run σ_∞ = σ/√(2a)</span>
                <strong>${fmtRatePct(lrs, 4)}</strong></div>`),
        card(t('view.vasicek.card.terminal_range'), `${fmtRatePct(res.min_terminal_rate)} – ${fmtRatePct(res.max_terminal_rate)}`),
        card(t('view.vasicek.card.mean_reversion_half_life'), fmtYears(hl)),
        card(t('view.vasicek.card.simulation_horizon'), fmtYears(horizon)),
        card(t('view.vasicek.card.paths_that_went_negative'),
            `${negPct.toFixed(1)}% (${Math.round(negPct / 100 * res.paths_run)} of ${res.paths_run})`,
            negPct > 0 ? 'neg' : 'pos'),
    ].join('');
}

function renderChart(res) {
    const el = document.getElementById('va-chart');
    if (!window.uPlot) { el.textContent = t('common.error.uplot_not_loaded'); return; }
    el.innerHTML = '';
    const { xs, ys } = normalDensityCurve(res.mean_terminal_rate, res.stdev_terminal_rate, 161);
    if (xs.length === 0) {
        el.innerHTML = `<div class="boot">${esc(t('view.monte_carlo.empty.stdev_zero'))}</div>`;
        return;
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: 'terminal rate' },
            { label: 'density', stroke: '#00e5ff', width: 2,
              fill: 'rgba(0,229,255,0.10)', points: { show: false } },
        ],
        axes: [
            { stroke: '#aab',
              values: (_, ticks) => ticks.map(t => `${(t * 100).toFixed(2)}%`) },
            { stroke: '#aab' },
        ],
    }, [xs, ys], el);
}

function card(label, value, cls = '', body = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
        ${body ? `<div class="value va-summary-value">${body}</div>` : ''}
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('va-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('va-err').style.display = 'none'; }
