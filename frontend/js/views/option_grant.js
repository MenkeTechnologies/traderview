// Stock option grant (ISO/NSO) generator — cliff+monthly vesting schedule,
// exercise spread, and ISO AMT preference, via /calc/option-grant.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 }));
let LAST_DOC = null;

export async function renderOptionGrant(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.option-grant.h1.title">// STOCK OPTION GRANT</span></h1>
        <p class="muted small" data-i18n="view.option-grant.hint.intro">
            An ISO/NSO option grant with a cliff-plus-monthly vesting schedule. It computes how many options
            are vested as of a date, the in-the-money spread, the cost to exercise, and — for an ISO — the
            AMT preference item created on exercise. Drafting aid, not legal/tax advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.option-grant.h2.inputs">Grant terms</h2>
            <form id="og-form" class="inline-form">
                <label><span data-i18n="view.option-grant.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.option-grant.label.company">Company</span>
                    <input type="text" name="company_name" value=""></label>
                <label><span data-i18n="view.option-grant.label.optionee">Optionee</span>
                    <input type="text" name="optionee_name" value=""></label>
                <label><span data-i18n="view.option-grant.label.type">Option type</span>
                    <select name="option_type">
                        <option value="ISO" data-i18n="view.option-grant.opt.iso">ISO (incentive)</option>
                        <option value="NSO" data-i18n="view.option-grant.opt.nso">NSO (non-qualified)</option>
                    </select></label>
                <label><span data-i18n="view.option-grant.label.total">Total options</span>
                    <input type="number" step="1000" min="0" name="total_options" value="48000" required></label>
                <label><span data-i18n="view.option-grant.label.strike">Strike price ($)</span>
                    <input type="number" step="0.01" min="0" name="strike_usd" value="1.00" required></label>
                <label><span data-i18n="view.option-grant.label.fmv">Current FMV ($)</span>
                    <input type="number" step="0.01" min="0" name="fmv_usd" value="5.00" required></label>
                <label><span data-i18n="view.option-grant.label.vesting">Vesting (months)</span>
                    <input type="number" step="1" min="1" name="vesting_months" value="48" required></label>
                <label><span data-i18n="view.option-grant.label.cliff">Cliff (months)</span>
                    <input type="number" step="1" min="0" name="cliff_months" value="12" required></label>
                <label><span data-i18n="view.option-grant.label.grant_date">Grant date</span>
                    <input type="date" name="grant_date" value="2024-01-01" required></label>
                <label><span data-i18n="view.option-grant.label.as_of">Valuation date</span>
                    <input type="date" name="as_of_date" value="2026-07-01" required></label>
                <label><span data-i18n="view.option-grant.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.option-grant.ph.statute'))}"></label>
            </form>
        </div>
        <div id="og-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#og-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            optionee_name: (fd.get('optionee_name') || '').trim(),
            total_options: Number(fd.get('total_options')) || 0,
            strike_usd: Number(fd.get('strike_usd')) || 0,
            fmv_usd: Number(fd.get('fmv_usd')) || 0,
            option_type: fd.get('option_type') || 'ISO',
            vesting_months: Number(fd.get('vesting_months')) || 0,
            cliff_months: Number(fd.get('cliff_months')) || 0,
            grant_date: fd.get('grant_date'),
            as_of_date: fd.get('as_of_date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcOptionGrant(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.option-grant.toast.error'), { level: 'error' });
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
    const amtRow = doc.iso_amt_preference_usd > 0
        ? `<div class="card"><div class="label" data-i18n="view.option-grant.card.amt">ISO AMT preference</div>
               <div class="value">${money(doc.iso_amt_preference_usd)}</div></div>`
        : '';
    const el = mount.querySelector('#og-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.option-grant.card.vested">Vested options</div>
                    <div class="value">${num(doc.vested_options)}</div></div>
                <div class="card"><div class="label" data-i18n="view.option-grant.card.unvested">Unvested</div>
                    <div class="value">${num(doc.unvested_options)}</div></div>
                <div class="card"><div class="label" data-i18n="view.option-grant.card.spread">Vested spread</div>
                    <div class="value">${money(doc.vested_spread_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.option-grant.card.cost">Exercise cost</div>
                    <div class="value">${money(doc.vested_exercise_cost_usd)}</div></div>
                ${amtRow}
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="og-copy" type="button" data-i18n="view.option-grant.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="og-download" type="button" data-i18n="view.option-grant.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#og-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.option-grant.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.option-grant.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#og-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'option-grant.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
