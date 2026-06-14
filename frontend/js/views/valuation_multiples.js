// Valuation multiples — P/E, P/B, P/S, PEG, and dividend/earnings/FCF yields,
// via /calc/valuation-multiples.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

export async function renderValuationMultiples(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.vm.h1.title">// VALUATION MULTIPLES</span></h1>
        <p class="muted small" data-i18n="view.vm.hint.intro">
            The standard per-share market multiples and yields: P/E, P/B, P/S, PEG (P/E ÷ growth), and the
            dividend, earnings, and free-cash-flow yields. Complements Graham number, EV/EBITDA, and EVA.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.vm.h2.inputs">Per-share inputs</h2>
            <form id="vm-form" class="inline-form">
                <label><span data-i18n="view.vm.label.ticker">Ticker</span>
                    <input type="text" name="ticker" value="ACME"></label>
                <label><span data-i18n="view.vm.label.price">Price ($)</span>
                    <input type="number" step="0.01" min="0" name="price_usd" value="100" required></label>
                <label><span data-i18n="view.vm.label.eps">EPS ($)</span>
                    <input type="number" step="0.01" name="eps_usd" value="5"></label>
                <label><span data-i18n="view.vm.label.bvps">Book value/share ($)</span>
                    <input type="number" step="0.01" name="book_value_per_share_usd" value="25"></label>
                <label><span data-i18n="view.vm.label.sps">Sales/share ($)</span>
                    <input type="number" step="0.01" name="sales_per_share_usd" value="50"></label>
                <label><span data-i18n="view.vm.label.dps">Dividend/share ($)</span>
                    <input type="number" step="0.01" name="dividend_per_share_usd" value="2"></label>
                <label><span data-i18n="view.vm.label.fcf">FCF/share ($)</span>
                    <input type="number" step="0.01" name="fcf_per_share_usd" value="6"></label>
                <label><span data-i18n="view.vm.label.growth">EPS growth (%)</span>
                    <input type="number" step="0.5" name="eps_growth_pct" value="15"></label>
            </form>
        </div>
        <div id="vm-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#vm-form');
    const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const generate = async () => {
        const body = {
            ticker: (form.querySelector('[name="ticker"]').value || '').trim(),
            price_usd: n('price_usd'),
            eps_usd: n('eps_usd'),
            book_value_per_share_usd: n('book_value_per_share_usd'),
            sales_per_share_usd: n('sales_per_share_usd'),
            dividend_per_share_usd: n('dividend_per_share_usd'),
            fcf_per_share_usd: n('fcf_per_share_usd'),
            eps_growth_pct: n('eps_growth_pct'),
        };
        try {
            const doc = await api.calcValuationMultiples(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.vm.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const cell = (lbl, val) => `<div class="card"><div class="label">${esc(lbl)}</div><div class="value">${val}</div></div>`;
    const el = mount.querySelector('#vm-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                ${cell('P/E', doc.pe ? num(doc.pe) : '—')}
                ${cell('P/B', doc.pb ? num(doc.pb) : '—')}
                ${cell('P/S', doc.ps ? num(doc.ps) : '—')}
                ${cell('PEG', doc.peg ? num(doc.peg) : '—')}
                ${cell(t('view.vm.card.divy'), pct(doc.dividend_yield_pct))}
                ${cell(t('view.vm.card.ey'), pct(doc.earnings_yield_pct))}
                ${cell(t('view.vm.card.fcfy'), pct(doc.fcf_yield_pct))}
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="vm-copy" type="button" data-i18n="view.vm.btn.copy">Copy</button>
            </div>
        </div>
    `;
    applyUiI18n(el);
    el.querySelector('#vm-copy').addEventListener('click', async () => {
        const txt = `P/E ${doc.pe}\nP/B ${doc.pb}\nP/S ${doc.ps}\nPEG ${doc.peg}\nDiv yield ${doc.dividend_yield_pct}%\nEarnings yield ${doc.earnings_yield_pct}%\nFCF yield ${doc.fcf_yield_pct}%`;
        try { await navigator.clipboard.writeText(txt); showToast(t('view.vm.toast.copied'), { level: 'success' }); }
        catch (e) { showToast(t('view.vm.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' }); }
    });
}
