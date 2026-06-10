// All-Weather (Dalio) + Permanent Portfolio (Browne) + 60/40 + 100% S&P
// backtest comparison. Simulates monthly rebalancing across cached ETF
// bars; reports annualised return / vol / Sharpe (with 95% CI) / max DD.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderPermanentPortfolio(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.permanent_portfolio.title">// PERMANENT PORTFOLIO / ALL-WEATHER COMPARISON</span></h1>
        <p class="muted small" data-i18n-html="view.permanent_portfolio.intro">
            Backtests 4 canonical passive allocations on cached ETF bars:
            <strong>All-Weather</strong> (Dalio: 40% TLT / 30% VTI / 15% IEF / 7.5% GLD / 7.5% DBC),
            <strong>Permanent Portfolio</strong> (Browne: 25% each VTI / TLT / GLD / BIL),
            <strong>60/40</strong> (60% VTI / 40% AGG), and
            <strong>100% S&P</strong> (SPY).
            Each portfolio is simulated with monthly rebalancing. Reports
            annualised return, vol, Sharpe + 95% CI, max DD.
            Requires cached price_bars for the constituent ETFs.
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <label>
                    <span class="muted small" data-i18n="view.permanent_portfolio.field.days">Days back</span>
                    <input type="number" id="pp-days" step="365" min="365" max="7300" value="1825" style="width:100px">
                </label>
                <button class="btn btn-sm primary" id="pp-run" data-shortcut="r" data-i18n="view.permanent_portfolio.btn.run">⚡ Compare</button>
                <span class="muted small" id="pp-meta"></span>
            </div>
            <div id="pp-result"></div>
        </div>
    `;
    mount.querySelector('#pp-run').addEventListener('click', () => runCompare(mount));
    await runCompare(mount);
}

async function runCompare(mount) {
    const result = mount.querySelector('#pp-result');
    const meta = mount.querySelector('#pp-meta');
    const days = parseInt(mount.querySelector('#pp-days').value, 10) || 1825;
    result.innerHTML = `<p class="muted">${esc(t('view.permanent_portfolio.status.running'))}</p>`;
    if (meta) meta.textContent = '';
    try {
        const r = await api.request(`/permanent-portfolio/compare?days_back=${days}`);
        if (meta) meta.textContent = t('view.permanent_portfolio.meta.summary')
            .replace('{e}', r.errors.length);
        result.innerHTML = `
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.permanent_portfolio.th.portfolio">Portfolio</th>
                    <th data-i18n="view.permanent_portfolio.th.allocations">Allocations</th>
                    <th data-i18n="view.permanent_portfolio.th.n">N Months</th>
                    <th data-i18n="view.permanent_portfolio.th.ann_return">Ann. Return %</th>
                    <th data-i18n="view.permanent_portfolio.th.ann_vol">Ann. Vol %</th>
                    <th data-i18n="view.permanent_portfolio.th.sharpe">Sharpe</th>
                    <th data-i18n="view.permanent_portfolio.th.sharpe_ci">95% CI</th>
                    <th data-i18n="view.permanent_portfolio.th.max_dd">Max DD %</th>
                </tr></thead>
                <tbody>${r.portfolios.map(p => {
                    const sharpeCls = p.annualised_sharpe >= 1.0 ? 'pos' : p.annualised_sharpe >= 0.5 ? '' : 'muted';
                    const ciCls = p.sharpe_ci_lo_95 > 0 ? 'pos' : 'neg';
                    const allocStr = p.allocations.map(([s, w]) => `${s}: ${(w * 100).toFixed(1)}%`).join(', ');
                    return `<tr>
                        <td><strong>${esc(p.name)}</strong></td>
                        <td class="muted small">${esc(allocStr)}</td>
                        <td>${p.n_months}</td>
                        <td class="${p.annualised_return_pct >= 0 ? 'pos' : 'neg'}">${p.annualised_return_pct.toFixed(2)}</td>
                        <td>${p.annualised_vol_pct.toFixed(2)}</td>
                        <td class="${sharpeCls}"><strong>${p.annualised_sharpe.toFixed(2)}</strong></td>
                        <td class="${ciCls} muted small">[${p.sharpe_ci_lo_95.toFixed(2)}, ${p.sharpe_ci_hi_95.toFixed(2)}]</td>
                        <td class="neg">${p.max_drawdown_pct.toFixed(2)}</td>
                    </tr>${p.note ? `<tr><td colspan="8" class="muted small">${esc(p.note)}</td></tr>` : ''}`;
                }).join('')}</tbody>
            </table>
            ${r.errors.length ? `<details><summary class="muted small">${r.errors.length} errors</summary>
                <ul>${r.errors.slice(0, 20).map(e => `<li class="muted small">${esc(e)}</li>`).join('')}</ul></details>` : ''}
            <p class="muted small">${esc(t('view.permanent_portfolio.hint.interpret'))}</p>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
