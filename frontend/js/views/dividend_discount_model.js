// Dividend discount model — Gordon growth or two-stage intrinsic value, via
// /calc/dividend-discount-model.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderDividendDiscountModel(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ddm.h1.title">// DIVIDEND DISCOUNT MODEL</span></h1>
        <p class="muted small" data-i18n="view.ddm.hint.intro">
            Intrinsic share value as the present value of future dividends. The Gordon model assumes constant
            growth in perpetuity; the two-stage model uses a high-growth phase for N years then a perpetual
            terminal growth. Requires the discount rate to exceed the perpetual growth rate.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ddm.h2.inputs">Model inputs</h2>
            <form id="ddm-form" class="inline-form">
                <label><span data-i18n="view.ddm.label.ticker">Ticker</span>
                    <input type="text" name="ticker" value="ACME"></label>
                <label><span data-i18n="view.ddm.label.model">Model</span>
                    <select name="model">
                        <option value="gordon" data-i18n="view.ddm.opt.gordon">Gordon (constant)</option>
                        <option value="two_stage" data-i18n="view.ddm.opt.two">Two-stage</option>
                    </select></label>
                <label><span data-i18n="view.ddm.label.d0">Current dividend D0 ($)</span>
                    <input type="number" step="0.01" min="0" name="current_dividend_usd" value="2" required></label>
                <label><span data-i18n="view.ddm.label.r">Required return (%)</span>
                    <input type="number" step="0.1" name="required_return_pct" value="9" required></label>
                <label><span data-i18n="view.ddm.label.g">Growth (%)</span>
                    <input type="number" step="0.1" name="growth_pct" value="4" required></label>
                <label><span data-i18n="view.ddm.label.years">High-growth years (two-stage)</span>
                    <input type="number" step="1" min="0" name="high_growth_years" value="5"></label>
                <label><span data-i18n="view.ddm.label.gt">Terminal growth (%, two-stage)</span>
                    <input type="number" step="0.1" name="terminal_growth_pct" value="4"></label>
            </form>
        </div>
        <div id="ddm-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ddm-form');
    const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const generate = async () => {
        const body = {
            ticker: (form.querySelector('[name="ticker"]').value || '').trim(),
            current_dividend_usd: n('current_dividend_usd'),
            required_return_pct: n('required_return_pct'),
            model: form.querySelector('[name="model"]').value,
            growth_pct: n('growth_pct'),
            high_growth_years: n('high_growth_years'),
            terminal_growth_pct: n('terminal_growth_pct'),
        };
        try {
            const doc = await api.calcDividendDiscount(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.ddm.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#ddm-result');
    if (!doc.valid) {
        el.innerHTML = `<p class="muted" data-i18n="view.ddm.invalid">Invalid — the discount rate must exceed the perpetual growth rate.</p>`;
        applyUiI18n(el);
        return;
    }
    const twoStage = doc.pv_dividends_usd > 0;
    const split = twoStage
        ? `<div class="card"><div class="label" data-i18n="view.ddm.card.pvdiv">PV of dividends</div>
               <div class="value">${money(doc.pv_dividends_usd)}</div></div>
           <div class="card"><div class="label" data-i18n="view.ddm.card.pvterm">PV of terminal</div>
               <div class="value">${money(doc.pv_terminal_usd)}</div></div>`
        : '';
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.ddm.card.fair">Fair value</div>
                    <div class="value">${money(doc.fair_value_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ddm.card.d1">Next dividend D1</div>
                    <div class="value">${money(doc.next_dividend_usd)}</div></div>
                ${split}
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="ddm-copy" type="button" data-i18n="view.ddm.btn.copy">Copy</button>
            </div>
        </div>
        <p class="muted small">${esc(doc.model_label)}</p>
    `;
    applyUiI18n(el);
    el.querySelector('#ddm-copy').addEventListener('click', async () => {
        const txt = `${doc.model_label} fair value: ${money(doc.fair_value_usd)} (D1 ${money(doc.next_dividend_usd)})`;
        try { await navigator.clipboard.writeText(txt); showToast(t('view.ddm.toast.copied'), { level: 'success' }); }
        catch (e) { showToast(t('view.ddm.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' }); }
    });
}
