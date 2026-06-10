// Roth Conversion Ladder — early-retirement bridge strategy.
// Convert $X/yr from Traditional IRA → Roth IRA. Each conversion has
// a 5-year season-out period before the principal can be withdrawn
// tax/penalty-free. Schedule converts now → access at age 59½ without
// 10% early-withdrawal penalty. Tracks: tax cost per conversion, taxable
// income filling lower brackets, year-by-year laddered access schedule.

import { esc } from '../util.js';
import { t } from '../i18n.js';

const STD_DED_2025 = { single: 15000, mfj: 30000, mfs: 15000, hoh: 22500 };
const BRACKETS_2025 = {
    single: [[0,0.10],[11925,0.12],[48475,0.22],[103350,0.24],[197300,0.32],[250525,0.35],[626350,0.37]],
    mfj:    [[0,0.10],[23850,0.12],[96950,0.22],[206700,0.24],[394600,0.32],[501050,0.35],[751600,0.37]],
    mfs:    [[0,0.10],[11925,0.12],[48475,0.22],[103350,0.24],[197300,0.32],[250525,0.35],[375800,0.37]],
    hoh:    [[0,0.10],[17000,0.12],[64850,0.22],[103350,0.24],[197300,0.32],[250500,0.35],[626350,0.37]],
};

export async function renderRothConversionLadder(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.roth_conversion_ladder.title">// ROTH CONVERSION LADDER</span></h1>
        <p class="muted small" data-i18n-html="view.roth_conversion_ladder.intro">
            Early-retirement bridge. Each $X moved from Traditional → Roth IRA
            becomes principal that can be withdrawn tax- and penalty-free
            after a <strong>5-year season-out period</strong> (per conversion).
            Stagger conversions to build a Roth "ladder" you can draw from
            before age 59½ without the 10% early-withdrawal penalty.
            Conversions count as ordinary income — size them to fill the
            <strong>12% or 22% brackets</strong> (whichever is your seam).
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Filing status</span>
                    <select id="rcl-status" style="width:100%">
                        <option value="single">Single</option>
                        <option value="mfj" selected>MFJ</option>
                        <option value="mfs">MFS</option>
                        <option value="hoh">HoH</option>
                    </select>
                </label>
                <label>
                    <span class="muted small">Other ordinary income / yr $</span>
                    <input type="number" id="rcl-other" step="500" min="0" value="0" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Annual conversion $</span>
                    <input type="number" id="rcl-amount" step="1000" min="0" value="60000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Years to ladder</span>
                    <input type="number" id="rcl-years" step="1" min="1" max="30" value="10" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Current age</span>
                    <input type="number" id="rcl-age" step="1" min="18" max="65" value="45" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Trad IRA balance $</span>
                    <input type="number" id="rcl-balance" step="10000" min="0" value="1200000" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="rcl-run">⚡ Build ladder</button>
            <div id="rcl-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#rcl-status, #rcl-other, #rcl-amount, #rcl-years, #rcl-age, #rcl-balance').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#rcl-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const status = mount.querySelector('#rcl-status').value;
    const other = parseFloat(mount.querySelector('#rcl-other').value) || 0;
    const amount = parseFloat(mount.querySelector('#rcl-amount').value) || 0;
    const years = Math.max(1, Math.min(30, parseInt(mount.querySelector('#rcl-years').value, 10) || 0));
    const age = parseInt(mount.querySelector('#rcl-age').value, 10) || 45;
    let balance = parseFloat(mount.querySelector('#rcl-balance').value) || 0;
    const result = mount.querySelector('#rcl-result');
    if (amount <= 0) {
        result.innerHTML = `<p class="muted">Conversion amount > 0 required.</p>`;
        return;
    }
    const dedu = STD_DED_2025[status];
    const brackets = BRACKETS_2025[status];

    const rows = [];
    let totalTax = 0;
    let totalConverted = 0;
    for (let i = 0; i < years; i++) {
        if (balance <= 0) break;
        const convert = Math.min(amount, balance);
        balance -= convert;
        const ordinaryThisYear = other + convert;
        const taxable = Math.max(0, ordinaryThisYear - dedu);
        const { tax: totalOrdTax } = applyBrackets(brackets, taxable);
        const baselineTaxable = Math.max(0, other - dedu);
        const { tax: baselineTax } = applyBrackets(brackets, baselineTaxable);
        const taxOnConversion = totalOrdTax - baselineTax;
        const effective = convert > 0 ? taxOnConversion / convert : 0;
        const marginal = marginalRateAt(brackets, taxable);
        const accessAtAge = age + i + 5;
        rows.push({
            yearIdx: i,
            convertYear: 2026 + i,
            age: age + i,
            convert,
            taxOnConversion,
            effective,
            marginal,
            accessAtAge,
            accessYear: 2026 + i + 5,
            tradBalanceAfter: balance,
        });
        totalTax += taxOnConversion;
        totalConverted += convert;
    }

    const avgEffective = totalConverted > 0 ? totalTax / totalConverted : 0;
    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Total converted</div><div class="value pos">$${fmt(totalConverted, 0)}</div></div>
            <div class="card"><div class="label">Total fed tax cost</div><div class="value neg">$${fmt(totalTax, 0)}</div></div>
            <div class="card"><div class="label">Avg effective rate</div><div class="value">${fmt(avgEffective * 100, 2)}%</div></div>
            <div class="card"><div class="label">Trad IRA remaining</div><div class="value">$${fmt(balance, 0)}</div></div>
            <div class="card"><div class="label">First access at age</div><div class="value">${rows[0]?.accessAtAge ?? '—'}</div></div>
        </div>
        <table class="trades" data-table-key="rcl-rows">
            <thead><tr>
                <th>Year</th>
                <th>Age</th>
                <th>Convert $</th>
                <th>Tax $</th>
                <th>Effective</th>
                <th>Marginal</th>
                <th>Access age</th>
                <th>Trad balance after</th>
            </tr></thead>
            <tbody>${rows.map(r => `<tr>
                <td>${r.convertYear}</td>
                <td>${r.age}</td>
                <td>$${fmt(r.convert, 0)}</td>
                <td class="neg">$${fmt(r.taxOnConversion, 0)}</td>
                <td>${fmt(r.effective * 100, 2)}%</td>
                <td>${fmt(r.marginal * 100, 0)}%</td>
                <td><strong>${r.accessAtAge}</strong> <span class="muted small">(${r.accessYear})</span></td>
                <td class="muted">$${fmt(r.tradBalanceAfter, 0)}</td>
            </tr>`).join('')}</tbody>
        </table>
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
    return { tax };
}
function marginalRateAt(brackets, taxable) {
    let m = 0;
    for (const [from, rate] of brackets) {
        if (taxable >= from) m = rate;
    }
    return m;
}
function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
