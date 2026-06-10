// CD ladder builder. Split principal equally across N rungs;
// each rung matures one term-length apart.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderCdLadder(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cd_ladder.title">// CD LADDER</span></h1>
        <p class="muted small" data-i18n-html="view.cd_ladder.intro">
            Split a lump sum equally across N CDs each maturing one year apart. As each
            rung matures, roll the proceeds into a new N-year CD at the top of the
            ladder. Annual liquidity + the higher long-term CD rate on most of the money.
            Reports per-rung interest at maturity + blended APY.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.cd_ladder.field.principal">Total principal $</span>
                    <input type="number" id="cd-principal" step="1000" min="0" value="50000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.cd_ladder.field.rungs">Number of rungs</span>
                    <input type="number" id="cd-rungs" step="1" min="1" max="30" value="5" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.cd_ladder.field.term">Term years per rung</span>
                    <input type="number" id="cd-term" step="1" min="1" max="30" value="1" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.cd_ladder.field.apys">Per-rung APYs (comma)</span>
                    <input type="text" id="cd-apys" value="4.5,4.7,4.8,4.9,5.0" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.cd_ladder.field.flat">Flat APY % (overrides)</span>
                    <input type="number" id="cd-flat" step="0.05" min="0" max="30" placeholder="leave blank" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="cd-run" data-shortcut="r" data-i18n="view.cd_ladder.btn.run">⚡ Compute Ladder</button>
            <div id="cd-result"></div>
        </div>
    `;
    mount.querySelector('#cd-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#cd-result');
    const apyStr = mount.querySelector('#cd-apys').value || '';
    const apys = apyStr.split(',').map(s => parseFloat(s.trim())).filter(n => isFinite(n));
    const flatStr = mount.querySelector('#cd-flat').value;
    const flat = flatStr ? parseFloat(flatStr) : null;
    const input = {
        total_principal_usd: parseFloat(mount.querySelector('#cd-principal').value) || 0,
        rungs: parseInt(mount.querySelector('#cd-rungs').value, 10) || 5,
        term_years_per_rung: parseInt(mount.querySelector('#cd-term').value, 10) || 1,
        per_rung_apy_pct: apys,
        flat_apy_pct: (flat != null && !isNaN(flat)) ? flat : null,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.cd_ladder.status.computing'))}</p>`;
    try {
        const r = await api.request('/cd-ladder/compute', { method: 'POST', body: JSON.stringify(input) });
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.cd_ladder.field.blended_apy'))}</div>
                    <strong style="font-size:1.4em">${r.blended_apy_pct.toFixed(2)}%</strong></div>
                <div><div class="muted small">${esc(t('view.cd_ladder.field.total_principal'))}</div>
                    <strong>$${r.total_principal_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.cd_ladder.field.total_interest'))}</div>
                    <strong class="pos">$${r.total_interest_full_ladder_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.cd_ladder.field.annual_proceeds'))}</div>
                    <strong>$${r.annual_maturity_proceeds_usd.toFixed(0)}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.cd_ladder.h2.rungs'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.cd_ladder.th.rung">Rung</th>
                    <th data-i18n="view.cd_ladder.th.maturity">Maturity (yr)</th>
                    <th data-i18n="view.cd_ladder.th.principal">Principal</th>
                    <th data-i18n="view.cd_ladder.th.apy">APY</th>
                    <th data-i18n="view.cd_ladder.th.interest">Interest at maturity</th>
                    <th data-i18n="view.cd_ladder.th.balance">Balance at maturity</th>
                </tr></thead>
                <tbody>${(r.rungs || []).map(rg => `
                    <tr>
                        <td><strong>${rg.rung}</strong></td>
                        <td>${rg.maturity_years}</td>
                        <td>$${rg.principal_usd.toFixed(0)}</td>
                        <td>${rg.apy_pct.toFixed(2)}%</td>
                        <td class="pos">$${rg.interest_at_maturity_usd.toFixed(2)}</td>
                        <td><strong>$${rg.balance_at_maturity_usd.toFixed(2)}</strong></td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
