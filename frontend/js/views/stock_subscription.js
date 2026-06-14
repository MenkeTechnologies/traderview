// Stock subscription agreement generator — investment + resulting ownership %,
// via /calc/stock-subscription. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

export async function renderStockSubscription(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ss.h1.title">// STOCK SUBSCRIPTION</span></h1>
        <p class="muted small" data-i18n="view.ss.hint.intro">
            An investor subscribes for newly-issued shares of a corporation. It computes the total
            investment (shares × price per share) and the investor's resulting ownership percentage of the
            shares outstanding after the issuance. Distinct from the LLC operating agreement. Drafting aid,
            not legal/securities advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ss.h2.inputs">Subscription details</h2>
            <form id="ss-form" class="inline-form">
                <label><span data-i18n="view.ss.label.state">State of incorporation</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.ss.label.company">Company</span>
                    <input type="text" name="company_name" value=""></label>
                <label><span data-i18n="view.ss.label.investor">Investor</span>
                    <input type="text" name="investor_name" value=""></label>
                <label><span data-i18n="view.ss.label.shares">Shares purchased</span>
                    <input type="number" step="1" min="0" name="shares_purchased" value="100000" required></label>
                <label><span data-i18n="view.ss.label.price">Price per share ($)</span>
                    <input type="number" step="0.0001" min="0" name="price_per_share_usd" value="1.00" required></label>
                <label><span data-i18n="view.ss.label.par">Par value ($)</span>
                    <input type="number" step="0.0001" min="0" name="par_value_usd" value="0.0001"></label>
                <label><span data-i18n="view.ss.label.total">Total shares after issuance</span>
                    <input type="number" step="1" min="1" name="total_shares_after" value="1000000" required></label>
                <label><span data-i18n="view.ss.label.closing">Closing date</span>
                    <input type="date" name="closing_date" value="2026-07-15" required></label>
                <label><span data-i18n="view.ss.label.accredited">Accredited investor</span>
                    <input type="checkbox" name="accredited" checked></label>
                <label><span data-i18n="view.ss.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.ss.ph.statute'))}"></label>
            </form>
        </div>
        <div id="ss-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ss-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            investor_name: (fd.get('investor_name') || '').trim(),
            shares_purchased: Number(fd.get('shares_purchased')) || 0,
            price_per_share_usd: Number(fd.get('price_per_share_usd')) || 0,
            par_value_usd: Number(fd.get('par_value_usd')) || 0,
            total_shares_after: Number(fd.get('total_shares_after')) || 0,
            closing_date: fd.get('closing_date'),
            accredited: fd.get('accredited') != null,
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcStockSubscription(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.ss.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#ss-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.ss.card.investment">Total investment</div>
                    <div class="value">${money(doc.total_investment_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ss.card.ownership">Ownership</div>
                    <div class="value">${pct(doc.ownership_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ss.card.price">Price / share</div>
                    <div class="value">${money(doc.price_per_share_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="ss-copy" type="button" data-i18n="view.ss.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="ss-download" type="button" data-i18n="view.ss.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#ss-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.ss.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.ss.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#ss-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'stock-subscription.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
