// CAM reconciliation generator — commercial-lease common-area-maintenance
// true-up: pro-rata-by-sqft share of actual CAM vs estimates, via
// /calc/cam-reconciliation.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

export async function renderCamReconciliation(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.camrec.h1.title">// CAM RECONCILIATION</span></h1>
        <p class="muted small" data-i18n="view.camrec.hint.intro">
            The annual common-area-maintenance true-up under a commercial lease. The tenant pays monthly CAM
            estimates; at year end the tenant's pro-rata share (by rentable square footage) of actual CAM is
            reconciled against what was paid, and the difference is billed or credited. An optional cap limits
            the increase over the prior year. Drafting aid, not legal/accounting advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.camrec.h2.inputs">Reconciliation inputs</h2>
            <form id="cam-form" class="inline-form">
                <label><span data-i18n="view.camrec.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.camrec.label.landlord">Landlord</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.camrec.label.tenant">Tenant</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.camrec.label.property">Premises</span>
                    <input type="text" name="property_label" value="Suite 100"></label>
                <label><span data-i18n="view.camrec.label.tenant_sqft">Tenant sq ft</span>
                    <input type="number" step="100" min="0" name="tenant_sqft" value="5000" required></label>
                <label><span data-i18n="view.camrec.label.building_sqft">Building sq ft</span>
                    <input type="number" step="100" min="0" name="building_sqft" value="50000" required></label>
                <label><span data-i18n="view.camrec.label.actual">Actual CAM ($)</span>
                    <input type="number" step="1000" min="0" name="actual_cam_usd" value="400000" required></label>
                <label><span data-i18n="view.camrec.label.estimate">Monthly estimate ($)</span>
                    <input type="number" step="100" min="0" name="monthly_estimate_usd" value="3000" required></label>
                <label><span data-i18n="view.camrec.label.months">Months</span>
                    <input type="number" step="1" min="1" name="months" value="12" required></label>
                <label><span data-i18n="view.camrec.label.cap">Cap on increase (%, 0 = none)</span>
                    <input type="number" step="0.1" min="0" name="cap_pct" value="0"></label>
                <label><span data-i18n="view.camrec.label.prior">Prior-year share ($)</span>
                    <input type="number" step="1000" min="0" name="prior_year_share_usd" value="0"></label>
                <label><span data-i18n="view.camrec.label.year">Reconciliation year</span>
                    <input type="text" name="year" value="2025" required></label>
                <label><span data-i18n="view.camrec.label.date">Statement date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.camrec.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.camrec.ph.statute'))}"></label>
            </form>
        </div>
        <div id="cam-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#cam-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            property_label: (fd.get('property_label') || '').trim(),
            tenant_sqft: Number(fd.get('tenant_sqft')) || 0,
            building_sqft: Number(fd.get('building_sqft')) || 0,
            actual_cam_usd: Number(fd.get('actual_cam_usd')) || 0,
            monthly_estimate_usd: Number(fd.get('monthly_estimate_usd')) || 0,
            months: Number(fd.get('months')) || 0,
            cap_pct: Number(fd.get('cap_pct')) || 0,
            prior_year_share_usd: Number(fd.get('prior_year_share_usd')) || 0,
            year: (fd.get('year') || '').trim(),
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcCamReconciliation(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.camrec.toast.error'), { level: 'error' });
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
    const owes = doc.balance_usd >= 0;
    const balLabel = owes ? t('view.camrec.card.owes') : t('view.camrec.card.credit');
    const capCard = doc.cap_applied
        ? `<div class="card"><div class="label" data-i18n="view.camrec.card.uncapped">Uncapped share</div>
               <div class="value">${money(doc.tenant_share_uncapped_usd)}</div></div>`
        : '';
    const el = mount.querySelector('#cam-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card ${owes ? 'neg' : 'pos'}"><div class="label">${esc(balLabel)}</div>
                    <div class="value">${money(Math.abs(doc.balance_usd))}</div></div>
                <div class="card"><div class="label" data-i18n="view.camrec.card.share">Pro-rata share</div>
                    <div class="value">${pct(doc.pro_rata_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.camrec.card.tenant_share">Tenant CAM share</div>
                    <div class="value">${money(doc.tenant_share_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.camrec.card.paid">Estimates paid</div>
                    <div class="value">${money(doc.estimates_paid_usd)}</div></div>
                ${capCard}
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="cam-copy" type="button" data-i18n="view.camrec.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="cam-download" type="button" data-i18n="view.camrec.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#cam-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.camrec.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.camrec.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#cam-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'cam-reconciliation.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
