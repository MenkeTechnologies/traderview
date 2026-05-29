// Monte Carlo Path Simulator view. Picks a stochastic model, runs the
// matching backend simulator, and visualizes terminal-distribution
// stats. fBm additionally returns the raw path for plotting.
//
// What this view CAN'T do (yet): show individual sample paths or a true
// terminal-price histogram. The backend simulators return summary
// statistics only (mean, stdev, etc.) — not per-path terminals. The
// "Distribution" chart is a normal-approximation density centered on
// the reported (mean, stdev); for jump models the real distribution is
// skewed and fat-tailed, so the curve is a sanity overlay, not the
// truth. The reported skew tells you how wrong the normal is.

import { api } from '../api.js';
import { esc, fmt, fmtMoney } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    MODELS, validateValues, defaultValues, normalDensityCurve,
} from '../_monte_carlo_models.js';

let state = {
    modelId: 'gbm',
    values: defaultValues('gbm'),
    lastStats: null,
};

export async function renderMonteCarlo(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 class="view-title">// MONTE CARLO</h1>

        <div class="chart-panel">
            <h2>Model</h2>
            <div class="inline-form">
                <label>Stochastic model
                    <select id="mc-model">
                        ${Object.entries(MODELS).map(([id, m]) =>
                            `<option value="${id}" ${id === state.modelId ? 'selected' : ''}>${esc(m.label)}</option>`
                        ).join('')}
                    </select>
                </label>
                <button id="mc-run" class="primary" type="button">Run</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2>Parameters</h2>
            <div id="mc-params" class="inline-form"></div>
        </div>

        <div id="mc-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 id="mc-chart-title">Terminal-price distribution (normal approximation)</h2>
            <div id="mc-chart" style="width:100%;height:340px"></div>
            <p class="muted" id="mc-chart-caption">
                Density curve is a normal approximation around (mean, stdev). For jump
                models the real distribution is skewed and fat-tailed — use the reported
                skew to gauge how misleading the normal is.
            </p>
        </div>

        <div id="mc-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    renderParamForm();
    wireForm(mount, tok);
    void fmt; void fmtMoney;
}

function renderParamForm() {
    const wrap = document.getElementById('mc-params');
    const model = MODELS[state.modelId];
    wrap.innerHTML = model.fields.map(f => {
        const v = state.values[f.key];
        const step = f.integer ? '1' : (f.step || 'any');
        const attrs = [
            `type="number"`, `step="${step}"`, `value="${v}"`,
            `data-field="${f.key}"`,
            f.min != null ? `min="${f.min}"` : '',
            f.max != null ? `max="${f.max}"` : '',
        ].filter(Boolean).join(' ');
        return `<label>${esc(f.label)} <input ${attrs}></label>`;
    }).join('');
    wrap.querySelectorAll('input[data-field]').forEach(el => {
        el.addEventListener('change', e => {
            const key = e.target.dataset.field;
            const f = MODELS[state.modelId].fields.find(x => x.key === key);
            state.values[key] = f.integer ? parseInt(e.target.value, 10) : Number(e.target.value);
        });
    });
}

function wireForm(mount, tok) {
    document.getElementById('mc-model').addEventListener('change', e => {
        state.modelId = e.target.value;
        state.values = defaultValues(state.modelId);
        renderParamForm();
        document.getElementById('mc-summary').innerHTML = '';
        document.getElementById('mc-chart').innerHTML = '';
    });
    document.getElementById('mc-run').addEventListener('click', () => {
        void runSim(mount, tok);
    });
}

async function runSim(mount, tok) {
    hideErr();
    const validation = validateValues(state.modelId, state.values);
    if (validation) { showErr(validation); return; }

    const model = MODELS[state.modelId];
    const body = model.buildBody(state.values);
    let res;
    try {
        res = await api[model.endpoint](body);
        if (res == null) throw new Error('simulator returned null (invalid parameters)');
    } catch (e) {
        showErr(`API error: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;

    const stats = model.extractTerminalStats(res);
    if (!stats) { showErr('simulator returned unexpected shape'); return; }
    state.lastStats = stats;
    renderSummary(stats);
    renderChart(stats);
}

function renderSummary(stats) {
    const cards = [];
    cards.push(card('Mean (terminal)', formatN(stats.mean, 4)));
    cards.push(card('Stdev (terminal)', formatN(stats.stdev, 4)));
    if (Number.isFinite(stats.min)) cards.push(card('Min', formatN(stats.min, 4)));
    if (Number.isFinite(stats.max)) cards.push(card('Max', formatN(stats.max, 4)));
    if (Number.isFinite(stats.skew)) {
        const cls = stats.skew < 0 ? 'neg' : 'pos';
        cards.push(card('Skew (log-return)', formatN(stats.skew, 4), cls));
    }
    if (stats.extra) {
        for (const [label, value] of Object.entries(stats.extra)) {
            cards.push(card(label, String(value)));
        }
    }
    if (Number.isFinite(stats.paths_run)) {
        cards.push(card('Paths run', String(stats.paths_run)));
    }
    document.getElementById('mc-summary').innerHTML = cards.join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(stats) {
    const el = document.getElementById('mc-chart');
    const title = document.getElementById('mc-chart-title');
    const caption = document.getElementById('mc-chart-caption');
    if (!window.uPlot) {
        el.textContent = 'uPlot not loaded — run scripts/vendor-uplot.sh';
        return;
    }
    el.innerHTML = '';

    // fBm returns the actual path; plot it as a time series.
    if (stats.path && stats.path.length > 1) {
        title.textContent = 'fBm path';
        caption.textContent = 'Single fractional Brownian motion realization. Higher Hurst H → smoother / more persistent.';
        const xs = stats.path.map((_, i) => i);
        new window.uPlot({
            title: '', width: el.clientWidth || 800, height: 340,
            scales: { x: {}, y: {} },
            series: [
                { label: 'sample' },
                { label: 'value', stroke: '#00e5ff', width: 1.5,
                  fill: 'rgba(0,229,255,0.06)' },
            ],
            axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
        }, [xs, stats.path], el);
        return;
    }

    // Default: normal-approximation density curve.
    title.textContent = 'Terminal-price distribution (normal approximation)';
    caption.textContent = 'Density curve is a normal approximation around (mean, stdev). '
        + 'For jump models the real distribution is skewed and fat-tailed — use the reported '
        + 'skew to gauge how misleading the normal is.';
    const { xs, ys } = normalDensityCurve(stats.mean, stats.stdev);
    if (xs.length === 0) {
        el.innerHTML = '<div class="boot">stdev was zero — distribution is a point mass.</div>';
        return;
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: 'terminal price' },
            { label: 'density (normal ≈)', stroke: '#00e5ff', width: 2,
              fill: 'rgba(0,229,255,0.10)' },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, ys], el);
}

function showErr(msg) {
    const el = document.getElementById('mc-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('mc-err').style.display = 'none'; }

function formatN(x, digits) {
    if (!Number.isFinite(x)) return '—';
    return x.toFixed(digits);
}
