// Earnest money receipt generator — deposit %, balance at closing, escrow
// terms, via /calc/earnest-money-receipt. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

export async function renderEarnestMoneyReceipt(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.em.h1.title">// EARNEST MONEY RECEIPT</span></h1>
        <p class="muted small" data-i18n="view.em.hint.intro">
            The escrow holder's acknowledgment of the buyer's good-faith deposit on a real-estate
            purchase. It records the deposit, computes it as a percent of the purchase price and the
            balance due at closing, and states the escrow/refund terms. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.em.h2.inputs">Receipt details</h2>
            <form id="em-form" class="inline-form">
                <label><span data-i18n="view.em.label.escrow">Escrow holder</span>
                    <input type="text" name="escrow_holder_name" value=""></label>
                <label><span data-i18n="view.em.label.buyer">Buyer</span>
                    <input type="text" name="buyer_name" value=""></label>
                <label><span data-i18n="view.em.label.seller">Seller</span>
                    <input type="text" name="seller_name" value=""></label>
                <label><span data-i18n="view.em.label.property">Property address</span>
                    <input type="text" name="property_address" value=""></label>
                <label><span data-i18n="view.em.label.earnest">Earnest money ($)</span>
                    <input type="number" step="100" min="0" name="earnest_money_usd" value="8000" required></label>
                <label><span data-i18n="view.em.label.price">Purchase price ($)</span>
                    <input type="number" step="1000" min="0" name="purchase_price_usd" value="400000" required></label>
                <label><span data-i18n="view.em.label.date">Received date</span>
                    <input type="date" name="received_date" value="2026-06-15" required></label>
                <label><span data-i18n="view.em.label.method">Payment method</span>
                    <input type="text" name="payment_method" value="wire transfer"></label>
                <label><span data-i18n="view.em.label.state">State (optional)</span>
                    <input type="text" name="state" value="Arizona"></label>
            </form>
        </div>
        <div id="em-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#em-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            escrow_holder_name: (fd.get('escrow_holder_name') || '').trim(),
            buyer_name: (fd.get('buyer_name') || '').trim(),
            seller_name: (fd.get('seller_name') || '').trim(),
            property_address: (fd.get('property_address') || '').trim(),
            earnest_money_usd: Number(fd.get('earnest_money_usd')) || 0,
            purchase_price_usd: Number(fd.get('purchase_price_usd')) || 0,
            received_date: fd.get('received_date'),
            payment_method: (fd.get('payment_method') || '').trim(),
            state: (fd.get('state') || '').trim(),
        };
        try {
            const doc = await api.calcEarnestMoneyReceipt(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.em.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#em-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.em.card.earnest">Earnest money</div>
                    <div class="value">${money(doc.earnest_money_usd)} · ${pct(doc.earnest_money_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.em.card.balance">Balance at closing</div>
                    <div class="value">${money(doc.balance_at_closing_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="em-copy" type="button" data-i18n="view.em.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="em-download" type="button" data-i18n="view.em.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#em-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.em.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.em.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#em-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'earnest-money-receipt.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
