// Lease buyout / early-termination settlement generator — PV of remaining rent
// + concessions + fee − reletting recovery, via /calc/lease-buyout.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderLeaseBuyout(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.buyout.h1.title">// LEASE BUYOUT SETTLEMENT</span></h1>
        <p class="muted small" data-i18n="view.buyout.hint.intro">
            The financial settlement a tenant pays to be released from a lease early. It computes the present
            value of the remaining base-rent stream, plus unamortized concessions (TI and commissions not yet
            earned back) and a termination fee, less the landlord's expected reletting recovery. Drafting aid,
            not legal/financial advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.buyout.h2.inputs">Buyout inputs</h2>
            <form id="buyout-form" class="inline-form">
                <label><span data-i18n="view.buyout.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.buyout.label.landlord">Landlord</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.buyout.label.tenant">Tenant</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.buyout.label.property">Premises</span>
                    <input type="text" name="property_label" value="Suite 900"></label>
                <label><span data-i18n="view.buyout.label.rent">Monthly rent ($)</span>
                    <input type="number" step="100" min="0" name="monthly_rent_usd" value="10000" required></label>
                <label><span data-i18n="view.buyout.label.months">Months remaining</span>
                    <input type="number" step="1" min="0" name="remaining_months" value="24" required></label>
                <label><span data-i18n="view.buyout.label.discount">Discount rate (%/yr)</span>
                    <input type="number" step="0.1" min="0" name="annual_discount_pct" value="6"></label>
                <label><span data-i18n="view.buyout.label.concessions">Unamortized concessions ($)</span>
                    <input type="number" step="1000" min="0" name="unamortized_concessions_usd" value="20000"></label>
                <label><span data-i18n="view.buyout.label.fee">Termination fee (months)</span>
                    <input type="number" step="0.5" min="0" name="termination_fee_months" value="3"></label>
                <label><span data-i18n="view.buyout.label.reletting">Reletting recovery ($)</span>
                    <input type="number" step="1000" min="0" name="reletting_recovery_usd" value="40000"></label>
                <label><span data-i18n="view.buyout.label.date">Date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.buyout.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.buyout.ph.statute'))}"></label>
            </form>
        </div>
        <div id="buyout-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#buyout-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            property_label: (fd.get('property_label') || '').trim(),
            monthly_rent_usd: Number(fd.get('monthly_rent_usd')) || 0,
            remaining_months: Number(fd.get('remaining_months')) || 0,
            annual_discount_pct: Number(fd.get('annual_discount_pct')) || 0,
            unamortized_concessions_usd: Number(fd.get('unamortized_concessions_usd')) || 0,
            termination_fee_months: Number(fd.get('termination_fee_months')) || 0,
            reletting_recovery_usd: Number(fd.get('reletting_recovery_usd')) || 0,
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcLeaseBuyout(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.buyout.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#buyout-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.buyout.card.settlement">Buyout settlement</div>
                    <div class="value">${money(doc.settlement_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.buyout.card.pv">PV remaining rent</div>
                    <div class="value">${money(doc.pv_remaining_rent_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.buyout.card.fee">Termination fee</div>
                    <div class="value">${money(doc.termination_fee_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.buyout.card.concessions">Concessions</div>
                    <div class="value">${money(doc.unamortized_concessions_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.buyout.card.reletting">Less reletting</div>
                    <div class="value">${money(doc.reletting_recovery_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="buyout-copy" type="button" data-i18n="view.buyout.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="buyout-download" type="button" data-i18n="view.buyout.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#buyout-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.buyout.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.buyout.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#buyout-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'lease-buyout.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
