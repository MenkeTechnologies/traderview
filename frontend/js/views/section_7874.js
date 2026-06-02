// IRC § 7874 — Anti-Inversion Rules.
// Foreign acquirer of US corp treated as US corp if former US shareholders own 80%+ → surrogate.
// 60-79% ownership: still treated as foreign but lose tax benefits ("inversion gain" taxed).
// Substantial business activities test in foreign jurisdiction (25% test).
// 2012 + 2016 regs: serial inversions + 3-year lookback to prevent stock-buildup workarounds.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    foreign_acquirer_us_shareholder_pct: 0,
    sba_test_country: '',
    sba_employees_pct: 0,
    sba_assets_pct: 0,
    sba_income_pct: 0,
    inversion_gain: 0,
    foreign_acquirer_jurisdiction: 'ireland',
    same_industry_acquisition: false,
};

export async function renderSection7874(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s7874.h1.title">// § 7874 ANTI-INVERSION</span></h1>
        <p class="muted small" data-i18n="view.s7874.hint.intro">
            Foreign acquirer of US corp treated as US corp ("surrogate foreign corporation") if former US
            shareholders own <strong>80%+</strong>. <strong>60-79%:</strong> stays foreign but loses tax
            benefits ("inversion gain" taxed at 35% — § 4985 also taxes officers + directors 20% excise).
            <strong>SBA exception:</strong> 25% employees + assets + income in foreign jurisdiction.
            <strong>2016 regs:</strong> 3-year lookback on serial inversions.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s7874.h2.inputs">Inputs</h2>
            <form id="s7874-form" class="inline-form">
                <label><span data-i18n="view.s7874.label.us_pct">Former US shareholder % of foreign acquirer</span>
                    <input type="number" step="0.01" name="foreign_acquirer_us_shareholder_pct" value="${state.foreign_acquirer_us_shareholder_pct}"></label>
                <label><span data-i18n="view.s7874.label.country">SBA test country</span>
                    <input type="text" name="sba_test_country" value="${esc(state.sba_test_country)}"></label>
                <label><span data-i18n="view.s7874.label.employees">SBA: employees % in foreign country</span>
                    <input type="number" step="0.1" name="sba_employees_pct" value="${state.sba_employees_pct}"></label>
                <label><span data-i18n="view.s7874.label.assets">SBA: assets % in foreign country</span>
                    <input type="number" step="0.1" name="sba_assets_pct" value="${state.sba_assets_pct}"></label>
                <label><span data-i18n="view.s7874.label.income">SBA: income % in foreign country</span>
                    <input type="number" step="0.1" name="sba_income_pct" value="${state.sba_income_pct}"></label>
                <label><span data-i18n="view.s7874.label.inversion_gain">Inversion gain (for 60-79%) ($)</span>
                    <input type="number" step="10000" name="inversion_gain" value="${state.inversion_gain}"></label>
                <label><span data-i18n="view.s7874.label.jurisdiction">Foreign acquirer jurisdiction</span>
                    <select name="foreign_acquirer_jurisdiction">
                        <option value="ireland" ${state.foreign_acquirer_jurisdiction === 'ireland' ? 'selected' : ''}>Ireland</option>
                        <option value="bermuda" ${state.foreign_acquirer_jurisdiction === 'bermuda' ? 'selected' : ''}>Bermuda</option>
                        <option value="netherlands" ${state.foreign_acquirer_jurisdiction === 'netherlands' ? 'selected' : ''}>Netherlands</option>
                        <option value="uk" ${state.foreign_acquirer_jurisdiction === 'uk' ? 'selected' : ''}>UK</option>
                        <option value="switzerland" ${state.foreign_acquirer_jurisdiction === 'switzerland' ? 'selected' : ''}>Switzerland</option>
                        <option value="other" ${state.foreign_acquirer_jurisdiction === 'other' ? 'selected' : ''}>Other</option>
                    </select>
                </label>
                <label><span data-i18n="view.s7874.label.same_industry">Same industry acquisition?</span>
                    <input type="checkbox" name="same_industry_acquisition" ${state.same_industry_acquisition ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s7874.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s7874-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7874.h2.consequences">Inversion consequences</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s7874.th.pct">Former US shareholder %</th>
                    <th data-i18n="view.s7874.th.treatment">Treatment</th>
                    <th data-i18n="view.s7874.th.consequences">Consequences</th>
                </tr></thead>
                <tbody>
                    <tr><td>≥ 80%</td><td>Treated as US corp (surrogate)</td><td>Full US tax on worldwide income; foreign tax holiday lost</td></tr>
                    <tr><td>60-79%</td><td>Inversion w/ tax penalties</td><td>Inversion gain taxed @ 35%; § 4985 20% excise on officers; loss of NOLs / credits</td></tr>
                    <tr><td>&lt; 60%</td><td>Genuine foreign acquisition</td><td>Normal foreign corp treatment</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7874.h2.sba">Substantial Business Activities (SBA) exception</h2>
            <ul class="muted small">
                <li data-i18n="view.s7874.sba.employees">Employees: ≥ 25% in foreign country (count + comp)</li>
                <li data-i18n="view.s7874.sba.assets">Assets: ≥ 25% located in foreign country</li>
                <li data-i18n="view.s7874.sba.income">Income: ≥ 25% derived in foreign country</li>
                <li data-i18n="view.s7874.sba.all_three">ALL THREE prongs required — not 2 of 3</li>
                <li data-i18n="view.s7874.sba.real_business">Real operating business, not shell company</li>
                <li data-i18n="view.s7874.sba.tax_residency">Tax residency in test country (not flag of convenience)</li>
                <li data-i18n="view.s7874.sba.bermuda_caps">Bermuda + Caymans: SBA almost impossible to satisfy due to no real economy</li>
                <li data-i18n="view.s7874.sba.ireland_potential">Ireland + Netherlands: real ops possible due to economic substance</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7874.h2.history">2014-2016 regulatory tightening</h2>
            <ul class="muted small">
                <li data-i18n="view.s7874.hist.notice2014_52">Notice 2014-52 attacked cash-box mergers + stuffing transactions</li>
                <li data-i18n="view.s7874.hist.notice2015_79">Notice 2015-79 expanded definitions of "domestic entity"</li>
                <li data-i18n="view.s7874.hist.regs2016">2016 final + temp regs: 3-year lookback on serial inversions, killed Pfizer-Allergan</li>
                <li data-i18n="view.s7874.hist.dba">Disqualified stock + Distributions</li>
                <li data-i18n="view.s7874.hist.passive">Passive asset rules</li>
                <li data-i18n="view.s7874.hist.modern">Modern inversions rare due to combined § 245A repatriation + GILTI minimum</li>
            </ul>
        </div>
    `;
    document.getElementById('s7874-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.foreign_acquirer_us_shareholder_pct = Number(fd.get('foreign_acquirer_us_shareholder_pct')) || 0;
        state.sba_test_country = fd.get('sba_test_country');
        state.sba_employees_pct = Number(fd.get('sba_employees_pct')) || 0;
        state.sba_assets_pct = Number(fd.get('sba_assets_pct')) || 0;
        state.sba_income_pct = Number(fd.get('sba_income_pct')) || 0;
        state.inversion_gain = Number(fd.get('inversion_gain')) || 0;
        state.foreign_acquirer_jurisdiction = fd.get('foreign_acquirer_jurisdiction');
        state.same_industry_acquisition = !!fd.get('same_industry_acquisition');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s7874-output');
    if (!el) return;
    const sbaQualifies = state.sba_employees_pct >= 25 && state.sba_assets_pct >= 25 && state.sba_income_pct >= 25;
    let treatment = 'foreign';
    let category = 'genuine';
    const pct = state.foreign_acquirer_us_shareholder_pct;
    if (pct >= 80 && !sbaQualifies) {
        treatment = 'surrogate_us';
        category = 'surrogate';
    } else if (pct >= 60 && pct < 80 && !sbaQualifies) {
        treatment = 'inversion_penalized';
        category = 'penalized';
    }
    const inversionGainTax = treatment === 'inversion_penalized' ? state.inversion_gain * 0.35 : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s7874.h2.result">§ 7874 analysis</h2>
            <div class="cards">
                <div class="card ${sbaQualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s7874.card.sba">SBA exception qualifies?</div>
                    <div class="value">${sbaQualifies ? esc(t('view.s7874.status.yes')) : esc(t('view.s7874.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7874.card.pct">Former US shareholder %</div>
                    <div class="value">${pct.toFixed(2)}%</div>
                </div>
                <div class="card ${category === 'surrogate' || category === 'penalized' ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s7874.card.treatment">Treatment</div>
                    <div class="value">${esc(t('view.s7874.treat.' + treatment))}</div>
                </div>
                ${treatment === 'inversion_penalized' ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s7874.card.gain_tax">Inversion gain tax (35%)</div>
                        <div class="value">$${inversionGainTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
            </div>
            ${treatment === 'surrogate_us' ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s7874.surrogate_note">
                    Foreign acquirer treated as US corp for ALL purposes — worldwide income subject to US tax,
                    no treaty benefits, dividends from foreign subs taxed without § 245A. Anti-inversion penalty.
                </p>
            ` : ''}
        </div>
    `;
}
