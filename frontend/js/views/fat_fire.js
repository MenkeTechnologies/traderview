// Fat FIRE — high-spend retirement (≥ $100k/yr expenses).
// Conservative 3.5% SWR default for fat tier's extra-margin needs.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderFatFire(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.fat_fire.title">// FAT FIRE</span></h1>
        <p class="muted small" data-i18n-html="view.fat_fire.intro">
            <strong>Fat FIRE</strong> = high-spend retirement, typically annual expenses
            ≥ <strong>$100k</strong>. Bigger portfolio target, longer timeline, more
            margin for inflation / sequence-of-returns risk. Default uses
            <strong>3.5% SWR</strong> (more conservative than the canonical 4%) for the
            extra-margin needs. Tiers: <strong>not fat</strong> &lt; $80k,
            <strong>borderline</strong> &lt; $100k, <strong>fat</strong> &lt; $250k,
            <strong>obese</strong> ≥ $250k.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.fat_fire.field.nw">Current net worth $</span>
                    <input type="number" id="ff-nw" step="10000" min="0" value="500000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.fat_fire.field.expenses">Annual expenses $</span>
                    <input type="number" id="ff-expenses" step="5000" min="0" value="150000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.fat_fire.field.swr">SWR %</span>
                    <input type="number" id="ff-swr" step="0.25" min="0.25" max="20" value="3.5" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.fat_fire.field.return">Real return %/yr</span>
                    <input type="number" id="ff-return" step="0.25" min="-10" max="20" value="5" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.fat_fire.field.contrib">Monthly contribution $</span>
                    <input type="number" id="ff-contrib" step="100" min="0" value="5000" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="ff-run" data-shortcut="r" data-i18n="view.fat_fire.btn.run">⚡ Compute Fat FI</button>
            <div id="ff-result"></div>
        </div>
    `;
    mount.querySelector('#ff-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#ff-result');
    const input = {
        current_nw_usd: parseFloat(mount.querySelector('#ff-nw').value) || 0,
        annual_expenses_usd: parseFloat(mount.querySelector('#ff-expenses').value) || 0,
        safe_withdrawal_rate_pct: parseFloat(mount.querySelector('#ff-swr').value) || 3.5,
        expected_real_return_pct: parseFloat(mount.querySelector('#ff-return').value) || 5,
        monthly_contribution_usd: parseFloat(mount.querySelector('#ff-contrib').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.fat_fire.status.computing'))}</p>`;
    try {
        const r = await api('/fat-fire/compute', { method: 'POST', body: JSON.stringify(input) });
        const fiCls = r.is_fat_fi ? 'pos' : 'neg';
        const tierCls = r.expense_tier === 'fat' || r.expense_tier === 'obese' ? 'pos'
                       : r.expense_tier === 'not_fat' ? 'neg' : '';
        const deltaCls = r.current_vs_fi_delta_usd >= 0 ? 'pos' : 'neg';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.fat_fire.field.fi_number'))}</div>
                    <strong style="font-size:1.4em">$${(r.fi_number_usd / 1000000).toFixed(2)}M</strong></div>
                <div><div class="muted small">${esc(t('view.fat_fire.field.delta'))}</div>
                    <strong class="${deltaCls}">${r.current_vs_fi_delta_usd >= 0 ? '+' : '−'}$${(Math.abs(r.current_vs_fi_delta_usd) / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.fat_fire.field.is_fi'))}</div>
                    <strong class="${fiCls}">${r.is_fat_fi ? '✓ ' + esc(t('view.fat_fire.status.fat_fi')) : '✗ ' + esc(t('view.fat_fire.status.not_yet'))}</strong></div>
                <div><div class="muted small">${esc(t('view.fat_fire.field.years_to'))}</div>
                    <strong>${r.years_to_fi == null ? '∞' : r.years_to_fi.toFixed(1) + 'y'}</strong></div>
                <div><div class="muted small">${esc(t('view.fat_fire.field.tier'))}</div>
                    <strong class="${tierCls}" style="text-transform:uppercase">${esc(t('view.fat_fire.tier.' + r.expense_tier) || r.expense_tier)}</strong></div>
                <div><div class="muted small">${esc(t('view.fat_fire.field.threshold'))}</div>
                    <strong>$${r.lower_fat_threshold_usd.toFixed(0)}/yr</strong></div>
            </div>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
