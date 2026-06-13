// Move-in/out inspection checklist generator — area-by-area condition record
// with a needs-attention count, via /calc/inspection-checklist. Previews live
// as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
let LAST_DOC = null;

const CONDITIONS = ['Excellent', 'Good', 'Fair', 'Poor', 'Damaged'];
const SEED = [
    { area: 'Living room walls', condition: 'Good', notes: '' },
    { area: 'Kitchen floor', condition: 'Fair', notes: 'scratches' },
    { area: 'Bathroom faucet', condition: 'Damaged', notes: 'leaks' },
];

function condOpts(sel) {
    return CONDITIONS.map((c) => `<option value="${c}" ${c === sel ? 'selected' : ''}>${c}</option>`).join('');
}

function rowHtml(a) {
    return `
        <div class="mpb-row ick-row">
            <input type="text" class="ick-area" placeholder="${esc(t('view.ick.ph.area'))}" value="${esc(a.area || '')}">
            <select class="ick-cond">${condOpts(a.condition || 'Good')}</select>
            <input type="text" class="ick-notes" placeholder="${esc(t('view.ick.ph.notes'))}" value="${esc(a.notes || '')}">
            <button type="button" class="ick-del" data-i18n="view.ick.remove">Remove</button>
        </div>`;
}

export async function renderInspectionChecklist(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ick.h1.title">// INSPECTION CHECKLIST</span></h1>
        <p class="muted small" data-i18n="view.ick.hint.intro">
            The area-by-area condition record a landlord and tenant complete at move-in and move-out. It
            records each area's condition and notes and counts the items flagged as needing attention
            (fair / poor / damaged). Pairing the move-in and move-out records is what justifies any
            deposit deduction. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ick.h2.inputs">Checklist details</h2>
            <form id="ick-form" class="inline-form">
                <label><span data-i18n="view.ick.label.type">Inspection</span>
                    <select name="inspection_type">
                        <option value="move_in" data-i18n="view.ick.type.move_in">Move-in</option>
                        <option value="move_out" data-i18n="view.ick.type.move_out">Move-out</option>
                    </select></label>
                <label><span data-i18n="view.ick.label.state">State (optional)</span>
                    <input type="text" name="state" value="California"></label>
                <label><span data-i18n="view.ick.label.landlord">Landlord name</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.ick.label.tenant">Tenant name</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.ick.label.premises">Premises address</span>
                    <input type="text" name="premises_address" value=""></label>
                <label><span data-i18n="view.ick.label.date">Inspection date</span>
                    <input type="date" name="inspection_date" value="2026-06-01" required></label>
            </form>
            <div class="mpb-head ick-head">
                <span data-i18n="view.ick.col.area">Area</span>
                <span data-i18n="view.ick.col.cond">Condition</span>
                <span data-i18n="view.ick.col.notes">Notes</span>
                <span></span>
            </div>
            <div id="ick-rows">${SEED.map(rowHtml).join('')}</div>
            <button type="button" id="ick-add" class="secondary" data-i18n="view.ick.add">+ Add area</button>
        </div>
        <div id="ick-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ick-form');
    const rowsEl = mount.querySelector('#ick-rows');

    const generate = async () => {
        const fd = new FormData(form);
        const areas = [...rowsEl.querySelectorAll('.ick-row')].map((r) => ({
            area: (r.querySelector('.ick-area').value || '').trim(),
            condition: r.querySelector('.ick-cond').value || 'Good',
            notes: (r.querySelector('.ick-notes').value || '').trim(),
        })).filter((a) => a.area);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            inspection_type: fd.get('inspection_type'),
            inspection_date: fd.get('inspection_date'),
            areas,
            state: (fd.get('state') || '').trim(),
        };
        try {
            const doc = await api.calcInspectionChecklist(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.ick.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);

    mount.querySelector('#ick-add').addEventListener('click', () => {
        rowsEl.insertAdjacentHTML('beforeend', rowHtml({ area: '', condition: 'Good', notes: '' }));
        applyUiI18n(rowsEl.lastElementChild);
        generate();
    });
    rowsEl.addEventListener('click', (e) => {
        if (e.target.classList.contains('ick-del')) {
            e.target.closest('.ick-row').remove();
            generate();
        }
    });
    form.addEventListener('input', () => { live(); });
    rowsEl.addEventListener('input', () => { live(); });
    rowsEl.addEventListener('change', () => { live(); });
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
    const el = mount.querySelector('#ick-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.ick.card.items">Areas recorded</div>
                    <div class="value">${doc.item_count}</div></div>
                <div class="card ${doc.needs_attention_count > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.ick.card.flagged">Needs attention</div>
                    <div class="value">${doc.needs_attention_count}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="ick-copy" type="button" data-i18n="view.ick.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="ick-download" type="button" data-i18n="view.ick.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#ick-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.ick.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.ick.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#ick-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'inspection-checklist.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
