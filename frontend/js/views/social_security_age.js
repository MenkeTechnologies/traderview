// Social Security claiming-age optimizer. Compare two claim ages
// using SSA's reduction (early) and DRC (delayed) formulas; report
// lifetime totals to life-expectancy + breakeven age.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderSocialSecurityAge(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.social_security_age.title">// SOCIAL SECURITY AGE</span></h1>
        <p class="muted small" data-i18n-html="view.social_security_age.intro">
            Compare two Social Security claiming ages using SSA's standard reduction /
            delayed-retirement-credit formulas. Early claim (62-FRA): first 36 months
            early reduce 5/9% / month, beyond 5/12% / month. Delayed claim (FRA-70):
            <strong>8% per year</strong> DRC up to age 70 (no further increase). Reports
            monthly + annual + lifetime total at each claim age, plus the breakeven age
            where cumulative benefits cross.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.social_security_age.field.fra_benefit">FRA monthly benefit $</span>
                    <input type="number" id="ss-fra-ben" step="50" min="0" value="2500" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.social_security_age.field.fra_age">FRA age</span>
                    <input type="number" id="ss-fra-age" step="1" min="62" max="70" value="67" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.social_security_age.field.claim_a">Claim age A</span>
                    <input type="number" id="ss-a" step="1" min="62" max="75" value="62" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.social_security_age.field.claim_b">Claim age B</span>
                    <input type="number" id="ss-b" step="1" min="62" max="75" value="70" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.social_security_age.field.life">Life expectancy age</span>
                    <input type="number" id="ss-life" step="1" min="62" max="120" value="85" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="ss-run" data-shortcut="r" data-i18n="view.social_security_age.btn.run">⚡ Compare Claim Ages</button>
            <div id="ss-result"></div>
        </div>
    `;
    mount.querySelector('#ss-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#ss-result');
    const input = {
        fra_monthly_benefit_usd: parseFloat(mount.querySelector('#ss-fra-ben').value) || 0,
        fra_age: parseInt(mount.querySelector('#ss-fra-age').value, 10) || 67,
        claim_age_a: parseInt(mount.querySelector('#ss-a').value, 10) || 62,
        claim_age_b: parseInt(mount.querySelector('#ss-b').value, 10) || 70,
        life_expectancy_age: parseInt(mount.querySelector('#ss-life').value, 10) || 85,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.social_security_age.status.computing'))}</p>`;
    try {
        const r = await api('/social-security-age/compute', { method: 'POST', body: JSON.stringify(input) });
        const winnerLabel = r.net_winner_at_life_expectancy === 'claim_a'
            ? `Claim @ ${r.claim_a.claim_age}`
            : r.net_winner_at_life_expectancy === 'claim_b'
                ? `Claim @ ${r.claim_b.claim_age}`
                : 'tied';
        const claimRow = c => `
            <tr>
                <td><strong>${c.claim_age}</strong></td>
                <td>$${c.monthly_benefit_usd.toFixed(0)}/mo</td>
                <td>$${(c.annual_benefit_usd / 1000).toFixed(1)}K/yr</td>
                <td>${c.pct_of_fra.toFixed(1)}%</td>
                <td><strong>$${(c.lifetime_total_to_life_expectancy_usd / 1000).toFixed(0)}K</strong></td>
            </tr>
        `;
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.social_security_age.field.breakeven'))}</div>
                    <strong style="font-size:1.4em">${r.breakeven_age == null ? '—' : 'age ' + r.breakeven_age}</strong></div>
                <div><div class="muted small">${esc(t('view.social_security_age.field.winner'))}</div>
                    <strong class="pos" style="font-size:1.2em">${esc(winnerLabel)}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.social_security_age.h2.compare'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.social_security_age.th.age">Claim age</th>
                    <th data-i18n="view.social_security_age.th.monthly">Monthly</th>
                    <th data-i18n="view.social_security_age.th.annual">Annual</th>
                    <th data-i18n="view.social_security_age.th.pct_fra">% of FRA</th>
                    <th data-i18n="view.social_security_age.th.lifetime">Lifetime total</th>
                </tr></thead>
                <tbody>
                    ${claimRow(r.claim_a)}
                    ${claimRow(r.claim_b)}
                </tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
