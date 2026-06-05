// "Categorize by merchant" — power tool for bulk-triaging 10k receipts.
//
// Walks every item across the user's done receipts, groups them by
// canonical merchant, and lets the user pick a category for the whole
// merchant in one click. Re-uses /bulk-patch-items for the write.
//
// Two modes (toggle in header):
//   * default-only — only show items whose category is still a parser
//     default (unclassified / office_supplies). The common case after
//     a fresh bulk upload.
//   * all — show every merchant. Handy for re-org / cleanup.

import { api } from '../api.js';
import { t } from '../i18n.js';
import { esc, fmtMoney } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const STATE = {
    defaultOnly: true,
    groups: [],
    categories: [],
    selections: new Map(),   // canonical → chosen category code
};

export async function renderCategorize(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h2 data-i18n="view.categorize.h2">${esc(t('view.categorize.h2'))}</h2>
        <p class="muted small" data-i18n="view.categorize.subtitle">${esc(t('view.categorize.subtitle'))}</p>

        <div class="cz-toolbar">
            <label class="cz-toggle">
                <input type="checkbox" id="cz-default-only" ${STATE.defaultOnly ? 'checked' : ''}>
                <span>${esc(t('view.categorize.toggle.default_only'))}</span>
            </label>
            <span id="cz-summary" class="muted small">—</span>
            <button type="button" id="cz-reload" class="btn btn-secondary btn-compact">${esc(t('view.categorize.action.reload'))}</button>
        </div>

        <div id="cz-list" class="cz-list"></div>
    `;

    const onReload = () => loadAndRender(mount, tok);
    mount.querySelector('#cz-default-only').addEventListener('change', e => {
        STATE.defaultOnly = e.target.checked;
        onReload();
    });
    mount.querySelector('#cz-reload').addEventListener('click', onReload);

    // Categories are static within a session — fetch once at first
    // render and reuse.
    if (STATE.categories.length === 0) {
        try { STATE.categories = await api.expenseCategories(); }
        catch (_) { STATE.categories = []; }
    }

    await loadAndRender(mount, tok);
}

async function loadAndRender(mount, tok) {
    const list = mount.querySelector('#cz-list');
    if (!list) return;
    list.innerHTML = `<div class="muted">${esc(t('common.loading'))}</div>`;
    try {
        STATE.groups = await api.receiptsByMerchant({
            default_only: STATE.defaultOnly ? 1 : 0,
            min_items: 1,
        });
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        list.innerHTML = `<div class="err">${esc(t('view.categorize.err.load', { err: e.message }))}</div>`;
        return;
    }
    if (!viewIsCurrent(tok)) return;

    const summary = mount.querySelector('#cz-summary');
    const totalItems = STATE.groups.reduce((s, g) => s + g.item_count, 0);
    const totalReceipts = STATE.groups.reduce((s, g) => s + g.receipt_count, 0);
    if (summary) {
        summary.textContent = t('view.categorize.summary', {
            merchants: STATE.groups.length,
            items: totalItems,
            receipts: totalReceipts,
        });
    }

    if (STATE.groups.length === 0) {
        list.innerHTML = `<div class="muted">${esc(t('view.categorize.empty'))}</div>`;
        return;
    }

    const catOptions = STATE.categories
        .map(c => `<option value="${esc(c.code)}">${esc(c.label || c.code)}</option>`)
        .join('');

    list.innerHTML = STATE.groups.map((g, i) => {
        const picked = STATE.selections.get(g.canonical_merchant) || g.learned_category || '';
        const learnedNote = g.learned_category
            ? `<span class="cz-learned">${esc(t('view.categorize.learned', { cat: g.learned_category }))}</span>`
            : '';
        return `
        <div class="cz-row" data-i="${i}">
            <div class="cz-row-head">
                <strong class="cz-merchant">${esc(g.canonical_merchant)}</strong>
                <span class="muted small">${g.item_count} ${esc(t('view.categorize.unit.items'))} · ${g.receipt_count} ${esc(t('view.categorize.unit.receipts'))}</span>
                <span class="cz-total">${esc(fmtMoney(g.total))}</span>
                ${learnedNote}
            </div>
            <div class="cz-row-sample muted small">${esc(g.sample_items.join(' · '))}</div>
            <div class="cz-row-actions">
                <select class="cz-cat" data-i="${i}">
                    <option value="">${esc(t('view.categorize.opt.no_change'))}</option>
                    ${catOptions}
                </select>
                <button type="button" class="btn btn-secondary btn-compact cz-apply" data-i="${i}">
                    ${esc(t('view.categorize.action.apply'))}
                </button>
            </div>
        </div>`;
    }).join('');

    // Restore previously-selected values + learned defaults.
    list.querySelectorAll('select.cz-cat').forEach(sel => {
        const i = +sel.dataset.i;
        const g = STATE.groups[i];
        const picked = STATE.selections.get(g.canonical_merchant) || g.learned_category || '';
        if (picked) sel.value = picked;
    });

    list.querySelectorAll('select.cz-cat').forEach(sel => {
        sel.addEventListener('change', e => {
            const i = +e.target.dataset.i;
            const g = STATE.groups[i];
            STATE.selections.set(g.canonical_merchant, e.target.value);
        });
    });

    list.querySelectorAll('button.cz-apply').forEach(btn => {
        btn.addEventListener('click', e => {
            const i = +e.currentTarget.dataset.i;
            applyGroup(mount, tok, i);
        });
    });
}

async function applyGroup(mount, tok, i) {
    const g = STATE.groups[i];
    if (!g) return;
    const sel = mount.querySelector(`select.cz-cat[data-i="${i}"]`);
    const cat = sel ? sel.value : '';
    if (!cat) {
        showToast(t('view.categorize.err.pick_category'), { level: 'warn' });
        return;
    }
    const btn = mount.querySelector(`button.cz-apply[data-i="${i}"]`);
    if (btn) btn.disabled = true;
    try {
        const r = await api.bulkPatchReceiptItems({
            ids: g.receipt_ids,
            category: cat,
        });
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.categorize.toast.applied', {
            merchant: g.canonical_merchant,
            n: r.affected,
            cat,
        }), { level: 'success' });
        // Optimistic: drop the group from the list so the user can
        // see progress without a full reload. Full reload at the end
        // of triage will reconcile.
        STATE.groups.splice(i, 1);
        STATE.selections.delete(g.canonical_merchant);
        await loadAndRender(mount, tok);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.categorize.err.apply', { err: e.message }), { level: 'error' });
        if (btn) btn.disabled = false;
    }
}
