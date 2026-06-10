// Three-fund portfolio recommender (Boglehead). US + International + Bonds.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderThreeFundPortfolio(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.three_fund_portfolio.title">// THREE-FUND PORTFOLIO</span></h1>
        <p class="muted small" data-i18n-html="view.three_fund_portfolio.intro">
            Canonical Boglehead three-fund portfolio: US stocks + International stocks +
            Bonds. Stock allocation by age + risk tolerance: <strong>aggressive</strong>
            110 − age, <strong>moderate</strong> 100 − age, <strong>conservative</strong>
            90 − age (clamped [10, 95]%). Within stocks default 70/30 US/international
            (Bogle); Vanguard TDF uses 60/40. Reports drift from target + suggested
            rebalance trades.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.three_fund_portfolio.field.age">Age</span>
                    <input type="number" id="tf-age" step="1" min="1" max="110" value="40" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.three_fund_portfolio.field.risk">Risk tolerance</span>
                    <select id="tf-risk">
                        <option value="conservative">Conservative (90 − age)</option>
                        <option value="moderate" selected>Moderate (100 − age)</option>
                        <option value="aggressive">Aggressive (110 − age)</option>
                    </select>
                </label>
                <label><span class="muted small" data-i18n="view.three_fund_portfolio.field.us_share">US share of stocks %</span>
                    <input type="number" id="tf-us-share" step="5" min="0" max="100" value="70" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.three_fund_portfolio.field.us">Current US stocks $</span>
                    <input type="number" id="tf-us" step="1000" min="0" value="100000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.three_fund_portfolio.field.intl">Current Intl stocks $</span>
                    <input type="number" id="tf-intl" step="1000" min="0" value="20000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.three_fund_portfolio.field.bonds">Current Bonds $</span>
                    <input type="number" id="tf-bonds" step="1000" min="0" value="30000" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="tf-run" data-shortcut="r" data-i18n="view.three_fund_portfolio.btn.run">⚡ Compute Allocation</button>
            <div id="tf-result"></div>
        </div>
    `;
    mount.querySelector('#tf-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#tf-result');
    const input = {
        age: parseInt(mount.querySelector('#tf-age').value, 10) || 0,
        risk_tolerance: mount.querySelector('#tf-risk').value,
        current_us_stocks_usd: parseFloat(mount.querySelector('#tf-us').value) || 0,
        current_intl_stocks_usd: parseFloat(mount.querySelector('#tf-intl').value) || 0,
        current_bonds_usd: parseFloat(mount.querySelector('#tf-bonds').value) || 0,
        us_within_stocks_pct: parseFloat(mount.querySelector('#tf-us-share').value) || 70,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.three_fund_portfolio.status.computing'))}</p>`;
    try {
        const r = await api.request('/three-fund-portfolio/compute', { method: 'POST', body: JSON.stringify(input) });
        const assetRow = (label, a) => `
            <tr>
                <td><strong>${esc(label)}</strong></td>
                <td>${a.target_weight_pct.toFixed(1)}%</td>
                <td>$${(a.target_dollar_usd / 1000).toFixed(1)}K</td>
                <td>$${(a.current_dollar_usd / 1000).toFixed(1)}K</td>
                <td>${a.current_weight_pct.toFixed(1)}%</td>
                <td class="${Math.abs(a.drift_pct) > 5 ? 'neg' : ''}">${a.drift_pct >= 0 ? '+' : ''}${a.drift_pct.toFixed(1)}%</td>
                <td class="${a.rebalance_buy_sell_usd >= 0 ? 'pos' : 'neg'}">${a.rebalance_buy_sell_usd >= 0 ? 'BUY $' : 'SELL $'}${(Math.abs(a.rebalance_buy_sell_usd) / 1000).toFixed(1)}K</td>
            </tr>
        `;
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.three_fund_portfolio.field.total'))}</div>
                    <strong style="font-size:1.4em">$${(r.total_portfolio_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.three_fund_portfolio.field.stock_target'))}</div>
                    <strong>${r.total_stock_target_pct.toFixed(0)}%</strong></div>
                <div><div class="muted small">${esc(t('view.three_fund_portfolio.field.bond_target'))}</div>
                    <strong>${r.total_bond_target_pct.toFixed(0)}%</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.three_fund_portfolio.h2.allocation'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.three_fund_portfolio.th.asset">Asset</th>
                    <th data-i18n="view.three_fund_portfolio.th.target_pct">Target %</th>
                    <th data-i18n="view.three_fund_portfolio.th.target_dollar">Target $</th>
                    <th data-i18n="view.three_fund_portfolio.th.current_dollar">Current $</th>
                    <th data-i18n="view.three_fund_portfolio.th.current_pct">Current %</th>
                    <th data-i18n="view.three_fund_portfolio.th.drift">Drift</th>
                    <th data-i18n="view.three_fund_portfolio.th.rebalance">Rebalance</th>
                </tr></thead>
                <tbody>
                    ${assetRow(t('view.three_fund_portfolio.asset.us'), r.us_stocks)}
                    ${assetRow(t('view.three_fund_portfolio.asset.intl'), r.intl_stocks)}
                    ${assetRow(t('view.three_fund_portfolio.asset.bonds'), r.bonds)}
                </tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
