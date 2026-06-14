// RSU grant generator — cliff+monthly vesting, vest value (ordinary income),
// and sell-to-cover net-share delivery, via /calc/rsu-grant.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 }));
let LAST_DOC = null;

export async function renderRsuGrant(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rsu-grant.h1.title">// RSU GRANT</span></h1>
        <p class="muted small" data-i18n="view.rsu-grant.hint.intro">
            A restricted stock unit grant — a full-value award with no strike price and no exercise. On
            vesting, the fair-market value of the vested units is ordinary income; the employer withholds
            taxes by holding back shares (sell-to-cover) and delivers the net. Drafting aid, not legal/tax advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.rsu-grant.h2.inputs">Grant terms</h2>
            <form id="rsu-form" class="inline-form">
                <label><span data-i18n="view.rsu-grant.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.rsu-grant.label.company">Company</span>
                    <input type="text" name="company_name" value=""></label>
                <label><span data-i18n="view.rsu-grant.label.grantee">Grantee</span>
                    <input type="text" name="grantee_name" value=""></label>
                <label><span data-i18n="view.rsu-grant.label.total">Total units</span>
                    <input type="number" step="1000" min="0" name="total_units" value="40000" required></label>
                <label><span data-i18n="view.rsu-grant.label.fmv">Current FMV ($)</span>
                    <input type="number" step="0.01" min="0" name="fmv_usd" value="10.00" required></label>
                <label><span data-i18n="view.rsu-grant.label.withholding">Tax withholding (%)</span>
                    <input type="number" step="0.1" min="0" name="withholding_pct" value="22"></label>
                <label><span data-i18n="view.rsu-grant.label.vesting">Vesting (months)</span>
                    <input type="number" step="1" min="1" name="vesting_months" value="48" required></label>
                <label><span data-i18n="view.rsu-grant.label.cliff">Cliff (months)</span>
                    <input type="number" step="1" min="0" name="cliff_months" value="12" required></label>
                <label><span data-i18n="view.rsu-grant.label.grant_date">Grant date</span>
                    <input type="date" name="grant_date" value="2024-01-01" required></label>
                <label><span data-i18n="view.rsu-grant.label.as_of">Valuation date</span>
                    <input type="date" name="as_of_date" value="2026-07-01" required></label>
                <label><span data-i18n="view.rsu-grant.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.rsu-grant.ph.statute'))}"></label>
            </form>
        </div>
        <div id="rsu-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#rsu-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            grantee_name: (fd.get('grantee_name') || '').trim(),
            total_units: Number(fd.get('total_units')) || 0,
            fmv_usd: Number(fd.get('fmv_usd')) || 0,
            withholding_pct: Number(fd.get('withholding_pct')) || 0,
            vesting_months: Number(fd.get('vesting_months')) || 0,
            cliff_months: Number(fd.get('cliff_months')) || 0,
            grant_date: fd.get('grant_date'),
            as_of_date: fd.get('as_of_date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcRsuGrant(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.rsu-grant.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#rsu-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.rsu-grant.card.vested">Vested units</div>
                    <div class="value">${num(doc.vested_units)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rsu-grant.card.value">Vest value (income)</div>
                    <div class="value">${money(doc.vest_value_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rsu-grant.card.withheld">Tax withheld</div>
                    <div class="value">${money(doc.tax_withheld_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rsu-grant.card.shares_withheld">Shares withheld</div>
                    <div class="value">${num(doc.shares_withheld)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.rsu-grant.card.net">Net shares delivered</div>
                    <div class="value">${num(doc.net_shares_delivered)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="rsu-copy" type="button" data-i18n="view.rsu-grant.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="rsu-download" type="button" data-i18n="view.rsu-grant.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#rsu-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.rsu-grant.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.rsu-grant.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#rsu-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'rsu-grant.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
