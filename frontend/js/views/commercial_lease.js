// Commercial (NNN) lease generator — base + triple-net charges, gross monthly
// rent, lease end date, via /calc/commercial-lease. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderCommercialLease(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cl.h1.title">// COMMERCIAL LEASE (NNN)</span></h1>
        <p class="muted small" data-i18n="view.cl.hint.intro">
            Leases business space where the tenant pays base rent plus its share of operating costs
            (common-area maintenance, property tax, insurance). It computes the base and NNN charges from
            the per-square-foot rates and the area, the gross monthly rent, and the lease end date.
            Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.cl.h2.inputs">Lease details</h2>
            <form id="cl-form" class="inline-form">
                <label><span data-i18n="view.cl.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="Texas" required></label>
                <label><span data-i18n="view.cl.label.landlord">Landlord</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.cl.label.tenant">Tenant (business)</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.cl.label.premises">Premises address</span>
                    <input type="text" name="premises_address" value=""></label>
                <label><span data-i18n="view.cl.label.sqft">Square feet</span>
                    <input type="number" step="1" min="0" name="square_feet" value="2000" required></label>
                <label><span data-i18n="view.cl.label.base">Base rent ($/sq ft/yr)</span>
                    <input type="number" step="0.01" min="0" name="base_rent_psf_annual" value="30" required></label>
                <label><span data-i18n="view.cl.label.cam">CAM ($/sq ft/yr)</span>
                    <input type="number" step="0.01" min="0" name="cam_psf_annual" value="5"></label>
                <label><span data-i18n="view.cl.label.tax">Property tax ($/sq ft/yr)</span>
                    <input type="number" step="0.01" min="0" name="property_tax_psf_annual" value="3"></label>
                <label><span data-i18n="view.cl.label.ins">Insurance ($/sq ft/yr)</span>
                    <input type="number" step="0.01" min="0" name="insurance_psf_annual" value="2"></label>
                <label><span data-i18n="view.cl.label.start">Lease start date</span>
                    <input type="date" name="lease_start_date" value="2026-08-01" required></label>
                <label><span data-i18n="view.cl.label.term">Term (months)</span>
                    <input type="number" step="1" min="1" name="term_months" value="60" required></label>
                <label><span data-i18n="view.cl.label.use">Permitted use</span>
                    <input type="text" name="permitted_use" value="Retail store"></label>
                <label><span data-i18n="view.cl.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.cl.ph.statute'))}"></label>
            </form>
        </div>
        <div id="cl-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#cl-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            square_feet: Number(fd.get('square_feet')) || 0,
            base_rent_psf_annual: Number(fd.get('base_rent_psf_annual')) || 0,
            cam_psf_annual: Number(fd.get('cam_psf_annual')) || 0,
            property_tax_psf_annual: Number(fd.get('property_tax_psf_annual')) || 0,
            insurance_psf_annual: Number(fd.get('insurance_psf_annual')) || 0,
            lease_start_date: fd.get('lease_start_date'),
            term_months: Math.round(Number(fd.get('term_months')) || 0),
            permitted_use: (fd.get('permitted_use') || '').trim(),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcCommercialLease(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.cl.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#cl-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.cl.card.gross">Gross rent / mo</div>
                    <div class="value">${money(doc.gross_monthly_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.cl.card.base">Base / mo</div>
                    <div class="value">${money(doc.base_monthly_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.cl.card.nnn">NNN / mo</div>
                    <div class="value">${money(doc.nnn_monthly_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.cl.card.end">Lease ends</div>
                    <div class="value">${esc(doc.lease_end_date || '—')}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="cl-copy" type="button" data-i18n="view.cl.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="cl-download" type="button" data-i18n="view.cl.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#cl-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.cl.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.cl.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#cl-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'commercial-lease.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
