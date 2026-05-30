// Fill quality analytics — bar-level slippage approximation since we
// don't cache intraday bid/ask. Shows fill-in-range %, slippage vs HLC/3
// typical price, and groupings by symbol / size / hour-of-day.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { applyUiI18n, t } from '../i18n.js';

export async function renderFillQuality(mount, state) {
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) { mount.innerHTML = `<p data-i18n="view.fill_quality.hint.no_account_selected" class="boot">No account selected.</p>`; return; }
    mount.innerHTML = `
        <h1 class="view-title">${esc(t('view.fill_quality.h1', { broker: acct.broker, name: acct.name }))}</h1>
        <p class="muted small" data-i18n="view.fill_quality.hint.intro">Bar-level approximation — TraderView caches daily OHLC, not intraday bid/ask, so this is a directional metric not tick-perfect slippage. fill-in-range places the fill on the day's high-low line; the fill-efficiency column flips that for buys (lower = better) and sells (higher = better) so 100% always means "best possible fill within the day's range". Slippage bps = deviation from typical price (HLC/3), sign-flipped so positive = worse than typical for that side.</p>

        <div id="fq-out"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
    `;
    try {
        const r = await api.fillQuality(acct.id);
        if (!viewIsCurrent(tok)) return;
        render(r, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#fq-out');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function render(r, mount) {
    const o = r.overall;
    const effCls   = o.avg_fill_efficiency >= 0.6 ? 'pos' : o.avg_fill_efficiency >= 0.4 ? '' : 'neg';
    const slipCls  = o.avg_slippage_bps <= 5 ? 'pos' : o.avg_slippage_bps <= 25 ? '' : 'neg';
    const el = mount.querySelector('#fq-out');
    if (!el) return;
    el.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.fill_quality.card.sampled_fills">Sampled fills</div>
                <div class="value">${o.samples}</div>
                ${r.skipped_no_bar > 0 ? `<div class="small muted">${esc(t('view.fill_quality.card.skipped_no_bar', { count: r.skipped_no_bar }))}</div>` : ''}
            </div>
            <div class="card"><div class="label" data-i18n="view.fill_quality.card.avg_efficiency">Avg fill efficiency</div>
                <div class="value ${effCls}">${(o.avg_fill_efficiency * 100).toFixed(1)}%</div>
                <div class="small muted">${esc(t('view.fill_quality.card.efficiency_legend'))}</div></div>
            <div class="card"><div class="label" data-i18n="view.fill_quality.card.avg_slippage">Avg slippage</div>
                <div class="value ${slipCls}">${(o.avg_slippage_bps >= 0 ? '+' : '') + o.avg_slippage_bps.toFixed(1)} bps</div>
                <div class="small muted">${esc(t('view.fill_quality.card.slippage_legend'))}</div></div>
            <div class="card"><div class="label" data-i18n="view.fill_quality.card.median_slippage">Median slippage</div>
                <div class="value">${o.median_slippage_bps.toFixed(1)} bps</div></div>
            <div class="card"><div class="label" data-i18n="view.fill_quality.card.worst_best">Worst / Best</div>
                <div class="value small"><span class="neg">+${o.worst_slippage_bps.toFixed(1)}</span> /
                    <span class="pos">${o.best_slippage_bps.toFixed(1)}</span></div></div>
        </div>

        <div class="panel-grid" style="display:grid;grid-template-columns:1fr 1fr;gap:10px;">
            ${bucketPanel(t('view.fill_quality.panel.by_symbol'), r.by_symbol.slice(0, 15))}
            ${bucketPanel(t('view.fill_quality.panel.by_size'), r.by_size)}
        </div>
        ${bucketPanel(t('view.fill_quality.panel.by_hour'), r.by_hour_et)}

        <div class="chart-panel">
            <h2 data-i18n="view.fill_quality.h2.hour_chart">Avg slippage (bps) by hour-of-day</h2>
            <div id="fq-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.fill_quality.h2.latest_50_fills">Latest 50 fills</h2>
            ${sampleTable(r.samples.slice(0, 50))}
        </div>
    `;
    try { applyUiI18n(el); } catch (_) {}
    renderHourChart(r.by_hour_et);
}

function renderHourChart(byHour) {
    const el = document.getElementById('fq-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const buckets = (byHour || []).filter(b => Number.isFinite(Number(b.avg_slippage_bps)));
    if (buckets.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.fill_quality.empty_chart">${esc(t('view.fill_quality.empty_chart'))}</div>`;
        return;
    }
    const labels = buckets.map(b => b.key);
    const ys = buckets.map(b => Number(b.avg_slippage_bps));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.fill_quality.chart.hour') },
            { label: t('view.fill_quality.chart.slip_bps'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.fill_quality.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function bucketPanel(title, rows) {
    return `<div class="chart-panel">
        <h2>${esc(title)}</h2>
        ${rows.length === 0
            ? '<p data-i18n="view.fill_quality.hint.no_data" class="muted small">no data</p>'
            : `<table class="trades">
                <thead><tr>
                    <th data-i18n="view.fill_quality.th.bucket">Bucket</th><th>N</th><th data-i18n="view.fill_quality.th.avg_eff">Avg eff</th>
                    <th data-i18n="view.fill_quality.th.avg_slip_bps">Avg slip (bps)</th><th data-i18n="view.fill_quality.th.median">Median</th><th data-i18n="view.fill_quality.th.worst">Worst</th>
                </tr></thead>
                <tbody>
                ${rows.map(b => {
                    const slipCls = b.avg_slippage_bps <= 5 ? 'pos' :
                                    b.avg_slippage_bps <= 25 ? '' : 'neg';
                    return `<tr>
                        <td>${esc(b.key)}</td>
                        <td>${b.samples}</td>
                        <td>${(b.avg_fill_efficiency * 100).toFixed(1)}%</td>
                        <td class="${slipCls}">${(b.avg_slippage_bps >= 0 ? '+' : '') + b.avg_slippage_bps.toFixed(1)}</td>
                        <td>${b.median_slippage_bps.toFixed(1)}</td>
                        <td class="neg">${b.worst_slippage_bps.toFixed(1)}</td>
                    </tr>`;
                }).join('')}
                </tbody>
            </table>`}
    </div>`;
}

function sampleTable(samples) {
    if (!samples.length) return '<p data-i18n="view.fill_quality.hint.no_fills" class="muted small">no fills</p>';
    return `<table class="trades">
        <thead><tr>
            <th data-i18n="view.fill_quality.th.when">When</th><th data-i18n="view.fill_quality.th.symbol">Symbol</th><th data-i18n="view.fill_quality.th.side">Side</th><th data-i18n="view.fill_quality.th.qty">Qty</th>
            <th data-i18n="view.fill_quality.th.fill">Fill</th><th data-i18n="view.fill_quality.th.day_o_h_l_c">Day O/H/L/C</th>
            <th data-i18n="view.fill_quality.th.typical">Typical</th><th data-i18n="view.fill_quality.th.in_range">In-range</th><th data-i18n="view.fill_quality.th.eff">Eff</th><th data-i18n="view.fill_quality.th.slip_bps">Slip bps</th>
        </tr></thead>
        <tbody>
        ${samples.map(s => {
            const slipCls = s.slippage_bps <= 5 ? 'pos' : s.slippage_bps <= 25 ? '' : 'neg';
            return `<tr>
                <td class="small">${new Date(s.executed_at).toLocaleString()}</td>
                <td>${esc(s.symbol)}</td>
                <td class="small">${esc(s.side)}</td>
                <td>${fmt(s.qty, 0)}</td>
                <td>${fmt(s.fill_price, s.fill_price < 10 ? 4 : 2)}</td>
                <td class="small muted">${fmt(s.bar_open, 2)}/${fmt(s.bar_high, 2)}/${fmt(s.bar_low, 2)}/${fmt(s.bar_close, 2)}</td>
                <td>${fmt(s.typical_price, 2)}</td>
                <td>${(s.fill_in_range * 100).toFixed(0)}%</td>
                <td class="${s.fill_efficiency >= 0.6 ? 'pos' : s.fill_efficiency >= 0.4 ? '' : 'neg'}">${(s.fill_efficiency * 100).toFixed(0)}%</td>
                <td class="${slipCls}">${(s.slippage_bps >= 0 ? '+' : '') + s.slippage_bps.toFixed(1)}</td>
            </tr>`;
        }).join('')}
        </tbody></table>`;
}
