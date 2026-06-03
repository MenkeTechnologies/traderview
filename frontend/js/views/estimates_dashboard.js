// Analyst Estimates Dashboard — combines all 8 Finnhub estimate endpoints
// into one comparable view: Revenue / EBITDA / EBIT / EPS / Net income /
// Pretax income / Gross income / DPS, switchable annual vs quarterly.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const FREQS = ['annual', 'quarterly'];
const METRICS = [
    { key: 'revenue',  fn: 'symbolRevenueEstimate',     label: 'Revenue' },
    { key: 'ebitda',   fn: 'symbolEbitdaEstimate',      label: 'EBITDA' },
    { key: 'ebit',     fn: 'symbolEbitEstimate',        label: 'EBIT' },
    { key: 'eps',      fn: 'symbolEpsEstimate',         label: 'EPS' },
    { key: 'netInc',   fn: 'symbolNetIncomeEstimate',   label: 'Net income' },
    { key: 'pretax',   fn: 'symbolPretaxIncomeEstimate',label: 'Pretax income' },
    { key: 'gross',    fn: 'symbolGrossIncomeEstimate', label: 'Gross income' },
    { key: 'dps',      fn: 'symbolDpsEstimate',         label: 'DPS' },
];

let state = { symbol: '', freq: 'annual' };

export async function renderEstimatesDashboard(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.estimates.h1.title">// ANALYST ESTIMATES</span></h1>
        <p class="muted small" data-i18n="view.estimates.hint.intro">
            Wall Street consensus estimates: Revenue, EBITDA, EBIT, EPS, Net income,
            Pretax income, Gross income, Dividends per share — switchable annual /
            quarterly. Compare against company-reported actuals for setup quality.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="est-form">
                <label><span data-i18n="view.estimates.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="MSFT" required></label>
                <label><span data-i18n="view.estimates.label.freq">Frequency</span>
                    <select name="freq">${FREQS.map(f =>
                        `<option value="${f}" ${f === state.freq ? 'selected' : ''}>${esc(t('view.estimates.freq.' + f))}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" type="submit" data-i18n="view.estimates.btn.load">Load</button>
            </form>
        </div>
        <div class="panel-grid" id="est-grid"></div>
    `;
    document.getElementById('est-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        state.freq = fd.get('freq') || 'annual';
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const grid = document.getElementById('est-grid');
    if (grid) grid.innerHTML = METRICS.map(m =>
        `<div class="chart-panel">
            <h2>${esc(t('view.estimates.metric.' + m.key))}</h2>
            <div id="est-${m.key}"><div class="boot">${esc(t('common.loading'))}</div></div>
        </div>`
    ).join('');

    const results = await Promise.all(
        METRICS.map(m => api[m.fn](state.symbol, state.freq).catch(() => null))
    );
    if (!viewIsCurrent(tok)) return;
    METRICS.forEach((m, i) => renderMetric(m, results[i]));
}

function renderMetric(meta, data) {
    const el = document.getElementById('est-' + meta.key);
    if (!el) return;
    const rows = data?.data || (Array.isArray(data) ? data : []);
    if (!rows.length) {
        el.innerHTML = `<p class="muted">${esc(t('view.estimates.empty'))}</p>`;
        return;
    }
    const sorted = [...rows].sort((a, b) =>
        String(a.period || '').localeCompare(String(b.period || '')));
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.estimates.th.period">Period</th>
            <th data-i18n="view.estimates.th.high">High</th>
            <th data-i18n="view.estimates.th.avg">Avg</th>
            <th data-i18n="view.estimates.th.low">Low</th>
            <th data-i18n="view.estimates.th.analysts"># An.</th>
        </tr></thead>
        <tbody>${sorted.slice(-12).reverse().map(r => `
            <tr>
                <td>${esc(r.period || '—')}</td>
                <td class="pos">${fmt(r.epsHigh ?? r.revenueHigh ?? r.high)}</td>
                <td>${fmt(r.epsAvg ?? r.revenueAvg ?? r.average ?? r.avg)}</td>
                <td class="neg">${fmt(r.epsLow ?? r.revenueLow ?? r.low)}</td>
                <td class="muted">${r.numberAnalysts ?? '—'}</td>
            </tr>
        `).join('')}</tbody>
    </table>`;
}

function fmt(v) {
    if (v == null || !Number.isFinite(Number(v))) return '—';
    const n = Number(v);
    if (Math.abs(n) >= 1e9) return (n / 1e9).toFixed(2) + 'B';
    if (Math.abs(n) >= 1e6) return (n / 1e6).toFixed(2) + 'M';
    if (Math.abs(n) >= 1e3) return (n / 1e3).toFixed(1) + 'k';
    return n.toFixed(2);
}
