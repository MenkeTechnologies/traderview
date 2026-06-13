// Bond accrued interest + dirty price — clean price plus the interest accrued
// since the last coupon, via /calc/accrued-interest. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const num = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 });

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
        </div>
        <div id="accrued-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#accrued-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            face_value: Number(fd.get('face_value')) || 0,
            coupon_rate_pct: Number(fd.get('coupon_rate_pct')) || 0,
            frequency: Number(fd.get('frequency')) || 1,
            clean_price: Number(fd.get('clean_price')) || 0,
            last_coupon: fd.get('last_coupon'),
            next_coupon: fd.get('next_coupon'),
            settlement: fd.get('settlement'),
            day_count: fd.get('day_count'),
        };
        try {
            const r = await api.calcAccruedInterest(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.accrued.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#accrued-result');
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
