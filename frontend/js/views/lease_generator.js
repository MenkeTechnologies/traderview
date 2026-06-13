// Residential lease generator — assembles a printable lease agreement and
// computes the move-in financials (term, prorated first month, move-in
// total) via /calc/lease-generator. Copy or download the document as text.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const TEXT = [
    ['landlord_name', 'Landlord name', 'Jane Owner'],
    ['tenant_name', 'Tenant name', 'John Renter'],
    ['property_address', 'Property address', '123 Main St, Apt 4'],
    ['state', 'Governing-law state', 'California'],
];
const NUM = [
    ['monthly_rent_usd', 'Monthly rent ($)', 2000],
    ['security_deposit_usd', 'Security deposit ($)', 2000],
    ['pet_deposit_usd', 'Pet deposit ($, 0 = none)', 0],
    ['rent_due_day', 'Rent due day of month', 1],
    ['late_fee_usd', 'Late fee ($)', 75],
    ['late_fee_grace_days', 'Late-fee grace (days)', 5],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));

let LAST_DOC = null;

export async function renderLeaseGenerator(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lease.h1.title">// RESIDENTIAL LEASE GENERATOR</span></h1>
        <p class="muted small" data-i18n="view.lease.hint.intro">
            Assemble a standard residential lease from your terms. It computes the lease
            length, prorates the first month when the tenant moves in mid-month, and totals
            the cash due at move-in (first month + deposits + optional last month), then
            renders a printable agreement you can copy or download. Drafting aid, not legal
            advice — deposit caps and required disclosures vary by state.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.lease.h2.inputs">Lease terms</h2>
            <form id="lease-form" class="inline-form">
                ${TEXT.map(([key, label, def]) => `
                    <label><span data-i18n="view.lease.label.${key}">${label}</span>
                        <input type="text" name="${key}" value="${esc(def)}" required></label>
                `).join('')}
                <label><span data-i18n="view.lease.label.lease_start">Lease start</span>
                    <input type="date" name="lease_start" value="2026-01-01" required></label>
                <label><span data-i18n="view.lease.label.lease_end">Lease end</span>
                    <input type="date" name="lease_end" value="2026-12-31" required></label>
                ${NUM.map(([key, label, def]) => `
                    <label><span data-i18n="view.lease.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
                <label data-tip="view.lease.tip.last_month"><input type="checkbox" name="last_month_required"> <span data-i18n="view.lease.label.last_month">Collect last month upfront</span></label>
                <button class="primary" type="submit" data-i18n="view.lease.btn.run">Generate lease</button>
            </form>
        </div>
        <div id="lease-result"></div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#lease-form');
    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = { last_month_required: fd.get('last_month_required') != null };
        for (const [key] of TEXT) body[key] = (fd.get(key) || '').trim();
        body.lease_start = fd.get('lease_start');
        body.lease_end = fd.get('lease_end');
        for (const [key] of NUM) body[key] = Number(fd.get(key)) || 0;
        body.rent_due_day = Math.round(body.rent_due_day);
        body.late_fee_grace_days = Math.round(body.late_fee_grace_days);
        try {
            const doc = await api.calcLeaseGenerator(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.lease.toast.error'), { level: 'error' });
        }
    });
    form.dispatchEvent(new Event('submit'));
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase(), ''];
    for (const c of doc.clauses) {
        lines.push(c.heading, c.body, '');
    }
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#lease-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.lease.h2.summary">Move-in summary</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.lease.card.term">Lease term</div>
                    <div class="value">${doc.term_months} <span data-i18n="view.lease.months">mo</span></div></div>
                <div class="card ${doc.first_month_is_prorated ? 'neg' : ''}"><div class="label" data-i18n="view.lease.card.first">First month${doc.first_month_is_prorated ? ' (prorated)' : ''}</div>
                    <div class="value">${money(doc.prorated_first_month_usd)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.lease.card.movein">Total due at move-in</div>
                    <div class="value">${money(doc.move_in_total_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.lease.card.total">Total lease value</div>
                    <div class="value">${money(doc.total_lease_value_usd)}</div></div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.lease.h2.document">The agreement</h2>
            <p>
                <button class="btn btn-secondary" id="lease-copy" type="button" data-i18n="view.lease.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="lease-download" type="button" data-i18n="view.lease.btn.download">Download .txt</button>
            </p>
            <pre class="small">${esc(docToText(doc))}</pre>
        </div>
    `;
    applyUiI18n(el);

    el.querySelector('#lease-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.lease.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.lease.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#lease-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'lease-agreement.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
