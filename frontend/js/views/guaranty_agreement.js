// Lease guaranty / co-signer agreement generator — total rent over term + the
// guaranty clauses, via /calc/guaranty. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderGuaranty(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.guar.h1.title">// LEASE GUARANTY / CO-SIGNER</span></h1>
        <p class="muted small" data-i18n="view.guar.hint.intro">
            A guarantor (often a parent or principal) unconditionally guarantees a tenant's obligations
            under a lease. It computes the total rent over the term as a measure of the guaranteed
            exposure and assembles the guaranty clauses. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.guar.h2.inputs">Guaranty details</h2>
            <form id="guar-form" class="inline-form">
                <label><span data-i18n="view.guar.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="Massachusetts" required></label>
                <label><span data-i18n="view.guar.label.guarantor">Guarantor</span>
                    <input type="text" name="guarantor_name" value=""></label>
                <label><span data-i18n="view.guar.label.tenant">Tenant</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.guar.label.landlord">Landlord</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.guar.label.premises">Premises address</span>
                    <input type="text" name="premises_address" value=""></label>
                <label><span data-i18n="view.guar.label.rent">Monthly rent ($)</span>
                    <input type="number" step="0.01" min="0" name="monthly_rent_usd" value="1500" required></label>
                <label><span data-i18n="view.guar.label.term">Lease term (months)</span>
                    <input type="number" step="1" min="1" name="lease_term_months" value="12" required></label>
                <label><span data-i18n="view.guar.label.start">Lease start date</span>
                    <input type="date" name="lease_start_date" value="2026-09-01" required></label>
                <label><span data-i18n="view.guar.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.guar.ph.statute'))}"></label>
            </form>
        </div>
        <div id="guar-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#guar-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            guarantor_name: (fd.get('guarantor_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            landlord_name: (fd.get('landlord_name') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            monthly_rent_usd: Number(fd.get('monthly_rent_usd')) || 0,
            lease_term_months: Math.round(Number(fd.get('lease_term_months')) || 0),
            lease_start_date: fd.get('lease_start_date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcGuaranty(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.guar.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#guar-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card neg"><div class="label" data-i18n="view.guar.card.total">Rent over term</div>
                    <div class="value">${money(doc.total_rent_over_term_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.guar.card.rent">Monthly rent</div>
                    <div class="value">${money(doc.monthly_rent_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.guar.card.term">Term</div>
                    <div class="value">${doc.lease_term_months} <span data-i18n="view.guar.months">mo</span></div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="guar-copy" type="button" data-i18n="view.guar.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="guar-download" type="button" data-i18n="view.guar.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#guar-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.guar.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.guar.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#guar-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'lease-guaranty.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
