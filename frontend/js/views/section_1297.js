// IRC § 1297 — Passive Foreign Investment Company (PFIC) Definition.
// PFIC if EITHER: (1) 75%+ income passive OR (2) 50%+ assets produce passive income (or held for such).
// "Once a PFIC, always a PFIC" in hands of shareholder (until purged).
// Look-through rule for 25%+ owned subsidiaries.
// Startup exception + CFC overlap rule (CFC US shareholders escape PFIC regime).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    gross_income_total: 0,
    passive_income_amount: 0,
    total_assets_fmv: 0,
    passive_assets_fmv: 0,
    look_through_25pct_sub: false,
    sub_passive_income_share: 0,
    sub_passive_assets_share: 0,
    is_startup_year: false,
    is_cfc_overlap: false,
    de_minimis_5pct: false,
    is_foreign_holding: false,
    holding_period_years: 0,
    purging_election_made: false,
    qef_election_active: false,
    mtm_election_active: false,
};

export async function renderSection1297(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1297.h1.title">// § 1297 PFIC DEFINITION</span></h1>
        <p class="muted small" data-i18n="view.s1297.hint.intro">
            <strong>PFIC if EITHER:</strong> (1) <strong>75%+ income passive</strong> OR (2) <strong>50%+ assets</strong>
            produce passive income (or held for such). <strong>"Once a PFIC, always a PFIC"</strong> in
            shareholder's hands until purged. <strong>Look-through rule:</strong> ≥ 25%-owned subsidiaries
            taxed on look-through basis. <strong>Startup exception</strong> § 1298(b)(2): first year of
            new corp may be exempt. <strong>CFC overlap</strong>: CFC US shareholders escape PFIC regime
            under § 1298(b)(8). <strong>Form 8621</strong> annually for each PFIC.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1297.h2.inputs">Inputs</h2>
            <form id="s1297-form" class="inline-form">
                <label><span data-i18n="view.s1297.label.gross">Gross income total ($)</span>
                    <input type="number" step="0.01" name="gross_income_total" value="${state.gross_income_total}"></label>
                <label><span data-i18n="view.s1297.label.passive_inc">Passive income amount ($)</span>
                    <input type="number" step="0.01" name="passive_income_amount" value="${state.passive_income_amount}"></label>
                <label><span data-i18n="view.s1297.label.assets">Total assets FMV ($)</span>
                    <input type="number" step="0.01" name="total_assets_fmv" value="${state.total_assets_fmv}"></label>
                <label><span data-i18n="view.s1297.label.passive_assets">Passive assets FMV ($)</span>
                    <input type="number" step="0.01" name="passive_assets_fmv" value="${state.passive_assets_fmv}"></label>
                <label><span data-i18n="view.s1297.label.lookthrough">25%+ sub look-through?</span>
                    <input type="checkbox" name="look_through_25pct_sub" ${state.look_through_25pct_sub ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1297.label.sub_passive_inc">Sub passive income share ($)</span>
                    <input type="number" step="0.01" name="sub_passive_income_share" value="${state.sub_passive_income_share}"></label>
                <label><span data-i18n="view.s1297.label.sub_passive_assets">Sub passive assets share ($)</span>
                    <input type="number" step="0.01" name="sub_passive_assets_share" value="${state.sub_passive_assets_share}"></label>
                <label><span data-i18n="view.s1297.label.startup">Startup year (§ 1298(b)(2))?</span>
                    <input type="checkbox" name="is_startup_year" ${state.is_startup_year ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1297.label.cfc">CFC overlap (§ 1298(b)(8))?</span>
                    <input type="checkbox" name="is_cfc_overlap" ${state.is_cfc_overlap ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1297.label.de_minimis">5% de minimis income?</span>
                    <input type="checkbox" name="de_minimis_5pct" ${state.de_minimis_5pct ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1297.label.foreign_holding">Foreign holding co structure?</span>
                    <input type="checkbox" name="is_foreign_holding" ${state.is_foreign_holding ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1297.label.holding">Holding period years</span>
                    <input type="number" step="0.5" name="holding_period_years" value="${state.holding_period_years}"></label>
                <label><span data-i18n="view.s1297.label.purging">Purging election made?</span>
                    <input type="checkbox" name="purging_election_made" ${state.purging_election_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1297.label.qef">QEF election active?</span>
                    <input type="checkbox" name="qef_election_active" ${state.qef_election_active ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1297.label.mtm">MTM election active?</span>
                    <input type="checkbox" name="mtm_election_active" ${state.mtm_election_active ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s1297.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1297-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1297.h2.tests">Two-prong PFIC test (§ 1297(a))</h2>
            <ul class="muted small">
                <li data-i18n="view.s1297.tests.income">INCOME test: 75%+ of gross income is "passive" (§ 1297(b))</li>
                <li data-i18n="view.s1297.tests.assets">ASSET test: 50%+ of FMV avg assets produce passive income / held for such</li>
                <li data-i18n="view.s1297.tests.either">EITHER triggers PFIC — not both required</li>
                <li data-i18n="view.s1297.tests.passive_def">Passive: interest, dividends, royalties, rents, net gains on personal property</li>
                <li data-i18n="view.s1297.tests.banking">Active banking exception: ≥ 4 financial professionals, regulatory oversight</li>
                <li data-i18n="view.s1297.tests.insurance">Active insurance: ≥ 25% applicable insurance liabilities</li>
                <li data-i18n="view.s1297.tests.real_estate">Rental income: passive UNLESS active real estate trade or business</li>
                <li data-i18n="view.s1297.tests.commodities">Commodities trading: special active dealer exception</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1297.h2.look_through">Look-through rule (§ 1297(c))</h2>
            <ul class="muted small">
                <li data-i18n="view.s1297.lt.threshold">≥ 25% owned subsidiaries treated as look-through (income + asset basis)</li>
                <li data-i18n="view.s1297.lt.allocation">Pro-rata share of sub's income + assets allocated to parent</li>
                <li data-i18n="view.s1297.lt.measurement">Asset measurement: FMV at end of each quarter, averaged</li>
                <li data-i18n="view.s1297.lt.intermediate">Multi-tier: look-through cascades through chain of 25%+ owned subs</li>
                <li data-i18n="view.s1297.lt.passive_assets_active">Foreign sub's active assets cleanse parent's passive characterization</li>
                <li data-i18n="view.s1297.lt.dividend_excl">Dividends from sub: NOT counted as passive income (avoid double-counting)</li>
                <li data-i18n="view.s1297.lt.intercompany">Intercompany debt: special rules to prevent abuse</li>
                <li data-i18n="view.s1297.lt.s1297c_2">§ 1297(c)(2): partnership look-through similar</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1297.h2.exceptions">Key exceptions</h2>
            <ul class="muted small">
                <li data-i18n="view.s1297.exc.startup">§ 1298(b)(2) STARTUP: first year of new active business may escape PFIC if not PFIC in first 3 yrs</li>
                <li data-i18n="view.s1297.exc.cfc_overlap">§ 1298(b)(8) CFC overlap: CFC US 10% shareholder NOT PFIC for that shareholder</li>
                <li data-i18n="view.s1297.exc.banking">§ 1297(b)(2)(A) active banking: ≥ 4 financial professionals + customary loans + deposits</li>
                <li data-i18n="view.s1297.exc.insurance">§ 1297(b)(2)(B) qualifying insurance: applicable insurance liabilities ≥ 25%</li>
                <li data-i18n="view.s1297.exc.related_party">Related-party rent / royalty: NOT passive (look-through to active business)</li>
                <li data-i18n="view.s1297.exc.foreign_personal">Foreign personal services: NOT passive if performed by service-providers</li>
                <li data-i18n="view.s1297.exc.commodities">Active commodities dealer: ≥ 75% income from regular trade dealer activity</li>
                <li data-i18n="view.s1297.exc.5pct_passive">5% de minimis: ≤ 5% income test exception in cleansing years</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1297.h2.consequences">PFIC regime consequences</h2>
            <ul class="muted small">
                <li data-i18n="view.s1297.cons.default">§ 1291 default: top ordinary rate + interest charge on excess distributions / sale gain</li>
                <li data-i18n="view.s1297.cons.qef">§ 1295 QEF election: annual current inclusion + preserve character</li>
                <li data-i18n="view.s1297.cons.mtm">§ 1296 MTM: mark-to-market on tradable PFIC stock</li>
                <li data-i18n="view.s1297.cons.taint">"Once PFIC, always PFIC" until purged via deemed sale</li>
                <li data-i18n="view.s1297.cons.form_8621">Annual Form 8621 reporting required (each PFIC)</li>
                <li data-i18n="view.s1297.cons.attribution">Pass-through entities: PFIC status flows to ultimate US owner</li>
                <li data-i18n="view.s1297.cons.estate">Estate planning: gift / inheritance triggers § 1291 unless purged</li>
                <li data-i18n="view.s1297.cons.cross_border_planning">Cross-border M&A: pre-deal PFIC analysis critical</li>
            </ul>
        </div>
    `;
    document.getElementById('s1297-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.gross_income_total = Number(fd.get('gross_income_total')) || 0;
        state.passive_income_amount = Number(fd.get('passive_income_amount')) || 0;
        state.total_assets_fmv = Number(fd.get('total_assets_fmv')) || 0;
        state.passive_assets_fmv = Number(fd.get('passive_assets_fmv')) || 0;
        state.look_through_25pct_sub = !!fd.get('look_through_25pct_sub');
        state.sub_passive_income_share = Number(fd.get('sub_passive_income_share')) || 0;
        state.sub_passive_assets_share = Number(fd.get('sub_passive_assets_share')) || 0;
        state.is_startup_year = !!fd.get('is_startup_year');
        state.is_cfc_overlap = !!fd.get('is_cfc_overlap');
        state.de_minimis_5pct = !!fd.get('de_minimis_5pct');
        state.is_foreign_holding = !!fd.get('is_foreign_holding');
        state.holding_period_years = Number(fd.get('holding_period_years')) || 0;
        state.purging_election_made = !!fd.get('purging_election_made');
        state.qef_election_active = !!fd.get('qef_election_active');
        state.mtm_election_active = !!fd.get('mtm_election_active');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1297-output');
    if (!el) return;
    const totalPassiveInc = state.passive_income_amount + (state.look_through_25pct_sub ? state.sub_passive_income_share : 0);
    const totalPassiveAssets = state.passive_assets_fmv + (state.look_through_25pct_sub ? state.sub_passive_assets_share : 0);
    const incomePct = state.gross_income_total > 0 ? (totalPassiveInc / state.gross_income_total * 100) : 0;
    const assetsPct = state.total_assets_fmv > 0 ? (totalPassiveAssets / state.total_assets_fmv * 100) : 0;
    const incomeTest = incomePct >= 75;
    const assetsTest = assetsPct >= 50;
    const isPFIC = (incomeTest || assetsTest) && !state.is_startup_year && !state.is_cfc_overlap;
    const regime = isPFIC ? (state.qef_election_active ? 'qef' : state.mtm_election_active ? 'mtm' : 'default_1291') : 'not_pfic';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1297.h2.result">§ 1297 PFIC determination</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1297.card.income_pct">Passive income %</div>
                    <div class="value">${incomePct.toFixed(2)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1297.card.assets_pct">Passive assets %</div>
                    <div class="value">${assetsPct.toFixed(2)}%</div>
                </div>
                <div class="card ${incomeTest ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1297.card.income_test">Income test (≥ 75%)</div>
                    <div class="value">${incomeTest ? esc(t('view.s1297.status.yes')) : esc(t('view.s1297.status.no'))}</div>
                </div>
                <div class="card ${assetsTest ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1297.card.assets_test">Asset test (≥ 50%)</div>
                    <div class="value">${assetsTest ? esc(t('view.s1297.status.yes')) : esc(t('view.s1297.status.no'))}</div>
                </div>
                <div class="card ${isPFIC ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1297.card.is_pfic">Is PFIC?</div>
                    <div class="value">${isPFIC ? esc(t('view.s1297.status.yes')) : esc(t('view.s1297.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1297.card.regime">Tax regime</div>
                    <div class="value">${esc(t('view.s1297.regime.' + regime))}</div>
                </div>
            </div>
            ${state.is_cfc_overlap ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s1297.cfc_note">
                    § 1298(b)(8) CFC OVERLAP exception: this US 10% shareholder of a CFC is NOT subject to
                    PFIC regime. Subpart F + GILTI apply instead. Critical when foreign sub has dual character —
                    must verify CFC status (50%+ US ownership). Form 5471 filing replaces / supplements
                    Form 8621 PFIC reporting.
                </p>
            ` : ''}
        </div>
    `;
}
