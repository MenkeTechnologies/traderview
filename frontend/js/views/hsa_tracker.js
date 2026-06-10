// HSA Triple-Tax Tracker — Health Savings Account is the only US account
// with three simultaneous tax advantages:
//   1) Contributions are pre-tax (federal + FICA via payroll, federal only via direct)
//   2) Growth is tax-free (no cap gains, no dividends taxed)
//   3) Qualified medical withdrawals are tax-free
// After age 65 it functions like a Traditional IRA (taxed at ordinary
// rates for non-medical), giving a 4th "stealth retirement account" mode.
// 2026 limits: $4,400 individual / $8,750 family / $1,000 age-55 catchup.

import { esc } from '../util.js';
import { t } from '../i18n.js';

const LIMITS_2026 = {
    individual: 4400,
    family:     8750,
    catchup:    1000,    // age 55+
};

export async function renderHsaTracker(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.hsa_tracker.title">// HSA TRIPLE-TAX TRACKER</span></h1>
        <p class="muted small" data-i18n-html="view.hsa_tracker.intro">
            HSA is the most tax-advantaged US account: <strong>pre-tax in</strong>,
            <strong>tax-free growth</strong>, <strong>tax-free out for medical</strong>.
            After age 65 non-medical withdrawals are taxed as ordinary income
            (like a Traditional IRA — no 20% penalty). The "shoebox method":
            pay current medical out-of-pocket, save receipts, let the HSA
            compound, withdraw decades later tax-free against those receipts.
            2026 limits: $4,400 individual / $8,750 family / +$1,000 catchup at 55+.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Coverage type</span>
                    <select id="hsa-cov" style="width:100%">
                        <option value="individual">Individual ($4,400)</option>
                        <option value="family" selected>Family ($8,750)</option>
                    </select>
                </label>
                <label>
                    <span class="muted small">Current age</span>
                    <input type="number" id="hsa-age" step="1" min="18" max="74" value="35" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Target age (≥65)</span>
                    <input type="number" id="hsa-target" step="1" min="55" max="80" value="65" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Annual contribution $</span>
                    <input type="number" id="hsa-contrib" step="100" min="0" value="8750" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Current balance $</span>
                    <input type="number" id="hsa-balance" step="100" min="0" value="15000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Expected return %</span>
                    <input type="number" id="hsa-return" step="0.1" min="-10" max="20" value="7" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Marginal income tax %</span>
                    <input type="number" id="hsa-tax" step="1" min="0" max="50" value="22" style="width:100%">
                </label>
                <label>
                    <span class="muted small">FICA % (employer payroll)</span>
                    <input type="number" id="hsa-fica" step="0.1" min="0" max="15.3" value="7.65" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="hsa-run">⚡ Project</button>
            <div id="hsa-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#hsa-cov, #hsa-age, #hsa-target, #hsa-contrib, #hsa-balance, #hsa-return, #hsa-tax, #hsa-fica').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#hsa-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const cov = mount.querySelector('#hsa-cov').value;
    const age = parseInt(mount.querySelector('#hsa-age').value, 10) || 35;
    const target = parseInt(mount.querySelector('#hsa-target').value, 10) || 65;
    const contrib = parseFloat(mount.querySelector('#hsa-contrib').value) || 0;
    let bal = parseFloat(mount.querySelector('#hsa-balance').value) || 0;
    const r = parseFloat(mount.querySelector('#hsa-return').value) / 100;
    const tax = parseFloat(mount.querySelector('#hsa-tax').value) / 100;
    const fica = parseFloat(mount.querySelector('#hsa-fica').value) / 100;
    const result = mount.querySelector('#hsa-result');

    const years = Math.max(0, target - age);
    const limit = LIMITS_2026[cov] + (age >= 55 ? LIMITS_2026.catchup : 0);
    const overContribFlag = contrib > limit;

    const rows = [];
    let totalContrib = bal;
    let totalTaxSaved = 0;
    for (let i = 0; i < years; i++) {
        const a = age + i;
        const annual = a >= 55 ? LIMITS_2026[cov] + LIMITS_2026.catchup : LIMITS_2026[cov];
        const actualContrib = Math.min(contrib, annual);
        bal = (bal + actualContrib) * (1 + r);
        totalContrib += actualContrib;
        const upfrontSave = actualContrib * (tax + fica);
        totalTaxSaved += upfrontSave;
        rows.push({
            year: i + 1,
            age: a + 1,
            contrib: actualContrib,
            upfrontSave,
            balance: bal,
        });
    }
    const totalGrowth = bal - totalContrib;
    // Compare to taxable account: same contrib amounts (post-tax), 15% LTCG drag annually.
    const r_taxable_eq = r * (1 - 0.15);
    let taxBal = parseFloat(mount.querySelector('#hsa-balance').value) || 0;
    for (let i = 0; i < years; i++) {
        const a = age + i;
        const annual = a >= 55 ? LIMITS_2026[cov] + LIMITS_2026.catchup : LIMITS_2026[cov];
        const actualContrib = Math.min(contrib, annual);
        const postTaxContrib = actualContrib * (1 - (tax + fica));
        taxBal = (taxBal + postTaxContrib) * (1 + r_taxable_eq);
    }
    const hsaAdvantage = bal - taxBal;

    result.innerHTML = `
        ${overContribFlag ? `<p class="neg small"><strong>Warning:</strong> contribution $${fmt(contrib, 0)} exceeds ${cov} limit $${fmt(limit, 0)}. Excess is subject to a 6% IRS excise tax until removed.</p>` : ''}
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">HSA balance at age ${target}</div><div class="value pos">$${fmt(bal, 0)}</div></div>
            <div class="card"><div class="label">Total contributed</div><div class="value">$${fmt(totalContrib, 0)}</div></div>
            <div class="card"><div class="label">Tax-free growth</div><div class="value pos">$${fmt(totalGrowth, 0)}</div></div>
            <div class="card"><div class="label">Upfront tax saved</div><div class="value pos">$${fmt(totalTaxSaved, 0)}</div><div class="muted small">@ ${fmt((tax + fica) * 100, 1)}% combined</div></div>
            <div class="card"><div class="label">vs taxable brokerage</div><div class="value pos">+$${fmt(hsaAdvantage, 0)}</div><div class="muted small">HSA wins by this much</div></div>
        </div>
        <p class="muted small">
            At age ${target} ≥ 65: non-medical withdrawals taxed as ordinary income
            (treat as Trad IRA) — projected after-tax non-medical value at your bracket
            <strong>$${fmt(bal * (1 - tax), 0)}</strong>. Qualified medical withdrawals
            stay tax-free for life — that's the optimum use.
        </p>
        <h3 class="section-title">Year-by-year projection</h3>
        <table class="trades" data-table-key="hsa-rows">
            <thead><tr>
                <th>Year</th>
                <th>Age (end)</th>
                <th>Contribution</th>
                <th>Upfront tax saved</th>
                <th>End balance</th>
            </tr></thead>
            <tbody>${rows.map(row => `<tr>
                <td>${row.year}</td>
                <td>${row.age}</td>
                <td>$${fmt(row.contrib, 0)}</td>
                <td class="pos">$${fmt(row.upfrontSave, 0)}</td>
                <td><strong>$${fmt(row.balance, 0)}</strong></td>
            </tr>`).join('')}</tbody>
        </table>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
