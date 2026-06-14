// SAFE generator — discount/cap conversion + shares (no interest, no maturity),
// via /calc/safe. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 }));
let LAST_DOC = null;

export async function renderSafe(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.safe.h1.title">// SAFE (FUTURE EQUITY)</span></h1>
        <p class="muted small" data-i18n="view.safe.hint.intro">
            A startup investment that, unlike a convertible note, is NOT debt — no interest and no
            maturity date. The investment converts to equity at the next priced round at the better of the
            discount price and the valuation-cap price. It computes the conversion price and the shares
            the investment buys. Drafting aid, not legal/securities advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.safe.h2.inputs">SAFE terms</h2>
            <form id="safe-form" class="inline-form">
                <label><span data-i18n="view.safe.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.safe.label.company">Company</span>
                    <input type="text" name="company_name" value=""></label>
                <label><span data-i18n="view.safe.label.investor">Investor</span>
                    <input type="text" name="investor_name" value=""></label>
                <label><span data-i18n="view.safe.label.investment">Investment ($)</span>
                    <input type="number" step="1000" min="0" name="investment_usd" value="100000" required></label>
                <label><span data-i18n="view.safe.label.discount">Discount (%)</span>
                    <input type="number" step="0.1" min="0" name="discount_pct" value="20"></label>
                <label><span data-i18n="view.safe.label.cap">Valuation cap ($)</span>
                    <input type="number" step="100000" min="0" name="valuation_cap_usd" value="5000000"></label>
                <label><span data-i18n="view.safe.label.roundprice">Assumed round price/share ($)</span>
                    <input type="number" step="0.01" min="0" name="assumed_round_price_per_share_usd" value="2.00"></label>
                <label><span data-i18n="view.safe.label.premoney">Assumed round pre-money ($)</span>
                    <input type="number" step="100000" min="0" name="assumed_round_pre_money_usd" value="10000000"></label>
                <label><span data-i18n="view.safe.label.date">Date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.safe.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.safe.ph.statute'))}"></label>
            </form>
        </div>
        <div id="safe-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#safe-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            investor_name: (fd.get('investor_name') || '').trim(),
            investment_usd: Number(fd.get('investment_usd')) || 0,
            discount_pct: Number(fd.get('discount_pct')) || 0,
            valuation_cap_usd: Number(fd.get('valuation_cap_usd')) || 0,
            assumed_round_price_per_share_usd: Number(fd.get('assumed_round_price_per_share_usd')) || 0,
            assumed_round_pre_money_usd: Number(fd.get('assumed_round_pre_money_usd')) || 0,
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcSafe(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.safe.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#safe-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.safe.card.conversion">Conversion price</div>
                    <div class="value">${money(doc.conversion_price_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.safe.card.shares">Shares on conversion</div>
                    <div class="value">${num(doc.shares_on_conversion)}</div></div>
                <div class="card"><div class="label" data-i18n="view.safe.card.investment">Investment</div>
                    <div class="value">${money(doc.investment_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="safe-copy" type="button" data-i18n="view.safe.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="safe-download" type="button" data-i18n="view.safe.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#safe-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.safe.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.safe.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#safe-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'safe.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
