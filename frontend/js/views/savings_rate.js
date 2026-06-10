// Savings rate + benchmark + MMM-style years-to-FI projection.
// Reports gross + net SR, benchmark label, FI number = annual expenses / SWR,
// years-to-FI at current SR, plus a 10/20/30/40/50/60/70% projection table.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderSavingsRate(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.savings_rate.title">// SAVINGS RATE + YEARS TO FI</span></h1>
        <p class="muted small" data-i18n-html="view.savings_rate.intro">
            Mr Money Mustache's shockingly-simple math of early retirement: at SR=10%
            you need ~51 years to retire, at 25% ~32 years, at 50% ~17 years, at 75% ~7
            years (5% real return + 4% SWR). Reports gross + net savings rate,
            benchmark label (elite ≥50% / excellent ≥30% / good ≥20% / ok ≥10% / poor &lt;10%),
            <strong>FI number</strong> = annual expenses ÷ SWR, years-to-FI at your current
            rate, and a sensitivity table across 10/20/30/40/50/60/70% rates.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.savings_rate.field.gross_income">Gross annual income $</span>
                    <input type="number" id="sr-gross" step="1000" min="0" value="100000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.savings_rate.field.net_income">Net annual income $</span>
                    <input type="number" id="sr-net" step="1000" min="0" value="75000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.savings_rate.field.expenses">Annual expenses $</span>
                    <input type="number" id="sr-expenses" step="1000" min="0" value="50000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.savings_rate.field.savings">Annual savings $</span>
                    <input type="number" id="sr-savings" step="1000" min="0" value="25000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.savings_rate.field.return">Expected real return %</span>
                    <input type="number" id="sr-return" step="0.5" min="-10" max="20" value="5" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.savings_rate.field.swr">SWR %</span>
                    <input type="number" id="sr-swr" step="0.25" min="0.5" max="20" value="4" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="sr-run" data-shortcut="r" data-i18n="view.savings_rate.btn.run">⚡ Compute Years to FI</button>
            <div id="sr-result"></div>
        </div>
    `;
    mount.querySelector('#sr-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#sr-result');
    const input = {
        gross_annual_income_usd: parseFloat(mount.querySelector('#sr-gross').value) || 0,
        net_annual_income_usd: parseFloat(mount.querySelector('#sr-net').value) || 0,
        annual_expenses_usd: parseFloat(mount.querySelector('#sr-expenses').value) || 0,
        annual_savings_usd: parseFloat(mount.querySelector('#sr-savings').value) || 0,
        expected_real_return_pct: parseFloat(mount.querySelector('#sr-return').value) || 5,
        safe_withdrawal_rate_pct: parseFloat(mount.querySelector('#sr-swr').value) || 4,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.savings_rate.status.computing'))}</p>`;
    try {
        const r = await api('/savings-rate/compute', { method: 'POST', body: JSON.stringify(input) });
        const benchCls = r.benchmark === 'elite' || r.benchmark === 'excellent' ? 'pos'
                       : r.benchmark === 'poor' ? 'neg' : '';
        const yrsFmt = isFinite(r.years_to_fi) ? r.years_to_fi.toFixed(1) : '∞';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.savings_rate.field.gross_sr'))}</div>
                    <strong style="font-size:1.4em">${r.gross_savings_rate_pct.toFixed(1)}%</strong></div>
                <div><div class="muted small">${esc(t('view.savings_rate.field.net_sr'))}</div>
                    <strong>${r.net_savings_rate_pct.toFixed(1)}%</strong></div>
                <div><div class="muted small">${esc(t('view.savings_rate.field.benchmark'))}</div>
                    <strong class="${benchCls}" style="text-transform:uppercase">${esc(t('view.savings_rate.benchmark.' + r.benchmark) || r.benchmark)}</strong></div>
                <div><div class="muted small">${esc(t('view.savings_rate.field.years_to_fi'))}</div>
                    <strong style="font-size:1.4em">${yrsFmt}</strong></div>
                <div><div class="muted small">${esc(t('view.savings_rate.field.fi_number'))}</div>
                    <strong>$${(r.fi_number_usd / 1000).toFixed(0)}K</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.savings_rate.h2.projection'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.savings_rate.th.sr">Savings rate</th>
                    <th data-i18n="view.savings_rate.th.years">Years to FI</th>
                </tr></thead>
                <tbody>${(r.projection || []).map(p => `
                    <tr>
                        <td>${p.savings_rate_pct.toFixed(0)}%</td>
                        <td>${isFinite(p.years_to_fi) ? p.years_to_fi.toFixed(1) : '∞'}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
