// IRC § 511 — Unrelated Business Income Tax (UBIT) on Tax-Exempt Organizations.
// 501(c) orgs taxed at regular corp rates (21%) on net Unrelated Business Taxable Income (UBTI).
// $1K specific deduction. Form 990-T required if UBTI ≥ $1K.
// UBTI = trade or business + regularly carried on + NOT substantially related to exempt purpose.
// § 512(a)(6): "siloing" — net losses ONLY against income within same trade/business (TCJA 2017).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    gross_unrelated_income: 0,
    directly_related_expenses: 0,
    indirect_allocations: 0,
    other_deductions: 0,
    nol_carryforward: 0,
    specific_deduction: 1_000,
    is_university: false,
    is_hospital: false,
    activity_type: 'sales_unrelated',
    is_substantially_related: false,
    is_regularly_carried_on: true,
    volunteer_labor_exception: false,
    is_convenience_for_members: false,
    is_donated_inventory: false,
    is_bingo_or_gaming: false,
    rental_real_property: false,
    debt_financed_property: false,
    siloed_other_businesses_loss: 0,
};

export async function renderSection511(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s511.h1.title">// § 511 UBIT</span></h1>
        <p class="muted small" data-i18n="view.s511.hint.intro">
            501(c) orgs taxed at <strong>21% corp rate</strong> on net <strong>UBTI</strong>
            (Unrelated Business Taxable Income). <strong>$1,000 specific deduction</strong>. <strong>Form
            990-T</strong> required if UBTI ≥ $1,000. <strong>UBTI =</strong> (1) trade or business + (2)
            regularly carried on + (3) NOT substantially related to exempt purpose. <strong>§ 512(a)(6)
            "siloing"</strong> (TCJA 2017): net losses ONLY against income WITHIN SAME trade/business; cannot
            offset across separate UBTI activities. <strong>Exceptions:</strong> volunteer labor, convenience,
            donated, bingo, real prop rent.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s511.h2.inputs">Inputs</h2>
            <form id="s511-form" class="inline-form">
                <label><span data-i18n="view.s511.label.gross">Gross unrelated income ($)</span>
                    <input type="number" step="0.01" name="gross_unrelated_income" value="${state.gross_unrelated_income}"></label>
                <label><span data-i18n="view.s511.label.direct">Directly related expenses ($)</span>
                    <input type="number" step="0.01" name="directly_related_expenses" value="${state.directly_related_expenses}"></label>
                <label><span data-i18n="view.s511.label.indirect">Indirect allocations ($)</span>
                    <input type="number" step="0.01" name="indirect_allocations" value="${state.indirect_allocations}"></label>
                <label><span data-i18n="view.s511.label.other">Other deductions ($)</span>
                    <input type="number" step="0.01" name="other_deductions" value="${state.other_deductions}"></label>
                <label><span data-i18n="view.s511.label.nol">NOL carryforward ($)</span>
                    <input type="number" step="0.01" name="nol_carryforward" value="${state.nol_carryforward}"></label>
                <label><span data-i18n="view.s511.label.specific">Specific deduction ($)</span>
                    <input type="number" step="0.01" name="specific_deduction" value="${state.specific_deduction}"></label>
                <label><span data-i18n="view.s511.label.university">University?</span>
                    <input type="checkbox" name="is_university" ${state.is_university ? 'checked' : ''}></label>
                <label><span data-i18n="view.s511.label.hospital">Hospital?</span>
                    <input type="checkbox" name="is_hospital" ${state.is_hospital ? 'checked' : ''}></label>
                <label><span data-i18n="view.s511.label.activity">Activity type</span>
                    <select name="activity_type">
                        <option value="sales_unrelated" ${state.activity_type === 'sales_unrelated' ? 'selected' : ''}>Sales / merchandise</option>
                        <option value="services" ${state.activity_type === 'services' ? 'selected' : ''}>Services for fee</option>
                        <option value="advertising" ${state.activity_type === 'advertising' ? 'selected' : ''}>Advertising / sponsorships</option>
                        <option value="rental" ${state.activity_type === 'rental' ? 'selected' : ''}>Rental income</option>
                        <option value="research" ${state.activity_type === 'research' ? 'selected' : ''}>Research grants</option>
                        <option value="royalties" ${state.activity_type === 'royalties' ? 'selected' : ''}>Royalties</option>
                        <option value="partnership_k1" ${state.activity_type === 'partnership_k1' ? 'selected' : ''}>K-1 from partnership</option>
                        <option value="gaming" ${state.activity_type === 'gaming' ? 'selected' : ''}>Gaming / bingo</option>
                    </select>
                </label>
                <label><span data-i18n="view.s511.label.related">Substantially related to exempt purpose?</span>
                    <input type="checkbox" name="is_substantially_related" ${state.is_substantially_related ? 'checked' : ''}></label>
                <label><span data-i18n="view.s511.label.regular">Regularly carried on?</span>
                    <input type="checkbox" name="is_regularly_carried_on" ${state.is_regularly_carried_on ? 'checked' : ''}></label>
                <label><span data-i18n="view.s511.label.volunteer">Volunteer labor exception?</span>
                    <input type="checkbox" name="volunteer_labor_exception" ${state.volunteer_labor_exception ? 'checked' : ''}></label>
                <label><span data-i18n="view.s511.label.convenience">Convenience for members?</span>
                    <input type="checkbox" name="is_convenience_for_members" ${state.is_convenience_for_members ? 'checked' : ''}></label>
                <label><span data-i18n="view.s511.label.donated">Donated inventory?</span>
                    <input type="checkbox" name="is_donated_inventory" ${state.is_donated_inventory ? 'checked' : ''}></label>
                <label><span data-i18n="view.s511.label.gaming">Bingo / qualified gaming?</span>
                    <input type="checkbox" name="is_bingo_or_gaming" ${state.is_bingo_or_gaming ? 'checked' : ''}></label>
                <label><span data-i18n="view.s511.label.rental_re">Rental real property?</span>
                    <input type="checkbox" name="rental_real_property" ${state.rental_real_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s511.label.debt">Debt-financed property?</span>
                    <input type="checkbox" name="debt_financed_property" ${state.debt_financed_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s511.label.silo">Siloed other businesses loss ($)</span>
                    <input type="number" step="0.01" name="siloed_other_businesses_loss" value="${state.siloed_other_businesses_loss}"></label>
                <button class="primary" type="submit" data-i18n="view.s511.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s511-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s511.h2.test">UBI three-part test (§ 512)</h2>
            <ol class="muted small">
                <li data-i18n="view.s511.test.trade">TRADE OR BUSINESS: activity for production of income from sale of goods / performance of services</li>
                <li data-i18n="view.s511.test.regular">REGULARLY CARRIED ON: continuous or recurring (vs occasional / one-time)</li>
                <li data-i18n="view.s511.test.unrelated">NOT SUBSTANTIALLY RELATED: causal connection to exempt purpose required (not just income generation)</li>
                <li data-i18n="view.s511.test.all_three">ALL THREE elements required → UBTI</li>
                <li data-i18n="view.s511.test.related">If substantially related (gift shop in museum) → NOT UBTI</li>
                <li data-i18n="view.s511.test.aggregation">Aggregation: multiple unrelated activities considered separately (post-TCJA siloing)</li>
                <li data-i18n="view.s511.test.fragmentation">Fragmentation: bundled activities may be evaluated separately</li>
                <li data-i18n="view.s511.test.s512a6">§ 512(a)(6): separate computation per "trade or business" — defined per § 1.512(a)-6</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s511.h2.exceptions">UBI exceptions (§ 513)</h2>
            <ul class="muted small">
                <li data-i18n="view.s511.exc.volunteer">Volunteer labor: substantially all (85%+) by volunteers (§ 513(a)(1))</li>
                <li data-i18n="view.s511.exc.convenience">Convenience for members / students / employees (§ 513(a)(2)) — cafeteria, bookstore</li>
                <li data-i18n="view.s511.exc.donated">Donated inventory: 100% inventory donated (§ 513(a)(3)) — thrift store</li>
                <li data-i18n="view.s511.exc.bingo">Bingo / qualifying gaming (§ 513(f)) — primary purpose religious / charitable</li>
                <li data-i18n="view.s511.exc.publication">Distribution of low-cost items (§ 513(h)) — items ≤ $13.20 (2024 indexed)</li>
                <li data-i18n="view.s511.exc.fundraising">Fundraising events: not regularly carried on if annual / occasional</li>
                <li data-i18n="view.s511.exc.mailings">Exchanges + rentals of mailing lists: NOT UBI (§ 513(h)(B))</li>
                <li data-i18n="view.s511.exc.research">Research income for federal / academic purposes: § 512(b)(8) exception</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s511.h2.passive_excluded">§ 512(b) modifications — passive income excluded</h2>
            <ul class="muted small">
                <li data-i18n="view.s511.passive.dividends">Dividends + interest + royalties + capital gains (§ 512(b)(1)(2))</li>
                <li data-i18n="view.s511.passive.rents_real">Rents from REAL PROPERTY (§ 512(b)(3))</li>
                <li data-i18n="view.s511.passive.rents_personal">Personal property rents: ≤ 10% with real property OK; otherwise UBI</li>
                <li data-i18n="view.s511.passive.research_grants">Research grants from US gov / academic (§ 512(b)(7)-(9))</li>
                <li data-i18n="view.s511.passive.gains_losses">Capital gains / losses on investments (§ 512(b)(5))</li>
                <li data-i18n="view.s511.passive.debt_financed">EXCEPTION — DEBT-FINANCED PROPERTY (§ 514): UBI to extent of debt %</li>
                <li data-i18n="view.s511.passive.controlled_org">Controlled organization income (§ 512(b)(13)): from > 50% owned sub may be UBI</li>
                <li data-i18n="view.s511.passive.foreign_corporation">Foreign corp passive: § 512(b)(17) anti-abuse</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s511.h2.siloing">§ 512(a)(6) siloing (TCJA 2017)</h2>
            <ul class="muted small">
                <li data-i18n="view.s511.silo.purpose">Prevent net loss in one UBI activity from offsetting income in another</li>
                <li data-i18n="view.s511.silo.per_business">Compute UBTI separately for EACH trade or business</li>
                <li data-i18n="view.s511.silo.naics">Trade or business defined by NAICS 2-digit code (Reg § 1.512(a)-6)</li>
                <li data-i18n="view.s511.silo.aggregate">Within same NAICS: aggregate income + expenses</li>
                <li data-i18n="view.s511.silo.investment_silos">Investment partnership K-1s: aggregate by "facts + circumstances"</li>
                <li data-i18n="view.s511.silo.nol_per">NOLs: tracked per silo; cannot cross-silo</li>
                <li data-i18n="view.s511.silo.qualified_partnership">"Qualifying partnership interest" (QPI): ≤ 2% direct OR ≤ 20% with no control = grouped</li>
                <li data-i18n="view.s511.silo.s990t_columns">Form 990-T: separate columns per silo with own loss carryover</li>
            </ul>
        </div>
    `;
    document.getElementById('s511-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.gross_unrelated_income = Number(fd.get('gross_unrelated_income')) || 0;
        state.directly_related_expenses = Number(fd.get('directly_related_expenses')) || 0;
        state.indirect_allocations = Number(fd.get('indirect_allocations')) || 0;
        state.other_deductions = Number(fd.get('other_deductions')) || 0;
        state.nol_carryforward = Number(fd.get('nol_carryforward')) || 0;
        state.specific_deduction = Number(fd.get('specific_deduction')) || 0;
        state.is_university = !!fd.get('is_university');
        state.is_hospital = !!fd.get('is_hospital');
        state.activity_type = fd.get('activity_type');
        state.is_substantially_related = !!fd.get('is_substantially_related');
        state.is_regularly_carried_on = !!fd.get('is_regularly_carried_on');
        state.volunteer_labor_exception = !!fd.get('volunteer_labor_exception');
        state.is_convenience_for_members = !!fd.get('is_convenience_for_members');
        state.is_donated_inventory = !!fd.get('is_donated_inventory');
        state.is_bingo_or_gaming = !!fd.get('is_bingo_or_gaming');
        state.rental_real_property = !!fd.get('rental_real_property');
        state.debt_financed_property = !!fd.get('debt_financed_property');
        state.siloed_other_businesses_loss = Number(fd.get('siloed_other_businesses_loss')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s511-output');
    if (!el) return;
    const is_ubi = state.is_regularly_carried_on && !state.is_substantially_related && !state.volunteer_labor_exception && !state.is_convenience_for_members && !state.is_donated_inventory && !state.is_bingo_or_gaming && !(state.rental_real_property && !state.debt_financed_property);
    const total_deductions = state.directly_related_expenses + state.indirect_allocations + state.other_deductions;
    const ubti_before_nol = is_ubi ? Math.max(0, state.gross_unrelated_income - total_deductions) : 0;
    const ubti_after_nol = Math.max(0, ubti_before_nol - state.nol_carryforward);
    const ubti_after_specific = Math.max(0, ubti_after_nol - state.specific_deduction);
    const ubit_tax = ubti_after_specific * 0.21;
    const filing_required = ubti_before_nol >= 1_000;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s511.h2.result">§ 511 UBIT computation</h2>
            <div class="cards">
                <div class="card ${is_ubi ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s511.card.ubi">Is UBI?</div>
                    <div class="value">${is_ubi ? esc(t('view.s511.status.yes')) : esc(t('view.s511.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s511.card.gross">Gross UBI</div>
                    <div class="value">$${state.gross_unrelated_income.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s511.card.before_nol">UBTI before NOL</div>
                    <div class="value">$${ubti_before_nol.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s511.card.after_nol">After NOL + specific</div>
                    <div class="value">$${ubti_after_specific.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s511.card.tax">UBIT tax (21%)</div>
                    <div class="value">$${ubit_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${filing_required ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s511.card.filing">Form 990-T required?</div>
                    <div class="value">${filing_required ? esc(t('view.s511.status.yes')) : esc(t('view.s511.status.no'))}</div>
                </div>
            </div>
            ${ubti_after_specific > 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s511.tax_note">
                    UBIT TRIGGERED — file Form 990-T. Significant UBI may trigger IRS scrutiny of overall
                    501(c)(3) classification (if UBI is "substantial" portion of activity). Risk of revocation
                    if UBI dominates. Consider spinning UBI activity into for-profit sub or partnership.
                    § 512(a)(6) siloing limits cross-activity loss netting (TCJA 2017).
                </p>
            ` : ''}
        </div>
    `;
}
