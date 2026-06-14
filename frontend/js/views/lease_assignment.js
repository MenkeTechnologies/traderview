// Assignment of lease generator — months remaining + obligation transferred,
// via /calc/lease-assignment. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderLeaseAssignment(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.la.h1.title">// ASSIGNMENT OF LEASE</span></h1>
        <p class="muted small" data-i18n="view.la.hint.intro">
            The original tenant (assignor) transfers the entire remaining interest in a lease to a new
            tenant (assignee), who steps into the assignor's shoes for the rest of the term. Distinct from
            a sublease. It computes the whole months remaining and the rent obligation transferred, and
            handles whether the assignor is released or remains secondarily liable. Drafting aid, not
            legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.la.h2.inputs">Assignment details</h2>
            <form id="la-form" class="inline-form">
                <label><span data-i18n="view.la.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="Michigan" required></label>
                <label><span data-i18n="view.la.label.assignor">Assignor (original tenant)</span>
                    <input type="text" name="assignor_name" value=""></label>
                <label><span data-i18n="view.la.label.assignee">Assignee (new tenant)</span>
                    <input type="text" name="assignee_name" value=""></label>
                <label><span data-i18n="view.la.label.landlord">Landlord</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.la.label.property">Premises address</span>
                    <input type="text" name="property_address" value=""></label>
                <label><span data-i18n="view.la.label.effective">Effective date</span>
                    <input type="date" name="assignment_effective_date" value="2026-09-01" required></label>
                <label><span data-i18n="view.la.label.end">Original lease end date</span>
                    <input type="date" name="original_lease_end_date" value="2027-09-01" required></label>
                <label><span data-i18n="view.la.label.rent">Monthly rent ($)</span>
                    <input type="number" step="0.01" min="0" name="monthly_rent_usd" value="1800" required></label>
                <label><span data-i18n="view.la.label.released">Assignor released from liability</span>
                    <input type="checkbox" name="assignor_released"></label>
                <label><span data-i18n="view.la.label.deposit">Security deposit transferred ($)</span>
                    <input type="number" step="0.01" min="0" name="security_deposit_transfer_usd" value="1800"></label>
                <label><span data-i18n="view.la.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.la.ph.statute'))}"></label>
            </form>
        </div>
        <div id="la-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#la-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            assignor_name: (fd.get('assignor_name') || '').trim(),
            assignee_name: (fd.get('assignee_name') || '').trim(),
            landlord_name: (fd.get('landlord_name') || '').trim(),
            property_address: (fd.get('property_address') || '').trim(),
            assignment_effective_date: fd.get('assignment_effective_date'),
            original_lease_end_date: fd.get('original_lease_end_date'),
            monthly_rent_usd: Number(fd.get('monthly_rent_usd')) || 0,
            assignor_released: fd.get('assignor_released') != null,
            security_deposit_transfer_usd: Number(fd.get('security_deposit_transfer_usd')) || 0,
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcLeaseAssignment(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.la.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#la-result');
    const relKey = doc.assignor_released ? 'view.la.status.released' : 'view.la.status.liable';
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.la.card.months">Months remaining</div>
                    <div class="value">${doc.months_remaining}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.la.card.obligation">Rent obligation</div>
                    <div class="value">${money(doc.remaining_rent_obligation_usd)}</div></div>
                <div class="card ${doc.assignor_released ? 'pos' : 'neg'}"><div class="label" data-i18n="view.la.card.assignor">Assignor</div>
                    <div class="value" data-i18n="${relKey}"></div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="la-copy" type="button" data-i18n="view.la.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="la-download" type="button" data-i18n="view.la.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#la-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.la.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.la.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#la-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'lease-assignment.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
