// IRC § 1291 — PFIC Excess Distribution (Default Punitive Regime).
// Passive Foreign Investment Co: 75% income passive OR 50% assets passive.
// Default regime: excess distributions taxed at HIGHEST ordinary rate + INTEREST CHARGE.
// "Excess distribution" = current dist > 125% of prior 3-yr avg; gain on sale = excess.
// Avoid via QEF election (§ 1295) or MTM election (§ 1296).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    current_distribution: 0,
    prior_3yr_avg: 0,
    gain_on_sale: 0,
    holding_period_years: 0,
    federal_short_term_rate: 5.0,
    elects_qef: false,
    elects_mtm: false,
    has_purged: false,
    pfic_test_income_pct: 0,
    pfic_test_assets_pct: 0,
};

export async function renderSection1291(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1291.h1.title">// § 1291 PFIC EXCESS DIST.</span></h1>
        <p class="muted small" data-i18n="view.s1291.hint.intro">
            <strong>PFIC test:</strong> 75% passive income OR 50% passive assets (FMV avg).
            <strong>Default regime:</strong> "excess distributions" taxed at <strong>HIGHEST ORDINARY RATE</strong>
            + <strong>INTEREST CHARGE</strong> back to start of holding period. <strong>Excess distribution</strong>
            = current dist &gt; 125% of prior 3-yr avg. <strong>Gain on sale = entirely excess</strong> (no LTCG
            rate). Avoid via <strong>QEF</strong> election (§ 1295 = current inclusion) or <strong>MTM</strong>
            (§ 1296 = mark-to-market). Form 8621.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1291.h2.inputs">Inputs</h2>
            <form id="s1291-form" class="inline-form">
                <label><span data-i18n="view.s1291.label.current">Current distribution ($)</span>
                    <input type="number" step="0.01" name="current_distribution" value="${state.current_distribution}"></label>
                <label><span data-i18n="view.s1291.label.prior">Prior 3-yr avg distribution ($)</span>
                    <input type="number" step="0.01" name="prior_3yr_avg" value="${state.prior_3yr_avg}"></label>
                <label><span data-i18n="view.s1291.label.gain">Gain on sale ($)</span>
                    <input type="number" step="0.01" name="gain_on_sale" value="${state.gain_on_sale}"></label>
                <label><span data-i18n="view.s1291.label.holding">Holding period years</span>
                    <input type="number" step="1" name="holding_period_years" value="${state.holding_period_years}"></label>
                <label><span data-i18n="view.s1291.label.short_term">Fed short-term rate %</span>
                    <input type="number" step="0.1" name="federal_short_term_rate" value="${state.federal_short_term_rate}"></label>
                <label><span data-i18n="view.s1291.label.qef">QEF election in place?</span>
                    <input type="checkbox" name="elects_qef" ${state.elects_qef ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1291.label.mtm">MTM election in place?</span>
                    <input type="checkbox" name="elects_mtm" ${state.elects_mtm ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1291.label.purged">Purged § 1291 taint?</span>
                    <input type="checkbox" name="has_purged" ${state.has_purged ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1291.label.income_pct">Passive income % of total</span>
                    <input type="number" step="0.1" name="pfic_test_income_pct" value="${state.pfic_test_income_pct}"></label>
                <label><span data-i18n="view.s1291.label.assets_pct">Passive assets % FMV</span>
                    <input type="number" step="0.1" name="pfic_test_assets_pct" value="${state.pfic_test_assets_pct}"></label>
                <button class="primary" type="submit" data-i18n="view.s1291.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1291-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1291.h2.pfic_test">PFIC determination (§ 1297)</h2>
            <ul class="muted small">
                <li data-i18n="view.s1291.test.income">Income test: ≥ 75% of gross income is passive (interest, div, royalties, rents, gains)</li>
                <li data-i18n="view.s1291.test.assets">Asset test: ≥ 50% of FMV avg assets produce passive income or held for production thereof</li>
                <li data-i18n="view.s1291.test.either">EITHER test triggers PFIC status (not both required)</li>
                <li data-i18n="view.s1291.test.lookthrough">≥ 25%-owned sub treated as look-through (PFIC attribution)</li>
                <li data-i18n="view.s1291.test.startup">Start-up exception: first year of new corp may be exempt</li>
                <li data-i18n="view.s1291.test.once">"Once a PFIC, always a PFIC" — taint persists in shareholder's hands across years</li>
                <li data-i18n="view.s1291.test.cfc_overlap">CFC overlap: CFC + PFIC overlap rules exclude CFC US shareholders from PFIC regime</li>
                <li data-i18n="view.s1291.test.foreign_fund">Foreign mutual funds + ETFs almost always PFICs</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1291.h2.election_compare">Compare three regimes</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s1291.th.regime">Regime</th>
                    <th data-i18n="view.s1291.th.timing">Timing</th>
                    <th data-i18n="view.s1291.th.rate">Rate</th>
                    <th data-i18n="view.s1291.th.interest">Interest charge</th>
                </tr></thead>
                <tbody>
                    <tr><td>§ 1291 Default</td><td>On distribution / sale</td><td>Top ordinary 37%</td><td>YES — back to start</td></tr>
                    <tr><td>§ 1295 QEF</td><td>Annual current inclusion</td><td>Ordinary / capital separately</td><td>NO</td></tr>
                    <tr><td>§ 1296 MTM</td><td>Annual mark-to-market</td><td>Ordinary on gain; ordinary loss to extent prior MTM gains</td><td>NO</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1291.h2.purge">Purging § 1291 taint</h2>
            <ul class="muted small">
                <li data-i18n="view.s1291.purge.deemed">"Deemed sale" purging election: recognize gain currently at § 1291 rates → then QEF / MTM clean</li>
                <li data-i18n="view.s1291.purge.late">Late QEF election: also requires purging via deemed sale</li>
                <li data-i18n="view.s1291.purge.no_loss">Cannot recognize LOSS on purging (asymmetric)</li>
                <li data-i18n="view.s1291.purge.pre_pfic">Pre-PFIC years not counted for interest charge (gain allocated to PFIC years only)</li>
                <li data-i18n="view.s1291.purge.allocate">Allocation: gain pro-rated DAY-BY-DAY across holding period</li>
                <li data-i18n="view.s1291.purge.current">Portion allocated to CURRENT year taxed at current top ordinary, no interest</li>
            </ul>
        </div>
    `;
    document.getElementById('s1291-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.current_distribution = Number(fd.get('current_distribution')) || 0;
        state.prior_3yr_avg = Number(fd.get('prior_3yr_avg')) || 0;
        state.gain_on_sale = Number(fd.get('gain_on_sale')) || 0;
        state.holding_period_years = Number(fd.get('holding_period_years')) || 0;
        state.federal_short_term_rate = Number(fd.get('federal_short_term_rate')) || 0;
        state.elects_qef = !!fd.get('elects_qef');
        state.elects_mtm = !!fd.get('elects_mtm');
        state.has_purged = !!fd.get('has_purged');
        state.pfic_test_income_pct = Number(fd.get('pfic_test_income_pct')) || 0;
        state.pfic_test_assets_pct = Number(fd.get('pfic_test_assets_pct')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1291-output');
    if (!el) return;
    const isPFIC = state.pfic_test_income_pct >= 75 || state.pfic_test_assets_pct >= 50;
    const inElection = state.elects_qef || state.elects_mtm;
    const default1291 = isPFIC && !inElection && !state.has_purged;
    const excessDist = Math.max(0, state.current_distribution - 1.25 * state.prior_3yr_avg);
    const totalExcess = excessDist + state.gain_on_sale;
    const allocPerYear = state.holding_period_years > 0 ? totalExcess / state.holding_period_years : 0;
    const ordinaryRate = 0.37;
    const interestRate = state.federal_short_term_rate / 100 + 0.03;
    let interestCharge = 0;
    if (state.holding_period_years > 1 && default1291) {
        for (let y = 0; y < state.holding_period_years - 1; y++) {
            const taxOnAlloc = allocPerYear * ordinaryRate;
            interestCharge += taxOnAlloc * interestRate * (state.holding_period_years - 1 - y);
        }
    }
    const baseTax = default1291 ? totalExcess * ordinaryRate : 0;
    const totalCost = baseTax + interestCharge;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1291.h2.result">§ 1291 computation</h2>
            <div class="cards">
                <div class="card ${isPFIC ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1291.card.is_pfic">Is PFIC?</div>
                    <div class="value">${isPFIC ? esc(t('view.s1291.status.yes')) : esc(t('view.s1291.status.no'))}</div>
                </div>
                <div class="card ${default1291 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1291.card.regime">Active regime</div>
                    <div class="value">${default1291 ? esc(t('view.s1291.regime.default')) : (state.elects_qef ? esc(t('view.s1291.regime.qef')) : state.elects_mtm ? esc(t('view.s1291.regime.mtm')) : esc(t('view.s1291.regime.none')))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1291.card.excess">Excess distribution</div>
                    <div class="value">$${excessDist.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1291.card.alloc">Per-year allocation</div>
                    <div class="value">$${allocPerYear.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1291.card.base_tax">Base tax (37%)</div>
                    <div class="value">$${baseTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1291.card.interest">Interest charge</div>
                    <div class="value">$${interestCharge.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1291.card.total">Total cost</div>
                    <div class="value">$${totalCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${default1291 && state.holding_period_years > 3 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s1291.long_hold_note">
                    Long PFIC holding period: interest charge compounds annually and can exceed the base
                    tax for 10+ year holdings. Consider purging election (deemed sale at § 1291 rates) +
                    QEF / MTM election going forward.
                </p>
            ` : ''}
        </div>
    `;
}
