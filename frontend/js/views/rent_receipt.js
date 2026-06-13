// Rent receipt generator — records a payment against rent due, computing any
// balance or overpayment credit, via /calc/rent-receipt. Previews live as you
// type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderRentReceipt(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rcpt.h1.title">// RENT RECEIPT</span></h1>
        <p class="muted small" data-i18n="view.rcpt.hint.intro">
            The written acknowledgment of a rent payment a landlord gives (and some states require) a
            tenant. It records the amount paid against the rent due, computes any remaining balance or
            overpayment credit, flags paid-in-full, and notes the period covered and payment method.
            Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.rcpt.h2.inputs">Receipt details</h2>
            <form id="rcpt-form" class="inline-form">
                <label><span data-i18n="view.rcpt.label.landlord_name">Landlord name</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.rcpt.label.landlord_address">Landlord address</span>
                    <input type="text" name="landlord_address" value=""></label>
                <label><span data-i18n="view.rcpt.label.landlord_phone">Landlord phone</span>
                    <input type="text" name="landlord_phone" value=""></label>
                <label><span data-i18n="view.rcpt.label.tenant_name">Tenant name</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.rcpt.label.premises">Premises address</span>
                    <input type="text" name="premises_address" value=""></label>
                <label><span data-i18n="view.rcpt.label.amount_paid">Amount paid ($)</span>
                    <input type="number" step="0.01" min="0" name="amount_paid_usd" value="1500" required></label>
                <label><span data-i18n="view.rcpt.label.rent_due">Rent due ($)</span>
                    <input type="number" step="0.01" min="0" name="rent_due_usd" value="1500" required></label>
                <label><span data-i18n="view.rcpt.label.payment_date">Payment date</span>
                    <input type="date" name="payment_date" value="2026-06-01" required></label>
                <label><span data-i18n="view.rcpt.label.period_start">Period start</span>
                    <input type="date" name="period_start" value="2026-06-01" required></label>
                <label><span data-i18n="view.rcpt.label.period_end">Period end</span>
                    <input type="date" name="period_end" value="2026-06-30" required></label>
                <label><span data-i18n="view.rcpt.label.method">Payment method</span>
                    <input type="text" name="payment_method" value="Check #1024"></label>
                <label><span data-i18n="view.rcpt.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.rcpt.ph.statute'))}"></label>
            </form>
        </div>
        <div id="rcpt-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#rcpt-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            landlord_address: (fd.get('landlord_address') || '').trim(),
            landlord_phone: (fd.get('landlord_phone') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            amount_paid_usd: Number(fd.get('amount_paid_usd')) || 0,
            rent_due_usd: Number(fd.get('rent_due_usd')) || 0,
            payment_date: fd.get('payment_date'),
            period_start: fd.get('period_start'),
            period_end: fd.get('period_end'),
            payment_method: (fd.get('payment_method') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcRentReceipt(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.rcpt.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#rcpt-result');
    const statusKey = doc.overpayment_usd > 0
        ? 'view.rcpt.status.credit'
        : (doc.paid_in_full ? 'view.rcpt.status.full' : 'view.rcpt.status.partial');
    const statusCls = doc.paid_in_full ? 'pos' : 'neg';
    const secondCard = doc.overpayment_usd > 0
        ? `<div class="card pos"><div class="label" data-i18n="view.rcpt.card.credit">Credit</div>
               <div class="value">${money(doc.overpayment_usd)}</div></div>`
        : `<div class="card ${doc.balance_remaining_usd > 0 ? 'neg' : ''}"><div class="label" data-i18n="view.rcpt.card.balance">Balance remaining</div>
               <div class="value">${money(doc.balance_remaining_usd)}</div></div>`;
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card ${statusCls}"><div class="label" data-i18n="view.rcpt.card.status">Status</div>
                    <div class="value" data-i18n="${statusKey}"></div></div>
                <div class="card"><div class="label" data-i18n="view.rcpt.card.paid">Amount paid</div>
                    <div class="value">${money(doc.amount_paid_usd)}</div></div>
                ${secondCard}
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="rcpt-copy" type="button" data-i18n="view.rcpt.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="rcpt-download" type="button" data-i18n="view.rcpt.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#rcpt-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.rcpt.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.rcpt.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#rcpt-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'rent-receipt.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
