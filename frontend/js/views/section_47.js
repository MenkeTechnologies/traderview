// IRC § 47 — Rehabilitation Tax Credit (Historic Buildings).
// 20% credit on qualified rehab expenditures for CERTIFIED HISTORIC structure.
// TCJA 2017: 10% credit (pre-1936 non-certified) ELIMINATED — only 20% historic remains.
// 5-year vesting: claim 4% per year over 5 years (vs prior single-year claim).
// NPS / SHPO three-part application: Parts 1 (eligibility), 2 (plans), 3 (completion).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    qualified_rehab_expenditures: 0,
    is_certified_historic: true,
    on_national_register: false,
    contributes_to_district: false,
    placed_in_service_year: 2024,
    npsApprovedParts: '',
    bldg_basis_before_rehab: 0,
    substantially_rehabilitated: false,
    leased_to_tax_exempt: false,
    elect_transferability: false,
    elect_direct_pay: false,
    five_year_recapture: false,
};

export async function renderSection47(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s47.h1.title">// § 47 HISTORIC REHAB</span></h1>
        <p class="muted small" data-i18n="view.s47.hint.intro">
            <strong>20% credit</strong> on Qualified Rehabilitation Expenditures (QREs) for CERTIFIED HISTORIC
            structures. <strong>TCJA 2017:</strong> 10% credit (pre-1936 non-certified) ELIMINATED. <strong>5-year
            vesting:</strong> claim 4% per year over 5 years (vs prior single-year). <strong>NPS / SHPO 3-part
            app:</strong> Part 1 evaluation of significance, Part 2 description of rehab, Part 3 certification.
            <strong>Substantial rehab test:</strong> QREs &gt; greater of $5K or building basis. <strong>5-yr
            recapture</strong> on sale or change of use. <strong>§ 6418 transferability.</strong> Form 3468.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s47.h2.inputs">Inputs</h2>
            <form id="s47-form" class="inline-form">
                <label><span data-i18n="view.s47.label.qre">QRE total ($)</span>
                    <input type="number" step="10000" name="qualified_rehab_expenditures" value="${state.qualified_rehab_expenditures}"></label>
                <label><span data-i18n="view.s47.label.certified">Certified Historic Structure?</span>
                    <input type="checkbox" name="is_certified_historic" ${state.is_certified_historic ? 'checked' : ''}></label>
                <label><span data-i18n="view.s47.label.register">On National Register?</span>
                    <input type="checkbox" name="on_national_register" ${state.on_national_register ? 'checked' : ''}></label>
                <label><span data-i18n="view.s47.label.district">Contributes to historic district?</span>
                    <input type="checkbox" name="contributes_to_district" ${state.contributes_to_district ? 'checked' : ''}></label>
                <label><span data-i18n="view.s47.label.year">Placed in service year</span>
                    <input type="number" step="1" name="placed_in_service_year" value="${state.placed_in_service_year}"></label>
                <label><span data-i18n="view.s47.label.nps">NPS approved parts (1, 2, 3 / 'all')</span>
                    <input type="text" name="npsApprovedParts" value="${esc(state.npsApprovedParts)}"></label>
                <label><span data-i18n="view.s47.label.basis_pre">Building basis before rehab ($)</span>
                    <input type="number" step="10000" name="bldg_basis_before_rehab" value="${state.bldg_basis_before_rehab}"></label>
                <label><span data-i18n="view.s47.label.substantial">Substantial rehabilitation test met?</span>
                    <input type="checkbox" name="substantially_rehabilitated" ${state.substantially_rehabilitated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s47.label.exempt_lease">Leased to tax-exempt &gt; 50%?</span>
                    <input type="checkbox" name="leased_to_tax_exempt" ${state.leased_to_tax_exempt ? 'checked' : ''}></label>
                <label><span data-i18n="view.s47.label.transfer">§ 6418 transferability?</span>
                    <input type="checkbox" name="elect_transferability" ${state.elect_transferability ? 'checked' : ''}></label>
                <label><span data-i18n="view.s47.label.direct">§ 6417 direct pay (limited)?</span>
                    <input type="checkbox" name="elect_direct_pay" ${state.elect_direct_pay ? 'checked' : ''}></label>
                <label><span data-i18n="view.s47.label.recapture">In 5-yr recapture period?</span>
                    <input type="checkbox" name="five_year_recapture" ${state.five_year_recapture ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s47.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s47-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s47.h2.qre_includes">QREs include / exclude</h2>
            <ul class="muted small">
                <li data-i18n="view.s47.qre.depreciable">Depreciable structural improvements: walls, floors, roofs, plumbing, electric, HVAC</li>
                <li data-i18n="view.s47.qre.architect">Architectural + engineering fees</li>
                <li data-i18n="view.s47.qre.developer">Developer / construction management fees</li>
                <li data-i18n="view.s47.qre.interior">Interior finish work to historic standard</li>
                <li data-i18n="view.s47.qre.exterior">Exterior preservation to historic standard (per Secretary of Interior Standards)</li>
                <li data-i18n="view.s47.qre.no_acquisition">EXCLUDES: acquisition cost of building, land</li>
                <li data-i18n="view.s47.qre.no_enlargement">EXCLUDES: enlargement / expansion (additions)</li>
                <li data-i18n="view.s47.qre.no_machinery">EXCLUDES: machinery, equipment, fixtures (personal property)</li>
                <li data-i18n="view.s47.qre.no_landscape">EXCLUDES: landscaping, site work, demolition (with some exceptions)</li>
                <li data-i18n="view.s47.qre.no_developer_above">EXCLUDES: developer fees in excess of arm's-length</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s47.h2.process">3-part NPS / SHPO process</h2>
            <ol class="muted small">
                <li data-i18n="view.s47.process.part1">Part 1 — Evaluation of Significance: confirms certified historic structure</li>
                <li data-i18n="view.s47.process.part2">Part 2 — Description of Rehabilitation: review rehabilitation plans BEFORE work</li>
                <li data-i18n="view.s47.process.part3">Part 3 — Request for Certification of Completed Work: AFTER project finished</li>
                <li data-i18n="view.s47.process.shpo">SHPO (State Historic Preservation Officer): initial reviewer in most states</li>
                <li data-i18n="view.s47.process.nps">NPS (National Park Service): final approving authority</li>
                <li data-i18n="view.s47.process.fee">User fees: $0-$5,000+ depending on project size</li>
                <li data-i18n="view.s47.process.timeline">Typical timeline: 6-9 months pre-construction; 6 months post-completion</li>
                <li data-i18n="view.s47.process.appeal">Appeals: 30 days to NPS within Washington</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s47.h2.recapture">Recapture rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s47.rec.5yr">5-year recapture period from placed-in-service date</li>
                <li data-i18n="view.s47.rec.schedule">Yr 1: 100% / Yr 2: 80% / Yr 3: 60% / Yr 4: 40% / Yr 5: 20%</li>
                <li data-i18n="view.s47.rec.triggers">Triggers: sale, change of use, demolition, change in historic character</li>
                <li data-i18n="view.s47.rec.exception">Casualty + restoration: no recapture if restored to historic standard</li>
                <li data-i18n="view.s47.rec.transfer_recapture">Transferability: recapture stays with ORIGINAL TAXPAYER</li>
                <li data-i18n="view.s47.rec.exempt_lease">Exempt lease &gt; 50% disqualifies (must rent at market to non-exempt)</li>
                <li data-i18n="view.s47.rec.basis_reduction">Basis reduction: 100% of credit reduces building basis</li>
                <li data-i18n="view.s47.rec.form_4255">Form 4255 to recapture if triggered</li>
            </ul>
        </div>
    `;
    document.getElementById('s47-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.qualified_rehab_expenditures = Number(fd.get('qualified_rehab_expenditures')) || 0;
        state.is_certified_historic = !!fd.get('is_certified_historic');
        state.on_national_register = !!fd.get('on_national_register');
        state.contributes_to_district = !!fd.get('contributes_to_district');
        state.placed_in_service_year = Number(fd.get('placed_in_service_year')) || 0;
        state.npsApprovedParts = fd.get('npsApprovedParts');
        state.bldg_basis_before_rehab = Number(fd.get('bldg_basis_before_rehab')) || 0;
        state.substantially_rehabilitated = !!fd.get('substantially_rehabilitated');
        state.leased_to_tax_exempt = !!fd.get('leased_to_tax_exempt');
        state.elect_transferability = !!fd.get('elect_transferability');
        state.elect_direct_pay = !!fd.get('elect_direct_pay');
        state.five_year_recapture = !!fd.get('five_year_recapture');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s47-output');
    if (!el) return;
    const eligible = state.is_certified_historic && state.substantially_rehabilitated && !state.leased_to_tax_exempt;
    const totalCredit = eligible ? state.qualified_rehab_expenditures * 0.20 : 0;
    const annualCredit5yr = totalCredit / 5;
    const basisReduction = totalCredit;
    const transferProceeds = state.elect_transferability ? totalCredit * 0.92 : totalCredit;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s47.h2.result">§ 47 credit computation</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s47.card.eligible">Eligible?</div>
                    <div class="value">${eligible ? esc(t('view.s47.status.yes')) : esc(t('view.s47.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s47.card.qre">QRE basis</div>
                    <div class="value">$${state.qualified_rehab_expenditures.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s47.card.total_credit">Total credit (20%)</div>
                    <div class="value">$${totalCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s47.card.annual">Annual claim (5-yr)</div>
                    <div class="value">$${annualCredit5yr.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s47.card.basis_reduce">Basis reduction</div>
                    <div class="value">$${basisReduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s47.card.transfer">Transfer cash (92%)</div>
                    <div class="value">$${transferProceeds.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!state.substantially_rehabilitated ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s47.subst_note">
                    Substantial rehabilitation test FAIL: QREs must exceed greater of $5K OR building basis
                    (excluding land) over 24-60 month measuring period. Phased work over multiple years may
                    require careful planning to meet test.
                </p>
            ` : ''}
        </div>
    `;
}
