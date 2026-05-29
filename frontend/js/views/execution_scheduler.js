// Execution Scheduler — compares 3 child-order algorithms side-by-side:
//
//   TWAP — equal slices across N bars.
//   VWAP — slices proportional to expected per-bar volume.
//   POV  — fixed participation rate of expected per-bar volume.
//
// Visualization: per-bar slice bars per algo on the same axis, plus a
// summary card per algo showing total filled, shortfall (POV only),
// and peak participation rate.
//
// Use cases:
//   * "I need to work 500k shares — what does each algo's slice
//      schedule look like against today's typical volume curve?"
//   * "At 10% POV, do I finish? If not, what's the shortfall?"
//   * "Where does VWAP put the largest slices vs TWAP?"

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseVolumeCurve, validateExecInputs,
    buildPovBody, buildTwapBody, buildVwapBody,
    summarizeSchedule,
} from '../_execution_scheduler_inputs.js';

const DEFAULT_VOLUME = `# Per-bar expected volume (shares). One value per token.
# Demo: 26 half-hour bars with U-shaped intraday liquidity.
${synthDemoVolume().join('\n')}
`;

function synthDemoVolume() {
    // Classic U-curve: heavy open + close, thin mid-day.
    const n = 26;
    const out = [];
    for (let i = 0; i < n; i++) {
        const x = (i - (n - 1) / 2) / ((n - 1) / 2);  // -1..+1
        const u = 0.3 + 0.7 * x * x;                  // 0.3 (mid) → 1.0 (edges)
        out.push(Math.round(u * 250_000));
    }
    return out;
}

let state = {
    totalOrder: 1_500_000,
    participationRate: 0.10,
    volumeText: DEFAULT_VOLUME,
};

export async function renderExecutionScheduler(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 class="view-title">// EXECUTION SCHEDULER</h1>

        <div class="chart-panel">
            <h2>Inputs</h2>
            <div class="inline-form">
                <label>Total order size
                    <input id="es-total" type="number" step="any" min="1" value="${state.totalOrder}"></label>
                <label>POV participation rate
                    <input id="es-rate" type="number" step="0.01" min="0.01" max="1" value="${state.participationRate}"></label>
                <button id="es-run" class="primary" type="button">Schedule</button>
            </div>
            <p class="muted">
                TWAP slices equally across all bars in the curve. VWAP weights by expected
                volume. POV takes the participation-rate fraction of each bar — may fall short.
            </p>
            <h3>Volume curve (per bar)</h3>
            <textarea id="es-vol" rows="8"
                style="width:100%;font-family:monospace;font-size:13px">${esc(state.volumeText)}</textarea>
        </div>

        <div id="es-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="es-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Per-bar slice schedule</h2>
            <div id="es-chart" style="width:100%;height:340px"></div>
            <p class="muted">Three series, one per algo. Vertical bars show shares per bar.</p>
        </div>

        <div id="es-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    wireForm(mount, tok);
    void fmt;
}

function wireForm(mount, tok) {
    document.getElementById('es-run').addEventListener('click', () => {
        state.totalOrder = Number(document.getElementById('es-total').value);
        state.participationRate = Number(document.getElementById('es-rate').value);
        state.volumeText = document.getElementById('es-vol').value;
        void schedule(mount, tok);
    });
}

async function schedule(mount, tok) {
    hideErrs();
    const parsed = parseVolumeCurve(state.volumeText);
    if (parsed.errors.length) renderParseErrors(parsed.errors);

    const err = validateExecInputs(state.totalOrder, parsed.value, state.participationRate);
    if (err) { showErr(err); return; }

    const numBars = parsed.value.length;
    let pov, twap, vwap;
    try {
        [pov, twap, vwap] = await Promise.all([
            api.anlyOptimalExecutionPov(buildPovBody(state.totalOrder, parsed.value, state.participationRate)),
            api.anlyOptimalExecutionTwap(buildTwapBody(state.totalOrder, numBars, parsed.value)),
            api.anlyOptimalExecutionVwap(buildVwapBody(state.totalOrder, parsed.value)),
        ]);
    } catch (e) {
        showErr(`API error: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(pov, twap, vwap);
    renderChart(parsed.value, pov, twap, vwap);
}

function renderSummary(pov, twap, vwap) {
    const cards = [];
    cards.push(algoCard('POV', '#00e5ff', pov, state.totalOrder));
    cards.push(algoCard('TWAP', '#ff9f1a', twap, state.totalOrder));
    cards.push(algoCard('VWAP', '#a06bff', vwap, state.totalOrder));
    document.getElementById('es-summary').innerHTML = cards.join('');
}

function algoCard(label, color, res, totalOrder) {
    if (!res) {
        return `<div class="card">
            <div class="label">${esc(label)}</div>
            <div class="value neg">failed</div>
        </div>`;
    }
    const s = summarizeSchedule(res);
    const pctFilled = (s.totalFilled / totalOrder) * 100;
    const rows = [];
    rows.push(kv('Filled', `${pctFilled.toFixed(1)}%`));
    if (s.shortfall != null && s.shortfall > 1e-6) {
        rows.push(kv('Shortfall', `${Math.round(s.shortfall).toLocaleString()} sh`));
    }
    if (s.completionBar != null) {
        rows.push(kv('Done at bar', String(s.completionBar)));
    } else if (s.lastFillBar != null) {
        rows.push(kv('Last fill bar', String(s.lastFillBar)));
    }
    if (s.maxParticipation != null) {
        rows.push(kv('Peak participation', `${(s.maxParticipation * 100).toFixed(2)}%`));
    }
    return `<div class="card">
        <div class="label"><span class="es-swatch ${esc(label.toLowerCase())}">▮</span> ${esc(label)}</div>
        <div class="value es-summary-value">${rows.join('')}</div>
    </div>`;
}

function kv(label, value) {
    return `<div class="vc-row"><span class="muted">${esc(label)}</span> <strong>${esc(value)}</strong></div>`;
}

function renderChart(volumeCurve, pov, twap, vwap) {
    const el = document.getElementById('es-chart');
    if (!window.uPlot) {
        el.textContent = 'uPlot not loaded';
        return;
    }
    el.innerHTML = '';
    const n = volumeCurve.length;
    const xs = Array.from({ length: n }, (_, i) => i);
    // Use uPlot's bars=true path via points. We'll draw 3 stacked-ish
    // line series sharing the same x. Real per-bar bars would need a
    // custom paths fn — for first cut, render each algo's slice as a
    // line with point markers, which makes the per-bar peaks visible
    // without custom canvas painting.
    const series = [
        { label: 'bar' },
        // Reference: expected volume in muted grey.
        { label: 'volume', stroke: 'rgba(170,170,170,0.45)', width: 1,
          fill: 'rgba(170,170,170,0.05)' },
        { label: 'POV slice', stroke: '#00e5ff', width: 2,
          points: { show: true, size: 6, stroke: '#00e5ff', fill: '#00e5ff' } },
        { label: 'TWAP slice', stroke: '#ff9f1a', width: 2,
          points: { show: true, size: 6, stroke: '#ff9f1a', fill: '#ff9f1a' } },
        { label: 'VWAP slice', stroke: '#a06bff', width: 2,
          points: { show: true, size: 6, stroke: '#a06bff', fill: '#a06bff' } },
    ];
    const data = [
        xs,
        volumeCurve,
        pov ? pov.slices : new Array(n).fill(null),
        twap ? padToLength(twap.slices, n) : new Array(n).fill(null),
        vwap ? vwap.slices : new Array(n).fill(null),
    ];
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series,
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, data, el);
}

function padToLength(arr, n) {
    if (!Array.isArray(arr)) return new Array(n).fill(null);
    if (arr.length >= n) return arr;
    return arr.concat(new Array(n - arr.length).fill(null));
}

function renderParseErrors(errors) {
    const el = document.getElementById('es-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">… (+${errors.length - 20} more)</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('es-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('es-parse-errors').style.display = 'none';
    document.getElementById('es-err').style.display = 'none';
}
