// Bond pricing — price a coupon bond from its yield to maturity, via
// /calc/bond-pricing. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '%';

const PD = { premium: 'view.bondprice.pd.premium', discount: 'view.bondprice.pd.discount', par: 'view.bondprice.pd.par' };
const VIEW = 'bond-pricing';
let lastReport = null;
let lastBody = null;

export async function renderBondPricing(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.bondprice.h1.title">// BOND PRICING</span></h1>
        <p class="muted small" data-i18n="view.bondprice.hint.intro">
            The present value of a coupon bond given its yield to maturity — the price you'd pay
            today. It trades above par (premium) when the coupon beats the yield, below (discount)
            when the yield beats the coupon. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.bondprice.h2.inputs">The bond</h2>
            <form id="bondprice-form" class="inline-form">
                <label><span data-i18n="view.bondprice.label.face">Face value ($)</span>
                    <input type="number" step="0.01" min="0" name="face_value_usd" value="1000" required></label>
                <label><span data-i18n="view.bondprice.label.coupon">Coupon rate (%)</span>
                    <input type="number" step="0.001" min="0" name="coupon_rate_pct" value="5" required></label>
                <label><span data-i18n="view.bondprice.label.ytm">Yield to maturity (%)</span>
                    <input type="number" step="0.001" min="0" name="ytm_pct" value="6" required></label>
                <label><span data-i18n="view.bondprice.label.years">Years to maturity</span>
                    <input type="number" step="0.5" min="0" name="years_to_maturity" value="10" required></label>
                <label><span data-i18n="view.bondprice.label.freq">Coupons / year</span>
                    <select name="frequency">
                        <option value="1" data-i18n="view.bondprice.freq.1">Annual</option>
                        <option value="2" selected data-i18n="view.bondprice.freq.2">Semiannual</option>
                        <option value="4" data-i18n="view.bondprice.freq.4">Quarterly</option>
                    </select></label>
            </form>
            <div id="bondprice-tools" class="ce-toolbar"></div>
            <button type="button" id="bondprice-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="bondprice-sens" class="ce-sens"></div>
        </div>
        <div id="bondprice-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#bondprice-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            face_value_usd: Number(fd.get('face_value_usd')) || 0,
            coupon_rate_pct: Number(fd.get('coupon_rate_pct')) || 0,
            ytm_pct: Number(fd.get('ytm_pct')) || 0,
            years_to_maturity: Number(fd.get('years_to_maturity')) || 0,
            frequency: Number(fd.get('frequency')) || 1,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcBondPricing(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.bondprice.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#bondprice-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'bond-pricing.csv' });
    mount.querySelector('#bondprice-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['price_usd', r.price_usd],
        ['premium_discount', r.premium_discount],
        ['current_yield_pct', r.current_yield_pct],
        ['annual_coupon_usd', r.annual_coupon_usd],
        ['capital_gain_at_maturity_usd', r.capital_gain_at_maturity_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#bondprice-result');
    const gainCls = r.capital_gain_at_maturity_usd >= 0 ? 'pos' : 'neg';
    // Line chart: the price-yield curve — price as YTM sweeps 1% → 12% (convex, downward).
    const xs = enh.linspace(1, 12, 13);
    const pts = await Promise.all(xs.map(async (y) => {
        const rr = await api.calcBondPricing({ ...body, ytm_pct: y });
        return { x: y, y: rr && rr.price_usd != null ? rr.price_usd : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'YTM %', ylabel: 'price $' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.bondprice.h2.result">The price</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.bondprice.card.price">Price</div>
                    <div class="value pos">${money(r.price_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.bondprice.card.cy">Current yield</div>
                    <div class="value">${pct(r.current_yield_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.bondprice.card.pd">Premium / discount</div>
                    <div class="value" data-i18n="${PD[r.premium_discount]}">—</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.bondprice.row.coupon">Annual coupon</td><td>${money(r.annual_coupon_usd)}</td></tr>
                    <tr><td data-i18n="view.bondprice.row.cy">Current yield</td><td>${pct(r.current_yield_pct)}</td></tr>
                    <tr class="${gainCls}"><td data-i18n="view.bondprice.row.gain">Capital gain at maturity</td><td>${money(r.capital_gain_at_maturity_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.bondprice.row.price">Price</td><td>${money(r.price_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#bondprice-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: YTM 1% → 12%; y: years to maturity 1 → 30. Output: price (longer + higher yield = bigger discount).
    const xVals = enh.linspace(1, 12, 5);
    const yVals = enh.linspace(1, 30, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'ytm_pct', yKey: 'years_to_maturity', xVals, yVals, compute: (b) => api.calcBondPricing(b), pick: (r) => (r ? r.price_usd : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : '$' + v.toFixed(0)), xfmt: (v) => v.toFixed(1) + '%', yfmt: (v) => v.toFixed(0) + 'y', xName: t('view.bondprice.label.ytm') || 'YTM', yName: t('view.bondprice.label.years') || 'Years' });
}
