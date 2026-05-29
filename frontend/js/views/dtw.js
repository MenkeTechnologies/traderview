// Dynamic Time Warping calculator. Paste two series, get the optimal
// warping path (a monotonic alignment of A's indices to B's indices
// that minimizes the L1 sum of |A[i] - B[j]|) and the resulting
// distance.
//
// Visualization:
//   * Overlay chart: A and B plotted on the same x-axis (raw, no
//     alignment) — for visual intuition of where they differ.
//   * Warping path chart: x = index into A, y = index into B. A
//     diagonal y=x line marks zero-stretch reference; the actual path
//     departs from the diagonal wherever DTW found a non-trivial
//     alignment. Strong departures = the two series ran the same
//     pattern but at different speeds.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseSeries, buildBody, validateInputs,
    normalizedDistance, maxStretch, pathToSeries,
} from '../_dtw_inputs.js';

import { t } from '../i18n.js';
const DEFAULT_A = `# Series A — sine wave starting at index 0
${synthSine(60, 0).join('\n')}
`;
const DEFAULT_B = `# Series B — same sine wave but shifted 10 steps right
# (so its first 10 values are baseline, then it follows A's pattern).
# DTW should align A[0..50] with B[10..60] and the warping path
# should be a diagonal offset by 10 in the middle.
${synthSineShifted(60, 10).join('\n')}
`;

function synthSine(n, _phase) {
    const out = [];
    for (let i = 0; i < n; i++) out.push(Math.sin(i * 0.2).toFixed(4));
    return out;
}

function synthSineShifted(n, shift) {
    const out = [];
    for (let i = 0; i < n; i++) {
        const v = i < shift ? 0 : Math.sin((i - shift) * 0.2);
        out.push(v.toFixed(4));
    }
    return out;
}

let state = { textA: DEFAULT_A, textB: DEFAULT_B, bandRadius: 0 };

export async function renderDtw(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.dtw.h1.dynamic_time_warping" class="view-title">// DYNAMIC TIME WARPING</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.dtw.h2.series">Series</h2>
            <div class="op-inputs-grid">
                <div>
                    <h3 data-i18n="view.dtw.h3.series_a">Series A</h3>
                    <textarea id="dt-a" rows="10"
                        style="width:100%;font-family:monospace;font-size:13px">${esc(state.textA)}</textarea>
                </div>
                <div>
                    <h3 data-i18n="view.dtw.h3.series_b">Series B</h3>
                    <textarea id="dt-b" rows="10"
                        style="width:100%;font-family:monospace;font-size:13px">${esc(state.textB)}</textarea>
                </div>
            </div>
            <div class="inline-form" style="margin-top:8px">
                <label><span data-i18n="view.dtw.label.band_radius">Band radius (0 = unconstrained)</span>
                    <input id="dt-band" type="number" step="1" min="0" value="${state.bandRadius}"></label>
                <button data-i18n="view.dtw.btn.warp" id="dt-run" class="primary" type="button">Warp</button>
            </div>
            <p data-i18n="view.dtw.hint.sakoe_chiba_band_constrains_the_warping_path_so_i_" class="muted">
                Sakoe-Chiba band constrains the warping path so |i−j| ≤ radius. Set 0 to
                disable (full O(n·m) DP); set a small radius (~10-20% of series length) to
                speed up long series and avoid pathological cross-warps.
            </p>
        </div>

        <div id="dt-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="dt-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.dtw.h2.series_overlay_raw_no_alignment">Series overlay (raw, no alignment)</h2>
            <div id="dt-overlay-chart" style="width:100%;height:280px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.dtw.h2.warping_path">Warping path</h2>
            <div id="dt-path-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.dtw.hint.cyan_optimal_alignment_from_dtw_a_s_index_b_s_inde" class="muted">
                Cyan: optimal alignment from DTW (A's index ↔ B's index). Dashed orange:
                diagonal reference (y = x). Departures from the diagonal indicate where DTW
                found one series leading or lagging the other.
            </p>
        </div>

        <div id="dt-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('dt-run').addEventListener('click', () => {
        state.textA = document.getElementById('dt-a').value;
        state.textB = document.getElementById('dt-b').value;
        state.bandRadius = parseInt(document.getElementById('dt-band').value, 10);
        void warp(mount, tok);
    });
    void fmt;
}

async function warp(mount, tok) {
    hideErrs();
    const parsedA = parseSeries(state.textA);
    const parsedB = parseSeries(state.textB);
    const errors = parsedA.errors.concat(parsedB.errors);
    if (errors.length) renderParseErrors(errors);

    const err = validateInputs(parsedA.value, parsedB.value, state.bandRadius);
    if (err) { showErr(err); return; }

    let res;
    try {
        res = await api.anlyDynamicTimeWarping(buildBody(parsedA.value, parsedB.value, state.bandRadius));
        if (!res) throw new Error(t('view.dtw.error.null_result'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(parsedA.value, parsedB.value, res);
    renderOverlay(parsedA.value, parsedB.value);
    renderPath(parsedA.value.length, parsedB.value.length, res);
}

function renderSummary(a, b, res) {
    const pathLen = res.path.length;
    const norm = normalizedDistance(res.distance, pathLen);
    const ms = maxStretch(res.path);
    document.getElementById('dt-summary').innerHTML = [
        card(t('view.dtw.card.dtw_distance'), res.distance.toFixed(4)),
        card(t('view.dtw.card.path_length'), String(pathLen)),
        card(t('view.dtw.card.distance_pair_normalized'), norm == null ? '—' : norm.toFixed(6)),
        card(t('view.dtw.card.max_stretch_i_j'), String(ms),
            ms === 0 ? 'pos' : (ms > Math.max(a.length, b.length) / 4 ? 'neg' : '')),
        card(t('view.dtw.card.series_lengths'), `A=${a.length}, B=${b.length}`),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderOverlay(a, b) {
    const el = document.getElementById('dt-overlay-chart');
    if (!window.uPlot) { el.textContent = t('common.error.uplot_not_loaded'); return; }
    el.innerHTML = '';
    const maxLen = Math.max(a.length, b.length);
    const xs = Array.from({ length: maxLen }, (_, i) => i);
    // Pad shorter series with null so uPlot aligns them on a shared x.
    const padded = (s) => {
        if (s.length === maxLen) return s;
        return s.concat(new Array(maxLen - s.length).fill(null));
    };
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 280,
        scales: { x: {}, y: {} },
        series: [
            { label: 'idx' },
            { label: 'A', stroke: '#00e5ff', width: 2, points: { show: false } },
            { label: 'B', stroke: '#ff9f1a', width: 2, points: { show: false } },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, padded(a), padded(b)], el);
}

function renderPath(nA, nB, res) {
    const el = document.getElementById('dt-path-chart');
    if (!window.uPlot) { el.textContent = t('common.error.uplot_not_loaded'); return; }
    el.innerHTML = '';
    const { xs, ys } = pathToSeries(res.path);
    // Diagonal reference: y = x · (nB-1)/(nA-1) so the line spans from
    // (0,0) to (nA-1, nB-1) regardless of relative lengths.
    const scale = nA > 1 ? (nB - 1) / (nA - 1) : 1;
    const diagXs = [0, nA - 1];
    const diagYs = [0, (nA - 1) * scale];
    // Merge xs for shared scale.
    const allXs = Array.from(new Set([...xs, ...diagXs])).sort((a, b) => a - b);
    const pathAligned = allXs.map(x => {
        const i = xs.indexOf(x);
        return i >= 0 ? ys[i] : null;
    });
    const diagAligned = allXs.map(x => x * scale);

    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: 'i (A)' },
            { label: 'warping path', stroke: '#00e5ff', width: 2,
              points: { show: true, size: 3, stroke: '#00e5ff', fill: '#00e5ff' } },
            { label: 'y = x diagonal', stroke: '#ff9f1a', width: 1,
              dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab' },
            { stroke: '#aab', label: 'j (B)' },
        ],
    }, [allXs, pathAligned, diagAligned], el);
}

function renderParseErrors(errors) {
    const el = document.getElementById('dt-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">… (+${errors.length - 20} more)</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('dt-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('dt-parse-errors').style.display = 'none';
    document.getElementById('dt-err').style.display = 'none';
}
