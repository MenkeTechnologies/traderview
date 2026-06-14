// Warrant agreement generator — cashless (net) exercise, intrinsic value,
// loan coverage, and expiration, via /calc/warrant.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

export async function renderWarrant(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.warrant.h1.title">// STOCK PURCHASE WARRANT</span></h1>
        <p class="muted small" data-i18n="view.warrant.hint.intro">
            A warrant gives the holder the right to buy shares at a fixed price until expiration, usually
            issued to an investor or lender as a sweetener. Unlike an employee option grant, it has no
            vesting and no AMT; its defining mechanic is the cashless (net) exercise — surrendering the
            in-the-money value for shares instead of paying cash. Drafting aid, not legal/securities advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.warrant.h2.inputs">Warrant terms</h2>
            <form id="warrant-form" class="inline-form">
                <label><span data-i18n="view.warrant.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.warrant.label.company">Company</span>
                    <input type="text" name="company_name" value=""></label>
                <label><span data-i18n="view.warrant.label.holder">Holder</span>
                    <input type="text" name="holder_name" value=""></label>
                <label><span data-i18n="view.warrant.label.shares">Warrant shares</span>
                    <input type="number" step="1000" min="0" name="warrant_shares" value="10000" required></label>
                <label><span data-i18n="view.warrant.label.strike">Strike price ($)</span>
                    <input type="number" step="0.01" min="0" name="strike_usd" value="2.00" required></label>
                <label><span data-i18n="view.warrant.label.fmv">Current FMV ($)</span>
                    <input type="number" step="0.01" min="0" name="fmv_usd" value="10.00" required></label>
                <label><span data-i18n="view.warrant.label.term">Term (years)</span>
                    <input type="number" step="1" min="1" name="term_years" value="5" required></label>
                <label><span data-i18n="view.warrant.label.loan">Referenced loan ($, optional)</span>
                    <input type="number" step="10000" min="0" name="loan_amount_usd" value="500000"></label>
                <label><span data-i18n="view.warrant.label.issue">Issue date</span>
                    <input type="date" name="issue_date" value="2026-07-01" required></label>
                <label><span data-i18n="view.warrant.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.warrant.ph.statute'))}"></label>
            </form>
        </div>
        <div id="warrant-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#warrant-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            holder_name: (fd.get('holder_name') || '').trim(),
            warrant_shares: Number(fd.get('warrant_shares')) || 0,
            strike_usd: Number(fd.get('strike_usd')) || 0,
            fmv_usd: Number(fd.get('fmv_usd')) || 0,
            term_years: Number(fd.get('term_years')) || 0,
            loan_amount_usd: Number(fd.get('loan_amount_usd')) || 0,
            issue_date: fd.get('issue_date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcWarrant(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.warrant.toast.error'), { level: 'error' });
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
    const coverageCard = doc.coverage_pct > 0
        ? `<div class="card"><div class="label" data-i18n="view.warrant.card.coverage">Loan coverage</div>
               <div class="value">${pct(doc.coverage_pct)}</div></div>`
        : '';
    const el = mount.querySelector('#warrant-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.warrant.card.net">Cashless net shares</div>
                    <div class="value">${num(doc.cashless_net_shares)}</div></div>
                <div class="card"><div class="label" data-i18n="view.warrant.card.intrinsic">Intrinsic value</div>
                    <div class="value">${money(doc.intrinsic_value_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.warrant.card.cash">Cash exercise cost</div>
                    <div class="value">${money(doc.cash_exercise_cost_usd)}</div></div>
                ${coverageCard}
                <div class="card"><div class="label" data-i18n="view.warrant.card.expiry">Expires</div>
                    <div class="value">${esc(doc.expiration_date || '—')}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="warrant-copy" type="button" data-i18n="view.warrant.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="warrant-download" type="button" data-i18n="view.warrant.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#warrant-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.warrant.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.warrant.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#warrant-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'warrant.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
