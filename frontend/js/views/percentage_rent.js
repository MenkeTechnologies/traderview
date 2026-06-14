// Percentage-rent statement generator — retail-lease overage: base + rate ×
// sales over the (natural or stated) breakpoint, via /calc/percentage-rent.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

export async function renderPercentageRent(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pctrent.h1.title">// PERCENTAGE RENT</span></h1>
        <p class="muted small" data-i18n="view.pctrent.hint.intro">
            The retail-lease overage calculation. A tenant pays fixed base rent plus a percentage of gross
            sales above a breakpoint — natural (base rent ÷ rate) unless the lease states one. It computes the
            natural breakpoint, the overage rent, total rent, and the occupancy-cost ratio (total rent ÷
            sales). Drafting aid, not legal/accounting advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.pctrent.h2.inputs">Lease terms</h2>
            <form id="pctrent-form" class="inline-form">
                <label><span data-i18n="view.pctrent.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.pctrent.label.landlord">Landlord</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.pctrent.label.tenant">Tenant</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.pctrent.label.property">Premises</span>
                    <input type="text" name="property_label" value="Store 12"></label>
                <label><span data-i18n="view.pctrent.label.base">Annual base rent ($)</span>
                    <input type="number" step="1000" min="0" name="base_rent_usd" value="120000" required></label>
                <label><span data-i18n="view.pctrent.label.rate">Percentage rate (%)</span>
                    <input type="number" step="0.1" min="0" name="rate_pct" value="6" required></label>
                <label><span data-i18n="view.pctrent.label.sales">Gross sales ($)</span>
                    <input type="number" step="10000" min="0" name="gross_sales_usd" value="3000000" required></label>
                <label><span data-i18n="view.pctrent.label.breakpoint">Stated breakpoint ($, 0 = natural)</span>
                    <input type="number" step="10000" min="0" name="stated_breakpoint_usd" value="0"></label>
                <label><span data-i18n="view.pctrent.label.period">Period (months)</span>
                    <input type="number" step="1" min="1" name="period_months" value="12" required></label>
                <label><span data-i18n="view.pctrent.label.date">Statement date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.pctrent.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.pctrent.ph.statute'))}"></label>
            </form>
        </div>
        <div id="pctrent-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#pctrent-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            property_label: (fd.get('property_label') || '').trim(),
            base_rent_usd: Number(fd.get('base_rent_usd')) || 0,
            rate_pct: Number(fd.get('rate_pct')) || 0,
            gross_sales_usd: Number(fd.get('gross_sales_usd')) || 0,
            stated_breakpoint_usd: Number(fd.get('stated_breakpoint_usd')) || 0,
            period_months: Number(fd.get('period_months')) || 0,
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcPercentageRent(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.pctrent.toast.error'), { level: 'error' });
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
    const bpLabel = doc.stated_breakpoint_applied ? t('view.pctrent.card.bp_stated') : t('view.pctrent.card.bp_natural');
    const el = mount.querySelector('#pctrent-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.pctrent.card.total">Total rent</div>
                    <div class="value">${money(doc.total_rent_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pctrent.card.overage">Overage rent</div>
                    <div class="value">${money(doc.overage_rent_usd)}</div></div>
                <div class="card"><div class="label">${esc(bpLabel)}</div>
                    <div class="value">${money(doc.breakpoint_used_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pctrent.card.occupancy">Occupancy cost</div>
                    <div class="value">${pct(doc.occupancy_cost_pct)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="pctrent-copy" type="button" data-i18n="view.pctrent.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="pctrent-download" type="button" data-i18n="view.pctrent.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#pctrent-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.pctrent.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.pctrent.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#pctrent-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'percentage-rent.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
