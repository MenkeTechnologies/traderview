// Target-Date Fund glide path generator.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import * as enh from '../calc_enhance.js';

export async function renderGlidePath(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.glide_path.title">// GLIDE PATH</span></h1>
        <p class="muted small" data-i18n-html="view.glide_path.intro">
            Generic Target-Date Fund (TDF) glide path. Two-segment linear ramp:
            <strong>working phase</strong> (current → retirement) from start_stock_pct down
            to retire_stock_pct, then <strong>to-landing</strong> (retirement → landing_age)
            from retire down to landing_stock_pct, then <strong>post-landing</strong> flat.
            Defaults approximate Vanguard's Target Retirement glide (90 → 50 → 30 across
            ages 25 / 65 / 72).
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.glide_path.field.current_age">Current age</span>
                    <input type="number" id="gp-age" step="1" min="1" max="110" value="25" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.glide_path.field.retire_age">Retirement age</span>
                    <input type="number" id="gp-retire" step="1" min="1" max="110" value="65" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.glide_path.field.landing_age">Landing age</span>
                    <input type="number" id="gp-landing" step="1" min="1" max="120" value="72" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.glide_path.field.horizon_age">Horizon age</span>
                    <input type="number" id="gp-horizon" step="1" min="1" max="120" value="90" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.glide_path.field.start_stock">Start stock %</span>
                    <input type="number" id="gp-start" step="5" min="0" max="100" value="90" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.glide_path.field.retire_stock">Retire stock %</span>
                    <input type="number" id="gp-retire-s" step="5" min="0" max="100" value="50" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.glide_path.field.landing_stock">Landing stock %</span>
                    <input type="number" id="gp-landing-s" step="5" min="0" max="100" value="30" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="gp-run" data-shortcut="r" data-i18n="view.glide_path.btn.run">⚡ Compute Glide</button>
            <div id="gp-result"></div>
        </div>
    `;
    mount.querySelector('#gp-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#gp-result');
    const input = {
        current_age: parseInt(mount.querySelector('#gp-age').value, 10) || 0,
        retirement_age: parseInt(mount.querySelector('#gp-retire').value, 10) || 0,
        landing_age: parseInt(mount.querySelector('#gp-landing').value, 10) || 0,
        horizon_age: parseInt(mount.querySelector('#gp-horizon').value, 10) || 0,
        start_stock_pct: parseFloat(mount.querySelector('#gp-start').value) || 0,
        retire_stock_pct: parseFloat(mount.querySelector('#gp-retire-s').value) || 0,
        landing_stock_pct: parseFloat(mount.querySelector('#gp-landing-s').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.glide_path.status.computing'))}</p>`;
    try {
        const r = await api.request('/glide-path/compute', { method: 'POST', body: JSON.stringify(input) });
        const phaseCls = p => p === 'pre' || p === 'working' ? '' : p === 'to_landing' ? 'neg' : '';
        // The glide path itself — stock allocation declining with age.
        const chart = enh.svgLineChart((r.glide || []).map(p => ({ x: p.age, y: p.stock_pct })), { xlabel: 'age', ylabel: 'stock %' });
        result.innerHTML = `
            ${chart}
            <div id="gp-tools" class="ce-toolbar"></div>
            <h2 style="margin-top:1rem">${esc(t('view.glide_path.h2.glide'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.glide_path.th.age">Age</th>
                    <th data-i18n="view.glide_path.th.stock">Stock %</th>
                    <th data-i18n="view.glide_path.th.bond">Bond %</th>
                    <th data-i18n="view.glide_path.th.phase">Phase</th>
                </tr></thead>
                <tbody>${(r.glide || []).map(p => `
                    <tr>
                        <td>${p.age}</td>
                        <td>${p.stock_pct.toFixed(1)}%</td>
                        <td>${p.bond_pct.toFixed(1)}%</td>
                        <td class="${phaseCls(p.phase)}" style="text-transform:uppercase">${esc(t('view.glide_path.phase.' + p.phase) || p.phase)}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
        // Glide-path export (Copy / CSV). No permalink — id-based inputs.
        enh.mountToolbar(mount.querySelector('#gp-tools'), {
            viewId: 'glide-path',
            link: false,
            filename: 'glide-path.csv',
            getRows: () => [['age', 'stock_pct', 'bond_pct', 'phase'],
                ...(r.glide || []).map(p => [p.age, p.stock_pct, p.bond_pct, p.phase])],
        });
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
