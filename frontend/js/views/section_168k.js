// IRC § 168(k) Bonus Depreciation — TCJA phase-down schedule.
// 2017-2022: 100%. 2023: 80%. 2024: 60%. 2025: 40%. 2026: 20%. 2027+: 0%.
// Property must be tangible, MACRS < 20 yr, AND placed in service in calendar year.
// "Qualified Improvement Property" (QIP) eligible after CARES Act fix.
// Election OUT made class-by-class (3-yr / 5-yr / 7-yr / ...) on timely return.

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const BONUS_BY_YEAR = {
    2017: 1.00, 2018: 1.00, 2019: 1.00, 2020: 1.00, 2021: 1.00, 2022: 1.00,
    2023: 0.80, 2024: 0.60, 2025: 0.40, 2026: 0.20, 2027: 0.00,
};

let state = {
    placed_year: new Date().getFullYear(),
    asset_cost: 0,
    asset_class_years: 5,
    business_use_pct: 100,
    elect_out: false,
    elect_section_179: false,
    section_179_amount: 0,
    marginal_rate: 0.32,
};

export async function renderSection168k(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s168k.h1.title">// § 168(k) BONUS DEPRECIATION</span></h1>
        <p class="muted small" data-i18n="view.s168k.hint.intro">
            TCJA phase-down: <strong>100% (2017-22) → 80% (2023) → 60% (2024) → 40% (2025)
            → 20% (2026) → 0% (2027+)</strong>. Property must be tangible, MACRS &lt; 20 yr
            recovery, placed in service in the calendar year. Qualified Improvement Property
            (QIP) now eligible after CARES Act fix. Election OUT made class-by-class on timely
            return. Stacks under § 179 (which goes first).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s168k.h2.inputs">Inputs</h2>
            <form id="s168k-form" class="inline-form">
                <label><span data-i18n="view.s168k.label.placed_year">Placed in service year</span>
                    <input type="number" step="1" name="placed_year" value="${state.placed_year}"></label>
                <label><span data-i18n="view.s168k.label.cost">Asset cost ($)</span>
                    <input type="number" step="100" name="asset_cost" value="${state.asset_cost}"></label>
                <label><span data-i18n="view.s168k.label.class_years">MACRS class (years)</span>
                    <select name="asset_class_years">
                        <option value="3" ${state.asset_class_years === 3 ? 'selected' : ''}>3 (tools, racehorses)</option>
                        <option value="5" ${state.asset_class_years === 5 ? 'selected' : ''}>5 (computers, autos)</option>
                        <option value="7" ${state.asset_class_years === 7 ? 'selected' : ''}>7 (office furniture)</option>
                        <option value="15" ${state.asset_class_years === 15 ? 'selected' : ''}>15 (QIP, land improvements)</option>
                        <option value="20" ${state.asset_class_years === 20 ? 'selected' : ''}>20 (farm buildings)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s168k.label.business_use">Business use %</span>
                    <input type="number" step="1" name="business_use_pct" value="${state.business_use_pct}"></label>
                <label><span data-i18n="view.s168k.label.elect_out">Elect OUT of bonus?</span>
                    <input type="checkbox" name="elect_out" ${state.elect_out ? 'checked' : ''}></label>
                <label><span data-i18n="view.s168k.label.elect_179">Take § 179 first?</span>
                    <input type="checkbox" name="elect_section_179" ${state.elect_section_179 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s168k.label.179_amount">§ 179 amount ($)</span>
                    <input type="number" step="100" name="section_179_amount" value="${state.section_179_amount}"></label>
                <label><span data-i18n="view.s168k.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s168k.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s168k-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s168k.h2.phase_table">Phase-down table</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s168k.th.year">Year</th>
                    <th data-i18n="view.s168k.th.bonus">Bonus %</th>
                    <th data-i18n="view.s168k.th.notes">Notes</th>
                </tr></thead>
                <tbody>
                    <tr><td>2017-2022</td><td class="pos">100%</td><td data-i18n="view.s168k.note.tcja_max">TCJA full expensing era</td></tr>
                    <tr><td>2023</td><td>80%</td><td data-i18n="view.s168k.note.first_step">First step-down</td></tr>
                    <tr><td>2024</td><td>60%</td><td data-i18n="view.s168k.note.current">Current year (assumed)</td></tr>
                    <tr><td>2025</td><td>40%</td><td data-i18n="view.s168k.note.next">Next year</td></tr>
                    <tr><td>2026</td><td>20%</td><td data-i18n="view.s168k.note.almost_gone">Almost gone</td></tr>
                    <tr><td>2027+</td><td class="neg">0%</td><td data-i18n="view.s168k.note.expired">Expired (unless Congress extends)</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s168k-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.placed_year = Number(fd.get('placed_year')) || new Date().getFullYear();
        state.asset_cost = Number(fd.get('asset_cost')) || 0;
        state.asset_class_years = Number(fd.get('asset_class_years')) || 5;
        state.business_use_pct = Number(fd.get('business_use_pct')) || 100;
        state.elect_out = !!fd.get('elect_out');
        state.elect_section_179 = !!fd.get('elect_section_179');
        state.section_179_amount = Number(fd.get('section_179_amount')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function macrsRate(year, classYrs) {
    // Simplified 200% DB / mid-year convention rates
    const tables = {
        3: [0.3333, 0.4445, 0.1481, 0.0741],
        5: [0.20, 0.32, 0.192, 0.1152, 0.1152, 0.0576],
        7: [0.1429, 0.2449, 0.1749, 0.1249, 0.0893, 0.0892, 0.0893, 0.0446],
        15: [0.05, 0.095, 0.0855, 0.077, 0.0693, 0.0623, 0.059, 0.059, 0.0591, 0.059, 0.0591, 0.059, 0.0591, 0.059, 0.0591, 0.0295],
        20: [0.0375, 0.07219, 0.06677, 0.06177, 0.05713, 0.05285, 0.04888, 0.04522, 0.04462, 0.04461],
    };
    const t = tables[classYrs] || tables[5];
    return t[year - 1] || 0;
}

function renderOutput() {
    const el = document.getElementById('s168k-output');
    if (!el) return;
    const bonusPct = state.elect_out ? 0 : (BONUS_BY_YEAR[state.placed_year] ?? 0);
    const businessShare = state.business_use_pct / 100;
    const businessBasis = state.asset_cost * businessShare;
    const after179 = Math.max(0, businessBasis - (state.elect_section_179 ? state.section_179_amount : 0));
    const bonus = after179 * bonusPct;
    const afterBonus = after179 - bonus;
    const macrs1 = afterBonus * macrsRate(1, state.asset_class_years);
    const year1Total = (state.elect_section_179 ? state.section_179_amount : 0) + bonus + macrs1;
    const year1Savings = year1Total * state.marginal_rate;
    const yearly = [];
    let remaining = afterBonus;
    for (let y = 1; y <= state.asset_class_years + 1; y++) {
        const rate = macrsRate(y, state.asset_class_years);
        const dep = afterBonus * rate;
        remaining -= dep;
        yearly.push({ y, dep });
        if (rate === 0) break;
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s168k.h2.result">Year-1 deduction</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s168k.card.bonus_pct">Bonus %</div>
                    <div class="value">${(bonusPct * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s168k.card.business_basis">Business basis</div>
                    <div class="value">$${businessBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s168k.card.bonus_amount">Bonus depreciation</div>
                    <div class="value">$${bonus.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s168k.card.macrs_y1">MACRS year-1</div>
                    <div class="value">$${macrs1.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s168k.card.y1_total">Year-1 total</div>
                    <div class="value">$${year1Total.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s168k.card.y1_savings">Year-1 tax savings</div>
                    <div class="value">$${year1Savings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s168k.h2.yearly">MACRS schedule (after bonus)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s168k.th.year_n">Year</th>
                    <th data-i18n="view.s168k.th.macrs_dep">MACRS depreciation</th>
                </tr></thead>
                <tbody>${yearly.map(r => `
                    <tr>
                        <td>${r.y}</td>
                        <td>$${r.dep.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        </div>
    `;
}
