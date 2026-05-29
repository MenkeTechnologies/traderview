// Pattern Discovery view — Matrix Profile motif & discord finder.
//
// Paste a 1-D series (price, returns, or anything else). Pick a window
// length `m`. The backend computes, for each window-of-length-m, the
// z-normalized Euclidean distance to its nearest non-trivial neighbor.
// Low distance → window has a close match elsewhere (a motif).
// High distance → window has no close match (a discord = anomaly).
//
// Visualization:
//   1. Top chart: raw series, with the motif pair highlighted in cyan
//      and the top discord windows highlighted in red.
//   2. Bottom chart: the matrix profile array itself, aligned to the
//      same x-axis so peaks and valleys line up visually.
//   3. Cards: motif pair indices + distance, top discord indices.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseSeries, validateMatrixProfileInputs,
    overlaySeriesForWindows, unpackMotifPair, unpackDiscords,
    indexAxis,
} from '../_matrix_profile_inputs.js';

import { t, applyUiI18n } from '../i18n.js';
const DEFAULT_TEXT = `# Paste a numeric series. One value per token.
# Demo: 200 samples with two embedded copies of a sine "pattern"
# (at positions 30 and 120) plus a single anomaly spike.
${synthDemoSeries().join('\n')}
`;

function synthDemoSeries() {
    const out = [];
    let s = 0x1234567ABn;
    const rand = () => {
        s = (s * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(s >> 11n) / 2 ** 53;
    };
    // Noise floor.
    for (let i = 0; i < 200; i++) out.push((rand() - 0.5) * 0.3);
    // Pattern at index 30..50: half-period sine.
    for (let k = 0; k < 20; k++) {
        out[30 + k] = Math.sin(k * Math.PI / 10) * 2.0 + (rand() - 0.5) * 0.1;
    }
    // Identical pattern at index 120..140.
    for (let k = 0; k < 20; k++) {
        out[120 + k] = Math.sin(k * Math.PI / 10) * 2.0 + (rand() - 0.5) * 0.1;
    }
    // Anomaly spike at index 80.
    out[80] = 8.0;
    return out.map(v => v.toFixed(4));
}

let state = {
    text: DEFAULT_TEXT,
    m: 20,
    top_k: 3,
};

export async function renderPatternDiscovery(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.pattern_discovery.h1.pattern_discovery" class="view-title">// PATTERN DISCOVERY</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.pattern_discovery.h2.inputs">Inputs</h2>
            <div class="inline-form">
                <label><span data-i18n="view.pattern_discovery.label.window_m">Window m</span>
                    <input id="pd-m" type="number" step="1" min="4" value="${state.m}"></label>
                <label><span data-i18n="view.pattern_discovery.label.top_k">Top-K discords</span>
                    <input id="pd-k" type="number" step="1" min="1" max="20" value="${state.top_k}"></label>
                <button data-i18n="view.pattern_discovery.btn.discover" id="pd-run" class="primary" type="button">Discover</button>
            </div>
            <p data-i18n="view.pattern_discovery.hint.m_sets_the_pattern_length_low_matrix_profile_value" class="muted">
                m sets the pattern length. Low matrix-profile values = repeated patterns (motifs);
                high values = anomalies (discords). Series must have ≥ 2·m samples.
            </p>
            <h3 data-i18n="view.pattern_discovery.h3.series">Series</h3>
            <textarea id="pd-text" rows="8"
                style="width:100%;font-family:monospace;font-size:13px">${esc(state.text)}</textarea>
        </div>

        <div id="pd-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="pd-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.pattern_discovery.h2.series_motif_cyan_top_discords_red">Series + motif (cyan) + top discords (red)</h2>
            <div id="pd-chart-series" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.pattern_discovery.h2.matrix_profile">Matrix profile</h2>
            <div id="pd-chart-profile" style="width:100%;height:200px"></div>
            <p data-i18n="view.pattern_discovery.hint.each_x_is_a_window_start_low_best_match_found_else" class="muted">Each x is a window start. Low = best match found elsewhere; high = no good match (anomaly).</p>
        </div>

        <div id="pd-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    wireForm(mount, tok);
    void fmt;
}

function wireForm(mount, tok) {
    document.getElementById('pd-run').addEventListener('click', () => {
        state.m = parseInt(document.getElementById('pd-m').value, 10);
        state.top_k = parseInt(document.getElementById('pd-k').value, 10);
        state.text = document.getElementById('pd-text').value;
        void discover(mount, tok);
    });
}

async function discover(mount, tok) {
    hideErrs();
    const parsed = parseSeries(state.text);
    if (parsed.errors.length) renderParseErrors(parsed.errors);

    const err = validateMatrixProfileInputs(parsed.value, state.m);
    if (err) { showErr(err); return; }
    if (!Number.isInteger(state.top_k) || state.top_k < 1 || state.top_k > 20) {
        showErr(t('view.pattern_discovery.err.top_k_must_be_an_integer_in_1_20')); return;
    }

    let res;
    try {
        res = await api.anlyMatrixProfile({
            series: parsed.value,
            m: state.m,
            top_k_discords: state.top_k,
        });
        if (!res) throw new Error(t('view.pattern_discovery.error.null'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    const motif = unpackMotifPair(res.top_motif_pair);
    const discords = unpackDiscords(res.top_discords);
    renderSummary(motif, discords);
    renderCharts(parsed.value, state.m, res.profile, motif, discords);
}

function renderSummary(motif, discords) {
    const motifCard = motif
        ? `<div class="card"><div class="label" data-i18n="view.pattern_discovery.card.top_motif_pair">Top motif pair</div>
             <div class="value">idx ${motif.i} ↔ idx ${motif.j}</div>
             <div class="vc-row"><span class="muted">distance</span> <strong>${motif.distance.toFixed(4)}</strong></div>
           </div>`
        : `<div class="card"><div class="label" data-i18n="view.pattern_discovery.card.top_motif_pair">Top motif pair</div>
             <div class="value">none</div>
             <div class="vc-row"><span class="muted" data-i18n="view.pattern_discovery.row.reason">reason</span> <strong data-i18n="view.pattern_discovery.row.flat_noisy">flat / too noisy</strong></div>
           </div>`;
    const discordCard = discords.length
        ? `<div class="card"><div class="label" data-i18n="view.pattern_discovery.card.top_discords_dist">Top discords (by distance)</div>
             <div class="value pd-discord-value">
                ${discords.map(d =>
                    `<div class="vc-row"><span class="muted">idx ${d.start}</span> <strong>${d.distance.toFixed(4)}</strong></div>`
                ).join('')}
             </div>
           </div>`
        : `<div class="card"><div class="label" data-i18n="view.pattern_discovery.card.top_discords">Top discords</div><div class="value">none</div></div>`;
    const pdSummary = document.getElementById('pd-summary');
    pdSummary.innerHTML = motifCard + discordCard;
    try { applyUiI18n(pdSummary); } catch (_) {}
}

function renderCharts(series, m, profile, motif, discords) {
    if (!window.uPlot) {
        document.getElementById('pd-chart-series').textContent = t('common.error.uplot_not_loaded');
        return;
    }
    const xs = indexAxis(series.length);

    // Overlay series for motif (two windows) and discords (top K).
    const motifWindows = motif ? [{ start: motif.i }, { start: motif.j }] : [];
    const motifOverlay = overlaySeriesForWindows(series, motifWindows, m);
    const discordOverlay = overlaySeriesForWindows(series, discords, m);

    // ── Series chart ─────────────────────────────────────────────────
    const seriesEl = document.getElementById('pd-chart-series');
    seriesEl.innerHTML = '';
    new window.uPlot({
        title: '', width: seriesEl.clientWidth || 800, height: 240,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.idx') },
            { label: t('chart.series.series'), stroke: 'rgba(170,170,170,0.55)', width: 1,
              points: { show: false } },
            { label: 'motif pair', stroke: '#00e5ff', width: 3,
              points: { show: false } },
            { label: 'discords', stroke: '#ff3860', width: 3,
              points: { show: false } },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, series, motifOverlay, discordOverlay], seriesEl);

    // ── Profile chart ────────────────────────────────────────────────
    // Profile length = N - m + 1. Pad with nulls to length N so the
    // x-axis aligns visually with the series chart above.
    const padded = new Array(series.length).fill(null);
    for (let i = 0; i < profile.length; i++) padded[i] = profile[i];
    const profEl = document.getElementById('pd-chart-profile');
    profEl.innerHTML = '';
    new window.uPlot({
        title: '', width: profEl.clientWidth || 800, height: 200,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.idx') },
            { label: 'profile distance', stroke: '#ff9f1a', width: 1.5,
              fill: 'rgba(255,159,26,0.10)', points: { show: false } },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, padded], profEl);
}

function renderParseErrors(errors) {
    const el = document.getElementById('pd-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">… (+${errors.length - 20} more)</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('pd-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('pd-parse-errors').style.display = 'none';
    document.getElementById('pd-err').style.display = 'none';
}
