// ISO (Incentive Stock Option) Exercise Calculator — IRC § 422.
// Bargain element (FMV - strike) = AMT preference at exercise.
// Holding period for QUALIFYING disposition: 2 yrs from grant + 1 yr from
// exercise. Qualifying = all gain as LT cap-gains. Disqualifying = some
// ordinary income (W-2 reported as compensation).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    grant_date: '2022-01-15',
    exercise_date: new Date().toISOString().slice(0, 10),
    sale_date: '',
    strike_price: 5,
    fmv_at_exercise: 50,
    sale_price: 0,
    shares: 1_000,
    your_marginal_rate: 0.32,
    lt_cap_gains_rate: 0.20,
    niit: 0.038,
    amt_rate: 0.28,
};

export async function renderIsoExercise(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.iso.h1.title">// ISO EXERCISE CALCULATOR</span></h1>
        <p class="muted small" data-i18n="view.iso.hint.intro">
            Incentive Stock Options § 422. <strong>Bargain element (FMV − strike)
            is AMT preference at exercise</strong> (not regular income). Hold 2 yrs from
            grant + 1 yr from exercise = QUALIFYING disposition: all gain as LT cap-gains.
            Disqualifying = some ordinary (W-2 reported). $100k ISO grant cap per year
            (excess treated as NSO).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.iso.h2.inputs">Inputs</h2>
            <form id="iso-form" class="inline-form">
                <label><span data-i18n="view.iso.label.grant_date">Grant date</span>
                    <input type="date" name="grant_date" value="${state.grant_date}"></label>
                <label><span data-i18n="view.iso.label.exercise_date">Exercise date</span>
                    <input type="date" name="exercise_date" value="${state.exercise_date}"></label>
                <label><span data-i18n="view.iso.label.sale_date">Sale date (empty if not yet)</span>
                    <input type="date" name="sale_date" value="${state.sale_date}"></label>
                <label><span data-i18n="view.iso.label.strike_price">Strike price ($/share)</span>
                    <input type="number" step="0.01" name="strike_price" value="${state.strike_price}"></label>
                <label><span data-i18n="view.iso.label.fmv_at_exercise">FMV at exercise ($/share)</span>
                    <input type="number" step="0.01" name="fmv_at_exercise" value="${state.fmv_at_exercise}"></label>
                <label><span data-i18n="view.iso.label.sale_price">Sale price ($/share, 0 if not sold)</span>
                    <input type="number" step="0.01" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.iso.label.shares">Shares</span>
                    <input type="number" step="1" name="shares" value="${state.shares}"></label>
                <label><span data-i18n="view.iso.label.your_marginal_rate">Marginal ordinary %</span>
                    <input type="number" step="0.5" name="your_marginal_rate" value="${(state.your_marginal_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.iso.label.lt_cap_gains_rate">LT cap-gains %</span>
                    <input type="number" step="0.5" name="lt_cap_gains_rate" value="${(state.lt_cap_gains_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.iso.label.amt_rate">AMT marginal %</span>
                    <input type="number" step="0.5" name="amt_rate" value="${(state.amt_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.iso.btn.compute">Compute</button>
            </form>
        </div>
        <div id="iso-output"></div>
    `;
    document.getElementById('iso-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.grant_date = fd.get('grant_date');
        state.exercise_date = fd.get('exercise_date');
        state.sale_date = fd.get('sale_date');
        state.strike_price = Number(fd.get('strike_price'));
        state.fmv_at_exercise = Number(fd.get('fmv_at_exercise'));
        state.sale_price = Number(fd.get('sale_price'));
        state.shares = Number(fd.get('shares'));
        state.your_marginal_rate = (Number(fd.get('your_marginal_rate')) || 32) / 100;
        state.lt_cap_gains_rate = (Number(fd.get('lt_cap_gains_rate')) || 20) / 100;
        state.amt_rate = (Number(fd.get('amt_rate')) || 28) / 100;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('iso-output');
    if (!el) return;
    const grant = new Date(state.grant_date);
    const exercise = new Date(state.exercise_date);
    const sale = state.sale_date ? new Date(state.sale_date) : null;
    const yearsFromGrant = sale ? (sale - grant) / (365.25 * 86_400_000) : null;
    const yearsFromExercise = sale ? (sale - exercise) / (365.25 * 86_400_000) : null;
    const qualifying = sale ? (yearsFromGrant >= 2 && yearsFromExercise >= 1) : null;

    const bargainPerShare = state.fmv_at_exercise - state.strike_price;
    const totalBargain = bargainPerShare * state.shares;
    const exerciseCost = state.strike_price * state.shares;
    const fmvCost = state.fmv_at_exercise * state.shares;

    // AMT at exercise
    const amtPreference = totalBargain;
    const amtOwed = amtPreference * state.amt_rate;

    // Sale calculation
    let saleGain = 0, ordinaryAtSale = 0, ltGainAtSale = 0, stGainAtSale = 0, saleTax = 0;
    if (sale && state.sale_price > 0) {
        const saleProceeds = state.sale_price * state.shares;
        const totalGainFromStrike = saleProceeds - exerciseCost;
        if (qualifying) {
            // All gain LT cap-gains
            ltGainAtSale = totalGainFromStrike;
            saleTax = ltGainAtSale * (state.lt_cap_gains_rate + state.niit);
        } else {
            // Disqualifying: bargain element to ordinary, remainder cap gain
            ordinaryAtSale = Math.min(totalGainFromStrike, totalBargain);
            const remainder = totalGainFromStrike - ordinaryAtSale;
            if (yearsFromExercise >= 1) {
                ltGainAtSale = remainder;
                saleTax = ordinaryAtSale * state.your_marginal_rate + ltGainAtSale * (state.lt_cap_gains_rate + state.niit);
            } else {
                stGainAtSale = remainder;
                saleTax = ordinaryAtSale * state.your_marginal_rate + stGainAtSale * state.your_marginal_rate;
            }
        }
        saleGain = totalGainFromStrike;
    }

    const cls = qualifying ? 'pos' : sale ? 'neg' : '';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.iso.h2.at_exercise">At exercise</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.iso.card.exercise_cost">Exercise cost</div>
                    <div class="value">$${exerciseCost.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.iso.card.bargain">Bargain element</div>
                    <div class="value">$${totalBargain.toLocaleString()}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.iso.card.amt_owed">AMT owed (preference × ${(state.amt_rate * 100).toFixed(0)}%)</div>
                    <div class="value">$${amtOwed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.iso.card.fmv_basis">AMT basis post-exercise</div>
                    <div class="value">$${(state.fmv_at_exercise).toFixed(2)}/sh</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.iso.card.regular_basis">Regular basis post-exercise</div>
                    <div class="value">$${(state.strike_price).toFixed(2)}/sh</div>
                </div>
            </div>
        </div>
        ${sale ? `
            <div class="chart-panel ${cls}">
                <h2 data-i18n="view.iso.h2.at_sale">At sale</h2>
                <div class="cards">
                    <div class="card ${qualifying ? 'pos' : 'neg'}">
                        <div class="label" data-i18n="view.iso.card.qualifying">Disposition type</div>
                        <div class="value">${qualifying ? esc(t('view.iso.status.qualifying')) : esc(t('view.iso.status.disqualifying'))}</div>
                    </div>
                    <div class="card">
                        <div class="label" data-i18n="view.iso.card.years_from_grant">From grant</div>
                        <div class="value">${yearsFromGrant?.toFixed(1)} yr</div>
                    </div>
                    <div class="card">
                        <div class="label" data-i18n="view.iso.card.years_from_exercise">From exercise</div>
                        <div class="value">${yearsFromExercise?.toFixed(1)} yr</div>
                    </div>
                    <div class="card">
                        <div class="label" data-i18n="view.iso.card.total_gain">Total gain</div>
                        <div class="value">$${saleGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                    <div class="card">
                        <div class="label" data-i18n="view.iso.card.ordinary_at_sale">Ordinary income at sale</div>
                        <div class="value">$${ordinaryAtSale.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                    <div class="card">
                        <div class="label" data-i18n="view.iso.card.lt_cap_gain">LT cap-gain</div>
                        <div class="value">$${ltGainAtSale.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                    <div class="card">
                        <div class="label" data-i18n="view.iso.card.st_cap_gain">ST cap-gain</div>
                        <div class="value">$${stGainAtSale.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                    <div class="card neg">
                        <div class="label" data-i18n="view.iso.card.sale_tax">Sale tax</div>
                        <div class="value">$${saleTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                    <div class="card">
                        <div class="label" data-i18n="view.iso.card.mtc_available">MTC recoverable (from AMT)</div>
                        <div class="value">$${amtOwed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                </div>
            </div>
        ` : `<div class="chart-panel"><h2 data-i18n="view.iso.h2.no_sale">Not yet sold</h2>
            <p class="muted small" data-i18n="view.iso.no_sale_note">Add sale date + price above to see disposition treatment.</p></div>`}
        <div class="chart-panel">
            <h2 data-i18n="view.iso.h2.strategy">Strategy</h2>
            <ul class="muted small">
                <li data-i18n="view.iso.strat.early_exercise">Early exercise (just-vested with low FMV) minimizes bargain element → less AMT</li>
                <li data-i18n="view.iso.strat.83b">Early exercise + 83(b) election starts holding-period clock immediately</li>
                <li data-i18n="view.iso.strat.disqualifying_same_year">Disqualifying disposition same calendar year as exercise = no AMT preference (regular income only)</li>
                <li data-i18n="view.iso.strat.mtc">MTC recovers AMT paid against FUTURE regular tax. Plan multi-year recovery.</li>
                <li data-i18n="view.iso.strat.cash_for_amt">Save cash for AMT bill: 28% × bargain element. Don't get blindsided in April.</li>
            </ul>
        </div>
    `;
}
