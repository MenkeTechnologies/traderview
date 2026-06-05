// Backdoor Roth IRA Tracker.
// High earners (> 2024 single $161k / MFJ $240k MAGI) can't contribute directly
// to Roth. Workaround: contribute $7k to non-deductible Traditional IRA, then
// CONVERT to Roth. CATCH: pro-rata rule (§ 408(d)(2)) — if you have ANY
// pre-tax IRA dollars elsewhere, conversion is taxable on the pre-tax %.
// Form 8606 tracks basis. Solo 401(k) can hold pre-tax to keep IRAs clean.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const TRAD_IRA_LIMITS = {
    2024: { regular: 7_000, catchup_50: 1_000 },
    2025: { regular: 7_000, catchup_50: 1_000 },
    2026: { regular: 7_500, catchup_50: 1_000 },
};

let state = {
    year: new Date().getFullYear(),
    age: 35,
    contribution: 7_000,
    pretax_trad_ira_balance: 0,
    rollover_ira_balance: 0,
    sep_ira_balance: 0,
    simple_ira_balance: 0,
    nondeductible_basis: 0,  // running basis tracker
    your_marginal_rate: 0.32,
};

export async function renderBackdoorRoth(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.bdr.h1.title">// BACKDOOR ROTH</span></h1>
        <p class="muted small" data-i18n="view.bdr.hint.intro">
            High earners above MAGI phase-out can't contribute to Roth directly.
            Workaround: contribute non-deductible to Traditional IRA, then convert
            to Roth. <strong>Pro-rata trap:</strong> if you have ANY pre-tax IRA
            balance (Traditional / Rollover / SEP / SIMPLE — but NOT 401(k)),
            conversion is partly taxable. Form 8606 tracks basis annually.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.bdr.h2.inputs">Inputs</h2>
            <form id="bdr-form" class="inline-form">
                <label><span data-i18n="view.bdr.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}"></label>
                <label><span data-i18n="view.bdr.label.age">Age</span>
                    <input type="number" step="1" name="age" value="${state.age}" min="18" max="100"></label>
                <label><span data-i18n="view.bdr.label.contribution">Non-deductible contribution ($)</span>
                    <input type="number" step="0.01" name="contribution" value="${state.contribution}"></label>
                <label><span data-i18n="view.bdr.label.pretax_trad">Pre-tax Traditional IRA balance ($)</span>
                    <input type="number" step="0.01" name="pretax_trad_ira_balance" value="${state.pretax_trad_ira_balance}"></label>
                <label><span data-i18n="view.bdr.label.rollover">Rollover IRA balance ($)</span>
                    <input type="number" step="0.01" name="rollover_ira_balance" value="${state.rollover_ira_balance}"></label>
                <label><span data-i18n="view.bdr.label.sep">SEP IRA balance ($)</span>
                    <input type="number" step="0.01" name="sep_ira_balance" value="${state.sep_ira_balance}"></label>
                <label><span data-i18n="view.bdr.label.simple">SIMPLE IRA balance ($)</span>
                    <input type="number" step="0.01" name="simple_ira_balance" value="${state.simple_ira_balance}"></label>
                <label><span data-i18n="view.bdr.label.basis">Existing non-deductible basis ($)</span>
                    <input type="number" step="0.01" name="nondeductible_basis" value="${state.nondeductible_basis}"></label>
                <label><span data-i18n="view.bdr.label.marginal_rate">Marginal federal %</span>
                    <input type="number" step="0.5" name="your_marginal_rate" value="${(state.your_marginal_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.bdr.btn.compute">Compute</button>
            </form>
        </div>
        <div id="bdr-output"></div>
    `;
    document.getElementById('bdr-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.year = Number(fd.get('year'));
        state.age = Number(fd.get('age'));
        state.contribution = Number(fd.get('contribution')) || 0;
        state.pretax_trad_ira_balance = Number(fd.get('pretax_trad_ira_balance')) || 0;
        state.rollover_ira_balance = Number(fd.get('rollover_ira_balance')) || 0;
        state.sep_ira_balance = Number(fd.get('sep_ira_balance')) || 0;
        state.simple_ira_balance = Number(fd.get('simple_ira_balance')) || 0;
        state.nondeductible_basis = Number(fd.get('nondeductible_basis')) || 0;
        state.your_marginal_rate = (Number(fd.get('your_marginal_rate')) || 32) / 100;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('bdr-output');
    if (!el) return;
    const limits = TRAD_IRA_LIMITS[state.year] || TRAD_IRA_LIMITS[2024];
    const maxContrib = limits.regular + (state.age >= 50 ? limits.catchup_50 : 0);
    const overContrib = Math.max(0, state.contribution - maxContrib);

    // Pro-rata calculation
    const pretaxTotal = state.pretax_trad_ira_balance + state.rollover_ira_balance
        + state.sep_ira_balance + state.simple_ira_balance;
    const totalIraValue = pretaxTotal + state.contribution + state.nondeductible_basis;
    const basisAfterContrib = state.nondeductible_basis + state.contribution;
    const conversionAmount = state.contribution;  // We're converting just the new contribution
    const basisRatio = totalIraValue > 0 ? basisAfterContrib / totalIraValue : 1;
    const nontaxableConversion = conversionAmount * basisRatio;
    const taxableConversion = conversionAmount - nontaxableConversion;
    const conversionTax = taxableConversion * state.your_marginal_rate;
    const cleanBackdoor = pretaxTotal === 0;
    const remainingBasis = basisAfterContrib - nontaxableConversion;
    const cls = cleanBackdoor ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel ${cls}">
            <h2 data-i18n="view.bdr.h2.result">Conversion result</h2>
            <div class="cards">
                <div class="card ${cleanBackdoor ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.bdr.card.clean">Clean backdoor?</div>
                    <div class="value">${cleanBackdoor ? esc(t('view.bdr.status.yes')) : esc(t('view.bdr.status.no_pro_rata'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.bdr.card.nontaxable">Non-taxable conversion</div>
                    <div class="value">$${nontaxableConversion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.bdr.card.taxable">Taxable conversion (pro-rata)</div>
                    <div class="value">$${taxableConversion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.bdr.card.tax_owed">Tax owed on conversion</div>
                    <div class="value">$${conversionTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.bdr.card.basis_ratio">Basis ratio</div>
                    <div class="value">${(basisRatio * 100).toFixed(1)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.bdr.card.remaining_basis">Remaining basis (carryforward)</div>
                    <div class="value">$${remainingBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${overContrib > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.bdr.card.over_contrib">Over-contribution (6% excise)</div>
                        <div class="value">$${overContrib.toLocaleString()}</div>
                    </div>
                ` : ''}
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.bdr.h2.cleanup">Fixing the pro-rata trap</h2>
            <ol class="muted small">
                <li data-i18n="view.bdr.cleanup.solo_401k">Roll pre-tax IRA balances INTO your Solo 401(k) or employer 401(k) — 401(k) NOT included in pro-rata calc</li>
                <li data-i18n="view.bdr.cleanup.conversion_full">Or fully convert pre-tax IRAs to Roth (taxable, but cleans the slate permanently)</li>
                <li data-i18n="view.bdr.cleanup.spousal">Spousal IRAs are SEPARATE — your spouse's pre-tax balance doesn't affect yours</li>
                <li data-i18n="view.bdr.cleanup.year_end">Year-end balance is what matters for pro-rata — not contribution date</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.bdr.h2.mechanics">Mechanics</h2>
            <ol class="muted small">
                <li data-i18n="view.bdr.step.contribute">Contribute up to $${maxContrib.toLocaleString()} to Traditional IRA (mark non-deductible)</li>
                <li data-i18n="view.bdr.step.convert">Within days: convert that $${state.contribution.toLocaleString()} to Roth IRA</li>
                <li data-i18n="view.bdr.step.form_8606">File Form 8606 to track basis (must file EVERY year you have basis, lifetime!)</li>
                <li data-i18n="view.bdr.step.step_transaction">No "step-transaction doctrine" enforcement to date — IRS has acquiesced</li>
                <li data-i18n="view.bdr.step.repeat_annually">Repeat annually — total tax-free Roth space accumulated over decades</li>
            </ol>
        </div>
    `;
}
