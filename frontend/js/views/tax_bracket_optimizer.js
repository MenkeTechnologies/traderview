// Federal marginal tax-bracket optimizer.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderTaxBracketOptimizer(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.tax_bracket_optimizer.title">// TAX BRACKET OPTIMIZER</span></h1>
        <p class="muted small" data-i18n-html="view.tax_bracket_optimizer.intro">
            See how much room is left in your current marginal bracket and how much you
            can realise (Roth conversion, IRA withdrawal, LT cap gains) before bumping
            into the next rate. Embedded 2026 IRS brackets for single / MFJ / HoH.
            Effective rate &lt; marginal because of progressive tiering.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.tax_bracket_optimizer.field.status">Filing status</span>
                    <select id="tbo-status">
                        <option value="single" selected>Single</option>
                        <option value="mfj">Married filing jointly</option>
                        <option value="hoh">Head of household</option>
                    </select>
                </label>
                <label><span class="muted small" data-i18n="view.tax_bracket_optimizer.field.income">Taxable ordinary income $</span>
                    <input type="number" id="tbo-income" step="1000" min="0" value="80000" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="tbo-run" data-shortcut="r" data-i18n="view.tax_bracket_optimizer.btn.run">⚡ Compute</button>
            <div id="tbo-result"></div>
        </div>
    `;
    mount.querySelector('#tbo-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#tbo-result');
    const input = {
        filing_status: mount.querySelector('#tbo-status').value,
        taxable_ordinary_income_usd: parseFloat(mount.querySelector('#tbo-income').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.tax_bracket_optimizer.status.computing'))}</p>`;
    try {
        const r = await api.request('/tax-bracket-optimizer/compute', { method: 'POST', body: JSON.stringify(input) });
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.tax_bracket_optimizer.field.marginal'))}</div>
                    <strong style="font-size:1.4em">${r.current_marginal_rate_pct.toFixed(0)}%</strong></div>
                <div><div class="muted small">${esc(t('view.tax_bracket_optimizer.field.room'))}</div>
                    <strong class="pos">$${r.room_in_current_bracket_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.tax_bracket_optimizer.field.next_rate'))}</div>
                    <strong>${r.next_bracket_rate_pct.toFixed(0)}%</strong></div>
                <div><div class="muted small">${esc(t('view.tax_bracket_optimizer.field.tax'))}</div>
                    <strong class="neg">$${r.federal_tax_liability_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.tax_bracket_optimizer.field.effective'))}</div>
                    <strong>${r.effective_rate_pct.toFixed(2)}%</strong></div>
                <div><div class="muted small">${esc(t('view.tax_bracket_optimizer.field.bracket_upper'))}</div>
                    <strong>${isFinite(r.current_bracket_upper_usd) ? '$' + r.current_bracket_upper_usd.toFixed(0) : '∞'}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.tax_bracket_optimizer.h2.brackets'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.tax_bracket_optimizer.th.rate">Rate</th>
                    <th data-i18n="view.tax_bracket_optimizer.th.lower">Lower</th>
                    <th data-i18n="view.tax_bracket_optimizer.th.upper">Upper</th>
                </tr></thead>
                <tbody>${(r.brackets || []).map(b => {
                    const isCurrent = b.rate_pct === r.current_marginal_rate_pct;
                    return `<tr style="${isCurrent ? 'background:rgba(255,42,109,0.08)' : ''}">
                        <td>${b.rate_pct.toFixed(0)}%</td>
                        <td>$${b.lower_usd.toFixed(0)}</td>
                        <td>${isFinite(b.upper_usd) ? '$' + b.upper_usd.toFixed(0) : '∞'}</td>
                    </tr>`;
                }).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
