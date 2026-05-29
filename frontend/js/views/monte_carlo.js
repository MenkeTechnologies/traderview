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

import { t } from '../i18n.js';
let state = {
    modelId: 'gbm',
    values: defaultValues('gbm'),
    lastStats: null,
};

export async function renderMonteCarlo(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.monte_carlo.h1.monte_carlo" class="view-title">// MONTE CARLO</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.monte_carlo.h2.model">Model</h2>
            <div class="inline-form">
                <label><span data-i18n="view.monte_carlo.label.model">Stochastic model</span>
                    <select id="mc-model">
                        ${Object.entries(MODELS).map(([id, m]) =>
                            `<option value="${id}" ${id === state.modelId ? 'selected' : ''}>${esc(m.label)}</option>`
                        ).join('')}
                    </select>
                </label>
                <button data-i18n="view.monte_carlo.btn.run" id="mc-run" class="primary" type="button">Run</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.monte_carlo.h2.parameters">Parameters</h2>
            <div id="mc-params" class="inline-form"></div>
        </div>

        <div id="mc-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.monte_carlo.h2.terminal_price_distribution_normal_approximation" id="mc-chart-title">Terminal-price distribution (normal approximation)</h2>
            <div id="mc-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.monte_carlo.hint.density_curve_is_a_normal_approximation_around_mea" class="muted" id="mc-chart-caption">
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
        if (res == null) throw new Error(t('view.monte_carlo.error.null'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    const stats = model.extractTerminalStats(res);
    if (!stats) { showErr(t('view.monte_carlo.err.simulator_returned_unexpected_shape')); return; }
    state.lastStats = stats;
    renderSummary(stats);
    renderChart(stats);
}

function renderSummary(stats) {
    const cards = [];
    cards.push(card(t('view.monte_carlo.card.mean_terminal'), formatN(stats.mean, 4)));
    cards.push(card(t('view.monte_carlo.card.stdev_terminal'), formatN(stats.stdev, 4)));
    if (Number.isFinite(stats.min)) cards.push(card(t('view.monte_carlo.card.min'), formatN(stats.min, 4)));
    if (Number.isFinite(stats.max)) cards.push(card(t('view.monte_carlo.card.max'), formatN(stats.max, 4)));
    if (Number.isFinite(stats.skew)) {
        const cls = stats.skew < 0 ? 'neg' : 'pos';
        cards.push(card(t('view.monte_carlo.card.skew_log_return'), formatN(stats.skew, 4), cls));
    }
    if (stats.extra) {
        for (const [label, value] of Object.entries(stats.extra)) {
            cards.push(card(label, String(value)));
        }
    }
    if (Number.isFinite(stats.paths_run)) {
        cards.push(card(t('view.monte_carlo.card.paths_run'), String(stats.paths_run)));
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
        el.textContent = t('common.error.uplot_not_loaded_install');
        return;
    }
    el.innerHTML = '';

    // fBm returns the actual path; plot it as a time series.
    if (stats.path && stats.path.length > 1) {
        title.textContent = t('view.monte_carlo.fbm.title');
        caption.textContent = t('view.monte_carlo.fbm.caption');
        const xs = stats.path.map((_, i) => i);
        new window.uPlot({
            title: '', width: el.clientWidth || 800, height: 340,
            scales: { x: {}, y: {} },
            series: [
                { label: t('chart.series.sample') },
                { label: t('chart.series.value'), stroke: '#00e5ff', width: 1.5,
                  fill: 'rgba(0,229,255,0.06)' },
            ],
            axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
        }, [xs, stats.path], el);
        return;
    }

    // Default: normal-approximation density curve.
    title.textContent = t('view.monte_carlo.terminal.title');
    caption.textContent = t('view.monte_carlo.terminal.caption');
    const { xs, ys } = normalDensityCurve(stats.mean, stats.stdev);
    if (xs.length === 0) {
        el.innerHTML = `<div class="boot">${esc(t('view.monte_carlo.empty.stdev_zero'))}</div>`;
        return;
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.terminal_price') },
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
