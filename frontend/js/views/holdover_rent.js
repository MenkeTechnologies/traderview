// Holdover rent generator — penalty-multiple daily rate over the holdover days
// plus the premium over ordinary rent, via /calc/holdover-rent.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderHoldoverRent(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.holdover.h1.title">// HOLDOVER RENT</span></h1>
        <p class="muted small" data-i18n="view.holdover.hint.intro">
            When a tenant stays past lease expiration without a renewal, the holdover clause charges rent at a
            penalty multiple of the daily base rate (commonly 150%–200%) for each holdover day. It computes the
            daily base rate, the holdover daily rate, the total holdover charge, and the premium over ordinary
            rent. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.holdover.h2.inputs">Holdover details</h2>
            <form id="holdover-form" class="inline-form">
                <label><span data-i18n="view.holdover.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.holdover.label.landlord">Landlord</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.holdover.label.tenant">Tenant</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.holdover.label.property">Premises</span>
                    <input type="text" name="property_label" value="Unit 4B"></label>
                <label><span data-i18n="view.holdover.label.rent">Monthly rent ($)</span>
                    <input type="number" step="100" min="0" name="monthly_rent_usd" value="5000" required></label>
                <label><span data-i18n="view.holdover.label.pct">Holdover rate (%)</span>
                    <input type="number" step="5" min="0" name="holdover_pct" value="150" required></label>
                <label><span data-i18n="view.holdover.label.days">Holdover days</span>
                    <input type="number" step="1" min="0" name="holdover_days" value="20" required></label>
                <label><span data-i18n="view.holdover.label.diy">Days per year</span>
                    <input type="number" step="1" min="1" name="days_in_year" value="365"></label>
                <label><span data-i18n="view.holdover.label.end">Lease end date</span>
                    <input type="date" name="lease_end_date" value="2026-06-30" required></label>
                <label><span data-i18n="view.holdover.label.date">Statement date</span>
                    <input type="date" name="date" value="2026-07-20" required></label>
                <label><span data-i18n="view.holdover.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.holdover.ph.statute'))}"></label>
            </form>
        </div>
        <div id="holdover-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#holdover-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            property_label: (fd.get('property_label') || '').trim(),
            monthly_rent_usd: Number(fd.get('monthly_rent_usd')) || 0,
            holdover_pct: Number(fd.get('holdover_pct')) || 0,
            holdover_days: Number(fd.get('holdover_days')) || 0,
            days_in_year: Number(fd.get('days_in_year')) || 365,
            lease_end_date: fd.get('lease_end_date'),
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcHoldoverRent(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.holdover.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#holdover-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.holdover.card.charge">Holdover charge</div>
                    <div class="value">${money(doc.holdover_charge_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.holdover.card.premium">Premium</div>
                    <div class="value">${money(doc.premium_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.holdover.card.holdover_daily">Holdover daily</div>
                    <div class="value">${money(doc.holdover_daily_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.holdover.card.daily_base">Daily base</div>
                    <div class="value">${money(doc.daily_base_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="holdover-copy" type="button" data-i18n="view.holdover.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="holdover-download" type="button" data-i18n="view.holdover.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#holdover-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.holdover.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.holdover.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#holdover-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'holdover-rent.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
