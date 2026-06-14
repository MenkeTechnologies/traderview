// Bond accrued interest + dirty price — clean price plus the interest accrued
// since the last coupon, via /calc/accrued-interest. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const num = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 });
const VIEW = 'accrued-interest';
let lastReport = null;
let lastBody = null;

export async function renderAccruedInterest(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.accrued.h1.title">// ACCRUED INTEREST</span></h1>
        <p class="muted small" data-i18n="view.accrued.hint.intro">
            Between coupon dates a bond buyer owes the seller the interest accrued since the last
            coupon. The quoted clean price excludes it; the dirty price you actually pay adds it
            back. Pick the day-count convention (30/360 for corporates/munis, Actual/Actual for
            Treasuries). Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.accrued.h2.inputs">The bond</h2>
            <form id="accrued-form" class="inline-form">
                <label><span data-i18n="view.accrued.label.face">Face value ($)</span>
                    <input type="number" step="0.01" min="0" name="face_value" value="1000" required></label>
                <label><span data-i18n="view.accrued.label.coupon">Coupon rate (%)</span>
                    <input type="number" step="0.001" min="0" name="coupon_rate_pct" value="6" required></label>
                <label><span data-i18n="view.accrued.label.freq">Coupons / year</span>
                    <select name="frequency">
                        <option value="1" data-i18n="view.accrued.freq.1">Annual</option>
                        <option value="2" selected data-i18n="view.accrued.freq.2">Semiannual</option>
                        <option value="4" data-i18n="view.accrued.freq.4">Quarterly</option>
                        <option value="12" data-i18n="view.accrued.freq.12">Monthly</option>
                    </select></label>
                <label><span data-i18n="view.accrued.label.clean">Clean price ($)</span>
                    <input type="number" step="0.01" min="0" name="clean_price" value="980" required></label>
                <label><span data-i18n="view.accrued.label.daycount">Day count</span>
                    <select name="day_count">
                        <option value="thirty360" data-i18n="view.accrued.dc.thirty360">30/360 (corporate/muni)</option>
                        <option value="actual_actual" data-i18n="view.accrued.dc.actualactual">Actual/Actual (Treasury)</option>
                    </select></label>
                <label><span data-i18n="view.accrued.label.last">Last coupon date</span>
                    <input type="date" name="last_coupon" value="2026-01-01" required></label>
                <label><span data-i18n="view.accrued.label.next">Next coupon date</span>
                    <input type="date" name="next_coupon" value="2026-07-01" required></label>
                <label><span data-i18n="view.accrued.label.settle">Settlement date</span>
                    <input type="date" name="settlement" value="2026-04-01" required></label>
            </form>
            <div id="accrued-tools" class="ce-toolbar"></div>
            <button type="button" id="accrued-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="accrued-sens" class="ce-sens"></div>
        </div>
        <div id="accrued-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#accrued-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            face_value: Number(fd.get('face_value')) || 0,
            coupon_rate_pct: Number(fd.get('coupon_rate_pct')) || 0,
            frequency: Number(fd.get('frequency')) || 1,
            clean_price: Number(fd.get('clean_price')) || 0,
            last_coupon: fd.get('last_coupon'),
            next_coupon: fd.get('next_coupon'),
            settlement: fd.get('settlement'),
            day_count: fd.get('day_count'),
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcAccruedInterest(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.accrued.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#accrued-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'accrued-interest.csv' });
    mount.querySelector('#accrued-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['accrued_interest', r.accrued_interest],
        ['dirty_price', r.dirty_price],
        ['clean_price', r.clean_price],
        ['coupon_payment', r.coupon_payment],
        ['days_accrued', r.days_accrued],
        ['accrual_fraction', r.accrual_fraction],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#accrued-result');
    // Line chart: accrued interest as coupon rate sweeps 0 → 12% (linear in coupon).
    const xs = enh.linspace(0, 12, 13);
    const pts = await Promise.all(xs.map(async (c) => {
        const rr = await api.calcAccruedInterest({ ...body, coupon_rate_pct: c });
        return { x: c, y: rr && rr.accrued_interest != null ? rr.accrued_interest : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'coupon %', ylabel: 'accrued $' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.accrued.h2.result">What you pay</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.accrued.card.clean">Clean price</div>
                    <div class="value">${money(r.clean_price)}</div></div>
                <div class="card"><div class="label" data-i18n="view.accrued.card.accrued">Accrued interest</div>
                    <div class="value">${money(r.accrued_interest)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.accrued.card.dirty">Dirty price</div>
                    <div class="value pos">${money(r.dirty_price)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.accrued.row.coupon">Coupon payment</td><td>${money(r.coupon_payment)}</td></tr>
                    <tr><td data-i18n="view.accrued.row.daysaccrued">Days accrued</td><td>${num(r.days_accrued)}</td></tr>
                    <tr><td data-i18n="view.accrued.row.daysperiod">Days in period</td><td>${num(r.days_in_period)}</td></tr>
                    <tr><td data-i18n="view.accrued.row.fraction">Accrual fraction</td><td>${num(r.accrual_fraction)}</td></tr>
                    <tr class="emph"><td data-i18n="view.accrued.row.accrued">Accrued interest</td><td>${money(r.accrued_interest)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#accrued-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: coupon rate 0 → 12%; y: clean price 0.8× → 1.2× current. Output: dirty price.
    const cp = base.clean_price || 980;
    const xVals = enh.linspace(0, 12, 5);
    const yVals = enh.linspace(cp * 0.8, cp * 1.2, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'coupon_rate_pct', yKey: 'clean_price', xVals, yVals, compute: (b) => api.calcAccruedInterest(b), pick: (r) => (r ? r.dirty_price : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : '$' + v.toFixed(0)), xfmt: (v) => v.toFixed(1) + '%', yfmt: (v) => '$' + v.toFixed(0), xName: t('view.accrued.label.coupon') || 'Coupon', yName: t('view.accrued.label.clean') || 'Clean' });
}
