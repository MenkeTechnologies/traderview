// Seller's property disclosure generator — known-defect statement with counts,
// via /calc/seller-disclosure. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
let LAST_DOC = null;

const STATUSES = [['no_issue', 'No known issue'], ['defect', 'Known defect'], ['unknown', 'Unknown']];
const SEED = [
    { cat: 'Roof', status: 'no_issue', note: '' },
    { cat: 'Plumbing', status: 'defect', note: 'leak under kitchen sink' },
    { cat: 'Foundation', status: 'unknown', note: 'never inspected' },
];

function statusOpts(sel) {
    return STATUSES.map(([v, l]) => `<option value="${v}" ${v === sel ? 'selected' : ''}>${esc(l)}</option>`).join('');
}

function rowHtml(it) {
    return `
        <div class="mpb-row ick-row">
            <input type="text" class="sd-cat" placeholder="${esc(t('view.sd.ph.cat'))}" value="${esc(it.cat || '')}">
            <select class="sd-status">${statusOpts(it.status || 'no_issue')}</select>
            <input type="text" class="sd-note" placeholder="${esc(t('view.sd.ph.note'))}" value="${esc(it.note || '')}">
            <button type="button" class="sd-del" data-i18n="view.sd.remove">Remove</button>
        </div>`;
}

export async function renderSellerDisclosure(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sd.h1.title">// SELLER'S PROPERTY DISCLOSURE</span></h1>
        <p class="muted small" data-i18n="view.sd.hint.intro">
            The statement of known material defects a seller of real property gives the buyer before sale
            (required in most states). For each item the seller marks no known issue, a known defect, or
            unknown, with an explanation; the form counts the disclosed defects and unknowns. Drafting
            aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.sd.h2.inputs">Disclosure details</h2>
            <form id="sd-form" class="inline-form">
                <label><span data-i18n="view.sd.label.seller">Seller</span>
                    <input type="text" name="seller_name" value=""></label>
                <label><span data-i18n="view.sd.label.buyer">Buyer</span>
                    <input type="text" name="buyer_name" value=""></label>
                <label><span data-i18n="view.sd.label.property">Property address</span>
                    <input type="text" name="property_address" value=""></label>
                <label><span data-i18n="view.sd.label.date">Disclosure date</span>
                    <input type="date" name="disclosure_date" value="2026-06-15" required></label>
                <label><span data-i18n="view.sd.label.state">State (optional)</span>
                    <input type="text" name="state" value="California"></label>
                <label><span data-i18n="view.sd.label.asis">Sold as-is</span>
                    <input type="checkbox" name="as_is" checked></label>
            </form>
            <div class="mpb-head ick-head">
                <span data-i18n="view.sd.col.cat">Item</span>
                <span data-i18n="view.sd.col.status">Status</span>
                <span data-i18n="view.sd.col.note">Explanation</span>
                <span></span>
            </div>
            <div id="sd-rows">${SEED.map(rowHtml).join('')}</div>
            <button type="button" id="sd-add" class="secondary" data-i18n="view.sd.add">+ Add item</button>
        </div>
        <div id="sd-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#sd-form');
    const rowsEl = mount.querySelector('#sd-rows');

    const generate = async () => {
        const fd = new FormData(form);
        const items = [...rowsEl.querySelectorAll('.ick-row')].map((r) => ({
            category: (r.querySelector('.sd-cat').value || '').trim(),
            status: r.querySelector('.sd-status').value || 'no_issue',
            explanation: (r.querySelector('.sd-note').value || '').trim(),
        })).filter((x) => x.category);
        const body = {
            seller_name: (fd.get('seller_name') || '').trim(),
            buyer_name: (fd.get('buyer_name') || '').trim(),
            property_address: (fd.get('property_address') || '').trim(),
            items,
            as_is: fd.get('as_is') != null,
            disclosure_date: fd.get('disclosure_date'),
            state: (fd.get('state') || '').trim(),
        };
        try {
            const doc = await api.calcSellerDisclosure(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.sd.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);

    mount.querySelector('#sd-add').addEventListener('click', () => {
        rowsEl.insertAdjacentHTML('beforeend', rowHtml({ cat: '', status: 'no_issue', note: '' }));
        applyUiI18n(rowsEl.lastElementChild);
        generate();
    });
    rowsEl.addEventListener('click', (e) => {
        if (e.target.classList.contains('sd-del')) {
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
    const el = mount.querySelector('#sd-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card ${doc.defect_count > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.sd.card.defects">Defects disclosed</div>
                    <div class="value">${doc.defect_count}</div></div>
                <div class="card"><div class="label" data-i18n="view.sd.card.unknown">Unknown</div>
                    <div class="value">${doc.unknown_count}</div></div>
                <div class="card"><div class="label" data-i18n="view.sd.card.items">Items</div>
                    <div class="value">${doc.item_count}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="sd-copy" type="button" data-i18n="view.sd.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="sd-download" type="button" data-i18n="view.sd.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#sd-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.sd.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.sd.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#sd-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'seller-disclosure.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
