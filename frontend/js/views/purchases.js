// Unified purchases view — one row per "purchase" across the whole
// year: every receipt line item AND every CSV/PDF-imported transaction
// that has no attached receipt detail. Click → drill back to the
// receipt photo or the source transaction.
//
// The backend's `/api/tax/purchases` endpoint UNIONs both halves and
// returns the rows pre-joined; we just paginate, filter, and render.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { openReceiptMatchModal } from './expenses.js';
import { initDragReorder } from '../drag_reorder.js';

const STATE = {
    filters: {
        from: '',
        to: '',
        category: '',
        tax_bucket: '',
        min_total: '',
        max_total: '',
        search: '',
    },
    page: 0,
    pageSize: 100,
    total: 0,
    loading: false,
};

const BUCKETS = ['', 'business', 'rental', 'personal', 'unclassified'];
const CATEGORIES = [
    '', 'advertising', 'vehicle_fuel', 'vehicle_maintenance',
    'travel_transport', 'travel_lodging', 'meals',
    'office_supplies', 'office_equipment_software', 'supplies_cogs',
    'repairs_maintenance', 'utilities', 'rent_lease',
    'insurance', 'professional_services', 'contract_labor',
    'wages_benefits', 'bank_fees', 'taxes_licenses_dues',
    'education_training', 'groceries', 'other',
];

export async function renderPurchases(mount, _state) {
    const tok = currentViewToken();
    const currentYear = new Date().getFullYear();
    if (!STATE.filters.from) {
        STATE.filters.from = `${currentYear}-01-01`;
        STATE.filters.to = new Date().toISOString().slice(0, 10);
    }

    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.purchases.h1.title">// PURCHASES</span></h1>
        <p class="muted small" data-i18n="view.purchases.hint.intro">Every receipt line item and every imported transaction that doesn't have a receipt — one row per purchase, year-to-date. Click a row's receipt or transaction badge to drill back.</p>

        <div class="purchases-filterbar">
            <label class="rf-field">
                <span>${esc(t('view.purchases.filter.year'))}</span>
                <select id="pf-year">
                    <option value="">${esc(t('view.purchases.filter.year_all'))}</option>
                    ${Array.from({ length: 7 }, (_, i) => currentYear - i)
                        .map(y => `<option value="${y}"${String(y) === String(STATE.filters._year || currentYear) ? ' selected' : ''}>${y}</option>`).join('')}
                </select></label>
            <div class="pf-period-pills" role="tablist">
                <button type="button" data-period="ytd" class="${(STATE.filters._period || 'ytd') === 'ytd' ? 'active' : ''}">${esc(t('view.expenses.tax.period_ytd'))}</button>
                <button type="button" data-period="q1" class="${STATE.filters._period === 'q1' ? 'active' : ''}">Q1</button>
                <button type="button" data-period="q2" class="${STATE.filters._period === 'q2' ? 'active' : ''}">Q2</button>
                <button type="button" data-period="q3" class="${STATE.filters._period === 'q3' ? 'active' : ''}">Q3</button>
                <button type="button" data-period="q4" class="${STATE.filters._period === 'q4' ? 'active' : ''}">Q4</button>
                <button type="button" data-period="full" class="${STATE.filters._period === 'full' ? 'active' : ''}">${esc(t('view.expenses.tax.period_full'))}</button>
            </div>
            <label class="rf-field">
                <span>${esc(t('view.purchases.filter.from'))}</span>
                <input type="date" id="pf-from" value="${esc(STATE.filters.from)}"></label>
            <label class="rf-field">
                <span>${esc(t('view.purchases.filter.to'))}</span>
                <input type="date" id="pf-to" value="${esc(STATE.filters.to)}"></label>
            <label class="rf-field">
                <span>${esc(t('view.purchases.filter.category'))}</span>
                <select id="pf-category">
                    ${CATEGORIES.map(c => `<option value="${c}">${c ? esc(t('view.expenses.cat.' + c)) : esc(t('view.receipts.filter.any'))}</option>`).join('')}
                </select>
            </label>
            <label class="rf-field">
                <span>${esc(t('view.purchases.filter.bucket'))}</span>
                <select id="pf-bucket">
                    ${BUCKETS.map(b => `<option value="${b}">${b ? esc(t('view.expenses.bucket.' + b)) : esc(t('view.receipts.filter.any'))}</option>`).join('')}
                </select>
            </label>
            <label class="rf-field">
                <span>${esc(t('view.purchases.filter.min'))}</span>
                <input type="number" id="pf-min" step="0.01" placeholder="0.00"></label>
            <label class="rf-field">
                <span>${esc(t('view.purchases.filter.max'))}</span>
                <input type="number" id="pf-max" step="0.01" placeholder="0.00"></label>
            <label class="rf-field">
                <span>${esc(t('view.purchases.filter.search'))}</span>
                <input type="text" id="pf-search" placeholder="${esc(t('view.purchases.filter.search_placeholder'))}"></label>
            <button type="button" id="pf-apply" class="btn btn-primary btn-compact">${esc(t('view.receipts.filter.apply'))}</button>
            <button type="button" id="pf-reset" class="btn btn-secondary btn-compact">${esc(t('view.receipts.filter.reset'))}</button>
        </div>

        <div class="purchases-actionbar">
            <span class="muted small" id="pf-count">—</span>
        </div>

        <div id="pf-table-wrap" class="rs-table-wrap"></div>
        <div class="receipts-pager" id="pf-pager"></div>
    `;

    const fb = mount.querySelector('.purchases-filterbar');

    // Year + period shortcut. Year-only blanks the date inputs (filter
    // by year boundary regardless of period); year + period sets exact
    // bounds. "All years" leaves both inputs blank → no date filter.
    const applyYearPeriod = () => {
        const yearVal = fb.querySelector('#pf-year').value;
        const periodBtn = fb.querySelector('.pf-period-pills button.active');
        const period = periodBtn ? periodBtn.dataset.period : 'ytd';
        STATE.filters._year = yearVal;
        STATE.filters._period = period;
        if (!yearVal) {
            // All years — keep whatever the user manually typed.
            return;
        }
        const y = Number(yearVal);
        const today = new Date();
        const todayIso = today.toISOString().slice(0, 10);
        const isCurrentYear = y === today.getFullYear();
        let from, to;
        switch (period) {
            case 'q1': from = `${y}-01-01`; to = `${y}-03-31`; break;
            case 'q2': from = `${y}-04-01`; to = `${y}-06-30`; break;
            case 'q3': from = `${y}-07-01`; to = `${y}-09-30`; break;
            case 'q4': from = `${y}-10-01`; to = `${y}-12-31`; break;
            case 'full': from = `${y}-01-01`; to = `${y}-12-31`; break;
            case 'ytd':
            default:
                from = `${y}-01-01`;
                to = isCurrentYear ? todayIso : `${y}-12-31`;
        }
        fb.querySelector('#pf-from').value = from;
        fb.querySelector('#pf-to').value   = to;
    };
    fb.querySelector('#pf-year').addEventListener('change', () => {
        applyYearPeriod();
        fb.querySelector('#pf-apply').click();
    });
    fb.querySelectorAll('.pf-period-pills button').forEach(btn => {
        btn.addEventListener('click', () => {
            fb.querySelectorAll('.pf-period-pills button')
                .forEach(b => b.classList.toggle('active', b === btn));
            applyYearPeriod();
            fb.querySelector('#pf-apply').click();
        });
    });
    // Drag-reorder the period pills — persisted per-user so favorite
    // periods (YTD, current quarter) drift left over time.
    const pillRow = fb.querySelector('.pf-period-pills');
    if (pillRow) {
        initDragReorder(pillRow, 'button[data-period]', 'pf_period_pill_order', {
            direction: 'horizontal',
            getKey: (el) => el.dataset.period,
            toastKey: 'toast.reordered_pills',
        });
    }

    fb.querySelector('#pf-apply').addEventListener('click', () => {
        STATE.filters.from      = fb.querySelector('#pf-from').value;
        STATE.filters.to        = fb.querySelector('#pf-to').value;
        STATE.filters.category  = fb.querySelector('#pf-category').value;
        STATE.filters.tax_bucket = fb.querySelector('#pf-bucket').value;
        STATE.filters.min_total = fb.querySelector('#pf-min').value.trim();
        STATE.filters.max_total = fb.querySelector('#pf-max').value.trim();
        STATE.filters.search    = fb.querySelector('#pf-search').value.trim();
        STATE.page = 0;
        loadAndRender(mount, tok);
    });
    fb.querySelector('#pf-reset').addEventListener('click', () => {
        STATE.filters = {
            from: `${currentYear}-01-01`,
            to: new Date().toISOString().slice(0, 10),
            category: '', tax_bucket: '',
            min_total: '', max_total: '', search: '',
        };
        fb.querySelector('#pf-from').value = STATE.filters.from;
        fb.querySelector('#pf-to').value   = STATE.filters.to;
        fb.querySelector('#pf-category').value = '';
        fb.querySelector('#pf-bucket').value   = '';
        fb.querySelector('#pf-min').value = '';
        fb.querySelector('#pf-max').value = '';
        fb.querySelector('#pf-search').value = '';
        STATE.page = 0;
        loadAndRender(mount, tok);
    });
    await loadAndRender(mount, tok);
}

function activeParams() {
    const p = { offset: STATE.page * STATE.pageSize, limit: STATE.pageSize };
    if (STATE.filters.from)       p.from        = STATE.filters.from;
    if (STATE.filters.to)         p.to          = STATE.filters.to;
    if (STATE.filters.category)   p.category    = STATE.filters.category;
    if (STATE.filters.tax_bucket) p.tax_bucket  = STATE.filters.tax_bucket;
    if (STATE.filters.min_total)  p.min_total   = STATE.filters.min_total;
    if (STATE.filters.max_total)  p.max_total   = STATE.filters.max_total;
    if (STATE.filters.search)     p.search      = STATE.filters.search;
    return p;
}

async function loadAndRender(mount, tok) {
    if (STATE.loading) return;
    STATE.loading = true;
    const wrap = mount.querySelector('#pf-table-wrap');
    wrap.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    let resp;
    try { resp = await api.listPurchases(activeParams()); }
    catch (e) {
        if (!viewIsCurrent(tok)) return;
        wrap.innerHTML = `<p class="boot">${esc(t('view.purchases.boot.load_failed', { err: e.message }))}</p>`;
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

function fmtMoney(v) {
    if (v == null || v === '') return '—';
    const n = Number(v);
    return Number.isFinite(n) ? `$${n.toFixed(2)}` : '—';
}

function renderTable(mount, rows) {
    const wrap = mount.querySelector('#pf-table-wrap');
    if (!rows.length) {
        wrap.innerHTML = `<p class="muted">${esc(t('view.purchases.empty'))}</p>`;
        return;
    }
    const trs = rows.map(r => {
        const qty = r.qty != null && r.qty !== '' ? esc(String(Number(r.qty))) : '—';
        const unit = fmtMoney(r.unit_price);
        const total = fmtMoney(r.total);
        const catChip = r.category
            ? `<span class="cat-tag cat-${esc(r.category)}">${esc(t('view.expenses.cat.' + r.category))}</span>`
            : `<span class="muted small">—</span>`;
        const bucketChip = r.tax_bucket
            ? `<span class="purchase-bucket bk-${esc(r.tax_bucket)}">${esc(t('view.expenses.bucket.' + r.tax_bucket))}</span>`
            : `<span class="muted small">—</span>`;
        const receiptBadge = r.receipt_id
            ? `<button type="button" class="purchase-link pl-receipt" data-rcpt="${esc(r.receipt_id)}" title="${esc(t('view.purchases.link.receipt'))}">📷</button>`
            : `<span class="muted">—</span>`;
        const txnBadge = r.transaction_id
            ? `<button type="button" class="purchase-link pl-txn" data-txn="${esc(r.transaction_id)}" title="${esc(t('view.purchases.link.transaction'))}">📄</button>`
            : `<span class="muted">—</span>`;
        const dateStr = r.date ? r.date : '—';
        return `<tr>
            <td class="num">${esc(dateStr)}</td>
            <td>${esc(r.name || '—')}<div class="muted small">${esc(r.merchant || '')}</div></td>
            <td class="num">${qty}</td>
            <td class="num">${unit}</td>
            <td class="num">${total}</td>
            <td>${catChip}</td>
            <td>${bucketChip}</td>
            <td>${receiptBadge}</td>
            <td>${txnBadge}</td>
        </tr>`;
    }).join('');
    wrap.innerHTML = `<table class="trades rs-table">
        <thead><tr>
            <th class="num">${esc(t('view.purchases.col.date'))}</th>
            <th>${esc(t('view.purchases.col.item'))}</th>
            <th class="num">${esc(t('view.purchases.col.qty'))}</th>
            <th class="num">${esc(t('view.purchases.col.unit'))}</th>
            <th class="num">${esc(t('view.purchases.col.total'))}</th>
            <th>${esc(t('view.purchases.col.category'))}</th>
            <th>${esc(t('view.purchases.col.bucket'))}</th>
            <th>${esc(t('view.purchases.col.receipt'))}</th>
            <th>${esc(t('view.purchases.col.transaction'))}</th>
        </tr></thead>
        <tbody>${trs}</tbody>
    </table>`;
    wrap.querySelectorAll('.pl-receipt').forEach(btn => {
        btn.addEventListener('click', async () => {
            const id = btn.dataset.rcpt;
            try {
                const meta = await api.receiptMeta(id);
                await openReceiptMatchModal(meta);
            } catch (e) {
                showToast(t('view.receipts.toast.open_err', { err: e.message }), { level: 'error' });
            }
        });
    });
    wrap.querySelectorAll('.pl-txn').forEach(btn => {
        btn.addEventListener('click', () => {
            // Navigate to the expenses page (legacy txn table) with the
            // search field pre-filled to the transaction id so the user
            // lands on the row. The legacy table's search ILIKEs the
            // merchant + description; we also URL-anchor for clarity.
            location.hash = `#expenses?txn=${btn.dataset.txn}`;
        });
    });
}

function renderPager(mount) {
    const pager = mount.querySelector('#pf-pager');
    const totalPages = Math.max(1, Math.ceil(STATE.total / STATE.pageSize));
    const cur = STATE.page + 1;
    pager.innerHTML = `
        <button type="button" id="pf-prev"${STATE.page <= 0 ? ' disabled' : ''}>${esc(t('view.receipts.pager.prev'))}</button>
        <span class="muted small">${esc(t('view.receipts.pager.position', { cur, total: totalPages }))}</span>
        <button type="button" id="pf-next"${cur >= totalPages ? ' disabled' : ''}>${esc(t('view.receipts.pager.next'))}</button>
    `;
    const tok = currentViewToken();
    pager.querySelector('#pf-prev').addEventListener('click', () => {
        if (STATE.page > 0) { STATE.page -= 1; loadAndRender(mount, tok); }
    });
    pager.querySelector('#pf-next').addEventListener('click', () => {
        if ((STATE.page + 1) * STATE.pageSize < STATE.total) {
            STATE.page += 1; loadAndRender(mount, tok);
        }
    });
}

function renderCount(mount) {
    const el = mount.querySelector('#pf-count');
    if (!el) return;
    const start = STATE.page * STATE.pageSize + 1;
    const end = Math.min(STATE.total, (STATE.page + 1) * STATE.pageSize);
    el.textContent = STATE.total > 0
        ? t('view.purchases.count', { start, end, total: fmt(STATE.total) })
        : t('view.purchases.empty');
}
