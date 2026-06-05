// IRC § 25E — Used Clean Vehicle Credit (IRA 2022).
// 30% of sale price OR $4,000 — whichever LOWER.
// Sale price ≤ $25,000. Must be at least 2 model years older than current.
// MAGI limit: $75K single / $112.5K HoH / $150K MFJ.
// Buyer can transfer credit to dealer at point of sale (similar to § 30D).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    sale_price: 0,
    model_year: 0,
    current_year: 2025,
    is_first_resale: true,
    is_qualified_dealer: true,
    magi: 0,
    filing_status: 'single',
    purchase_year: 2025,
    transfer_to_dealer: false,
    used_before_by_taxpayer: false,
    used_phev_or_bev: 'bev',
    battery_kwh: 0,
};

export async function renderSection25E(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s25E.h1.title">// § 25E USED CLEAN VEHICLE</span></h1>
        <p class="muted small" data-i18n="view.s25E.hint.intro">
            <strong>30% of sale price</strong> OR <strong>$4,000</strong> — LOWER applies. <strong>Sale price
            ≤ $25,000.</strong> Vehicle must be at least <strong>2 model years</strong> older than current.
            <strong>MAGI limit:</strong> $75K single / $112.5K HoH / $150K MFJ. <strong>First resale only</strong>
            (not flippers). <strong>Qualified dealer required</strong> (registered with IRS Energy Credits
            Online). <strong>Buyer transfers credit to dealer</strong> at POS. <strong>One-per-three-year limit</strong>
            for buyer. <strong>Form 8936.</strong> <strong>NO domestic assembly / minerals / battery components</strong>
            requirements — unlike § 30D.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s25E.h2.inputs">Inputs</h2>
            <form id="s25E-form" class="inline-form">
                <label><span data-i18n="view.s25E.label.price">Sale price ($)</span>
                    <input type="number" step="0.01" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.s25E.label.model_year">Vehicle model year</span>
                    <input type="number" step="1" name="model_year" value="${state.model_year}"></label>
                <label><span data-i18n="view.s25E.label.current">Current year</span>
                    <input type="number" step="1" name="current_year" value="${state.current_year}"></label>
                <label><span data-i18n="view.s25E.label.first_resale">First resale only?</span>
                    <input type="checkbox" name="is_first_resale" ${state.is_first_resale ? 'checked' : ''}></label>
                <label><span data-i18n="view.s25E.label.dealer">Qualified dealer (IRS-registered)?</span>
                    <input type="checkbox" name="is_qualified_dealer" ${state.is_qualified_dealer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s25E.label.magi">MAGI ($)</span>
                    <input type="number" step="0.01" name="magi" value="${state.magi}"></label>
                <label><span data-i18n="view.s25E.label.filing">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single / MFS ($75K)</option>
                        <option value="hoh" ${state.filing_status === 'hoh' ? 'selected' : ''}>HoH ($112.5K)</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ ($150K)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s25E.label.year">Purchase year</span>
                    <input type="number" step="1" name="purchase_year" value="${state.purchase_year}"></label>
                <label><span data-i18n="view.s25E.label.transfer">Transfer to dealer (POS)?</span>
                    <input type="checkbox" name="transfer_to_dealer" ${state.transfer_to_dealer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s25E.label.before">Used by taxpayer before?</span>
                    <input type="checkbox" name="used_before_by_taxpayer" ${state.used_before_by_taxpayer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s25E.label.bev_phev">BEV / FCV / PHEV</span>
                    <select name="used_phev_or_bev">
                        <option value="bev" ${state.used_phev_or_bev === 'bev' ? 'selected' : ''}>BEV</option>
                        <option value="fcv" ${state.used_phev_or_bev === 'fcv' ? 'selected' : ''}>FCV (fuel cell)</option>
                        <option value="phev" ${state.used_phev_or_bev === 'phev' ? 'selected' : ''}>PHEV</option>
                    </select>
                </label>
                <label><span data-i18n="view.s25E.label.kwh">Battery kWh capacity</span>
                    <input type="number" step="0.1" name="battery_kwh" value="${state.battery_kwh}"></label>
                <button class="primary" type="submit" data-i18n="view.s25E.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s25E-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s25E.h2.eligibility">Eligibility cascade</h2>
            <ul class="muted small">
                <li data-i18n="view.s25E.elig.price">Sale price ≤ $25,000 (includes options, dealer prep, document fees)</li>
                <li data-i18n="view.s25E.elig.age">Model year at least 2 years older than current year</li>
                <li data-i18n="view.s25E.elig.first_resale">FIRST resale after original sale (not flippers / repeat sellers)</li>
                <li data-i18n="view.s25E.elig.dealer">Sold by IRS-registered dealer (NOT private party)</li>
                <li data-i18n="view.s25E.elig.battery_phev">PHEV: battery ≥ 7 kWh; BEV: no battery min stated; FCV: any</li>
                <li data-i18n="view.s25E.elig.no_personal_use_before">Not used by purchasing taxpayer before</li>
                <li data-i18n="view.s25E.elig.frequency">One-per-three-year limit on buyer</li>
                <li data-i18n="view.s25E.elig.income">MAGI: $75K / $112.5K / $150K (CURRENT or PRIOR yr — pick lower)</li>
                <li data-i18n="view.s25E.elig.no_domestic">NO domestic assembly / minerals / battery components requirement</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s25E.h2.transfer">Transfer at dealer (§ 25E(d))</h2>
            <ul class="muted small">
                <li data-i18n="view.s25E.tx.discount">Discount applied at POS — dealer registers via IRS Energy Credits Online</li>
                <li data-i18n="view.s25E.tx.dealer_advance">Dealer receives advance payment from IRS (Form 8300)</li>
                <li data-i18n="view.s25E.tx.refundable_effect">Refundable effect: even if no tax liability, buyer gets discount</li>
                <li data-i18n="view.s25E.tx.repay_excess">Buyer repays on next return if MAGI exceeds limit</li>
                <li data-i18n="view.s25E.tx.dealer_certificates">Dealer issues Time-of-Sale Report (Form 15400)</li>
                <li data-i18n="view.s25E.tx.buyer_keeps_records">Buyer keeps: sale invoice + 15400 + 8936</li>
                <li data-i18n="view.s25E.tx.no_clawback_safe_harbor">No clawback if buyer not aware of excess MAGI</li>
                <li data-i18n="view.s25E.tx.dealer_3rd_party">3rd-party finance ok; dealer must process credit transfer</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s25E.h2.examples">Real-world examples (post-IRA 2022)</h2>
            <ul class="muted small">
                <li data-i18n="view.s25E.ex.tesla">Used Tesla Model 3 (2022, $24K): credit $4,000</li>
                <li data-i18n="view.s25E.ex.leaf">Used Nissan Leaf (2020, $14K): credit min($4,000, 30% × 14K = $4,200) = $4,000</li>
                <li data-i18n="view.s25E.ex.bolt">Used Chevy Bolt (2021, $20K): credit min($4,000, 30% × 20K = $6,000) = $4,000</li>
                <li data-i18n="view.s25E.ex.prime">Used Prius Prime PHEV (2020, $18K): credit min($4,000, $5,400) = $4,000</li>
                <li data-i18n="view.s25E.ex.kia_niro">Used Kia Niro EV (2020, $19K): credit $4,000</li>
                <li data-i18n="view.s25E.ex.ford_lightning">Used Ford Lightning (2022, $30K): NO CREDIT — over $25K cap</li>
                <li data-i18n="view.s25E.ex.mach_e">Used Mach-E (2022, $26K): NO CREDIT — over $25K cap</li>
                <li data-i18n="view.s25E.ex.private_party">Private party sale (no dealer): NO CREDIT — must be qualified dealer</li>
            </ul>
        </div>
    `;
    document.getElementById('s25E-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.sale_price = Number(fd.get('sale_price')) || 0;
        state.model_year = Number(fd.get('model_year')) || 0;
        state.current_year = Number(fd.get('current_year')) || 0;
        state.is_first_resale = !!fd.get('is_first_resale');
        state.is_qualified_dealer = !!fd.get('is_qualified_dealer');
        state.magi = Number(fd.get('magi')) || 0;
        state.filing_status = fd.get('filing_status');
        state.purchase_year = Number(fd.get('purchase_year')) || 0;
        state.transfer_to_dealer = !!fd.get('transfer_to_dealer');
        state.used_before_by_taxpayer = !!fd.get('used_before_by_taxpayer');
        state.used_phev_or_bev = fd.get('used_phev_or_bev');
        state.battery_kwh = Number(fd.get('battery_kwh')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s25E-output');
    if (!el) return;
    const ageMet = (state.current_year - state.model_year) >= 2;
    const priceMet = state.sale_price <= 25_000;
    const magiLimit = state.filing_status === 'mfj' ? 150_000 : state.filing_status === 'hoh' ? 112_500 : 75_000;
    const magiMet = state.magi <= magiLimit;
    const batteryMet = state.used_phev_or_bev === 'phev' ? state.battery_kwh >= 7 : true;
    const eligible = ageMet && priceMet && magiMet && state.is_first_resale && state.is_qualified_dealer && batteryMet && !state.used_before_by_taxpayer;
    const credit = eligible ? Math.min(4_000, state.sale_price * 0.30) : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s25E.h2.result">§ 25E credit computation</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s25E.card.eligible">Eligible?</div>
                    <div class="value">${eligible ? esc(t('view.s25E.status.yes')) : esc(t('view.s25E.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s25E.card.age">2+ yr age met?</div>
                    <div class="value">${ageMet ? esc(t('view.s25E.status.yes')) : esc(t('view.s25E.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s25E.card.price">Price ≤ $25K met?</div>
                    <div class="value">${priceMet ? esc(t('view.s25E.status.yes')) : esc(t('view.s25E.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s25E.card.magi_limit">MAGI limit</div>
                    <div class="value">$${magiLimit.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s25E.card.credit">§ 25E credit</div>
                    <div class="value">$${credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s25E.card.eff_price">Effective price</div>
                    <div class="value">$${(state.sale_price - credit).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!eligible && (state.sale_price > 25_000) ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s25E.over_price_note">
                    Sale price exceeds $25,000 cap. Negotiate price down or look at lower-priced inventory.
                    For higher-priced new vehicles, consider § 30D (new) or § 45W (commercial / lease loophole)
                    instead. § 25E is targeted at affordability for used EV market.
                </p>
            ` : ''}
        </div>
    `;
}
