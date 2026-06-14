// Prorated rent generator — partial-month rent on actual calendar-month days for
// a move-in or move-out, via /calc/prorated-rent.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderProratedRent(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.prorate.h1.title">// PRORATED RENT</span></h1>
        <p class="muted small" data-i18n="view.prorate.hint.intro">
            The partial-month rent owed when a tenant moves in or out mid-month. Rent is prorated on the
            actual number of days in that calendar month (so February, 30-day, and 31-day months each compute
            correctly), counting the days the tenant occupies the unit. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.prorate.h2.inputs">Proration inputs</h2>
            <form id="prorate-form" class="inline-form">
                <label><span data-i18n="view.prorate.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.prorate.label.landlord">Landlord</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.prorate.label.tenant">Tenant</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.prorate.label.property">Premises</span>
                    <input type="text" name="property_label" value="Unit 4B"></label>
                <label><span data-i18n="view.prorate.label.rent">Monthly rent ($)</span>
                    <input type="number" step="50" min="0" name="monthly_rent_usd" value="3000" required></label>
                <label><span data-i18n="view.prorate.label.mode">Event</span>
                    <select name="mode">
                        <option value="move_in" data-i18n="view.prorate.opt.movein">Move-in</option>
                        <option value="move_out" data-i18n="view.prorate.opt.moveout">Move-out</option>
                    </select></label>
                <label><span data-i18n="view.prorate.label.date">Event date</span>
                    <input type="date" name="event_date" value="2026-06-15" required></label>
                <label><span data-i18n="view.prorate.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.prorate.ph.statute'))}"></label>
            </form>
        </div>
        <div id="prorate-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#prorate-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            property_label: (fd.get('property_label') || '').trim(),
            monthly_rent_usd: Number(fd.get('monthly_rent_usd')) || 0,
            mode: fd.get('mode') || 'move_in',
            event_date: fd.get('event_date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcProratedRent(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.prorate.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#prorate-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.prorate.card.prorated">Prorated rent</div>
                    <div class="value">${money(doc.prorated_rent_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.prorate.card.occupied">Occupied days</div>
                    <div class="value">${doc.occupied_days} / ${doc.days_in_month}</div></div>
                <div class="card"><div class="label" data-i18n="view.prorate.card.daily">Daily rate</div>
                    <div class="value">${money(doc.daily_rate_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="prorate-copy" type="button" data-i18n="view.prorate.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="prorate-download" type="button" data-i18n="view.prorate.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#prorate-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.prorate.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.prorate.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#prorate-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'prorated-rent.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
