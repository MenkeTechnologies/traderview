// Series Smoother — paste a 1-D series and overlay multiple smoothers
// on the same chart. Useful for visually choosing between LOWESS,
// Kalman RTS, robust Theil-Sen line, and polynomial fit when the
// "right" smoother depends on the noise profile of your data.
//
// Each smoother is opt-in via a checkbox; running with none ticked
// shows only the raw series. Params live next to their checkboxes so
// the user can see what they're tuning.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseSeries, validateSeries, indexAxis,
    buildSmootherPayloads, theilSenFittedY,
    defaultOptions, validateOptions,
} from '../_series_smoother_inputs.js';

const DEFAULT_TEXT = `# Paste a price OR return series — one value per token.
# Whitespace, comma, or newline separated. # comments skipped.
# Demo: 60-point noisy sine with mild upward drift.
${synthDemoSeries(60).join('\n')}
`;

function synthDemoSeries(n) {
    let s = 0x12345678n;
    const rand = () => {
        s = (s * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(s >> 11n) / 2 ** 53;
    };
    const out = [];
    for (let i = 0; i < n; i++) {
        const trend = 100 + 0.1 * i;
        const cycle = 3 * Math.sin(i * 0.35);
        const noise = (rand() - 0.5) * 4;
        out.push((trend + cycle + noise).toFixed(3));
    }
    return out;
}

let state = {
    text: DEFAULT_TEXT,
    enabled: { lowess: true, kalman_rts: true, theil_sen: true, polynomial: true },
    opts: defaultOptions(),
};

const SMOOTHER_META = {
    lowess:     { label: 'LOWESS',        color: '#00e5ff' },
    kalman_rts: { label: 'Kalman (RTS)',  color: '#ff9f1a' },
    theil_sen:  { label: 'Theil-Sen',     color: '#a06bff' },
    polynomial: { label: 'Polynomial',    color: '#39ff14' },
};

export async function renderSeriesSmoother(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.series_smoother.h1.series_smoother" class="view-title">// SERIES SMOOTHER</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.series_smoother.h2.series">Series</h2>
            <textarea id="ss-text" rows="8"
                style="width:100%;font-family:monospace;font-size:13px">${esc(state.text)}</textarea>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.series_smoother.h2.smoothers">Smoothers</h2>
            <div class="ss-smoothers">
                <div class="ss-smoother">
                    <label><input type="checkbox" data-toggle="lowess" ${state.enabled.lowess ? 'checked' : ''}>
                        <span class="ss-swatch lowess">▮</span> LOWESS</label>
                    <label>frac <input type="number" step="0.05" min="0.05" max="1" value="${state.opts.lowess_frac}" data-opt="lowess_frac"></label>
                    <label>robust iter <input type="number" step="1" min="0" max="5" value="${state.opts.lowess_robust}" data-opt="lowess_robust"></label>
                </div>
                <div class="ss-smoother">
                    <label><input type="checkbox" data-toggle="kalman_rts" ${state.enabled.kalman_rts ? 'checked' : ''}>
                        <span class="ss-swatch kalman_rts">▮</span> Kalman (RTS)</label>
                    <label>process q <input type="number" step="any" min="0" value="${state.opts.kalman_q}" data-opt="kalman_q"></label>
                    <label>obs r <input type="number" step="any" min="1e-9" value="${state.opts.kalman_r}" data-opt="kalman_r"></label>
                </div>
                <div class="ss-smoother">
                    <label><input type="checkbox" data-toggle="theil_sen" ${state.enabled.theil_sen ? 'checked' : ''}>
                        <span class="ss-swatch theil_sen">▮</span> Theil-Sen (robust line)</label>
                    <span class="muted">(no params — pair-slope median)</span>
                </div>
                <div class="ss-smoother">
                    <label><input type="checkbox" data-toggle="polynomial" ${state.enabled.polynomial ? 'checked' : ''}>
                        <span class="ss-swatch polynomial">▮</span> Polynomial</label>
                    <label>degree <input type="number" step="1" min="1" max="10" value="${state.opts.poly_degree}" data-opt="poly_degree"></label>
                </div>
            </div>
            <button data-i18n="view.series_smoother.btn.smooth" id="ss-run" class="primary" type="button" style="margin-top:10px">Smooth</button>
        </div>

        <div id="ss-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="ss-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.series_smoother.h2.series_smoothers">Series + smoothers</h2>
            <div id="ss-chart" style="width:100%;height:380px"></div>
        </div>

        <div id="ss-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    wireForm(mount, tok);
    void fmt;
}

function wireForm(mount, tok) {
    document.querySelectorAll('input[data-toggle]').forEach(el => {
        el.addEventListener('change', e => {
            state.enabled[e.target.dataset.toggle] = e.target.checked;
        });
    });
    document.querySelectorAll('input[data-opt]').forEach(el => {
        el.addEventListener('change', e => {
            const key = e.target.dataset.opt;
            const v = Number(e.target.value);
            state.opts[key] = Number.isInteger(state.opts[key]) ? Math.round(v) : v;
        });
    });
    document.getElementById('ss-run').addEventListener('click', () => {
        state.text = document.getElementById('ss-text').value;
        void run(mount, tok);
    });
}

async function run(mount, tok) {
    hideErrs();
    const parsed = parseSeries(state.text);
    if (parsed.errors.length) renderParseErrors(parsed.errors);

    const seriesErr = validateSeries(parsed.value);
    if (seriesErr) { showErr(seriesErr); return; }
    const optsErr = validateOptions(state.opts);
    if (optsErr) { showErr(optsErr); return; }

    const series = parsed.value;
    const xs = indexAxis(series.length);
    const payloads = buildSmootherPayloads(series, state.opts);

    const wanted = Object.keys(SMOOTHER_META).filter(k => state.enabled[k]);
    const requests = wanted.map(k => callSmoother(k, payloads[k]));

    let results;
    try {
        results = await Promise.all(requests);
    } catch (e) {
        showErr(`API error: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;

    // Each result is { y, summary } or null if the backend rejected
    // the inputs. Filter nulls; show summary stats.
    const smoothed = {};
    const summaries = [];
    wanted.forEach((k, i) => {
        const r = results[i];
        if (!r) {
            summaries.push({ id: k, label: SMOOTHER_META[k].label, status: 'failed', note: 'endpoint returned null' });
            return;
        }
        smoothed[k] = r.y;
        summaries.push({ id: k, label: SMOOTHER_META[k].label, status: 'ok', note: r.summary || '' });
    });

    renderSummary(summaries);
    renderChart(xs, series, smoothed);
}

async function callSmoother(id, body) {
    if (id === 'lowess') {
        const y = await api.anlyLowessSmoother(body);
        return y ? { y, summary: `frac=${body.frac}, robust=${body.robustness_iter}` } : null;
    }
    if (id === 'kalman_rts') {
        const r = await api.anlyKalmanSmootherRts(body);
        return r ? { y: r.smoothed_state, summary: `q=${body.process_noise_q}, r=${body.obs_noise_r}` } : null;
    }
    if (id === 'theil_sen') {
        const r = await api.anlyTheilSenEstimator(body);
        if (!r) return null;
        return {
            y: theilSenFittedY(body.x, r.slope, r.intercept),
            summary: `slope=${r.slope.toFixed(4)}, intercept=${r.intercept.toFixed(3)}, n_pairs=${r.n_pairs}`,
        };
    }
    if (id === 'polynomial') {
        const r = await api.anlyPolynomialRegression(body);
        return r ? { y: r.fitted, summary: `deg=${body.degree}, R²=${r.r_squared.toFixed(4)}` } : null;
    }
    return null;
}

function renderSummary(summaries) {
    const cards = summaries.map(s => {
        return `<div class="card">
            <div class="label"><span class="ss-swatch ${esc(s.id)}">▮</span> ${esc(s.label)}</div>
            <div class="value ss-summary-value">
                ${s.status === 'failed' ? `<span class="neg">${esc(s.note)}</span>` : esc(s.note)}
            </div>
        </div>`;
    });
    document.getElementById('ss-summary').innerHTML = cards.join('');
}

function renderChart(xs, raw, smoothed) {
    const el = document.getElementById('ss-chart');
    if (!window.uPlot) {
        el.textContent = t('common.error.uplot_not_loaded_install');
        return;
    }
    el.innerHTML = '';

    const series = [
        { label: 'idx' },
        { label: 'raw', stroke: 'rgba(170,170,170,0.55)', width: 1, points: { show: false } },
    ];
    const data = [xs, raw];
    for (const id of Object.keys(SMOOTHER_META)) {
        if (smoothed[id]) {
            series.push({
                label: SMOOTHER_META[id].label,
                stroke: SMOOTHER_META[id].color,
                width: 2,
                points: { show: false },
            });
            data.push(smoothed[id]);
        }
    }

    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 380,
        scales: { x: {}, y: {} },
        series,
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, data, el);
}

function renderParseErrors(errors) {
    const el = document.getElementById('ss-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">… (+${errors.length - 20} more)</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('ss-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('ss-parse-errors').style.display = 'none';
    document.getElementById('ss-err').style.display = 'none';
}
