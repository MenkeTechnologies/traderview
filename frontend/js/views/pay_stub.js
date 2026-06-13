// Pay stub generator — auto FICA + withholding + deductions → net pay, via
// /calc/pay-stub. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderPayStub(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ps.h1.title">// PAY STUB</span></h1>
        <p class="muted small" data-i18n="view.ps.hint.intro">
            An earnings statement for one pay period. It computes the employee FICA withholding
            automatically (Social Security 6.2% + Medicare 1.45% of gross) and combines it with the
            income-tax withholding and any other deductions to produce total deductions and net pay.
            Drafting aid, not payroll/tax advice (FICA ignores the wage base and Additional Medicare).
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ps.h2.inputs">Pay stub details</h2>
            <form id="ps-form" class="inline-form">
                <label><span data-i18n="view.ps.label.company">Employer</span>
                    <input type="text" name="company_name" value=""></label>
                <label><span data-i18n="view.ps.label.employee">Employee</span>
                    <input type="text" name="employee_name" value=""></label>
                <label><span data-i18n="view.ps.label.paydate">Pay date</span>
                    <input type="date" name="pay_date" value="2026-06-15" required></label>
                <label><span data-i18n="view.ps.label.start">Period start</span>
                    <input type="date" name="period_start" value="2026-06-01" required></label>
                <label><span data-i18n="view.ps.label.end">Period end</span>
                    <input type="date" name="period_end" value="2026-06-14" required></label>
                <label><span data-i18n="view.ps.label.gross">Gross pay ($)</span>
                    <input type="number" step="0.01" min="0" name="gross_pay_usd" value="5000" required></label>
                <label><span data-i18n="view.ps.label.fed">Federal withholding ($)</span>
                    <input type="number" step="0.01" min="0" name="federal_withholding_usd" value="600"></label>
                <label><span data-i18n="view.ps.label.state">State withholding ($)</span>
                    <input type="number" step="0.01" min="0" name="state_withholding_usd" value="200"></label>
                <label><span data-i18n="view.ps.label.other">Other deductions ($)</span>
                    <input type="number" step="0.01" min="0" name="other_deductions_usd" value="250"></label>
                <label><span data-i18n="view.ps.label.ytd">YTD gross ($)</span>
                    <input type="number" step="0.01" min="0" name="ytd_gross_usd" value="60000"></label>
            </form>
        </div>
        <div id="ps-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ps-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            employee_name: (fd.get('employee_name') || '').trim(),
            pay_date: fd.get('pay_date'),
            period_start: fd.get('period_start'),
            period_end: fd.get('period_end'),
            gross_pay_usd: Number(fd.get('gross_pay_usd')) || 0,
            federal_withholding_usd: Number(fd.get('federal_withholding_usd')) || 0,
            state_withholding_usd: Number(fd.get('state_withholding_usd')) || 0,
            other_deductions_usd: Number(fd.get('other_deductions_usd')) || 0,
            ytd_gross_usd: Number(fd.get('ytd_gross_usd')) || 0,
        };
        try {
            const doc = await api.calcPayStub(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.ps.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#ps-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.ps.card.net">Net pay</div>
                    <div class="value">${money(doc.net_pay_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ps.card.gross">Gross pay</div>
                    <div class="value">${money(doc.gross_pay_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.ps.card.deductions">Total deductions</div>
                    <div class="value">${money(doc.total_deductions_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="ps-copy" type="button" data-i18n="view.ps.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="ps-download" type="button" data-i18n="view.ps.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#ps-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.ps.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.ps.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#ps-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'pay-stub.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
