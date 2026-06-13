// Sales commission agreement generator — projected commission + draw + clauses,
// via /calc/commission-agreement. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderCommissionAgreement(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.comm.h1.title">// SALES COMMISSION AGREEMENT</span></h1>
        <p class="muted small" data-i18n="view.comm.hint.intro">
            Engages a salesperson on commission, optionally with a recoverable base draw. It computes the
            projected commission from the rate and expected sales (plus the draw for a projected period
            total) and assembles the commission, draw, payment-timing, and chargeback clauses. Drafting
            aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.comm.h2.inputs">Agreement details</h2>
            <form id="comm-form" class="inline-form">
                <label><span data-i18n="view.comm.label.company">Company</span>
                    <input type="text" name="company_name" value=""></label>
                <label><span data-i18n="view.comm.label.rep">Sales representative</span>
                    <input type="text" name="rep_name" value=""></label>
                <label><span data-i18n="view.comm.label.engagement">Products / territory</span>
                    <input type="text" name="engagement_description" value="Acme widgets in the Northeast territory"></label>
                <label><span data-i18n="view.comm.label.rate">Commission rate (%)</span>
                    <input type="number" step="0.001" min="0" name="commission_rate_pct" value="8" required></label>
                <label><span data-i18n="view.comm.label.sales">Expected sales ($)</span>
                    <input type="number" step="100" min="0" name="expected_sales_usd" value="200000"></label>
                <label><span data-i18n="view.comm.label.draw">Base draw per period ($)</span>
                    <input type="number" step="0.01" min="0" name="base_draw_usd" value="2000"></label>
                <label><span data-i18n="view.comm.label.terms">Payment terms (days)</span>
                    <input type="number" step="1" min="0" name="payment_terms_days" value="15" required></label>
                <label><span data-i18n="view.comm.label.start">Start date</span>
                    <input type="date" name="start_date" value="2026-07-01" required></label>
                <label><span data-i18n="view.comm.label.state">State</span>
                    <input type="text" name="state" value="New York" required></label>
            </form>
        </div>
        <div id="comm-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#comm-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            rep_name: (fd.get('rep_name') || '').trim(),
            engagement_description: (fd.get('engagement_description') || '').trim(),
            commission_rate_pct: Number(fd.get('commission_rate_pct')) || 0,
            expected_sales_usd: Number(fd.get('expected_sales_usd')) || 0,
            base_draw_usd: Number(fd.get('base_draw_usd')) || 0,
            payment_terms_days: Math.round(Number(fd.get('payment_terms_days')) || 0),
            start_date: fd.get('start_date'),
            state: (fd.get('state') || '').trim(),
        };
        try {
            const doc = await api.calcCommissionAgreement(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.comm.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#comm-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.comm.card.commission">Projected commission</div>
                    <div class="value">${money(doc.projected_commission_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.comm.card.total">Period total</div>
                    <div class="value">${money(doc.projected_period_total_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.comm.card.draw">Base draw</div>
                    <div class="value">${money(doc.base_draw_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="comm-copy" type="button" data-i18n="view.comm.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="comm-download" type="button" data-i18n="view.comm.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#comm-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.comm.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.comm.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#comm-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'commission-agreement.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
