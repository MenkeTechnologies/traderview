// Sector Heatmap — Finnhub /sector/metrics.
// Visualizes sector-level performance + valuation metrics for rotation analysis.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const REGIONS = [
    { value: 'NA', label: 'North America' },
    { value: 'EU', label: 'Europe' },
    { value: 'AS', label: 'Asia' },
    { value: 'GBL', label: 'Global' },
];

let state = { region: 'NA' };

export async function renderSectorHeatmap(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sector_heatmap.h1.title">// SECTOR HEATMAP</span></h1>
        <p class="muted small" data-i18n="view.sector_heatmap.hint.intro">
            Sector-level performance + valuation metrics from Finnhub. Use the per-metric
            sort to find rotation candidates (best YTD return, lowest P/E, highest div).
        </p>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.sector_heatmap.label.region">Region</span>
                    <select id="sh-region">${REGIONS.map(r =>
                        `<option value="${r.value}" ${r.value === state.region ? 'selected' : ''}>${esc(r.label)}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" id="sh-refresh" type="button" data-i18n="view.sector_heatmap.btn.refresh">Refresh</button>
            </div>
            <div id="sh-grid" style="margin-top:10px"></div>
            <div id="sh-table" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('sh-region').addEventListener('change', e => {
        state.region = e.target.value;
        void load(tok);
    });
    document.getElementById('sh-refresh').addEventListener('click', () => void load(tok));
    await load(tok);
}

async function load(tok) {
    const grid = document.getElementById('sh-grid');
    const tableEl = document.getElementById('sh-table');
    if (grid) grid.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const data = await api.finnhubSectorMetrics(state.region);
        if (!viewIsCurrent(tok)) return;
        const sectors = data?.data || [];
        if (!sectors.length) {
            grid.innerHTML = `<p class="muted" data-i18n="view.sector_heatmap.empty">No sector metrics for this region (may require premium).</p>`;
            if (tableEl) tableEl.innerHTML = '';
            return;
        }
        renderHeatmapTiles(grid, sectors);
        renderTable(tableEl, sectors);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (grid) grid.innerHTML = `<p class="muted neg">${esc(t('view.sector_heatmap.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.sector_heatmap.toast.failed'), { level: 'error' });
    }
}

function renderHeatmapTiles(el, sectors) {
    if (!el) return;
    // Use ytd return as the primary heat dimension; fall back to dividend yield.
    el.innerHTML = `<div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(180px,1fr));gap:6px">${sectors.map(s => {
        const m = s.metrics || {};
        const ytd = Number(m.priceUsingTtmEpsAnnualYoy || m.priceChangeTTM || m['52WeekPriceReturnDaily'] || 0);
        const cls = ytd >= 5 ? 'pos' : ytd <= -5 ? 'neg' : '';
        return `<div class="tile" style="padding:10px;border:1px solid var(--border);${cls === 'pos' ? 'background:rgba(0,255,128,0.06)' : cls === 'neg' ? 'background:rgba(255,64,64,0.06)' : ''}">
            <div style="font-weight:600">${esc(s.sector || '—')}</div>
            <div class="${cls}" style="font-size:1.4em;font-weight:600;margin-top:4px">
                ${ytd >= 0 ? '+' : ''}${ytd.toFixed(1)}%
            </div>
            <div class="muted small" style="margin-top:2px">
                P/E ${m.peTTM != null ? Number(m.peTTM).toFixed(1) : '—'}
                · DY ${m.dividendYieldIndicatedAnnual != null ? Number(m.dividendYieldIndicatedAnnual).toFixed(2) + '%' : '—'}
            </div>
        </div>`;
    }).join('')}</div>`;
}

function renderTable(el, sectors) {
    if (!el) return;
    const rows = sectors.map(s => ({
        sector: s.sector || '—',
        metrics: s.metrics || {},
    }));
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.sector_heatmap.th.sector">Sector</th>
            <th data-i18n="view.sector_heatmap.th.pe">P/E TTM</th>
            <th data-i18n="view.sector_heatmap.th.pb">P/B</th>
            <th data-i18n="view.sector_heatmap.th.ps">P/S</th>
            <th data-i18n="view.sector_heatmap.th.div_yield">Div %</th>
            <th data-i18n="view.sector_heatmap.th.roe">ROE</th>
            <th data-i18n="view.sector_heatmap.th.margin">Net margin</th>
            <th data-i18n="view.sector_heatmap.th.debt_eq">Debt/Eq</th>
        </tr></thead>
        <tbody>${rows.map(r => {
            const m = r.metrics;
            return `<tr>
                <td>${esc(r.sector)}</td>
                <td>${num(m.peTTM)}</td>
                <td>${num(m.pbAnnual)}</td>
                <td>${num(m.psTTM)}</td>
                <td>${pct(m.dividendYieldIndicatedAnnual)}</td>
                <td>${pct(m.roeTTM)}</td>
                <td>${pct(m.netProfitMarginTTM)}</td>
                <td>${num(m['totalDebt/totalEquityAnnual'])}</td>
            </tr>`;
        }).join('')}</tbody>
    </table>`;
}

function num(v) {
    if (v == null || !Number.isFinite(Number(v))) return '—';
    return Number(v).toFixed(2);
}
function pct(v) {
    if (v == null || !Number.isFinite(Number(v))) return '—';
    return Number(v).toFixed(2) + '%';
}
