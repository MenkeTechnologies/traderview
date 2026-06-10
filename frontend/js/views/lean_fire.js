// Lean FIRE — minimalist retirement variant (≤ $40k annual expenses).
// Validates the expense plan is within lean tier and reports FI
// number + years-to-target.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderLeanFire(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lean_fire.title">// LEAN FIRE</span></h1>
        <p class="muted small" data-i18n-html="view.lean_fire.intro">
            <strong>Lean FIRE</strong> = FIRE at minimalist spending — typically annual
            expenses ≤ <strong>$40k</strong>. Smaller portfolio target, faster timeline,
            less margin if expenses surprise. Tiers: <strong>ultralean</strong> ≤ $25k,
            <strong>lean</strong> ≤ $40k, <strong>borderline</strong> ≤ $55k, otherwise
            <strong>not lean</strong>.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.lean_fire.field.nw">Current net worth $</span>
                    <input type="number" id="lf-nw" step="1000" min="0" value="200000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lean_fire.field.expenses">Annual expenses $</span>
                    <input type="number" id="lf-expenses" step="1000" min="0" value="30000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lean_fire.field.swr">SWR %</span>
                    <input type="number" id="lf-swr" step="0.25" min="0.25" max="20" value="4" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lean_fire.field.return">Real return %/yr</span>
                    <input type="number" id="lf-return" step="0.25" min="-10" max="20" value="5" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lean_fire.field.contrib">Monthly contribution $</span>
                    <input type="number" id="lf-contrib" step="50" min="0" value="1500" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="lf-run" data-shortcut="r" data-i18n="view.lean_fire.btn.run">⚡ Compute Lean FI</button>
            <div id="lf-result"></div>
        </div>
    `;
    mount.querySelector('#lf-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#lf-result');
    const input = {
        current_nw_usd: parseFloat(mount.querySelector('#lf-nw').value) || 0,
        annual_expenses_usd: parseFloat(mount.querySelector('#lf-expenses').value) || 0,
        safe_withdrawal_rate_pct: parseFloat(mount.querySelector('#lf-swr').value) || 4,
        expected_real_return_pct: parseFloat(mount.querySelector('#lf-return').value) || 5,
        monthly_contribution_usd: parseFloat(mount.querySelector('#lf-contrib').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.lean_fire.status.computing'))}</p>`;
    try {
        const r = await api('/lean-fire/compute', { method: 'POST', body: JSON.stringify(input) });
        const fiCls = r.is_lean_fi ? 'pos' : 'neg';
        const tierCls = r.expense_tier === 'ultralean' || r.expense_tier === 'lean' ? 'pos'
                       : r.expense_tier === 'not_lean' ? 'neg' : '';
        const deltaCls = r.current_vs_fi_delta_usd >= 0 ? 'pos' : 'neg';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.lean_fire.field.fi_number'))}</div>
                    <strong style="font-size:1.4em">$${(r.fi_number_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.lean_fire.field.delta'))}</div>
                    <strong class="${deltaCls}">${r.current_vs_fi_delta_usd >= 0 ? '+' : '−'}$${(Math.abs(r.current_vs_fi_delta_usd) / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.lean_fire.field.is_fi'))}</div>
                    <strong class="${fiCls}">${r.is_lean_fi ? '✓ ' + esc(t('view.lean_fire.status.lean_fi')) : '✗ ' + esc(t('view.lean_fire.status.not_yet'))}</strong></div>
                <div><div class="muted small">${esc(t('view.lean_fire.field.years_to'))}</div>
                    <strong>${r.years_to_fi == null ? '∞' : r.years_to_fi.toFixed(1) + 'y'}</strong></div>
                <div><div class="muted small">${esc(t('view.lean_fire.field.tier'))}</div>
                    <strong class="${tierCls}" style="text-transform:uppercase">${esc(t('view.lean_fire.tier.' + r.expense_tier) || r.expense_tier)}</strong></div>
                <div><div class="muted small">${esc(t('view.lean_fire.field.cap'))}</div>
                    <strong>$${r.upper_lean_threshold_usd.toFixed(0)}/yr</strong></div>
            </div>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
