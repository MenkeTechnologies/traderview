// Roth Conversion Ladder Planner.
// Convert Traditional → Roth in low-income years to fill lower brackets.
// Each conversion has a 5-year seasoning period before principal can be
// withdrawn penalty-free (under 59½). Strategic for FIRE / early retirement.

import { currentViewToken, viewIsCurrent } from '../app.js';

const BRACKETS_2024_SINGLE = [
    [11_600,  0.10],
    [47_150,  0.12],
    [100_525, 0.22],
    [191_950, 0.24],
    [243_725, 0.32],
    [609_350, 0.35],
    [Infinity, 0.37],
];

let state = {
    filing: 'single',
    target_top_bracket: 0.12,
    current_trad_balance: 500_000,
    current_age: 50,
    annual_other_income: 20_000,
    annual_growth: 0.07,
};

export async function renderRothLadder(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.roth.h1.title">// ROTH CONVERSION LADDER</span></h1>
        <p class="muted small" data-i18n="view.roth.hint.intro">
            Convert Traditional IRA / 401(k) → Roth in low-income years to fill the
            lower brackets. Each year's conversion has a 5-year seasoning period before
            principal can be withdrawn penalty-free (under 59½). Strategic for FIRE.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.roth.h2.inputs">Inputs</h2>
            <form id="roth-form" class="inline-form">
                <label><span data-i18n="view.roth.label.filing">Filing</span>
                    <select name="filing">
                        <option value="single" ${state.filing === 'single' ? 'selected' : ''}>Single</option>
                        <option value="mfj" ${state.filing === 'mfj' ? 'selected' : ''}>MFJ</option>
                    </select>
                </label>
                <label><span data-i18n="view.roth.label.target_top_bracket">Target top bracket %</span>
                    <select name="target_top_bracket">
                        <option value="0.10" ${state.target_top_bracket === 0.10 ? 'selected' : ''}>10%</option>
                        <option value="0.12" ${state.target_top_bracket === 0.12 ? 'selected' : ''}>12%</option>
                        <option value="0.22" ${state.target_top_bracket === 0.22 ? 'selected' : ''}>22%</option>
                        <option value="0.24" ${state.target_top_bracket === 0.24 ? 'selected' : ''}>24%</option>
                    </select>
                </label>
                <label><span data-i18n="view.roth.label.trad_balance">Current Traditional balance ($)</span>
                    <input type="number" step="0.01" name="current_trad_balance" value="${state.current_trad_balance}"></label>
                <label><span data-i18n="view.roth.label.current_age">Current age</span>
                    <input type="number" step="1" name="current_age" value="${state.current_age}" min="18" max="80"></label>
                <label><span data-i18n="view.roth.label.other_income">Annual other income ($)</span>
                    <input type="number" step="0.01" name="annual_other_income" value="${state.annual_other_income}"></label>
                <label><span data-i18n="view.roth.label.annual_growth">Expected annual growth %</span>
                    <input type="number" step="0.5" name="annual_growth" value="${(state.annual_growth * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.roth.btn.plan">Plan</button>
            </form>
        </div>
        <div id="roth-output"></div>
    `;
    document.getElementById('roth-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing = fd.get('filing');
        state.target_top_bracket = Number(fd.get('target_top_bracket'));
        state.current_trad_balance = Number(fd.get('current_trad_balance'));
        state.current_age = Number(fd.get('current_age'));
        state.annual_other_income = Number(fd.get('annual_other_income'));
        state.annual_growth = (Number(fd.get('annual_growth')) || 7) / 100;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('roth-output');
    if (!el) return;
    // Find the income ceiling for target top bracket
    const brackets = BRACKETS_2024_SINGLE;  // simplified; would extend for MFJ
    const stdDeduction = state.filing === 'mfj' ? 29_200 : 14_600;
    const targetCeiling = brackets.find(([_, rate]) => rate > state.target_top_bracket)?.[0]
        || brackets[brackets.length - 1][0];
    const annualHeadroom = targetCeiling + stdDeduction - state.annual_other_income;
    const annualConversion = Math.max(0, annualHeadroom);
    const annualTax = computeTax(annualConversion + state.annual_other_income - stdDeduction);
    const yearsToFullConversion = annualConversion > 0
        ? Math.ceil(state.current_trad_balance / annualConversion) : Infinity;

    // Simulate the ladder
    const ladder = [];
    let tradBal = state.current_trad_balance;
    let rothBal = 0;
    for (let y = 0; y < Math.min(15, yearsToFullConversion); y++) {
        const conv = Math.min(annualConversion, tradBal);
        const tax = computeTaxOnConversion(conv);
        ladder.push({
            year: state.current_age + y,
            conversion: conv,
            tax_owed: tax,
            trad_balance: tradBal,
            roth_balance: rothBal,
            withdrawable_at: state.current_age + y + 5,
        });
        tradBal = (tradBal - conv) * (1 + state.annual_growth);
        rothBal = (rothBal + conv) * (1 + state.annual_growth);
    }
    const totalTaxOverLadder = ladder.reduce((s, r) => s + r.tax_owed, 0);
    const totalConverted = ladder.reduce((s, r) => s + r.conversion, 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.roth.h2.summary">Optimal annual conversion</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.roth.card.annual_conversion">Annual conversion</div>
                    <div class="value">$${annualConversion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.roth.card.annual_tax">Annual tax</div>
                    <div class="value">$${annualTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.roth.card.effective_rate">Effective rate</div>
                    <div class="value">${annualConversion > 0 ? (annualTax / annualConversion * 100).toFixed(1) : '0.0'}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.roth.card.years">Years to convert all</div>
                    <div class="value">${Number.isFinite(yearsToFullConversion) ? yearsToFullConversion : '∞'}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.roth.card.target_top">Target top bracket</div>
                    <div class="value">${(state.target_top_bracket * 100).toFixed(0)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.roth.card.total_tax">Total tax across ladder</div>
                    <div class="value">$${totalTaxOverLadder.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.roth.h2.ladder">Ladder projection (next ${ladder.length} years)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.roth.th.age">Age</th>
                    <th data-i18n="view.roth.th.conversion">Conversion</th>
                    <th data-i18n="view.roth.th.tax">Tax owed</th>
                    <th data-i18n="view.roth.th.trad_balance">Trad balance</th>
                    <th data-i18n="view.roth.th.roth_balance">Roth balance</th>
                    <th data-i18n="view.roth.th.withdrawable_at">Penalty-free at age</th>
                </tr></thead>
                <tbody>${ladder.map(r => `
                    <tr>
                        <td><strong>${r.year}</strong></td>
                        <td class="pos">$${r.conversion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td class="neg">$${r.tax_owed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td>$${r.trad_balance.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td>$${r.roth_balance.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td class="muted">${r.withdrawable_at}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.roth.h2.notes">Strategy notes</h2>
            <ul class="muted small">
                <li data-i18n="view.roth.note.5yr">5-year seasoning per conversion: $50k converted in 2024 not withdrawable as principal until 2029</li>
                <li data-i18n="view.roth.note.early_retirement">FIRE strategy: convert during 50-65 gap years before SS / RMDs kick in</li>
                <li data-i18n="view.roth.note.tax_brackets">Best opportunities: $0-23k MFJ (10%), $23-94k MFJ (12%) — fill these annually</li>
                <li data-i18n="view.roth.note.irmaa">Watch IRMAA cliff at 65 — Medicare Part B/D surcharges trigger at MAGI tiers</li>
                <li data-i18n="view.roth.note.pay_tax_outside">Pay conversion tax from TAXABLE account, not the conversion itself, for max benefit</li>
                <li data-i18n="view.roth.note.tcja_sunset">2025 brackets reset higher post-TCJA — front-load conversions before then</li>
            </ul>
        </div>
    `;
}

function computeTax(taxableIncome) {
    let owe = 0;
    let lastCap = 0;
    for (const [cap, rate] of BRACKETS_2024_SINGLE) {
        const slice = Math.max(0, Math.min(taxableIncome, cap) - lastCap);
        owe += slice * rate;
        if (taxableIncome <= cap) break;
        lastCap = cap;
    }
    return owe;
}

function computeTaxOnConversion(conv) {
    const stdDeduction = state.filing === 'mfj' ? 29_200 : 14_600;
    const baseTax = computeTax(state.annual_other_income - stdDeduction);
    const withConversionTax = computeTax(state.annual_other_income + conv - stdDeduction);
    return Math.max(0, withConversionTax - Math.max(0, baseTax));
}
