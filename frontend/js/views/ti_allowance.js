// TI allowance reconciliation generator — per-sqft allowance vs actual build-out
// cost → tenant overage or unused allowance, via /calc/ti-allowance.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderTiAllowance(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ti.h1.title">// TI ALLOWANCE RECONCILIATION</span></h1>
        <p class="muted small" data-i18n="view.ti.hint.intro">
            A landlord funds a tenant's build-out up to an allowance, usually quoted per rentable square foot.
            At completion the actual construction cost is reconciled against the allowance: an overrun is paid
            by the tenant, an underrun is forfeited or credited to rent. It computes the total allowance, the
            actual cost per square foot, and the overage or unused balance. Drafting aid, not legal/accounting advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ti.h2.inputs">Allowance inputs</h2>
            <form id="ti-form" class="inline-form">
                <label><span data-i18n="view.ti.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.ti.label.landlord">Landlord</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.ti.label.tenant">Tenant</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.ti.label.property">Premises</span>
                    <input type="text" name="property_label" value="Suite 200"></label>
                <label><span data-i18n="view.ti.label.sqft">Tenant sq ft</span>
                    <input type="number" step="100" min="0" name="tenant_sqft" value="5000" required></label>
                <label><span data-i18n="view.ti.label.psf">Allowance per sq ft ($)</span>
                    <input type="number" step="1" min="0" name="allowance_per_sqft_usd" value="50" required></label>
                <label><span data-i18n="view.ti.label.actual">Actual cost ($)</span>
                    <input type="number" step="1000" min="0" name="actual_cost_usd" value="300000" required></label>
                <label><span data-i18n="view.ti.label.treatment">Unused balance</span>
                    <select name="unused_treatment">
                        <option value="forfeited" data-i18n="view.ti.opt.forfeited">Forfeited</option>
                        <option value="credited" data-i18n="view.ti.opt.credited">Credited to rent</option>
                    </select></label>
                <label><span data-i18n="view.ti.label.date">Date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.ti.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.ti.ph.statute'))}"></label>
            </form>
        </div>
        <div id="ti-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ti-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            property_label: (fd.get('property_label') || '').trim(),
            tenant_sqft: Number(fd.get('tenant_sqft')) || 0,
            allowance_per_sqft_usd: Number(fd.get('allowance_per_sqft_usd')) || 0,
            actual_cost_usd: Number(fd.get('actual_cost_usd')) || 0,
            unused_treatment: fd.get('unused_treatment') || 'forfeited',
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcTiAllowance(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.ti.toast.error'), { level: 'error' });
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
    const balanceCard = doc.tenant_overage_usd > 0
        ? `<div class="card neg"><div class="label" data-i18n="view.ti.card.overage">Tenant overage</div>
               <div class="value">${money(doc.tenant_overage_usd)}</div></div>`
        : `<div class="card pos"><div class="label" data-i18n="view.ti.card.unused">Unused allowance</div>
               <div class="value">${money(doc.unused_allowance_usd)}</div></div>`;
    const el = mount.querySelector('#ti-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                ${balanceCard}
                <div class="card"><div class="label" data-i18n="view.ti.card.allowance">Total allowance</div>
                    <div class="value">${money(doc.total_allowance_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ti.card.psf">Actual $/sq ft</div>
                    <div class="value">${money(doc.actual_cost_per_sqft_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="ti-copy" type="button" data-i18n="view.ti.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="ti-download" type="button" data-i18n="view.ti.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#ti-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.ti.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.ti.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#ti-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'ti-allowance.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
