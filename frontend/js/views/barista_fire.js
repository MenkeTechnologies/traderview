// Barista FIRE — portfolio covers the gap between expenses and a
// low-stress part-time income (canonical: Starbucks barista with
// healthcare). Smaller FI number than full FIRE.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderBaristaFire(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.barista_fire.title">// BARISTA FIRE</span></h1>
        <p class="muted small" data-i18n-html="view.barista_fire.intro">
            <strong>Barista FI</strong> = portfolio is large enough to cover the gap
            between your living expenses and a low-stress part-time job's after-tax income.
            Canonical example: Starbucks-barista wages but with healthcare coverage. You can
            quit the high-stress career while the portfolio + part-time income carry you to
            traditional retirement. <code>barista_fi = (expenses − part_time_income) / SWR</code>.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.barista_fire.field.nw">Current net worth $</span>
                    <input type="number" id="bf-nw" step="1000" min="0" value="200000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.barista_fire.field.age">Current age</span>
                    <input type="number" id="bf-age" step="1" min="1" max="110" value="35" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.barista_fire.field.expenses">Annual expenses $</span>
                    <input type="number" id="bf-expenses" step="1000" min="0" value="40000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.barista_fire.field.part_time">Annual part-time income $</span>
                    <input type="number" id="bf-pt" step="1000" min="0" value="25000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.barista_fire.field.swr">SWR %</span>
                    <input type="number" id="bf-swr" step="0.25" min="0.25" max="20" value="4" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.barista_fire.field.return">Real return %/yr</span>
                    <input type="number" id="bf-return" step="0.25" min="-10" max="20" value="5" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.barista_fire.field.contrib">Monthly contribution $</span>
                    <input type="number" id="bf-contrib" step="50" min="0" value="500" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="bf-run" data-shortcut="r" data-i18n="view.barista_fire.btn.run">⚡ Compute Barista FI</button>
            <div id="bf-result"></div>
        </div>
    `;
    mount.querySelector('#bf-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#bf-result');
    const input = {
        current_nw_usd: parseFloat(mount.querySelector('#bf-nw').value) || 0,
        current_age: parseInt(mount.querySelector('#bf-age').value, 10) || 0,
        annual_expenses_usd: parseFloat(mount.querySelector('#bf-expenses').value) || 0,
        annual_part_time_income_usd: parseFloat(mount.querySelector('#bf-pt').value) || 0,
        safe_withdrawal_rate_pct: parseFloat(mount.querySelector('#bf-swr').value) || 4,
        expected_real_return_pct: parseFloat(mount.querySelector('#bf-return').value) || 5,
        current_monthly_contribution_usd: parseFloat(mount.querySelector('#bf-contrib').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.barista_fire.status.computing'))}</p>`;
    try {
        const r = await api('/barista-fire/compute', { method: 'POST', body: JSON.stringify(input) });
        const stCls = r.status === 'barista_fi' ? 'pos' : r.status === 'no_gap' ? 'pos' : 'neg';
        const deltaCls = r.current_vs_barista_delta_usd >= 0 ? 'pos' : 'neg';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.barista_fire.field.full_fi'))}</div>
                    <strong>$${(r.full_fi_number_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.barista_fire.field.barista_fi'))}</div>
                    <strong style="font-size:1.4em">$${(r.barista_fi_number_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.barista_fire.field.savings'))}</div>
                    <strong class="pos">$${(r.barista_savings_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.barista_fire.field.gap'))}</div>
                    <strong>$${r.gap_annual_usd.toFixed(0)}/yr</strong></div>
                <div><div class="muted small">${esc(t('view.barista_fire.field.delta'))}</div>
                    <strong class="${deltaCls}">${r.current_vs_barista_delta_usd >= 0 ? '+' : '−'}$${(Math.abs(r.current_vs_barista_delta_usd) / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.barista_fire.field.years_to'))}</div>
                    <strong>${r.years_until_barista_fi == null ? '∞' : r.years_until_barista_fi.toFixed(1) + 'y'}</strong></div>
                <div><div class="muted small">${esc(t('view.barista_fire.field.status'))}</div>
                    <strong class="${stCls}" style="text-transform:uppercase">${esc(t('view.barista_fire.status.' + r.status) || r.status)}</strong></div>
            </div>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
