// Income Tax Estimator — running YTD federal effective-tax estimate
// against 2025 IRS brackets (most recent published). Supports single,
// MFJ, MFS, HoH filing statuses. Includes standard deduction default,
// optional itemized override, LTCG segregated at 0/15/20%, and FICA
// (Social Security + Medicare + Additional Medicare 0.9% over the
// threshold). Output is an effective+marginal breakdown table.

import { esc } from '../util.js';
import { t } from '../i18n.js';

// 2025 federal brackets (IRS Rev. Proc. 2024-40, official inflation
// adjustments for tax year 2025).
const BRACKETS = {
    single: [
        [0,        0.10],
        [11925,    0.12],
        [48475,    0.22],
        [103350,   0.24],
        [197300,   0.32],
        [250525,   0.35],
        [626350,   0.37],
    ],
    mfj: [
        [0,        0.10],
        [23850,    0.12],
        [96950,    0.22],
        [206700,   0.24],
        [394600,   0.32],
        [501050,   0.35],
        [751600,   0.37],
    ],
    mfs: [
        [0,        0.10],
        [11925,    0.12],
        [48475,    0.22],
        [103350,   0.24],
        [197300,   0.32],
        [250525,   0.35],
        [375800,   0.37],
    ],
    hoh: [
        [0,        0.10],
        [17000,    0.12],
        [64850,    0.22],
        [103350,   0.24],
        [197300,   0.32],
        [250500,   0.35],
        [626350,   0.37],
    ],
};

const STD_DEDUCTION = {
    single: 15000,
    mfj:    30000,
    mfs:    15000,
    hoh:    22500,
};

const LTCG_BRACKETS = {
    single: [ [0, 0.00], [48350, 0.15], [533400, 0.20] ],
    mfj:    [ [0, 0.00], [96700, 0.15], [600050, 0.20] ],
    mfs:    [ [0, 0.00], [48350, 0.15], [300000, 0.20] ],
    hoh:    [ [0, 0.00], [64750, 0.15], [566700, 0.20] ],
};

const SS_WAGE_BASE_2025 = 176100;
const SS_RATE = 0.062;
const MEDI_RATE = 0.0145;
const ADDL_MEDI_RATE = 0.009;
const ADDL_MEDI_THRESHOLD = { single: 200000, mfj: 250000, mfs: 125000, hoh: 200000 };

export async function renderIncomeTaxEstimator(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.income_tax_estimator.title">// INCOME TAX ESTIMATOR · TY2025</span></h1>
        <p class="muted small" data-i18n-html="view.income_tax_estimator.intro">
            Federal estimate against TY2025 IRS-published brackets (Rev. Proc. 2024-40).
            Includes ordinary income + LTCG segregated at 0/15/20%, standard or
            itemized deduction, FICA, and Additional Medicare 0.9% over the threshold.
            Does <strong>not</strong> include state tax, AMT, NIIT, self-employment tax,
            or credits — for those, use a CPA or tax software.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Filing status</span>
                    <select id="tx-status" style="width:100%">
                        <option value="single" selected>Single</option>
                        <option value="mfj">Married filing jointly</option>
                        <option value="mfs">Married filing separately</option>
                        <option value="hoh">Head of household</option>
                    </select>
                </label>
                <label>
                    <span class="muted small">W-2 / ordinary income $</span>
                    <input type="number" id="tx-wages" step="1000" min="0" value="180000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Short-term cap gains $</span>
                    <input type="number" id="tx-stcg" step="100" value="0" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Long-term cap gains $</span>
                    <input type="number" id="tx-ltcg" step="100" value="0" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Qualified dividends $</span>
                    <input type="number" id="tx-qdiv" step="100" value="0" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Itemized override $ (0 = use std)</span>
                    <input type="number" id="tx-item" step="500" value="0" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Pre-tax 401(k) / HSA $</span>
                    <input type="number" id="tx-pretax" step="500" value="23500" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="tx-run" data-shortcut="r">⚡ Compute</button>
            <div id="tx-result" style="margin-top:12px"></div>
        </div>
    `;

    const inputs = mount.querySelectorAll('#tx-status, #tx-wages, #tx-stcg, #tx-ltcg, #tx-qdiv, #tx-item, #tx-pretax');
    inputs.forEach(el => el.addEventListener('input', () => compute(mount)));
    mount.querySelector('#tx-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const status = mount.querySelector('#tx-status').value;
    const wages = parseFloat(mount.querySelector('#tx-wages').value) || 0;
    const stcg = parseFloat(mount.querySelector('#tx-stcg').value) || 0;
    const ltcg = parseFloat(mount.querySelector('#tx-ltcg').value) || 0;
    const qdiv = parseFloat(mount.querySelector('#tx-qdiv').value) || 0;
    const itemOverride = parseFloat(mount.querySelector('#tx-item').value) || 0;
    const pretax = parseFloat(mount.querySelector('#tx-pretax').value) || 0;

    // Ordinary income: W-2 + STCG, minus pre-tax deferrals.
    const ordinaryGross = wages + stcg;
    const ordinaryAfterPretax = Math.max(0, ordinaryGross - pretax);
    const deduction = itemOverride > 0 ? itemOverride : STD_DEDUCTION[status];
    const ordinaryTaxable = Math.max(0, ordinaryAfterPretax - deduction);
    // Preferential income: LTCG + qualified dividends.
    const prefTaxable = ltcg + qdiv;

    // Ordinary federal income tax (progressive brackets).
    const { tax: ordTax, marginalRate: ordMarginal, breakdown: ordBreak } = applyBrackets(BRACKETS[status], ordinaryTaxable);

    // LTCG: stacked ON TOP of ordinary taxable for bracket-position purposes.
    // The 0%/15%/20% breakpoints are based on TOTAL taxable income, not LTCG
    // alone. Compute LTCG tax by walking the preferential brackets starting
    // at ordinaryTaxable as the "floor".
    const { tax: prefTax, breakdown: prefBreak } = applyLtcgBrackets(LTCG_BRACKETS[status], ordinaryTaxable, prefTaxable);

    // FICA: only on W-2 wages, NOT on STCG/LTCG/dividends. Employee side only.
    const ssWages = Math.min(wages, SS_WAGE_BASE_2025);
    const ssTax = ssWages * SS_RATE;
    const mediTax = wages * MEDI_RATE;
    const addlMediBase = Math.max(0, wages - ADDL_MEDI_THRESHOLD[status]);
    const addlMedi = addlMediBase * ADDL_MEDI_RATE;
    const fica = ssTax + mediTax + addlMedi;

    const totalTax = ordTax + prefTax + fica;
    const totalIncome = wages + stcg + ltcg + qdiv;
    const effectiveRate = totalIncome > 0 ? totalTax / totalIncome : 0;
    const takeHome = totalIncome - pretax - totalTax;

    const result = mount.querySelector('#tx-result');
    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Total income</div><div class="value">$${fmt(totalIncome, 0)}</div></div>
            <div class="card"><div class="label">Ordinary federal</div><div class="value neg">-$${fmt(ordTax, 0)}</div><div class="muted small">marginal ${pct(ordMarginal)}</div></div>
            <div class="card"><div class="label">LTCG / qualified</div><div class="value neg">-$${fmt(prefTax, 0)}</div></div>
            <div class="card"><div class="label">FICA</div><div class="value neg">-$${fmt(fica, 0)}</div><div class="muted small">SS $${fmt(ssTax, 0)} · Medi $${fmt(mediTax + addlMedi, 0)}</div></div>
            <div class="card"><div class="label">Effective tax rate</div><div class="value">${pct(effectiveRate)}</div></div>
            <div class="card"><div class="label">Est. take-home</div><div class="value pos">$${fmt(takeHome, 0)}</div></div>
        </div>
        <h3 class="section-title">Ordinary bracket walk</h3>
        <table class="trades" data-table-key="tx-ord">
            <thead><tr><th>Rate</th><th>Bracket range</th><th>Income in bracket</th><th>Tax in bracket</th></tr></thead>
            <tbody>${ordBreak.map(row => `<tr>
                <td>${pct(row.rate)}</td>
                <td class="muted">$${fmt(row.from, 0)} → ${row.to == null ? '∞' : '$' + fmt(row.to, 0)}</td>
                <td>$${fmt(row.amount, 0)}</td>
                <td><strong>$${fmt(row.tax, 0)}</strong></td>
            </tr>`).join('')}</tbody>
        </table>
        ${prefBreak.length ? `
        <h3 class="section-title" style="margin-top:18px">LTCG bracket walk (stacked above ordinary)</h3>
        <table class="trades" data-table-key="tx-pref">
            <thead><tr><th>Rate</th><th>Bracket range</th><th>Income in bracket</th><th>Tax in bracket</th></tr></thead>
            <tbody>${prefBreak.map(row => `<tr>
                <td>${pct(row.rate)}</td>
                <td class="muted">$${fmt(row.from, 0)} → ${row.to == null ? '∞' : '$' + fmt(row.to, 0)}</td>
                <td>$${fmt(row.amount, 0)}</td>
                <td><strong>$${fmt(row.tax, 0)}</strong></td>
            </tr>`).join('')}</tbody>
        </table>` : ''}
    `;
}

function applyBrackets(brackets, taxable) {
    let tax = 0;
    let marginalRate = 0;
    const breakdown = [];
    for (let i = 0; i < brackets.length; i++) {
        const [from, rate] = brackets[i];
        const to = brackets[i + 1] ? brackets[i + 1][0] : null;
        if (taxable <= from) {
            breakdown.push({ rate, from, to, amount: 0, tax: 0 });
            continue;
        }
        const top = to == null ? taxable : Math.min(taxable, to);
        const amount = Math.max(0, top - from);
        const segTax = amount * rate;
        tax += segTax;
        marginalRate = rate;
        breakdown.push({ rate, from, to, amount, tax: segTax });
    }
    return { tax, marginalRate, breakdown };
}

function applyLtcgBrackets(brackets, floor, pref) {
    let tax = 0;
    const breakdown = [];
    if (pref <= 0) return { tax, breakdown };
    const ceiling = floor + pref;
    for (let i = 0; i < brackets.length; i++) {
        const [from, rate] = brackets[i];
        const to = brackets[i + 1] ? brackets[i + 1][0] : null;
        if (ceiling <= from) {
            breakdown.push({ rate, from, to, amount: 0, tax: 0 });
            continue;
        }
        const segLo = Math.max(floor, from);
        const segHi = to == null ? ceiling : Math.min(ceiling, to);
        const amount = Math.max(0, segHi - segLo);
        const segTax = amount * rate;
        tax += segTax;
        breakdown.push({ rate, from, to, amount, tax: segTax });
    }
    return { tax, breakdown };
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}

function pct(r) {
    return (r * 100).toFixed(1) + '%';
}
