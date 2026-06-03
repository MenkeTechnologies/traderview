// IRC § 956 — Investment of Earnings in United States Property by CFC.
// US shareholder of CFC must include in income amount equal to CFC's increase in "US property" holdings.
// "US property" includes: US tangible property, US obligation, US stock, certain right to acquire US property.
// Post-TCJA: § 956 still applies but coordinates with § 245A foreign DRD; reg § 1.956-1(a)(2) provides hybrid relief.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    us_shareholder_pct: 0,
    cfc_country: 'Ireland',
    cfc_e_p: 0,
    us_property_increase: 0,
    s956_a_inclusion_amount: 0,
    s956_a_2_b_average_quarterly_balance: 0,
    s956_b_1_us_real_property: 0,
    s956_b_1_us_obligations: 0,
    s956_b_1_us_corporation_stock: 0,
    s956_b_1_right_to_acquire_us_property: 0,
    s956_c_excluded_inventory: false,
    s956_c_us_obligation_treasury: false,
    s956_c_us_obligation_short_term: false,
    cfc_short_term_loan_30day: 0,
    is_qualified_investment_excluded: false,
    is_consolidated_subsidiary: false,
    s956_d_pledge_or_guarantee: 0,
    s956_e_anti_avoidance: false,
    s245a_eligible_dividend: 0,
    s245a_drd_amount: 0,
    s956_proposed_regs_2018: false,
    s956_final_regs_2019: false,
    s956_e_partnership_interest: 0,
    s956_e_disregarded_entity: false,
    is_pti_distribution: false,
    pti_previously_taxed_income: 0,
    s951a_gilti_inclusion: 0,
    s962_election_made: false,
    s960_foreign_tax_credit: 0,
    cfc_qbai: 0,
    cfc_active_business_income: 0,
    is_treaty_country: false,
    treaty_dividend_rate: 0,
};

export async function renderSection956(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s956.h1.title">// § 956 CFC INVESTMENT IN US PROPERTY</span></h1>
        <p class="muted small" data-i18n="view.s956.hint.intro">
            <strong>§ 956</strong> — US shareholder of CFC must include in current income AMOUNT
            equal to lesser of: (1) CFC's INCREASE in US property holdings + (2) CFC's earnings &
            profits (E&P). <strong>"US property":</strong> tangible property in US, US obligations
            (excl Treasuries + short-term), stock of US corporation, rights to acquire US property,
            certain partnership interests + DREs. <strong>Purpose:</strong> deters CFCs from making
            functional dividends to US shareholders by investing CFC E&P in US assets while avoiding
            taxable dividend. <strong>Post-TCJA + § 245A:</strong> proposed regs (2018) + final regs
            (2019) provide HYBRID RELIEF — reduces § 956 inclusion to the extent dividend would have
            qualified for § 245A 100% DRD. <strong>§ 956 inclusion</strong> retains ordinary character
            (not qualified dividend) + does NOT count toward GILTI. <strong>§ 956(d) pledges +
            guarantees</strong> treated as US property investment.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s956.h2.inputs">Inputs</h2>
            <form id="s956-form" class="inline-form">
                <label><span data-i18n="view.s956.label.us_shareholder">US shareholder %</span>
                    <input type="number" step="0.1" name="us_shareholder_pct" value="${state.us_shareholder_pct}"></label>
                <label><span data-i18n="view.s956.label.country">CFC country</span>
                    <input type="text" name="cfc_country" value="${esc(state.cfc_country)}"></label>
                <label><span data-i18n="view.s956.label.ep">CFC E&P ($)</span>
                    <input type="number" step="100000" name="cfc_e_p" value="${state.cfc_e_p}"></label>
                <label><span data-i18n="view.s956.label.property_increase">US property increase ($)</span>
                    <input type="number" step="100000" name="us_property_increase" value="${state.us_property_increase}"></label>
                <label><span data-i18n="view.s956.label.inclusion">§ 956(a) inclusion ($)</span>
                    <input type="number" step="100000" name="s956_a_inclusion_amount" value="${state.s956_a_inclusion_amount}"></label>
                <label><span data-i18n="view.s956.label.avg_quarterly">Avg quarterly ($)</span>
                    <input type="number" step="100000" name="s956_a_2_b_average_quarterly_balance" value="${state.s956_a_2_b_average_quarterly_balance}"></label>
                <label><span data-i18n="view.s956.label.real_property">US real property ($)</span>
                    <input type="number" step="100000" name="s956_b_1_us_real_property" value="${state.s956_b_1_us_real_property}"></label>
                <label><span data-i18n="view.s956.label.obligations">US obligations ($)</span>
                    <input type="number" step="100000" name="s956_b_1_us_obligations" value="${state.s956_b_1_us_obligations}"></label>
                <label><span data-i18n="view.s956.label.stock">US corp stock ($)</span>
                    <input type="number" step="100000" name="s956_b_1_us_corporation_stock" value="${state.s956_b_1_us_corporation_stock}"></label>
                <label><span data-i18n="view.s956.label.rights">Rights to acquire US ($)</span>
                    <input type="number" step="100000" name="s956_b_1_right_to_acquire_us_property" value="${state.s956_b_1_right_to_acquire_us_property}"></label>
                <label><span data-i18n="view.s956.label.inv_excluded">Inventory excluded?</span>
                    <input type="checkbox" name="s956_c_excluded_inventory" ${state.s956_c_excluded_inventory ? 'checked' : ''}></label>
                <label><span data-i18n="view.s956.label.treasury_excluded">Treasury excluded?</span>
                    <input type="checkbox" name="s956_c_us_obligation_treasury" ${state.s956_c_us_obligation_treasury ? 'checked' : ''}></label>
                <label><span data-i18n="view.s956.label.short_term">Short-term excluded?</span>
                    <input type="checkbox" name="s956_c_us_obligation_short_term" ${state.s956_c_us_obligation_short_term ? 'checked' : ''}></label>
                <label><span data-i18n="view.s956.label.short_loan">30-day loan ($)</span>
                    <input type="number" step="100000" name="cfc_short_term_loan_30day" value="${state.cfc_short_term_loan_30day}"></label>
                <label><span data-i18n="view.s956.label.qualified_inv">Qualified inv excluded?</span>
                    <input type="checkbox" name="is_qualified_investment_excluded" ${state.is_qualified_investment_excluded ? 'checked' : ''}></label>
                <label><span data-i18n="view.s956.label.consolidated">Consolidated sub?</span>
                    <input type="checkbox" name="is_consolidated_subsidiary" ${state.is_consolidated_subsidiary ? 'checked' : ''}></label>
                <label><span data-i18n="view.s956.label.pledge">Pledge / guarantee ($)</span>
                    <input type="number" step="100000" name="s956_d_pledge_or_guarantee" value="${state.s956_d_pledge_or_guarantee}"></label>
                <label><span data-i18n="view.s956.label.anti_avoid">§ 956(e) anti-avoid?</span>
                    <input type="checkbox" name="s956_e_anti_avoidance" ${state.s956_e_anti_avoidance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s956.label.s245a_eligible">§ 245A eligible div ($)</span>
                    <input type="number" step="100000" name="s245a_eligible_dividend" value="${state.s245a_eligible_dividend}"></label>
                <label><span data-i18n="view.s956.label.s245a_drd">§ 245A DRD ($)</span>
                    <input type="number" step="100000" name="s245a_drd_amount" value="${state.s245a_drd_amount}"></label>
                <label><span data-i18n="view.s956.label.proposed">2018 proposed regs?</span>
                    <input type="checkbox" name="s956_proposed_regs_2018" ${state.s956_proposed_regs_2018 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s956.label.final">2019 final regs?</span>
                    <input type="checkbox" name="s956_final_regs_2019" ${state.s956_final_regs_2019 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s956.label.ps_interest">Partnership interest ($)</span>
                    <input type="number" step="100000" name="s956_e_partnership_interest" value="${state.s956_e_partnership_interest}"></label>
                <label><span data-i18n="view.s956.label.dre">DRE?</span>
                    <input type="checkbox" name="s956_e_disregarded_entity" ${state.s956_e_disregarded_entity ? 'checked' : ''}></label>
                <label><span data-i18n="view.s956.label.pti">PTI distribution?</span>
                    <input type="checkbox" name="is_pti_distribution" ${state.is_pti_distribution ? 'checked' : ''}></label>
                <label><span data-i18n="view.s956.label.pti_amt">PTI ($)</span>
                    <input type="number" step="100000" name="pti_previously_taxed_income" value="${state.pti_previously_taxed_income}"></label>
                <label><span data-i18n="view.s956.label.gilti">§ 951A GILTI ($)</span>
                    <input type="number" step="100000" name="s951a_gilti_inclusion" value="${state.s951a_gilti_inclusion}"></label>
                <label><span data-i18n="view.s956.label.s962">§ 962 election?</span>
                    <input type="checkbox" name="s962_election_made" ${state.s962_election_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s956.label.s960">§ 960 FTC ($)</span>
                    <input type="number" step="100000" name="s960_foreign_tax_credit" value="${state.s960_foreign_tax_credit}"></label>
                <label><span data-i18n="view.s956.label.qbai">QBAI ($)</span>
                    <input type="number" step="100000" name="cfc_qbai" value="${state.cfc_qbai}"></label>
                <label><span data-i18n="view.s956.label.active">Active biz income ($)</span>
                    <input type="number" step="100000" name="cfc_active_business_income" value="${state.cfc_active_business_income}"></label>
                <label><span data-i18n="view.s956.label.treaty">Treaty country?</span>
                    <input type="checkbox" name="is_treaty_country" ${state.is_treaty_country ? 'checked' : ''}></label>
                <label><span data-i18n="view.s956.label.treaty_rate">Treaty div rate %</span>
                    <input type="number" step="0.1" name="treaty_dividend_rate" value="${state.treaty_dividend_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s956.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s956-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s956.h2.us_property">"US property" — § 956(c)(1)</h2>
            <ul class="muted small">
                <li data-i18n="view.s956.usp.tangible">Tangible property situated in US</li>
                <li data-i18n="view.s956.usp.obligations">Obligations of US person (loans, debt)</li>
                <li data-i18n="view.s956.usp.stock">Stock of US domestic corporation</li>
                <li data-i18n="view.s956.usp.rights">Right to acquire US property (option, contract)</li>
                <li data-i18n="view.s956.usp.partnership">Interest in domestic partnership (with US partners)</li>
                <li data-i18n="view.s956.usp.dre">Disregarded entity (US owner)</li>
                <li data-i18n="view.s956.usp.exclusions">EXCLUSIONS § 956(c)(2): inventory, obligations &lt; 30 days, Treasury securities, certain insurance, US bank deposits ($1.4M cap)</li>
                <li data-i18n="view.s956.usp.s245a_dividend_eligible">Post-TCJA: § 956 inclusion reduced if § 245A would otherwise apply (Reg § 1.956-1(a)(2))</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s956.h2.computation">§ 956 amount computation</h2>
            <ol class="muted small">
                <li data-i18n="view.s956.comp.average">Calculate AVERAGE quarterly balance of US property</li>
                <li data-i18n="view.s956.comp.increase">Increase over prior year = US property "invested"</li>
                <li data-i18n="view.s956.comp.lesser">§ 956(a) inclusion = LESSER of: (1) increase OR (2) E&P</li>
                <li data-i18n="view.s956.comp.character">Inclusion treated as ORDINARY (NOT qualified dividend)</li>
                <li data-i18n="view.s956.comp.no_drd">NO § 245A DRD on § 956 inclusion (subject to hybrid mismatch rules)</li>
                <li data-i18n="view.s956.comp.s960">§ 960 indirect foreign tax credit available</li>
                <li data-i18n="view.s956.comp.distinct_gilti">DISTINCT from GILTI inclusion under § 951A</li>
                <li data-i18n="view.s956.comp.reduces_ep">Subsequent dividend tax-free via PTI ordering — pre-2018 timing</li>
                <li data-i18n="view.s956.comp.hybrid_relief_post_TCJA">§ 245A Reg § 1.956-1(a)(2) hybrid relief reduces § 956 inclusion</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s956.h2.exclusions">§ 956(c)(2) exclusions</h2>
            <ul class="muted small">
                <li data-i18n="view.s956.exc.inventory">Property used in active US trade or business (limited)</li>
                <li data-i18n="view.s956.exc.short_term">Obligations &lt; 30 days held</li>
                <li data-i18n="view.s956.exc.treasury">US Treasury securities + agency securities</li>
                <li data-i18n="view.s956.exc.bank_deposits">US bank deposits up to $1.4M aggregate per CFC</li>
                <li data-i18n="view.s956.exc.insurance">Insurance contracts — certain CFCs</li>
                <li data-i18n="view.s956.exc.s956_c_2_g">§ 956(c)(2)(G) — assets used for foreign branch operations</li>
                <li data-i18n="view.s956.exc.s956_c_2_l">§ 956(c)(2)(L) — derivatives + similar</li>
                <li data-i18n="view.s956.exc.s956_c_3">§ 956(c)(3) — special rules for shipping, aircraft, container business</li>
                <li data-i18n="view.s956.exc.s956_c_2_J">§ 956(c)(2)(J) — securities of unrelated person</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s956.h2.pledge_guarantee">§ 956(d) pledges + guarantees</h2>
            <ul class="muted small">
                <li data-i18n="view.s956.pg.included">CFC guarantee of US obligation = investment in US property</li>
                <li data-i18n="view.s956.pg.s956_d_1">§ 956(d)(1) — pledge of stock of US affiliate</li>
                <li data-i18n="view.s956.pg.s956_d_2">§ 956(d)(2) — guarantee of debt incurred for US benefit</li>
                <li data-i18n="view.s956.pg.economic_substance">No requirement that guarantee be CALLED — mere existence triggers</li>
                <li data-i18n="view.s956.pg.indirect_pledge">Indirect pledges (assignment of CFC receivables) — covered</li>
                <li data-i18n="view.s956.pg.common_structure">Common: US parent borrows + CFC subsidiary guarantees → § 956 inclusion</li>
                <li data-i18n="view.s956.pg.s245a_relief">Post-TCJA § 245A reduces 65/35 split for hybrid CFCs</li>
                <li data-i18n="view.s956.pg.fmv_basis">FMV of pledged stock / guaranteed obligation determines amount</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s956.h2.s245a_relief">§ 245A hybrid relief (post-TCJA)</h2>
            <ul class="muted small">
                <li data-i18n="view.s956.s245a.purpose">Eliminates § 956 to extent dividend would qualify for § 245A 100% DRD</li>
                <li data-i18n="view.s956.s245a.proposed_2018">Proposed regs Oct 2018 (REG-114540-18)</li>
                <li data-i18n="view.s956.s245a.final_2019">Final regs May 2019 — § 1.956-1(a)(2)</li>
                <li data-i18n="view.s956.s245a.computation">"Hypothetical dividend" comparison — would § 245A apply?</li>
                <li data-i18n="view.s956.s245a.hybrid_dividend">§ 245A(e) excludes hybrid dividends from DRD</li>
                <li data-i18n="view.s956.s245a.partnerships_not_covered">Partnerships NOT eligible for § 245A — full § 956 applies</li>
                <li data-i18n="view.s956.s245a.us_individuals">US individual shareholders: § 245A unavailable — full § 956</li>
                <li data-i18n="view.s956.s245a.s962_election">§ 962 election helps individuals access § 250 + § 245A protections</li>
                <li data-i18n="view.s956.s245a.gilti_treatment">GILTI subpart F inclusions: separate analysis</li>
            </ul>
        </div>
    `;
    document.getElementById('s956-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.us_shareholder_pct = Number(fd.get('us_shareholder_pct')) || 0;
        state.cfc_country = fd.get('cfc_country') || '';
        state.cfc_e_p = Number(fd.get('cfc_e_p')) || 0;
        state.us_property_increase = Number(fd.get('us_property_increase')) || 0;
        state.s956_a_inclusion_amount = Number(fd.get('s956_a_inclusion_amount')) || 0;
        state.s956_a_2_b_average_quarterly_balance = Number(fd.get('s956_a_2_b_average_quarterly_balance')) || 0;
        state.s956_b_1_us_real_property = Number(fd.get('s956_b_1_us_real_property')) || 0;
        state.s956_b_1_us_obligations = Number(fd.get('s956_b_1_us_obligations')) || 0;
        state.s956_b_1_us_corporation_stock = Number(fd.get('s956_b_1_us_corporation_stock')) || 0;
        state.s956_b_1_right_to_acquire_us_property = Number(fd.get('s956_b_1_right_to_acquire_us_property')) || 0;
        state.s956_c_excluded_inventory = !!fd.get('s956_c_excluded_inventory');
        state.s956_c_us_obligation_treasury = !!fd.get('s956_c_us_obligation_treasury');
        state.s956_c_us_obligation_short_term = !!fd.get('s956_c_us_obligation_short_term');
        state.cfc_short_term_loan_30day = Number(fd.get('cfc_short_term_loan_30day')) || 0;
        state.is_qualified_investment_excluded = !!fd.get('is_qualified_investment_excluded');
        state.is_consolidated_subsidiary = !!fd.get('is_consolidated_subsidiary');
        state.s956_d_pledge_or_guarantee = Number(fd.get('s956_d_pledge_or_guarantee')) || 0;
        state.s956_e_anti_avoidance = !!fd.get('s956_e_anti_avoidance');
        state.s245a_eligible_dividend = Number(fd.get('s245a_eligible_dividend')) || 0;
        state.s245a_drd_amount = Number(fd.get('s245a_drd_amount')) || 0;
        state.s956_proposed_regs_2018 = !!fd.get('s956_proposed_regs_2018');
        state.s956_final_regs_2019 = !!fd.get('s956_final_regs_2019');
        state.s956_e_partnership_interest = Number(fd.get('s956_e_partnership_interest')) || 0;
        state.s956_e_disregarded_entity = !!fd.get('s956_e_disregarded_entity');
        state.is_pti_distribution = !!fd.get('is_pti_distribution');
        state.pti_previously_taxed_income = Number(fd.get('pti_previously_taxed_income')) || 0;
        state.s951a_gilti_inclusion = Number(fd.get('s951a_gilti_inclusion')) || 0;
        state.s962_election_made = !!fd.get('s962_election_made');
        state.s960_foreign_tax_credit = Number(fd.get('s960_foreign_tax_credit')) || 0;
        state.cfc_qbai = Number(fd.get('cfc_qbai')) || 0;
        state.cfc_active_business_income = Number(fd.get('cfc_active_business_income')) || 0;
        state.is_treaty_country = !!fd.get('is_treaty_country');
        state.treaty_dividend_rate = Number(fd.get('treaty_dividend_rate')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s956-output');
    if (!el) return;
    const inclusion = Math.min(state.us_property_increase, state.cfc_e_p);
    const after_s245a = state.s956_final_regs_2019 ? Math.max(0, inclusion - state.s245a_drd_amount) : inclusion;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s956.h2.result">§ 956 inclusion analysis</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s956.card.increase">US property increase</div><div class="value">$${state.us_property_increase.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s956.card.ep">CFC E&P</div><div class="value">$${state.cfc_e_p.toLocaleString()}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s956.card.inclusion">Lesser (raw)</div><div class="value">$${inclusion.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s956.card.s245a">After § 245A relief</div><div class="value">$${after_s245a.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s956.card.s960">§ 960 FTC</div><div class="value">$${state.s960_foreign_tax_credit.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
