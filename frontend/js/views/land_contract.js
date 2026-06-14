// Land contract (contract for deed) generator — amortized installment sale +
// clauses, via /calc/land-contract. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderLandContract(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lc.h1.title">// LAND CONTRACT (CONTRACT FOR DEED)</span></h1>
        <p class="muted small" data-i18n="view.lc.hint.intro">
            An owner-financed installment sale where the seller keeps legal title until the buyer pays in
            full. It computes the monthly payment on the seller-financed balance, the total of payments,
            and the total interest, plus the maturity date. Distinct from the purchase agreement and the
            promissory note. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.lc.h2.inputs">Contract details</h2>
            <form id="lc-form" class="inline-form">
                <label><span data-i18n="view.lc.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="Ohio" required></label>
                <label><span data-i18n="view.lc.label.seller">Seller (vendor)</span>
                    <input type="text" name="seller_name" value=""></label>
                <label><span data-i18n="view.lc.label.buyer">Buyer (vendee)</span>
                    <input type="text" name="buyer_name" value=""></label>
                <label><span data-i18n="view.lc.label.property">Property address</span>
                    <input type="text" name="property_address" value=""></label>
                <label><span data-i18n="view.lc.label.price">Purchase price ($)</span>
                    <input type="number" step="1000" min="0" name="purchase_price_usd" value="200000" required></label>
                <label><span data-i18n="view.lc.label.down">Down payment ($)</span>
                    <input type="number" step="100" min="0" name="down_payment_usd" value="20000"></label>
                <label><span data-i18n="view.lc.label.rate">Annual rate (%)</span>
                    <input type="number" step="0.001" min="0" name="annual_rate_pct" value="6" required></label>
                <label><span data-i18n="view.lc.label.term">Term (months)</span>
                    <input type="number" step="1" min="1" name="term_months" value="360" required></label>
                <label><span data-i18n="view.lc.label.start">Start date</span>
                    <input type="date" name="start_date" value="2026-08-01" required></label>
                <label><span data-i18n="view.lc.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.lc.ph.statute'))}"></label>
            </form>
        </div>
        <div id="lc-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#lc-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            seller_name: (fd.get('seller_name') || '').trim(),
            buyer_name: (fd.get('buyer_name') || '').trim(),
            property_address: (fd.get('property_address') || '').trim(),
            purchase_price_usd: Number(fd.get('purchase_price_usd')) || 0,
            down_payment_usd: Number(fd.get('down_payment_usd')) || 0,
            annual_rate_pct: Number(fd.get('annual_rate_pct')) || 0,
            term_months: Math.round(Number(fd.get('term_months')) || 0),
            start_date: fd.get('start_date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcLandContract(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.lc.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase()];
    if (doc.statutory_citation) lines.push(doc.statutory_citation);
    lines.push('');
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#lc-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.lc.card.payment">Monthly payment</div>
                    <div class="value">${money(doc.monthly_payment_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.lc.card.financed">Financed balance</div>
                    <div class="value">${money(doc.financed_balance_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.lc.card.interest">Total interest</div>
                    <div class="value">${money(doc.total_interest_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.lc.card.maturity">Maturity</div>
                    <div class="value">${esc(doc.maturity_date || '—')}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="lc-copy" type="button" data-i18n="view.lc.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="lc-download" type="button" data-i18n="view.lc.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#lc-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.lc.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.lc.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#lc-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'land-contract.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
