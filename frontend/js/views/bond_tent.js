// Bond-tent / rising-equity glide path (Kitces, Pfau).

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderBondTent(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.bond_tent.title">// BOND TENT</span></h1>
        <p class="muted small" data-i18n-html="view.bond_tent.intro">
            <strong>Bond tent</strong> (Kitces / Pfau) — ramp UP bond allocation in the
            years leading into retirement to dampen <strong>sequence-of-returns
            risk</strong>, then ramp BACK DOWN after, finishing retirement at the original
            stock allocation. SORR is highest in the first ~10 years of retirement;
            holding more bonds across that window absorbs a bad early sequence without
            forcing equity sales at the bottom.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.bond_tent.field.current_age">Current age</span>
                    <input type="number" id="bt-age" step="1" min="1" max="110" value="50" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.bond_tent.field.retire_age">Retirement age</span>
                    <input type="number" id="bt-retire" step="1" min="1" max="110" value="65" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.bond_tent.field.pre">Pre-tent bond %</span>
                    <input type="number" id="bt-pre" step="5" min="0" max="100" value="30" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.bond_tent.field.peak">Tent peak bond %</span>
                    <input type="number" id="bt-peak" step="5" min="0" max="100" value="60" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.bond_tent.field.post">Post-tent bond %</span>
                    <input type="number" id="bt-post" step="5" min="0" max="100" value="30" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.bond_tent.field.ramp">Ramp-up years</span>
                    <input type="number" id="bt-ramp" step="1" min="0" max="50" value="10" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.bond_tent.field.descent">Descent years</span>
                    <input type="number" id="bt-descent" step="1" min="0" max="50" value="10" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.bond_tent.field.horizon">Horizon age</span>
                    <input type="number" id="bt-horizon" step="1" min="1" max="120" value="90" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="bt-run" data-shortcut="r" data-i18n="view.bond_tent.btn.run">⚡ Compute Glide</button>
            <div id="bt-result"></div>
        </div>
    `;
    mount.querySelector('#bt-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#bt-result');
    const input = {
        current_age: parseInt(mount.querySelector('#bt-age').value, 10) || 0,
        retirement_age: parseInt(mount.querySelector('#bt-retire').value, 10) || 0,
        pre_tent_bond_pct: parseFloat(mount.querySelector('#bt-pre').value) || 0,
        tent_peak_bond_pct: parseFloat(mount.querySelector('#bt-peak').value) || 0,
        post_tent_bond_pct: parseFloat(mount.querySelector('#bt-post').value) || 0,
        tent_ramp_years: parseInt(mount.querySelector('#bt-ramp').value, 10) || 0,
        tent_descent_years: parseInt(mount.querySelector('#bt-descent').value, 10) || 0,
        horizon_age: parseInt(mount.querySelector('#bt-horizon').value, 10) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.bond_tent.status.computing'))}</p>`;
    try {
        const r = await api.request('/bond-tent/compute', { method: 'POST', body: JSON.stringify(input) });
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.bond_tent.field.tent_start'))}</div>
                    <strong>${r.tent_start_age}</strong></div>
                <div><div class="muted small">${esc(t('view.bond_tent.field.tent_peak'))}</div>
                    <strong>${r.tent_peak_age}</strong></div>
                <div><div class="muted small">${esc(t('view.bond_tent.field.tent_end'))}</div>
                    <strong>${r.tent_end_age}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.bond_tent.h2.glide'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.bond_tent.th.age">Age</th>
                    <th data-i18n="view.bond_tent.th.bond">Bond %</th>
                    <th data-i18n="view.bond_tent.th.stock">Stock %</th>
                </tr></thead>
                <tbody>${(r.glide || []).map(p => {
                    const inTent = p.age >= r.tent_start_age && p.age <= r.tent_end_age;
                    return `<tr style="${inTent ? 'background:rgba(255,42,109,0.06)' : ''}">
                        <td>${p.age}</td>
                        <td>${p.bond_pct.toFixed(1)}%</td>
                        <td>${p.stock_pct.toFixed(1)}%</td>
                    </tr>`;
                }).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
