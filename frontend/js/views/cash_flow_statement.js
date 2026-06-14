// Cash flow statement (indirect method) generator — derive CFO/CFI/CFF and net
// change in cash, via /calc/cash-flow-statement.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderCashFlowStatement(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cfs.h1.title">// CASH FLOW STATEMENT</span></h1>
        <p class="muted small" data-i18n="view.cfs.hint.intro">
            The indirect-method cash flow statement reconciles net income to operating cash flow by adding back
            non-cash charges and adjusting for changes in working capital (an increase in a current asset uses
            cash; an increase in a current liability provides cash), then adds the investing and financing
            sections to reach the net change in cash. Drafting aid, not accounting advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.cfs.h2.inputs">Statement inputs</h2>
            <form id="cfs-form" class="inline-form">
                <label><span data-i18n="view.cfs.label.company">Company</span>
                    <input type="text" name="company_name" value="Acme Co" required></label>
                <label><span data-i18n="view.cfs.label.period">Period</span>
                    <input type="text" name="period_label" value="FY2025" required></label>
                <label><span data-i18n="view.cfs.label.ni">Net income ($)</span>
                    <input type="number" step="1000" name="net_income_usd" value="100000" required></label>
                <label><span data-i18n="view.cfs.label.da">Depreciation & amortization ($)</span>
                    <input type="number" step="1000" name="depreciation_amortization_usd" value="20000"></label>
                <label><span data-i18n="view.cfs.label.ar">Change in AR ($, + = increase)</span>
                    <input type="number" step="1000" name="change_in_ar_usd" value="15000"></label>
                <label><span data-i18n="view.cfs.label.inv">Change in inventory ($)</span>
                    <input type="number" step="1000" name="change_in_inventory_usd" value="10000"></label>
                <label><span data-i18n="view.cfs.label.ap">Change in AP ($)</span>
                    <input type="number" step="1000" name="change_in_ap_usd" value="5000"></label>
                <label><span data-i18n="view.cfs.label.capex">Capital expenditures ($)</span>
                    <input type="number" step="1000" name="capex_usd" value="30000"></label>
                <label><span data-i18n="view.cfs.label.sales">Asset sales ($)</span>
                    <input type="number" step="1000" name="asset_sales_usd" value="0"></label>
                <label><span data-i18n="view.cfs.label.debtissued">Debt issued ($)</span>
                    <input type="number" step="1000" name="debt_issued_usd" value="0"></label>
                <label><span data-i18n="view.cfs.label.debtrepaid">Debt repaid ($)</span>
                    <input type="number" step="1000" name="debt_repaid_usd" value="10000"></label>
                <label><span data-i18n="view.cfs.label.equity">Equity issued ($)</span>
                    <input type="number" step="1000" name="equity_issued_usd" value="0"></label>
                <label><span data-i18n="view.cfs.label.dividends">Dividends paid ($)</span>
                    <input type="number" step="1000" name="dividends_usd" value="8000"></label>
                <label><span data-i18n="view.cfs.label.begin">Beginning cash ($)</span>
                    <input type="number" step="1000" name="beginning_cash_usd" value="40000"></label>
                <label><span data-i18n="view.cfs.label.date">Date</span>
                    <input type="date" name="date" value="2026-01-31" required></label>
            </form>
        </div>
        <div id="cfs-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#cfs-form');
    const num = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const generate = async () => {
        const body = {
            company_name: (form.querySelector('[name="company_name"]').value || '').trim(),
            period_label: (form.querySelector('[name="period_label"]').value || '').trim(),
            net_income_usd: num('net_income_usd'),
            depreciation_amortization_usd: num('depreciation_amortization_usd'),
            change_in_ar_usd: num('change_in_ar_usd'),
            change_in_inventory_usd: num('change_in_inventory_usd'),
            change_in_ap_usd: num('change_in_ap_usd'),
            capex_usd: num('capex_usd'),
            asset_sales_usd: num('asset_sales_usd'),
            debt_issued_usd: num('debt_issued_usd'),
            debt_repaid_usd: num('debt_repaid_usd'),
            equity_issued_usd: num('equity_issued_usd'),
            dividends_usd: num('dividends_usd'),
            beginning_cash_usd: num('beginning_cash_usd'),
            date: form.querySelector('[name="date"]').value,
        };
        try {
            const doc = await api.calcCashFlowStatement(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.cfs.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#cfs-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.cfs.card.cfo">Operating</div>
                    <div class="value">${money(doc.operating_cash_flow_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.cfs.card.cfi">Investing</div>
                    <div class="value">${money(doc.investing_cash_flow_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.cfs.card.cff">Financing</div>
                    <div class="value">${money(doc.financing_cash_flow_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.cfs.card.net">Net change</div>
                    <div class="value">${money(doc.net_change_usd)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.cfs.card.ending">Ending cash</div>
                    <div class="value">${money(doc.ending_cash_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="cfs-copy" type="button" data-i18n="view.cfs.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="cfs-download" type="button" data-i18n="view.cfs.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#cfs-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.cfs.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.cfs.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#cfs-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'cash-flow-statement.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
