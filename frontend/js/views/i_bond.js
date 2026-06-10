// Series I savings bond calculator.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderIBond(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.i_bond.title">// SERIES I SAVINGS BOND</span></h1>
        <p class="muted small" data-i18n-html="view.i_bond.intro">
            US Series I savings bond (TreasuryDirect). Composite rate =
            <code>fixed + 2 × semi_inflation + (fixed × semi_inflation)</code>. Rate set
            every May 1 / Nov 1. Holding rules: <strong>12-month lockup</strong> (no
            redemption), <strong>1-5 years</strong> = 3-month interest penalty on
            redemption, <strong>5+ years</strong> = no penalty, <strong>30-year cap</strong>
            (stops earning).
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.i_bond.field.amount">Purchase amount $</span>
                    <input type="number" id="ib-amt" step="500" min="0" value="10000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.i_bond.field.fixed">Fixed rate %</span>
                    <input type="number" id="ib-fixed" step="0.05" min="-10" max="20" value="1.30" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.i_bond.field.semi">Semi-annual inflation %</span>
                    <input type="number" id="ib-semi" step="0.05" min="-20" max="20" value="1.97" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.i_bond.field.months">Holding period months</span>
                    <input type="number" id="ib-months" step="1" min="0" max="600" value="60" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="ib-run" data-shortcut="r" data-i18n="view.i_bond.btn.run">⚡ Compute I-Bond</button>
            <div id="ib-result"></div>
        </div>
    `;
    mount.querySelector('#ib-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#ib-result');
    const input = {
        purchase_amount_usd: parseFloat(mount.querySelector('#ib-amt').value) || 0,
        fixed_rate_pct: parseFloat(mount.querySelector('#ib-fixed').value) || 0,
        semi_annual_inflation_pct: parseFloat(mount.querySelector('#ib-semi').value) || 0,
        holding_period_months: parseInt(mount.querySelector('#ib-months').value, 10) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.i_bond.status.computing'))}</p>`;
    try {
        const r = await api.request('/i-bond/compute', { method: 'POST', body: JSON.stringify(input) });
        const stCls = r.penalty_status === 'none' ? 'pos' : r.penalty_status === 'locked' ? 'neg' : '';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.i_bond.field.composite'))}</div>
                    <strong style="font-size:1.4em">${r.composite_rate_pct.toFixed(3)}%</strong></div>
                <div><div class="muted small">${esc(t('view.i_bond.field.months_capped'))}</div>
                    <strong>${r.holding_period_months}</strong></div>
                <div><div class="muted small">${esc(t('view.i_bond.field.final_pre'))}</div>
                    <strong>$${r.final_value_before_penalty_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.i_bond.field.interest'))}</div>
                    <strong class="pos">$${r.interest_earned_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.i_bond.field.penalty'))}</div>
                    <strong class="${r.early_withdrawal_penalty_usd > 0 ? 'neg' : 'pos'}">$${r.early_withdrawal_penalty_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.i_bond.field.net'))}</div>
                    <strong style="font-size:1.4em">$${r.net_value_after_penalty_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.i_bond.field.status'))}</div>
                    <strong class="${stCls}" style="text-transform:uppercase">${esc(t('view.i_bond.status.' + r.penalty_status) || r.penalty_status)}</strong></div>
            </div>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
