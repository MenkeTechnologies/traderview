// Coast FIRE calculator. Point at which your portfolio compounds
// to your FI number on its own with NO further contributions —
// "I'm done saving for retirement, just covering today's expenses
// from now on".

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderCoastFire(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.coast_fire.title">// COAST FIRE</span></h1>
        <p class="muted small" data-i18n-html="view.coast_fire.intro">
            <strong>Coast FI</strong> is the point at which your portfolio is large enough
            to compound on its own — no new contributions needed — to your full FI number
            by your target retirement age. Once you hit Coast FI you can stop saving for
            retirement; you still need income to cover today's expenses but compounding
            handles the long-term work.
            <code>Coast_FI_today = FI_number / (1+r)^years_until_retirement</code>.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.coast_fire.field.nw">Current net worth $</span>
                    <input type="number" id="cf-nw" step="1000" min="0" value="100000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.coast_fire.field.age">Current age</span>
                    <input type="number" id="cf-age" step="1" min="1" max="110" value="30" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.coast_fire.field.retire">Target retirement age</span>
                    <input type="number" id="cf-retire" step="1" min="2" max="110" value="65" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.coast_fire.field.expenses">Annual expenses $</span>
                    <input type="number" id="cf-expenses" step="1000" min="0" value="40000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.coast_fire.field.swr">SWR %</span>
                    <input type="number" id="cf-swr" step="0.25" min="0.25" max="20" value="4" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.coast_fire.field.return">Real return %/yr</span>
                    <input type="number" id="cf-return" step="0.25" min="-10" max="20" value="5" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.coast_fire.field.contrib">Monthly contribution $</span>
                    <input type="number" id="cf-contrib" step="50" min="0" value="1000" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="cf-run" data-shortcut="r" data-i18n="view.coast_fire.btn.run">⚡ Compute Coast FI</button>
            <div id="cf-result"></div>
        </div>
    `;
    mount.querySelector('#cf-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#cf-result');
    const input = {
        current_nw_usd: parseFloat(mount.querySelector('#cf-nw').value) || 0,
        current_age: parseInt(mount.querySelector('#cf-age').value, 10) || 0,
        target_retirement_age: parseInt(mount.querySelector('#cf-retire').value, 10) || 0,
        annual_expenses_usd: parseFloat(mount.querySelector('#cf-expenses').value) || 0,
        safe_withdrawal_rate_pct: parseFloat(mount.querySelector('#cf-swr').value) || 4,
        expected_real_return_pct: parseFloat(mount.querySelector('#cf-return').value) || 5,
        current_monthly_contribution_usd: parseFloat(mount.querySelector('#cf-contrib').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.coast_fire.status.computing'))}</p>`;
    try {
        const r = await api('/coast-fire/compute', { method: 'POST', body: JSON.stringify(input) });
        const coastCls = r.is_coast_fi ? 'pos' : 'neg';
        const deltaCls = r.current_vs_coast_delta_usd >= 0 ? 'pos' : 'neg';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.coast_fire.field.fi_number'))}</div>
                    <strong>$${(r.fi_number_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.coast_fire.field.coast_today'))}</div>
                    <strong style="font-size:1.4em">$${(r.coast_fi_today_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.coast_fire.field.delta'))}</div>
                    <strong class="${deltaCls}">${r.current_vs_coast_delta_usd >= 0 ? '+' : '−'}$${(Math.abs(r.current_vs_coast_delta_usd) / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.coast_fire.field.is_coast'))}</div>
                    <strong class="${coastCls}" style="text-transform:uppercase">${r.is_coast_fi ? '✓ ' + esc(t('view.coast_fire.status.coasting')) : '✗ ' + esc(t('view.coast_fire.status.not_yet'))}</strong></div>
                <div><div class="muted small">${esc(t('view.coast_fire.field.projected'))}</div>
                    <strong>$${(r.projected_nw_at_retirement_no_contributions_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.coast_fire.field.years_to_coast'))}</div>
                    <strong>${r.years_until_coast_fi == null ? '∞' : r.years_until_coast_fi.toFixed(1) + 'y'}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.coast_fire.h2.per_year'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.coast_fire.th.age">Age</th>
                    <th data-i18n="view.coast_fire.th.coast">Coast number required</th>
                </tr></thead>
                <tbody>${(r.per_year_coast_required || []).map(y => `
                    <tr>
                        <td>${y.age}</td>
                        <td>$${(y.coast_required_usd / 1000).toFixed(1)}K</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
