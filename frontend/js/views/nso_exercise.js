// NSO (Non-Qualified Stock Option) Exercise Calculator.
// Bargain element (FMV - strike) = ORDINARY income at exercise, reported on
// W-2 box 1, plus FICA + Medicare. Federal supplemental W/H: 22% (or 37%
// over $1M YTD). Subsequent sale: gain = sale - FMV at exercise, ST or LT
// depending on hold after exercise.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const SUPPLEMENTAL_W_H_22 = 0.22;
const SUPPLEMENTAL_W_H_37 = 0.37;
const SUPPLEMENTAL_THRESHOLD = 1_000_000;
const SS_BASE = 168_600;
const SS_RATE = 0.062;
const MEDICARE_RATE = 0.0145;
const MEDICARE_ADDL_THRESHOLD = 200_000;  // single
const MEDICARE_ADDL_RATE = 0.009;

let state = {
    exercise_date: new Date().toISOString().slice(0, 10),
    sale_date: '',
    strike_price: 10,
    fmv_at_exercise: 50,
    sale_price: 0,
    shares: 5_000,
    ytd_w2_wages: 200_000,
    marginal_rate: 0.32,
    lt_cap_gains_rate: 0.20,
    state_rate: 0.05,
    niit: 0.038,
};

export async function renderNsoExercise(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.nso.h1.title">// NSO EXERCISE CALCULATOR</span></h1>
        <p class="muted small" data-i18n="view.nso.hint.intro">
            Non-Qualified Stock Options. <strong>Bargain element (FMV − strike)
            = ordinary income at exercise</strong>, reported on W-2 box 1, plus FICA +
            Medicare + state. Federal supplemental W/H: 22% (37% over $1M). At sale:
            gain = sale price − FMV at exercise, ST or LT depending on hold-after-exercise.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.nso.h2.inputs">Inputs</h2>
            <form id="nso-form" class="inline-form">
                <label><span data-i18n="view.nso.label.exercise_date">Exercise date</span>
                    <input type="date" name="exercise_date" value="${state.exercise_date}"></label>
                <label><span data-i18n="view.nso.label.sale_date">Sale date (empty if not yet)</span>
                    <input type="date" name="sale_date" value="${state.sale_date}"></label>
                <label><span data-i18n="view.nso.label.strike_price">Strike price ($/share)</span>
                    <input type="number" step="0.01" name="strike_price" value="${state.strike_price}"></label>
                <label><span data-i18n="view.nso.label.fmv_at_exercise">FMV at exercise ($/share)</span>
                    <input type="number" step="0.01" name="fmv_at_exercise" value="${state.fmv_at_exercise}"></label>
                <label><span data-i18n="view.nso.label.sale_price">Sale price ($/share, 0 if not sold)</span>
                    <input type="number" step="0.01" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.nso.label.shares">Shares</span>
                    <input type="number" step="1" name="shares" value="${state.shares}"></label>
                <label><span data-i18n="view.nso.label.ytd_w2_wages">YTD W-2 wages BEFORE this exercise ($)</span>
                    <input type="number" step="1000" name="ytd_w2_wages" value="${state.ytd_w2_wages}"></label>
                <label><span data-i18n="view.nso.label.marginal_rate">Marginal federal %</span>
                    <input type="number" step="0.5" name="marginal_rate" value="${(state.marginal_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.nso.label.lt_cap_gains_rate">LT cap-gains %</span>
                    <input type="number" step="0.5" name="lt_cap_gains_rate" value="${(state.lt_cap_gains_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.nso.label.state_rate">State %</span>
                    <input type="number" step="0.5" name="state_rate" value="${(state.state_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.nso.btn.compute">Compute</button>
            </form>
        </div>
        <div id="nso-output"></div>
    `;
    document.getElementById('nso-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.exercise_date = fd.get('exercise_date');
        state.sale_date = fd.get('sale_date');
        state.strike_price = Number(fd.get('strike_price'));
        state.fmv_at_exercise = Number(fd.get('fmv_at_exercise'));
        state.sale_price = Number(fd.get('sale_price'));
        state.shares = Number(fd.get('shares'));
        state.ytd_w2_wages = Number(fd.get('ytd_w2_wages')) || 0;
        state.marginal_rate = (Number(fd.get('marginal_rate')) || 32) / 100;
        state.lt_cap_gains_rate = (Number(fd.get('lt_cap_gains_rate')) || 20) / 100;
        state.state_rate = (Number(fd.get('state_rate')) || 0) / 100;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('nso-output');
    if (!el) return;
    const bargainPerShare = state.fmv_at_exercise - state.strike_price;
    const totalBargain = bargainPerShare * state.shares;
    const exerciseCost = state.strike_price * state.shares;

    // Federal supplemental W/H
    const fedSupplementalRate = state.ytd_w2_wages + totalBargain > SUPPLEMENTAL_THRESHOLD
        ? SUPPLEMENTAL_W_H_37 : SUPPLEMENTAL_W_H_22;
    const fedSupplementalWh = totalBargain * fedSupplementalRate;

    // Actual federal tax owed at marginal (may differ from W/H)
    const actualFedTax = totalBargain * state.marginal_rate;
    const fedShortfall = Math.max(0, actualFedTax - fedSupplementalWh);

    // FICA: SS up to $168.6k base, Medicare unlimited, additional 0.9% > $200k
    const remainingSsBase = Math.max(0, SS_BASE - state.ytd_w2_wages);
    const ssTaxableBargain = Math.min(totalBargain, remainingSsBase);
    const ssTax = ssTaxableBargain * SS_RATE;
    const medicareTax = totalBargain * MEDICARE_RATE;
    const addlMedicareBargain = Math.max(0, (state.ytd_w2_wages + totalBargain) - MEDICARE_ADDL_THRESHOLD)
        - Math.max(0, state.ytd_w2_wages - MEDICARE_ADDL_THRESHOLD);
    const addlMedicare = Math.max(0, addlMedicareBargain) * MEDICARE_ADDL_RATE;
    const totalFica = ssTax + medicareTax + addlMedicare;

    const stateTax = totalBargain * state.state_rate;
    const totalTaxAtExercise = actualFedTax + totalFica + stateTax;
    const netCashFromExercise = totalBargain - totalTaxAtExercise;

    // Sale tax
    const sale = state.sale_date ? new Date(state.sale_date) : null;
    const exercise = new Date(state.exercise_date);
    let saleGain = 0, saleTax = 0, isLT = false;
    if (sale && state.sale_price > 0) {
        const yrs = (sale - exercise) / (365.25 * 86_400_000);
        isLT = yrs >= 1;
        saleGain = (state.sale_price - state.fmv_at_exercise) * state.shares;
        const rate = isLT ? (state.lt_cap_gains_rate + state.niit) : state.marginal_rate;
        saleTax = saleGain > 0 ? saleGain * rate : 0;
    }

    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.nso.h2.at_exercise">At exercise</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.nso.card.exercise_cost">Exercise cost</div>
                    <div class="value">$${exerciseCost.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.nso.card.bargain">Ordinary income (bargain)</div>
                    <div class="value">$${totalBargain.toLocaleString()}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.nso.card.fed_w_h">Federal supplemental W/H ${(fedSupplementalRate * 100).toFixed(0)}%</div>
                    <div class="value">$${fedSupplementalWh.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.nso.card.actual_fed_tax">Actual federal tax @ ${(state.marginal_rate * 100).toFixed(0)}%</div>
                    <div class="value">$${actualFedTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${fedShortfall > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.nso.card.fed_shortfall">Federal shortfall vs W/H</div>
                    <div class="value">$${fedShortfall.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.nso.card.fica">FICA (SS + Medicare + addl)</div>
                    <div class="value">$${totalFica.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.nso.card.state_tax">State tax</div>
                    <div class="value">$${stateTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.nso.card.total_tax">Total tax at exercise</div>
                    <div class="value">$${totalTaxAtExercise.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.nso.card.net">Net after tax</div>
                    <div class="value">$${netCashFromExercise.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
        ${sale ? `
            <div class="chart-panel ${isLT ? 'pos' : ''}">
                <h2 data-i18n="view.nso.h2.at_sale">At sale</h2>
                <div class="cards">
                    <div class="card ${isLT ? 'pos' : 'neg'}">
                        <div class="label" data-i18n="view.nso.card.hold_period">Hold period</div>
                        <div class="value">${isLT ? esc(t('view.nso.status.lt')) : esc(t('view.nso.status.st'))}</div>
                    </div>
                    <div class="card">
                        <div class="label" data-i18n="view.nso.card.cap_gain">Capital gain</div>
                        <div class="value">$${saleGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                    <div class="card neg">
                        <div class="label" data-i18n="view.nso.card.sale_tax">Sale tax</div>
                        <div class="value">$${saleTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                </div>
            </div>
        ` : ''}
        <div class="chart-panel">
            <h2 data-i18n="view.nso.h2.tips">Tips</h2>
            <ul class="muted small">
                <li data-i18n="view.nso.tip.w_h_short">22% supplemental W/H is OFTEN insufficient if you're in 32%+ bracket — save cash for April</li>
                <li data-i18n="view.nso.tip.year_end_timing">Exercise late December to defer FICA cost into next year's SS base (if not already capped)</li>
                <li data-i18n="view.nso.tip.same_day">Same-day exercise & sell = no cap-gains exposure (sale price ≈ FMV)</li>
                <li data-i18n="view.nso.tip.cashless">Cashless exercise: company sells enough shares to cover exercise cost + taxes — no out-of-pocket</li>
                <li data-i18n="view.nso.tip.spread_out">Spread exercise across years to avoid bracket-jump in high-income year</li>
            </ul>
        </div>
    `;
}
