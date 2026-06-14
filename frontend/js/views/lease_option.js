// Lease-option (rent-to-own) generator — rent credits + net price at exercise,
// via /calc/lease-option. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderLeaseOption(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lo.h1.title">// LEASE-OPTION (RENT-TO-OWN)</span></h1>
        <p class="muted small" data-i18n="view.lo.hint.intro">
            A tenant leases with an option to buy at an agreed price, with the up-front option fee and a
            portion of each rent payment credited toward the purchase. It computes the accumulated rent
            credits over the option period, the total credits, the net price at exercise, and the option
            expiration date. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.lo.h2.inputs">Agreement details</h2>
            <form id="lo-form" class="inline-form">
                <label><span data-i18n="view.lo.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="Georgia" required></label>
                <label><span data-i18n="view.lo.label.landlord">Landlord / optionor</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.lo.label.tenant">Tenant / optionee</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.lo.label.property">Property address</span>
                    <input type="text" name="property_address" value=""></label>
                <label><span data-i18n="view.lo.label.fee">Option fee ($)</span>
                    <input type="number" step="100" min="0" name="option_fee_usd" value="5000" required></label>
                <label><span data-i18n="view.lo.label.credited">Option fee credited</span>
                    <input type="checkbox" name="option_fee_credited" checked></label>
                <label><span data-i18n="view.lo.label.rent">Monthly rent ($)</span>
                    <input type="number" step="0.01" min="0" name="monthly_rent_usd" value="2000" required></label>
                <label><span data-i18n="view.lo.label.credit">Monthly rent credit ($)</span>
                    <input type="number" step="0.01" min="0" name="monthly_rent_credit_usd" value="300"></label>
                <label><span data-i18n="view.lo.label.start">Option start date</span>
                    <input type="date" name="option_start_date" value="2026-08-01" required></label>
                <label><span data-i18n="view.lo.label.months">Option period (months)</span>
                    <input type="number" step="1" min="1" name="option_months" value="24" required></label>
                <label><span data-i18n="view.lo.label.price">Purchase price ($)</span>
                    <input type="number" step="1000" min="0" name="purchase_price_usd" value="350000" required></label>
                <label><span data-i18n="view.lo.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.lo.ph.statute'))}"></label>
            </form>
        </div>
        <div id="lo-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#lo-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            property_address: (fd.get('property_address') || '').trim(),
            option_fee_usd: Number(fd.get('option_fee_usd')) || 0,
            option_fee_credited: fd.get('option_fee_credited') != null,
            monthly_rent_usd: Number(fd.get('monthly_rent_usd')) || 0,
            monthly_rent_credit_usd: Number(fd.get('monthly_rent_credit_usd')) || 0,
            option_start_date: fd.get('option_start_date'),
            option_months: Math.round(Number(fd.get('option_months')) || 0),
            purchase_price_usd: Number(fd.get('purchase_price_usd')) || 0,
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcLeaseOption(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.lo.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#lo-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.lo.card.net">Net price at exercise</div>
                    <div class="value">${money(doc.net_price_at_exercise_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.lo.card.credits">Total credits</div>
                    <div class="value">${money(doc.total_credits_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.lo.card.end">Option ends</div>
                    <div class="value">${esc(doc.option_end_date || '—')}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="lo-copy" type="button" data-i18n="view.lo.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="lo-download" type="button" data-i18n="view.lo.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#lo-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.lo.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.lo.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#lo-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'lease-option.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
