// Income statement (P&L) generator — revenue → gross profit → EBIT → pre-tax →
// net income with margins, via /calc/income-statement.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

export async function renderIncomeStatement(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.incstmt.h1.title">// INCOME STATEMENT</span></h1>
        <p class="muted small" data-i18n="view.incstmt.hint.intro">
            The multi-step business P&L: revenue less cost of goods sold gives gross profit; less operating
            expenses gives operating income (EBIT); less interest gives pre-tax income; less income tax gives
            net income. It reports the margin at each level as a percentage of revenue. Drafting aid, not
            accounting advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.incstmt.h2.inputs">Statement inputs</h2>
            <form id="is-form" class="inline-form">
                <label><span data-i18n="view.incstmt.label.company">Company</span>
                    <input type="text" name="company_name" value="Acme Co" required></label>
                <label><span data-i18n="view.incstmt.label.period">Period</span>
                    <input type="text" name="period_label" value="FY2025" required></label>
                <label><span data-i18n="view.incstmt.label.revenue">Revenue ($)</span>
                    <input type="number" step="1000" min="0" name="revenue_usd" value="1000000" required></label>
                <label><span data-i18n="view.incstmt.label.cogs">Cost of goods sold ($)</span>
                    <input type="number" step="1000" min="0" name="cogs_usd" value="600000"></label>
                <label><span data-i18n="view.incstmt.label.opex">Operating expenses ($)</span>
                    <input type="number" step="1000" min="0" name="operating_expenses_usd" value="250000"></label>
                <label><span data-i18n="view.incstmt.label.interest">Interest expense ($)</span>
                    <input type="number" step="1000" min="0" name="interest_expense_usd" value="20000"></label>
                <label><span data-i18n="view.incstmt.label.tax">Tax rate (%)</span>
                    <input type="number" step="0.1" min="0" name="tax_rate_pct" value="21"></label>
                <label><span data-i18n="view.incstmt.label.date">Date</span>
                    <input type="date" name="date" value="2026-01-31" required></label>
            </form>
        </div>
        <div id="is-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#is-form');
    const num = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const generate = async () => {
        const body = {
            company_name: (form.querySelector('[name="company_name"]').value || '').trim(),
            period_label: (form.querySelector('[name="period_label"]').value || '').trim(),
            revenue_usd: num('revenue_usd'),
            cogs_usd: num('cogs_usd'),
            operating_expenses_usd: num('operating_expenses_usd'),
            interest_expense_usd: num('interest_expense_usd'),
            tax_rate_pct: num('tax_rate_pct'),
            date: form.querySelector('[name="date"]').value,
        };
        try {
            const doc = await api.calcIncomeStatement(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.incstmt.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase(), ''];
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const niCls = doc.net_income_usd >= 0 ? 'pos' : 'neg';
    const el = mount.querySelector('#is-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card ${niCls}"><div class="label" data-i18n="view.incstmt.card.net">Net income</div>
                    <div class="value">${money(doc.net_income_usd)} (${pct(doc.net_margin_pct)})</div></div>
                <div class="card"><div class="label" data-i18n="view.incstmt.card.gross">Gross profit</div>
                    <div class="value">${money(doc.gross_profit_usd)} (${pct(doc.gross_margin_pct)})</div></div>
                <div class="card"><div class="label" data-i18n="view.incstmt.card.ebit">Operating income</div>
                    <div class="value">${money(doc.operating_income_usd)} (${pct(doc.operating_margin_pct)})</div></div>
                <div class="card"><div class="label" data-i18n="view.incstmt.card.pretax">Pre-tax income</div>
                    <div class="value">${money(doc.pretax_income_usd)} (${pct(doc.pretax_margin_pct)})</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="is-copy" type="button" data-i18n="view.incstmt.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="is-download" type="button" data-i18n="view.incstmt.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#is-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.incstmt.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.incstmt.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#is-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'income-statement.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
