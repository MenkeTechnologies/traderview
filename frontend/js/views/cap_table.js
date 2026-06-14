// Cap table generator — per-holder ownership % of fully-diluted shares, via
// /calc/cap-table. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

const CLASSES = ['Common', 'Preferred', 'Options'];
const SEED = [
    { name: 'Founder', shares: 800000, cls: 'Common' },
    { name: 'Investor', shares: 200000, cls: 'Preferred' },
];

function classOpts(sel) {
    return CLASSES.map((c) => `<option value="${c}" ${c === sel ? 'selected' : ''}>${c}</option>`).join('');
}

function rowHtml(h) {
    return `
        <div class="mpb-row ick-row">
            <input type="text" class="ct-name" placeholder="${esc(t('view.ct.ph.name'))}" value="${esc(h.name || '')}">
            <input type="number" step="1" min="0" class="ct-shares" value="${h.shares}">
            <select class="ct-class">${classOpts(h.cls || 'Common')}</select>
            <button type="button" class="ct-del" data-i18n="view.ct.remove">Remove</button>
        </div>`;
}

export async function renderCapTable(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ct.h1.title">// CAP TABLE</span></h1>
        <p class="muted small" data-i18n="view.ct.hint.intro">
            The ledger of who owns a company's equity. Each holder's shares as a percentage of the
            fully-diluted total (issued shares plus the unallocated option pool) gives their ownership.
            Drafting aid, not legal/securities advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ct.h2.inputs">Capitalization</h2>
            <form id="ct-form" class="inline-form">
                <label><span data-i18n="view.ct.label.company">Company</span>
                    <input type="text" name="company_name" value="Widgets Inc" required></label>
                <label><span data-i18n="view.ct.label.date">As of date</span>
                    <input type="date" name="as_of_date" value="2026-07-15" required></label>
                <label><span data-i18n="view.ct.label.pool">Option pool (unallocated shares)</span>
                    <input type="number" step="1" min="0" name="option_pool_shares" value="100000"></label>
            </form>
            <div class="mpb-head ick-head">
                <span data-i18n="view.ct.col.name">Holder</span>
                <span data-i18n="view.ct.col.shares">Shares</span>
                <span data-i18n="view.ct.col.class">Class</span>
                <span></span>
            </div>
            <div id="ct-rows">${SEED.map(rowHtml).join('')}</div>
            <button type="button" id="ct-add" class="secondary" data-i18n="view.ct.add">+ Add holder</button>
        </div>
        <div id="ct-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ct-form');
    const rowsEl = mount.querySelector('#ct-rows');

    const generate = async () => {
        const fd = new FormData(form);
        const holders = [...rowsEl.querySelectorAll('.ick-row')].map((r) => ({
            name: (r.querySelector('.ct-name').value || '').trim(),
            shares: Number(r.querySelector('.ct-shares').value) || 0,
            class: r.querySelector('.ct-class').value || 'Common',
        })).filter((h) => h.name);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            as_of_date: fd.get('as_of_date'),
            holders,
            option_pool_shares: Number(fd.get('option_pool_shares')) || 0,
        };
        try {
            const doc = await api.calcCapTable(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.ct.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);

    mount.querySelector('#ct-add').addEventListener('click', () => {
        rowsEl.insertAdjacentHTML('beforeend', rowHtml({ name: '', shares: 0, cls: 'Common' }));
        applyUiI18n(rowsEl.lastElementChild);
        generate();
    });
    rowsEl.addEventListener('click', (e) => {
        if (e.target.classList.contains('ct-del')) {
            e.target.closest('.ick-row').remove();
            generate();
        }
    });
    form.addEventListener('input', () => { live(); });
    rowsEl.addEventListener('input', () => { live(); });
    rowsEl.addEventListener('change', () => { live(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase(), ''];
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#ct-result');
    const rows = doc.holders.map((h) => `
        <tr><td>${esc(h.name)}</td><td>${esc(h.class)}</td><td>${num(h.shares)}</td><td>${pct(h.ownership_pct)}</td></tr>
    `).join('');
    const poolRow = doc.option_pool_shares > 0
        ? `<tr><td data-i18n="view.ct.pool">Option pool</td><td>—</td><td>${num(doc.option_pool_shares)}</td><td>${pct(doc.option_pool_pct)}</td></tr>`
        : '';
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.ct.card.fd">Fully-diluted</div>
                    <div class="value">${num(doc.total_fully_diluted_shares)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ct.card.issued">Issued</div>
                    <div class="value">${num(doc.total_issued_shares)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ct.card.holders">Holders</div>
                    <div class="value">${doc.holder_count}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="ct-copy" type="button" data-i18n="view.ct.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="ct-download" type="button" data-i18n="view.ct.btn.download">Download .txt</button>
            </div>
        </div>
        <table class="data-table">
            <thead><tr>
                <th data-i18n="view.ct.th.name">Holder</th>
                <th data-i18n="view.ct.th.class">Class</th>
                <th data-i18n="view.ct.th.shares">Shares</th>
                <th data-i18n="view.ct.th.own">Ownership</th>
            </tr></thead>
            <tbody>${rows}${poolRow}</tbody>
        </table>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#ct-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.ct.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.ct.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#ct-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'cap-table.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
