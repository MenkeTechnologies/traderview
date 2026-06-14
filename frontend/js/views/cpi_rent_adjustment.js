// CPI rent adjustment generator — index-ratio escalation bounded by a
// floor/ceiling collar, via /calc/cpi-rent-adjustment.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 }) + '%');
let LAST_DOC = null;

export async function renderCpiRentAdjustment(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cpirent.h1.title">// CPI RENT ADJUSTMENT</span></h1>
        <p class="muted small" data-i18n="view.cpirent.hint.intro">
            The index-based escalation used in commercial leases. Rent is reset by the ratio of a current
            consumer price index to a base-period index, bounded by an optional collar (a floor and/or ceiling
            on the percentage increase). Unlike fixed-percentage escalation, the increase tracks published CPI.
            Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.cpirent.h2.inputs">Adjustment inputs</h2>
            <form id="cpirent-form" class="inline-form">
                <label><span data-i18n="view.cpirent.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.cpirent.label.landlord">Landlord</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.cpirent.label.tenant">Tenant</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.cpirent.label.property">Premises</span>
                    <input type="text" name="property_label" value="Floor 7"></label>
                <label><span data-i18n="view.cpirent.label.base">Current annual rent ($)</span>
                    <input type="number" step="1000" min="0" name="base_rent_usd" value="50000" required></label>
                <label><span data-i18n="view.cpirent.label.cpi_base">Base CPI</span>
                    <input type="number" step="0.1" min="0" name="cpi_base" value="280.0" required></label>
                <label><span data-i18n="view.cpirent.label.cpi_current">Current CPI</span>
                    <input type="number" step="0.1" min="0" name="cpi_current" value="295.4" required></label>
                <label><span data-i18n="view.cpirent.label.min">Floor on increase (%, 0 = none)</span>
                    <input type="number" step="0.1" min="0" name="min_increase_pct" value="2"></label>
                <label><span data-i18n="view.cpirent.label.max">Ceiling on increase (%, 0 = none)</span>
                    <input type="number" step="0.1" min="0" name="max_increase_pct" value="5"></label>
                <label><span data-i18n="view.cpirent.label.index">Index name</span>
                    <input type="text" name="index_label" value="CPI-U, U.S. city average"></label>
                <label><span data-i18n="view.cpirent.label.date">Effective date</span>
                    <input type="date" name="effective_date" value="2026-07-01" required></label>
                <label><span data-i18n="view.cpirent.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.cpirent.ph.statute'))}"></label>
            </form>
        </div>
        <div id="cpirent-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#cpirent-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            property_label: (fd.get('property_label') || '').trim(),
            base_rent_usd: Number(fd.get('base_rent_usd')) || 0,
            cpi_base: Number(fd.get('cpi_base')) || 0,
            cpi_current: Number(fd.get('cpi_current')) || 0,
            min_increase_pct: Number(fd.get('min_increase_pct')) || 0,
            max_increase_pct: Number(fd.get('max_increase_pct')) || 0,
            index_label: (fd.get('index_label') || '').trim(),
            effective_date: fd.get('effective_date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcCpiRentAdjustment(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.cpirent.toast.error'), { level: 'error' });
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
    let bound = '';
    if (doc.ceiling_applied) bound = ` <span class="muted small" data-i18n="view.cpirent.ceiling">(ceiling)</span>`;
    else if (doc.floor_applied) bound = ` <span class="muted small" data-i18n="view.cpirent.floor">(floor)</span>`;
    const el = mount.querySelector('#cpirent-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.cpirent.card.new">New annual rent</div>
                    <div class="value">${money(doc.new_rent_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.cpirent.card.increase">Increase</div>
                    <div class="value">${money(doc.dollar_increase_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.cpirent.card.applied">Applied increase</div>
                    <div class="value">${pct(doc.applied_increase_pct)}${bound}</div></div>
                <div class="card"><div class="label" data-i18n="view.cpirent.card.raw">Raw index change</div>
                    <div class="value">${pct(doc.raw_increase_pct)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="cpirent-copy" type="button" data-i18n="view.cpirent.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="cpirent-download" type="button" data-i18n="view.cpirent.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#cpirent-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.cpirent.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.cpirent.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#cpirent-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'cpi-rent-adjustment.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
