// Cease and desist letter generator — comply-by deadline + demand clauses, via
// /calc/cease-desist. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
let LAST_DOC = null;

export async function renderCeaseDesist(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cd.h1.title">// CEASE AND DESIST LETTER</span></h1>
        <p class="muted small" data-i18n="view.cd.hint.intro">
            A formal demand that the recipient stop a specified conduct (harassment, IP infringement,
            defamation, breach). Unlike a demand for payment, the remedy sought is stopping an action. It
            computes the comply-by date from the service date plus the response window. Drafting aid, not
            legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.cd.h2.inputs">Letter details</h2>
            <form id="cd-form" class="inline-form">
                <label><span data-i18n="view.cd.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="New York" required></label>
                <label><span data-i18n="view.cd.label.sender">Sender</span>
                    <input type="text" name="sender_name" value=""></label>
                <label><span data-i18n="view.cd.label.sender_address">Sender address</span>
                    <input type="text" name="sender_address" value=""></label>
                <label><span data-i18n="view.cd.label.recipient">Recipient</span>
                    <input type="text" name="recipient_name" value=""></label>
                <label><span data-i18n="view.cd.label.conduct">Conduct to stop</span>
                    <input type="text" name="conduct_description" value="use of the ACME mark on competing products"></label>
                <label><span data-i18n="view.cd.label.basis">Legal basis</span>
                    <input type="text" name="legal_basis" value="trademark infringement"></label>
                <label><span data-i18n="view.cd.label.served">Date served</span>
                    <input type="date" name="served_date" value="2026-06-01" required></label>
                <label><span data-i18n="view.cd.label.days">Response window (days)</span>
                    <input type="number" step="1" min="1" name="response_days" value="14" required></label>
                <label><span data-i18n="view.cd.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.cd.ph.statute'))}"></label>
            </form>
        </div>
        <div id="cd-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#cd-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            sender_name: (fd.get('sender_name') || '').trim(),
            sender_address: (fd.get('sender_address') || '').trim(),
            recipient_name: (fd.get('recipient_name') || '').trim(),
            conduct_description: (fd.get('conduct_description') || '').trim(),
            legal_basis: (fd.get('legal_basis') || '').trim(),
            served_date: fd.get('served_date'),
            response_days: Math.round(Number(fd.get('response_days')) || 0),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcCeaseDesist(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.cd.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#cd-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card neg"><div class="label" data-i18n="view.cd.card.comply">Comply by</div>
                    <div class="value">${esc(doc.comply_by_date || '—')}</div></div>
                <div class="card"><div class="label" data-i18n="view.cd.card.days">Response window</div>
                    <div class="value">${doc.response_days} <span data-i18n="view.cd.days">days</span></div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="cd-copy" type="button" data-i18n="view.cd.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="cd-download" type="button" data-i18n="view.cd.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#cd-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.cd.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.cd.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#cd-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'cease-and-desist.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
