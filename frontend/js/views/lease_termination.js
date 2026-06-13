// Lease termination letter generator — move-out date from the service date +
// notice period, with landlord/tenant wording, via /calc/lease-termination.
// Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
let LAST_DOC = null;

export async function renderLeaseTermination(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lter.h1.title">// LEASE TERMINATION LETTER</span></h1>
        <p class="muted small" data-i18n="view.lter.hint.intro">
            The written notice either party gives to end a tenancy. Pick who is sending it — the wording
            adapts for a landlord or a tenant. It computes the termination/move-out date from the service
            date plus the notice period. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.lter.h2.inputs">Notice details</h2>
            <form id="lter-form" class="inline-form">
                <label><span data-i18n="view.lter.label.role">Sender</span>
                    <select name="sender_role" id="lter-role">
                        <option value="tenant" data-i18n="view.lter.role.tenant">Tenant (giving notice)</option>
                        <option value="landlord" data-i18n="view.lter.role.landlord">Landlord (giving notice)</option>
                    </select></label>
                <label><span data-i18n="view.lter.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="Illinois" required></label>
                <label><span data-i18n="view.lter.label.sender_name">Sender name</span>
                    <input type="text" name="sender_name" value=""></label>
                <label><span data-i18n="view.lter.label.sender_address">Sender address</span>
                    <input type="text" name="sender_address" value=""></label>
                <label><span data-i18n="view.lter.label.sender_phone">Sender phone</span>
                    <input type="text" name="sender_phone" value=""></label>
                <label><span data-i18n="view.lter.label.recipient">Recipient name</span>
                    <input type="text" name="recipient_name" value=""></label>
                <label><span data-i18n="view.lter.label.premises">Premises address</span>
                    <input type="text" name="premises_address" value=""></label>
                <label><span data-i18n="view.lter.label.served">Date served</span>
                    <input type="date" name="served_date" value="2026-06-01" required></label>
                <label><span data-i18n="view.lter.label.notice_days">Notice period (days)</span>
                    <input type="number" step="1" min="1" name="notice_days" value="30" required></label>
                <label id="lter-reason-wrap"><span data-i18n="view.lter.label.reason">Reason (optional)</span>
                    <input type="text" name="reason" value=""></label>
                <label><span data-i18n="view.lter.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.lter.ph.statute'))}"></label>
            </form>
        </div>
        <div id="lter-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#lter-form');
    const role = mount.querySelector('#lter-role');
    const reasonWrap = mount.querySelector('#lter-reason-wrap');
    const syncFields = () => { reasonWrap.style.display = role.value === 'landlord' ? '' : 'none'; };

    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            sender_role: fd.get('sender_role'),
            sender_name: (fd.get('sender_name') || '').trim(),
            sender_address: (fd.get('sender_address') || '').trim(),
            sender_phone: (fd.get('sender_phone') || '').trim(),
            recipient_name: (fd.get('recipient_name') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            served_date: fd.get('served_date'),
            notice_days: Math.round(Number(fd.get('notice_days')) || 0),
            reason: (fd.get('reason') || '').trim(),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcLeaseTermination(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.lter.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    role.addEventListener('change', () => { syncFields(); generate(); });
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
    const el = mount.querySelector('#lter-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card neg"><div class="label" data-i18n="view.lter.card.term">Termination date</div>
                    <div class="value">${esc(doc.termination_date || '—')}</div></div>
                <div class="card"><div class="label" data-i18n="view.lter.card.days">Notice period</div>
                    <div class="value">${doc.notice_days} <span data-i18n="view.lter.days">days</span></div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="lter-copy" type="button" data-i18n="view.lter.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="lter-download" type="button" data-i18n="view.lter.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#lter-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.lter.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.lter.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#lter-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'lease-termination.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
