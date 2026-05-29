// Deflated Sharpe Ratio (DSR) view — Bailey & López de Prado 2014.
//
// Why this matters: if you backtest 100 strategies and pick the best
// observed Sharpe, that number is biased upward by the max-of-N noise
// floor √(2·ln(N_trials)). DSR computes the probability that your TRUE
// Sharpe exceeds that noise floor — the only honest answer to "is this
// strategy actually any good or did I just data-mine luck?"
//
// Inputs:
//   observed_sharpe — sample SR (annualized or whatever convention)
//   n_observations  — bars / months / days in the backtest
//   skewness        — sample skew of returns
//   kurtosis        — sample kurtosis (Pearson convention; normal = 3)
//   n_trials        — number of strategies you tried
//
// Outputs: deflated threshold SR, sample-SR variance, z-score, PSR.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    validateInputs, buildBody,
    confidenceTier, trialsSweep,
    fmtSR, fmtProb, fmtZ,
} from '../_deflated_sharpe_inputs.js';

import { t } from '../i18n.js';
const DEFAULTS = {
    observed_sharpe: 1.5,
    n_observations: 252,
    skewness: -0.3,
    kurtosis: 4.5,
    n_trials: 20,
};

let state = { params: { ...DEFAULTS } };

export async function renderDeflatedSharpe(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.deflated_sharpe.h1.deflated_sharpe_ratio" class="view-title">// DEFLATED SHARPE RATIO</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.deflated_sharpe.h2.backtest_sample">Backtest sample</h2>
            <div class="inline-form">
                <label><span data-i18n="view.deflated_sharpe.label.observed_sr">Observed SR</span>
                    <input id="ds-sr" type="number" step="any" value="${state.params.observed_sharpe}"></label>
                <label><span data-i18n="view.deflated_sharpe.label.n_obs">n observations</span>
                    <input id="ds-n" type="number" step="1" min="4" value="${state.params.n_observations}"></label>
                <label><span data-i18n="view.deflated_sharpe.label.skewness">Skewness</span>
                    <input id="ds-skew" type="number" step="any" value="${state.params.skewness}"></label>
                <label><span data-i18n="view.deflated_sharpe.label.kurtosis">Kurtosis (normal = 3)</span>
                    <input id="ds-kurt" type="number" step="any" value="${state.params.kurtosis}"></label>
                <label><span data-i18n="view.deflated_sharpe.label.trials">Trials run</span>
                    <input id="ds-trials" type="number" step="1" min="1" value="${state.params.n_trials}"></label>
                <button data-i18n="view.deflated_sharpe.btn.deflate" id="ds-run" class="primary" type="button">Deflate</button>
                <button data-i18n="view.deflated_sharpe.btn.trials_sweep" id="ds-sweep" class="secondary" type="button">+ Trials sweep</button>
            </div>
            <p class="muted">
                The deflated threshold SR<sub>★</sub> ≈ √(2 ln N) is the noise floor
                you have to clear when you've backtested N strategies. PSR = P(true SR
                &gt; SR<sub>★</sub>). Below 95% PSR, your edge is statistically indistinguishable
                from luck given how many tries you took.
            </p>
        </div>

        <div id="ds-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.deflated_sharpe.h2.sr_comparison">SR comparison</h2>
            <div id="ds-srbar"></div>
            <p data-i18n="view.deflated_sharpe.hint.cyan_observed_sr_magenta_deflated_threshold_the_no" class="muted">Cyan = observed SR. Magenta = deflated threshold (the noise floor).
                If cyan doesn't clearly exceed magenta you're inside the multiple-testing band.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.deflated_sharpe.h2.trials_sensitivity">Trials sensitivity</h2>
            <div id="ds-sweep-chart" style="height:260px"></div>
            <p data-i18n="view.deflated_sharpe.hint.psr_vs_n_trials_tells_you_how_fragile_your_conclus" class="muted">PSR vs N_trials. Tells you how fragile your conclusion is —
                if PSR collapses past N = 100 you've overfit even by Bailey-LdP's
                generous benchmark.</p>
        </div>

        <div id="ds-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('ds-run').addEventListener('click', () => {
        readInputs(); void compute(tok);
    });
    document.getElementById('ds-sweep').addEventListener('click', () => {
        readInputs(); void computeSweep(tok);
    });
    readInputs(); void compute(tok);
}

function readInputs() {
    const get = id => document.getElementById(id).value;
    state.params = {
        observed_sharpe: Number(get('ds-sr')),
        n_observations:  parseInt(get('ds-n'), 10),
        skewness:        Number(get('ds-skew')),
        kurtosis:        Number(get('ds-kurt')),
        n_trials:        parseInt(get('ds-trials'), 10),
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.params);
    if (err) { showErr(err); return; }
    let res;
    try {
        res = await api.anlyDeflatedSharpe(buildBody(state.params));
        if (!res) throw new Error(t('view.deflated_sharpe.error.null'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res);
    renderSrBar(res);
}

async function computeSweep(tok) {
    hideErr();
    const err = validateInputs(state.params);
    if (err) { showErr(err); return; }
    const trials = trialsSweep(state.params.n_trials);
    let results;
    try {
        results = await Promise.all(
            trials.map(n => api.anlyDeflatedSharpe(buildBody({ ...state.params, n_trials: n }))),
        );
    } catch (e) {
        showErr(t('view.deflated_sharpe.error.sweep', { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    const points = results
        .map((r, i) => r ? { n: trials[i], prob: r.probability_true_sr_above_threshold, sr_star: r.deflated_threshold_sharpe } : null)
        .filter(Boolean);
    renderSweep(points, state.params.n_trials);
}

function renderSummary(r) {
    const tier = confidenceTier(r.probability_true_sr_above_threshold);
    const headroom = r.observed_sharpe - r.deflated_threshold_sharpe;
    document.getElementById('ds-summary').innerHTML = [
        card(t('view.deflated_sharpe.card.observed_sr'),     fmtSR(r.observed_sharpe)),
        card(t('view.deflated_sharpe.card.deflated_sr'),    fmtSR(r.deflated_threshold_sharpe)),
        card(t('view.deflated_sharpe.card.headroom'),        fmtSR(headroom), headroom > 0 ? 'pos' : 'neg'),
        card(t('view.deflated_sharpe.card.sr_variance'),     fmtSR(r.sharpe_variance)),
        card(t('view.deflated_sharpe.card.z_score'),         fmtZ(r.z_score),  r.z_score >= 1.96 ? 'pos' : 'neg'),
        card(t('view.deflated_sharpe.card.psr_p_sr'),   fmtProb(r.probability_true_sr_above_threshold), tier.cls),
        card(t('view.deflated_sharpe.card.confidence'),      tier.label, tier.cls),
        card(t('view.deflated_sharpe.card.n_trials'),      `${state.params.n_observations} / ${state.params.n_trials}`),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderSrBar(r) {
    const wrap = document.getElementById('ds-srbar');
    const maxAbs = Math.max(Math.abs(r.observed_sharpe), Math.abs(r.deflated_threshold_sharpe), 0.1);
    const obsPct = (r.observed_sharpe / maxAbs * 100).toFixed(2);
    const starPct = (r.deflated_threshold_sharpe / maxAbs * 100).toFixed(2);
    wrap.innerHTML = `
        <div class="is-bar-row">
            <div class="is-bar-label" data-i18n="view.deflated_sharpe.bar.observed">Observed</div>
            <div class="is-bar-track">
                <div class="is-bar-midline"></div>
                <div class="is-bar-fill ds-fill-obs ${r.observed_sharpe >= 0 ? 'is-fill-pos' : 'is-fill-neg'}"
                     data-bar-pct="${Math.abs(obsPct) / 2}"></div>
            </div>
            <div class="is-bar-value">${esc(fmtSR(r.observed_sharpe))}</div>
        </div>
        <div class="is-bar-row">
            <div class="is-bar-label" data-i18n="view.deflated_sharpe.bar.deflated">Deflated ★</div>
            <div class="is-bar-track">
                <div class="is-bar-midline"></div>
                <div class="is-bar-fill ds-fill-star ${r.deflated_threshold_sharpe >= 0 ? 'is-fill-pos' : 'is-fill-neg'}"
                     data-bar-pct="${Math.abs(starPct) / 2}"></div>
            </div>
            <div class="is-bar-value">${esc(fmtSR(r.deflated_threshold_sharpe))}</div>
        </div>
    `;
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.is-bar-fill').forEach(el => {
            const pct = Number(el.dataset.barPct);
            if (Number.isFinite(pct)) el.style.width = pct + '%';
        });
    });
}

function renderSweep(points, currentN) {
    if (!window.uPlot || points.length === 0) return;
    const el = document.getElementById('ds-sweep-chart');
    const xs = points.map(p => p.n);
    const ys = points.map(p => p.prob);
    const youYs = xs.map((n, i) => n === currentN ? ys[i] : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 260,
        scales: { x: { distr: 3, log: 10 }, y: { range: [0, 1] } },
        series: [
            { label: 'N_trials (log)' },
            { label: 'PSR', stroke: '#a06bff', width: 1.5,
              fill: '#a06bff1A', points: { show: true, size: 5 } },
            { label: 'you are here', stroke: '#fff', width: 0,
              points: { show: true, size: 10, stroke: '#fff', fill: 'transparent' } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 40 }],
        legend: { show: true },
    }, [xs, ys, youYs], el);
}

function showErr(msg) {
    const el = document.getElementById('ds-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ds-err').style.display = 'none'; }
