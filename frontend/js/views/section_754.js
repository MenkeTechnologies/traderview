// IRC § 754 — Partnership Inside Basis Step-Up Election.
// Triggers § 743(b) adjustment on transfer of partnership interest (sale, death).
// Triggers § 734(b) adjustment on partnership distribution causing inside/outside disparity.
// One-time irrevocable election (revocable only with IRS consent).
// Mandatory § 743(b) adjustment if "substantial built-in loss" (> $250K disparity post-2017 TCJA).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    purchase_price: 0,
    inside_basis_proportionate: 0,
    section_754_election: false,
    transfer_type: 'purchase',
    substantial_built_in_loss: false,
    distributed_property_basis: 0,
    distributee_outside_basis: 0,
    gain_recognized_distribution: 0,
    is_real_estate: false,
    years_remaining_depreciation: 15,
};

export async function renderSection754(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s754.h1.title">// § 754 PARTNERSHIP STEP-UP</span></h1>
        <p class="muted small" data-i18n="view.s754.hint.intro">
            <strong>§ 743(b)</strong> adjustment on TRANSFER of partnership interest (sale, death). Steps up
            (or down) the buyer's INSIDE basis in partnership assets to match OUTSIDE basis paid. <strong>§
            734(b)</strong> adjustment on partnership DISTRIBUTION when distributee's outside basis differs
            from inside basis of distributed property. <strong>§ 754 election</strong> = one-time irrevocable
            (rev. w/ IRS consent). <strong>Mandatory § 743(b)</strong> if substantial built-in loss
            (&gt; $250K). Partnership-level election affects ALL transfers in / out.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s754.h2.inputs">Inputs</h2>
            <form id="s754-form" class="inline-form">
                <label><span data-i18n="view.s754.label.purchase">Purchase price (outside basis) ($)</span>
                    <input type="number" step="0.01" name="purchase_price" value="${state.purchase_price}"></label>
                <label><span data-i18n="view.s754.label.inside">Proportionate inside basis ($)</span>
                    <input type="number" step="0.01" name="inside_basis_proportionate" value="${state.inside_basis_proportionate}"></label>
                <label><span data-i18n="view.s754.label.election">§ 754 election in effect?</span>
                    <input type="checkbox" name="section_754_election" ${state.section_754_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s754.label.type">Transfer type</span>
                    <select name="transfer_type">
                        <option value="purchase" ${state.transfer_type === 'purchase' ? 'selected' : ''}>Purchase (§ 743(b))</option>
                        <option value="death" ${state.transfer_type === 'death' ? 'selected' : ''}>Death (§ 1014 + § 743(b))</option>
                        <option value="gift" ${state.transfer_type === 'gift' ? 'selected' : ''}>Gift (no step-up)</option>
                        <option value="distribution_liquidating" ${state.transfer_type === 'distribution_liquidating' ? 'selected' : ''}>Liquidating distribution (§ 734(b))</option>
                        <option value="distribution_non_liquidating" ${state.transfer_type === 'distribution_non_liquidating' ? 'selected' : ''}>Non-liquidating distribution (§ 734(b))</option>
                    </select>
                </label>
                <label><span data-i18n="view.s754.label.sbil">Substantial built-in loss (> $250K)?</span>
                    <input type="checkbox" name="substantial_built_in_loss" ${state.substantial_built_in_loss ? 'checked' : ''}></label>
                <label><span data-i18n="view.s754.label.dist_prop">Distributed property basis ($)</span>
                    <input type="number" step="0.01" name="distributed_property_basis" value="${state.distributed_property_basis}"></label>
                <label><span data-i18n="view.s754.label.outside">Distributee outside basis ($)</span>
                    <input type="number" step="0.01" name="distributee_outside_basis" value="${state.distributee_outside_basis}"></label>
                <label><span data-i18n="view.s754.label.gain_dist">Gain recognized on distribution ($)</span>
                    <input type="number" step="0.01" name="gain_recognized_distribution" value="${state.gain_recognized_distribution}"></label>
                <label><span data-i18n="view.s754.label.real">Real estate assets?</span>
                    <input type="checkbox" name="is_real_estate" ${state.is_real_estate ? 'checked' : ''}></label>
                <label><span data-i18n="view.s754.label.years">Years remaining depreciation</span>
                    <input type="number" step="1" name="years_remaining_depreciation" value="${state.years_remaining_depreciation}"></label>
                <button class="primary" type="submit" data-i18n="view.s754.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s754-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s754.h2.allocation">§ 755 Allocation of § 743(b) adjustment</h2>
            <ol class="muted small">
                <li data-i18n="view.s754.alloc.classes">Split partnership assets into "capital gain" and "ordinary income" classes</li>
                <li data-i18n="view.s754.alloc.ordinary_first">Ordinary class allocated first to extent of unrealized OI; remainder to capital class</li>
                <li data-i18n="view.s754.alloc.within_class">Within class: by relative FMV of assets</li>
                <li data-i18n="view.s754.alloc.real_estate">Real estate: stepped-up basis becomes new depreciation schedule for buyer</li>
                <li data-i18n="view.s754.alloc.depreciable">Adjustment recovered over remaining life of asset</li>
                <li data-i18n="view.s754.alloc.intangibles">Intangibles: § 197 15-year amortization on adjustment</li>
                <li data-i18n="view.s754.alloc.no_increase">No increase in basis of partnership cash / receivables (already at FMV)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s754.h2.compare">Election timing comparison</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s754.th.scenario">Scenario</th>
                    <th data-i18n="view.s754.th.with_election">With § 754</th>
                    <th data-i18n="view.s754.th.without">Without § 754</th>
                </tr></thead>
                <tbody>
                    <tr><td>Sale of interest at premium</td><td>Buyer steps up inside basis → bigger depreciation</td><td>Inside basis stays low → buyer pays tax on phantom gain</td></tr>
                    <tr><td>Sale at discount + SBIL > $250K</td><td>MANDATORY step-down to prevent loss shifting</td><td>Step-down still required (TCJA 2017)</td></tr>
                    <tr><td>Death of partner</td><td>§ 743(b) + § 1014 outside step-up → inside also stepped up</td><td>Outside stepped up but inside unchanged → eventual mismatch</td></tr>
                    <tr><td>Distribution + gain recog</td><td>§ 734(b) adjustment retains partnership inside basis</td><td>Lost basis — never recovered</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s754.h2.tradeoffs">Election tradeoffs</h2>
            <ul class="muted small">
                <li data-i18n="view.s754.trade.pro_step_up">PRO: avoid phantom income for buyer; align inside / outside basis after transfers</li>
                <li data-i18n="view.s754.trade.con_admin">CON: administrative burden (track adjustment per partner per asset)</li>
                <li data-i18n="view.s754.trade.con_step_down">CON: mandatory step-down on loss transfers; cannot opt out post-2017</li>
                <li data-i18n="view.s754.trade.permanent">PERMANENT: irrevocable except with IRS consent</li>
                <li data-i18n="view.s754.trade.real_estate">Real estate partnerships: § 754 nearly universal because step-up offsets depreciation recapture</li>
                <li data-i18n="view.s754.trade.future_transfers">Affects FUTURE transfers as well — both sales + deaths thereafter</li>
                <li data-i18n="view.s754.trade.ptp">Publicly traded partnership: § 754 elections nearly always in effect</li>
            </ul>
        </div>
    `;
    document.getElementById('s754-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.purchase_price = Number(fd.get('purchase_price')) || 0;
        state.inside_basis_proportionate = Number(fd.get('inside_basis_proportionate')) || 0;
        state.section_754_election = !!fd.get('section_754_election');
        state.transfer_type = fd.get('transfer_type');
        state.substantial_built_in_loss = !!fd.get('substantial_built_in_loss');
        state.distributed_property_basis = Number(fd.get('distributed_property_basis')) || 0;
        state.distributee_outside_basis = Number(fd.get('distributee_outside_basis')) || 0;
        state.gain_recognized_distribution = Number(fd.get('gain_recognized_distribution')) || 0;
        state.is_real_estate = !!fd.get('is_real_estate');
        state.years_remaining_depreciation = Number(fd.get('years_remaining_depreciation')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s754-output');
    if (!el) return;
    const isDistribution = state.transfer_type.startsWith('distribution');
    const adjustment = isDistribution ?
        (state.gain_recognized_distribution + state.distributee_outside_basis - state.distributed_property_basis) :
        (state.purchase_price - state.inside_basis_proportionate);
    const mandatory = !isDistribution && state.substantial_built_in_loss;
    const electionApplies = state.section_754_election || mandatory;
    const adjustmentApplied = electionApplies ? adjustment : 0;
    const annualDepreciationBenefit = adjustmentApplied > 0 && state.years_remaining_depreciation > 0 ?
        adjustmentApplied / state.years_remaining_depreciation : 0;
    const taxSavingsAnnual = annualDepreciationBenefit * 0.37;
    const npv = state.years_remaining_depreciation > 0 ?
        taxSavingsAnnual * (1 - Math.pow(1 + 0.05, -state.years_remaining_depreciation)) / 0.05 : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s754.h2.result">§ 754 / § 743(b) / § 734(b) computation</h2>
            <div class="cards">
                <div class="card ${electionApplies ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s754.card.election_applies">Election / mandatory?</div>
                    <div class="value">${electionApplies ? esc(t('view.s754.status.yes')) : esc(t('view.s754.status.no'))}</div>
                </div>
                <div class="card ${mandatory ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s754.card.mandatory">Mandatory (SBIL)?</div>
                    <div class="value">${mandatory ? esc(t('view.s754.status.yes')) : esc(t('view.s754.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s754.card.adjustment">${isDistribution ? esc(t('view.s754.label.s734')) : esc(t('view.s754.label.s743'))}</div>
                    <div class="value">$${adjustment.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${adjustmentApplied > 0 ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s754.card.applied">Applied basis adjustment</div>
                    <div class="value">$${adjustmentApplied.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s754.card.depr_benefit">Annual depreciation benefit</div>
                    <div class="value">$${annualDepreciationBenefit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s754.card.tax_savings_annual">Annual tax savings (37%)</div>
                    <div class="value">$${taxSavingsAnnual.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s754.card.npv">NPV of step-up benefit (5%)</div>
                    <div class="value">$${npv.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
