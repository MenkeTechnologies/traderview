// IRC § 1015 — Basis of Property Acquired by Gift.
// CARRYOVER BASIS rule: donee takes donor's adjusted basis.
// Exception: if FMV at gift < donor's basis, dual basis applies (loss = FMV; gain = donor's basis).
// Holding period TACKS from donor's holding period.
// Gift tax basis adjustment: § 1015(d)(6) — add portion of gift tax attributable to net appreciation.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    donor_basis: 0,
    fmv_at_gift: 0,
    sale_price: 0,
    gift_tax_paid: 0,
    annual_exclusion_used: 0,
    donor_holding_period_days: 0,
    is_appreciated_property: true,
    is_step_up_eligible: false,
    is_post_death_gift: false,
    donor_acquired_year: 0,
    annual_exclusion_2025: 19_000,
    related_party_donor: false,
    spousal_gift: false,
};

export async function renderSection1015(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1015.h1.title">// § 1015 GIFT BASIS</span></h1>
        <p class="muted small" data-i18n="view.s1015.hint.intro">
            <strong>CARRYOVER basis</strong>: donee takes donor's adjusted basis. <strong>Holding period TACKS</strong>
            from donor's. <strong>Dual basis rule:</strong> if FMV at gift &lt; donor's basis: GAIN uses donor's basis;
            LOSS uses FMV at gift. <strong>"NO GAIN / NO LOSS" zone</strong>: between FMV at gift and donor's basis.
            <strong>§ 1015(d)(6) gift tax adjustment:</strong> add portion of gift tax attributable to net
            appreciation. <strong>Spousal gift:</strong> § 1041 carryover basis + no gift tax (with elections).
            <strong>Compared to § 1014:</strong> death gives FULL STEP-UP to FMV (gift loses this).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1015.h2.inputs">Inputs</h2>
            <form id="s1015-form" class="inline-form">
                <label><span data-i18n="view.s1015.label.donor">Donor's adjusted basis ($)</span>
                    <input type="number" step="1000" name="donor_basis" value="${state.donor_basis}"></label>
                <label><span data-i18n="view.s1015.label.fmv">FMV at gift date ($)</span>
                    <input type="number" step="1000" name="fmv_at_gift" value="${state.fmv_at_gift}"></label>
                <label><span data-i18n="view.s1015.label.sale">Sale price by donee ($)</span>
                    <input type="number" step="1000" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.s1015.label.gift_tax">Gift tax paid by donor ($)</span>
                    <input type="number" step="100" name="gift_tax_paid" value="${state.gift_tax_paid}"></label>
                <label><span data-i18n="view.s1015.label.annual">Annual exclusion used ($)</span>
                    <input type="number" step="1000" name="annual_exclusion_used" value="${state.annual_exclusion_used}"></label>
                <label><span data-i18n="view.s1015.label.holding">Donor's holding period (days)</span>
                    <input type="number" step="1" name="donor_holding_period_days" value="${state.donor_holding_period_days}"></label>
                <label><span data-i18n="view.s1015.label.appreciated">Appreciated property?</span>
                    <input type="checkbox" name="is_appreciated_property" ${state.is_appreciated_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1015.label.step_up">Step-up eligible (1014 death)?</span>
                    <input type="checkbox" name="is_step_up_eligible" ${state.is_step_up_eligible ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1015.label.post_death">Post-death gift?</span>
                    <input type="checkbox" name="is_post_death_gift" ${state.is_post_death_gift ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1015.label.acq_year">Donor acquired year</span>
                    <input type="number" step="1" name="donor_acquired_year" value="${state.donor_acquired_year}"></label>
                <label><span data-i18n="view.s1015.label.exclusion_2025">Annual exclusion 2025 ($)</span>
                    <input type="number" step="500" name="annual_exclusion_2025" value="${state.annual_exclusion_2025}"></label>
                <label><span data-i18n="view.s1015.label.related">Related party donor?</span>
                    <input type="checkbox" name="related_party_donor" ${state.related_party_donor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1015.label.spousal">Spousal gift (§ 1041)?</span>
                    <input type="checkbox" name="spousal_gift" ${state.spousal_gift ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s1015.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1015-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1015.h2.dual_basis">Dual basis rule (loss / gain different basis)</h2>
            <ul class="muted small">
                <li data-i18n="view.s1015.db.appreciation">If FMV at gift ≥ donor's basis: STANDARD carryover (single basis = donor's basis)</li>
                <li data-i18n="view.s1015.db.depreciation">If FMV at gift &lt; donor's basis: DUAL BASIS applies</li>
                <li data-i18n="view.s1015.db.gain_basis">FOR GAIN: use donor's basis (no recognition of pre-gift loss)</li>
                <li data-i18n="view.s1015.db.loss_basis">FOR LOSS: use FMV at gift (no recognition of pre-gift depreciation)</li>
                <li data-i18n="view.s1015.db.zone">"No-gain / no-loss zone": sale price between FMV at gift and donor's basis</li>
                <li data-i18n="view.s1015.db.example">Example: donor's basis $100K, FMV at gift $80K; sale at $90K = NO GAIN / NO LOSS</li>
                <li data-i18n="view.s1015.db.example_loss">Sale at $70K → LOSS of $10K (basis = $80K FMV)</li>
                <li data-i18n="view.s1015.db.example_gain">Sale at $110K → GAIN of $10K (basis = $100K donor's)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1015.h2.gift_tax_adj">§ 1015(d)(6) gift tax basis adjustment</h2>
            <ol class="muted small">
                <li data-i18n="view.s1015.gtb.formula">Adjustment = gift tax × (net appreciation / net gift)</li>
                <li data-i18n="view.s1015.gtb.appreciation">Net appreciation = FMV at gift − donor's basis</li>
                <li data-i18n="view.s1015.gtb.net_gift">Net gift = FMV at gift − annual exclusion used ($19K 2025)</li>
                <li data-i18n="view.s1015.gtb.who_paid">Applied when DONOR PAID gift tax (not donee)</li>
                <li data-i18n="view.s1015.gtb.add_to_basis">Adjusted basis = donor's basis + gift tax adjustment</li>
                <li data-i18n="view.s1015.gtb.cannot_exceed_fmv">Adjusted basis cannot EXCEED FMV at gift (no further step-up)</li>
                <li data-i18n="view.s1015.gtb.example">Example: donor's basis $50K, FMV $200K, annual excl $19K, gift tax $30K. Adjustment = $30K × ($150K / $181K) = $24,862</li>
                <li data-i18n="view.s1015.gtb.adjusted">Adjusted basis = $50K + $24,862 = $74,862</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1015.h2.comparison">Compare § 1014 (death) vs § 1015 (gift) vs § 1041 (spouse)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s1015.th.method">Transfer type</th>
                    <th data-i18n="view.s1015.th.basis">Basis to recipient</th>
                    <th data-i18n="view.s1015.th.holding">Holding period</th>
                    <th data-i18n="view.s1015.th.gain_tax">Gain tax on transfer</th>
                </tr></thead>
                <tbody>
                    <tr><td>§ 1014 Death</td><td>FMV at death (FULL step-up)</td><td>LTCG (always)</td><td>No gain tax (§ 1014)</td></tr>
                    <tr><td>§ 1015 Gift inter vivos</td><td>Carryover from donor + gift tax adj</td><td>TACKS from donor's</td><td>Gift tax on donor (above annual excl)</td></tr>
                    <tr><td>§ 1041 Spousal gift</td><td>Carryover basis</td><td>TACKS from spouse</td><td>Tax-free (§ 1041)</td></tr>
                    <tr><td>§ 1041 Divorce transfer</td><td>Carryover basis</td><td>TACKS from spouse</td><td>Tax-free if within 1 yr or pursuant to divorce</td></tr>
                    <tr><td>Bequest in trust</td><td>Trust takes carryover from estate</td><td>Varies — § 1014 if at death</td><td>Generally no gain (subject to estate tax)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1015.h2.planning">Estate planning implications</h2>
            <ul class="muted small">
                <li data-i18n="view.s1015.plan.death_better_appreciated">DEATH is BETTER for highly appreciated assets (full step-up vs carryover)</li>
                <li data-i18n="view.s1015.plan.gift_better_depreciated">GIFT is BETTER for depreciating assets (no loss waste at carryover basis)</li>
                <li data-i18n="view.s1015.plan.annual_exclusion">Use annual exclusion ($19K 2025) for gift tax-free transfers</li>
                <li data-i18n="view.s1015.plan.lifetime_exemption">Lifetime exemption $13.99M 2025 (gift + estate combined)</li>
                <li data-i18n="view.s1015.plan.gst">GST tax: separate exemption for generation-skipping transfers</li>
                <li data-i18n="view.s1015.plan.529">§ 529 plans: special 5-yr forward gift treatment (5× annual exclusion)</li>
                <li data-i18n="view.s1015.plan.med_education">Direct medical / educational payments: NOT counted (no $19K limit)</li>
                <li data-i18n="view.s1015.plan.unified_credit">2026 sunset: unified credit may drop ~$7M unless extended</li>
            </ul>
        </div>
    `;
    document.getElementById('s1015-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.donor_basis = Number(fd.get('donor_basis')) || 0;
        state.fmv_at_gift = Number(fd.get('fmv_at_gift')) || 0;
        state.sale_price = Number(fd.get('sale_price')) || 0;
        state.gift_tax_paid = Number(fd.get('gift_tax_paid')) || 0;
        state.annual_exclusion_used = Number(fd.get('annual_exclusion_used')) || 0;
        state.donor_holding_period_days = Number(fd.get('donor_holding_period_days')) || 0;
        state.is_appreciated_property = !!fd.get('is_appreciated_property');
        state.is_step_up_eligible = !!fd.get('is_step_up_eligible');
        state.is_post_death_gift = !!fd.get('is_post_death_gift');
        state.donor_acquired_year = Number(fd.get('donor_acquired_year')) || 0;
        state.annual_exclusion_2025 = Number(fd.get('annual_exclusion_2025')) || 0;
        state.related_party_donor = !!fd.get('related_party_donor');
        state.spousal_gift = !!fd.get('spousal_gift');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1015-output');
    if (!el) return;
    const netAppreciation = Math.max(0, state.fmv_at_gift - state.donor_basis);
    const netGift = Math.max(0, state.fmv_at_gift - state.annual_exclusion_used);
    const giftTaxAdj = netGift > 0 ? state.gift_tax_paid * (netAppreciation / netGift) : 0;
    const adjustedGainBasis = Math.min(state.donor_basis + giftTaxAdj, state.fmv_at_gift);
    const dualBasis = state.fmv_at_gift < state.donor_basis;
    const lossBasis = state.fmv_at_gift;
    let recognized = 0;
    let character = 'none';
    if (state.sale_price > adjustedGainBasis) {
        recognized = state.sale_price - adjustedGainBasis;
        character = 'gain';
    } else if (state.sale_price < lossBasis && dualBasis) {
        recognized = state.sale_price - lossBasis;
        character = 'loss';
    } else {
        character = 'none';
    }
    const taxAtLTCG = recognized > 0 ? recognized * 0.20 : recognized * 0.37;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1015.h2.result">§ 1015 computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1015.card.donor_basis">Donor's basis</div>
                    <div class="value">$${state.donor_basis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1015.card.gift_tax_adj">Gift tax adjustment</div>
                    <div class="value">$${giftTaxAdj.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1015.card.adjusted_gain">Adjusted gain basis</div>
                    <div class="value">$${adjustedGainBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${dualBasis ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1015.card.dual">Dual basis (loss zone)?</div>
                    <div class="value">${dualBasis ? esc(t('view.s1015.status.yes')) : esc(t('view.s1015.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1015.card.loss_basis">Loss basis (if dual)</div>
                    <div class="value">$${lossBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${character === 'gain' ? 'neg' : character === 'loss' ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s1015.card.recognized">Recognized (sale)</div>
                    <div class="value">$${recognized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1015.card.character">Character</div>
                    <div class="value">${esc(t('view.s1015.char.' + character))}</div>
                </div>
                <div class="card ${recognized > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1015.card.tax">Tax estimate</div>
                    <div class="value">$${taxAtLTCG.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${dualBasis && character === 'none' ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s1015.no_gain_loss_note">
                    Sale falls in "no-gain / no-loss zone" between FMV at gift and donor's basis.
                    No gain or loss recognized — donor's pre-gift unrealized loss is PERMANENTLY LOST.
                    Plan accordingly: hold or sell at lower price for tax loss benefit.
                </p>
            ` : ''}
        </div>
    `;
}
