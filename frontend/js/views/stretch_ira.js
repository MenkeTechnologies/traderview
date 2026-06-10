// Stretch IRA — SECURE Act 1.0 (2019) killed the lifetime "stretch"
// for most non-spouse inherited IRAs. Now the 10-year rule applies:
// the entire account must be emptied within 10 years of the original
// owner's death. SECURE Act 2.0 (2022) added Eligible Designated
// Beneficiaries who can still stretch (surviving spouse, minor child,
// disabled, chronically ill, person <10 years younger than decedent).
//
// This tool models: even-distribution, back-loaded, RMD-style, and
// year-10-everything strategies, with year-by-year tax cost projection
// based on user's expected ordinary-income bracket.

import { esc } from '../util.js';

export async function renderStretchIra(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.stretch_ira.title">// STRETCH IRA · SECURE 10-YEAR RULE</span></h1>
        <p class="muted small" data-i18n-html="view.stretch_ira.intro">
            SECURE Act 1.0 (effective 2020) killed the lifetime stretch for most
            non-spouse inherited IRAs. The <strong>entire inherited account
            must be distributed within 10 years</strong> of the original owner's
            death. RMDs required years 1-9 if the decedent had started RMDs;
            year 10 must bring the balance to zero. Comparing 4 distribution
            strategies for total tax cost.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Inherited IRA balance $</span>
                    <input type="number" id="si-balance" step="10000" min="0" value="500000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Expected growth %/yr</span>
                    <input type="number" id="si-growth" step="0.5" min="-10" max="20" value="6" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Your other ordinary income $</span>
                    <input type="number" id="si-other" step="1000" min="0" value="120000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Filing status</span>
                    <select id="si-status" style="width:100%">
                        <option value="single">Single</option>
                        <option value="mfj" selected>MFJ</option>
                    </select>
                </label>
            </div>
            <button class="btn btn-sm primary" id="si-run">⚡ Compare strategies</button>
            <div id="si-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#si-balance, #si-growth, #si-other, #si-status').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#si-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

const STD_DED_2025 = { single: 15000, mfj: 30000 };
const BRACKETS_2025 = {
    single: [[0,0.10],[11925,0.12],[48475,0.22],[103350,0.24],[197300,0.32],[250525,0.35],[626350,0.37]],
    mfj:    [[0,0.10],[23850,0.12],[96950,0.22],[206700,0.24],[394600,0.32],[501050,0.35],[751600,0.37]],
};

function compute(mount) {
    const balance0 = parseFloat(mount.querySelector('#si-balance').value) || 0;
    const growth = parseFloat(mount.querySelector('#si-growth').value) / 100;
    const other = parseFloat(mount.querySelector('#si-other').value) || 0;
    const status = mount.querySelector('#si-status').value;
    const result = mount.querySelector('#si-result');

    const strategies = [
        { name: 'Even (1/10 per year)',     payouts: (bal, y) => bal / (10 - y) },
        { name: 'Back-loaded (year 10 only)', payouts: (bal, y) => y === 9 ? bal : 0 },
        { name: 'RMD-mimic (small early, big late)', payouts: (bal, y) => bal / Math.max(1, 10 - y) * (0.4 + 0.06 * y) },
        { name: 'Front-loaded (year 1 heavy)', payouts: (bal, y) => y === 0 ? bal * 0.4 : bal / (9 - y + 1) },
    ];

    const baselineTaxable = Math.max(0, other - STD_DED_2025[status]);
    const baselineTax = applyBrackets(BRACKETS_2025[status], baselineTaxable);

    const runs = strategies.map(s => {
        let bal = balance0;
        let totalTax = 0;
        let totalReceived = 0;
        const rows = [];
        for (let y = 0; y < 10; y++) {
            const distribution = Math.min(bal, s.payouts(bal, y));
            const taxableNew = other + distribution;
            const taxable = Math.max(0, taxableNew - STD_DED_2025[status]);
            const totalIncomeTax = applyBrackets(BRACKETS_2025[status], taxable);
            const taxOnDist = totalIncomeTax - baselineTax;
            const afterTax = distribution - taxOnDist;
            totalTax += taxOnDist;
            totalReceived += afterTax;
            bal = (bal - distribution) * (1 + growth);
            rows.push({ year: y + 1, distribution, taxOnDist, afterTax, balanceAfter: bal });
        }
        return { name: s.name, totalTax, totalReceived, rows };
    });

    const winner = runs.reduce((best, r) => r.totalReceived > best.totalReceived ? r : best, runs[0]);

    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:8px;margin-bottom:12px">
            ${runs.map(r => `<div class="card">
                <div class="label">${esc(r.name)}</div>
                <div class="value ${r === winner ? 'pos' : ''}">$${fmt(r.totalReceived, 0)}</div>
                <div class="muted small">After-tax total · -$${fmt(r.totalTax, 0)} tax${r === winner ? ' · <strong>WINNER</strong>' : ''}</div>
            </div>`).join('')}
        </div>
        <h3 class="section-title">Year-by-year distributions</h3>
        <table class="trades" data-table-key="si-rows">
            <thead><tr>
                <th>Year</th>
                ${runs.map(r => `<th>${esc(r.name)}<br><span class="muted small">After-tax</span></th>`).join('')}
            </tr></thead>
            <tbody>${[0,1,2,3,4,5,6,7,8,9].map(y => `<tr>
                <td>${y + 1}</td>
                ${runs.map(r => {
                    const cell = r.rows[y];
                    return `<td><span class="muted small">$${fmt(cell.distribution, 0)} → -$${fmt(cell.taxOnDist, 0)} tax</span><br><strong>$${fmt(cell.afterTax, 0)}</strong></td>`;
                }).join('')}
            </tr>`).join('')}</tbody>
        </table>
        <p class="muted small" style="margin-top:8px">
            <strong>Note:</strong> Eligible Designated Beneficiaries (surviving spouse,
            minor child, disabled, chronically ill, person &lt;10yr younger than
            decedent) can still use the lifetime stretch under SECURE 2.0 — this
            10-year rule does NOT apply to them.
        </p>
    `;
}

function applyBrackets(brackets, taxable) {
    let tax = 0;
    for (let i = 0; i < brackets.length; i++) {
        const [from, rate] = brackets[i];
        const to = brackets[i + 1] ? brackets[i + 1][0] : null;
        if (taxable <= from) break;
        const top = to == null ? taxable : Math.min(taxable, to);
        tax += Math.max(0, top - from) * rate;
    }
    return tax;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
