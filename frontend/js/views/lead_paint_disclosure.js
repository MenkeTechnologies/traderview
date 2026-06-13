// Lead-based paint disclosure generator — pre-1978 applicability + lessor
// disclosure, via /calc/lead-paint-disclosure. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
let LAST_DOC = null;

export async function renderLeadPaintDisclosure(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lead.h1.title">// LEAD-BASED PAINT DISCLOSURE</span></h1>
        <p class="muted small" data-i18n="view.lead.hint.intro">
            The disclosure a landlord of pre-1978 "target housing" must give before a lease under federal
            law (42 U.S.C. § 4852d). It determines from the year built whether the disclosure is required,
            then assembles the lessor's disclosure, the lead-pamphlet acknowledgment, and the
            certifications. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.lead.h2.inputs">Disclosure details</h2>
            <form id="lead-form" class="inline-form">
                <label><span data-i18n="view.lead.label.landlord">Lessor (landlord)</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.lead.label.tenant">Lessee (tenant)</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.lead.label.premises">Premises address</span>
                    <input type="text" name="premises_address" value=""></label>
                <label><span data-i18n="view.lead.label.year">Year built</span>
                    <input type="number" step="1" min="1700" name="year_built" value="1965" required></label>
                <label><span data-i18n="view.lead.label.known">Known lead present</span>
                    <input type="checkbox" name="known_lead_present"></label>
                <label><span data-i18n="view.lead.label.details">Lead details (if known)</span>
                    <input type="text" name="lead_details" value=""></label>
                <label><span data-i18n="view.lead.label.records">Records available</span>
                    <input type="checkbox" name="records_available"></label>
                <label><span data-i18n="view.lead.label.records_desc">Records description</span>
                    <input type="text" name="records_description" value=""></label>
                <label><span data-i18n="view.lead.label.date">Disclosure date</span>
                    <input type="date" name="disclosure_date" value="2026-06-01" required></label>
            </form>
        </div>
        <div id="lead-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#lead-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            year_built: Math.round(Number(fd.get('year_built')) || 0),
            known_lead_present: fd.get('known_lead_present') != null,
            lead_details: (fd.get('lead_details') || '').trim(),
            records_available: fd.get('records_available') != null,
            records_description: (fd.get('records_description') || '').trim(),
            disclosure_date: fd.get('disclosure_date'),
        };
        try {
            const doc = await api.calcLeadPaintDisclosure(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.lead.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase(), ''];
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#lead-result');
    const reqKey = doc.disclosure_required ? 'view.lead.status.required' : 'view.lead.status.exempt';
    const reqCls = doc.disclosure_required ? 'neg' : 'pos';
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card ${reqCls}"><div class="label" data-i18n="view.lead.card.status">Disclosure</div>
                    <div class="value" data-i18n="${reqKey}"></div></div>
                <div class="card"><div class="label" data-i18n="view.lead.card.year">Year built</div>
                    <div class="value">${doc.year_built}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="lead-copy" type="button" data-i18n="view.lead.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="lead-download" type="button" data-i18n="view.lead.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#lead-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.lead.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.lead.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#lead-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'lead-paint-disclosure.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
