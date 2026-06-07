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
import { tConfirm, tPrompt } from '../dialog.js';
import { openReceiptMatchModal } from './expenses.js';
import {
    mountBusinessSelector,
    onChange as onBusinessChange,
} from '../business_context.js';

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
    searchQuery: '',
};

const STATUS_OPTIONS = ['', 'pending', 'matching', 'done', 'failed', 'needs_image'];

export async function renderReceipts(mount, _state) {
    const tok = currentViewToken();
    STATE.selected = new Set();

    mount.innerHTML = `
        <div class="receipts-title-row">
            <h1 class="view-title"><span data-i18n="view.receipts.h1.title">// RECEIPTS</span></h1>
            <span id="receipts-biz-selector"></span>
        </div>
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
            <span class="ra-reocr-wrap">
                <select id="ra-reocr-filter" class="rs-reocr-select">
                    <option value="non_ensemble">${esc(t('view.receipts.action.reocr.non_ensemble'))}</option>
                    <option value="non_vision">${esc(t('view.receipts.action.reocr.non_vision'))}</option>
                    <option value="failed">${esc(t('view.receipts.action.reocr.failed'))}</option>
                    <option value="low_confidence">${esc(t('view.receipts.action.reocr.low_confidence'))}</option>
                    <option value="all">${esc(t('view.receipts.action.reocr.all'))}</option>
                </select>
                <button type="button" id="ra-reocr" class="btn btn-secondary btn-compact">${esc(t('view.receipts.action.reocr.run'))}</button>
            </span>
            <span id="ra-reocr-status" class="muted small" hidden></span>
            <button type="button" id="ra-duplicates" class="btn btn-secondary btn-compact">${esc(t('view.receipts.action.find_duplicates'))}</button>
            <a id="ra-csv" class="btn btn-secondary btn-compact" download="receipts-tax-rollup.csv">${esc(t('view.receipts.action.csv'))}</a>
        </div>

        <div class="receipts-search-bar">
            <input type="search" id="rs-search" class="rs-search-input"
                   placeholder="${esc(t('view.receipts.search.placeholder'))}"
                   autocomplete="off" autocorrect="off" spellcheck="false">
            <button type="button" id="rs-search-btn" class="btn btn-secondary btn-compact">${esc(t('view.receipts.search.run'))}</button>
            <button type="button" id="rs-search-clear" class="btn btn-secondary btn-compact" hidden>${esc(t('view.receipts.search.clear'))}</button>
            <span id="rs-search-meta" class="muted small"></span>
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
    mount.querySelector('#ra-reocr').addEventListener('click', () => bulkReocr(mount, tok));
    mount.querySelector('#ra-duplicates').addEventListener('click', () => openDuplicatesModal(mount, tok));

    // Full-text search controls. Enter submits; clear restores the
    // paginated browser view.
    const searchInput = mount.querySelector('#rs-search');
    const searchBtn   = mount.querySelector('#rs-search-btn');
    const searchClear = mount.querySelector('#rs-search-clear');
    const runSearch = () => {
        const q = (searchInput.value || '').trim();
        if (!q) {
            STATE.searchQuery = '';
            searchClear.hidden = true;
            loadAndRender(mount, tok);
        } else {
            STATE.searchQuery = q;
            searchClear.hidden = false;
            runReceiptSearch(mount, tok);
        }
    };
    searchBtn.addEventListener('click', runSearch);
    searchInput.addEventListener('keydown', e => {
        if (e.key === 'Enter') { e.preventDefault(); runSearch(); }
    });
    searchClear.addEventListener('click', () => {
        searchInput.value = '';
        STATE.searchQuery = '';
        searchClear.hidden = true;
        const meta = mount.querySelector('#rs-search-meta');
        if (meta) meta.textContent = '';
        loadAndRender(mount, tok);
    });

    // CSV download: re-uses the existing /tax-rollup.csv endpoint with
    // the current page filters' date window. Backend ignores other
    // filters for the CSV (rollup is bucket+category, not row-level).
    refreshCsvHref(mount);

    // Business selector — reloads the receipts table when switched.
    const rcptBizHost = mount.querySelector('#receipts-biz-selector');
    if (rcptBizHost) mountBusinessSelector(rcptBizHost);
    const unsubRcptBiz = onBusinessChange(() => loadAndRender(mount, tok));
    mount.__rcptUnsubBiz = unsubRcptBiz;

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
    const bucket = await tPrompt(
        'view.receipts.bulk_reclassify.bucket_prompt',
        { buckets: BUCKETS.join(' / ') },
        { defaultValue: 'business' },
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
            const choice = await tPrompt(
                'view.receipts.bulk_reclassify.property_prompt',
                { list: label },
                { defaultValue: '1' },
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

// Bulk re-OCR — re-run the engine across the user's receipt corpus
// according to a filter. The /bulk-reocr endpoint returns {queued} and
// flips matching rows to ocr_status='pending'; the OCR semaphore
// drains the queue at min(4, num_cpus) concurrency. We poll
// /bulk-reocr/progress every 2s to update the status line + table.
async function bulkReocr(mount, tok) {
    const sel  = mount.querySelector('#ra-reocr-filter');
    const btn  = mount.querySelector('#ra-reocr');
    const stat = mount.querySelector('#ra-reocr-status');
    const filter = sel.value;
    const ok = await tConfirm('view.receipts.confirm.bulk_reocr', { filter });
    if (!ok) return;

    btn.disabled = true;
    const orig = btn.textContent;
    btn.textContent = t('view.receipts.action.reocr.queueing');
    try {
        const r = await api.bulkReocr(filter);
        if (!viewIsCurrent(tok)) return;
        if (r.queued === 0) {
            showToast(t('view.receipts.toast.reocr_nothing'), { level: 'info' });
            return;
        }
        showToast(t('view.receipts.toast.reocr_queued', { n: r.queued }), { level: 'success' });
        stat.hidden = false;
        btn.textContent = t('view.receipts.action.reocr.running');

        // Poll progress. Stop once pending==0 AND we've already seen
        // ≥1 progress tick (avoid quitting before the worker pool
        // picks up the first job).
        const queued = r.queued;
        let sawProgress = false;
        let lastPending = Infinity;
        while (viewIsCurrent(tok)) {
            await new Promise(res => setTimeout(res, 2000));
            if (!viewIsCurrent(tok)) return;
            let prog;
            try { prog = await api.reocrProgress(); }
            catch (_) { continue; }
            stat.textContent = t('view.receipts.reocr.progress', {
                pending: prog.pending,
                queued,
                failed: prog.failed,
            });
            if (prog.pending < lastPending) sawProgress = true;
            lastPending = prog.pending;
            if (sawProgress && prog.pending === 0) break;
        }
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.receipts.toast.reocr_done'), { level: 'success' });
        stat.hidden = true;
        await loadAndRender(mount, tok);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.receipts.toast.reocr_err', { err: e.message }), { level: 'error' });
    } finally {
        btn.disabled = false;
        btn.textContent = orig;
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

// ── Full-text search (Track C3) ────────────────────────────────────────
//
// Replaces the paginated table with a relevance-ranked result list when
// the user types a query. Backend returns ts_headline snippets that
// already wrap matched terms in « » — we render those as <mark>.

async function runReceiptSearch(mount, tok) {
    const q = STATE.searchQuery;
    const wrap = mount.querySelector('#rs-table-wrap');
    const meta = mount.querySelector('#rs-search-meta');
    const pager = mount.querySelector('#rs-pager');
    if (!q || !wrap) return;
    if (pager) pager.innerHTML = '';
    wrap.innerHTML = `<div class="muted">${esc(t('common.loading'))}</div>`;
    if (meta) meta.textContent = '';

    let hits;
    try { hits = await api.searchReceipts(q, 100); }
    catch (e) {
        if (!viewIsCurrent(tok)) return;
        wrap.innerHTML = `<div class="err">${esc(t('view.receipts.search.err', { err: e.message }))}</div>`;
        return;
    }
    if (!viewIsCurrent(tok)) return;

    if (meta) {
        meta.textContent = t('view.receipts.search.meta', { n: hits.length, q });
    }
    if (hits.length === 0) {
        wrap.innerHTML = `<div class="muted">${esc(t('view.receipts.search.empty'))}</div>`;
        return;
    }

    // Render search hits. Snippet rendering escapes EVERYTHING first,
    // then unescapes only the « » markers to <mark>. Never trust raw
    // snippet text from the DB into innerHTML directly.
    const renderSnippet = (raw) => {
        const e = esc(raw || '');
        return e.replace(/«/g, '<mark>').replace(/»/g, '</mark>');
    };

    wrap.innerHTML = `
        <table class="rs-table rs-search-table">
            <thead>
                <tr>
                    <th>${esc(t('view.receipts.col.merchant'))}</th>
                    <th>${esc(t('view.receipts.col.date'))}</th>
                    <th class="num">${esc(t('view.receipts.col.total'))}</th>
                    <th>${esc(t('view.receipts.col.snippet'))}</th>
                    <th>${esc(t('view.receipts.col.rank'))}</th>
                    <th></th>
                </tr>
            </thead>
            <tbody>
                ${hits.map(h => `
                    <tr>
                        <td>${esc(h.merchant || '—')}</td>
                        <td>${esc(h.date || '')}</td>
                        <td class="num">${h.total != null ? esc(fmt(Number(h.total))) : ''}</td>
                        <td class="rs-search-snippet">${renderSnippet(h.snippet)}</td>
                        <td>${esc((h.rank ?? 0).toFixed(3))}</td>
                        <td><button type="button" class="btn btn-secondary btn-compact rs-open" data-id="${esc(h.id)}">${esc(t('view.receipts.action.open'))}</button></td>
                    </tr>
                `).join('')}
            </tbody>
        </table>
    `;

    wrap.querySelectorAll('button.rs-open').forEach(btn => {
        btn.addEventListener('click', e => {
            const id = e.currentTarget.dataset.id;
            if (id) openReceiptMatchModal(id);
        });
    });
}

// ── Duplicate detector (Track C2) ──────────────────────────────────────
//
// Opens a modal listing groups of suspected duplicates. Each group has
// a "Keep this one, delete the rest" affordance per row. Uses the
// existing bulk-delete endpoint for the action.

async function openDuplicatesModal(mount, tok) {
    let groups;
    try { groups = await api.receiptDuplicates({ within_days: 3 }); }
    catch (e) {
        showToast(t('view.receipts.dupes.err.load', { err: e.message }), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;

    // Strip any existing modal so re-opening doesn't stack.
    document.querySelectorAll('.dupes-modal-host').forEach(el => el.remove());

    const host = document.createElement('div');
    host.className = 'dupes-modal-host';
    host.innerHTML = `
      <div class="dupes-modal-bg"></div>
      <div class="dupes-modal-panel">
        <header class="dupes-modal-head">
          <h2>${esc(t('view.receipts.dupes.title'))}</h2>
          <span class="muted small">${t('view.receipts.dupes.summary', { n: groups.length })}</span>
          <button type="button" class="btn btn-secondary btn-compact dupes-close">${esc(t('common.close'))}</button>
        </header>
        <div class="dupes-modal-body">
          ${groups.length === 0
            ? `<p class="muted">${esc(t('view.receipts.dupes.empty'))}</p>`
            : groups.map((g, gi) => `
              <section class="dupes-group" data-gi="${gi}">
                <header class="dupes-group-head">
                  <strong>${esc(g.canonical_merchant)}</strong>
                  <span class="muted small">${esc(fmt(Number(g.total)))} · ${g.receipts.length} ${esc(t('view.receipts.dupes.unit.copies'))}</span>
                </header>
                <table class="dupes-table">
                  <thead><tr>
                    <th>${esc(t('view.receipts.col.date'))}</th>
                    <th>${esc(t('view.receipts.dupes.col.filename'))}</th>
                    <th>${esc(t('view.receipts.dupes.col.attached'))}</th>
                    <th></th>
                  </tr></thead>
                  <tbody>
                    ${g.receipts.map(r => `
                      <tr data-id="${esc(r.id)}">
                        <td>${esc(r.ocr_date || '')}</td>
                        <td>${esc(r.filename)}</td>
                        <td>${r.transaction_id ? '✓' : ''}</td>
                        <td>
                          <button type="button" class="btn btn-secondary btn-compact dupes-keep" data-gi="${gi}" data-id="${esc(r.id)}">
                            ${esc(t('view.receipts.dupes.action.keep'))}
                          </button>
                          <button type="button" class="btn btn-secondary btn-compact dupes-open" data-id="${esc(r.id)}">
                            ${esc(t('view.receipts.action.open'))}
                          </button>
                        </td>
                      </tr>
                    `).join('')}
                  </tbody>
                </table>
              </section>
            `).join('')
          }
        </div>
      </div>
    `;
    document.body.appendChild(host);

    const close = () => host.remove();
    host.querySelector('.dupes-close').addEventListener('click', close);
    host.querySelector('.dupes-modal-bg').addEventListener('click', close);

    host.querySelectorAll('button.dupes-open').forEach(btn => {
        btn.addEventListener('click', e => {
            const id = e.currentTarget.dataset.id;
            if (id) openReceiptMatchModal(id);
        });
    });

    host.querySelectorAll('button.dupes-keep').forEach(btn => {
        btn.addEventListener('click', async e => {
            const gi = +e.currentTarget.dataset.gi;
            const keepId = e.currentTarget.dataset.id;
            const group = groups[gi];
            if (!group) return;
            const toDelete = group.receipts
                .map(r => r.id)
                .filter(id => id !== keepId);
            if (toDelete.length === 0) return;
            // Refuse to delete a receipt that's attached to a CSV
            // transaction without explicit confirm — losing the
            // attached audit trail is a real cost.
            const anyAttached = group.receipts.some(r =>
                r.id !== keepId && r.transaction_id);
            const confirmKey = anyAttached
                ? 'view.receipts.dupes.confirm.delete_with_attached'
                : 'view.receipts.dupes.confirm.delete';
            const ok = await tConfirm(confirmKey, { n: toDelete.length });
            if (!ok) return;
            try {
                const r = await api.bulkDeleteReceipts(toDelete);
                showToast(t('view.receipts.dupes.toast.deleted', { n: r.affected }), { level: 'success' });
                // Remove the group's row block from the modal.
                const sec = host.querySelector(`section.dupes-group[data-gi="${gi}"]`);
                if (sec) sec.remove();
                // Refresh the underlying table.
                await loadAndRender(mount, tok);
            } catch (err) {
                showToast(t('view.receipts.dupes.toast.delete_err', { err: err.message }), { level: 'error' });
            }
        });
    });
}
