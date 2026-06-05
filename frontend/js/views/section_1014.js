// IRC § 1014 — Basis Step-Up at Death.
// Heir's basis in inherited property = FMV at decedent's death (or alternate valuation date).
// Wipes out all built-in gain accrued during decedent's lifetime — major estate-planning tool.
// § 1014(c) IRD: NO step-up for income-in-respect-of-decedent (traditional IRA, annuity, etc.).
// Joint property: only DECEDENT'S half steps up; surviving spouse retains original basis on theirs.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    asset_type: 'stock',
    decedent_basis: 0,
    fmv_at_death: 0,
    use_alt_valuation: false,
    fmv_at_alt_date: 0,
    ownership_kind: 'sole',
    spouse_basis_in_share: 0,
    is_community_property: false,
    is_ird: false,
    estate_marginal_rate: 0.40,
    heir_ltcg_rate: 0.20,
    expected_growth_after_inherit: 0.07,
    years_until_heir_sells: 10,
};

export async function renderSection1014(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1014.h1.title">// § 1014 STEP-UP AT DEATH</span></h1>
        <p class="muted small" data-i18n="view.s1014.hint.intro">
            Heir's basis = <strong>FMV at decedent's death</strong> (or alternate valuation
            6 months later). Wipes built-in gain accrued during decedent's lifetime. <strong>§ 1014(c)
            IRD exception:</strong> NO step-up for traditional IRA / 401(k) / annuity / NQ deferred
            comp. <strong>Joint property:</strong> only decedent's half steps up. <strong>Community
            property (AZ, CA, ID, LA, NV, NM, TX, WA, WI + AK opt-in):</strong> BOTH halves step up.
            Most powerful estate-planning rule.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1014.h2.inputs">Inputs</h2>
            <form id="s1014-form" class="inline-form">
                <label><span data-i18n="view.s1014.label.asset_type">Asset type</span>
                    <select name="asset_type">
                        <option value="stock" ${state.asset_type === 'stock' ? 'selected' : ''}>Stock / securities</option>
                        <option value="real_estate" ${state.asset_type === 'real_estate' ? 'selected' : ''}>Real estate</option>
                        <option value="biz_interest" ${state.asset_type === 'biz_interest' ? 'selected' : ''}>Business interest</option>
                        <option value="collectibles" ${state.asset_type === 'collectibles' ? 'selected' : ''}>Collectibles</option>
                        <option value="ira" ${state.asset_type === 'ira' ? 'selected' : ''}>Traditional IRA / 401(k) (NO step-up)</option>
                        <option value="roth" ${state.asset_type === 'roth' ? 'selected' : ''}>Roth IRA (already tax-free)</option>
                        <option value="annuity" ${state.asset_type === 'annuity' ? 'selected' : ''}>NQ Annuity (IRD)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1014.label.basis">Decedent's basis ($)</span>
                    <input type="number" step="0.01" name="decedent_basis" value="${state.decedent_basis}"></label>
                <label><span data-i18n="view.s1014.label.fmv">FMV at death ($)</span>
                    <input type="number" step="0.01" name="fmv_at_death" value="${state.fmv_at_death}"></label>
                <label><span data-i18n="view.s1014.label.alt_val">Use alternate valuation (6 mo)?</span>
                    <input type="checkbox" name="use_alt_valuation" ${state.use_alt_valuation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1014.label.fmv_alt">FMV at alt date ($)</span>
                    <input type="number" step="0.01" name="fmv_at_alt_date" value="${state.fmv_at_alt_date}"></label>
                <label><span data-i18n="view.s1014.label.ownership">Ownership kind</span>
                    <select name="ownership_kind">
                        <option value="sole" ${state.ownership_kind === 'sole' ? 'selected' : ''}>Sole-owned</option>
                        <option value="jtwros_spouse" ${state.ownership_kind === 'jtwros_spouse' ? 'selected' : ''}>JTWROS with spouse</option>
                        <option value="jtwros_other" ${state.ownership_kind === 'jtwros_other' ? 'selected' : ''}>JTWROS with non-spouse</option>
                        <option value="tic" ${state.ownership_kind === 'tic' ? 'selected' : ''}>Tenants in Common</option>
                        <option value="community" ${state.ownership_kind === 'community' ? 'selected' : ''}>Community property</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1014.label.spouse_basis">Spouse's basis in joint share ($)</span>
                    <input type="number" step="0.01" name="spouse_basis_in_share" value="${state.spouse_basis_in_share}"></label>
                <label><span data-i18n="view.s1014.label.community">Community property state?</span>
                    <input type="checkbox" name="is_community_property" ${state.is_community_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1014.label.ird">Income in Respect of Decedent (IRD)?</span>
                    <input type="checkbox" name="is_ird" ${state.is_ird ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1014.label.estate_rate">Estate marginal rate</span>
                    <input type="number" step="0.01" name="estate_marginal_rate" value="${state.estate_marginal_rate}"></label>
                <label><span data-i18n="view.s1014.label.heir_rate">Heir LTCG rate</span>
                    <input type="number" step="0.01" name="heir_ltcg_rate" value="${state.heir_ltcg_rate}"></label>
                <label><span data-i18n="view.s1014.label.growth">Expected post-inherit growth</span>
                    <input type="number" step="0.01" name="expected_growth_after_inherit" value="${state.expected_growth_after_inherit}"></label>
                <label><span data-i18n="view.s1014.label.years">Years until heir sells</span>
                    <input type="number" step="1" name="years_until_heir_sells" value="${state.years_until_heir_sells}"></label>
                <button class="primary" type="submit" data-i18n="view.s1014.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1014-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1014.h2.strategies">Step-up planning strategies</h2>
            <ul class="muted small">
                <li data-i18n="view.s1014.strat.hold">Hold appreciated assets in taxable estate until death (vs. selling pre-death)</li>
                <li data-i18n="view.s1014.strat.upstream">"Upstream gifting" to elderly parents: they die, full step-up, you reinherit</li>
                <li data-i18n="view.s1014.strat.community_lookback">Community property: BOTH halves step up — vs JTWROS (only decedent's)</li>
                <li data-i18n="view.s1014.strat.section_2038">§ 2038 retained powers: keep assets in estate intentionally for step-up</li>
                <li data-i18n="view.s1014.strat.section_2036">§ 2036 retained life estate: includable in estate → full step-up</li>
                <li data-i18n="view.s1014.strat.idgt">IDGT: grantor trust during life, step-up if power swap exercised at death</li>
                <li data-i18n="view.s1014.strat.qtip">QTIP marital deduction property step-up at SECOND spouse's death</li>
                <li data-i18n="view.s1014.strat.roth_pre_death">Roth conversion before death = step-up wasted (already tax-free); for IRD assets, Roth convert PRE-death</li>
                <li data-i18n="view.s1014.strat.aging_parent_dollar">"Aging parent dollar": gift cash to elderly parent + they buy appreciated asset → step-up at parent's death → reinherit</li>
            </ul>
        </div>
    `;
    document.getElementById('s1014-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.asset_type = fd.get('asset_type');
        state.decedent_basis = Number(fd.get('decedent_basis')) || 0;
        state.fmv_at_death = Number(fd.get('fmv_at_death')) || 0;
        state.use_alt_valuation = !!fd.get('use_alt_valuation');
        state.fmv_at_alt_date = Number(fd.get('fmv_at_alt_date')) || 0;
        state.ownership_kind = fd.get('ownership_kind');
        state.spouse_basis_in_share = Number(fd.get('spouse_basis_in_share')) || 0;
        state.is_community_property = !!fd.get('is_community_property');
        state.is_ird = !!fd.get('is_ird');
        state.estate_marginal_rate = Number(fd.get('estate_marginal_rate')) || 0.40;
        state.heir_ltcg_rate = Number(fd.get('heir_ltcg_rate')) || 0.20;
        state.expected_growth_after_inherit = Number(fd.get('expected_growth_after_inherit')) || 0.07;
        state.years_until_heir_sells = Number(fd.get('years_until_heir_sells')) || 10;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1014-output');
    if (!el) return;
    const noStepUp = state.is_ird || state.asset_type === 'ira' || state.asset_type === 'annuity';
    const useFmv = state.use_alt_valuation ? state.fmv_at_alt_date : state.fmv_at_death;
    let newBasis;
    if (noStepUp) {
        newBasis = state.decedent_basis;
    } else if (state.ownership_kind === 'sole') {
        newBasis = useFmv;
    } else if (state.ownership_kind === 'community' || state.is_community_property) {
        newBasis = useFmv;
    } else if (state.ownership_kind === 'jtwros_spouse') {
        newBasis = useFmv * 0.5 + state.spouse_basis_in_share * 0.5;
    } else {
        newBasis = useFmv * 0.5 + state.decedent_basis * 0.5;
    }
    const builtInGainEliminated = noStepUp ? 0 : Math.max(0, newBasis - state.decedent_basis);
    const heirTaxSavedNow = builtInGainEliminated * state.heir_ltcg_rate;
    const futureValueAtSale = useFmv * Math.pow(1 + state.expected_growth_after_inherit, state.years_until_heir_sells);
    const futureGain = Math.max(0, futureValueAtSale - newBasis);
    const futureTax = futureGain * state.heir_ltcg_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1014.h2.result">Step-up outcome</h2>
            <div class="cards">
                <div class="card ${noStepUp ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1014.card.step_up">Step-up available?</div>
                    <div class="value">${noStepUp ? esc(t('view.s1014.status.no')) : esc(t('view.s1014.status.yes'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1014.card.original">Decedent's basis</div>
                    <div class="value">$${state.decedent_basis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1014.card.new_basis">Heir's new basis</div>
                    <div class="value">$${newBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1014.card.eliminated">Built-in gain eliminated</div>
                    <div class="value">$${builtInGainEliminated.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1014.card.tax_saved">Tax saved at step-up</div>
                    <div class="value">$${heirTaxSavedNow.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1014.card.future_value">Future value at sale</div>
                    <div class="value">$${futureValueAtSale.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1014.card.future_tax">Future LTCG tax</div>
                    <div class="value">$${futureTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${noStepUp ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s1014.warning.ird">
                    § 1014(c) IRD exception: no basis step-up. Heir owes ordinary income tax on
                    distributions. § 691(c) deduction available for estate tax attributable to IRD.
                </p>
            ` : ''}
        </div>
    `;
}
