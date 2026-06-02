// Inherited IRA RMD Calculator — post-SECURE Act (2020+) + SECURE 2.0 (2023+).
// Non-spouse non-eligible beneficiaries (most kids/grandkids): 10-year rule.
// Spouse: can elect treat as own OR 10-yr rule OR life expectancy.
// "Eligible Designated Beneficiaries" (EDB): disabled, chronically ill,
// minor, less-than-10-yr-younger sibling → life expectancy table.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const SINGLE_LIFE_TABLE = {
    20: 65.0, 25: 60.2, 30: 55.3, 35: 50.5, 40: 45.7, 45: 41.0,
    50: 36.2, 55: 31.6, 60: 27.1, 65: 22.9, 70: 18.8, 75: 14.8,
    80: 11.2,  85: 8.1,  90: 5.5,  95: 3.6,  100: 2.5,
};

let state = {
    beneficiary_type: '10_year',  // 10_year | spouse_own | spouse_inherited | edb_life
    beneficiary_age_at_inheritance: 45,
    decedent_age: 75,
    decedent_was_taking_rmds: true,
    inherited_balance: 500_000,
    inheritance_year: new Date().getFullYear(),
    expected_growth: 0.07,
};

export async function renderInheritedIraRmd(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.iir.h1.title">// INHERITED IRA RMD</span></h1>
        <p class="muted small" data-i18n="view.iir.hint.intro">
            Post-SECURE Act rules: non-spouse non-EDB beneficiaries must empty inherited
            IRA within 10 years. SECURE 2.0 clarified: if decedent was already taking
            RMDs at death, beneficiary must also take annual RMDs in years 1-9 AND
            empty by year 10. Spouses + EDBs (disabled, chronically ill, minor children,
            siblings &lt; 10 yrs younger) can use life-expectancy stretch.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.iir.h2.inputs">Inputs</h2>
            <form id="iir-form" class="inline-form">
                <label><span data-i18n="view.iir.label.beneficiary_type">Beneficiary type</span>
                    <select name="beneficiary_type">
                        <option value="10_year" ${state.beneficiary_type === '10_year' ? 'selected' : ''}>Non-spouse / non-EDB (10-yr rule)</option>
                        <option value="spouse_own" ${state.beneficiary_type === 'spouse_own' ? 'selected' : ''}>Spouse — treat as own</option>
                        <option value="spouse_inherited" ${state.beneficiary_type === 'spouse_inherited' ? 'selected' : ''}>Spouse — keep as inherited</option>
                        <option value="edb_life" ${state.beneficiary_type === 'edb_life' ? 'selected' : ''}>Eligible Designated Beneficiary (life expectancy)</option>
                    </select>
                </label>
                <label><span data-i18n="view.iir.label.benef_age">Beneficiary age at inheritance</span>
                    <input type="number" step="1" name="beneficiary_age_at_inheritance" value="${state.beneficiary_age_at_inheritance}" min="0" max="100"></label>
                <label><span data-i18n="view.iir.label.decedent_age">Decedent age at death</span>
                    <input type="number" step="1" name="decedent_age" value="${state.decedent_age}" min="18" max="120"></label>
                <label><span data-i18n="view.iir.label.was_taking_rmds">Decedent was taking RMDs?</span>
                    <input type="checkbox" name="decedent_was_taking_rmds" ${state.decedent_was_taking_rmds ? 'checked' : ''}></label>
                <label><span data-i18n="view.iir.label.balance">Inherited balance ($)</span>
                    <input type="number" step="1000" name="inherited_balance" value="${state.inherited_balance}"></label>
                <label><span data-i18n="view.iir.label.inheritance_year">Year of inheritance</span>
                    <input type="number" step="1" name="inheritance_year" value="${state.inheritance_year}"></label>
                <label><span data-i18n="view.iir.label.growth">Expected annual growth %</span>
                    <input type="number" step="0.5" name="expected_growth" value="${(state.expected_growth * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.iir.btn.compute">Compute</button>
            </form>
        </div>
        <div id="iir-output"></div>
    `;
    document.getElementById('iir-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.beneficiary_type = fd.get('beneficiary_type');
        state.beneficiary_age_at_inheritance = Number(fd.get('beneficiary_age_at_inheritance'));
        state.decedent_age = Number(fd.get('decedent_age'));
        state.decedent_was_taking_rmds = !!fd.get('decedent_was_taking_rmds');
        state.inherited_balance = Number(fd.get('inherited_balance')) || 0;
        state.inheritance_year = Number(fd.get('inheritance_year'));
        state.expected_growth = (Number(fd.get('expected_growth')) || 7) / 100;
        renderOutput();
    });
    renderOutput();
}

function lifeExpectancyAt(age) {
    const buckets = Object.keys(SINGLE_LIFE_TABLE).map(Number).sort((a, b) => a - b);
    const lower = buckets.filter(b => b <= age).pop();
    return SINGLE_LIFE_TABLE[lower || buckets[0]] || 30;
}

function renderOutput() {
    const el = document.getElementById('iir-output');
    if (!el) return;
    const schedule = [];
    let balance = state.inherited_balance;
    const isLifeExpectancy = state.beneficiary_type === 'edb_life' || state.beneficiary_type === 'spouse_inherited';
    const isTenYear = state.beneficiary_type === '10_year';
    const isSpouseOwn = state.beneficiary_type === 'spouse_own';
    const annualRmdRequired = isLifeExpectancy
        || (isTenYear && state.decedent_was_taking_rmds);
    const maxYears = isTenYear ? 10
        : isLifeExpectancy ? Math.ceil(lifeExpectancyAt(state.beneficiary_age_at_inheritance))
        : 40;  // spouse-own: defer until 73
    const startAge = state.beneficiary_age_at_inheritance;
    for (let y = 0; y < Math.min(20, maxYears); y++) {
        let factor = 0;
        let rmd = 0;
        if (annualRmdRequired) {
            const currentAge = startAge + y;
            factor = lifeExpectancyAt(currentAge) - y;
            if (factor > 0) {
                rmd = balance / factor;
            }
            if (isTenYear && y === 9) {
                rmd = balance;  // empty by year 10
            }
        } else if (isTenYear && y === 9) {
            rmd = balance;
        }
        schedule.push({
            year: state.inheritance_year + y,
            age: state.beneficiary_age_at_inheritance + y,
            beginning_balance: balance,
            factor,
            rmd,
            ending_balance: Math.max(0, (balance - rmd) * (1 + state.expected_growth)),
        });
        balance = schedule[schedule.length - 1].ending_balance;
        if (balance <= 0 || (isTenYear && y === 9)) break;
    }
    const totalDistributions = schedule.reduce((s, r) => s + r.rmd, 0);
    const finalBalance = schedule.length > 0 ? schedule[schedule.length - 1].ending_balance : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.iir.h2.regime">Applicable regime</h2>
            <p><strong>${esc(regimeLabel())}</strong></p>
            <p class="muted small">${esc(regimeDescription())}</p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.iir.h2.summary">Summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.iir.card.starting">Starting balance</div>
                    <div class="value">$${state.inherited_balance.toLocaleString()}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.iir.card.total_distributions">Total distributions</div>
                    <div class="value">$${totalDistributions.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.iir.card.final_balance">Final balance</div>
                    <div class="value">$${finalBalance.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.iir.card.schedule_years">Schedule length</div>
                    <div class="value">${schedule.length} ${esc(t('view.iir.years'))}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.iir.h2.schedule">RMD schedule</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.iir.th.year">Year</th>
                    <th data-i18n="view.iir.th.age">Age</th>
                    <th data-i18n="view.iir.th.beginning">Beginning</th>
                    <th data-i18n="view.iir.th.factor">Factor</th>
                    <th data-i18n="view.iir.th.rmd">RMD</th>
                    <th data-i18n="view.iir.th.ending">Ending</th>
                </tr></thead>
                <tbody>${schedule.map(r => `
                    <tr>
                        <td>${r.year}</td>
                        <td>${r.age}</td>
                        <td>$${r.beginning_balance.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td>${r.factor.toFixed(1)}</td>
                        <td class="neg">$${r.rmd.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td>$${r.ending_balance.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        </div>
    `;
}

function regimeLabel() {
    switch (state.beneficiary_type) {
        case '10_year': return t('view.iir.regime.10_year');
        case 'spouse_own': return t('view.iir.regime.spouse_own');
        case 'spouse_inherited': return t('view.iir.regime.spouse_inherited');
        case 'edb_life': return t('view.iir.regime.edb_life');
        default: return '';
    }
}
function regimeDescription() {
    switch (state.beneficiary_type) {
        case '10_year': return t('view.iir.desc.10_year');
        case 'spouse_own': return t('view.iir.desc.spouse_own');
        case 'spouse_inherited': return t('view.iir.desc.spouse_inherited');
        case 'edb_life': return t('view.iir.desc.edb_life');
        default: return '';
    }
}
