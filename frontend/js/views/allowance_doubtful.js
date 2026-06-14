// Allowance for doubtful accounts generator — aging-method bad-debt reserve and
// net realizable AR, via /calc/allowance-doubtful.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

const SEED = [
    { label: 'Current', balance: 100000, rate: 1 },
    { label: '31-60', balance: 50000, rate: 5 },
    { label: '61-90', balance: 30000, rate: 20 },
    { label: '90+', balance: 20000, rate: 50 },
];

function rowHtml(tr) {
    return `
        <div class="mpb-row nec-row">
            <input type="text" class="adt-label" placeholder="${esc(t('view.adt.ph.label'))}" value="${esc(tr.label || '')}">
            <input type="number" step="1000" min="0" class="adt-balance" value="${tr.balance}">
            <input type="number" step="0.1" min="0" class="adt-rate" value="${tr.rate}">
            <button type="button" class="adt-del" data-i18n="view.adt.remove">Remove</button>
        </div>`;
}

export async function renderAllowanceDoubtful(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.adt.h1.title">// ALLOWANCE FOR DOUBTFUL ACCOUNTS</span></h1>
        <p class="muted small" data-i18n="view.adt.hint.intro">
            The aging-method estimate of uncollectible receivables. Each AR aging tier carries an estimated
            uncollectible percentage; the allowance is the sum of each tier's balance times its percentage, and
            net realizable AR is the total less the allowance. It also computes the adjusting entry to raise or
            lower an existing allowance balance. Drafting aid, not accounting advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.adt.h2.inputs">Reserve details</h2>
            <form id="adt-form" class="inline-form">
                <label><span data-i18n="view.adt.label.company">Company</span>
                    <input type="text" name="company_name" value="Acme Supply" required></label>
                <label><span data-i18n="view.adt.label.date">As-of date</span>
                    <input type="date" name="as_of_date" value="2026-06-30" required></label>
                <label><span data-i18n="view.adt.label.existing">Existing allowance ($)</span>
                    <input type="number" step="1000" min="0" name="existing_allowance_usd" value="12000"></label>
                <label><span data-i18n="view.adt.label.note">Note (optional)</span>
                    <input type="text" name="note" value="" placeholder="${esc(t('view.adt.ph.note'))}"></label>
            </form>
            <div class="mpb-head nec-head">
                <span data-i18n="view.adt.col.label">Aging tier</span>
                <span data-i18n="view.adt.col.balance">Balance ($)</span>
                <span data-i18n="view.adt.col.rate">Uncollectible %</span>
                <span></span>
            </div>
            <div id="adt-rows">${SEED.map(rowHtml).join('')}</div>
            <button type="button" id="adt-add" class="secondary" data-i18n="view.adt.add">+ Add tier</button>
        </div>
        <div id="adt-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#adt-form');
    const rowsEl = mount.querySelector('#adt-rows');

    const generate = async () => {
        const fd = new FormData(form);
        const tiers = [...rowsEl.querySelectorAll('.nec-row')].map((r) => ({
            label: (r.querySelector('.adt-label').value || '').trim(),
            balance_usd: Number(r.querySelector('.adt-balance').value) || 0,
            uncollectible_pct: Number(r.querySelector('.adt-rate').value) || 0,
        })).filter((tr) => tr.label);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            as_of_date: fd.get('as_of_date'),
            tiers,
            existing_allowance_usd: Number(fd.get('existing_allowance_usd')) || 0,
            note: (fd.get('note') || '').trim(),
        };
        try {
            const doc = await api.calcAllowanceDoubtful(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.adt.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);

    mount.querySelector('#adt-add').addEventListener('click', () => {
        rowsEl.insertAdjacentHTML('beforeend', rowHtml({ label: '', balance: 0, rate: 0 }));
        applyUiI18n(rowsEl.lastElementChild);
        generate();
    });
    rowsEl.addEventListener('click', (e) => {
        if (e.target.classList.contains('adt-del')) {
            e.target.closest('.nec-row').remove();
            generate();
        }
    });
    form.addEventListener('input', () => { live(); });
    rowsEl.addEventListener('input', () => { live(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase(), ''];
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const rows = doc.rows.map((r) => `
        <tr><td>${esc(r.label)}</td><td>${money(r.balance_usd)}</td><td>${r.uncollectible_pct}%</td><td>${money(r.reserve_usd)}</td></tr>
    `).join('');
    const adj = doc.adjusting_entry_usd;
    const adjKey = adj >= 0 ? 'view.adt.card.entry_up' : 'view.adt.card.entry_down';
    const el = mount.querySelector('#adt-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card neg"><div class="label" data-i18n="view.adt.card.allowance">Allowance</div>
                    <div class="value">${money(doc.allowance_usd)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.adt.card.net">Net realizable AR</div>
                    <div class="value">${money(doc.net_realizable_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.adt.card.pct">% of AR</div>
                    <div class="value">${pct(doc.allowance_pct_of_ar)}</div></div>
                <div class="card"><div class="label" data-i18n="${adjKey}">Adjusting entry</div>
                    <div class="value">${money(Math.abs(adj))}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="adt-copy" type="button" data-i18n="view.adt.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="adt-download" type="button" data-i18n="view.adt.btn.download">Download .txt</button>
            </div>
        </div>
        <table class="data-table">
            <thead><tr>
                <th data-i18n="view.adt.th.label">Aging tier</th>
                <th data-i18n="view.adt.th.balance">Balance</th>
                <th data-i18n="view.adt.th.rate">Uncollectible</th>
                <th data-i18n="view.adt.th.reserve">Reserve</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#adt-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.adt.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.adt.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#adt-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'allowance-doubtful.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
