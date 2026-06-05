// Receipts browser — full-page, paginated, filterable, bulk-actionable.
//
// Built for the "10k receipts" workload where the old modal capped at 200
// rows was a dead-end. Filter by status / merchant / date range / total
// range / attachment status; sort by created_at desc; paginate at 50/page
// with prev/next buttons.
//
// Bulk operations supported via row-checkboxes + an action bar:
//   * Auto-attach selected (or all unattached above threshold).
//   * Open the existing match modal for any row by clicking it.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { tConfirm } from '../dialog.js';
import { openReceiptMatchModal } from './expenses.js';

// View-local state. Survives re-renders but resets on route change.
const STATE = {
    filters: {
        status: '',
        merchant: '',
        from: '',
        to: '',
        min_total: '',
        max_total: '',
        unattached: false,
    },
    page: 0,
    pageSize: 50,
    total: 0,
    loading: false,
    selected: new Set(),
};

const STATUS_OPTIONS = ['', 'pending', 'matching', 'done', 'failed', 'needs_image'];

export async function renderReceipts(mount, _state) {
    const tok = currentViewToken();
    STATE.selected = new Set();

    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.receipts.h1.title">// RECEIPTS</span></h1>
        <p class="muted small" data-i18n="view.receipts.hint.intro">Every receipt you've uploaded — filterable, paginated, bulk-actionable. Click any row to edit. Use the bulk bar at the top to auto-attach matched candidates across the whole library.</p>

        <div class="receipts-filterbar">
            <label class="rf-field">
                <span>${esc(t('view.receipts.filter.status'))}</span>
                <select id="rf-status">
                    ${STATUS_OPTIONS.map(s => `<option value="${s}">${esc(s ? t('view.receipts.status.' + s) : t('view.receipts.filter.any'))}</option>`).join('')}
                </select>
            </label>
            <label class="rf-field">
                <span>${esc(t('view.receipts.filter.merchant'))}</span>
                <input type="text" id="rf-merchant" placeholder="${esc(t('view.receipts.filter.merchant_placeholder'))}">
            </label>
            <label class="rf-field">
                <span>${esc(t('view.purchases.filter.year'))}</span>
                <select id="rf-year">
                    <option value="">${esc(t('view.purchases.filter.year_all'))}</option>
                    ${Array.from({ length: 7 }, (_, i) => new Date().getFullYear() - i)
                        .map(y => `<option value="${y}">${y}</option>`).join('')}
                </select>
            </label>
            <label class="rf-field">
                <span>${esc(t('view.receipts.filter.from'))}</span>
                <input type="date" id="rf-from">
            </label>
            <label class="rf-field">
                <span>${esc(t('view.receipts.filter.to'))}</span>
                <input type="date" id="rf-to">
            </label>
            <label class="rf-field">
                <span>${esc(t('view.receipts.filter.min_total'))}</span>
                <input type="number" id="rf-min" step="0.01" placeholder="0.00">
            </label>
            <label class="rf-field">
                <span>${esc(t('view.receipts.filter.max_total'))}</span>
                <input type="number" id="rf-max" step="0.01" placeholder="0.00">
            </label>
            <label class="rf-field rf-field-checkbox">
                <input type="checkbox" id="rf-unattached">
                <span>${esc(t('view.receipts.filter.unattached_only'))}</span>
            </label>
            <button type="button" id="rf-apply" class="btn btn-primary btn-compact">${esc(t('view.receipts.filter.apply'))}</button>
            <button type="button" id="rf-reset" class="btn btn-secondary btn-compact">${esc(t('view.receipts.filter.reset'))}</button>
        </div>

        <div class="receipts-actionbar">
            <span class="muted small" id="rs-count">—</span>
            <button type="button" id="ra-attach" class="btn btn-secondary btn-compact">${esc(t('view.receipts.action.auto_attach'))}</button>
            <button type="button" id="ra-reclassify" class="btn btn-secondary btn-compact" disabled>${esc(t('view.receipts.action.reclassify'))}</button>
            <button type="button" id="ra-delete" class="btn btn-secondary btn-compact danger" disabled>${esc(t('view.receipts.action.delete_selected'))}</button>
            <a id="ra-csv" class="btn btn-secondary btn-compact" download="receipts-tax-rollup.csv">${esc(t('view.receipts.action.csv'))}</a>
        </div>

        <div id="rs-table-wrap" class="rs-table-wrap"></div>

        <div class="receipts-pager" id="rs-pager"></div>
    `;

    const fb = mount.querySelector('.receipts-filterbar');
    fb.querySelector('#rf-status').value = STATE.filters.status;
    fb.querySelector('#rf-merchant').value = STATE.filters.merchant;
    fb.querySelector('#rf-from').value = STATE.filters.from;
    fb.querySelector('#rf-to').value = STATE.filters.to;

    // Year shortcut — selecting a year auto-fills from/to to that
    // year's boundaries and applies. "All years" leaves the inputs
    // untouched so a manually-typed range can stand.
    fb.querySelector('#rf-year').addEventListener('change', () => {
        const yv = fb.querySelector('#rf-year').value;
        if (!yv) return;
        const y = Number(yv);
        fb.querySelector('#rf-from').value = `${y}-01-01`;
        const today = new Date();
        fb.querySelector('#rf-to').value =
            y === today.getFullYear() ? today.toISOString().slice(0, 10) : `${y}-12-31`;
        fb.querySelector('#rf-apply').click();
    });
    fb.querySelector('#rf-min').value = STATE.filters.min_total;
    fb.querySelector('#rf-max').value = STATE.filters.max_total;
    fb.querySelector('#rf-unattached').checked = !!STATE.filters.unattached;

    fb.querySelector('#rf-apply').addEventListener('click', () => {
        STATE.filters.status     = fb.querySelector('#rf-status').value;
        STATE.filters.merchant   = fb.querySelector('#rf-merchant').value.trim();
        STATE.filters.from       = fb.querySelector('#rf-from').value;
        STATE.filters.to         = fb.querySelector('#rf-to').value;
        STATE.filters.min_total  = fb.querySelector('#rf-min').value.trim();
        STATE.filters.max_total  = fb.querySelector('#rf-max').value.trim();
        STATE.filters.unattached = fb.querySelector('#rf-unattached').checked;
        STATE.page = 0;
        loadAndRender(mount, tok);
    });
    fb.querySelector('#rf-reset').addEventListener('click', () => {
        STATE.filters = {
            status: '', merchant: '', from: '', to: '',
            min_total: '', max_total: '', unattached: false,
        };
        fb.querySelector('#rf-status').value = '';
        fb.querySelector('#rf-merchant').value = '';
        fb.querySelector('#rf-from').value = '';
        fb.querySelector('#rf-to').value = '';
        fb.querySelector('#rf-min').value = '';
        fb.querySelector('#rf-max').value = '';
        fb.querySelector('#rf-unattached').checked = false;
        STATE.page = 0;
        loadAndRender(mount, tok);
    });

    mount.querySelector('#ra-attach').addEventListener('click', () => bulkAttach(mount, tok));
    mount.querySelector('#ra-delete').addEventListener('click', () => bulkDelete(mount, tok));
    mount.querySelector('#ra-reclassify').addEventListener('click', () => bulkReclassify(mount, tok));

    // CSV download: re-uses the existing /tax-rollup.csv endpoint with
    // the current page filters' date window. Backend ignores other
    // filters for the CSV (rollup is bucket+category, not row-level).
    refreshCsvHref(mount);

    await loadAndRender(mount, tok);
}

function refreshCsvHref(mount) {
    const a = mount.querySelector('#ra-csv');
    if (!a) return;
    const params = {};
    if (STATE.filters.from) params.from = STATE.filters.from;
    if (STATE.filters.to)   params.to   = STATE.filters.to;
    a.href = api.taxRollupCsvUrl(params);
}

function activeQueryParams() {
    const p = { offset: STATE.page * STATE.pageSize, limit: STATE.pageSize };
    if (STATE.filters.status)     p.status      = STATE.filters.status;
    if (STATE.filters.merchant)   p.merchant    = STATE.filters.merchant;
    if (STATE.filters.from)       p.from        = STATE.filters.from;
    if (STATE.filters.to)         p.to          = STATE.filters.to;
    if (STATE.filters.min_total)  p.min_total   = STATE.filters.min_total;
    if (STATE.filters.max_total)  p.max_total   = STATE.filters.max_total;
    if (STATE.filters.unattached) p.unattached  = 'true';
    return p;
}

async function loadAndRender(mount, tok) {
    if (STATE.loading) return;
    STATE.loading = true;
    const wrap = mount.querySelector('#rs-table-wrap');
    wrap.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    let resp;
    try {
        resp = await api.receipts(activeQueryParams());
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        wrap.innerHTML = `<p class="boot">${esc(t('view.receipts.boot.load_failed', { err: e.message }))}</p>`;
        STATE.loading = false;
        return;
    }
    if (!viewIsCurrent(tok)) return;
    STATE.total = resp.total || 0;
    STATE.loading = false;

    renderTable(mount, resp.rows || []);
    renderPager(mount);
    renderCount(mount);
}

function renderTable(mount, rows) {
    const wrap = mount.querySelector('#rs-table-wrap');
    if (!rows.length) {
        wrap.innerHTML = `<p class="muted">${esc(t('view.receipts.empty'))}</p>`;
        return;
    }
    const trs = rows.map(r => {
        const attached = r.transaction_id ? '✓' : '';
        const totalCell = r.ocr_total != null ? `$${Number(r.ocr_total).toFixed(2)}` : '—';
        return `<tr data-receipt-id="${r.id}">
            <td><input type="checkbox" class="rs-select" data-id="${r.id}"${STATE.selected.has(r.id) ? ' checked' : ''}></td>
            <td>${esc(r.created_at.slice(0, 10))}</td>
            <td>${esc(r.ocr_date || '—')}</td>
            <td><span class="rs-status rs-status-${esc(r.ocr_status)}">${esc(t('view.receipts.status.' + r.ocr_status))}</span></td>
            <td>${esc(r.ocr_merchant || '—')}</td>
            <td class="num">${totalCell}</td>
            <td class="num">${r.ocr_confidence != null ? (Number(r.ocr_confidence) * 100).toFixed(0) + '%' : '—'}</td>
            <td>${esc(attached)}</td>
            <td><button type="button" class="btn btn-secondary btn-compact rs-open" data-id="${r.id}">${esc(t('view.receipts.action.open'))}</button></td>
        </tr>`;
    }).join('');
    wrap.innerHTML = `<table class="trades rs-table">
        <thead><tr>
            <th><input type="checkbox" id="rs-select-all"></th>
            <th>${esc(t('view.receipts.col.uploaded'))}</th>
            <th>${esc(t('view.receipts.col.receipt_date'))}</th>
            <th>${esc(t('view.receipts.col.status'))}</th>
            <th>${esc(t('view.receipts.col.merchant'))}</th>
            <th class="num">${esc(t('view.receipts.col.total'))}</th>
            <th class="num">${esc(t('view.receipts.col.conf'))}</th>
            <th>${esc(t('view.receipts.col.attached'))}</th>
            <th></th>
        </tr></thead>
        <tbody>${trs}</tbody>
    </table>`;
    wrap.querySelectorAll('.rs-select').forEach(cb => {
        cb.addEventListener('change', () => {
            if (cb.checked) STATE.selected.add(cb.dataset.id);
            else STATE.selected.delete(cb.dataset.id);
            renderCount(mount);
        });
    });
    const selectAll = wrap.querySelector('#rs-select-all');
    if (selectAll) {
        selectAll.addEventListener('change', () => {
            const ids = [...wrap.querySelectorAll('.rs-select')].map(cb => cb.dataset.id);
            if (selectAll.checked) ids.forEach(id => STATE.selected.add(id));
            else ids.forEach(id => STATE.selected.delete(id));
            wrap.querySelectorAll('.rs-select').forEach(cb => { cb.checked = selectAll.checked; });
            renderCount(mount);
        });
    }
    wrap.querySelectorAll('.rs-open').forEach(btn => {
        btn.addEventListener('click', async (ev) => {
            ev.stopPropagation();
            const id = btn.dataset.id;
            try {
                const meta = await api.receiptMeta(id);
                await openReceiptMatchModal(meta);
            } catch (e) {
                showToast(t('view.receipts.toast.open_err', { err: e.message }),
                    { level: 'error' });
            }
        });
    });
}

function renderPager(mount) {
    const pager = mount.querySelector('#rs-pager');
    const totalPages = Math.max(1, Math.ceil(STATE.total / STATE.pageSize));
    const cur = STATE.page + 1;
    pager.innerHTML = `
        <button type="button" id="rs-prev"${STATE.page <= 0 ? ' disabled' : ''}>${esc(t('view.receipts.pager.prev'))}</button>
        <span class="muted small">${esc(t('view.receipts.pager.position', { cur, total: totalPages }))}</span>
        <button type="button" id="rs-next"${cur >= totalPages ? ' disabled' : ''}>${esc(t('view.receipts.pager.next'))}</button>
    `;
    const tok = currentViewToken();
    pager.querySelector('#rs-prev').addEventListener('click', () => {
        if (STATE.page > 0) { STATE.page -= 1; loadAndRender(mount, tok); }
    });
    pager.querySelector('#rs-next').addEventListener('click', () => {
        if ((STATE.page + 1) * STATE.pageSize < STATE.total) {
            STATE.page += 1; loadAndRender(mount, tok);
        }
    });
}

function renderCount(mount) {
    const el = mount.querySelector('#rs-count');
    if (!el) return;
    const start = STATE.page * STATE.pageSize + 1;
    const end = Math.min(STATE.total, (STATE.page + 1) * STATE.pageSize);
    const sel = STATE.selected.size;
    el.textContent = STATE.total > 0
        ? t('view.receipts.count', { start, end, total: fmt(STATE.total), sel })
        : t('view.receipts.empty');
    // Enable bulk Delete / Reclassify only when at least one row is
    // selected. CSV + Auto-attach are always available.
    const delBtn = mount.querySelector('#ra-delete');
    const rcBtn  = mount.querySelector('#ra-reclassify');
    if (delBtn) delBtn.disabled = sel === 0;
    if (rcBtn)  rcBtn.disabled  = sel === 0;
}

async function bulkDelete(mount, tok) {
    const ids = [...STATE.selected];
    if (!ids.length) return;
    const ok = await tConfirm('view.receipts.confirm.bulk_delete', { n: ids.length });
    if (!ok) return;
    try {
        const r = await api.bulkDeleteReceipts(ids);
        if (!viewIsCurrent(tok)) return;
        STATE.selected = new Set();
        showToast(t('view.receipts.toast.bulk_deleted', { n: r.affected }),
            { level: 'success' });
        await loadAndRender(mount, tok);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.receipts.toast.bulk_delete_err', { err: e.message }),
            { level: 'error' });
    }
}

async function bulkReclassify(mount, tok) {
    const ids = [...STATE.selected];
    if (!ids.length) return;
    // Quick inline picker: ask for category, tax bucket, and (if
    // rental) which property. Categories are exposed via the existing
    // i18n catalog — restricted to a curated 5-bucket set here for the
    // bulk path (the full 21-category list is overkill for batch
    // re-classification).
    const BUCKETS = ['business', 'rental', 'personal', 'unclassified'];
    const bucket = window.prompt(
        t('view.receipts.bulk_reclassify.bucket_prompt', { buckets: BUCKETS.join(' / ') }),
        'business',
    );
    if (!bucket) return;
    if (!BUCKETS.includes(bucket)) {
        showToast(t('view.receipts.bulk_reclassify.invalid_bucket'), { level: 'error' });
        return;
    }
    let property_id = null;
    if (bucket === 'rental') {
        try {
            const props = await api.rentalProperties();
            if (!props.length) {
                showToast(t('view.expenses.bucket.no_properties'), { level: 'warning' });
                return;
            }
            const label = props.map((p, i) => `${i + 1}. ${p.nickname || p.id}`).join('\n');
            const choice = window.prompt(
                t('view.receipts.bulk_reclassify.property_prompt', { list: label }),
                '1',
            );
            const n = parseInt(choice, 10);
            if (!Number.isFinite(n) || n < 1 || n > props.length) {
                showToast(t('view.receipts.bulk_reclassify.invalid_property'), { level: 'error' });
                return;
            }
            property_id = props[n - 1].id;
        } catch (_) { /* fall through */ }
    }
    try {
        const r = await api.bulkPatchReceiptItems({
            ids, tax_bucket: bucket,
            rental_property_id: bucket === 'rental' ? property_id : null,
        });
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.receipts.toast.bulk_reclassified', {
            affected: r.affected, total: ids.length,
        }), { level: 'success' });
        STATE.selected = new Set();
        await loadAndRender(mount, tok);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.receipts.toast.bulk_reclassify_err', { err: e.message }),
            { level: 'error' });
    }
}

async function bulkAttach(mount, tok) {
    const ok = await tConfirm('view.receipts.confirm.bulk_attach', {});
    if (!ok) return;
    const btn = mount.querySelector('#ra-attach');
    btn.disabled = true;
    const orig = btn.textContent;
    btn.textContent = t('view.receipts.action.auto_attach_running');
    try {
        // Server attaches every unattached `done` receipt above the
        // threshold. Filter narrowing comes from the date range only
        // for now — narrower than the table filters is a future TODO.
        const body = {};
        if (STATE.filters.from) body.from = STATE.filters.from;
        if (STATE.filters.to)   body.to   = STATE.filters.to;
        const r = await api.bulkAttachReceipts(body);
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.receipts.toast.bulk_attached', {
            attached: r.attached, examined: r.examined,
            low: r.skipped_low_score, none: r.skipped_no_candidates,
        }), { level: 'success' });
        await loadAndRender(mount, tok);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.receipts.toast.bulk_attach_err', { err: e.message }), { level: 'error' });
    } finally {
        btn.disabled = false;
        btn.textContent = orig;
    }
}
