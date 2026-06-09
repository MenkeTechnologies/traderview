// Historical Market Cap — Finnhub /stock/historical-market-cap.
// Shows year-over-year evolution to flag dilution vs. growth.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const HORIZONS = [
    { value: 365,  key: 'view.hist_mc.horizon.1y' },
    { value: 730,  key: 'view.hist_mc.horizon.2y' },
    { value: 1825, key: 'view.hist_mc.horizon.5y' },
];

let state = { symbol: '', horizon: 730 };

export async function renderHistoricalMarketCap(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.hist_mc.h1.title">// HISTORICAL MARKET CAP</span></h1>
        <p class="muted small" data-i18n="view.hist_mc.hint.intro">
            Market cap over time. Sharp drops without comparable price drops = dilution.
            Useful for spotting failing companies issuing shares to stay afloat.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="hmc-form">
                <label><span data-i18n="view.hist_mc.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="AAPL" required></label>
                <label><span data-i18n="view.hist_mc.label.horizon">Horizon</span>
                    <select name="horizon">${HORIZONS.map(h =>
                        `<option value="${h.value}" ${h.value === state.horizon ? 'selected' : ''}>${esc(t(h.key))}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" type="submit" data-i18n="view.hist_mc.btn.load">Load</button>
            </form>
            <div id="hmc-chart" style="width:100%;height:300px;margin-top:10px"></div>
            <div id="hmc-summary" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('hmc-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        state.horizon = Number(fd.get('horizon') || 730);
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const chartEl = document.getElementById('hmc-chart');
    const sumEl = document.getElementById('hmc-summary');
    if (sumEl) sumEl.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const to = new Date();
        const from = new Date(to);
        from.setDate(from.getDate() - state.horizon);
        const data = await api.symbolHistoricalMarketCap(state.symbol, fmtDay(from), fmtDay(to));
        if (!viewIsCurrent(tok)) return;
        const rows = data?.data || [];
        if (!rows.length) {
            if (sumEl) sumEl.innerHTML = `<p class="muted" data-i18n="view.hist_mc.empty">No historical market cap data.</p>`;
            if (chartEl) chartEl.innerHTML = '';
            return;
        }
        const sorted = [...rows].sort((a, b) =>
            String(a.atDate || '').localeCompare(String(b.atDate || '')));
        const first = sorted[0];
        const last = sorted[sorted.length - 1];
        const pct = first.marketCapitalization > 0
            ? ((last.marketCapitalization - first.marketCapitalization) / first.marketCapitalization) * 100
            : null;
        if (sumEl) {
            sumEl.innerHTML = `
                <div class="cards">
                    <div class="card"><div class="label" data-i18n="view.hist_mc.card.first">First</div>
                        <div class="value">${fmtMC(first.marketCapitalization)} <span class="muted small">${esc(first.atDate || '')}</span></div></div>
                    <div class="card"><div class="label" data-i18n="view.hist_mc.card.last">Last</div>
                        <div class="value">${fmtMC(last.marketCapitalization)} <span class="muted small">${esc(last.atDate || '')}</span></div></div>
                    <div class="card ${pct == null ? '' : pct >= 0 ? 'pos' : 'neg'}">
                        <div class="label" data-i18n="view.hist_mc.card.change">Change</div>
                        <div class="value">${pct != null ? (pct >= 0 ? '+' : '') + pct.toFixed(1) + '%' : '—'}</div></div>
                </div>
            `;
        }
        if (chartEl && window.uPlot) {
            chartEl.innerHTML = '';
            const xs = sorted.map(r => new Date(r.atDate).getTime() / 1000);
            const ys = sorted.map(r => Number(r.marketCapitalization) || 0);
            new window.uPlot({
                title: '', width: chartEl.clientWidth || 800, height: 300,
                scales: { x: { time: true }, y: { auto: true } },
                series: [
                    {},
                    { label: t('view.hist_mc.label.market_cap'),
                      stroke: '#00e5ff', width: 1.5, fill: 'rgba(0,229,255,0.10)' },
                ],
                axes: [{ stroke: '#aab', size: 28 }, {
                    stroke: '#aab', size: 60,
                    values: (_u, splits) => splits.map(fmtMC),
                }],
                legend: { show: false },
            }, [xs, ys], chartEl);
        }
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (sumEl) sumEl.innerHTML = `<p class="muted neg">${esc(t('view.hist_mc.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.hist_mc.toast.failed'), { level: 'error' });
    }
}

function fmtDay(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}
function fmtMC(v) {
    if (!Number.isFinite(v)) return '—';
    if (Math.abs(v) >= 1e12) return '$' + (v / 1e12).toFixed(2) + 'T';
    if (Math.abs(v) >= 1e9)  return '$' + (v / 1e9).toFixed(2) + 'B';
    if (Math.abs(v) >= 1e6)  return '$' + (v / 1e6).toFixed(2) + 'M';
    return '$' + v.toFixed(0);
}
