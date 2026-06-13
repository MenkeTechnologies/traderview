// Lease renewal / extension agreement generator — new end date + rent change,
// via /calc/lease-renewal. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

export async function renderLeaseRenewal(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lrnw.h1.title">// LEASE RENEWAL / EXTENSION</span></h1>
        <p class="muted small" data-i18n="view.lrnw.hint.intro">
            Extends an existing tenancy for a new fixed term, optionally at an adjusted rent. It computes
            the new lease end date (renewal start + term, ending the day before the next period) and the
            rent change versus the expiring rent, then assembles the renewal agreement. Drafting aid, not
            legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.lrnw.h2.inputs">Renewal details</h2>
            <form id="lrnw-form" class="inline-form">
                <label><span data-i18n="view.lrnw.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="Ohio" required></label>
                <label><span data-i18n="view.lrnw.label.landlord_name">Landlord name</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.lrnw.label.landlord_address">Landlord address</span>
                    <input type="text" name="landlord_address" value=""></label>
                <label><span data-i18n="view.lrnw.label.landlord_phone">Landlord phone</span>
                    <input type="text" name="landlord_phone" value=""></label>
                <label><span data-i18n="view.lrnw.label.tenant_name">Tenant name</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.lrnw.label.premises">Premises address</span>
                    <input type="text" name="premises_address" value=""></label>
                <label><span data-i18n="view.lrnw.label.start">Renewal start date</span>
                    <input type="date" name="renewal_start_date" value="2026-08-01" required></label>
                <label><span data-i18n="view.lrnw.label.term">Term (months)</span>
                    <input type="number" step="1" min="1" name="term_months" value="12" required></label>
                <label><span data-i18n="view.lrnw.label.current_rent">Current rent ($)</span>
                    <input type="number" step="0.01" min="0" name="current_rent_usd" value="1500" required></label>
                <label><span data-i18n="view.lrnw.label.new_rent">New rent ($)</span>
                    <input type="number" step="0.01" min="0" name="new_rent_usd" value="1575" required></label>
                <label><span data-i18n="view.lrnw.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.lrnw.ph.statute'))}"></label>
            </form>
        </div>
        <div id="lrnw-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#lrnw-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            landlord_address: (fd.get('landlord_address') || '').trim(),
            landlord_phone: (fd.get('landlord_phone') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            renewal_start_date: fd.get('renewal_start_date'),
            term_months: Math.round(Number(fd.get('term_months')) || 0),
            current_rent_usd: Number(fd.get('current_rent_usd')) || 0,
            new_rent_usd: Number(fd.get('new_rent_usd')) || 0,
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcLeaseRenewal(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.lrnw.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#lrnw-result');
    const change = doc.rent_change_usd;
    const changeTxt = Math.abs(change) < 0.005
        ? t('view.lrnw.unchanged')
        : `${money(change)} · ${pct(doc.rent_change_pct)}`;
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.lrnw.card.end">Renewal ends</div>
                    <div class="value">${esc(doc.renewal_end_date || '—')}</div></div>
                <div class="card"><div class="label" data-i18n="view.lrnw.card.new_rent">New rent</div>
                    <div class="value">${money(doc.new_rent_usd)}</div></div>
                <div class="card ${change > 0 ? 'neg' : ''}"><div class="label" data-i18n="view.lrnw.card.change">Rent change</div>
                    <div class="value">${changeTxt}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="lrnw-copy" type="button" data-i18n="view.lrnw.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="lrnw-download" type="button" data-i18n="view.lrnw.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#lrnw-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.lrnw.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.lrnw.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#lrnw-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'lease-renewal.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
