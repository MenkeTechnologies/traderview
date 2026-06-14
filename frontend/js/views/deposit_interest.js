// Security-deposit interest generator — simple or annually-compounded interest
// owed to the tenant over the tenancy, via /calc/deposit-interest.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderDepositInterest(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.depint.h1.title">// SECURITY DEPOSIT INTEREST</span></h1>
        <p class="muted small" data-i18n="view.depint.hint.intro">
            Many jurisdictions require a landlord to pay the tenant interest on a held security deposit. This
            computes that interest over the tenancy at the applicable rate, simple or annually compounded, and
            the total returnable (deposit + interest). Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.depint.h2.inputs">Deposit details</h2>
            <form id="depint-form" class="inline-form">
                <label><span data-i18n="view.depint.label.state">State</span>
                    <input type="text" name="state" value="Illinois" required></label>
                <label><span data-i18n="view.depint.label.landlord">Landlord</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.depint.label.tenant">Tenant</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.depint.label.property">Premises</span>
                    <input type="text" name="property_label" value="Unit 4B"></label>
                <label><span data-i18n="view.depint.label.deposit">Deposit held ($)</span>
                    <input type="number" step="50" min="0" name="deposit_usd" value="2000" required></label>
                <label><span data-i18n="view.depint.label.rate">Annual rate (%)</span>
                    <input type="number" step="0.01" min="0" name="annual_rate_pct" value="1.5" required></label>
                <label><span data-i18n="view.depint.label.term">Tenancy (months)</span>
                    <input type="number" step="1" min="0" name="term_months" value="36" required></label>
                <label><span data-i18n="view.depint.label.compounding">Method</span>
                    <select name="compounding">
                        <option value="simple" data-i18n="view.depint.opt.simple">Simple</option>
                        <option value="annual" data-i18n="view.depint.opt.annual">Annual compound</option>
                    </select></label>
                <label><span data-i18n="view.depint.label.date">Statement date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.depint.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.depint.ph.statute'))}"></label>
            </form>
        </div>
        <div id="depint-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#depint-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            property_label: (fd.get('property_label') || '').trim(),
            deposit_usd: Number(fd.get('deposit_usd')) || 0,
            annual_rate_pct: Number(fd.get('annual_rate_pct')) || 0,
            term_months: Number(fd.get('term_months')) || 0,
            compounding: fd.get('compounding') || 'simple',
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcDepositInterest(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.depint.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#depint-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.depint.card.interest">Interest owed</div>
                    <div class="value">${money(doc.interest_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.depint.card.deposit">Deposit</div>
                    <div class="value">${money(doc.deposit_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.depint.card.total">Total returnable</div>
                    <div class="value">${money(doc.total_returnable_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="depint-copy" type="button" data-i18n="view.depint.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="depint-download" type="button" data-i18n="view.depint.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#depint-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.depint.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.depint.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#depint-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'deposit-interest.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
