// Sublease agreement generator — end date + rent markup/discount vs the master
// lease, via /calc/sublease. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

export async function renderSublease(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.subl.h1.title">// SUBLEASE AGREEMENT</span></h1>
        <p class="muted small" data-i18n="view.subl.hint.intro">
            The original tenant (sublessor) rents the premises to a subtenant (sublessee) while remaining
            liable to the landlord under the master lease. It computes the sublease end date from the term
            and the markup/discount of the sublease rent versus the master-lease rent. Most leases require
            the landlord's written consent to sublet. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.subl.h2.inputs">Sublease details</h2>
            <form id="subl-form" class="inline-form">
                <label><span data-i18n="view.subl.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="New York" required></label>
                <label><span data-i18n="view.subl.label.sublessor">Sublessor (original tenant)</span>
                    <input type="text" name="sublessor_name" value=""></label>
                <label><span data-i18n="view.subl.label.sublessee">Sublessee (subtenant)</span>
                    <input type="text" name="sublessee_name" value=""></label>
                <label><span data-i18n="view.subl.label.landlord">Landlord</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.subl.label.premises">Premises address</span>
                    <input type="text" name="premises_address" value=""></label>
                <label><span data-i18n="view.subl.label.start">Sublease start date</span>
                    <input type="date" name="sublease_start_date" value="2026-08-01" required></label>
                <label><span data-i18n="view.subl.label.term">Term (months)</span>
                    <input type="number" step="1" min="1" name="term_months" value="6" required></label>
                <label><span data-i18n="view.subl.label.rent">Sublease rent ($)</span>
                    <input type="number" step="0.01" min="0" name="monthly_rent_usd" value="1600" required></label>
                <label><span data-i18n="view.subl.label.orig_rent">Master-lease rent ($)</span>
                    <input type="number" step="0.01" min="0" name="original_rent_usd" value="1500" required></label>
                <label><span data-i18n="view.subl.label.deposit">Security deposit ($)</span>
                    <input type="number" step="0.01" min="0" name="security_deposit_usd" value="1600"></label>
                <label><span data-i18n="view.subl.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.subl.ph.statute'))}"></label>
            </form>
        </div>
        <div id="subl-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#subl-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            sublessor_name: (fd.get('sublessor_name') || '').trim(),
            sublessee_name: (fd.get('sublessee_name') || '').trim(),
            landlord_name: (fd.get('landlord_name') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            sublease_start_date: fd.get('sublease_start_date'),
            term_months: Math.round(Number(fd.get('term_months')) || 0),
            monthly_rent_usd: Number(fd.get('monthly_rent_usd')) || 0,
            original_rent_usd: Number(fd.get('original_rent_usd')) || 0,
            security_deposit_usd: Number(fd.get('security_deposit_usd')) || 0,
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcSublease(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.subl.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#subl-result');
    const diff = doc.rent_difference_usd;
    const diffTxt = Math.abs(diff) < 0.005 ? t('view.subl.same') : `${money(diff)} · ${pct(doc.rent_difference_pct)}`;
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.subl.card.end">Sublease ends</div>
                    <div class="value">${esc(doc.sublease_end_date || '—')}</div></div>
                <div class="card"><div class="label" data-i18n="view.subl.card.rent">Sublease rent</div>
                    <div class="value">${money(doc.monthly_rent_usd)}</div></div>
                <div class="card ${diff > 0 ? 'pos' : (diff < 0 ? 'neg' : '')}"><div class="label" data-i18n="view.subl.card.diff">Vs master lease</div>
                    <div class="value">${diffTxt}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="subl-copy" type="button" data-i18n="view.subl.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="subl-download" type="button" data-i18n="view.subl.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#subl-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.subl.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.subl.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#subl-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'sublease-agreement.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
