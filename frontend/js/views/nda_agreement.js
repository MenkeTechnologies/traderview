// Non-disclosure agreement (NDA) generator — one-way or mutual, with expiration
// from effective date + term, via /calc/nda. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
let LAST_DOC = null;

export async function renderNdaAgreement(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.nda.h1.title">// NON-DISCLOSURE AGREEMENT</span></h1>
        <p class="muted small" data-i18n="view.nda.hint.intro">
            Protects confidential information shared between parties — one-way (one discloser) or mutual
            (both disclose). It computes the expiration date from the effective date plus the term and
            assembles the operative clauses: purpose, definition, obligations, exclusions, term, return of
            materials, and no-license. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.nda.h2.inputs">Agreement details</h2>
            <form id="nda-form" class="inline-form">
                <label><span data-i18n="view.nda.label.state">Governing state</span>
                    <input type="text" name="governing_state" value="New York" required></label>
                <label><span data-i18n="view.nda.label.disclosing">Disclosing party</span>
                    <input type="text" name="disclosing_party" value=""></label>
                <label><span data-i18n="view.nda.label.receiving">Receiving party</span>
                    <input type="text" name="receiving_party" value=""></label>
                <label><span data-i18n="view.nda.label.purpose">Purpose</span>
                    <input type="text" name="purpose" value="Evaluating a potential business relationship"></label>
                <label><span data-i18n="view.nda.label.effective">Effective date</span>
                    <input type="date" name="effective_date" value="2026-01-01" required></label>
                <label><span data-i18n="view.nda.label.term">Term (years)</span>
                    <input type="number" step="1" min="1" name="term_years" value="3" required></label>
                <label><span data-i18n="view.nda.label.mutual">Mutual (both disclose)</span>
                    <input type="checkbox" name="mutual"></label>
                <label><span data-i18n="view.nda.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.nda.ph.statute'))}"></label>
            </form>
        </div>
        <div id="nda-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#nda-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            disclosing_party: (fd.get('disclosing_party') || '').trim(),
            receiving_party: (fd.get('receiving_party') || '').trim(),
            mutual: fd.get('mutual') != null,
            purpose: (fd.get('purpose') || '').trim(),
            effective_date: fd.get('effective_date'),
            term_years: Math.round(Number(fd.get('term_years')) || 0),
            governing_state: (fd.get('governing_state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcNda(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.nda.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#nda-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.nda.card.type">Type</div>
                    <div class="value" data-i18n="${doc.mutual ? 'view.nda.type.mutual' : 'view.nda.type.oneway'}"></div></div>
                <div class="card"><div class="label" data-i18n="view.nda.card.expires">Expires</div>
                    <div class="value">${esc(doc.expiration_date || '—')}</div></div>
                <div class="card"><div class="label" data-i18n="view.nda.card.term">Term</div>
                    <div class="value">${doc.term_years} <span data-i18n="view.nda.years">yr</span></div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="nda-copy" type="button" data-i18n="view.nda.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="nda-download" type="button" data-i18n="view.nda.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#nda-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.nda.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.nda.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#nda-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'nda.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
