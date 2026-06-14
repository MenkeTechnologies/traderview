// Seller's closing statement generator — commission + payoff + tax proration →
// net to seller, via /calc/closing-statement. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderClosingStatement(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cs.h1.title">// CLOSING STATEMENT</span></h1>
        <p class="muted small" data-i18n="view.cs.hint.intro">
            The seller's net sheet at a real-estate closing. From the sale price it subtracts the
            commission, the mortgage payoff, the seller's prorated share of annual property tax (days
            owed ÷ 365), and other closing costs to produce the net proceeds to the seller. Drafting aid,
            not legal/closing advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.cs.h2.inputs">Closing details</h2>
            <form id="cs-form" class="inline-form">
                <label><span data-i18n="view.cs.label.seller">Seller</span>
                    <input type="text" name="seller_name" value=""></label>
                <label><span data-i18n="view.cs.label.buyer">Buyer</span>
                    <input type="text" name="buyer_name" value=""></label>
                <label><span data-i18n="view.cs.label.property">Property address</span>
                    <input type="text" name="property_address" value=""></label>
                <label><span data-i18n="view.cs.label.price">Sale price ($)</span>
                    <input type="number" step="1000" min="0" name="sale_price_usd" value="400000" required></label>
                <label><span data-i18n="view.cs.label.closing">Closing date</span>
                    <input type="date" name="closing_date" value="2026-09-30" required></label>
                <label><span data-i18n="view.cs.label.commission">Commission (%)</span>
                    <input type="number" step="0.1" min="0" name="commission_pct" value="6"></label>
                <label><span data-i18n="view.cs.label.payoff">Mortgage payoff ($)</span>
                    <input type="number" step="100" min="0" name="mortgage_payoff_usd" value="250000"></label>
                <label><span data-i18n="view.cs.label.tax">Annual property tax ($)</span>
                    <input type="number" step="10" min="0" name="annual_property_tax_usd" value="4800"></label>
                <label><span data-i18n="view.cs.label.days">Tax days owed (of 365)</span>
                    <input type="number" step="1" min="0" max="365" name="tax_days_owed" value="180"></label>
                <label><span data-i18n="view.cs.label.other">Other closing costs ($)</span>
                    <input type="number" step="100" min="0" name="other_costs_usd" value="3000"></label>
            </form>
        </div>
        <div id="cs-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#cs-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            seller_name: (fd.get('seller_name') || '').trim(),
            buyer_name: (fd.get('buyer_name') || '').trim(),
            property_address: (fd.get('property_address') || '').trim(),
            sale_price_usd: Number(fd.get('sale_price_usd')) || 0,
            closing_date: fd.get('closing_date'),
            commission_pct: Number(fd.get('commission_pct')) || 0,
            mortgage_payoff_usd: Number(fd.get('mortgage_payoff_usd')) || 0,
            annual_property_tax_usd: Number(fd.get('annual_property_tax_usd')) || 0,
            tax_days_owed: Number(fd.get('tax_days_owed')) || 0,
            other_costs_usd: Number(fd.get('other_costs_usd')) || 0,
        };
        try {
            const doc = await api.calcClosingStatement(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.cs.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase(), ''];
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#cs-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.cs.card.net">Net to seller</div>
                    <div class="value">${money(doc.net_to_seller_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.cs.card.charges">Total charges</div>
                    <div class="value">${money(doc.total_deductions_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.cs.card.proration">Tax proration</div>
                    <div class="value">${money(doc.tax_proration_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="cs-copy" type="button" data-i18n="view.cs.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="cs-download" type="button" data-i18n="view.cs.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#cs-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.cs.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.cs.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#cs-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'closing-statement.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
