// Landlord notice generator — jurisdiction-agnostic landlord-tenant notices
// (Pay Rent or Quit / Notice to Quit), with the comply-by date computed from
// the service date, via /calc/landlord-notice. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
let LAST_DOC = null;

export async function renderLandlordNotice(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.notice.h1.title">// LANDLORD NOTICE GENERATOR</span></h1>
        <p class="muted small" data-i18n="view.notice.hint.intro">
            Generates the pre-eviction notices a landlord serves, in generic language that
            names your state rather than baking in any one jurisdiction's form: a Pay Rent
            or Quit (nonpayment) notice and a Notice to Quit (termination). It computes the
            comply-by date from the service date and notice period, and includes a statute
            citation verbatim if you supply one. Drafting aid, not legal advice — it does
            not replace any official court form your state requires.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.notice.h2.inputs">Notice details</h2>
            <form id="notice-form" class="inline-form">
                <label><span data-i18n="view.notice.label.type">Notice type</span>
                    <select name="notice_type" id="notice-type">
                        <option value="pay_or_quit" data-i18n="view.notice.type.pay_or_quit">Pay Rent or Quit (nonpayment)</option>
                        <option value="notice_to_quit" data-i18n="view.notice.type.quit">Notice to Quit (termination)</option>
                    </select></label>
                <label><span data-i18n="view.notice.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="Michigan" required></label>
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
                <label id="notice-rent-wrap"><span data-i18n="view.notice.label.rent_owed">Rent owed ($)</span>
                    <input type="number" step="0.01" min="0" name="rent_owed_usd" value="0"></label>
                <label id="notice-reason-wrap"><span data-i18n="view.notice.label.reason">Reason for termination</span>
                    <input type="text" name="reason" value=""></label>
                <label><span data-i18n="view.notice.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.notice.ph.statute'))}"></label>
            </form>
        </div>
        <div id="notice-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#notice-form');
    const typeSel = mount.querySelector('#notice-type');
    const daysInput = mount.querySelector('#notice-days');
    const rentWrap = mount.querySelector('#notice-rent-wrap');
    const reasonWrap = mount.querySelector('#notice-reason-wrap');
    let userTouchedDays = false;
    daysInput.addEventListener('input', () => { userTouchedDays = true; });
    const syncFields = () => {
        const isPay = typeSel.value === 'pay_or_quit';
        rentWrap.style.display = isPay ? '' : 'none';
        reasonWrap.style.display = isPay ? 'none' : '';
        if (!userTouchedDays) daysInput.value = isPay ? 7 : 30;
    };

    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            notice_type: fd.get('notice_type'),
            state: (fd.get('state') || '').trim(),
            landlord_name: (fd.get('landlord_name') || '').trim(),
            landlord_address: (fd.get('landlord_address') || '').trim(),
            landlord_phone: (fd.get('landlord_phone') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            served_date: fd.get('served_date'),
            notice_days: Math.round(Number(fd.get('notice_days')) || 0),
            rent_owed_usd: Number(fd.get('rent_owed_usd')) || 0,
            reason: (fd.get('reason') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcLandlordNotice(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.notice.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);

    typeSel.addEventListener('change', () => { syncFields(); generate(); });
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    syncFields();
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
    const el = mount.querySelector('#notice-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card neg"><div class="label" data-i18n="view.notice.card.complyby">Comply / move by</div>
                    <div class="value">${esc(doc.comply_by_date || '—')}</div></div>
                <div class="card"><div class="label" data-i18n="view.notice.card.days">Notice period</div>
                    <div class="value">${doc.notice_days} <span data-i18n="view.notice.days">days</span></div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="notice-copy" type="button" data-i18n="view.notice.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="notice-download" type="button" data-i18n="view.notice.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
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
        a.download = 'notice.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
