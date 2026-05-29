// Hurst Exponent view — fit H from a return series via rescaled-range
// (R/S) analysis across multiple chunk sizes. Tells the trader whether
// the series exhibits long-memory (H > 0.5, trending), no-memory
// (H ≈ 0.5, random walk), or anti-persistence (H < 0.5, mean-reverting).
//
// Chart: log(n) vs log(R/S) scatter from the backend. A linear fit of
// slope H is the underlying regression — the closer R² is to 1, the
// more trustworthy the H estimate.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseReturns, parseChunkSizes, validateInputs, buildBody,
    regimeLabelKey, regimeStrengthKey, regimeCssClass,
} from '../_hurst_inputs.js';

import { t } from '../i18n.js';
const DEFAULT_RETURNS = `# Paste a return series. One value per token.
# Demo: 500 random-walk returns (should give H ≈ 0.5).
${synthRandomWalk(500).join('\n')}
`;

function synthRandomWalk(n) {
    let s = 0xBEEFCAFEn;
    const rand = () => {
        s = (s * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(s >> 11n) / 2 ** 53;
    };
    const normal = () => {
        const u1 = Math.max(rand(), 1e-10);
        const u2 = rand();
        return Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
    };
    const out = [];
    for (let i = 0; i < n; i++) out.push((normal() * 0.01).toFixed(5));
    return out;
}

let state = {
    returnsText: DEFAULT_RETURNS,
    chunkText: '10 20 50 100 250',
};

export async function renderHurst(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.hurst.h1.hurst_exponent" class="view-title">// HURST EXPONENT</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.hurst.h2.inputs">Inputs</h2>
            <div class="op-inputs-grid">
                <div>
                    <h3 data-i18n="view.hurst.h3.return_series">Return series</h3>
                    <textarea id="hu-returns" rows="11"
                        style="width:100%;font-family:monospace;font-size:13px">${esc(state.returnsText)}</textarea>
                </div>
                <div>
                    <h3 data-i18n="view.hurst.h3.chunk_sizes_r_s_regression_points">Chunk sizes (R/S regression points)</h3>
                    <textarea id="hu-chunks" rows="11"
                        style="width:100%;font-family:monospace;font-size:13px">${esc(state.chunkText)}</textarea>
                </div>
            </div>
            <button data-i18n="view.hurst.btn.estimate" id="hu-run" class="primary" type="button" style="margin-top:8px">Estimate</button>
            <p data-i18n="view.hurst.hint.the_r_s_log_log_regression_slope_is_the_hurst_expo" class="muted">
                The R/S log-log regression slope is the Hurst exponent. H &lt; 0.5 =
                anti-persistent (mean-reverting), H ≈ 0.5 = random walk, H &gt; 0.5 =
                persistent (long-memory / trending). R² flags fit quality — low R² means
                the series isn't well-described by a single Hurst exponent.
            </p>
        </div>

        <div id="hu-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="hu-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.hurst.h2.r_s_regression_log_log">R/S regression (log-log)</h2>
            <div id="hu-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.hurst.hint.cyan_log_n_vs_log_r_s_scatter_from_the_backend_ora" class="muted">
                Cyan: log(n) vs log(R/S) scatter from the backend. Orange dashed: linear-
                fit line (slope = H). A clean straight line implies a meaningful Hurst
                estimate; scatter / curvature implies the series mixes regimes.
            </p>
        </div>

        <div id="hu-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('hu-run').addEventListener('click', () => {
        state.returnsText = document.getElementById('hu-returns').value;
        state.chunkText = document.getElementById('hu-chunks').value;
        void estimate(mount, tok);
    });
    void fmt;
}

async function estimate(mount, tok) {
    hideErrs();
    const retParsed = parseReturns(state.returnsText);
    const chunkParsed = parseChunkSizes(state.chunkText);
    const errors = retParsed.errors.concat(chunkParsed.errors);
    if (errors.length) renderParseErrors(errors);

    const err = validateInputs(retParsed.value, chunkParsed.value);
    if (err) { showErr(err); return; }

    let res;
    try {
        res = await api.anlyHurstExponent(buildBody(retParsed.value, chunkParsed.value));
        if (!res) throw new Error(t('view.hurst.error.null_result'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(res);
    renderChart(res);
}

function renderSummary(res) {
    const cls = regimeCssClass(res.hurst);
    const label = t(regimeLabelKey(res.hurst));
    const strength = t(regimeStrengthKey(res.hurst));
    const fitKey = res.r_squared >= 0.95 ? 'view.hurst.fit.clean'
                 : (res.r_squared < 0.80 ? 'view.hurst.fit.noisy' : 'view.hurst.fit.acceptable');
    document.getElementById('hu-summary').innerHTML = [
        card(t('view.hurst.card.hurst_h'), res.hurst.toFixed(4), cls,
            `<div class="vc-row"><span class="muted" data-i18n="view.hurst.row.distance_from_half">distance from 0.5</span>
                <strong>${Math.abs(res.hurst - 0.5).toFixed(3)}</strong></div>`),
        card(t('view.hurst.card.regime'), t('view.hurst.label.regime_strength', { label, strength }), cls),
        card(t('view.hurst.card.r_of_log_log_fit'), res.r_squared.toFixed(4),
            res.r_squared >= 0.95 ? 'pos' : (res.r_squared < 0.80 ? 'neg' : ''),
            `<div class="vc-row"><span class="muted">${t(fitKey)}</span><strong></strong></div>`),
        card(t('view.hurst.card.regression_points'), String(res.log_n.length)),
    ].join('');
}

function card(label, value, cls = '', body = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
        ${body ? `<div class="value hu-summary-value">${body}</div>` : ''}
    </div>`;
}

function renderChart(res) {
    const el = document.getElementById('hu-chart');
    if (!window.uPlot) { el.textContent = t('common.error.uplot_not_loaded'); return; }
    el.innerHTML = '';

    const xs = res.log_n;
    const ys = res.log_rs;
    if (!Array.isArray(xs) || xs.length < 2) {
        el.innerHTML = `<div class="boot">${esc(t('view.hurst.empty.not_enough_points'))}</div>`;
        return;
    }

    // Reconstruct the fit line locally for the overlay: slope = H,
    // intercept = mean(log_rs) - H * mean(log_n).
    const n = xs.length;
    const meanX = xs.reduce((a, b) => a + b, 0) / n;
    const meanY = ys.reduce((a, b) => a + b, 0) / n;
    const intercept = meanY - res.hurst * meanX;
    const fitYs = xs.map(x => res.hurst * x + intercept);

    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: 'log(n)' },
            { label: 'log(R/S)', stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, stroke: '#00e5ff', fill: '#00e5ff' } },
            { label: `fit (slope = ${res.hurst.toFixed(4)})`, stroke: '#ff9f1a',
              width: 2, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, ys, fitYs], el);
}

function renderParseErrors(errors) {
    const el = document.getElementById('hu-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">${esc(t("common.plus_n_more", { n: errors.length - 20 }))}</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('hu-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('hu-parse-errors').style.display = 'none';
    document.getElementById('hu-err').style.display = 'none';
}
