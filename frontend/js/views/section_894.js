// IRC § 894 — Treaty-Based Return Position (Form 8833).
// Required disclosure when taxpayer takes return position based on US tax treaty.
// § 6114(a) requires disclosure if treaty position reduces US tax by $10K+ (non-individual)
// or in any amount for individuals where treaty waived withholding.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    treaty_country: 'UK',
    treaty_article: '',
    is_individual: false,
    treaty_position: 'reduced_withholding',
    us_tax_reduction: 0,
    form_8833_filed: false,
    is_dual_resident: false,
    tie_breaker_invoked: false,
    is_permanent_establishment: false,
    business_profits_excluded: 0,
    dividend_treaty_rate: 15,
    interest_treaty_rate: 0,
    royalty_treaty_rate: 0,
    payee_is_qualified_resident: true,
    article_22_lob_satisfied: true,
    is_fdap_income: true,
    is_eci: false,
    days_present_in_us: 0,
    is_treaty_resident: true,
};

export async function renderSection894(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s894.h1.title">// § 894 TREATY POSITION (Form 8833)</span></h1>
        <p class="muted small" data-i18n="view.s894.hint.intro">
            <strong>Form 8833</strong> (Treaty-Based Return Position) required under <strong>§ 6114(a)</strong>
            when taking return position based on US tax treaty. <strong>Threshold:</strong> $10K+ tax
            reduction (non-individual) OR any amount where treaty waived withholding (individual).
            <strong>§ 894 limits:</strong> treaty does NOT reduce tax on US-source income unless
            beneficial owner is qualified resident + satisfies LOB (Limitation on Benefits) Article 22.
            <strong>§ 894(c)</strong> hybrid entity rule + <strong>§ 7701(l)</strong> conduit rules
            override treaty for back-to-back arrangements. <strong>§ 6712 penalty:</strong> $1,000 per
            failure to disclose ($10,000 for C-corp). <strong>Common treaties:</strong> UK (1980), Canada
            (1980/1995/2008 Protocol), Germany (1989/2006 Protocol), Japan (2003/2013 Protocol).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s894.h2.inputs">Inputs</h2>
            <form id="s894-form" class="inline-form">
                <label><span data-i18n="view.s894.label.country">Treaty country</span>
                    <select name="treaty_country">
                        <option value="UK" ${state.treaty_country === 'UK' ? 'selected' : ''}>United Kingdom (1980 + 2001 Protocol)</option>
                        <option value="Canada" ${state.treaty_country === 'Canada' ? 'selected' : ''}>Canada (1980 + 1995 + 2008 Protocols)</option>
                        <option value="Germany" ${state.treaty_country === 'Germany' ? 'selected' : ''}>Germany (1989 + 2006 Protocol)</option>
                        <option value="Japan" ${state.treaty_country === 'Japan' ? 'selected' : ''}>Japan (2003 + 2013 Protocol)</option>
                        <option value="France" ${state.treaty_country === 'France' ? 'selected' : ''}>France (1994 + 2009 Protocol)</option>
                        <option value="Switzerland" ${state.treaty_country === 'Switzerland' ? 'selected' : ''}>Switzerland (1996 + 2019 Protocol)</option>
                        <option value="Netherlands" ${state.treaty_country === 'Netherlands' ? 'selected' : ''}>Netherlands (1992 + 2004 Protocol)</option>
                        <option value="Ireland" ${state.treaty_country === 'Ireland' ? 'selected' : ''}>Ireland (1997 + 1999 Protocol)</option>
                        <option value="other" ${state.treaty_country === 'other' ? 'selected' : ''}>Other (50+ US treaties)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s894.label.article">Treaty article</span>
                    <input type="text" name="treaty_article" value="${esc(state.treaty_article)}"></label>
                <label><span data-i18n="view.s894.label.individual">Individual?</span>
                    <input type="checkbox" name="is_individual" ${state.is_individual ? 'checked' : ''}></label>
                <label><span data-i18n="view.s894.label.position">Treaty position type</span>
                    <select name="treaty_position">
                        <option value="reduced_withholding" ${state.treaty_position === 'reduced_withholding' ? 'selected' : ''}>Reduced § 1441 withholding</option>
                        <option value="exempt_business_profits" ${state.treaty_position === 'exempt_business_profits' ? 'selected' : ''}>Exempt business profits (no PE)</option>
                        <option value="tie_breaker" ${state.treaty_position === 'tie_breaker' ? 'selected' : ''}>Dual-resident tie-breaker</option>
                        <option value="permanent_home" ${state.treaty_position === 'permanent_home' ? 'selected' : ''}>Permanent home (residence)</option>
                        <option value="article_19" ${state.treaty_position === 'article_19' ? 'selected' : ''}>Article 19 (government service)</option>
                        <option value="article_20" ${state.treaty_position === 'article_20' ? 'selected' : ''}>Article 20 (students/trainees)</option>
                        <option value="article_21" ${state.treaty_position === 'article_21' ? 'selected' : ''}>Article 21 (other income)</option>
                        <option value="183_day_test" ${state.treaty_position === '183_day_test' ? 'selected' : ''}>183-day test (dependent services)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s894.label.reduction">US tax reduction ($)</span>
                    <input type="number" step="0.01" name="us_tax_reduction" value="${state.us_tax_reduction}"></label>
                <label><span data-i18n="view.s894.label.filed">Form 8833 filed?</span>
                    <input type="checkbox" name="form_8833_filed" ${state.form_8833_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s894.label.dual">Dual-resident?</span>
                    <input type="checkbox" name="is_dual_resident" ${state.is_dual_resident ? 'checked' : ''}></label>
                <label><span data-i18n="view.s894.label.tiebreaker">Tie-breaker invoked?</span>
                    <input type="checkbox" name="tie_breaker_invoked" ${state.tie_breaker_invoked ? 'checked' : ''}></label>
                <label><span data-i18n="view.s894.label.pe">Permanent establishment?</span>
                    <input type="checkbox" name="is_permanent_establishment" ${state.is_permanent_establishment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s894.label.business_excluded">Business profits excluded ($)</span>
                    <input type="number" step="0.01" name="business_profits_excluded" value="${state.business_profits_excluded}"></label>
                <label><span data-i18n="view.s894.label.div_rate">Dividend treaty %</span>
                    <input type="number" step="0.1" name="dividend_treaty_rate" value="${state.dividend_treaty_rate}"></label>
                <label><span data-i18n="view.s894.label.int_rate">Interest treaty %</span>
                    <input type="number" step="0.1" name="interest_treaty_rate" value="${state.interest_treaty_rate}"></label>
                <label><span data-i18n="view.s894.label.roy_rate">Royalty treaty %</span>
                    <input type="number" step="0.1" name="royalty_treaty_rate" value="${state.royalty_treaty_rate}"></label>
                <label><span data-i18n="view.s894.label.qres">Qualified resident?</span>
                    <input type="checkbox" name="payee_is_qualified_resident" ${state.payee_is_qualified_resident ? 'checked' : ''}></label>
                <label><span data-i18n="view.s894.label.lob">Art 22 LOB satisfied?</span>
                    <input type="checkbox" name="article_22_lob_satisfied" ${state.article_22_lob_satisfied ? 'checked' : ''}></label>
                <label><span data-i18n="view.s894.label.fdap">FDAP income?</span>
                    <input type="checkbox" name="is_fdap_income" ${state.is_fdap_income ? 'checked' : ''}></label>
                <label><span data-i18n="view.s894.label.eci">ECI?</span>
                    <input type="checkbox" name="is_eci" ${state.is_eci ? 'checked' : ''}></label>
                <label><span data-i18n="view.s894.label.days_us">Days in US</span>
                    <input type="number" step="1" name="days_present_in_us" value="${state.days_present_in_us}"></label>
                <label><span data-i18n="view.s894.label.treaty_resident">Treaty resident?</span>
                    <input type="checkbox" name="is_treaty_resident" ${state.is_treaty_resident ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s894.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s894-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s894.h2.thresholds">Form 8833 disclosure thresholds (§ 6114)</h2>
            <ul class="muted small">
                <li data-i18n="view.s894.thresh.10k">$10,000+ US tax reduction (non-individuals)</li>
                <li data-i18n="view.s894.thresh.any_individual">Any amount where withholding waived (individuals)</li>
                <li data-i18n="view.s894.thresh.s7701_l">§ 7701(l) anti-conduit rules treaty override</li>
                <li data-i18n="view.s894.thresh.s894_c">§ 894(c) hybrid entity bypass — treaty inapplicable</li>
                <li data-i18n="view.s894.thresh.dual_resident">Dual-resident treaty position (tie-breaker)</li>
                <li data-i18n="view.s894.thresh.exempt_pe">Treaty article exempts US trade or business profits (no PE)</li>
                <li data-i18n="view.s894.thresh.no_disclosure">Routine treaty rate withholding (1441) — generally no disclosure</li>
                <li data-i18n="view.s894.thresh.s6712">§ 6712 penalty: $1K individual / $10K C-corp per failure</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s894.h2.lob">Limitation on Benefits (Article 22)</h2>
            <ol class="muted small">
                <li data-i18n="view.s894.lob.individual_test">Individual qualified resident test</li>
                <li data-i18n="view.s894.lob.publicly_traded">Publicly traded company test (substantial trading)</li>
                <li data-i18n="view.s894.lob.subsidiary">Subsidiary of publicly traded company</li>
                <li data-i18n="view.s894.lob.ownership_base_erosion">Ownership/base erosion test (50% test)</li>
                <li data-i18n="view.s894.lob.active_trade">Active trade or business test</li>
                <li data-i18n="view.s894.lob.derivative">Derivative benefits test (US treaty country resident)</li>
                <li data-i18n="view.s894.lob.headquarters">Headquarters company test</li>
                <li data-i18n="view.s894.lob.competent_authority">Competent authority discretionary determination</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s894.h2.tiebreaker">Tie-breaker rules (Article 4)</h2>
            <ol class="muted small">
                <li data-i18n="view.s894.tie.permanent_home">Permanent home available in only one contracting state</li>
                <li data-i18n="view.s894.tie.center_vital">Center of vital interests (closer personal + economic relations)</li>
                <li data-i18n="view.s894.tie.habitual">Habitual abode</li>
                <li data-i18n="view.s894.tie.national">National of only one state</li>
                <li data-i18n="view.s894.tie.competent">Competent authorities mutual agreement</li>
                <li data-i18n="view.s894.tie.lcus">Long-term green-card holder rule: 10/15-year US tax resident</li>
                <li data-i18n="view.s894.tie.s7701b_e">§ 7701(b)(6) treaty election to be taxed as nonresident</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s894.h2.treaty_rates">Common treaty rates (US-source FDAP)</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s894.tbl.country">Country</th><th data-i18n="view.s894.tbl.dividends">Dividends</th><th data-i18n="view.s894.tbl.interest">Interest</th><th data-i18n="view.s894.tbl.royalties">Royalties</th></tr></thead>
                <tbody>
                    <tr><td>UK</td><td>0/15%</td><td>0%</td><td>0%</td></tr>
                    <tr><td>Canada</td><td>5/15%</td><td>0%</td><td>0/10%</td></tr>
                    <tr><td>Germany</td><td>0/5/15%</td><td>0%</td><td>0%</td></tr>
                    <tr><td>Japan</td><td>0/5/10%</td><td>10%</td><td>0%</td></tr>
                    <tr><td>France</td><td>5/15%</td><td>0%</td><td>0/5%</td></tr>
                    <tr><td>Switzerland</td><td>5/15%</td><td>0%</td><td>0%</td></tr>
                    <tr><td>Netherlands</td><td>0/5/15%</td><td>0%</td><td>0%</td></tr>
                    <tr><td>Ireland</td><td>5/15%</td><td>0%</td><td>0%</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s894.h2.treaty_override">§ 894 + § 7701(l) treaty override mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s894.over.hybrid">§ 894(c) hybrid entity: payment to person not eligible</li>
                <li data-i18n="view.s894.over.conduit">§ 7701(l) anti-conduit: back-to-back arrangement collapsed</li>
                <li data-i18n="view.s894.over.beneficial">Beneficial ownership test essential — not pass-through</li>
                <li data-i18n="view.s894.over.dr_2019">§ 894(c)(1) post-TCJA: covers hybrid dividends + § 245A connection</li>
                <li data-i18n="view.s894.over.s954_d">§ 954(d)(4) hybrid mismatch rules subpart F coordination</li>
                <li data-i18n="view.s894.over.beat">§ 59A BEAT applies despite treaty (10% base erosion threshold)</li>
                <li data-i18n="view.s894.over.s871_m">§ 871(m) dividend equivalents not treaty-eligible (substitute payments)</li>
                <li data-i18n="view.s894.over.savings_clause">Savings clause: US can tax US citizens regardless of treaty</li>
                <li data-i18n="view.s894.over.lwc">Last-in-time rule: later statute overrides treaty (Cook v. United States)</li>
            </ul>
        </div>
    `;
    document.getElementById('s894-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.treaty_country = fd.get('treaty_country');
        state.treaty_article = fd.get('treaty_article') || '';
        state.is_individual = !!fd.get('is_individual');
        state.treaty_position = fd.get('treaty_position');
        state.us_tax_reduction = Number(fd.get('us_tax_reduction')) || 0;
        state.form_8833_filed = !!fd.get('form_8833_filed');
        state.is_dual_resident = !!fd.get('is_dual_resident');
        state.tie_breaker_invoked = !!fd.get('tie_breaker_invoked');
        state.is_permanent_establishment = !!fd.get('is_permanent_establishment');
        state.business_profits_excluded = Number(fd.get('business_profits_excluded')) || 0;
        state.dividend_treaty_rate = Number(fd.get('dividend_treaty_rate')) || 0;
        state.interest_treaty_rate = Number(fd.get('interest_treaty_rate')) || 0;
        state.royalty_treaty_rate = Number(fd.get('royalty_treaty_rate')) || 0;
        state.payee_is_qualified_resident = !!fd.get('payee_is_qualified_resident');
        state.article_22_lob_satisfied = !!fd.get('article_22_lob_satisfied');
        state.is_fdap_income = !!fd.get('is_fdap_income');
        state.is_eci = !!fd.get('is_eci');
        state.days_present_in_us = Number(fd.get('days_present_in_us')) || 0;
        state.is_treaty_resident = !!fd.get('is_treaty_resident');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s894-output');
    if (!el) return;
    const required = state.is_individual ? state.us_tax_reduction > 0 : state.us_tax_reduction >= 10_000;
    const penalty = required && !state.form_8833_filed ? (state.is_individual ? 1000 : 10000) : 0;
    const treaty_benefits_disqualified = !state.payee_is_qualified_resident || !state.article_22_lob_satisfied;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s894.h2.result">§ 894 treaty position assessment</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s894.card.country">Treaty country</div>
                    <div class="value">${esc(state.treaty_country)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s894.card.reduction">US tax reduction</div>
                    <div class="value">$${state.us_tax_reduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${required ? 'warn' : 'pos'}">
                    <div class="label" data-i18n="view.s894.card.required">Form 8833 required?</div>
                    <div class="value">${required ? 'YES' : 'NO'}</div>
                </div>
                <div class="card ${penalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s894.card.penalty">§ 6712 penalty</div>
                    <div class="value">$${penalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${treaty_benefits_disqualified ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s894.card.lob">LOB / qualified res</div>
                    <div class="value">${treaty_benefits_disqualified ? 'DISQUALIFIED' : 'QUALIFIES'}</div>
                </div>
            </div>
        </div>
    `;
}
