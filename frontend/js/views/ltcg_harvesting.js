// Long-Term Capital Gains Harvesting — "fill the 0% LTCG bracket."
// For low-income retirees / sabbatical takers: realize LTCG up to the
// top of the 0% bracket (where LTCG is taxed at 0%), immediately
// repurchase the same security, and step up cost basis for free.
// Unlike loss harvesting, there's NO wash-sale rule on gains —
// repurchase is fine. Critical caveat: realized gains stack ON TOP
// of ordinary income, so the 0% room is (bracket ceiling) − (ordinary
// taxable income). Below the ceiling = 0% federal. Above it = 15%.

import { esc } from '../util.js';
import { t } from '../i18n.js';

const STD_DED_2025 = { single: 15000, mfj: 30000, mfs: 15000, hoh: 22500 };
// 2025 LTCG brackets (IRS Rev. Proc. 2024-40).
const LTCG_TOP_0 = { single: 48350, mfj: 96700, mfs: 48350, hoh: 64750 };
const LTCG_TOP_15 = { single: 533400, mfj: 600050, mfs: 300000, hoh: 566700 };

export async function renderLtcgHarvesting(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ltcg_harvesting.title">// LTCG HARVESTING · 0% BRACKET FILL</span></h1>
        <p class="muted small" data-i18n-html="view.ltcg_harvesting.intro">
            Realize long-term gains up to the top of the <strong>0% LTCG bracket</strong>,
            then immediately repurchase to step up cost basis tax-free.
            <strong>No wash-sale rule on gains.</strong> 2025 ceilings (top of 0% bracket):
            single $48,350 · MFJ $96,700 · HoH $64,750 — measured on
            <strong>taxable income</strong> (ordinary + LTCG), so room = ceiling minus
            ordinary taxable income. Most useful in low-income years (sabbatical,
            early retirement, layoff, first year of FIRE).
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Filing status</span>
                    <select id="lh-status" style="width:100%">
                        <option value="single" selected>Single</option>
                        <option value="mfj">Married filing jointly</option>
                        <option value="mfs">Married filing separately</option>
                        <option value="hoh">Head of household</option>
                    </select>
                </label>
                <label>
                    <span class="muted small">Ordinary income $</span>
                    <input type="number" id="lh-ord" step="500" min="0" value="30000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Itemized override $ (0 = std ded)</span>
                    <input type="number" id="lh-item" step="500" value="0" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Unrealized LTCG available $</span>
                    <input type="number" id="lh-pool" step="500" min="0" value="80000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">State LTCG rate %</span>
                    <input type="number" id="lh-state" step="0.5" min="0" max="20" value="0" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="lh-run">⚡ Compute</button>
            <div id="lh-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#lh-status, #lh-ord, #lh-item, #lh-pool, #lh-state').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#lh-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const status = mount.querySelector('#lh-status').value;
    const ordIncome = parseFloat(mount.querySelector('#lh-ord').value) || 0;
    const itemOverride = parseFloat(mount.querySelector('#lh-item').value) || 0;
    const pool = Math.max(0, parseFloat(mount.querySelector('#lh-pool').value) || 0);
    const stRate = parseFloat(mount.querySelector('#lh-state').value) / 100;
    const result = mount.querySelector('#lh-result');

    const dedu = itemOverride > 0 ? itemOverride : STD_DED_2025[status];
    const ordTaxable = Math.max(0, ordIncome - dedu);
    const top0 = LTCG_TOP_0[status];
    const top15 = LTCG_TOP_15[status];

    const room0 = Math.max(0, top0 - ordTaxable);                     // headroom in 0% bracket
    const room15 = Math.max(0, top15 - Math.max(ordTaxable, top0));   // headroom in 15% bracket above 0%
    const harvestable0 = Math.min(pool, room0);
    const remainingAfter0 = pool - harvestable0;
    const harvestable15 = Math.min(remainingAfter0, room15);
    const overflow20 = remainingAfter0 - harvestable15;

    const fedTax0 = harvestable0 * 0.00;
    const fedTax15 = harvestable15 * 0.15;
    const fedTax20 = overflow20 * 0.20;
    const totalFedTax = fedTax0 + fedTax15 + fedTax20;
    const stateTax = pool * stRate;
    const total = totalFedTax + stateTax;
    const stepupValue = harvestable0;                                  // free basis step-up = harvestable at 0%
    const effRate = pool > 0 ? total / pool : 0;

    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Top of 0% bracket</div><div class="value">$${fmt(top0, 0)}</div><div class="muted small">Total taxable income</div></div>
            <div class="card"><div class="label">Your ordinary taxable</div><div class="value">$${fmt(ordTaxable, 0)}</div><div class="muted small">After $${fmt(dedu, 0)} deduction</div></div>
            <div class="card"><div class="label">Room in 0% bracket</div><div class="value pos">$${fmt(room0, 0)}</div></div>
            <div class="card">
                <div class="label">FREE basis step-up</div>
                <div class="value pos">$${fmt(stepupValue, 0)}</div>
                <div class="muted small">Harvest + repurchase = $0 federal tax</div>
            </div>
        </div>
        <h3 class="section-title">Walk if you sell all $${fmt(pool, 0)}</h3>
        <table class="trades" data-table-key="lh-walk">
            <thead><tr>
                <th>Bracket</th>
                <th>Range</th>
                <th>Gain in bracket</th>
                <th>Federal tax</th>
            </tr></thead>
            <tbody>
                <tr>
                    <td class="pos"><strong>0%</strong></td>
                    <td class="muted">$${fmt(ordTaxable, 0)} → $${fmt(top0, 0)}</td>
                    <td>$${fmt(harvestable0, 0)}</td>
                    <td>$0</td>
                </tr>
                <tr>
                    <td>15%</td>
                    <td class="muted">$${fmt(top0, 0)} → $${fmt(top15, 0)}</td>
                    <td>$${fmt(harvestable15, 0)}</td>
                    <td class="neg">$${fmt(fedTax15, 0)}</td>
                </tr>
                <tr>
                    <td class="neg">20%</td>
                    <td class="muted">$${fmt(top15, 0)} → ∞</td>
                    <td>$${fmt(overflow20, 0)}</td>
                    <td class="neg">$${fmt(fedTax20, 0)}</td>
                </tr>
            </tbody>
        </table>
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-top:12px">
            <div class="card"><div class="label">Total federal tax</div><div class="value neg">$${fmt(totalFedTax, 0)}</div></div>
            <div class="card"><div class="label">State tax @ ${fmt(stRate * 100, 1)}%</div><div class="value neg">$${fmt(stateTax, 0)}</div></div>
            <div class="card"><div class="label">Total tax</div><div class="value neg">$${fmt(total, 0)}</div></div>
            <div class="card"><div class="label">Effective rate</div><div class="value">${fmt(effRate * 100, 2)}%</div></div>
        </div>
        ${room0 > 0 ? `<p class="pos small" style="margin-top:8px"><strong>Optimal play:</strong> sell $${fmt(harvestable0, 0)} of LTCG and immediately repurchase the same security. Federal tax: <strong>$0</strong>. Cost basis steps up by that amount, reducing future tax exposure. ${stRate > 0 ? `State tax of $${fmt(harvestable0 * stRate, 0)} still applies.` : ''}</p>` : `<p class="muted small" style="margin-top:8px">Ordinary income exceeds the 0% ceiling — no free harvesting available.</p>`}
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
