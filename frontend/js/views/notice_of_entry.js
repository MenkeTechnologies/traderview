// Notice of entry generator — earliest lawful entry date from the service date
// + notice period, via /calc/notice-of-entry. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
let LAST_DOC = null;

export async function renderNoticeOfEntry(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.noe.h1.title">// NOTICE OF ENTRY</span></h1>
        <p class="muted small" data-i18n="view.noe.hint.intro">
            The advance notice a landlord must give before entering a tenant's unit — most states require
            24–48 hours for non-emergency entry. It computes the earliest lawful entry date from the
            service date plus the notice period and assembles the notice with the purpose and time window.
            Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.noe.h2.inputs">Notice details</h2>
            <form id="noe-form" class="inline-form">
                <label><span data-i18n="view.noe.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="California" required></label>
                <label><span data-i18n="view.noe.label.landlord_name">Landlord name</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.noe.label.landlord_address">Landlord address</span>
                    <input type="text" name="landlord_address" value=""></label>
                <label><span data-i18n="view.noe.label.landlord_phone">Landlord phone</span>
                    <input type="text" name="landlord_phone" value=""></label>
                <label><span data-i18n="view.noe.label.tenant_name">Tenant name</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.noe.label.premises">Premises address</span>
                    <input type="text" name="premises_address" value=""></label>
                <label><span data-i18n="view.noe.label.served">Date served</span>
                    <input type="date" name="served_date" value="2026-06-13" required></label>
                <label><span data-i18n="view.noe.label.notice_days">Notice period (days)</span>
                    <input type="number" step="1" min="1" name="notice_days" value="2" required></label>
                <label><span data-i18n="view.noe.label.purpose">Purpose of entry</span>
                    <input type="text" name="purpose" value="Repair the kitchen faucet"></label>
                <label><span data-i18n="view.noe.label.window">Time window</span>
                    <input type="text" name="time_window" value="9:00 AM – 12:00 PM"></label>
                <label><span data-i18n="view.noe.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.noe.ph.statute'))}"></label>
            </form>
        </div>
        <div id="noe-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#noe-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            landlord_address: (fd.get('landlord_address') || '').trim(),
            landlord_phone: (fd.get('landlord_phone') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            served_date: fd.get('served_date'),
            notice_days: Math.round(Number(fd.get('notice_days')) || 0),
            purpose: (fd.get('purpose') || '').trim(),
            time_window: (fd.get('time_window') || '').trim(),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcNoticeOfEntry(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.noe.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#noe-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.noe.card.entry">Earliest entry</div>
                    <div class="value">${esc(doc.entry_date || '—')}</div></div>
                <div class="card"><div class="label" data-i18n="view.noe.card.days">Notice period</div>
                    <div class="value">${doc.notice_days} <span data-i18n="view.noe.days">days</span></div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="noe-copy" type="button" data-i18n="view.noe.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="noe-download" type="button" data-i18n="view.noe.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#noe-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.noe.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.noe.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#noe-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'notice-of-entry.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
