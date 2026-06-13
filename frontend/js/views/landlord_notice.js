// Landlord notice generator — Michigan SCAO landlord-tenant notices
// (DC 100a nonpayment demand / DC 100c notice to quit), with the comply-by
// date computed from the service date, via /calc/landlord-notice.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
let LAST_DOC = null;

export async function renderLandlordNotice(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.notice.h1.title">// LANDLORD NOTICE GENERATOR</span></h1>
        <p class="muted small" data-i18n="view.notice.hint.intro">
            Generates the pre-eviction notices a Michigan landlord serves — the DC 100a
            Demand for Possession (nonpayment of rent, 7-day) and the DC 100c Notice to Quit
            to recover possession (one rental period; 30 days for a month-to-month). It fills
            in the SCAO form language and statutory citations and computes the comply-by date
            from the service date. Drafting aid, not legal advice — it does not replace
            filing the official court form.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.notice.h2.inputs">Notice details</h2>
            <form id="notice-form" class="inline-form">
                <label><span data-i18n="view.notice.label.type">Notice type</span>
                    <select name="notice_type" id="notice-type">
                        <option value="demand_nonpayment_rent" data-i18n="view.notice.type.demand">DC 100a — Demand (nonpayment of rent)</option>
                        <option value="notice_to_quit" data-i18n="view.notice.type.quit">DC 100c — Notice to quit (recover possession)</option>
                    </select></label>
                <label><span data-i18n="view.notice.label.landlord_name">Landlord name</span>
                    <input type="text" name="landlord_name" value="" required></label>
                <label><span data-i18n="view.notice.label.landlord_address">Landlord address</span>
                    <input type="text" name="landlord_address" value="" required></label>
                <label><span data-i18n="view.notice.label.landlord_phone">Landlord phone</span>
                    <input type="text" name="landlord_phone" value=""></label>
                <label><span data-i18n="view.notice.label.tenant_name">Tenant name</span>
                    <input type="text" name="tenant_name" value="" required></label>
                <label><span data-i18n="view.notice.label.premises_address">Premises address</span>
                    <input type="text" name="premises_address" value="" required></label>
                <label><span data-i18n="view.notice.label.served_date">Date served</span>
                    <input type="date" name="served_date" value="2026-06-01" required></label>
                <label><span data-i18n="view.notice.label.notice_days">Notice period (days)</span>
                    <input type="number" step="1" min="1" name="notice_days" id="notice-days" value="7" required></label>
                <label id="notice-rent-wrap"><span data-i18n="view.notice.label.rent_owed">Rent owed ($) — DC 100a</span>
                    <input type="number" step="0.01" min="0" name="rent_owed_usd" value="0"></label>
                <label id="notice-reason-wrap"><span data-i18n="view.notice.label.reason">Other reason — DC 100c</span>
                    <input type="text" name="reason" value=""></label>
                <button class="primary" type="submit" data-i18n="view.notice.btn.run">Generate notice</button>
            </form>
        </div>
        <div id="notice-result"></div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#notice-form');
    const typeSel = mount.querySelector('#notice-type');
    const daysInput = mount.querySelector('#notice-days');
    const rentWrap = mount.querySelector('#notice-rent-wrap');
    const reasonWrap = mount.querySelector('#notice-reason-wrap');
    const syncFields = () => {
        const isDemand = typeSel.value === 'demand_nonpayment_rent';
        rentWrap.style.display = isDemand ? '' : 'none';
        reasonWrap.style.display = isDemand ? 'none' : '';
        daysInput.value = isDemand ? 7 : 30;
    };
    typeSel.addEventListener('change', syncFields);
    syncFields();

    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {
            notice_type: fd.get('notice_type'),
            landlord_name: (fd.get('landlord_name') || '').trim(),
            landlord_address: (fd.get('landlord_address') || '').trim(),
            landlord_phone: (fd.get('landlord_phone') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            served_date: fd.get('served_date'),
            notice_days: Math.round(Number(fd.get('notice_days')) || 0),
            rent_owed_usd: Number(fd.get('rent_owed_usd')) || 0,
            reason: (fd.get('reason') || '').trim(),
        };
        try {
            const doc = await api.calcLandlordNotice(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.notice.toast.error'), { level: 'error' });
        }
    });
    form.dispatchEvent(new Event('submit'));
}

function docToText(doc) {
    const lines = [
        'STATE OF MICHIGAN',
        doc.title.toUpperCase(),
        `Form ${doc.form_id}    ${doc.statutory_citation}`,
        '',
    ];
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#notice-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.notice.h2.summary">Notice</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.notice.card.form">SCAO form</div>
                    <div class="value">${esc(doc.form_id)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.notice.card.complyby">Comply / move by</div>
                    <div class="value">${esc(doc.comply_by_date)}</div></div>
                <div class="card"><div class="label" data-i18n="view.notice.card.days">Notice period</div>
                    <div class="value">${doc.notice_days} <span data-i18n="view.notice.days">days</span></div></div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.notice.h2.document">The notice</h2>
            <p>
                <button class="btn btn-secondary" id="notice-copy" type="button" data-i18n="view.notice.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="notice-download" type="button" data-i18n="view.notice.btn.download">Download .txt</button>
            </p>
            <pre class="small">${esc(docToText(doc))}</pre>
        </div>
    `;
    applyUiI18n(el);

    el.querySelector('#notice-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.notice.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.notice.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#notice-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = (LAST_DOC.form_id || 'notice').replace(/\s+/g, '-') + '.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
