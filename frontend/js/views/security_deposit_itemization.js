// Security-deposit itemization statement — landlord's itemized deposit
// accounting (deductions, balance returned/owed, statutory deadline) via
// /calc/security-deposit-itemization. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

const SEED = [
    { desc: 'Carpet cleaning', amt: 200 },
    { desc: 'Wall repair', amt: 350 },
];

function rowHtml(d) {
    return `
        <div class="mpb-row sdi-row">
            <input type="text" class="sdi-desc" placeholder="${esc(t('view.sdi.ph.desc'))}" value="${esc(d.desc || '')}">
            <input type="number" step="0.01" min="0" class="sdi-amt" placeholder="${esc(t('view.sdi.ph.amt'))}" value="${d.amt}">
            <button type="button" class="sdi-del" data-i18n="view.sdi.remove">Remove</button>
        </div>`;
}

export async function renderSecurityDepositItemization(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sdi.h1.title">// SECURITY DEPOSIT ITEMIZATION</span></h1>
        <p class="muted small" data-i18n="view.sdi.hint.intro">
            The itemized statement a landlord must send a departing tenant: the deposit held, each
            deduction with its amount, and the balance returned (or still owed). Most states require
            this in writing within a set number of days of move-out — miss it and you can forfeit the
            right to withhold anything. It computes the totals and the statutory deadline date.
            Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.sdi.h2.inputs">Statement details</h2>
            <form id="sdi-form" class="inline-form">
                <label><span data-i18n="view.sdi.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="California" required></label>
                <label><span data-i18n="view.sdi.label.landlord_name">Landlord name</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.sdi.label.landlord_address">Landlord address</span>
                    <input type="text" name="landlord_address" value=""></label>
                <label><span data-i18n="view.sdi.label.landlord_phone">Landlord phone</span>
                    <input type="text" name="landlord_phone" value=""></label>
                <label><span data-i18n="view.sdi.label.tenant_name">Tenant name</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.sdi.label.forwarding">Tenant forwarding address</span>
                    <input type="text" name="tenant_forwarding_address" value=""></label>
                <label><span data-i18n="view.sdi.label.premises">Premises address</span>
                    <input type="text" name="premises_address" value=""></label>
                <label><span data-i18n="view.sdi.label.deposit">Deposit held ($)</span>
                    <input type="number" step="0.01" min="0" name="deposit_held_usd" value="1500" required></label>
                <label><span data-i18n="view.sdi.label.interest">Interest owed ($)</span>
                    <input type="number" step="0.01" min="0" name="interest_owed_usd" value="0"></label>
                <label><span data-i18n="view.sdi.label.end_date">Tenancy end date</span>
                    <input type="date" name="tenancy_end_date" value="2026-06-30" required></label>
                <label><span data-i18n="view.sdi.label.deadline_days">Return deadline (days)</span>
                    <input type="number" step="1" min="1" name="return_deadline_days" value="21" required></label>
                <label><span data-i18n="view.sdi.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.sdi.ph.statute'))}"></label>
            </form>
            <div class="mpb-head sdi-head">
                <span data-i18n="view.sdi.col.desc">Deduction</span>
                <span data-i18n="view.sdi.col.amt">Amount ($)</span>
                <span></span>
            </div>
            <div id="sdi-rows">${SEED.map(rowHtml).join('')}</div>
            <button type="button" id="sdi-add" class="secondary" data-i18n="view.sdi.add">+ Add deduction</button>
        </div>
        <div id="sdi-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#sdi-form');
    const rowsEl = mount.querySelector('#sdi-rows');

    const generate = async () => {
        const fd = new FormData(form);
        const deductions = [...rowsEl.querySelectorAll('.sdi-row')].map((r) => ({
            description: (r.querySelector('.sdi-desc').value || '').trim(),
            amount_usd: Number(r.querySelector('.sdi-amt').value) || 0,
        })).filter((d) => d.description || d.amount_usd);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            landlord_address: (fd.get('landlord_address') || '').trim(),
            landlord_phone: (fd.get('landlord_phone') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            tenant_forwarding_address: (fd.get('tenant_forwarding_address') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            deposit_held_usd: Number(fd.get('deposit_held_usd')) || 0,
            interest_owed_usd: Number(fd.get('interest_owed_usd')) || 0,
            deductions,
            tenancy_end_date: fd.get('tenancy_end_date'),
            return_deadline_days: Math.round(Number(fd.get('return_deadline_days')) || 0),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcSecurityDepositItemization(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.sdi.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);

    mount.querySelector('#sdi-add').addEventListener('click', () => {
        rowsEl.insertAdjacentHTML('beforeend', rowHtml({ desc: '', amt: '' }));
        applyUiI18n(rowsEl.lastElementChild);
        generate();
    });
    rowsEl.addEventListener('click', (e) => {
        if (e.target.classList.contains('sdi-del')) {
            e.target.closest('.sdi-row').remove();
            generate();
        }
    });
    form.addEventListener('input', () => { live(); });
    rowsEl.addEventListener('input', () => { live(); });
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
    const el = mount.querySelector('#sdi-result');
    const owed = doc.balance_owed_by_tenant_usd > 0;
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card ${owed ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="${owed ? 'view.sdi.card.owed' : 'view.sdi.card.returned'}">${owed ? 'Owed by tenant' : 'Returned to tenant'}</div>
                    <div class="value">${money(owed ? doc.balance_owed_by_tenant_usd : doc.balance_returned_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.sdi.card.deductions">Total deductions</div>
                    <div class="value">${money(doc.total_deductions_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.sdi.card.deadline">Return by</div>
                    <div class="value">${esc(doc.return_deadline_date || '—')}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="sdi-copy" type="button" data-i18n="view.sdi.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="sdi-download" type="button" data-i18n="view.sdi.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#sdi-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.sdi.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.sdi.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#sdi-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'security-deposit-itemization.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
