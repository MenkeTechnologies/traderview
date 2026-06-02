// ESPP § 423 Calculator.
// Qualifying disposition: 2 yrs from offering date + 1 yr from purchase date.
//   - Discount (% × FMV-at-offering) = ordinary income, capped at total gain
//   - Remainder = LT cap-gain
// Disqualifying disposition (sold early):
//   - Discount × FMV-at-purchase = ordinary income (W-2 adj)
//   - Remainder = ST or LT cap-gain by hold-after-purchase
// $25,000/yr per-employee cap on stock value (at offering price).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    offering_date: '2023-01-01',
    purchase_date: '2023-06-30',
    sale_date: new Date().toISOString().slice(0, 10),
    fmv_at_offering: 100,
    fmv_at_purchase: 120,
    purchase_price_paid: 85,  // 15% discount off lower of offer/purchase
    sale_price: 200,
    shares: 100,
    your_marginal_rate: 0.32,
    lt_cap_gains_rate: 0.20,
    niit: 0.038,
};

export async function renderEsppCalc(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.espp.h1.title">// ESPP § 423 CALCULATOR</span></h1>
        <p class="muted small" data-i18n="view.espp.hint.intro">
            <strong>Qualifying disposition:</strong> 2 yrs from offering date + 1 yr from
            purchase date. Discount × FMV-at-offering = ordinary income (capped at gain),
            rest = LT cap-gain. <strong>Disqualifying:</strong> discount × FMV-at-purchase
            = ordinary, rest = ST/LT by hold. $25k/yr employee cap (offering price).
            15% discount + lookback = guaranteed ~17.6%+ return at purchase.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.espp.h2.inputs">Inputs</h2>
            <form id="espp-form" class="inline-form">
                <label><span data-i18n="view.espp.label.offering_date">Offering date</span>
                    <input type="date" name="offering_date" value="${state.offering_date}"></label>
                <label><span data-i18n="view.espp.label.purchase_date">Purchase date</span>
                    <input type="date" name="purchase_date" value="${state.purchase_date}"></label>
                <label><span data-i18n="view.espp.label.sale_date">Sale date</span>
                    <input type="date" name="sale_date" value="${state.sale_date}"></label>
                <label><span data-i18n="view.espp.label.fmv_at_offering">FMV at offering ($)</span>
                    <input type="number" step="0.01" name="fmv_at_offering" value="${state.fmv_at_offering}"></label>
                <label><span data-i18n="view.espp.label.fmv_at_purchase">FMV at purchase ($)</span>
                    <input type="number" step="0.01" name="fmv_at_purchase" value="${state.fmv_at_purchase}"></label>
                <label><span data-i18n="view.espp.label.purchase_price_paid">Purchase price paid ($)</span>
                    <input type="number" step="0.01" name="purchase_price_paid" value="${state.purchase_price_paid}"></label>
                <label><span data-i18n="view.espp.label.sale_price">Sale price ($)</span>
                    <input type="number" step="0.01" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.espp.label.shares">Shares</span>
                    <input type="number" step="1" name="shares" value="${state.shares}"></label>
                <label><span data-i18n="view.espp.label.your_marginal_rate">Marginal ordinary %</span>
                    <input type="number" step="0.5" name="your_marginal_rate" value="${(state.your_marginal_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.espp.label.lt_cap_gains_rate">LT cap-gains %</span>
                    <input type="number" step="0.5" name="lt_cap_gains_rate" value="${(state.lt_cap_gains_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.espp.btn.compute">Compute</button>
            </form>
        </div>
        <div id="espp-output"></div>
    `;
    document.getElementById('espp-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.offering_date = fd.get('offering_date');
        state.purchase_date = fd.get('purchase_date');
        state.sale_date = fd.get('sale_date');
        state.fmv_at_offering = Number(fd.get('fmv_at_offering'));
        state.fmv_at_purchase = Number(fd.get('fmv_at_purchase'));
        state.purchase_price_paid = Number(fd.get('purchase_price_paid'));
        state.sale_price = Number(fd.get('sale_price'));
        state.shares = Number(fd.get('shares'));
        state.your_marginal_rate = (Number(fd.get('your_marginal_rate')) || 32) / 100;
        state.lt_cap_gains_rate = (Number(fd.get('lt_cap_gains_rate')) || 20) / 100;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('espp-output');
    if (!el) return;
    const offering = new Date(state.offering_date);
    const purchase = new Date(state.purchase_date);
    const sale = new Date(state.sale_date);
    const yrsFromOffering = (sale - offering) / (365.25 * 86_400_000);
    const yrsFromPurchase = (sale - purchase) / (365.25 * 86_400_000);
    const qualifying = yrsFromOffering >= 2 && yrsFromPurchase >= 1;

    const totalGainPerShare = state.sale_price - state.purchase_price_paid;
    const totalGain = totalGainPerShare * state.shares;
    const discountPerShare = state.fmv_at_offering - state.purchase_price_paid;  // for qualifying
    const discountPerShareDisq = state.fmv_at_purchase - state.purchase_price_paid;  // for disqualifying

    let ordinary = 0, ltCapGain = 0, stCapGain = 0;
    if (qualifying) {
        ordinary = Math.min(discountPerShare * state.shares, totalGain);
        ltCapGain = totalGain - ordinary;
    } else {
        ordinary = discountPerShareDisq * state.shares;
        const remainder = totalGain - ordinary;
        if (yrsFromPurchase >= 1) ltCapGain = remainder;
        else stCapGain = remainder;
    }
    const tax = ordinary * state.your_marginal_rate
        + Math.max(0, ltCapGain) * (state.lt_cap_gains_rate + state.niit)
        + Math.max(0, stCapGain) * state.your_marginal_rate;
    const netProceeds = state.sale_price * state.shares - tax;
    const totalCost = state.purchase_price_paid * state.shares;
    const netGain = netProceeds - totalCost;

    const cls = qualifying ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel ${cls}">
            <h2 data-i18n="view.espp.h2.disposition">Disposition</h2>
            <div class="cards">
                <div class="card ${cls}">
                    <div class="label" data-i18n="view.espp.card.qualifying">Type</div>
                    <div class="value">${qualifying ? esc(t('view.espp.status.qualifying')) : esc(t('view.espp.status.disqualifying'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.espp.card.yrs_offering">From offering</div>
                    <div class="value">${yrsFromOffering.toFixed(1)} yr</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.espp.card.yrs_purchase">From purchase</div>
                    <div class="value">${yrsFromPurchase.toFixed(1)} yr</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.espp.card.total_gain">Total gain</div>
                    <div class="value">$${totalGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.espp.card.ordinary">Ordinary (W-2)</div>
                    <div class="value">$${ordinary.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.espp.card.lt_gain">LT cap-gain</div>
                    <div class="value">$${ltCapGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.espp.card.st_gain">ST cap-gain</div>
                    <div class="value">$${stCapGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.espp.card.tax">Total tax</div>
                    <div class="value">$${tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.espp.card.net_proceeds">Net proceeds</div>
                    <div class="value">$${netProceeds.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.espp.card.net_gain">Net gain vs cost</div>
                    <div class="value">$${netGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.espp.h2.strategy">Strategy</h2>
            <ul class="muted small">
                <li data-i18n="view.espp.strat.always_enroll">If you can afford it, ALWAYS enroll at max % — 15% discount + lookback = ~17.6% guaranteed minimum return at purchase</li>
                <li data-i18n="view.espp.strat.sell_immediately">Sell-immediately strategy: locks in ~15% discount as ordinary income, no holding risk</li>
                <li data-i18n="view.espp.strat.qualifying_wait">Qualifying-wait: requires concentrated bet on employer stock (correlated with W-2 income)</li>
                <li data-i18n="view.espp.strat.25k_cap">$25k/yr ESPP cap is based on stock at OFFERING price, not purchase</li>
                <li data-i18n="view.espp.strat.broker_basis">CRITICAL: broker basis is often WRONG (basis = purchase price, but ordinary income is reported on W-2). Adjust basis on Form 8949 to avoid double-tax!</li>
            </ul>
        </div>
    `;
}
