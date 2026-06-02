// IRC § 2032 — Alternate Valuation Date (AVD).
// Election to value gross estate at 6 months after date of death (vs date-of-death FMV).
// Requirement: results in reduction in (a) gross estate value AND (b) sum of federal estate + GST tax.
// Election made on Form 706 — irrevocable once made.
// All-or-nothing: apply AVD to ENTIRE gross estate, not selectively.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    date_of_death_value: 0,
    avd_value_6_months_later: 0,
    sold_between_date_avd_value: 0,
    distributed_between_avd_value: 0,
    federal_estate_tax_dod: 0,
    federal_estate_tax_avd: 0,
    gst_tax_dod: 0,
    gst_tax_avd: 0,
    avd_election_made: false,
    estate_tax_return_due: '',
    six_months_after_death_date: '',
    decrease_in_gross_estate: 0,
    decrease_in_tax: 0,
    is_marital_deduction_estate: false,
    is_charitable_deduction_estate: false,
    portability_election: false,
};

export async function renderSection2032(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s2032.h1.title">// § 2032 ALTERNATE VALUATION</span></h1>
        <p class="muted small" data-i18n="view.s2032.hint.intro">
            ELECTION to value gross estate at <strong>6 months after date of death</strong> (vs date-of-death
            FMV). <strong>REQUIREMENT:</strong> election must result in REDUCTION in (a) gross estate value
            AND (b) sum of federal estate + GST tax. <strong>ALL OR NOTHING:</strong> applies to ENTIRE gross
            estate, not selectively. <strong>Property sold / distributed during 6-month period:</strong> use
            sale / distribution date (not 6-month date). <strong>Election on Form 706</strong> — IRREVOCABLE.
            <strong>Income tax basis</strong>: § 1014 → AVD value if elected.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s2032.h2.inputs">Inputs</h2>
            <form id="s2032-form" class="inline-form">
                <label><span data-i18n="view.s2032.label.dod">Date-of-death value ($)</span>
                    <input type="number" step="10000" name="date_of_death_value" value="${state.date_of_death_value}"></label>
                <label><span data-i18n="view.s2032.label.avd">AVD value 6 months later ($)</span>
                    <input type="number" step="10000" name="avd_value_6_months_later" value="${state.avd_value_6_months_later}"></label>
                <label><span data-i18n="view.s2032.label.sold">Sold between date / AVD value ($)</span>
                    <input type="number" step="10000" name="sold_between_date_avd_value" value="${state.sold_between_date_avd_value}"></label>
                <label><span data-i18n="view.s2032.label.distributed">Distributed value during period ($)</span>
                    <input type="number" step="10000" name="distributed_between_avd_value" value="${state.distributed_between_avd_value}"></label>
                <label><span data-i18n="view.s2032.label.tax_dod">Federal estate tax at DOD ($)</span>
                    <input type="number" step="10000" name="federal_estate_tax_dod" value="${state.federal_estate_tax_dod}"></label>
                <label><span data-i18n="view.s2032.label.tax_avd">Federal estate tax at AVD ($)</span>
                    <input type="number" step="10000" name="federal_estate_tax_avd" value="${state.federal_estate_tax_avd}"></label>
                <label><span data-i18n="view.s2032.label.gst_dod">GST tax at DOD ($)</span>
                    <input type="number" step="10000" name="gst_tax_dod" value="${state.gst_tax_dod}"></label>
                <label><span data-i18n="view.s2032.label.gst_avd">GST tax at AVD ($)</span>
                    <input type="number" step="10000" name="gst_tax_avd" value="${state.gst_tax_avd}"></label>
                <label><span data-i18n="view.s2032.label.elected">AVD election made?</span>
                    <input type="checkbox" name="avd_election_made" ${state.avd_election_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2032.label.return_due">Estate tax return due date</span>
                    <input type="date" name="estate_tax_return_due" value="${state.estate_tax_return_due}"></label>
                <label><span data-i18n="view.s2032.label.six_months">6 months after death date</span>
                    <input type="date" name="six_months_after_death_date" value="${state.six_months_after_death_date}"></label>
                <label><span data-i18n="view.s2032.label.dec_estate">Decrease in gross estate ($)</span>
                    <input type="number" step="10000" name="decrease_in_gross_estate" value="${state.decrease_in_gross_estate}"></label>
                <label><span data-i18n="view.s2032.label.dec_tax">Decrease in total tax ($)</span>
                    <input type="number" step="10000" name="decrease_in_tax" value="${state.decrease_in_tax}"></label>
                <label><span data-i18n="view.s2032.label.marital">Marital deduction estate?</span>
                    <input type="checkbox" name="is_marital_deduction_estate" ${state.is_marital_deduction_estate ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2032.label.charitable">Charitable deduction estate?</span>
                    <input type="checkbox" name="is_charitable_deduction_estate" ${state.is_charitable_deduction_estate ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2032.label.portability">Portability election?</span>
                    <input type="checkbox" name="portability_election" ${state.portability_election ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s2032.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s2032-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2032.h2.requirements">§ 2032 election requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.s2032.req.election">Election made by executor on Form 706 (Part 1, Line 17)</li>
                <li data-i18n="view.s2032.req.reduce_value">Must REDUCE gross estate value</li>
                <li data-i18n="view.s2032.req.reduce_tax">Must REDUCE total federal estate + GST tax</li>
                <li data-i18n="view.s2032.req.both_required">BOTH conditions required (cannot have one without other)</li>
                <li data-i18n="view.s2032.req.timing">Election deadline: due date of Form 706 (incl. extensions) — 9 months + 6 month extension</li>
                <li data-i18n="view.s2032.req.irrevocable">Election IRREVOCABLE once made on timely return</li>
                <li data-i18n="view.s2032.req.all_or_nothing">All or nothing: applies to ENTIRE gross estate</li>
                <li data-i18n="view.s2032.req.purpose">Purpose: relief for estates of declining value (post-mortem market crash)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2032.h2.special_property">Special property valuation</h2>
            <ul class="muted small">
                <li data-i18n="view.s2032.spec.sold">Sold during 6-month period: use SALE date FMV</li>
                <li data-i18n="view.s2032.spec.distributed">Distributed during period: use DISTRIBUTION date FMV</li>
                <li data-i18n="view.s2032.spec.exchanged">Exchanged: use EXCHANGE date FMV</li>
                <li data-i18n="view.s2032.spec.depreciable">Mere passage of time: use AVD if no significant change</li>
                <li data-i18n="view.s2032.spec.section_2032a">Cannot combine with § 2032A special use valuation (must choose one)</li>
                <li data-i18n="view.s2032.spec.farm_real_estate">Farm real estate: AVD often less than § 2032A special use</li>
                <li data-i18n="view.s2032.spec.partnership_interests">Partnership interests: subject to ongoing valuation discounts</li>
                <li data-i18n="view.s2032.spec.life_insurance">Life insurance: paid in cash before 6 months — payout date used</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2032.h2.benefits_drawbacks">Benefits vs drawbacks</h2>
            <ul class="muted small">
                <li data-i18n="view.s2032.bd.benefit_estate_tax">BENEFIT: reduce estate tax (40% top rate) in declining market</li>
                <li data-i18n="view.s2032.bd.benefit_appreciated">BENEFIT: also reduces income tax basis (§ 1014 → AVD value)</li>
                <li data-i18n="view.s2032.bd.benefit_post_mortem">BENEFIT: hedge against post-mortem market crash</li>
                <li data-i18n="view.s2032.bd.drawback_basis">DRAWBACK: lower step-up basis = more capital gain on future sale</li>
                <li data-i18n="view.s2032.bd.drawback_appreciated">DRAWBACK: NOT useful for estate near exemption (no tax to save)</li>
                <li data-i18n="view.s2032.bd.consideration_growth">CONSIDERATION: if estate grew, AVD increases tax — election rejected</li>
                <li data-i18n="view.s2032.bd.coordination_marital">Marital deduction: AVD may slightly affect QTIP calculations</li>
                <li data-i18n="view.s2032.bd.executor_judgment">Executor judgment: monitor 6-month value, decide before return deadline</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2032.h2.coordination">Coordination with other estate planning</h2>
            <ul class="muted small">
                <li data-i18n="view.s2032.coord.s1014">§ 1014 basis step-up: heirs' basis = AVD value (lower step-up if AVD lower)</li>
                <li data-i18n="view.s2032.coord.s2032a">§ 2032A special use: alternative valuation method (real property only)</li>
                <li data-i18n="view.s2032.coord.s2056">§ 2056 marital deduction: applies to AVD-valued property</li>
                <li data-i18n="view.s2032.coord.s2055">§ 2055 charitable deduction: similar</li>
                <li data-i18n="view.s2032.coord.s6166">§ 6166 installment: extended payment if AVD applied</li>
                <li data-i18n="view.s2032.coord.s2503">§ 2503 gift tax: prior 3-yr gifts NOT subject to AVD</li>
                <li data-i18n="view.s2032.coord.portability">DSUE portability (§ 2010(c)(4)): consider impact on surviving spouse</li>
                <li data-i18n="view.s2032.coord.disclaimer">§ 2518 disclaimers: separate election affecting AVD analysis</li>
            </ul>
        </div>
    `;
    document.getElementById('s2032-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.date_of_death_value = Number(fd.get('date_of_death_value')) || 0;
        state.avd_value_6_months_later = Number(fd.get('avd_value_6_months_later')) || 0;
        state.sold_between_date_avd_value = Number(fd.get('sold_between_date_avd_value')) || 0;
        state.distributed_between_avd_value = Number(fd.get('distributed_between_avd_value')) || 0;
        state.federal_estate_tax_dod = Number(fd.get('federal_estate_tax_dod')) || 0;
        state.federal_estate_tax_avd = Number(fd.get('federal_estate_tax_avd')) || 0;
        state.gst_tax_dod = Number(fd.get('gst_tax_dod')) || 0;
        state.gst_tax_avd = Number(fd.get('gst_tax_avd')) || 0;
        state.avd_election_made = !!fd.get('avd_election_made');
        state.estate_tax_return_due = fd.get('estate_tax_return_due');
        state.six_months_after_death_date = fd.get('six_months_after_death_date');
        state.decrease_in_gross_estate = Number(fd.get('decrease_in_gross_estate')) || 0;
        state.decrease_in_tax = Number(fd.get('decrease_in_tax')) || 0;
        state.is_marital_deduction_estate = !!fd.get('is_marital_deduction_estate');
        state.is_charitable_deduction_estate = !!fd.get('is_charitable_deduction_estate');
        state.portability_election = !!fd.get('portability_election');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s2032-output');
    if (!el) return;
    const decrease_estate = state.date_of_death_value - state.avd_value_6_months_later;
    const decrease_tax = (state.federal_estate_tax_dod + state.gst_tax_dod) - (state.federal_estate_tax_avd + state.gst_tax_avd);
    const election_eligible = decrease_estate > 0 && decrease_tax > 0;
    const tax_savings = election_eligible ? decrease_tax : 0;
    const basis_reduction = election_eligible ? decrease_estate : 0;
    const future_capital_gain_tax = basis_reduction * 0.20;
    const net_benefit = tax_savings - future_capital_gain_tax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s2032.h2.result">§ 2032 AVD analysis</h2>
            <div class="cards">
                <div class="card ${election_eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s2032.card.eligible">Election eligible?</div>
                    <div class="value">${election_eligible ? esc(t('view.s2032.status.yes')) : esc(t('view.s2032.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s2032.card.dec_estate">Decrease gross estate</div>
                    <div class="value">$${decrease_estate.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s2032.card.dec_tax">Decrease total tax</div>
                    <div class="value">$${decrease_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s2032.card.savings">Estate tax savings</div>
                    <div class="value">$${tax_savings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s2032.card.basis_reduce">Basis reduction (heirs)</div>
                    <div class="value">$${basis_reduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s2032.card.future_tax">Future capital gain tax (20%)</div>
                    <div class="value">$${future_capital_gain_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${net_benefit > 0 ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s2032.card.net">Net benefit</div>
                    <div class="value">$${net_benefit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!election_eligible ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s2032.not_eligible_note">
                    Election NOT eligible: estate value INCREASED OR estate tax NOT reduced. § 2032 requires
                    BOTH conditions. Likely scenarios: market recovery, exemption fully covers estate (no tax
                    benefit), or growing assets. Default to date-of-death valuation. Consider § 2032A special
                    use for real property if applicable.
                </p>
            ` : ''}
        </div>
    `;
}
