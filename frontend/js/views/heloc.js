// HELOC calculator. Variable-rate revolving line tied to home equity,
// with draw phase (interest-only or % of balance) and repayment phase
// (amortizing). Reports utilization, draw-phase payments, projected
// repayment-phase amortization, total lifetime interest, and status.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderHeloc(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.heloc.title">// HELOC CALCULATOR</span></h1>
        <p class="muted small" data-i18n-html="view.heloc.intro">
            Home Equity Line of Credit — revolving credit tied to home equity. Typical
            structure: 10-year <strong>draw phase</strong> (minimum is usually
            interest-only or 1-2% of balance) followed by 20-year <strong>repayment
            phase</strong> where balance amortizes at the current variable rate. Variable
            APR floats with prime; the report assumes the current rate persists for the
            projection.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.heloc.field.line">Line size $</span>
                    <input type="number" id="hc-line" step="5000" min="0" value="100000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.heloc.field.balance">Current balance $</span>
                    <input type="number" id="hc-balance" step="1000" min="0" value="50000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.heloc.field.apr">Variable APR %</span>
                    <input type="number" id="hc-apr" step="0.125" min="0" max="30" value="8.5" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.heloc.field.draw_months">Draw period months</span>
                    <input type="number" id="hc-draw" step="12" min="1" max="360" value="120" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.heloc.field.repay_months">Repayment period months</span>
                    <input type="number" id="hc-repay" step="12" min="1" max="480" value="240" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.heloc.field.min_pct">Draw min % of balance</span>
                    <input type="number" id="hc-minpct" step="0.25" min="0" max="10" value="0" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.heloc.field.voluntary">Voluntary principal $/mo</span>
                    <input type="number" id="hc-vol" step="50" min="0" value="0" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="hc-run" data-shortcut="r" data-i18n="view.heloc.btn.run">⚡ Compute HELOC</button>
            <div id="hc-result"></div>
        </div>
    `;
    mount.querySelector('#hc-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#hc-result');
    const input = {
        line_size_usd: parseFloat(mount.querySelector('#hc-line').value) || 0,
        current_balance_usd: parseFloat(mount.querySelector('#hc-balance').value) || 0,
        variable_apr_pct: parseFloat(mount.querySelector('#hc-apr').value) || 0,
        draw_period_months: parseInt(mount.querySelector('#hc-draw').value, 10) || 120,
        repayment_period_months: parseInt(mount.querySelector('#hc-repay').value, 10) || 240,
        draw_phase_min_pct: parseFloat(mount.querySelector('#hc-minpct').value) || 0,
        monthly_voluntary_principal_usd: parseFloat(mount.querySelector('#hc-vol').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.heloc.status.computing'))}</p>`;
    try {
        const r = await api.request('/heloc/compute', { method: 'POST', body: JSON.stringify(input) });
        const utilCls = r.utilization_pct > 80 ? 'neg' : r.utilization_pct < 10 ? 'pos' : '';
        const stCls = r.status === 'maxed' ? 'neg' : r.status === 'underutilized' || r.status === 'principal_reducing' ? 'pos' : '';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.heloc.field.util'))}</div>
                    <strong class="${utilCls}" style="font-size:1.4em">${r.utilization_pct.toFixed(1)}%</strong></div>
                <div><div class="muted small">${esc(t('view.heloc.field.status'))}</div>
                    <strong class="${stCls}" style="text-transform:uppercase">${esc(t('view.heloc.status.' + r.status) || r.status)}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.heloc.h2.draw'))}</h2>
            <table class="trades">
                <tbody>
                    <tr><td><strong>${esc(t('view.heloc.field.monthly_interest'))}</strong></td>
                        <td>$${r.draw_phase_monthly_interest_usd.toFixed(2)}/mo</td></tr>
                    <tr><td><strong>${esc(t('view.heloc.field.min_pay'))}</strong></td>
                        <td>$${r.draw_phase_min_payment_usd.toFixed(2)}/mo</td></tr>
                    <tr><td><strong>${esc(t('view.heloc.field.total_pay'))}</strong></td>
                        <td>$${r.draw_phase_total_payment_usd.toFixed(2)}/mo</td></tr>
                    <tr><td><strong>${esc(t('view.heloc.field.draw_interest'))}</strong></td>
                        <td class="neg">$${r.draw_phase_total_interest_usd.toFixed(0)}</td></tr>
                </tbody>
            </table>
            <h2 style="margin-top:1rem">${esc(t('view.heloc.h2.repay'))}</h2>
            <table class="trades">
                <tbody>
                    <tr><td><strong>${esc(t('view.heloc.field.repay_balance'))}</strong></td>
                        <td>$${r.repayment_phase_balance_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.heloc.field.repay_pi'))}</strong></td>
                        <td>$${r.repayment_phase_monthly_pi_usd.toFixed(2)}/mo</td></tr>
                    <tr><td><strong>${esc(t('view.heloc.field.repay_interest'))}</strong></td>
                        <td class="neg">$${r.repayment_phase_total_interest_usd.toFixed(0)}</td></tr>
                </tbody>
            </table>
            <h2 style="margin-top:1rem">${esc(t('view.heloc.h2.total'))}</h2>
            <div><strong class="neg" style="font-size:1.4em">$${r.total_lifetime_interest_usd.toFixed(0)}</strong></div>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
