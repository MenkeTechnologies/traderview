// Operating-expense escalation generator — base-year stop + occupancy gross-up,
// pro-rata tenant share, via /calc/opex-escalation.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
const fac = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 }));
let LAST_DOC = null;

export async function renderOpexEscalation(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.opex.h1.title">// OPERATING EXPENSE ESCALATION</span></h1>
        <p class="muted small" data-i18n="view.opex.hint.intro">
            The expense pass-through of a full-service / gross commercial lease. Unlike CAM (full pro-rata),
            a base-year lease passes through only the tenant's share of expenses above a base-year stop. It
            also grosses up occupancy-variable expenses to a target occupancy so a partly-vacant year doesn't
            understate the stop. Drafting aid, not legal/accounting advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.opex.h2.inputs">Escalation inputs</h2>
            <form id="opex-form" class="inline-form">
                <label><span data-i18n="view.opex.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.opex.label.landlord">Landlord</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.opex.label.tenant">Tenant</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.opex.label.property">Premises</span>
                    <input type="text" name="property_label" value="Suite 500"></label>
                <label><span data-i18n="view.opex.label.tenant_sqft">Tenant sq ft</span>
                    <input type="number" step="100" min="0" name="tenant_sqft" value="5000" required></label>
                <label><span data-i18n="view.opex.label.building_sqft">Building sq ft</span>
                    <input type="number" step="100" min="0" name="building_sqft" value="50000" required></label>
                <label><span data-i18n="view.opex.label.base">Base-year expenses ($)</span>
                    <input type="number" step="1000" min="0" name="base_year_opex_usd" value="500000" required></label>
                <label><span data-i18n="view.opex.label.current">Current-year expenses ($)</span>
                    <input type="number" step="1000" min="0" name="current_opex_usd" value="600000" required></label>
                <label><span data-i18n="view.opex.label.variable">Variable portion (%)</span>
                    <input type="number" step="1" min="0" max="100" name="variable_pct" value="60"></label>
                <label><span data-i18n="view.opex.label.actual_occ">Actual occupancy (%)</span>
                    <input type="number" step="1" min="0" max="100" name="actual_occupancy_pct" value="80"></label>
                <label><span data-i18n="view.opex.label.target_occ">Gross-up occupancy (%)</span>
                    <input type="number" step="1" min="0" max="100" name="gross_up_occupancy_pct" value="95"></label>
                <label><span data-i18n="view.opex.label.year">Expense year</span>
                    <input type="text" name="year" value="2025" required></label>
                <label><span data-i18n="view.opex.label.date">Statement date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.opex.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.opex.ph.statute'))}"></label>
            </form>
        </div>
        <div id="opex-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#opex-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            property_label: (fd.get('property_label') || '').trim(),
            tenant_sqft: Number(fd.get('tenant_sqft')) || 0,
            building_sqft: Number(fd.get('building_sqft')) || 0,
            base_year_opex_usd: Number(fd.get('base_year_opex_usd')) || 0,
            current_opex_usd: Number(fd.get('current_opex_usd')) || 0,
            variable_pct: Number(fd.get('variable_pct')) || 0,
            actual_occupancy_pct: Number(fd.get('actual_occupancy_pct')) || 0,
            gross_up_occupancy_pct: Number(fd.get('gross_up_occupancy_pct')) || 0,
            year: (fd.get('year') || '').trim(),
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcOpexEscalation(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.opex.toast.error'), { level: 'error' });
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
    const guCard = doc.gross_up_factor > 1.0
        ? `<div class="card"><div class="label" data-i18n="view.opex.card.factor">Gross-up factor</div>
               <div class="value">${fac(doc.gross_up_factor)}</div></div>`
        : '';
    const el = mount.querySelector('#opex-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.opex.card.share">Tenant share</div>
                    <div class="value">${money(doc.tenant_share_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.opex.card.increment">Over base year</div>
                    <div class="value">${money(doc.increment_over_base_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.opex.card.grossed">Grossed-up expenses</div>
                    <div class="value">${money(doc.grossed_up_opex_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.opex.card.prorata">Pro-rata share</div>
                    <div class="value">${pct(doc.pro_rata_pct)}</div></div>
                ${guCard}
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="opex-copy" type="button" data-i18n="view.opex.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="opex-download" type="button" data-i18n="view.opex.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#opex-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.opex.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.opex.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#opex-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'opex-escalation.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
