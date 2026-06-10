// Home maintenance budget projector. 1% rule for general upkeep
// plus per-major-system replacement schedule with monthly set-aside.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const STATE = {
    home_value_usd: 500000,
    general_pct_of_value: 1.0,
    current_year: 2026,
    systems: [
        { name: 'Roof',           install_year: 2010, expected_life_years: 25, replacement_cost_usd: 18000 },
        { name: 'HVAC',           install_year: 2015, expected_life_years: 15, replacement_cost_usd: 9000 },
        { name: 'Water heater',   install_year: 2018, expected_life_years: 12, replacement_cost_usd: 1800 },
        { name: 'Appliances',     install_year: 2020, expected_life_years: 12, replacement_cost_usd: 5000 },
        { name: 'Driveway / paint', install_year: 2018, expected_life_years: 15, replacement_cost_usd: 7000 },
    ],
};

export async function renderHomeMaintenance(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.home_maintenance.title">// HOME MAINTENANCE BUDGET</span></h1>
        <p class="muted small" data-i18n-html="view.home_maintenance.intro">
            Two-part calculation: the classic <strong>1% rule</strong> — set aside ~1% of
            home value each year for general maintenance + repairs (range 0.5–2% by age
            and condition) — plus a <strong>per-system replacement schedule</strong> for
            major systems (roof, HVAC, water heater, appliances, etc.). Per system:
            install year + expected life + replacement cost → years until replacement,
            monthly set-aside, status (ok / due_soon / overdue).
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.home_maintenance.field.home_value">Home value $</span>
                    <input type="number" id="hm-value" step="5000" min="0" value="${STATE.home_value_usd}" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.home_maintenance.field.pct">General % of value/yr</span>
                    <input type="number" id="hm-pct" step="0.1" min="0" max="10" value="${STATE.general_pct_of_value}" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.home_maintenance.field.year">Current year</span>
                    <input type="number" id="hm-year" step="1" min="1900" max="2200" value="${STATE.current_year}" style="width:100%"></label>
            </div>
            <h2>${esc(t('view.home_maintenance.h2.systems'))}</h2>
            <table class="trades" id="hm-table">
                <thead><tr>
                    <th data-i18n="view.home_maintenance.th.name">System</th>
                    <th data-i18n="view.home_maintenance.th.install">Install year</th>
                    <th data-i18n="view.home_maintenance.th.life">Life years</th>
                    <th data-i18n="view.home_maintenance.th.cost">Replacement $</th>
                    <th></th>
                </tr></thead>
                <tbody id="hm-body"></tbody>
            </table>
            <button class="btn btn-sm" id="hm-add" data-i18n="view.home_maintenance.btn.add">＋ Add system</button>
            <div style="margin-top:1rem">
                <button class="btn btn-sm primary" id="hm-run" data-shortcut="r" data-i18n="view.home_maintenance.btn.run">⚡ Compute Budget</button>
            </div>
            <div id="hm-result"></div>
        </div>
    `;
    drawRows(mount);
    mount.querySelector('#hm-value').addEventListener('input', e => {
        STATE.home_value_usd = parseFloat(e.target.value) || 0;
    });
    mount.querySelector('#hm-pct').addEventListener('input', e => {
        STATE.general_pct_of_value = parseFloat(e.target.value) || 0;
    });
    mount.querySelector('#hm-year').addEventListener('input', e => {
        STATE.current_year = parseInt(e.target.value, 10) || 2026;
    });
    mount.querySelector('#hm-add').addEventListener('click', () => {
        STATE.systems.push({ name: 'New system', install_year: STATE.current_year, expected_life_years: 10, replacement_cost_usd: 1000 });
        drawRows(mount);
    });
    mount.querySelector('#hm-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

function drawRows(mount) {
    const body = mount.querySelector('#hm-body');
    body.innerHTML = STATE.systems.map((s, i) => `
        <tr>
            <td><input type="text" data-k="name" data-i="${i}" value="${esc(s.name)}" style="width:100%"></td>
            <td><input type="number" step="1" min="1800" max="2200" data-k="install_year" data-i="${i}" value="${s.install_year}" style="width:100%"></td>
            <td><input type="number" step="1" min="1" max="200" data-k="expected_life_years" data-i="${i}" value="${s.expected_life_years}" style="width:100%"></td>
            <td><input type="number" step="100" min="0" data-k="replacement_cost_usd" data-i="${i}" value="${s.replacement_cost_usd}" style="width:100%"></td>
            <td><button class="btn btn-xs" data-del="${i}" data-tip="common.tip.remove_row" data-i18n-aria-label="common.aria.remove" aria-label="Remove">✕</button></td>
        </tr>
    `).join('');
    body.querySelectorAll('input').forEach(inp => {
        inp.addEventListener('input', () => {
            const i = parseInt(inp.dataset.i, 10);
            const k = inp.dataset.k;
            if (k === 'name') STATE.systems[i][k] = inp.value;
            else if (k === 'install_year' || k === 'expected_life_years') STATE.systems[i][k] = parseInt(inp.value, 10) || 0;
            else STATE.systems[i][k] = parseFloat(inp.value) || 0;
        });
    });
    body.querySelectorAll('button[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            STATE.systems.splice(parseInt(btn.dataset.del, 10), 1);
            drawRows(mount);
        });
    });
}

async function runCompute(mount) {
    const result = mount.querySelector('#hm-result');
    result.innerHTML = `<p class="muted">${esc(t('view.home_maintenance.status.computing'))}</p>`;
    try {
        const r = await api.request('/home-maintenance/compute', { method: 'POST', body: JSON.stringify(STATE) });
        const stCls = s => s === 'overdue' ? 'neg' : s === 'due_soon' ? '' : 'pos';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.home_maintenance.field.general_annual'))}</div>
                    <strong>$${r.general_annual_usd.toFixed(0)}/yr</strong></div>
                <div><div class="muted small">${esc(t('view.home_maintenance.field.general_monthly'))}</div>
                    <strong>$${r.general_monthly_usd.toFixed(0)}/mo</strong></div>
                <div><div class="muted small">${esc(t('view.home_maintenance.field.system_monthly'))}</div>
                    <strong>$${r.total_system_monthly_set_aside_usd.toFixed(0)}/mo</strong></div>
                <div><div class="muted small">${esc(t('view.home_maintenance.field.total_monthly'))}</div>
                    <strong style="font-size:1.4em">$${r.total_monthly_budget_usd.toFixed(0)}/mo</strong></div>
                <div><div class="muted small">${esc(t('view.home_maintenance.field.overdue'))}</div>
                    <strong class="${r.overdue_count > 0 ? 'neg' : 'pos'}">${r.overdue_count}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.home_maintenance.h2.per_system'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.home_maintenance.th.name">System</th>
                    <th data-i18n="view.home_maintenance.th.eol">End-of-life year</th>
                    <th data-i18n="view.home_maintenance.th.years">Years until</th>
                    <th data-i18n="view.home_maintenance.th.cost_h">Cost</th>
                    <th data-i18n="view.home_maintenance.th.aside">Monthly aside</th>
                    <th data-i18n="view.home_maintenance.th.status">Status</th>
                </tr></thead>
                <tbody>${(r.systems || []).map(s => `
                    <tr>
                        <td><strong>${esc(s.name)}</strong></td>
                        <td>${s.end_of_life_year}</td>
                        <td>${s.years_until_replacement}</td>
                        <td>$${s.replacement_cost_usd.toFixed(0)}</td>
                        <td>$${s.monthly_set_aside_usd.toFixed(0)}</td>
                        <td class="${stCls(s.status)}" style="text-transform:uppercase"><strong>${esc(t('view.home_maintenance.status.' + s.status) || s.status)}</strong></td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
