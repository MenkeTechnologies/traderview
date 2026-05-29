// Kalman Dynamic Beta view — time-varying hedge ratio between an
// asset and a benchmark.
//
// Use cases:
//   * Pair trading: continuously estimate β so your hedge stays
//     dollar-neutral as the relationship drifts.
//   * Risk management: detect when a high-β stock decoupled from the
//     market (β trace falls toward 0 = idiosyncratic move).
//   * Factor exposure: get β to SPY for a portfolio's daily return
//     stream as a 1-D PnL hedge ratio.
//
// Visualization:
//   * Top chart: asset and bench returns overlaid (raw context).
//   * Bottom chart: β trace from the Kalman filter.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseSeries, validateInputs, buildBody,
    summarizeBetaTrace, fmtBeta,
} from '../_kalman_beta_inputs.js';

import { t } from '../i18n.js';
const DEFAULT_ASSET = `# Asset returns (the "y"). Demo: synthetic high-beta stock whose
# β to bench drifts from ~1.5 to ~2.5 over 200 days.
${synthAsset(200).join('\n')}
`;
const DEFAULT_BENCH = `# Bench returns (the "x"). Demo: synthetic broad-market index.
${synthBench(200).join('\n')}
`;

function rng(seedTag) {
    let s = seedTag;
    return () => {
        s = (s * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(s >> 11n) / 2 ** 53;
    };
}

function synthBench(n) {
    const r = rng(0xBEEF00000n);
    const normal = () => {
        const u1 = Math.max(r(), 1e-10);
        const u2 = r();
        return Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
    };
    const out = [];
    for (let i = 0; i < n; i++) out.push((normal() * 0.01).toFixed(5));
    return out;
}

function synthAsset(n) {
    const r = rng(0xBEEF00000n);
    const r2 = rng(0xCAFE12345n);
    const normal = (gen) => {
        const u1 = Math.max(gen(), 1e-10);
        const u2 = gen();
        return Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
    };
    const out = [];
    for (let i = 0; i < n; i++) {
        const benchR = normal(r) * 0.01;     // same seed → same bench
        const beta = 1.5 + i / 200;          // drift 1.5 → 2.5
        const idio = normal(r2) * 0.005;     // idiosyncratic noise
        out.push((beta * benchR + idio).toFixed(5));
    }
    return out;
}

let state = {
    assetText: DEFAULT_ASSET,
    benchText: DEFAULT_BENCH,
    params: { process_noise_q: 1e-4, obs_noise_r: 1e-4, beta0: 1.0, p0: 1.0 },
};

export async function renderKalmanBeta(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.kalman_beta.h1.kalman_dynamic" class="view-title">// KALMAN DYNAMIC β</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.kalman_beta.h2.returns">Returns</h2>
            <div class="op-inputs-grid">
                <div>
                    <h3 data-i18n="view.kalman_beta.h3.asset_returns_y">Asset returns (y)</h3>
                    <textarea id="kb-asset" rows="10"
                        style="width:100%;font-family:monospace;font-size:13px">${esc(state.assetText)}</textarea>
                </div>
                <div>
                    <h3 data-i18n="view.kalman_beta.h3.bench_returns_x">Bench returns (x)</h3>
                    <textarea id="kb-bench" rows="10"
                        style="width:100%;font-family:monospace;font-size:13px">${esc(state.benchText)}</textarea>
                </div>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.kalman_beta.h2.kalman_hyperparameters">Kalman hyperparameters</h2>
            <div class="inline-form">
                <label><span data-i18n="view.kalman_beta.label.q">Q (process noise)</span>
                    <input id="kb-q"    type="number" step="any" min="0" value="${state.params.process_noise_q}"></label>
                <label><span data-i18n="view.kalman_beta.label.r">R (observation noise)</span>
                    <input id="kb-r"    type="number" step="any" min="1e-12" value="${state.params.obs_noise_r}"></label>
                <label><span data-i18n="view.kalman_beta.label.beta0">β₀ (prior mean)</span>
                    <input id="kb-b0"   type="number" step="any" value="${state.params.beta0}"></label>
                <label><span data-i18n="view.kalman_beta.label.p0">P₀ (prior variance)</span>
                    <input id="kb-p0"   type="number" step="any" min="1e-12" value="${state.params.p0}"></label>
                <button data-i18n="view.kalman_beta.btn.run" id="kb-run" class="primary" type="button">Run</button>
            </div>
            <p data-i18n="view.kalman_beta.hint.lower_q_smoother_assumes_slow_drift_higher_q_adapt" class="muted">
                Lower Q → smoother β (assumes slow drift). Higher Q → β adapts faster.
                Lower R → trust observations more (β tracks noise). Try Q=1e-4 + R=1e-4
                for daily equity pairs as a starting point.
            </p>
        </div>

        <div id="kb-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="kb-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.kalman_beta.h2.return_series_raw_for_context">Return series (raw, for context)</h2>
            <div id="kb-returns-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.kalman_beta.h2.time_varying">Time-varying β</h2>
            <div id="kb-beta-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.kalman_beta.hint.cyan_per_bar_estimate_from_the_kalman_filter_orang" class="muted">
                Cyan: per-bar β estimate from the Kalman filter. Orange dashed: β = 1
                reference (would mean perfect 1:1 hedge ratio).
            </p>
        </div>

        <div id="kb-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('kb-run').addEventListener('click', () => {
        readInputs();
        void run(mount, tok);
    });
    void fmt;
}

function readInputs() {
    const get = id => document.getElementById(id).value;
    state.assetText = get('kb-asset');
    state.benchText = get('kb-bench');
    state.params = {
        process_noise_q: Number(get('kb-q')),
        obs_noise_r:     Number(get('kb-r')),
        beta0:           Number(get('kb-b0')),
        p0:              Number(get('kb-p0')),
    };
}

async function run(mount, tok) {
    hideErrs();
    const assetParsed = parseSeries(state.assetText);
    const benchParsed = parseSeries(state.benchText);
    const errors = assetParsed.errors.concat(benchParsed.errors);
    if (errors.length) renderParseErrors(errors);

    const err = validateInputs(assetParsed.value, benchParsed.value, state.params);
    if (err) { showErr(err); return; }

    let betas;
    try {
        betas = await api.anlyKalmanDynamicBeta(
            buildBody(assetParsed.value, benchParsed.value, state.params),
        );
        if (!Array.isArray(betas)) throw new Error(t('view.kalman_beta.error.non_array'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(betas);
    renderReturnsChart(assetParsed.value, benchParsed.value);
    renderBetaChart(betas);
}

function renderSummary(betas) {
    const s = summarizeBetaTrace(betas);
    if (!s) {
        document.getElementById('kb-summary').innerHTML =
            `<div class="boot">${esc(t('view.kalman_beta.empty.no_beta'))}</div>`;
        return;
    }
    const driftCls = s.drift > 0 ? 'pos' : (s.drift < 0 ? 'neg' : '');
    const cards = [
        card(t('view.kalman_beta.card.latest'), fmtBeta(s.latest)),
        card(t('view.kalman_beta.card.mean'),   fmtBeta(s.mean)),
        card(t('view.kalman_beta.card.range'),  `${fmtBeta(s.min)} – ${fmtBeta(s.max)}`),
        card(t('view.kalman_beta.card.stdev_of_trace'), fmtBeta(s.stdev)),
        card(t('view.kalman_beta.card.drift_latest_first'), fmtBeta(s.drift), driftCls,
            `<div class="vc-row"><span class="muted" data-i18n="view.kalman_beta.row.first_latest">first → latest</span>
                <strong>${fmtBeta(s.first)} → ${fmtBeta(s.latest)}</strong></div>`),
        card(t('view.kalman_beta.card.finite_samples'), String(s.count)),
    ];
    document.getElementById('kb-summary').innerHTML = cards.join('');
}

function card(label, value, cls = '', body = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
        ${body ? `<div class="value kb-summary-value">${body}</div>` : ''}
    </div>`;
}

function renderReturnsChart(asset, bench) {
    const el = document.getElementById('kb-returns-chart');
    if (!window.uPlot) { el.textContent = t('common.error.uplot_not_loaded'); return; }
    el.innerHTML = '';
    const xs = Array.from({ length: asset.length }, (_, i) => i);
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 240,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bar' },
            { label: 'asset', stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: 'bench', stroke: '#ff9f1a', width: 1.5, points: { show: false } },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, asset, bench], el);
}

function renderBetaChart(betas) {
    const el = document.getElementById('kb-beta-chart');
    if (!window.uPlot) { el.textContent = t('common.error.uplot_not_loaded'); return; }
    el.innerHTML = '';
    const xs = Array.from({ length: betas.length }, (_, i) => i);
    const ys = betas.map(b => Number.isFinite(b) ? b : null);
    const oneRef = xs.map(() => 1.0);
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bar' },
            { label: 'β (Kalman)', stroke: '#00e5ff', width: 2,
              fill: 'rgba(0,229,255,0.08)', points: { show: false } },
            { label: 'β = 1 reference', stroke: '#ff9f1a', width: 1,
              dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, ys, oneRef], el);
}

function renderParseErrors(errors) {
    const el = document.getElementById('kb-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">… (+${errors.length - 20} more)</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('kb-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('kb-parse-errors').style.display = 'none';
    document.getElementById('kb-err').style.display = 'none';
}
