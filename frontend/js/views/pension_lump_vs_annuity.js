// Pension lump-sum vs annuity decision. PV(annuity) at expected_return,
// implied IRR where PV = lump, runs-out year, leftover at life expectancy,
// net winner.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderPensionLumpVsAnnuity(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pension_lump_vs_annuity.title">// PENSION LUMP VS ANNUITY</span></h1>
        <p class="muted small" data-i18n-html="view.pension_lump_vs_annuity.intro">
            Pension buyout decision: take a <strong>lump sum</strong> today (you invest
            and draw down) or take a <strong>monthly annuity</strong> for life. Compares
            on a <strong>present-value</strong> basis at your expected return; reports the
            implied internal rate where the annuity stream equals the lump, the year the
            lump would run out if you withdraw at the annuity-equivalent rate, and what's
            left at life expectancy.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.pension_lump_vs_annuity.field.lump">Lump sum $</span>
                    <input type="number" id="pl-lump" step="5000" min="0" value="500000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.pension_lump_vs_annuity.field.monthly">Monthly annuity $</span>
                    <input type="number" id="pl-monthly" step="100" min="0" value="3000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.pension_lump_vs_annuity.field.age">Current age</span>
                    <input type="number" id="pl-age" step="1" min="1" max="110" value="65" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.pension_lump_vs_annuity.field.life">Life expectancy</span>
                    <input type="number" id="pl-life" step="1" min="2" max="120" value="85" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.pension_lump_vs_annuity.field.return">Real return %/yr</span>
                    <input type="number" id="pl-return" step="0.25" min="-20" max="30" value="5" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="pl-run" data-shortcut="r" data-i18n="view.pension_lump_vs_annuity.btn.run">⚡ Compare</button>
            <div id="pl-result"></div>
        </div>
    `;
    mount.querySelector('#pl-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#pl-result');
    const input = {
        lump_sum_usd: parseFloat(mount.querySelector('#pl-lump').value) || 0,
        monthly_annuity_usd: parseFloat(mount.querySelector('#pl-monthly').value) || 0,
        current_age: parseInt(mount.querySelector('#pl-age').value, 10) || 0,
        life_expectancy_age: parseInt(mount.querySelector('#pl-life').value, 10) || 0,
        expected_real_return_pct: parseFloat(mount.querySelector('#pl-return').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.pension_lump_vs_annuity.status.computing'))}</p>`;
    try {
        const r = await api.request('/pension-lump-vs-annuity/compute', { method: 'POST', body: JSON.stringify(input) });
        const winCls = 'pos';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.pension_lump_vs_annuity.field.lump_disp'))}</div>
                    <strong>$${(r.lump_sum_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.pension_lump_vs_annuity.field.pv_annuity'))}</div>
                    <strong>$${(r.annuity_present_value_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.pension_lump_vs_annuity.field.implied'))}</div>
                    <strong>${r.implied_internal_rate_pct == null ? '—' : r.implied_internal_rate_pct.toFixed(2) + '%'}</strong></div>
                <div><div class="muted small">${esc(t('view.pension_lump_vs_annuity.field.runs_out'))}</div>
                    <strong>${r.lump_runs_out_year == null ? '∞' : 'year ' + r.lump_runs_out_year}</strong></div>
                <div><div class="muted small">${esc(t('view.pension_lump_vs_annuity.field.leftover'))}</div>
                    <strong>$${(r.leftover_at_life_expectancy_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.pension_lump_vs_annuity.field.annuity_total'))}</div>
                    <strong>$${(r.annuity_total_payments_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.pension_lump_vs_annuity.field.winner'))}</div>
                    <strong class="${winCls}" style="font-size:1.4em;text-transform:uppercase">${esc(t('view.pension_lump_vs_annuity.winner.' + r.net_winner) || r.net_winner)}</strong></div>
                <div><div class="muted small">${esc(t('view.pension_lump_vs_annuity.field.advantage'))}</div>
                    <strong class="pos">$${(r.winner_advantage_usd / 1000).toFixed(1)}K</strong></div>
            </div>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
