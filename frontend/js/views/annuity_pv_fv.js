// Annuity present-value / future-value calculator.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderAnnuityPvFv(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.annuity_pv_fv.title">// ANNUITY PV / FV</span></h1>
        <p class="muted small" data-i18n-html="view.annuity_pv_fv.intro">
            Time-value-of-money for a level annuity (equal periodic payments). Computes
            <strong>PV</strong> (what the stream is worth today) and <strong>FV</strong>
            (what it grows to). <strong>Ordinary</strong>: payment at end of period
            (loans, bond coupons). <strong>Due</strong>: payment at start (rent, lease);
            multiplies PV/FV by (1 + r).
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.annuity_pv_fv.field.payment">Payment per period $</span>
                    <input type="number" id="ap-pmt" step="50" min="0" value="1000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.annuity_pv_fv.field.rate">Annual rate %</span>
                    <input type="number" id="ap-rate" step="0.25" min="-50" max="50" value="6" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.annuity_pv_fv.field.periods">Periods per year</span>
                    <input type="number" id="ap-periods" step="1" min="1" max="365" value="12" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.annuity_pv_fv.field.years">Years</span>
                    <input type="number" id="ap-years" step="1" min="0" max="100" value="10" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.annuity_pv_fv.field.kind">Annuity kind</span>
                    <select id="ap-kind">
                        <option value="ordinary" selected>Ordinary (end of period)</option>
                        <option value="due">Due (start of period)</option>
                    </select>
                </label>
            </div>
            <button class="btn btn-sm primary" id="ap-run" data-shortcut="r" data-i18n="view.annuity_pv_fv.btn.run">⚡ Compute</button>
            <div id="ap-result"></div>
        </div>
    `;
    mount.querySelector('#ap-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#ap-result');
    const input = {
        payment_per_period_usd: parseFloat(mount.querySelector('#ap-pmt').value) || 0,
        annual_rate_pct: parseFloat(mount.querySelector('#ap-rate').value) || 0,
        periods_per_year: parseInt(mount.querySelector('#ap-periods').value, 10) || 12,
        years: parseFloat(mount.querySelector('#ap-years').value) || 0,
        annuity_kind: mount.querySelector('#ap-kind').value,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.annuity_pv_fv.status.computing'))}</p>`;
    try {
        const r = await api.request('/annuity-pv-fv/compute', { method: 'POST', body: JSON.stringify(input) });
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.annuity_pv_fv.field.n'))}</div>
                    <strong>${r.n_periods.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.annuity_pv_fv.field.per_rate'))}</div>
                    <strong>${(r.periodic_rate * 100).toFixed(4)}%</strong></div>
                <div><div class="muted small">${esc(t('view.annuity_pv_fv.field.pv'))}</div>
                    <strong style="font-size:1.4em">$${r.present_value_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.annuity_pv_fv.field.fv'))}</div>
                    <strong style="font-size:1.4em">$${r.future_value_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.annuity_pv_fv.field.total_payments'))}</div>
                    <strong>$${r.total_payments_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.annuity_pv_fv.field.interest_pv'))}</div>
                    <strong class="neg">$${r.total_interest_pv_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.annuity_pv_fv.field.interest_fv'))}</div>
                    <strong class="pos">$${r.total_interest_fv_usd.toFixed(0)}</strong></div>
            </div>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
