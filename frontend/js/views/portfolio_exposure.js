// Portfolio factor exposure dashboard — decomposes current paper
// account into: total β to SPY, GICS sector concentration, single-name
// HHI, portfolio annualised vol, 1-day 95% parametric VaR. Shows you
// what factor you're loaded on before you stack the 6th tech name.

import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';

export async function renderPortfolioExposure(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.portfolio_exposure.title">// PORTFOLIO FACTOR EXPOSURE</span></h1>
        <p class="muted small" data-i18n-html="view.portfolio_exposure.intro">
            Decomposes your default paper account: total β to SPY (weighted-average of
            per-position β from 60d OLS regression), GICS sector concentration using the
            existing heatmap UNIVERSE mapping for the top S&P names, single-name HHI
            (Herfindahl-Hirschman — 1.0 = single position, 1/N = perfectly diversified
            across N), portfolio annualised vol (weighted return series, ignores
            cross-correlations), and parametric 1-day 95% VaR (1.645 × daily vol × MV).
            <strong>Correlation gate prevents stacking; this view shows what you've already stacked.</strong>
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:12px;align-items:center;margin-bottom:8px">
                <button class="btn btn-sm primary" id="pe-refresh" data-shortcut="r" data-i18n="view.portfolio_exposure.btn.refresh">⚡ Refresh</button>
                <span class="muted small" id="pe-meta"></span>
            </div>
            <div id="pe-summary"></div>
            <h2 style="margin-top:1rem" data-i18n="view.portfolio_exposure.h2.sectors">Sector concentration</h2>
            <table class="trades" id="pe-sectors">
                <thead><tr>
                    <th data-i18n="view.portfolio_exposure.th.sector">Sector</th>
                    <th data-i18n="view.portfolio_exposure.th.weight">Weight %</th>
                    <th data-i18n="view.portfolio_exposure.th.positions">Positions</th>
                </tr></thead>
                <tbody><tr><td colspan="3" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
            <h2 style="margin-top:1rem" data-i18n="view.portfolio_exposure.h2.positions">Per-position exposure</h2>
            <table class="trades" id="pe-positions">
                <thead><tr>
                    <th data-i18n="view.portfolio_exposure.th.symbol">Symbol</th>
                    <th data-i18n="view.portfolio_exposure.th.qty">Qty</th>
                    <th data-i18n="view.portfolio_exposure.th.mv">MV</th>
                    <th data-i18n="view.portfolio_exposure.th.weight">Weight %</th>
                    <th data-i18n="view.portfolio_exposure.th.sector">Sector</th>
                    <th data-i18n="view.portfolio_exposure.th.beta">β to SPY</th>
                    <th data-i18n="view.portfolio_exposure.th.vol">Vol % (ann.)</th>
                </tr></thead>
                <tbody><tr><td colspan="7" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#pe-refresh').addEventListener('click', () => fetchAndRender(mount));
    await fetchAndRender(mount);
}

async function fetchAndRender(mount) {
    const summary = mount.querySelector('#pe-summary');
    const sectTbody = mount.querySelector('#pe-sectors tbody');
    const posTbody = mount.querySelector('#pe-positions tbody');
    const meta = mount.querySelector('#pe-meta');
    summary.innerHTML = `<p class="muted">${esc(t('view.portfolio_exposure.status.computing'))}</p>`;
    if (meta) meta.textContent = '';
    try {
        const r = await api('/portfolio-exposure');
        if (!r || r.total_market_value === 0 || !r.positions.length) {
            summary.innerHTML = `<p class="muted">${esc(t('view.portfolio_exposure.empty.no_positions'))}</p>`;
            sectTbody.innerHTML = '';
            posTbody.innerHTML = '';
            return;
        }
        const betaCls = r.portfolio_beta_to_spy == null ? 'muted'
            : Math.abs(r.portfolio_beta_to_spy) >= 1.2 ? 'neg'
            : Math.abs(r.portfolio_beta_to_spy) <= 0.7 ? 'pos' : '';
        const hhiCls = r.single_name_hhi >= 0.25 ? 'neg' : r.single_name_hhi >= 0.10 ? '' : 'pos';
        const volStr = r.portfolio_vol_pct_annualised == null
            ? '—' : r.portfolio_vol_pct_annualised.toFixed(2) + '%';
        const varStr = r.var_95_1day_usd == null
            ? '—' : '$' + r.var_95_1day_usd.toFixed(0);
        summary.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px">
                <div><div class="muted small">${esc(t('view.portfolio_exposure.field.total_mv'))}</div>
                    <strong>$${r.total_market_value.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.portfolio_exposure.field.beta'))}</div>
                    <strong class="${betaCls}">${r.portfolio_beta_to_spy == null ? '—' : r.portfolio_beta_to_spy.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.portfolio_exposure.field.hhi'))}</div>
                    <strong class="${hhiCls}">${r.single_name_hhi.toFixed(3)}</strong></div>
                <div><div class="muted small">${esc(t('view.portfolio_exposure.field.vol'))}</div>
                    <strong>${volStr}</strong></div>
                <div><div class="muted small">${esc(t('view.portfolio_exposure.field.var'))}</div>
                    <strong class="neg">${varStr}</strong></div>
                <div><div class="muted small">${esc(t('view.portfolio_exposure.field.positions_count'))}</div>
                    <strong>${r.positions.length}</strong></div>
            </div>
        `;
        if (!r.sector_weights.length) {
            sectTbody.innerHTML = `<tr><td colspan="3" class="muted">${esc(t('view.portfolio_exposure.empty.no_sectors'))}</td></tr>`;
        } else {
            sectTbody.innerHTML = r.sector_weights.map(s => {
                const cls = s.weight_pct >= 50 ? 'neg' : s.weight_pct >= 25 ? '' : 'pos';
                return `<tr>
                    <td><strong>${esc(s.sector)}</strong></td>
                    <td class="${cls}"><strong>${s.weight_pct.toFixed(1)}%</strong></td>
                    <td>${s.position_count}</td>
                </tr>`;
            }).join('');
        }
        posTbody.innerHTML = r.positions.map(p => {
            const betaStr = p.beta_to_spy == null ? '—' : p.beta_to_spy.toFixed(2);
            const betaCls = p.beta_to_spy == null ? 'muted'
                : Math.abs(p.beta_to_spy) >= 1.5 ? 'neg' : '';
            const volStr = p.realized_vol_pct_annualised == null
                ? '—' : p.realized_vol_pct_annualised.toFixed(1) + '%';
            return `<tr>
                <td><strong>${esc(p.symbol)}</strong></td>
                <td>${p.qty.toFixed(0)}</td>
                <td>$${p.market_value.toFixed(0)}</td>
                <td>${p.weight_pct.toFixed(1)}%</td>
                <td class="muted small">${esc(p.sector || '—')}</td>
                <td class="${betaCls}">${betaStr}</td>
                <td>${volStr}</td>
            </tr>`;
        }).join('');
        if (meta) meta.textContent = t('view.portfolio_exposure.meta.computed').replace('{t}', fmtDateTime(new Date().toISOString()));
    } catch (e) {
        summary.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
