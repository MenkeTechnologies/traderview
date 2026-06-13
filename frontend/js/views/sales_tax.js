// Sales tax / VAT — add tax to a net price or extract it from a gross total,
// across a stack of jurisdiction rates, via /calc/sales-tax. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 }) + '%';

// Parse a comma / space separated list of rate percents into a number array.
function parseRates(raw) {
    return String(raw || '')
        .split(/[\s,]+/)
        .map((s) => s.trim())
        .filter((s) => s.length)
        .map(Number)
        .filter((n) => Number.isFinite(n));
}

export async function renderSalesTax(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.salestax.h1.title">// SALES TAX / VAT</span></h1>
        <p class="muted small" data-i18n="view.salestax.hint.intro">
            Add sales tax to a pre-tax price, or back the tax out of a receipt total that already
            includes it. Enter one or more rates (state, county, city — or a single VAT rate) as
            percents separated by commas. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.salestax.h2.inputs">The sale</h2>
            <form id="salestax-form" class="inline-form">
                <label><span data-i18n="view.salestax.label.mode">Direction</span>
                    <select name="mode">
                        <option value="add_tax" data-i18n="view.salestax.mode.add">Add tax to a net price</option>
                        <option value="extract_tax" data-i18n="view.salestax.mode.extract">Extract tax from a gross total</option>
                    </select></label>
                <label><span data-i18n="view.salestax.label.amount">Amount ($)</span>
                    <input type="number" step="0.01" min="0" name="amount_usd" value="100" required></label>
                <label><span data-i18n="view.salestax.label.rates">Rates (%, comma-separated)</span>
                    <input type="text" name="rates_pct" value="6.25, 1.0, 0.75" required></label>
            </form>
        </div>
        <div id="salestax-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#salestax-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            mode: fd.get('mode'),
            amount_usd: Number(fd.get('amount_usd')) || 0,
            rates_pct: parseRates(fd.get('rates_pct')),
        };
        try {
            const r = await api.calcSalesTax(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.salestax.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#salestax-result');
    const rows = r.breakdown.map((p, i) =>
        `<tr><td>${t('view.salestax.row.jurisdiction', { n: i + 1 })} (${pct(p.rate_pct)})</td><td>${money(p.tax_usd)}</td></tr>`
    ).join('');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.salestax.h2.result">The breakdown</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.salestax.card.net">Net (pre-tax)</div>
                    <div class="value">${money(r.net_usd)}</div></div>
                <div class="card"><div class="label">${t('view.salestax.card.tax')} (${pct(r.combined_rate_pct)})</div>
                    <div class="value">${money(r.tax_usd)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.salestax.card.gross">Gross (total)</div>
                    <div class="value pos">${money(r.gross_usd)}</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.salestax.col.rate">Jurisdiction rate</th><th data-i18n="view.salestax.col.tax">Tax</th></tr></thead>
                <tbody>
                    ${rows}
                    <tr class="emph"><td data-i18n="view.salestax.row.total">Total tax</td><td>${money(r.tax_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
