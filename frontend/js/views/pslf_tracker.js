// Public Service Loan Forgiveness tracker. 120 qualifying payments
// under an IDR plan while employed by an eligible public-service /
// 501(c)(3) employer → remaining federal balance forgiven tax-free.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderPslfTracker(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pslf_tracker.title">// PSLF TRACKER</span></h1>
        <p class="muted small" data-i18n-html="view.pslf_tracker.intro">
            Public Service Loan Forgiveness — established by the College Cost Reduction
            and Access Act of 2007. Make <strong>120 qualifying monthly payments</strong>
            (not consecutive) under an income-driven repayment plan while employed by an
            eligible <strong>government / 501(c)(3) non-profit</strong> employer. After
            the 120th payment is verified, remaining federal loan balance is forgiven
            <strong>tax-free</strong>.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.pslf_tracker.field.made">Qualifying payments made</span>
                    <input type="number" id="ps-made" step="1" min="0" max="200" value="60" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.pslf_tracker.field.balance">Current balance $</span>
                    <input type="number" id="ps-balance" step="1000" min="0" value="80000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.pslf_tracker.field.apr">APR %</span>
                    <input type="number" id="ps-apr" step="0.25" min="0" max="30" value="6.5" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.pslf_tracker.field.payment">Monthly IDR payment $</span>
                    <input type="number" id="ps-payment" step="25" min="0" value="350" style="width:100%"></label>
                <label style="display:flex;align-items:center;gap:6px">
                    <input type="checkbox" id="ps-eligible" checked> <span data-i18n="view.pslf_tracker.field.eligible">Currently at eligible employer</span>
                </label>
            </div>
            <button class="btn btn-sm primary" id="ps-run" data-shortcut="r" data-i18n="view.pslf_tracker.btn.run">⚡ Compute PSLF</button>
            <div id="ps-result"></div>
        </div>
    `;
    mount.querySelector('#ps-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#ps-result');
    const input = {
        qualifying_payments_made: parseInt(mount.querySelector('#ps-made').value, 10) || 0,
        current_balance_usd: parseFloat(mount.querySelector('#ps-balance').value) || 0,
        apr_pct: parseFloat(mount.querySelector('#ps-apr').value) || 0,
        monthly_payment_usd: parseFloat(mount.querySelector('#ps-payment').value) || 0,
        currently_eligible_employer: mount.querySelector('#ps-eligible').checked,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.pslf_tracker.status.computing'))}</p>`;
    try {
        const r = await api('/pslf-tracker/compute', { method: 'POST', body: JSON.stringify(input) });
        const stCls = r.status === 'complete' || r.status === 'on_track' ? 'pos'
                    : r.status === 'paused' || r.status === 'ineligible_employer' ? 'neg' : '';
        const pctDone = (r.qualifying_payments_made / 120 * 100).toFixed(1);
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.pslf_tracker.field.progress'))}</div>
                    <strong style="font-size:1.4em">${r.qualifying_payments_made} / 120</strong>
                    <div class="muted small">${pctDone}%</div></div>
                <div><div class="muted small">${esc(t('view.pslf_tracker.field.remaining'))}</div>
                    <strong>${r.payments_remaining}</strong></div>
                <div><div class="muted small">${esc(t('view.pslf_tracker.field.years'))}</div>
                    <strong>${r.years_to_forgiveness.toFixed(1)}</strong></div>
                <div><div class="muted small">${esc(t('view.pslf_tracker.field.total_paid'))}</div>
                    <strong>$${r.total_paid_until_forgiveness_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.pslf_tracker.field.forgiven'))}</div>
                    <strong class="pos">$${r.projected_forgiven_balance_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.pslf_tracker.field.status'))}</div>
                    <strong class="${stCls}" style="text-transform:uppercase">${esc(t('view.pslf_tracker.status.' + r.status) || r.status)}</strong></div>
            </div>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
