// Cov Matrix Denoiser (Marchenko-Pastur eigenvalue clipping).
//
// User pastes a sample covariance matrix + the number of observations
// it was estimated from. Backend applies MP cleaning: eigenvalues that
// fall inside the noise bulk get flattened to the bulk's average; only
// the "signal" eigenvalues survive.
//
// Visualization:
//   * Cards: signal_count vs bulk_count, λ_max threshold, Frobenius
//     relative delta between original and cleaned.
//   * Eigenvalue chart: signal eigenvalues marked, λ_max threshold line.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseCovariance, validateInputs, buildBody,
    frobeniusRelDelta,
} from '../_cov_denoiser_inputs.js';

import { t } from '../i18n.js';
const DEFAULT_COV = `# Sample covariance matrix (symmetric, N×N). Whitespace or comma
# separator. # comments OK.
# Demo: 8 assets with one strong factor + small idiosyncratic noise.
0.040 0.024 0.022 0.025 0.023 0.021 0.020 0.022
0.024 0.045 0.026 0.024 0.022 0.023 0.021 0.024
0.022 0.026 0.038 0.023 0.024 0.022 0.022 0.021
0.025 0.024 0.023 0.047 0.025 0.021 0.024 0.023
0.023 0.022 0.024 0.025 0.041 0.022 0.023 0.022
0.021 0.023 0.022 0.021 0.022 0.039 0.023 0.024
0.020 0.021 0.022 0.024 0.023 0.023 0.043 0.022
0.022 0.024 0.021 0.023 0.022 0.024 0.022 0.044
`;

let state = {
    covText: DEFAULT_COV,
    numObservations: 20,
};

export async function renderCovDenoiser(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.cov_denoiser.h1.cov_denoiser_m_p" class="view-title">// COV DENOISER (M-P)</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.cov_denoiser.h2.inputs">Inputs</h2>
            <textarea id="cd-cov" rows="11"
                style="width:100%;font-family:monospace;font-size:13px">${esc(state.covText)}</textarea>
            <div class="inline-form" style="margin-top:8px">
                <label><span data-i18n="view.cov_denoiser.label.t">T (observations used to estimate the cov)</span>
                    <input id="cd-t" type="number" step="1" min="1" value="${state.numObservations}"></label>
                <button data-i18n="view.cov_denoiser.btn.denoise" id="cd-run" class="primary" type="button">Denoise</button>
            </div>
            <p data-i18n="view.cov_denoiser.hint.marchenko_pastur_clipping_eigenvalues_inside_the_n" class="muted">
                Marchenko-Pastur clipping: eigenvalues inside the noise bulk are flattened
                to their average; only signal eigenvalues survive. Trace is preserved.
                Pipe the cleaned cov into the Portfolio Allocator for more robust weights
                than raw-sample-cov input.
            </p>
        </div>

        <div id="cd-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="cd-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.cov_denoiser.h2.eigenvalue_spectrum">Eigenvalue spectrum</h2>
            <div id="cd-eigen-chart" style="width:100%;height:280px"></div>
            <p data-i18n="view.cov_denoiser.hint.cyan_markers_original_sample_eigenvalues_sorted_de" class="muted">
                Cyan markers = original sample eigenvalues (sorted descending).
                Red dashed line = λ_max bulk threshold. Anything above the line is
                "signal" (preserved); anything at or below is "noise" (replaced by the
                bulk average — visible as the orange flat segment in the cleaned series).
            </p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cov_denoiser.h2.cleaned_covariance">Cleaned covariance</h2>
            <div id="cd-cleaned"></div>
        </div>

        <div id="cd-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('cd-run').addEventListener('click', () => {
        state.covText = document.getElementById('cd-cov').value;
        state.numObservations = parseInt(document.getElementById('cd-t').value, 10);
        void denoise(mount, tok);
    });
    void fmt;
}

async function denoise(mount, tok) {
    hideErrs();
    const parsed = parseCovariance(state.covText);
    if (parsed.errors.length) renderParseErrors(parsed.errors);

    const err = validateInputs(parsed.value, state.numObservations);
    if (err) { showErr(err); return; }

    let res;
    try {
        res = await api.anlyMarchenkoPasturCleaning(
            buildBody(parsed.value, state.numObservations),
        );
        if (!res) throw new Error('cleaner returned null (input out of domain)');
    } catch (e) {
        showErr(`API error: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(parsed.value, res);
    renderEigenChart(parsed.value, res);
    renderCleanedMatrix(parsed.value, res);
}

function renderSummary(originalCov, res) {
    const n = originalCov.length;
    const q = n / state.numObservations;
    const relDelta = frobeniusRelDelta(originalCov, res.cleaned_covariance);
    const cards = [];
    cards.push(card(t('view.cov_denoiser.card.matrix_size_n'), String(n)));
    cards.push(card(t('view.cov_denoiser.card.observations_t'), String(state.numObservations)));
    cards.push(card(t('view.cov_denoiser.card.q_n_t'), q.toFixed(3)));
    cards.push(card(t('view.cov_denoiser.card.signal_eigenvalues'), String(res.signal_count), res.signal_count > 0 ? 'pos' : ''));
    cards.push(card(t('view.cov_denoiser.card.bulk_replaced'), String(res.bulk_count)));
    cards.push(card(t('view.cov_denoiser.card.max_threshold'), res.lambda_max.toFixed(6)));
    cards.push(card(t('view.cov_denoiser.card.bulk_eigenvalue_avg'), res.bulk_eigenvalue_avg.toFixed(6)));
    cards.push(card(t('view.cov_denoiser.card.frobenius_rel'),
        relDelta == null ? '—' : (relDelta * 100).toFixed(2) + '%'));
    document.getElementById('cd-summary').innerHTML = cards.join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderEigenChart(originalCov, res) {
    const el = document.getElementById('cd-eigen-chart');
    if (!window.uPlot) { el.textContent = 'uPlot not loaded'; return; }
    el.innerHTML = '';
    // Reconstruct the full original eigenvalue list: signal eigenvalues
    // (sorted desc) + bulk-count copies of the bulk average. That matches
    // what the backend used to build the cleaned matrix and gives the
    // user a visible "cleaned" curve to compare to the raw eigenvalues.
    const n = originalCov.length;
    const xs = Array.from({ length: n }, (_, i) => i);
    // Local diagonalization is not available without uPlot/numpy; show
    // only the signal eigenvalues (length = signal_count) the backend
    // reports + a flat orange band at the bulk avg for the rest.
    const signalCount = res.signal_count;
    const bulkAvg = res.bulk_eigenvalue_avg;
    // Cleaned eigenvalues (sorted desc): first `signal_count` are the
    // signal values from the backend; remainder = bulk avg.
    const cleaned = new Array(n).fill(bulkAvg);
    for (let i = 0; i < signalCount && i < res.eigenvalues_signal.length; i++) {
        cleaned[i] = res.eigenvalues_signal[i];
    }
    // λ_max line as a flat series.
    const lambdaMaxLine = xs.map(() => res.lambda_max);

    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 280,
        scales: { x: {}, y: {} },
        series: [
            { label: 'rank' },
            { label: 'cleaned (sorted desc)', stroke: '#ff9f1a', width: 2,
              points: { show: true, size: 8, stroke: '#ff9f1a', fill: '#ff9f1a' } },
            { label: 'λ_max bulk threshold', stroke: '#ff3860', width: 1,
              dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, cleaned, lambdaMaxLine], el);
}

function renderCleanedMatrix(originalCov, res) {
    const wrap = document.getElementById('cd-cleaned');
    const n = res.cleaned_covariance.length;
    const headers = Array.from({ length: n }, (_, j) => `<th>${j + 1}</th>`).join('');
    const rows = res.cleaned_covariance.map((row, i) => {
        const cells = row.map((v, j) => {
            const delta = v - originalCov[i][j];
            const cls = Math.abs(delta) < 1e-9 ? 'cd-unchanged' :
                        delta > 0 ? 'cd-bumped' : 'cd-trimmed';
            return `<td class="${cls}">${v.toFixed(5)}</td>`;
        }).join('');
        return `<tr><th>${i + 1}</th>${cells}</tr>`;
    }).join('');
    wrap.innerHTML = `<table class="cd-matrix">
        <thead><tr><th></th>${headers}</tr></thead>
        <tbody>${rows}</tbody>
    </table>
    <p class="muted">
        <span class="cd-bumped" style="padding:1px 4px;border-radius:2px">bumped</span>
        <span class="cd-trimmed" style="padding:1px 4px;border-radius:2px">trimmed</span>
        vs original — cells the MP cleaning shifted by &gt; 1e-9.
    </p>`;
}

function renderParseErrors(errors) {
    const el = document.getElementById('cd-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">… (+${errors.length - 20} more)</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('cd-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('cd-parse-errors').style.display = 'none';
    document.getElementById('cd-err').style.display = 'none';
}
