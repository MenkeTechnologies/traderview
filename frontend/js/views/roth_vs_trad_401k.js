// Roth vs Traditional 401(k) decision. Apples-to-apples by investing
// the Traditional's tax savings in a taxable side account.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderRothVsTrad401k(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.roth_vs_trad_401k.title">// ROTH VS TRADITIONAL 401(K)</span></h1>
        <p class="muted small" data-i18n-html="view.roth_vs_trad_401k.intro">
            Apples-to-apples Roth vs Traditional 401(k) decision. Both paths assume the same
            <strong>pre-tax</strong> contribution amount. Traditional defers tax → grows
            tax-deferred → taxed at retirement marginal rate. Roth: contribution is
            after-tax (smaller) → grows tax-free → no tax at withdrawal. We model the
            Traditional's tax savings as invested in a <strong>taxable side account</strong>
            (subject to LTCG at exit) to keep contributions equivalent. Breakeven retire
            tax rate = where the two paths land at the same after-tax dollars.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.roth_vs_trad_401k.field.contribution">Annual pre-tax contribution $</span>
                    <input type="number" id="rt-contrib" step="500" min="0" value="22500" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.roth_vs_trad_401k.field.curr_tax">Current marginal tax %</span>
                    <input type="number" id="rt-curr" step="1" min="0" max="60" value="32" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.roth_vs_trad_401k.field.retire_tax">Retirement marginal tax %</span>
                    <input type="number" id="rt-retire" step="1" min="0" max="60" value="22" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.roth_vs_trad_401k.field.return">Expected return %/yr</span>
                    <input type="number" id="rt-return" step="0.25" min="-20" max="30" value="7" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.roth_vs_trad_401k.field.years">Years to retirement</span>
                    <input type="number" id="rt-years" step="1" min="1" max="70" value="30" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.roth_vs_trad_401k.field.ltcg">LTCG rate %</span>
                    <input type="number" id="rt-ltcg" step="1" min="0" max="60" value="15" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="rt-run" data-shortcut="r" data-i18n="view.roth_vs_trad_401k.btn.run">⚡ Compare</button>
            <div id="rt-result"></div>
        </div>
    `;
    mount.querySelector('#rt-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#rt-result');
    const input = {
        annual_pretax_contribution_usd: parseFloat(mount.querySelector('#rt-contrib').value) || 0,
        current_marginal_tax_rate_pct: parseFloat(mount.querySelector('#rt-curr').value) || 0,
        retirement_marginal_tax_rate_pct: parseFloat(mount.querySelector('#rt-retire').value) || 0,
        expected_annual_return_pct: parseFloat(mount.querySelector('#rt-return').value) || 0,
        years_to_retirement: parseInt(mount.querySelector('#rt-years').value, 10) || 30,
        ltcg_rate_pct: parseFloat(mount.querySelector('#rt-ltcg').value) || 15,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.roth_vs_trad_401k.status.computing'))}</p>`;
    try {
        const r = await api('/roth-vs-trad-401k/compute', { method: 'POST', body: JSON.stringify(input) });
        const winCls = r.net_winner === 'traditional' ? 'pos' : r.net_winner === 'roth' ? 'pos' : '';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.roth_vs_trad_401k.field.trad_total'))}</div>
                    <strong>$${(r.traditional_after_tax_total_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.roth_vs_trad_401k.field.roth_total'))}</div>
                    <strong>$${(r.roth_after_tax_total_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.roth_vs_trad_401k.field.winner'))}</div>
                    <strong class="${winCls}" style="font-size:1.4em;text-transform:uppercase">${esc(t('view.roth_vs_trad_401k.winner.' + r.net_winner) || r.net_winner)}</strong></div>
                <div><div class="muted small">${esc(t('view.roth_vs_trad_401k.field.advantage'))}</div>
                    <strong class="pos">$${(r.winner_advantage_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.roth_vs_trad_401k.field.breakeven_rate'))}</div>
                    <strong>${r.breakeven_retirement_tax_rate_pct.toFixed(1)}%</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.roth_vs_trad_401k.h2.detail'))}</h2>
            <table class="trades">
                <tbody>
                    <tr><td><strong>${esc(t('view.roth_vs_trad_401k.row.trad_pretax'))}</strong></td>
                        <td>$${(r.traditional_pretax_balance_usd / 1000).toFixed(0)}K</td></tr>
                    <tr><td><strong>${esc(t('view.roth_vs_trad_401k.row.trad_side_pretax'))}</strong></td>
                        <td>$${(r.traditional_side_account_balance_usd / 1000).toFixed(0)}K</td></tr>
                    <tr><td><strong>${esc(t('view.roth_vs_trad_401k.row.trad_side_after_ltcg'))}</strong></td>
                        <td>$${(r.traditional_side_account_after_ltcg_usd / 1000).toFixed(0)}K</td></tr>
                </tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
