// Monte Carlo equity curve forecast — fan chart + ruin / double probabilities.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t, applyUiI18n } from '../i18n.js';
import { showToast } from '../toast.js';

export async function renderEquityForecast(mount, state) {
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) { mount.innerHTML = `<p data-i18n="view.equity_forecast.hint.no_account_selected" class="boot">No account selected.</p>`; return; }
    mount.innerHTML = `
        <h1 class="view-title">// EQUITY FORECAST — ${esc(acct.broker)} · ${esc(acct.name)}</h1>
        <p class="muted small" data-i18n="view.equity_forecast.hint.intro">Bootstraps R-multiples from your closed-trade history and resamples thousands of forward paths. Each step risks risk_pct × equity and gains equity × risk × R where R is drawn with replacement. Ruin = equity drops to ≤ ruin_threshold of starting (default 50%); ruined paths freeze at zero for the rest of the horizon so percentile bands stay meaningful at the bottom of the fan.</p>

        <div class="chart-panel">
            <form id="ef-form" class="inline-form">
                <label><span data-i18n="view.equity_forecast.label.starting_equity">Starting equity</span>
                    <input name="starting_equity" type="number" min="100" step="0.01" value="10000" style="width:120px;" data-tip="view.equity_forecast.tip.start">
                </label>
                <label><span data-i18n="view.equity_forecast.label.risk_pct">Risk per trade %</span>
                    <input name="risk_pct" type="number" min="0.1" max="100" step="0.1" value="1" style="width:90px;" data-tip="view.equity_forecast.tip.risk">
                </label>
                <label><span data-i18n="view.equity_forecast.label.num_trades">Trades</span>
                    <input name="num_trades" type="number" min="10" max="2000" value="200" style="width:90px;" data-tip="view.equity_forecast.tip.trades">
                </label>
                <label><span data-i18n="view.equity_forecast.label.num_paths">Paths</span>
                    <input name="num_paths" type="number" min="100" max="50000" value="5000" style="width:100px;" data-tip="view.equity_forecast.tip.paths">
                </label>
                <label><span data-i18n="view.equity_forecast.label.ruin_pct">Ruin at %</span>
                    <input name="ruin_pct" type="number" min="0" max="100" step="1" value="50" style="width:80px;" data-tip="view.equity_forecast.tip.ruin">
                </label>
                <label><span data-i18n="view.equity_forecast.label.seed">Seed (opt)</span>
                    <input name="seed" type="number" style="width:120px;" data-tip="view.equity_forecast.tip.seed">
                </label>
                <button data-i18n="view.equity_forecast.btn.run_forecast" data-tip="view.equity_forecast.tip.run" data-shortcut="forecast_run" class="primary" type="submit">Run forecast</button>
                <span id="ef-status" class="muted small"></span>
            </form>
        </div>

        <div id="ef-out"></div>
    `;
    mount.querySelector('#ef-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const status = mount.querySelector('#ef-status');
        if (status) status.textContent = t('common.status.running');
        const body = {
            account_id: acct.id,
            starting_equity: Number(fd.get('starting_equity')) || 10_000,
            risk_pct_per_trade: (Number(fd.get('risk_pct')) || 1) / 100,
            num_trades: Number(fd.get('num_trades')) || 200,
            num_paths: Number(fd.get('num_paths')) || 5000,
            ruin_threshold_pct: (Number(fd.get('ruin_pct')) || 50) / 100,
            seed: fd.get('seed') ? Number(fd.get('seed')) : null,
        };
        try {
            const r = await api.equityForecast(body);
            if (!viewIsCurrent(tok)) return;
            render(r, mount);
            const status2 = mount.querySelector('#ef-status');
            if (status2) status2.textContent = t('view.equity_forecast.status.result', { paths: r.paths, steps: r.steps, samples: r.samples_used });
            showToast(t('view.equity_forecast.toast.done', {
                p_ruin: (r.ruin_probability * 100).toFixed(1),
                p_double: (r.double_probability * 100).toFixed(1),
            }), { level: r.ruin_probability >= 0.10 ? 'warning' : 'success' });
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#ef-status');
            if (status2) status2.textContent = t('common.error', { err: err.message });
            showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
        }
    });
}

function render(r, mount) {
    const out = mount.querySelector('#ef-out');
    if (!out) return;
    const ruinCls = r.ruin_probability >= 0.10 ? 'neg' : r.ruin_probability >= 0.02 ? 'warn' : 'pos';
    const dblCls  = r.double_probability >= 0.50 ? 'pos' : r.double_probability >= 0.25 ? '' : 'neg';
    const exCls   = r.mean_r >= 0 ? 'pos' : 'neg';
    out.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.equity_forecast.card.mean_r">Mean R (sampled)</div>
                <div class="value ${exCls}">${(r.mean_r >= 0 ? '+' : '') + r.mean_r.toFixed(3)}R</div>
                <div class="small muted">${esc(t('view.equity_forecast.card.stdev_r', { stdev: r.stdev_r.toFixed(3) }))}</div></div>
            <div class="card"><div class="label" data-i18n="view.equity_forecast.card.p_ruin">P(ruin)</div>
                <div class="value ${ruinCls}">${(r.ruin_probability * 100).toFixed(2)}%</div>
                <div class="small muted">${esc(t('view.equity_forecast.card.ruin_at', { pct: (r.ruin_threshold_pct * 100).toFixed(0) }))}</div></div>
            <div class="card"><div class="label" data-i18n="view.equity_forecast.card.p_double">P(double)</div>
                <div class="value ${dblCls}">${(r.double_probability * 100).toFixed(2)}%</div>
                <div class="small muted" data-i18n="view.equity_forecast.card.double_target">final ≥ 2× start</div></div>
            <div class="card"><div class="label" data-i18n="view.equity_forecast.card.median_final">Median final</div>
                <div class="value ${r.final_bands.p50 >= r.starting_equity ? 'pos' : 'neg'}">
                    $${fmt(r.final_bands.p50)}
                </div>
                <div class="small muted">${esc(t('view.equity_forecast.card.from', { amount: fmt(r.starting_equity) }))}</div></div>
            <div class="card"><div class="label" data-i18n="view.equity_forecast.card.p5_to_p95">p5 → p95</div>
                <div class="value small">$${fmt(r.final_bands.p5)} → $${fmt(r.final_bands.p95)}</div>
                <div class="small muted" data-i18n="view.equity_forecast.card.confidence_band">90% confidence band</div></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.equity_forecast.h2.equity_fan_p5_p25_p50_p75_p95">Equity fan (p5 / p25 / p50 / p75 / p95)</h2>
            ${fanSvg(r)}
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.equity_forecast.h2.sample_paths_first_50">Sample paths (first 50)</h2>
            ${spaghettiSvg(r)}
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.equity_forecast.h2.percentile_chart">Percentile bands by trade #</h2>
            <div id="ef-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.equity_forecast.h2.uncertainty_chart">Forecast uncertainty growth — (p95 − p5) / p50 by trade #</h2>
            <div id="ef-uncertainty-chart" style="width:100%;height:220px"></div>
        </div>
    `;
    try { applyUiI18n(out); } catch (_) {}
    renderPercentileChart(r);
    renderUncertaintyChart(r);
}

function renderPercentileChart(r) {
    const el = document.getElementById('ef-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const stats = r.steps_stats || [];
    if (stats.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.equity_forecast.empty_chart">${esc(t('view.equity_forecast.empty_chart'))}</div>`;
        return;
    }
    const xs = stats.map((_, i) => i);
    const p5 = stats.map(s => Number(s.bands.p5));
    const p25 = stats.map(s => Number(s.bands.p25));
    const p50 = stats.map(s => Number(s.bands.p50));
    const p75 = stats.map(s => Number(s.bands.p75));
    const p95 = stats.map(s => Number(s.bands.p95));
    const start = xs.map(() => Number(r.starting_equity));
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.equity_forecast.chart.trade_idx') },
            { label: t('view.equity_forecast.chart.p5'),
              stroke: '#ff3860', width: 1.0, points: { show: false } },
            { label: t('view.equity_forecast.chart.p25'),
              stroke: '#ff7a1f', width: 1.0, points: { show: false } },
            { label: t('view.equity_forecast.chart.p50'),
              stroke: '#00ffaa', width: 1.8, points: { show: false } },
            { label: t('view.equity_forecast.chart.p75'),
              stroke: '#7af0a8', width: 1.0, points: { show: false } },
            { label: t('view.equity_forecast.chart.p95'),
              stroke: '#00e5ff', width: 1.0, points: { show: false } },
            { label: t('view.equity_forecast.chart.starting_eq'),
              stroke: '#ffd24a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 60 } ],
        legend: { show: true },
    }, [xs, p5, p25, p50, p75, p95, start], el);
}

function renderUncertaintyChart(r) {
    const el = document.getElementById('ef-uncertainty-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const stats = r.steps_stats || [];
    if (stats.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.equity_forecast.empty_uncertainty_chart">${esc(t('view.equity_forecast.empty_uncertainty_chart'))}</div>`;
        return;
    }
    const xs = stats.map((_, i) => i);
    const uncertainty = stats.map(s => {
        const p50 = Number(s.bands.p50);
        const p5 = Number(s.bands.p5);
        const p95 = Number(s.bands.p95);
        if (!(p50 > 0) || !Number.isFinite(p5) || !Number.isFinite(p95)) return null;
        return (p95 - p5) / p50;
    });
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.equity_forecast.chart.trade_idx') },
            { label: t('view.equity_forecast.chart.band_width_ratio'),
              stroke: '#b86bff', width: 1.5,
              fill: 'rgba(184,107,255,0.10)',
              points: { show: false } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 } ],
        legend: { show: true },
    }, [xs, uncertainty], el);
}

function fanSvg(r) {
    if (!r.steps_stats.length) return '<p data-i18n="view.equity_forecast.hint.no_data" class="muted small">no data</p>';
    const W = 1000, H = 380, padL = 60, padR = 10, padT = 10, padB = 30;
    const innerW = W - padL - padR, innerH = H - padT - padB;
    const stats = r.steps_stats;
    const allY = stats.flatMap(s => [s.bands.p5, s.bands.p95, r.starting_equity]);
    const yMin = Math.min(...allY) * 0.95;
    const yMax = Math.max(...allY) * 1.05;
    const sx = (k) => padL + (k / Math.max(stats.length - 1, 1)) * innerW;
    const sy = (v) => padT + (1 - (v - yMin) / Math.max(yMax - yMin, 1e-9)) * innerH;

    const band = (lo, hi, color) => {
        const top    = stats.map((s, i) => `${sx(i)},${sy(s.bands[hi])}`).join(' ');
        const bottom = stats.slice().reverse().map((s, i) => {
            const idx = stats.length - 1 - i;
            return `${sx(idx)},${sy(s.bands[lo])}`;
        }).join(' ');
        return `<polygon points="${top} ${bottom}" fill="${color}" opacity="0.35"/>`;
    };
    const line = (key, color, width = 1.5) => {
        const d = stats.map((s, i) => (i ? 'L' : 'M') + sx(i) + ',' + sy(s.bands[key])).join(' ');
        return `<path d="${d}" stroke="${color}" stroke-width="${width}" fill="none"/>`;
    };
    const meanLine = `<path d="${stats.map((s, i) => (i ? 'L' : 'M') + sx(i) + ',' + sy(s.mean)).join(' ')}"
                            stroke="#ffd24a" stroke-width="1.5" stroke-dasharray="4,3" fill="none"/>`;
    const baseY = sy(r.starting_equity);
    const labels = [0, 0.25, 0.5, 0.75, 1].map(t => {
        const y = padT + t * innerH;
        const v = yMax - t * (yMax - yMin);
        return `<text x="${padL - 4}" y="${y + 3}" text-anchor="end" fill="#9aa0c8" font-size="10">$${fmt(v, 0)}</text>`;
    }).join('');

    return `<svg viewBox="0 0 ${W} ${H}" width="100%" style="display:block;">
        <rect x="${padL}" y="${padT}" width="${innerW}" height="${innerH}" fill="#0d0d22" stroke="#222"/>
        ${band('p5', 'p95', '#00e5ff')}
        ${band('p25', 'p75', '#00e5ff')}
        ${line('p50', '#00ffaa', 2)}
        ${meanLine}
        <line x1="${padL}" y1="${baseY}" x2="${padL + innerW}" y2="${baseY}" stroke="#666" stroke-dasharray="3,3"/>
        <text x="${padL + 4}" y="${baseY - 4}" fill="#9aa0c8" font-size="10">${esc(t('view.equity_forecast.chart.starting', { eq: fmt(r.starting_equity, 0) }))}</text>
        ${labels}
        <text x="${padL + innerW / 2}" y="${H - 8}" text-anchor="middle" fill="#9aa0c8" font-size="11">${esc(t('chart.series.trade_num'))}</text>
        <g transform="translate(${padL + 8}, ${padT + 14})">
            <rect width="10" height="10" fill="#00e5ff" opacity="0.35"/><text x="14" y="9" fill="#cfd2e8" font-size="10">${esc(t('view.equity_forecast.legend.band_90'))}</text>
            <rect y="14" width="10" height="10" fill="#00e5ff" opacity="0.55"/><text x="14" y="23" fill="#cfd2e8" font-size="10">${esc(t('view.equity_forecast.legend.band_50'))}</text>
            <line x1="0" y1="32" x2="10" y2="32" stroke="#00ffaa" stroke-width="2"/><text x="14" y="35" fill="#cfd2e8" font-size="10">${esc(t('view.equity_forecast.legend.median'))}</text>
            <line x1="0" y1="46" x2="10" y2="46" stroke="#ffd24a" stroke-dasharray="3,2"/><text x="14" y="49" fill="#cfd2e8" font-size="10">${esc(t('view.equity_forecast.legend.mean'))}</text>
        </g>
    </svg>`;
}

function spaghettiSvg(r) {
    if (!r.sample_paths.length) return '<p data-i18n="view.equity_forecast.hint.no_sample_paths" class="muted small">no sample paths</p>';
    const W = 1000, H = 240, padL = 60, padR = 10, padT = 10, padB = 24;
    const innerW = W - padL - padR, innerH = H - padT - padB;
    const allY = r.sample_paths.flat().concat([r.starting_equity]);
    const yMin = Math.min(...allY) * 0.95;
    const yMax = Math.max(...allY) * 1.05;
    const stepsTotal = r.sample_paths[0]?.length || 1;
    const sx = (k) => padL + (k / Math.max(stepsTotal - 1, 1)) * innerW;
    const sy = (v) => padT + (1 - (v - yMin) / Math.max(yMax - yMin, 1e-9)) * innerH;
    const baseY = sy(r.starting_equity);
    const paths = r.sample_paths.map(p => {
        const final = p[p.length - 1];
        const color = final >= 2 * r.starting_equity ? '#00ffaa'
                    : final <= 0 ? '#ff1f7a' : '#9aa0c8';
        const d = p.map((v, i) => (i ? 'L' : 'M') + sx(i) + ',' + sy(v)).join(' ');
        return `<path d="${d}" stroke="${color}" stroke-width="0.7" fill="none" opacity="0.55"/>`;
    }).join('');
    return `<svg viewBox="0 0 ${W} ${H}" width="100%" style="display:block;">
        <rect x="${padL}" y="${padT}" width="${innerW}" height="${innerH}" fill="#0d0d22" stroke="#222"/>
        ${paths}
        <line x1="${padL}" y1="${baseY}" x2="${padL + innerW}" y2="${baseY}" stroke="#666" stroke-dasharray="3,3"/>
        <text x="${padL + 4}" y="${baseY - 4}" fill="#9aa0c8" font-size="10">${esc(t('view.equity_forecast.chart.starting', { eq: fmt(r.starting_equity, 0) }))}</text>
    </svg>`;
}
