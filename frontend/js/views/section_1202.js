// IRC § 1202 — Qualified Small Business Stock (QSBS) Gain Exclusion.
// Up to 100% exclusion of gain on sale of QSBS held > 5 years (post-Sept 27, 2010).
// Pre-Feb 18, 2009: 50%; Feb 18 2009 - Sept 27 2010: 75%.
// Cap: greater of $10M or 10× basis per issuer.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    is_qsbs: false,
    acquisition_date: '',
    sale_date: '',
    holding_days: 0,
    original_basis: 0,
    sale_proceeds: 0,
    gain_realized: 0,
    issuer_gross_assets_at_issuance: 0,
    is_c_corporation: false,
    is_active_business: false,
    business_excluded: false,
    excluded_industry: 'none',
    acquired_at_original_issuance: false,
    held_for_5_years: false,
    s1045_rollover_election: false,
    s1045_replacement_qsbs: 0,
    s1045_replacement_period_60d: false,
    exclusion_pct: 100,
    cap_10m: 10_000_000,
    cap_basis_10x_multiple: 0,
    cumulative_exclusion_prior: 0,
    is_pass_through_holder: false,
    s1202_amt_pref: false,
};

export async function renderSection1202(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1202.h1.title">// § 1202 QSBS GAIN EXCLUSION</span></h1>
        <p class="muted small" data-i18n="view.s1202.hint.intro">
            <strong>Up to 100% gain exclusion</strong> on sale of Qualified Small Business Stock (QSBS)
            held > 5 years. <strong>Acquisition windows:</strong> pre-Feb 18, 2009 = 50% exclusion;
            Feb 18 2009-Sept 27 2010 = 75%; post-Sept 27 2010 = 100%. <strong>Cap:</strong> greater of
            $10M OR 10× original basis per issuer per taxpayer. <strong>QSB requirements (§ 1202(d)):</strong>
            (1) domestic C-corp (NOT S-corp, NOT partnership), (2) ≤ $50M aggregate gross assets at
            issuance + immediately after, (3) ≥ 80% (by value) used in active qualified trade or
            business, (4) NOT excluded industry (banking, insurance, financial services, farming,
            mineral extraction, hospitality, certain professional services), (5) acquired at ORIGINAL
            issuance (not secondary). <strong>§ 1045 rollover:</strong> 60-day reinvestment in another
            QSBS defers gain.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1202.h2.inputs">Inputs</h2>
            <form id="s1202-form" class="inline-form">
                <label><span data-i18n="view.s1202.label.qsbs">Is QSBS?</span>
                    <input type="checkbox" name="is_qsbs" ${state.is_qsbs ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1202.label.acq">Acquisition date</span>
                    <input type="date" name="acquisition_date" value="${state.acquisition_date}"></label>
                <label><span data-i18n="view.s1202.label.sale">Sale date</span>
                    <input type="date" name="sale_date" value="${state.sale_date}"></label>
                <label><span data-i18n="view.s1202.label.holding">Holding days</span>
                    <input type="number" step="1" name="holding_days" value="${state.holding_days}"></label>
                <label><span data-i18n="view.s1202.label.basis">Original basis ($)</span>
                    <input type="number" step="10000" name="original_basis" value="${state.original_basis}"></label>
                <label><span data-i18n="view.s1202.label.proceeds">Sale proceeds ($)</span>
                    <input type="number" step="10000" name="sale_proceeds" value="${state.sale_proceeds}"></label>
                <label><span data-i18n="view.s1202.label.gain">Gain realized ($)</span>
                    <input type="number" step="10000" name="gain_realized" value="${state.gain_realized}"></label>
                <label><span data-i18n="view.s1202.label.assets">Issuer gross assets at issuance ($)</span>
                    <input type="number" step="100000" name="issuer_gross_assets_at_issuance" value="${state.issuer_gross_assets_at_issuance}"></label>
                <label><span data-i18n="view.s1202.label.ccorp">C-corporation?</span>
                    <input type="checkbox" name="is_c_corporation" ${state.is_c_corporation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1202.label.active">Active business (80% test)?</span>
                    <input type="checkbox" name="is_active_business" ${state.is_active_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1202.label.excluded">Excluded business?</span>
                    <input type="checkbox" name="business_excluded" ${state.business_excluded ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1202.label.industry">Industry</span>
                    <select name="excluded_industry">
                        <option value="none" ${state.excluded_industry === 'none' ? 'selected' : ''}>None (qualified)</option>
                        <option value="banking" ${state.excluded_industry === 'banking' ? 'selected' : ''}>Banking / insurance / financing / brokerage</option>
                        <option value="services" ${state.excluded_industry === 'services' ? 'selected' : ''}>Health / law / accounting / consulting / brokerage</option>
                        <option value="farming" ${state.excluded_industry === 'farming' ? 'selected' : ''}>Farming</option>
                        <option value="extraction" ${state.excluded_industry === 'extraction' ? 'selected' : ''}>Mineral extraction (§ 613)</option>
                        <option value="hospitality" ${state.excluded_industry === 'hospitality' ? 'selected' : ''}>Hotel / motel / restaurant</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1202.label.original_issue">Original issuance?</span>
                    <input type="checkbox" name="acquired_at_original_issuance" ${state.acquired_at_original_issuance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1202.label.5yr">Held 5+ years?</span>
                    <input type="checkbox" name="held_for_5_years" ${state.held_for_5_years ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1202.label.s1045">§ 1045 rollover election?</span>
                    <input type="checkbox" name="s1045_rollover_election" ${state.s1045_rollover_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1202.label.replacement">Replacement QSBS ($)</span>
                    <input type="number" step="10000" name="s1045_replacement_qsbs" value="${state.s1045_replacement_qsbs}"></label>
                <label><span data-i18n="view.s1202.label.replacement_period">60-day period satisfied?</span>
                    <input type="checkbox" name="s1045_replacement_period_60d" ${state.s1045_replacement_period_60d ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1202.label.pct">Exclusion %</span>
                    <input type="number" step="1" name="exclusion_pct" value="${state.exclusion_pct}"></label>
                <label><span data-i18n="view.s1202.label.cap_10m">$10M cap ($)</span>
                    <input type="number" step="100000" name="cap_10m" value="${state.cap_10m}"></label>
                <label><span data-i18n="view.s1202.label.cap_basis">10× basis cap ($)</span>
                    <input type="number" step="10000" name="cap_basis_10x_multiple" value="${state.cap_basis_10x_multiple}"></label>
                <label><span data-i18n="view.s1202.label.prior_excl">Prior cumulative exclusion ($)</span>
                    <input type="number" step="10000" name="cumulative_exclusion_prior" value="${state.cumulative_exclusion_prior}"></label>
                <label><span data-i18n="view.s1202.label.passthrough">Pass-through holder?</span>
                    <input type="checkbox" name="is_pass_through_holder" ${state.is_pass_through_holder ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1202.label.amt_pref">§ 1202 AMT preference (pre-2010)?</span>
                    <input type="checkbox" name="s1202_amt_pref" ${state.s1202_amt_pref ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s1202.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1202-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1202.h2.windows">Acquisition window → exclusion %</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s1202.tbl.acq_date">Acquisition date</th><th data-i18n="view.s1202.tbl.pct">Exclusion %</th><th data-i18n="view.s1202.tbl.amt">AMT preference?</th><th data-i18n="view.s1202.tbl.s1411">§ 1411 NIIT excl?</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s1202.tbl.window1">Pre-Feb 18, 2009</td><td>50%</td><td>YES (§ 57(a)(7))</td><td>NO</td></tr>
                    <tr><td data-i18n="view.s1202.tbl.window2">Feb 18 2009 - Sept 27 2010</td><td>75%</td><td>YES</td><td>NO</td></tr>
                    <tr><td data-i18n="view.s1202.tbl.window3">Post-Sept 27, 2010</td><td>100%</td><td>NO (PATH Act)</td><td>YES (no NIIT on excluded gain)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1202.h2.requirements">QSB requirements (§ 1202(d), (e))</h2>
            <ol class="muted small">
                <li data-i18n="view.s1202.req.ccorp">Domestic C-corporation (NOT S-corp, NOT partnership, NOT LLC default)</li>
                <li data-i18n="view.s1202.req.gross_assets">≤ $50M aggregate gross assets at issuance + IMMEDIATELY after</li>
                <li data-i18n="view.s1202.req.active_business">≥ 80% (by value) used in active qualified trade or business</li>
                <li data-i18n="view.s1202.req.original_issuance">Acquired at ORIGINAL issuance (NOT secondary purchase)</li>
                <li data-i18n="view.s1202.req.holding_5">Held for > 5 years (60+ months)</li>
                <li data-i18n="view.s1202.req.qualified_business">Qualified business — excludes banking, insurance, services, farming, extraction, hospitality</li>
                <li data-i18n="view.s1202.req.s1202_e_3">§ 1202(e)(3): excluded fields = health, law, engineering, architecture, accounting, actuarial science, performing arts, consulting, athletics, financial services, brokerage, "reputation/skill" principal asset</li>
                <li data-i18n="view.s1202.req.real_estate_excl">Real estate held for ≥ 10% of value disqualifies</li>
                <li data-i18n="view.s1202.req.s1202_e_5">§ 1202(e)(5): no more than 10% of value in stock or securities of unrelated corporations</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1202.h2.cap">$10M / 10× basis cap mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s1202.cap.greater">Cap = GREATER of $10M (lifetime) OR 10× aggregate basis per ISSUER</li>
                <li data-i18n="view.s1202.cap.per_issuer">Cap applied per ISSUER (not per stock holding) per TAXPAYER</li>
                <li data-i18n="view.s1202.cap.cumulative">Cumulative across multiple sale years (Form 8949 reporting)</li>
                <li data-i18n="view.s1202.cap.basis_advantage">10× basis advantage: $500K basis → $5M cap from basis alone (so $10M baseline still controls)</li>
                <li data-i18n="view.s1202.cap.spouse">Spouse: $10M each ($20M MFJ — separately computed)</li>
                <li data-i18n="view.s1202.cap.stacking">Stacking with multiple beneficiaries: trust + family members each get separate cap</li>
                <li data-i18n="view.s1202.cap.gift_split">Lifetime gifts: recipient inherits holding period + acquisition date + basis</li>
                <li data-i18n="view.s1202.cap.deathbed">Death: stepped-up basis terminates QSBS treatment for heirs (§ 1014)</li>
                <li data-i18n="view.s1202.cap.s1411">100% excluded: also exempt from § 1411 3.8% NIIT</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1202.h2.s1045">§ 1045 rollover (taxpayer ≤ corp)</h2>
            <ul class="muted small">
                <li data-i18n="view.s1202.s1045.purpose">Defer gain on QSBS sale by reinvesting in another QSBS</li>
                <li data-i18n="view.s1202.s1045.60d">60-day period from sale to reinvestment</li>
                <li data-i18n="view.s1202.s1045.holding_carryover">Holding period carries over to replacement</li>
                <li data-i18n="view.s1202.s1045.basis_carryover">Basis = cost of replacement reduced by deferred gain</li>
                <li data-i18n="view.s1202.s1045.partial">Partial rollover allowed — defer pro-rata</li>
                <li data-i18n="view.s1202.s1045.preserves_qsbs">Preserves QSBS character + cumulative holding for eventual § 1202 sale</li>
                <li data-i18n="view.s1202.s1045.election">Election made on Schedule D + Form 8949</li>
                <li data-i18n="view.s1202.s1045.individuals">Individuals only — NOT corporations or partnerships at entity level</li>
                <li data-i18n="view.s1202.s1045.no_loss">NO loss recognition allowed under § 1045</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1202.h2.strategy">Stacking + multiplication strategies</h2>
            <ul class="muted small">
                <li data-i18n="view.s1202.strat.gift">Lifetime gift to trust / family: each donee gets separate $10M cap</li>
                <li data-i18n="view.s1202.strat.trust">Multiple irrevocable trusts ("QSBS multipliers"): each trust = separate cap</li>
                <li data-i18n="view.s1202.strat.dynasty">Dynasty trust: pass to grandchildren skipping § 1202 cap exhaustion</li>
                <li data-i18n="view.s1202.strat.qbsg">Qualified small business growing: convert to C-corp BEFORE valuation > $50M</li>
                <li data-i18n="view.s1202.strat.timing">5-year clock starts: ORIGINAL issuance date of QSBS</li>
                <li data-i18n="view.s1202.strat.f_reorg">F reorganization can preserve QSBS — careful planning</li>
                <li data-i18n="view.s1202.strat.s83b">§ 83(b) election: tacks holding period for restricted stock</li>
                <li data-i18n="view.s1202.strat.exchanges_pre">§ 351 stock-for-stock exchange may continue QSBS clock</li>
                <li data-i18n="view.s1202.strat.s1202h">§ 1202(h) gift/inheritance — donee tacks holding period + basis</li>
            </ul>
        </div>
    `;
    document.getElementById('s1202-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_qsbs = !!fd.get('is_qsbs');
        state.acquisition_date = fd.get('acquisition_date') || '';
        state.sale_date = fd.get('sale_date') || '';
        state.holding_days = Number(fd.get('holding_days')) || 0;
        state.original_basis = Number(fd.get('original_basis')) || 0;
        state.sale_proceeds = Number(fd.get('sale_proceeds')) || 0;
        state.gain_realized = Number(fd.get('gain_realized')) || 0;
        state.issuer_gross_assets_at_issuance = Number(fd.get('issuer_gross_assets_at_issuance')) || 0;
        state.is_c_corporation = !!fd.get('is_c_corporation');
        state.is_active_business = !!fd.get('is_active_business');
        state.business_excluded = !!fd.get('business_excluded');
        state.excluded_industry = fd.get('excluded_industry');
        state.acquired_at_original_issuance = !!fd.get('acquired_at_original_issuance');
        state.held_for_5_years = !!fd.get('held_for_5_years');
        state.s1045_rollover_election = !!fd.get('s1045_rollover_election');
        state.s1045_replacement_qsbs = Number(fd.get('s1045_replacement_qsbs')) || 0;
        state.s1045_replacement_period_60d = !!fd.get('s1045_replacement_period_60d');
        state.exclusion_pct = Number(fd.get('exclusion_pct')) || 0;
        state.cap_10m = Number(fd.get('cap_10m')) || 0;
        state.cap_basis_10x_multiple = Number(fd.get('cap_basis_10x_multiple')) || 0;
        state.cumulative_exclusion_prior = Number(fd.get('cumulative_exclusion_prior')) || 0;
        state.is_pass_through_holder = !!fd.get('is_pass_through_holder');
        state.s1202_amt_pref = !!fd.get('s1202_amt_pref');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1202-output');
    if (!el) return;
    const cap = Math.max(state.cap_10m, state.original_basis * 10) - state.cumulative_exclusion_prior;
    const eligible_gain = Math.min(state.gain_realized, cap);
    const excluded = eligible_gain * (state.exclusion_pct / 100);
    const taxable = state.gain_realized - excluded;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1202.h2.result">§ 1202 exclusion result</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s1202.card.gain">Gain realized</div><div class="value">$${state.gain_realized.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s1202.card.cap">Cap (max $10M / 10× basis)</div><div class="value">$${cap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card"><div class="label" data-i18n="view.s1202.card.eligible">Eligible for exclusion</div><div class="value">$${eligible_gain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s1202.card.excluded">Excluded (${state.exclusion_pct}%)</div><div class="value">$${excluded.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s1202.card.taxable">Taxable gain</div><div class="value">$${taxable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
            </div>
        </div>
    `;
}
