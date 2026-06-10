// Dollar-cost-averaging scheduler simulator. Walks cached daily closes,
// simulates buying $N worth of symbol every period (weekly/monthly/
// quarterly), compares the realized result to lump-summing the same
// total at period 0.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderDcaSimulator(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.dca_simulator.title">// DCA SCHEDULER</span></h1>
        <p class="muted small" data-i18n-html="view.dca_simulator.intro">
            Simulates buying $N worth of a symbol every period for N years.
            Compares the realized result to lump-summing the same total at period 0.
            Lump-sum usually beats DCA in absolute return because markets generally
            go up (time-in-market wins), but DCA wins on emotional sustainability
            and avoids the timing-luck risk of "what if you lump-summed at the top."
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.dca_simulator.field.symbol">Symbol</span>
                    <input type="text" id="dca-symbol" value="SPY" style="width:80px;text-transform:uppercase">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.dca_simulator.field.contribution">Contribution $</span>
                    <input type="number" id="dca-contribution" step="100" min="10" max="1000000" value="500" style="width:100px">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.dca_simulator.field.frequency">Frequency</span>
                    <select id="dca-frequency" style="padding:4px">
                        <option value="weekly">Weekly</option>
                        <option value="monthly" selected>Monthly</option>
                        <option value="quarterly">Quarterly</option>
                    </select>
                </label>
                <label>
                    <span class="muted small" data-i18n="view.dca_simulator.field.days_back">Days back</span>
                    <input type="number" id="dca-days" step="30" min="30" max="7300" value="1825" style="width:100px">
                </label>
                <button class="btn btn-sm primary" id="dca-run" data-shortcut="r" data-i18n="view.dca_simulator.btn.run">⚡ Simulate</button>
                <span class="muted small" id="dca-meta"></span>
            </div>
            <div id="dca-summary"></div>
            <div id="dca-purchases"></div>
        </div>
    `;
    mount.querySelector('#dca-run').addEventListener('click', () => runSim(mount));
    await runSim(mount);
}

async function runSim(mount) {
    const summary = mount.querySelector('#dca-summary');
    const purchases = mount.querySelector('#dca-purchases');
    const symbol = mount.querySelector('#dca-symbol').value.trim().toUpperCase() || 'SPY';
    const contribution = parseFloat(mount.querySelector('#dca-contribution').value) || 500;
    const frequency = mount.querySelector('#dca-frequency').value || 'monthly';
    const days = parseInt(mount.querySelector('#dca-days').value, 10) || 1825;
    summary.innerHTML = `<p class="muted">${esc(t('view.dca_simulator.status.simulating'))}</p>`;
    purchases.innerHTML = '';
    try {
        const r = await api.request(`/dca-simulator/run?symbol=${symbol}&contribution_usd=${contribution}&frequency=${frequency}&days_back=${days}`);
        if (!r) {
            summary.innerHTML = `<p class="muted">${esc(t('view.dca_simulator.empty.no_data'))}</p>`;
            return;
        }
        const winnerCls = r.dca_vs_lump_ratio >= 1.0 ? 'pos' : 'neg';
        summary.innerHTML = `
            <h2>${esc(r.symbol)} · ${esc(r.frequency)} · ${r.n_purchases} purchases</h2>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px">
                <div><div class="muted small">${esc(t('view.dca_simulator.field.total_contributed'))}</div>
                    <strong>$${r.total_contributed_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.dca_simulator.field.total_shares'))}</div>
                    <strong>${r.total_shares.toFixed(3)}</strong></div>
                <div><div class="muted small">${esc(t('view.dca_simulator.field.avg_cost'))}</div>
                    <strong>$${r.avg_cost_per_share.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.dca_simulator.field.current_price'))}</div>
                    <strong>$${r.current_price.toFixed(2)}</strong></div>
            </div>
            <h3 style="margin-top:1rem">${esc(t('view.dca_simulator.h3.comparison'))}</h3>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.dca_simulator.th.strategy">Strategy</th>
                    <th data-i18n="view.dca_simulator.th.final_value">Final Value</th>
                    <th data-i18n="view.dca_simulator.th.total_return">Total Return %</th>
                    <th data-i18n="view.dca_simulator.th.ann_return">Ann. Return %</th>
                </tr></thead>
                <tbody>
                    <tr>
                        <td><strong>${esc(t('view.dca_simulator.field.dca'))}</strong></td>
                        <td>$${r.final_value_usd.toFixed(2)}</td>
                        <td class="${r.dca_total_return_pct >= 0 ? 'pos' : 'neg'}">${r.dca_total_return_pct >= 0 ? '+' : ''}${r.dca_total_return_pct.toFixed(2)}</td>
                        <td class="${r.dca_annualised_return_pct >= 0 ? 'pos' : 'neg'}">${r.dca_annualised_return_pct >= 0 ? '+' : ''}${r.dca_annualised_return_pct.toFixed(2)}</td>
                    </tr>
                    <tr>
                        <td><strong>${esc(t('view.dca_simulator.field.lump_sum'))}</strong></td>
                        <td>$${r.lump_sum_final_value_usd.toFixed(2)}</td>
                        <td class="${r.lump_sum_total_return_pct >= 0 ? 'pos' : 'neg'}">${r.lump_sum_total_return_pct >= 0 ? '+' : ''}${r.lump_sum_total_return_pct.toFixed(2)}</td>
                        <td class="${r.lump_sum_annualised_return_pct >= 0 ? 'pos' : 'neg'}">${r.lump_sum_annualised_return_pct >= 0 ? '+' : ''}${r.lump_sum_annualised_return_pct.toFixed(2)}</td>
                    </tr>
                </tbody>
            </table>
            <p class="small">${esc(t('view.dca_simulator.field.dca_vs_lump'))}:
                <strong class="${winnerCls}">${r.dca_vs_lump_ratio.toFixed(3)}×</strong>
                <span class="muted small">(${r.dca_vs_lump_ratio >= 1.0
                    ? esc(t('view.dca_simulator.hint.dca_won'))
                    : esc(t('view.dca_simulator.hint.lump_won'))})</span></p>
        `;
        const recent = r.purchases.slice(-20).reverse();
        purchases.innerHTML = `
            <h3 style="margin-top:1rem">${esc(t('view.dca_simulator.h3.purchases'))}</h3>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.dca_simulator.th.date">Date</th>
                    <th data-i18n="view.dca_simulator.th.price">Price</th>
                    <th data-i18n="view.dca_simulator.th.shares">Shares Bought</th>
                    <th data-i18n="view.dca_simulator.th.running">Running Total</th>
                </tr></thead>
                <tbody>${recent.map(p => `
                    <tr>
                        <td class="muted small">${esc(p.purchase_date)}</td>
                        <td>$${p.purchase_price.toFixed(2)}</td>
                        <td>${p.shares_bought.toFixed(3)}</td>
                        <td>${p.running_shares.toFixed(3)}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        summary.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
