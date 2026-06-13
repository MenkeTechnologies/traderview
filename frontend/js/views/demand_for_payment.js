// Demand for payment (demand letter) generator — totals principal + interest +
// fees and computes the pay-by date, via /calc/demand-for-payment. Previews
// live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderDemandForPayment(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.demand.h1.title">// DEMAND FOR PAYMENT</span></h1>
        <p class="muted small" data-i18n="view.demand.hint.intro">
            The formal written demand a creditor sends before pursuing collection or suit — often a
            prerequisite to small-claims or collection action. It totals the principal, accrued interest,
            and late fees, and computes the pay-by date from the demand date plus the response window.
            Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.demand.h2.inputs">Letter details</h2>
            <form id="demand-form" class="inline-form">
                <label><span data-i18n="view.demand.label.state">Governing state</span>
                    <input type="text" name="governing_state" value="New York" required></label>
                <label><span data-i18n="view.demand.label.creditor">Creditor name</span>
                    <input type="text" name="creditor_name" value=""></label>
                <label><span data-i18n="view.demand.label.creditor_address">Creditor address</span>
                    <input type="text" name="creditor_address" value=""></label>
                <label><span data-i18n="view.demand.label.creditor_phone">Creditor phone</span>
                    <input type="text" name="creditor_phone" value=""></label>
                <label><span data-i18n="view.demand.label.debtor">Debtor name</span>
                    <input type="text" name="debtor_name" value=""></label>
                <label><span data-i18n="view.demand.label.description">What is owed for</span>
                    <input type="text" name="debt_description" value="Invoice #1042, unpaid"></label>
                <label><span data-i18n="view.demand.label.principal">Principal ($)</span>
                    <input type="number" step="0.01" min="0" name="principal_usd" value="5000" required></label>
                <label><span data-i18n="view.demand.label.interest">Accrued interest ($)</span>
                    <input type="number" step="0.01" min="0" name="accrued_interest_usd" value="150"></label>
                <label><span data-i18n="view.demand.label.fees">Late fees ($)</span>
                    <input type="number" step="0.01" min="0" name="late_fees_usd" value="50"></label>
                <label><span data-i18n="view.demand.label.demand_date">Demand date</span>
                    <input type="date" name="demand_date" value="2026-06-01" required></label>
                <label><span data-i18n="view.demand.label.response_days">Response window (days)</span>
                    <input type="number" step="1" min="1" name="response_days" value="15" required></label>
                <label><span data-i18n="view.demand.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.demand.ph.statute'))}"></label>
            </form>
        </div>
        <div id="demand-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#demand-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            creditor_name: (fd.get('creditor_name') || '').trim(),
            creditor_address: (fd.get('creditor_address') || '').trim(),
            creditor_phone: (fd.get('creditor_phone') || '').trim(),
            debtor_name: (fd.get('debtor_name') || '').trim(),
            principal_usd: Number(fd.get('principal_usd')) || 0,
            accrued_interest_usd: Number(fd.get('accrued_interest_usd')) || 0,
            late_fees_usd: Number(fd.get('late_fees_usd')) || 0,
            debt_description: (fd.get('debt_description') || '').trim(),
            demand_date: fd.get('demand_date'),
            response_days: Math.round(Number(fd.get('response_days')) || 0),
            governing_state: (fd.get('governing_state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcDemandForPayment(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.demand.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#demand-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card neg"><div class="label" data-i18n="view.demand.card.total">Total now due</div>
                    <div class="value">${money(doc.total_due_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.demand.card.principal">Principal</div>
                    <div class="value">${money(doc.principal_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.demand.card.payby">Pay by</div>
                    <div class="value">${esc(doc.pay_by_date || '—')}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="demand-copy" type="button" data-i18n="view.demand.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="demand-download" type="button" data-i18n="view.demand.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#demand-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.demand.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.demand.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#demand-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'demand-for-payment.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
