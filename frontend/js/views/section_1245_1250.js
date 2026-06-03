// IRC § 1245 + § 1250 — Depreciation Recapture on Sale of Depreciated Property.
// § 1245 (personal property): recapture ALL depreciation as ORDINARY income up to gain.
// § 1250 (real property): only EXCESS depreciation over straight-line is recapture; balance
// = "unrecaptured § 1250 gain" capped at 25%. § 291 corp-only adds 20% of would-be-1245 excess.

import { currentViewToken, viewIsCurrent } from '../app.js';

const UNRECAP_1250_MAX_RATE = 0.25;

let state = {
    is_personal_property: true,
    sale_price: 0,
    original_cost: 0,
    accumulated_depreciation: 0,
    straight_line_would_have: 0,
    is_corp: false,
    marginal_rate: 0.32,
    ltcg_rate: 0.20,
};

export async function renderSection12451250(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1245.h1.title">// § 1245 / § 1250 DEPRECIATION RECAPTURE</span></h1>
        <p class="muted small" data-i18n="view.s1245.hint.intro">
            <strong>§ 1245 personal property:</strong> ALL accumulated depreciation recapture
            as ORDINARY income (up to total gain). <strong>§ 1250 real property:</strong>
            only EXCESS over straight-line recapture (residential rental + most post-1986 real estate
            = $0 since all SL); rest = "<strong>Unrecaptured § 1250 gain</strong>" capped at 25%.
            <strong>§ 291 corp adds 20%</strong> of would-be-§1245 amount. Reported on Form 4797.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1245.h2.inputs">Inputs</h2>
            <form id="s1245-form" class="inline-form">
                <label><span data-i18n="view.s1245.label.personal">Personal property (§ 1245)?</span>
                    <input type="checkbox" name="is_personal_property" ${state.is_personal_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1245.label.sale">Sale price ($)</span>
                    <input type="number" step="100" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.s1245.label.cost">Original cost ($)</span>
                    <input type="number" step="100" name="original_cost" value="${state.original_cost}"></label>
                <label><span data-i18n="view.s1245.label.accum">Accumulated depreciation ($)</span>
                    <input type="number" step="100" name="accumulated_depreciation" value="${state.accumulated_depreciation}"></label>
                <label><span data-i18n="view.s1245.label.sl">SL would-have-been (real property) ($)</span>
                    <input type="number" step="100" name="straight_line_would_have" value="${state.straight_line_would_have}"></label>
                <label><span data-i18n="view.s1245.label.corp">C-corp seller (§ 291)?</span>
                    <input type="checkbox" name="is_corp" ${state.is_corp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1245.label.marginal">Ordinary marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.s1245.label.ltcg">LTCG %</span>
                    <input type="number" step="0.01" name="ltcg_rate" value="${state.ltcg_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s1245.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1245-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1245.h2.examples">§ 1245 vs § 1250 examples</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s1245.th.asset">Asset</th>
                    <th data-i18n="view.s1245.th.section">Section</th>
                    <th data-i18n="view.s1245.th.recapture">Recapture treatment</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s1245.ex.equipment">Office equipment / computers</td><td>§ 1245</td><td>All as ordinary</td></tr>
                    <tr><td data-i18n="view.s1245.ex.vehicles">Vehicles / trucks</td><td>§ 1245</td><td>All as ordinary</td></tr>
                    <tr><td data-i18n="view.s1245.ex.machinery">Machinery + production equipment</td><td>§ 1245</td><td>All as ordinary</td></tr>
                    <tr><td data-i18n="view.s1245.ex.qip">Qualified Improvement Property</td><td>§ 1250 (real, but personal-property recapture per Bonus rules)</td><td>SL → no recapture; bonus may trigger</td></tr>
                    <tr><td data-i18n="view.s1245.ex.residential">Residential rental real estate (post-1986)</td><td>§ 1250 SL</td><td>$0 § 1250 + 25% unrecaptured § 1250</td></tr>
                    <tr><td data-i18n="view.s1245.ex.commercial">Non-residential real (post-1986)</td><td>§ 1250 SL</td><td>$0 § 1250 + 25% unrecaptured § 1250</td></tr>
                    <tr><td data-i18n="view.s1245.ex.pre_1987">Pre-1987 accelerated real estate</td><td>§ 1250 ACRS</td><td>Excess over SL recaptured ordinary</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1245.h2.planning">Planning</h2>
            <ul class="muted small">
                <li data-i18n="view.s1245.plan.1031">§ 1031 like-kind exchange DEFERS recapture (real property only post-TCJA)</li>
                <li data-i18n="view.s1245.plan.installment">§ 453 installment sale: § 1245 recapture all RECOGNIZED in year of sale (no deferral)</li>
                <li data-i18n="view.s1245.plan.charitable">Gift of depreciated property: reduce basis by acc. depr; recipient inherits</li>
                <li data-i18n="view.s1245.plan.death">Step-up at death (§ 1014): recapture WIPED OUT — heir gets fresh basis</li>
                <li data-i18n="view.s1245.plan.cost_seg">Cost segregation: more depreciation now, more § 1245 recapture later — net positive</li>
            </ul>
        </div>
    `;
    document.getElementById('s1245-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_personal_property = !!fd.get('is_personal_property');
        state.sale_price = Number(fd.get('sale_price')) || 0;
        state.original_cost = Number(fd.get('original_cost')) || 0;
        state.accumulated_depreciation = Number(fd.get('accumulated_depreciation')) || 0;
        state.straight_line_would_have = Number(fd.get('straight_line_would_have')) || 0;
        state.is_corp = !!fd.get('is_corp');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        state.ltcg_rate = Number(fd.get('ltcg_rate')) || 0.20;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1245-output');
    if (!el) return;
    const adjustedBasis = state.original_cost - state.accumulated_depreciation;
    const totalGain = state.sale_price - adjustedBasis;
    let s1245Recapture = 0, s1250Recapture = 0, unrecap1250 = 0, sec291Add = 0;
    let remainingLtcg = 0;
    if (totalGain > 0) {
        if (state.is_personal_property) {
            s1245Recapture = Math.min(state.accumulated_depreciation, totalGain);
            remainingLtcg = totalGain - s1245Recapture;
            if (state.is_corp) {
                sec291Add = (state.accumulated_depreciation - state.straight_line_would_have) * 0.20;
            }
        } else {
            const excessDep = Math.max(0, state.accumulated_depreciation - state.straight_line_would_have);
            s1250Recapture = Math.min(excessDep, totalGain);
            unrecap1250 = Math.min(state.accumulated_depreciation, totalGain) - s1250Recapture;
            remainingLtcg = totalGain - s1250Recapture - unrecap1250;
            if (state.is_corp) {
                sec291Add = Math.min(excessDep * 0.20, totalGain - s1250Recapture);
                unrecap1250 -= sec291Add;
            }
        }
    }
    const taxRecapture = (s1245Recapture + s1250Recapture + sec291Add) * state.marginal_rate;
    const taxUnrecap = unrecap1250 * UNRECAP_1250_MAX_RATE;
    const taxLtcg = Math.max(0, remainingLtcg) * state.ltcg_rate;
    const totalTax = taxRecapture + taxUnrecap + taxLtcg;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1245.h2.result">Gain breakdown</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1245.card.basis">Adjusted basis</div>
                    <div class="value">$${adjustedBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1245.card.gain">Total gain</div>
                    <div class="value">$${totalGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${s1245Recapture > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s1245.card.s1245">§ 1245 ordinary recapture</div>
                        <div class="value">$${s1245Recapture.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                ${s1250Recapture > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s1245.card.s1250">§ 1250 ordinary recapture</div>
                        <div class="value">$${s1250Recapture.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                ${unrecap1250 > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s1245.card.unrecap">Unrecaptured § 1250 (25% cap)</div>
                        <div class="value">$${unrecap1250.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                ${sec291Add > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s1245.card.sec291">§ 291 corp 20% add-on</div>
                        <div class="value">$${sec291Add.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card pos">
                    <div class="label" data-i18n="view.s1245.card.ltcg">Remaining LTCG (15/20%)</div>
                    <div class="value">$${Math.max(0, remainingLtcg).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1245.card.total_tax">Total federal tax</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
