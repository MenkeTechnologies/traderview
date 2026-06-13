// Expenses view — business expense tracking + Schedule C mapping.
//
// Layout:
//   ┌─ toolbar: account picker, source picker, csv upload, rules button ─┐
//   ├─ filter row: date range, category, business toggle ────────────────┤
//   ├─ transaction table (inline category dropdown per row) ─────────────┤
//   └─ rules modal (lazy) ─────────────────────────────────────────────────┘
//
// CSV parsers are stubs until real samples arrive — upload returns 400 with
// the parser-stub message, which we surface verbatim.

import { api, ApiError } from '../api.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm, tPrompt } from '../dialog.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { applyBarWidths, fmtUsd } from '../util.js';
import { initDragReorder } from '../drag_reorder.js';
import {
    mountBusinessSelector,
    onChange as onBusinessChange,
} from '../business_context.js';

const state = {
    accounts: [],
    categories: [],
    currentAccountId: '',     // '' = ALL
    filters: { from: '', to: '', category: '', is_business: '', search: '' },
    transactions: [],
    mount: null,
    tok: 0,
};

function renderMonthChart() {
    const el = document.getElementById('exp-month-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const totals = new Map();
    for (const tx of state.transactions || []) {
        if (tx.is_transfer) continue;
        const amt = Math.abs(Number(tx.amount));
        if (!Number.isFinite(amt)) continue;
        const m = String(tx.posted_at || '').slice(0, 7);
        if (!m) continue;
        totals.set(m, (totals.get(m) || 0) + amt);
    }
    if (totals.size < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.expenses.empty_month_chart">${esc(t('view.expenses.empty_month_chart'))}</div>`;
        return;
    }
    const months = Array.from(totals.keys()).sort();
    const ys = months.map(m => totals.get(m));
    const xs = months.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false }, y: { auto: true } },
        series: [
            { label: t('view.expenses.chart.month') },
            { label: t('view.expenses.chart.monthly_spend'),
              stroke: '#b86bff', width: 1.5,
              fill: 'rgba(184,107,255,0.10)',
              points: { show: true, size: 8, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => months[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderCategoryChart() {
    const el = document.getElementById('exp-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const codeToLabel = new Map(state.categories.map(c => [c.code, c.label]));
    const totals = new Map();
    for (const tx of state.transactions || []) {
        if (tx.is_transfer) continue;
        const amt = Math.abs(Number(tx.amount));
        if (!Number.isFinite(amt)) continue;
        const key = tx.category_code || '__none__';
        totals.set(key, (totals.get(key) || 0) + amt);
    }
    const rows = [...totals.entries()].map(([k, v]) => ({
        label: k === '__none__' ? t('view.expenses.chart.uncategorized') : (codeToLabel.get(k) || k),
        v,
    })).sort((a, b) => b.v - a.v).slice(0, 10);
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.expenses.empty_chart">${esc(t('view.expenses.empty_chart'))}</div>`;
        return;
    }
    const labels = rows.map(r => r.label);
    const xs = labels.map((_, i) => i + 1);
    const ys = rows.map(r => r.v);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false }, y: { auto: true } },
        series: [
            { label: t('view.expenses.chart.category') },
            { label: t('view.expenses.chart.total_spend'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 14, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 56 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

export async function renderExpensesView(mount) {
    const tok = currentViewToken();
    state.mount = mount;
    state.tok = tok;
    mount.innerHTML = '<div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.status.loading">loading…</div></div>';
    try {
        const [accts, cats] = await Promise.all([
            api.expenseAccounts(),
            api.expenseCategories(),
        ]);
        if (!viewIsCurrent(tok)) return;
        state.accounts = accts;
        state.categories = cats;
        // If the previously-active expense account was deleted, drop the
        // stale pointer so subsequent `api.expenseTransactions({account_id})`
        // calls don't 404 on a now-gone UUID (same class of bug as the
        // trader-account stale-id fix in app.js:loadAccounts).
        if (state.currentAccountId
            && !state.accounts.some(a => a.id === state.currentAccountId)) {
            state.currentAccountId = '';
        }
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        mount.innerHTML = `<p class="boot">${esc(t('view.expenses.empty.load_failed', { err: e.message }))}</p>`;
        return;
    }
    drawShell(mount);
    await refresh();
}

function drawShell(mount) {
    const accountOpts = ['<option data-i18n="view.expenses.opt.all_accounts" value="">all accounts</option>']
        .concat(state.accounts.map(a => `<option value="${a.id}">${esc(a.name)} (${a.kind})</option>`))
        .join('');

    const sourceOpts = [
        ['amazon', t('view.expenses.source.amazon')],
        ['bofa', t('view.expenses.source.bofa')],
        ['chase', t('view.expenses.source.chase')],
        ['apple_card', t('view.expenses.source.apple_card')],
        ['generic', t('view.expenses.source.generic')],
    ].map(([v, l]) => `<option value="${v}">${esc(l)}</option>`).join('');

    const catOpts = state.categories
        .map(c => `<option value="${c.code}">${c.schedule_c_line}. ${esc(c.label)}</option>`)
        .join('');

    const currentYear = new Date().getFullYear();
    const yearOpts = Array.from({ length: 6 }, (_, i) => currentYear - i)
        .map(y => `<option value="${y}">${y}</option>`).join('');
    mount.innerHTML = `
    <div class="tax-dashboard-header">
        <h1 class="view-title"><span data-i18n="view.expenses.h1.tax_dashboard">// TAX DASHBOARD</span></h1>
        <div class="tax-period-bar">
            <span id="tax-biz-selector"></span>
            <label>${esc(t('view.expenses.tax.year'))}
                <select id="tax-year">${yearOpts}</select></label>
            <div class="tax-period-pills" role="tablist">
                <button type="button" class="active" data-period="ytd">${esc(t('view.expenses.tax.period_ytd'))}</button>
                <button type="button" data-period="q1">Q1</button>
                <button type="button" data-period="q2">Q2</button>
                <button type="button" data-period="q3">Q3</button>
                <button type="button" data-period="q4">Q4</button>
                <button type="button" data-period="full">${esc(t('view.expenses.tax.period_full'))}</button>
            </div>
            <button type="button" id="tax-export-csv" class="btn btn-secondary btn-compact">${esc(t('view.expenses.tax.export_csv'))}</button>
            <button type="button" id="tax-export-pdf-summary" class="btn btn-secondary btn-compact">${esc(t('view.expenses.tax.export_pdf_summary'))}</button>
            <button type="button" id="tax-export-pdf-detail" class="btn btn-secondary btn-compact">${esc(t('view.expenses.tax.export_pdf_detail'))}</button>
        </div>
    </div>

    <div class="tax-stat-grid" id="tax-stat-grid">
        <div class="tax-stat tax-stat-loading">
            <div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>
        </div>
    </div>

    <div class="tax-source-grid">
        <button type="button" class="tax-source-card" data-action="goto-receipts">
            <div class="tax-source-icon">📷</div>
            <div class="tax-source-label">${esc(t('view.expenses.tax.source.receipts'))}</div>
            <div class="tax-source-count" id="tax-source-receipts">—</div>
            <div class="muted small">${esc(t('view.expenses.tax.source.receipts_hint'))}</div>
        </button>
        <button type="button" class="tax-source-card" data-action="goto-statements">
            <div class="tax-source-icon">📄</div>
            <div class="tax-source-label">${esc(t('view.expenses.tax.source.statements'))}</div>
            <div class="tax-source-count" id="tax-source-statements">—</div>
            <div class="muted small">${esc(t('view.expenses.tax.source.statements_hint'))}</div>
        </button>
        <button type="button" class="tax-source-card warn" data-action="goto-uncategorized">
            <div class="tax-source-icon">⚠</div>
            <div class="tax-source-label">${esc(t('view.expenses.tax.source.uncategorized'))}</div>
            <div class="tax-source-count" id="tax-source-uncat">—</div>
            <div class="muted small">${esc(t('view.expenses.tax.source.uncategorized_hint'))}</div>
        </button>
        <button type="button" class="tax-source-card" data-action="goto-purchases">
            <div class="tax-source-icon">🛒</div>
            <div class="tax-source-label">${esc(t('view.expenses.tax.source.purchases'))}</div>
            <div class="tax-source-count">${esc(t('view.expenses.tax.source.open'))}</div>
            <div class="muted small">${esc(t('view.expenses.tax.source.purchases_hint'))}</div>
        </button>
        <button type="button" class="tax-source-card" data-action="open-rollup">
            <div class="tax-source-icon">🧮</div>
            <div class="tax-source-label">${esc(t('view.expenses.tax.source.detailed_rollup'))}</div>
            <div class="tax-source-count">${esc(t('view.expenses.tax.source.open'))}</div>
            <div class="muted small">${esc(t('view.expenses.tax.source.detailed_rollup_hint'))}</div>
        </button>
    </div>

    <div class="tax-quarterly-strip" id="tax-quarterly-strip"></div>

    <div class="tax-sankey-wrap">
        <div class="tax-sankey-label">${esc(t('view.expenses.tax.flow_label'))}</div>
        <canvas id="tax-sankey" width="900" height="240"></canvas>
    </div>

    <div class="tax-chart-grid">
        <div class="tax-chart-card">
            <div class="tax-chart-label">${esc(t('view.expenses.tax.chart.monthly'))}</div>
            <div id="tax-monthly-chart" class="tax-chart-mount"></div>
        </div>
        <div class="tax-chart-card">
            <div class="tax-chart-label">${esc(t('view.expenses.tax.chart.category_pie'))}</div>
            <canvas id="tax-pie" width="380" height="220"></canvas>
        </div>
    </div>

    <div class="tax-chart-card tax-chart-card-wide">
        <div class="tax-chart-label">${esc(t('view.expenses.tax.chart.yoy'))}</div>
        <div id="tax-yoy-chart" class="tax-chart-mount"></div>
    </div>

    <div class="tax-chart-card tax-chart-card-wide" id="tax-top-merchants-card">
        <div class="tax-chart-label">${esc(t('view.expenses.tax.chart.top_merchants'))}</div>
        <div id="tax-top-merchants" class="tax-top-merchants-mount"></div>
    </div>

    <div class="tax-chart-card tax-chart-card-wide" id="tax-calendar-card">
        <div class="tax-chart-label">${esc(t('view.expenses.tax.chart.calendar'))}</div>
        <canvas id="tax-calendar" width="980" height="160"></canvas>
        <div class="tax-calendar-legend">
            <span>${esc(t('view.expenses.tax.chart.calendar_less'))}</span>
            <span class="tax-cal-swatch s0"></span>
            <span class="tax-cal-swatch s1"></span>
            <span class="tax-cal-swatch s2"></span>
            <span class="tax-cal-swatch s3"></span>
            <span class="tax-cal-swatch s4"></span>
            <span>${esc(t('view.expenses.tax.chart.calendar_more'))}</span>
            <span class="tax-calendar-hover" id="tax-calendar-hover"></span>
        </div>
    </div>

    <div class="tax-chart-grid">
        <div class="tax-chart-card">
            <div class="tax-chart-label">${esc(t('view.expenses.tax.chart.cumulative'))}</div>
            <div id="tax-cumulative-chart" class="tax-chart-mount"></div>
        </div>
        <div class="tax-chart-card">
            <div class="tax-chart-label">${esc(t('view.expenses.tax.chart.dow'))}</div>
            <canvas id="tax-dow" width="380" height="200"></canvas>
        </div>
    </div>

    <details class="tax-schedule-detail" id="tax-c-details">
        <summary>${esc(t('view.expenses.tax.schedule_c_breakdown'))}</summary>
        <div id="tax-c-table"></div>
    </details>

    <details class="tax-schedule-detail" id="tax-e-details">
        <summary>${esc(t('view.expenses.tax.schedule_e_breakdown'))}</summary>
        <div id="tax-e-table"></div>
    </details>

    <div class="upload-tabs" role="tablist">
        <button type="button" class="upload-tab active" data-uptab="auto">${esc(t('view.expenses.uptab.auto'))}</button>
        <button type="button" class="upload-tab" data-uptab="receipts">${esc(t('view.expenses.uptab.receipts'))}</button>
        <button type="button" class="upload-tab" data-uptab="statements">${esc(t('view.expenses.uptab.statements'))}</button>
        <button type="button" class="upload-tab" data-uptab="folder">${esc(t('view.expenses.uptab.folder'))}</button>
    </div>
    <div class="receipt-dropzone unified-dropzone" id="receipt-dz" data-uptab="auto">
        <span id="dz-hint-auto" class="dz-hint"><strong>${esc(t('view.expenses.dropzone.auto_title'))}</strong> — ${esc(t('view.expenses.dropzone.auto_body'))}</span>
        <span id="dz-hint-receipts" class="dz-hint hidden"><strong>${esc(t('view.expenses.dropzone.receipts_title'))}</strong> — ${esc(t('view.expenses.dropzone.receipts_body'))}</span>
        <span id="dz-hint-statements" class="dz-hint hidden"><strong>${esc(t('view.expenses.dropzone.statements_title'))}</strong> — ${esc(t('view.expenses.dropzone.statements_body'))}</span>
        <span id="dz-hint-folder" class="dz-hint hidden"><strong>${esc(t('view.expenses.dropzone.folder_title'))}</strong> — ${esc(t('view.expenses.dropzone.folder_body'))}</span>
        <input type="file" id="receipt-file" class="hidden" accept="image/jpeg,image/png,image/webp,image/bmp,image/heic,image/heif,.heic,.heif,application/pdf">
        <input type="file" id="receipt-folder" class="hidden" webkitdirectory directory multiple>
        <input type="file" id="exp-file" class="hidden"
               accept=".csv,.xlsx,.xls,.ods,.pdf,text/csv,application/vnd.openxmlformats-officedocument.spreadsheetml.sheet,application/vnd.oasis.opendocument.spreadsheet,application/pdf">
    </div>

    <details class="legacy-expense-tools">
        <summary>${esc(t('view.expenses.tools.transactions'))}</summary>
        <div class="expense-toolbar">
            <select id="exp-account">${accountOpts}</select>
            <button data-i18n="view.expenses.btn.account" class="primary" id="exp-new-account">+ Account</button>
            <span class="sep"></span>
            <select id="exp-source">${sourceOpts}</select>
            <span id="exp-generic-map" class="hidden">
                <label class="inl">${esc(t('view.expenses.generic.date_col'))}<input type="number" min="0" step="1" id="gm-date" value="0"></label>
                <label class="inl">${esc(t('view.expenses.generic.amount_col'))}<input type="number" min="0" step="1" id="gm-amount" value="1"></label>
                <label class="inl">${esc(t('view.expenses.generic.desc_col'))}<input type="number" min="0" step="1" id="gm-desc" value="2"></label>
                <label class="inl"><input type="checkbox" id="gm-header" checked> ${esc(t('view.expenses.generic.has_header'))}</label>
                <label class="inl"><input type="checkbox" id="gm-negate"> ${esc(t('view.expenses.generic.negate'))}</label>
            </span>
            <button data-i18n="view.expenses.btn.upload_statement" class="primary" id="exp-upload">Upload statement</button>
            <span class="sep"></span>
            <button data-i18n="view.expenses.btn.seed_default_rules" id="exp-seed-rules">Seed default rules</button>
            <button data-i18n="view.expenses.btn.rules" id="exp-rules-btn">Rules</button>
            <button data-i18n="view.expenses.btn.receipts" id="exp-receipts-btn">Receipts</button>
            <button data-i18n="view.expenses.btn.scan_folder" id="exp-scan-folder">Scan folder</button>
            <button data-i18n="view.expenses.btn.schedule_c" id="exp-report-btn">Schedule C</button>
            <button data-i18n="view.expenses.btn.tax_rollup" id="exp-tax-rollup-btn">Tax rollup</button>
        </div>
    </details>

    <div class="expense-filters">
        <label><span data-i18n="view.expenses.label.from">From</span>
            <input type="date" id="exp-from"></label>
        <label><span data-i18n="view.expenses.label.to">To</span>
            <input type="date" id="exp-to"></label>
        <label><span data-i18n="view.expenses.label.category">Category</span>
            <select id="exp-category">
                <option data-i18n="view.expenses.opt.all" value="">all</option>
                <option data-i18n="view.expenses.opt.uncategorized" value="__none__">(uncategorized)</option>
                ${catOpts}
            </select>
        </label>
        <label><span data-i18n="view.expenses.label.business">Business</span>
            <select id="exp-business">
                <option data-i18n="view.expenses.opt.all_2" value="">all</option>
                <option data-i18n="view.expenses.opt.business_only" value="true">business only</option>
                <option data-i18n="view.expenses.opt.personal_only" value="false">personal only</option>
            </select>
        </label>
        <label><span data-i18n="view.expenses.label.search">Search</span>
            <input type="text" id="exp-search" data-shortcut="focus_search" placeholder="merchant / description"
                   data-i18n-placeholder="view.expenses.placeholder.search"></label>
        <button data-i18n="view.expenses.btn.apply" id="exp-apply">Apply</button>
    </div>

    <div id="exp-status" class="expense-status"></div>
    <div id="exp-table"></div>
    <div id="exp-rules-modal" class="modal hidden"></div>
    `;

    mount.querySelector('#exp-account').addEventListener('change', e => {
        state.currentAccountId = e.target.value;
        refresh();
    });
    mount.querySelector('#exp-new-account').addEventListener('click', createAccountFlow);
    const srcSel = mount.querySelector('#exp-source');
    const genMap = mount.querySelector('#exp-generic-map');
    const syncGenericMap = () => genMap.classList.toggle('hidden', srcSel.value !== 'generic');
    srcSel.addEventListener('change', syncGenericMap);
    syncGenericMap();
    mount.querySelector('#exp-upload').addEventListener('click', () => {
        mount.querySelector('#exp-file').click();
    });
    mount.querySelector('#exp-file').addEventListener('change', handleUpload);
    mount.querySelector('#exp-seed-rules').addEventListener('click', seedRulesFlow);
    mount.querySelector('#exp-rules-btn').addEventListener('click', openRulesModal);
    mount.querySelector('#exp-receipts-btn').addEventListener('click', openReceiptsModal);
    mount.querySelector('#exp-scan-folder').addEventListener('click', () => {
        mount.querySelector('#receipt-folder').click();
    });
    mount.querySelector('#receipt-folder').addEventListener('change', e => {
        receiptScanFolder(e.target.files);
        e.target.value = '';
    });
    mount.querySelector('#exp-report-btn').addEventListener('click', () => openScheduleCModal());
    mount.querySelector('#exp-tax-rollup-btn').addEventListener('click', () => openTaxRollupModal());

    bindReceiptDropzone();

    // Tax dashboard wiring — year + period drive the top stats.
    state.taxPeriod = state.taxPeriod || { year: currentYear, slice: 'ytd' };
    const yearSel = mount.querySelector('#tax-year');
    if (yearSel) yearSel.value = String(state.taxPeriod.year);
    const refreshDash = async () => {
        try { await renderTaxDashboard(mount); }
        catch (_) { /* surfaced inline */ }
    };
    if (yearSel) {
        yearSel.addEventListener('change', () => {
            state.taxPeriod.year = Number(yearSel.value);
            refreshDash();
        });
    }
    mount.querySelectorAll('.tax-period-pills button').forEach(btn => {
        btn.addEventListener('click', () => {
            mount.querySelectorAll('.tax-period-pills button')
                .forEach(b => b.classList.toggle('active', b === btn));
            state.taxPeriod.slice = btn.dataset.period;
            refreshDash();
        });
    });
    // Drag-reorder the period pills + the upload-source tabs. Order is
    // persisted per user so YTD/Q1/Q2/Q3/Q4/Full can be ordered to taste.
    const pillRow = mount.querySelector('.tax-period-pills');
    if (pillRow) {
        initDragReorder(pillRow, 'button[data-period]', 'tax_period_pill_order', {
            direction: 'horizontal',
            getKey: (el) => el.dataset.period,
            toastKey: 'toast.reordered_pills',
        });
    }
    const upTabs = mount.querySelector('.upload-tabs');
    if (upTabs) {
        initDragReorder(upTabs, '.upload-tab', 'upload_tab_order', {
            direction: 'horizontal',
            getKey: (el) => el.dataset.uptab || el.textContent.trim().slice(0, 16),
            toastKey: 'toast.reordered_upload_tabs',
        });
    }
    const csvBtn = mount.querySelector('#tax-export-csv');
    if (csvBtn) {
        csvBtn.addEventListener('click', () => {
            const { from, to } = currentTaxRange();
            window.location.href = api.taxRollupCsvUrl({ from, to });
        });
    }
    const pdfSumBtn = mount.querySelector('#tax-export-pdf-summary');
    if (pdfSumBtn) {
        pdfSumBtn.addEventListener('click', () => {
            const { from, to } = currentTaxRange();
            window.location.href = api.taxRollupPdfUrl({ from, to });
        });
    }
    const pdfDetBtn = mount.querySelector('#tax-export-pdf-detail');
    if (pdfDetBtn) {
        pdfDetBtn.addEventListener('click', () => {
            const { from, to } = currentTaxRange();
            window.location.href = api.taxRollupPdfUrl({ from, to, detail: 1 });
        });
    }

    // Upload tabs: select which picker/path the dropzone triggers when
    // clicked. The drop-handler itself still auto-routes by file type.
    state.uploadTab = state.uploadTab || 'auto';
    const applyUpTab = () => {
        const dz = mount.querySelector('#receipt-dz');
        if (dz) dz.dataset.uptab = state.uploadTab;
        mount.querySelectorAll('.dz-hint').forEach(el => el.classList.add('hidden'));
        const hint = mount.querySelector(`#dz-hint-${state.uploadTab}`);
        if (hint) hint.classList.remove('hidden');
    };
    mount.querySelectorAll('.upload-tab').forEach(btn => {
        btn.addEventListener('click', () => {
            state.uploadTab = btn.dataset.uptab;
            mount.querySelectorAll('.upload-tab').forEach(b =>
                b.classList.toggle('active', b === btn));
            applyUpTab();
        });
    });
    applyUpTab();
    mount.querySelectorAll('.tax-source-card').forEach(card => {
        card.addEventListener('click', () => {
            const act = card.dataset.action;
            if (act === 'goto-receipts')      location.hash = '#receipts';
            else if (act === 'goto-purchases') location.hash = '#purchases';
            else if (act === 'goto-statements') {
                const det = mount.querySelector('.legacy-expense-tools');
                if (det) { det.open = true; det.scrollIntoView({ behavior: 'smooth' }); }
            }
            else if (act === 'goto-uncategorized') {
                const det = mount.querySelector('.legacy-expense-tools');
                if (det) {
                    det.open = true;
                    mount.querySelector('#exp-category').value = '__none__';
                    state.filters.category = '__none__';
                    refresh();
                    det.scrollIntoView({ behavior: 'smooth' });
                }
            }
            else if (act === 'open-rollup')   openTaxRollupModal();
        });
    });
    refreshDash();

    // Business selector + onChange → re-render dashboard when filtered.
    const taxBizHost = mount.querySelector('#tax-biz-selector');
    if (taxBizHost) mountBusinessSelector(taxBizHost);
    const unsubTaxBiz = onBusinessChange(() => refreshDash());
    mount.__taxUnsubBiz = unsubTaxBiz;

    // Receipts page → open-row navigates here with `?receipt=<id>`
    // appended after the hash. Pick that up + auto-open the matcher.
    const m = (location.hash || '').match(/[?&]receipt=([0-9a-f-]{36})/i);
    if (m) {
        (async () => {
            try {
                const meta = await api.receiptMeta(m[1]);
                if (!viewIsCurrent(tok)) return;
                openReceiptMatchModal(meta);
                // Clean the param so a manual refresh doesn't keep re-opening.
                history.replaceState(null, '', '#expenses');
            } catch (_) { /* receipt deleted or not yet ready — silent */ }
        })();
    }

    mount.querySelector('#exp-apply').addEventListener('click', () => {
        state.filters.from = mount.querySelector('#exp-from').value;
        state.filters.to = mount.querySelector('#exp-to').value;
        const catVal = mount.querySelector('#exp-category').value;
        state.filters.category = catVal === '__none__' ? '__none__' : catVal;
        state.filters.is_business = mount.querySelector('#exp-business').value;
        state.filters.search = mount.querySelector('#exp-search').value;
        refresh();
    });
}

// Tax dashboard — top-of-page stats. Pulls from the existing rollup +
// transactions endpoints, aggregates client-side. No new backend.
function currentTaxRange() {
    const y = (state.taxPeriod && state.taxPeriod.year) || new Date().getFullYear();
    const slice = (state.taxPeriod && state.taxPeriod.slice) || 'ytd';
    const today = new Date();
    const todayIso = today.toISOString().slice(0, 10);
    const yearStart = `${y}-01-01`;
    const yearEnd   = `${y}-12-31`;
    const isCurrentYear = y === today.getFullYear();
    switch (slice) {
        case 'q1': return { from: `${y}-01-01`, to: `${y}-03-31` };
        case 'q2': return { from: `${y}-04-01`, to: `${y}-06-30` };
        case 'q3': return { from: `${y}-07-01`, to: `${y}-09-30` };
        case 'q4': return { from: `${y}-10-01`, to: `${y}-12-31` };
        case 'full': return { from: yearStart, to: yearEnd };
        case 'ytd':
        default:
            return { from: yearStart, to: isCurrentYear ? todayIso : yearEnd };
    }
}
async function renderTaxDashboard(mount) {
    const grid = mount.querySelector('#tax-stat-grid');
    if (!grid) return;
    const { from, to } = currentTaxRange();
    grid.innerHTML = `<div class="tax-stat tax-stat-loading"><div class="tv-spinner-wrap"><div class="tv-spinner"></div></div></div>`;
    let rollup = null, txs = [];
    try {
        rollup = await api.taxRollup({ from, to });
    } catch (_) { rollup = null; }
    try {
        txs = await api.expenseTransactions({ from, to, account_id: '' });
    } catch (_) { txs = []; }

    // Income: positive-amount transactions categorized as income, or
    // simply the sum of positives if categories aren't set up. Negative
    // amounts are expenses on most card-statement parsers.
    let income = 0, txExpense = 0;
    for (const r of (txs || [])) {
        const a = Number(r.amount);
        if (!Number.isFinite(a)) continue;
        if (a > 0) income += a;
        else txExpense += -a;
    }
    const scheduleC = rollup ? Number(rollup.business?.grand_total || 0) : 0;
    const scheduleE = rollup ? Number(rollup.rental?.grand_total || 0)   : 0;
    const personal  = rollup ? Number(rollup.personal?.grand_total || 0) : 0;
    const unclass   = rollup ? Number(rollup.unclassified?.grand_total || 0) : 0;
    const deductible = scheduleC + scheduleE;
    const netTaxable = income - deductible;

    grid.innerHTML = `
        <div class="tax-stat tax-stat-income">
            <div class="tax-stat-label">${esc(t('view.expenses.tax.stat.income'))}</div>
            <div class="tax-stat-value">${esc(fmtUsd(income))}</div>
            <div class="muted small">${(txs || []).filter(r => Number(r.amount) > 0).length} ${esc(t('view.expenses.tax.stat.transactions'))}</div>
        </div>
        <div class="tax-stat tax-stat-sched-c">
            <div class="tax-stat-label">${esc(t('view.expenses.tax.stat.schedule_c'))}</div>
            <div class="tax-stat-value">${esc(fmtUsd(scheduleC))}</div>
            <div class="muted small">${rollup ? rollup.items_counted : '—'} ${esc(t('view.expenses.tax.stat.items'))}</div>
        </div>
        <div class="tax-stat tax-stat-sched-e">
            <div class="tax-stat-label">${esc(t('view.expenses.tax.stat.schedule_e'))}</div>
            <div class="tax-stat-value">${esc(fmtUsd(scheduleE))}</div>
            <div class="muted small">${rollup ? (rollup.rental?.properties?.length || 0) : '—'} ${esc(t('view.expenses.tax.stat.properties'))}</div>
        </div>
        <div class="tax-stat tax-stat-net">
            <div class="tax-stat-label">${esc(t('view.expenses.tax.stat.net'))}</div>
            <div class="tax-stat-value">${esc(fmtUsd(netTaxable))}</div>
            <div class="muted small">${esc(t('view.expenses.tax.stat.income_minus_deduct'))}</div>
        </div>
        ${renderKpiAvgDaily(scheduleC + scheduleE + personal)}
        ${renderKpiDeductiblePct(deductible, scheduleC + scheduleE + personal)}
        ${renderKpiBurnRate(scheduleC + scheduleE + personal)}
        ${renderKpiUncategorized(unclass, rollup ? rollup.items_counted : 0)}
    `;

    // Schedule C / E breakdown tables under <details>.
    const cTbl = mount.querySelector('#tax-c-table');
    if (cTbl && rollup) {
        const rows = (rollup.business?.categories || [])
            .map(c => `<tr>
                <td><span class="sched-line">${c.schedule_c_line ? 'C' + esc(c.schedule_c_line) : '—'}</span></td>
                <td>${esc(t('view.expenses.cat.' + c.category))}</td>
                <td class="num">${fmtUsd(c.total)}</td>
            </tr>`).join('');
        cTbl.innerHTML = rows
            ? `<table class="trades"><tbody>${rows}</tbody></table>`
            : `<p class="muted small">${esc(t('view.expenses.tax.no_business'))}</p>`;
    }
    const eTbl = mount.querySelector('#tax-e-table');
    if (eTbl && rollup) {
        const props = rollup.rental?.properties || [];
        eTbl.innerHTML = props.length
            ? props.map(p => `
                <div class="rollup-property">
                    <div class="rollup-property-name">${esc(p.property_name || t('view.expenses.bucket.no_property'))} — ${fmtUsd(p.grand_total)}</div>
                    <table class="trades"><tbody>${(p.categories || []).map(c => `<tr>
                        <td><span class="sched-line">${c.schedule_e_line ? 'E' + esc(c.schedule_e_line) : '—'}</span></td>
                        <td>${esc(t('view.expenses.cat.' + c.category))}</td>
                        <td class="num">${fmtUsd(c.total)}</td>
                    </tr>`).join('')}</tbody></table>
                </div>`).join('')
            : `<p class="muted small">${esc(t('view.expenses.tax.no_rental'))}</p>`;
    }

    // Quarterly estimated-tax strip + Sankey flow viz.
    const dashYear = (state.taxPeriod || {}).year || new Date().getFullYear();
    await renderQuarterlyStrip(mount, dashYear, netTaxable);
    renderSankey(mount, {
        income, scheduleC, scheduleE, personal, unclassified: unclass,
        cCategories: rollup?.business?.categories || [],
        eProperties: rollup?.rental?.properties || [],
    });
    // Three additional charts: monthly stacked bars, category pie, YoY trend.
    void renderMonthlyBarChart(mount, dashYear);
    renderCategoryPie(mount, rollup?.business?.categories || []);
    void renderYoyTrendChart(mount);
    void renderTopMerchants(mount, dashYear);
    // Analytics chart suite — calendar heatmap, cumulative spend, day-of-week.
    void renderSpendCalendar(mount, dashYear);
    void renderCumulativeChart(mount, dashYear);
    void renderDowChart(mount, dashYear);

    // Source cards — counts.
    const recC = mount.querySelector('#tax-source-receipts');
    if (recC) recC.textContent = rollup ? `${rollup.receipts_counted} ${t('view.expenses.tax.source.receipts_label')}` : '—';
    const stmtC = mount.querySelector('#tax-source-statements');
    if (stmtC) stmtC.textContent = `${(txs || []).length} ${t('view.expenses.tax.source.txns_label')}`;
    const unc = mount.querySelector('#tax-source-uncat');
    if (unc) {
        const uncTx = (txs || []).filter(r => !r.category_id || r.category_id === '').length;
        const uncTotal = (rollup ? Number(rollup.unclassified?.grand_total || 0) : 0);
        // Personal + Unclassified is the "review me" bucket.
        unc.textContent = `${uncTx} ${t('view.expenses.tax.source.uncat_label')} · ${fmtUsd(uncTotal + personal)}`;
    }
}

// Quarterly estimated tax strip. Shows the 4 IRS due dates with the
// running total of payments logged via api.listEstimatedPayments, plus
// a simple "Log payment" button. Estimated due is the dashboard's
// guess: 25% of net taxable * (federal+SE bracket estimate ≈ 25%) — a
// rough back-of-envelope to surface the gap, not authoritative tax
// advice. Users can override by logging actuals.
const Q_DUE_DATES = (year) => [
    { quarter: 1, due: `${year}-04-15`, label: 'Q1' },
    { quarter: 2, due: `${year}-06-15`, label: 'Q2' },
    { quarter: 3, due: `${year}-09-15`, label: 'Q3' },
    { quarter: 4, due: `${year + 1}-01-15`, label: 'Q4' },
];
async function renderQuarterlyStrip(mount, year, netTaxable) {
    const wrap = mount.querySelector('#tax-quarterly-strip');
    if (!wrap) return;
    let payments = [];
    try { payments = await api.listEstimatedPayments({ tax_year: year }); }
    catch (_) { payments = []; }
    const byQ = new Map();
    for (const p of payments) {
        const sum = byQ.get(p.quarter) || 0;
        byQ.set(p.quarter, sum + Number(p.amount));
    }
    // Crude estimate: 25% of YTD net per quarter (split evenly).
    const estPerQ = Math.max(0, (netTaxable * 0.25) / 4);
    wrap.innerHTML = `
        <div class="quarterly-header">
            <strong>${esc(t('view.expenses.tax.quarterly.title', { year }))}</strong>
            <button type="button" id="quarterly-add" class="btn btn-secondary btn-compact">${esc(t('view.expenses.tax.quarterly.log'))}</button>
        </div>
        <div class="quarterly-cells">
            ${Q_DUE_DATES(year).map(q => {
                const paid = byQ.get(q.quarter) || 0;
                const remaining = Math.max(0, estPerQ - paid);
                const pct = estPerQ > 0 ? Math.min(100, (paid / estPerQ) * 100) : 0;
                return `<div class="quarterly-cell${paid > 0 ? ' has-payment' : ''}">
                    <div class="qc-label">${q.label} <span class="muted small">${esc(t('view.expenses.tax.quarterly.due'))} ${q.due}</span></div>
                    <div class="qc-amount">${fmtUsd(paid)} / <span class="muted">${fmtUsd(estPerQ)}</span></div>
                    <div class="qc-bar"><div class="qc-bar-fill" data-bar-pct="${pct.toFixed(1)}"></div></div>
                    ${remaining > 0 ? `<div class="muted small">${esc(t('view.expenses.tax.quarterly.remaining', { amt: fmtUsd(remaining) }))}</div>` : ''}
                </div>`;
            }).join('')}
        </div>
    `;
    // Tauri-release-safe bar widths — applied via rAF, see util.applyBarWidths.
    applyBarWidths(wrap);
    const addBtn = wrap.querySelector('#quarterly-add');
    if (addBtn) {
        addBtn.addEventListener('click', async () => {
            const qStr = await tPrompt('view.expenses.tax.quarterly.prompt_q', {}, { defaultValue: '1' });
            const q = parseInt(qStr, 10);
            if (!(q >= 1 && q <= 4)) return;
            const amtStr = await tPrompt('view.expenses.tax.quarterly.prompt_amount', {}, { defaultValue: '0' });
            const amount = Number(amtStr);
            if (!Number.isFinite(amount) || amount <= 0) return;
            const today = new Date().toISOString().slice(0, 10);
            try {
                await api.createEstimatedPayment({
                    tax_year: year, quarter: q, paid_at: today,
                    amount, method: '', note: '',
                });
                showToast(t('view.expenses.tax.quarterly.saved'), { level: 'success' });
                renderQuarterlyStrip(mount, year, netTaxable);
            } catch (e) {
                showToast(t('view.expenses.tax.quarterly.save_err', { err: e.message }),
                    { level: 'error' });
            }
        });
    }
}

// Canvas-based Sankey flow. Income on the left, Schedule C/E/Personal/
// Unclassified on the right. Simple 2-tier layout — keeps the code
// small while showing the relative magnitudes the user actually cares
// about. Each "ribbon" is drawn as a filled bezier.
function renderSankey(mount, data) {
    const canvas = mount.querySelector('#tax-sankey');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const W = canvas.width, H = canvas.height;
    ctx.clearRect(0, 0, W, H);
    const total = data.income;
    if (!total || total <= 0) {
        ctx.fillStyle = '#7a8ba8';
        ctx.font = '12px monospace';
        ctx.textAlign = 'center';
        ctx.fillText(t('view.expenses.tax.flow_empty'), W / 2, H / 2);
        return;
    }
    // Outflows = Schedule C + Schedule E + Personal + Unclassified +
    // (income - sum) as "remaining/net".
    const out = [
        { label: t('view.expenses.bucket.business'),     value: data.scheduleC,    color: '#05d9e8' },
        { label: t('view.expenses.bucket.rental'),       value: data.scheduleE,    color: '#ff7a3d' },
        { label: t('view.expenses.bucket.personal'),     value: data.personal,     color: '#888' },
        { label: t('view.expenses.bucket.unclassified'), value: data.unclassified, color: '#aab' },
        { label: t('view.expenses.tax.stat.net'),        value: Math.max(0, data.income - data.scheduleC - data.scheduleE - data.personal - data.unclassified),
          color: '#39ff14' },
    ].filter(o => o.value > 0);
    const outSum = out.reduce((a, o) => a + o.value, 0);
    if (outSum <= 0) {
        ctx.fillStyle = '#7a8ba8';
        ctx.font = '12px monospace';
        ctx.textAlign = 'center';
        ctx.fillText(t('view.expenses.tax.flow_empty'), W / 2, H / 2);
        return;
    }
    const marginX = 110, topMargin = 20, botMargin = 20;
    const flowH = H - topMargin - botMargin;
    const leftX = marginX, rightX = W - marginX;
    const barW = 14;
    // Income bar
    ctx.fillStyle = '#39ff14';
    ctx.fillRect(leftX - barW, topMargin, barW, flowH);
    ctx.fillStyle = '#e0f0ff';
    ctx.font = 'bold 11px monospace';
    ctx.textAlign = 'right';
    ctx.fillText(t('view.expenses.tax.stat.income'), leftX - barW - 6, topMargin + 10);
    ctx.font = '10px monospace';
    ctx.fillStyle = '#aab';
    ctx.fillText(fmtUsd(data.income), leftX - barW - 6, topMargin + 22);

    // Right-side outflow bars + ribbons
    let yL = topMargin;
    let yR = topMargin;
    for (const o of out) {
        const ribbonH = (o.value / outSum) * flowH;
        const leftTop = yL, leftBot = yL + ribbonH;
        const rightTop = yR, rightBot = yR + ribbonH;
        // Ribbon
        ctx.fillStyle = hexAlpha(o.color, 0.35);
        ctx.beginPath();
        ctx.moveTo(leftX, leftTop);
        const cpx1 = leftX + (rightX - leftX) * 0.5;
        const cpx2 = leftX + (rightX - leftX) * 0.5;
        ctx.bezierCurveTo(cpx1, leftTop, cpx2, rightTop, rightX, rightTop);
        ctx.lineTo(rightX, rightBot);
        ctx.bezierCurveTo(cpx2, rightBot, cpx1, leftBot, leftX, leftBot);
        ctx.closePath();
        ctx.fill();
        // Right bar
        ctx.fillStyle = o.color;
        ctx.fillRect(rightX, rightTop, barW, ribbonH);
        // Label
        ctx.fillStyle = '#e0f0ff';
        ctx.font = 'bold 11px monospace';
        ctx.textAlign = 'left';
        ctx.fillText(o.label, rightX + barW + 6, rightTop + 10);
        ctx.font = '10px monospace';
        ctx.fillStyle = '#aab';
        ctx.fillText(fmtUsd(o.value), rightX + barW + 6, rightTop + 22);

        yL += ribbonH;
        yR += ribbonH;
    }
}
// Monthly stacked-bar chart via uPlot. Four series (business / rental
// / personal / unclassified) drawn as stacked bars, one cluster per
// month. Uses a custom paths fn since uPlot doesn't ship a built-in
// stacked-bar renderer.
async function renderMonthlyBarChart(mount, year) {
    const el = mount.querySelector('#tax-monthly-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    let rows = [];
    try { rows = await api.monthlyTotals(year); }
    catch (_) { rows = []; }
    if (!rows.length) {
        el.innerHTML = `<p class="muted small">${esc(t('view.expenses.tax.chart.empty'))}</p>`;
        return;
    }
    const xs = rows.map((_, i) => i + 1);
    const biz   = rows.map(r => Number(r.business)   || 0);
    const rent  = rows.map(r => Number(r.rental)     || 0);
    const pers  = rows.map(r => Number(r.personal)   || 0);
    const uncl  = rows.map(r => Number(r.unclassified) || 0);
    // Stacked cumulative arrays for drawing.
    const yMax = Math.max(...rows.map(r =>
        Number(r.business) + Number(r.rental) + Number(r.personal) + Number(r.unclassified)
    ), 1);
    const W = el.clientWidth || 540, H = 200;
    const stackedPath = (color, offsetSeries, valueSeries) => (u) => {
        const ctx = u.ctx;
        ctx.save();
        ctx.fillStyle = color;
        const cw = u.bbox.width;
        const barW = Math.max(2, (cw / xs.length) * 0.65);
        const yZero = u.valToPos(0, 'y', true);
        for (let i = 0; i < xs.length; i++) {
            const x = u.valToPos(xs[i], 'x', true);
            const yBot = u.valToPos(offsetSeries[i], 'y', true);
            const yTop = u.valToPos(offsetSeries[i] + valueSeries[i], 'y', true);
            if (valueSeries[i] === 0) continue;
            ctx.fillRect(x - barW / 2, yTop, barW, yBot - yTop);
            // Silence unused yZero warning.
            void yZero;
        }
        ctx.restore();
        return null;
    };
    const offsetsBiz   = new Array(xs.length).fill(0);
    const offsetsRent  = biz.slice();
    const offsetsPers  = biz.map((v, i) => v + rent[i]);
    const offsetsUncl  = biz.map((v, i) => v + rent[i] + pers[i]);
    new window.uPlot({
        title: '', width: W, height: H,
        scales: { x: { time: false }, y: { range: [0, yMax * 1.1] } },
        series: [
            { label: 'Month', value: (_u, raw) => MONTH_NAMES[Math.round(Number(raw)) - 1] || '—' },
            { label: 'Business',     stroke: 'transparent', paths: stackedPath('#05d9e8', offsetsBiz,  biz)  },
            { label: 'Rental',       stroke: 'transparent', paths: stackedPath('#ff7a3d', offsetsRent, rent) },
            { label: 'Personal',     stroke: 'transparent', paths: stackedPath('#888',    offsetsPers, pers) },
            { label: 'Unclassified', stroke: 'transparent', paths: stackedPath('#aab',    offsetsUncl, uncl) },
        ],
        axes: [
            { stroke: '#aab',
              values: (_u, splits) => splits.map(v =>
                  MONTH_NAMES[Math.round(v) - 1] || ''),
              size: 28, rotate: 0 },
            { stroke: '#aab', size: 56,
              values: (_, splits) => splits.map(v => fmtUsd(v)) },
        ],
    }, [xs, biz, rent, pers, uncl], el);
}
const MONTH_NAMES = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];

// Category pie via custom canvas. Top 10 categories by total; remainder
// folded into "Other". Slices labeled with category + percentage.
function renderCategoryPie(mount, categories) {
    const canvas = mount.querySelector('#tax-pie');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const W = canvas.width, H = canvas.height;
    ctx.clearRect(0, 0, W, H);
    if (!categories || !categories.length) {
        ctx.fillStyle = '#7a8ba8';
        ctx.font = '11px monospace';
        ctx.textAlign = 'center';
        ctx.fillText(t('view.expenses.tax.chart.empty'), W / 2, H / 2);
        return;
    }
    // Top 10 + "Other"
    const sorted = [...categories].sort((a, b) => Number(b.total) - Number(a.total));
    const top = sorted.slice(0, 10);
    const rest = sorted.slice(10);
    const restTotal = rest.reduce((a, c) => a + Number(c.total), 0);
    if (restTotal > 0) {
        top.push({ category: 'other_aggregate', total: restTotal });
    }
    const grand = top.reduce((a, c) => a + Number(c.total), 0);
    if (grand <= 0) return;
    // Use the same category color map as the cat-tag CSS.
    const colorFor = (cat) => ({
        vehicle_fuel: '#ff7a3d', vehicle_maintenance: '#ff7a3d',
        travel_transport: '#05d9e8', travel_lodging: '#05d9e8',
        meals: '#ffd84a',
        office_supplies: '#b86bff', office_equipment_software: '#b86bff',
        supplies_cogs: '#ff5fa7',
        utilities: '#7fff8a', rent_lease: '#7fff8a', insurance: '#7fff8a',
        professional_services: '#79c8ff', contract_labor: '#79c8ff', wages_benefits: '#79c8ff',
        bank_fees: '#ff3860', taxes_licenses_dues: '#ff3860',
        education_training: '#c9b6ff', advertising: '#ff9aef',
        repairs_maintenance: '#ffab73', groceries: '#888',
    })[cat] || '#aab';
    const cx = H / 2 + 6, cy = H / 2;
    const radius = H / 2 - 14;
    let acc = -Math.PI / 2;
    for (const c of top) {
        const slice = (Number(c.total) / grand) * Math.PI * 2;
        ctx.beginPath();
        ctx.moveTo(cx, cy);
        ctx.arc(cx, cy, radius, acc, acc + slice);
        ctx.closePath();
        ctx.fillStyle = colorFor(c.category);
        ctx.fill();
        acc += slice;
    }
    // Legend
    let ly = 12;
    ctx.font = '10px monospace';
    ctx.textAlign = 'left';
    const legendX = H + 20;
    for (const c of top) {
        const pct = (Number(c.total) / grand * 100).toFixed(1);
        ctx.fillStyle = colorFor(c.category);
        ctx.fillRect(legendX, ly - 8, 10, 10);
        ctx.fillStyle = '#e0f0ff';
        const label = c.category === 'other_aggregate'
            ? t('view.expenses.cat.other')
            : t('view.expenses.cat.' + c.category);
        ctx.fillText(`${label} ${pct}%`, legendX + 14, ly);
        ly += 14;
        if (ly > H - 6) break;
    }
}

// YoY trend chart via uPlot — one line per bucket, x-axis is year.
async function renderYoyTrendChart(mount) {
    const el = mount.querySelector('#tax-yoy-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    let rows = [];
    try { rows = await api.yoyTrend(5); }
    catch (_) { rows = []; }
    if (!rows.length) {
        el.innerHTML = `<p class="muted small">${esc(t('view.expenses.tax.chart.empty'))}</p>`;
        return;
    }
    const xs = rows.map(r => r.year);
    const biz  = rows.map(r => Number(r.business)   || 0);
    const rent = rows.map(r => Number(r.rental)     || 0);
    const pers = rows.map(r => Number(r.personal)   || 0);
    const W = el.clientWidth || 900, H = 200;
    new window.uPlot({
        title: '', width: W, height: H,
        scales: { x: { time: false }, y: {} },
        series: [
            { label: 'Year', value: (_u, raw) => String(Math.round(Number(raw))) },
            { label: 'Business', stroke: '#05d9e8', width: 2, points: { show: true, size: 6 } },
            { label: 'Rental',   stroke: '#ff7a3d', width: 2, points: { show: true, size: 6 } },
            { label: 'Personal', stroke: '#888',    width: 2, points: { show: true, size: 6 } },
        ],
        axes: [
            { stroke: '#aab',
              values: (_, splits) => splits.map(v => String(Math.round(v))),
              size: 28 },
            { stroke: '#aab', size: 56,
              values: (_, splits) => splits.map(v => fmtUsd(v)) },
        ],
        legend: { show: true },
    }, [xs, biz, rent, pers], el);
}

// ── KPI cards ports from trade dashboard: average daily, % deductible,
//   30-day burn-rate trailing average, uncategorized backlog. Each
//   returns a static HTML snippet inlined into the stat grid.
function renderKpiAvgDaily(totalSpend) {
    const dayOfYear = Math.floor((Date.now() - new Date(new Date().getFullYear(), 0, 0).getTime()) / 86400000);
    const avg = dayOfYear > 0 ? totalSpend / dayOfYear : 0;
    return `<div class="tax-stat tax-stat-avg-daily">
        <div class="tax-stat-label">${esc(t('view.expenses.tax.stat.avg_daily'))}</div>
        <div class="tax-stat-value">${esc(fmtUsd(avg))}</div>
        <div class="muted small">${esc(t('view.expenses.tax.stat.ytd_pace'))}</div>
    </div>`;
}
function renderKpiDeductiblePct(deductible, totalSpend) {
    const pct = totalSpend > 0 ? (deductible / totalSpend) * 100 : 0;
    const cls = pct >= 70 ? 'tw-refund' : pct >= 40 ? '' : 'tw-owed';
    return `<div class="tax-stat tax-stat-deductible">
        <div class="tax-stat-label">${esc(t('view.expenses.tax.stat.deductible_pct'))}</div>
        <div class="tax-stat-value ${cls}">${pct.toFixed(1)}%</div>
        <div class="muted small">${esc(t('view.expenses.tax.stat.biz_plus_rental'))}</div>
    </div>`;
}
function renderKpiBurnRate(totalSpend) {
    const dayOfYear = Math.floor((Date.now() - new Date(new Date().getFullYear(), 0, 0).getTime()) / 86400000);
    const monthlyBurn = dayOfYear > 0 ? (totalSpend / dayOfYear) * 30 : 0;
    return `<div class="tax-stat tax-stat-burn">
        <div class="tax-stat-label">${esc(t('view.expenses.tax.stat.burn_rate'))}</div>
        <div class="tax-stat-value">${esc(fmtUsd(monthlyBurn))}</div>
        <div class="muted small">${esc(t('view.expenses.tax.stat.month_pace'))}</div>
    </div>`;
}
function renderKpiUncategorized(unclassTotal, totalItems) {
    const cls = unclassTotal > 0 ? 'tw-owed' : 'tw-refund';
    return `<div class="tax-stat tax-stat-uncat">
        <div class="tax-stat-label">${esc(t('view.expenses.tax.stat.uncategorized'))}</div>
        <div class="tax-stat-value ${cls}">${esc(fmtUsd(unclassTotal))}</div>
        <div class="muted small">${totalItems} ${esc(t('view.expenses.tax.stat.items'))}</div>
    </div>`;
}

// ── Calendar heatmap: GitHub-style year grid of daily spend. Each
//   column is one week (Sunday top → Saturday bottom), shaded by the
//   day's total. 5 buckets: 0, then quartiles of the year's spend
//   distribution. Hover surfaces the exact value via the legend strip.
async function renderSpendCalendar(mount, year) {
    const canvas = mount.querySelector('#tax-calendar');
    const hover = mount.querySelector('#tax-calendar-hover');
    if (!canvas) return;
    let days = [];
    try { days = await api.receiptsSpendCalendar(year); } catch { days = []; }

    const ctx = canvas.getContext('2d');
    const W = canvas.width, H = canvas.height;
    ctx.clearRect(0, 0, W, H);
    if (!days.length) {
        ctx.fillStyle = '#7a8ba8';
        ctx.font = '11px monospace';
        ctx.textAlign = 'center';
        ctx.fillText(t('view.expenses.tax.chart.empty'), W / 2, H / 2);
        return;
    }
    // Bucket thresholds from positive days only.
    const positives = days.filter(d => +d.total > 0).map(d => +d.total).sort((a, b) => a - b);
    const q = (p) => positives[Math.floor(positives.length * p)] || 0;
    const t1 = q(0.25), t2 = q(0.50), t3 = q(0.75), t4 = q(0.93);
    const bucketOf = (v) => {
        if (v <= 0) return 0;
        if (v <= t1) return 1;
        if (v <= t2) return 2;
        if (v <= t3) return 3;
        if (v <= t4) return 4;
        return 5;
    };
    // 5-step heat palette (terminal-cyan family for app theme).
    const COL = ['#16202c', '#0d3a4d', '#0d556e', '#108a9f', '#36c8d4', '#7af0ff'];

    // Layout: 53 weeks × 7 days. Leave 28px top for month labels.
    const TOP = 28;
    const cell = Math.floor(Math.min((W - 30) / 53, (H - TOP) / 7));
    const pad = Math.max(1, Math.floor(cell * 0.12));
    const cells = []; // {x, y, w, day, value}
    const firstDay = new Date(days[0].day + 'T00:00:00');
    const firstColOffset = firstDay.getDay(); // 0 = Sun

    let prevMonth = -1;
    ctx.font = '10px monospace';
    ctx.fillStyle = '#7a8ba8';
    ctx.textAlign = 'left';

    for (let i = 0; i < days.length; i++) {
        const d = days[i];
        const dt = new Date(d.day + 'T00:00:00');
        const dayOfWeek = dt.getDay(); // Sun=0 … Sat=6
        const col = Math.floor((i + firstColOffset) / 7);
        const x = 24 + col * cell;
        const y = TOP + dayOfWeek * cell;
        // Month label at first column where month changes.
        const m = dt.getMonth();
        if (m !== prevMonth && dayOfWeek <= 2) {
            ctx.fillStyle = '#7a8ba8';
            ctx.fillText(['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'][m], x, TOP - 12);
            prevMonth = m;
        }
        const b = bucketOf(+d.total);
        ctx.fillStyle = COL[b];
        ctx.fillRect(x + pad, y + pad, cell - pad * 2, cell - pad * 2);
        cells.push({ x, y, w: cell, h: cell, day: d.day, total: +d.total, count: d.count });
    }
    // Hover handler: convert mouse to cell, show in legend.
    canvas.onmousemove = (ev) => {
        const rect = canvas.getBoundingClientRect();
        const mx = ev.clientX - rect.left, my = ev.clientY - rect.top;
        for (const c of cells) {
            if (mx >= c.x && mx < c.x + c.w && my >= c.y && my < c.y + c.h) {
                if (hover) hover.textContent = `${c.day} · ${fmtUsd(c.total)} (${c.count})`;
                canvas.style.cursor = 'pointer';
                return;
            }
        }
        if (hover) hover.textContent = '';
        canvas.style.cursor = '';
    };
    canvas.onmouseleave = () => { if (hover) hover.textContent = ''; };
}

// ── Cumulative spend curve via uPlot — the expense analog of an
//   equity curve. One line, year-to-date pace, useful for spotting
//   the month where burn accelerated.
async function renderCumulativeChart(mount, year) {
    const el = mount.querySelector('#tax-cumulative-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    let rows = [];
    try {
        const from = `${year}-01-01`;
        const to = `${year}-12-31`;
        rows = await api.receiptsCumulative({ from, to });
    } catch { rows = []; }
    if (!rows.length) {
        el.innerHTML = `<p class="muted small">${esc(t('view.expenses.tax.chart.empty'))}</p>`;
        return;
    }
    const xs = rows.map(r => Math.floor(new Date(r.day + 'T00:00:00').getTime() / 1000));
    const ys = rows.map(r => Number(r.cumulative) || 0);
    new uPlot({
        width: el.clientWidth || 480,
        height: 200,
        scales: { x: { time: true }, y: { auto: true } },
        series: [
            {},
            { label: t('view.expenses.tax.chart.cumulative'), stroke: '#36c8d4', width: 2,
              fill: 'rgba(54,200,212,0.15)' },
        ],
        axes: [
            { stroke: '#aab' },
            { stroke: '#aab', size: 64,
              values: (_u, splits) => splits.map(v => fmtUsd(v)) },
        ],
    }, [xs, ys], el);
}

// ── Day-of-week bar chart — surfaces "groceries every Sunday" vs
//   "business meals weekday-loaded" patterns. Bars are total $; label
//   includes per-DOW receipt count for context.
async function renderDowChart(mount, year) {
    const canvas = mount.querySelector('#tax-dow');
    if (!canvas) return;
    let rows = [];
    try {
        const from = `${year}-01-01`;
        const to = `${year}-12-31`;
        rows = await api.receiptsDow({ from, to });
    } catch { rows = []; }
    const ctx = canvas.getContext('2d');
    const W = canvas.width, H = canvas.height;
    ctx.clearRect(0, 0, W, H);
    if (!rows.length) {
        ctx.fillStyle = '#7a8ba8';
        ctx.font = '11px monospace';
        ctx.textAlign = 'center';
        ctx.fillText(t('view.expenses.tax.chart.empty'), W / 2, H / 2);
        return;
    }
    // Order Mon-Sun (locale-agnostic readability) by shifting Sun.
    const order = [1, 2, 3, 4, 5, 6, 0];
    const labels = ['Mon','Tue','Wed','Thu','Fri','Sat','Sun'];
    const totals = order.map(i => Number(rows[i]?.total) || 0);
    const counts = order.map(i => rows[i]?.count || 0);
    const max = totals.reduce((m, v) => Math.max(m, v), 0) || 1;
    const padL = 50, padR = 14, padT = 14, padB = 30;
    const plotW = W - padL - padR;
    const plotH = H - padT - padB;
    const barW = Math.floor(plotW / labels.length) - 8;
    ctx.font = '10px monospace';
    ctx.fillStyle = '#7a8ba8';
    ctx.textAlign = 'right';
    // Y-axis labels at 0 / max/2 / max.
    [0, 0.5, 1].forEach((p) => {
        const v = max * p;
        const y = padT + plotH * (1 - p);
        ctx.fillStyle = '#33424f';
        ctx.fillRect(padL, y, plotW, 1);
        ctx.fillStyle = '#7a8ba8';
        ctx.fillText(fmtUsd(v), padL - 6, y + 3);
    });
    ctx.textAlign = 'center';
    for (let i = 0; i < labels.length; i++) {
        const x = padL + i * (barW + 8) + 4;
        const h = Math.round((totals[i] / max) * plotH);
        ctx.fillStyle = '#36c8d4';
        ctx.fillRect(x, padT + plotH - h, barW, h);
        ctx.fillStyle = '#aab';
        ctx.fillText(labels[i], x + barW / 2, H - 14);
        ctx.fillStyle = '#7a8ba8';
        ctx.fillText(String(counts[i]), x + barW / 2, H - 2);
    }
}

// Top merchants by spend for the year — horizontal table with a bar
// inside each row. Click a row to drill in via the Purchases view
// pre-filtered to that merchant (search box on Purchases handles the
// rest).
async function renderTopMerchants(mount, year) {
    const el = mount.querySelector('#tax-top-merchants');
    if (!el) return;
    el.innerHTML = `<div class="muted small"><div class="tv-spinner-wrap"><div class="tv-spinner"></div></div></div>`;
    let rows = [];
    try { rows = await api.topMerchants({ year, limit: 20 }); }
    catch (_) { rows = []; }
    if (!rows.length) {
        el.innerHTML = `<p class="muted small">${esc(t('view.expenses.tax.merchants.empty'))}</p>`;
        return;
    }
    const max = rows.reduce((m, r) => Math.max(m, Number(r.total) || 0), 0) || 1;
    const fmt = (n) => fmtUsd(Number(n) || 0);
    el.innerHTML = `
        <table class="tax-merchants-table">
            <thead>
                <tr>
                    <th>${esc(t('view.expenses.tax.merchants.col.merchant'))}</th>
                    <th class="num">${esc(t('view.expenses.tax.merchants.col.total'))}</th>
                    <th class="num">${esc(t('view.expenses.tax.merchants.col.receipts'))}</th>
                    <th class="num">${esc(t('view.expenses.tax.merchants.col.items'))}</th>
                    <th class="num">${esc(t('view.expenses.tax.merchants.col.business'))}</th>
                    <th class="num">${esc(t('view.expenses.tax.merchants.col.rental'))}</th>
                    <th class="num">${esc(t('view.expenses.tax.merchants.col.personal'))}</th>
                    <th>${esc(t('view.expenses.tax.merchants.col.window'))}</th>
                </tr>
            </thead>
            <tbody>
                ${rows.map(r => {
                    const pct = Math.round((Number(r.total) / max) * 100);
                    return `<tr class="tax-merchants-row" data-merchant="${esc(r.canonical_merchant)}">
                        <td class="tax-merchant-name">
                            <div class="tax-merchant-bar" data-bar-pct="${pct}"></div>
                            <span>${esc(r.canonical_merchant)}</span>
                        </td>
                        <td class="num">${esc(fmt(r.total))}</td>
                        <td class="num">${r.receipt_count}</td>
                        <td class="num">${r.item_count}</td>
                        <td class="num">${esc(fmt(r.business_total))}</td>
                        <td class="num">${esc(fmt(r.rental_total))}</td>
                        <td class="num">${esc(fmt(r.personal_total))}</td>
                        <td class="muted small">${esc(r.first_date || '')} → ${esc(r.last_date || '')}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
    // Tauri-release-safe bar widths — see util.applyBarWidths.
    applyBarWidths(el);
    el.querySelectorAll('tr.tax-merchants-row').forEach(tr => {
        tr.addEventListener('click', () => {
            const m = tr.dataset.merchant || '';
            if (!m) return;
            // Hop to Purchases pre-filtered. Purchases reads the
            // ?search= query string on init.
            location.hash = `#purchases?search=${encodeURIComponent(m)}`;
        });
    });
}

function hexAlpha(hex, a) {
    // Accept `#rrggbb` or named-ish; convert to rgba(r,g,b,a). Falls
    // back to a muted grey if parsing fails.
    if (typeof hex !== 'string' || hex[0] !== '#' || hex.length !== 7) {
        return `rgba(150,150,160,${a})`;
    }
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    return `rgba(${r},${g},${b},${a})`;
}

// Browser print-to-PDF — `window.print()` on a freshly-opened
// document populated with a clean tax-summary or tax-detail markup
// and a print-only stylesheet. User then picks "Save as PDF" in the
// system print dialog. Avoids dragging in a JS PDF library (jsPDF
// ~200 KB + DOM canvas dance) and produces a much better-looking
// output via the browser's print engine.
async function exportTaxPdfSummary() {
    const { from, to } = currentTaxRange();
    let rollup = null;
    try { rollup = await api.taxRollup({ from, to }); } catch (_) {}
    const year = (state.taxPeriod || {}).year || new Date().getFullYear();
    const html = renderPdfHtml({
        title: t('view.expenses.pdf.summary_title', { year }),
        period: `${from} → ${to}`,
        rollup,
        detailRows: null,
    });
    openPrintWindow(html);
}
async function exportTaxPdfDetail() {
    const { from, to } = currentTaxRange();
    let rollup = null;
    try { rollup = await api.taxRollup({ from, to }); } catch (_) {}
    // Pull every transaction in window for the detail listing. Capped
    // to avoid a 1M-row PDF; user can narrow via filters.
    let txs = [];
    try { txs = await api.expenseTransactions({ from, to, limit: 5000 }); } catch (_) {}
    const year = (state.taxPeriod || {}).year || new Date().getFullYear();
    const html = renderPdfHtml({
        title: t('view.expenses.pdf.detail_title', { year }),
        period: `${from} → ${to}`,
        rollup,
        detailRows: txs || [],
    });
    openPrintWindow(html);
}
function renderPdfHtml({ title, period, rollup, detailRows }) {
    const fmt = (v) => {
        const n = Number(v);
        return Number.isFinite(n) ? `$${n.toFixed(2)}` : '—';
    };
    const cCats = rollup?.business?.categories || [];
    const eProps = rollup?.rental?.properties || [];
    const personal = rollup?.personal?.grand_total || 0;
    const uncl = rollup?.unclassified?.grand_total || 0;
    const income = (detailRows || []).filter(r => Number(r.amount) > 0)
        .reduce((a, r) => a + Number(r.amount || 0), 0);
    const detailHtml = Array.isArray(detailRows) && detailRows.length
        ? `<h2>${esc(t('view.expenses.pdf.detail_section_title'))}</h2>
           <table class="pdf-tbl">
             <thead><tr>
                <th>${esc(t('view.expenses.col.date'))}</th>
                <th>${esc(t('view.expenses.col.merchant'))}</th>
                <th>${esc(t('view.expenses.col.category'))}</th>
                <th style="text-align:right">${esc(t('view.expenses.col.amount'))}</th>
             </tr></thead>
             <tbody>${detailRows.map(r => `
                <tr>
                    <td>${esc((r.posted_at || '').slice(0, 10))}</td>
                    <td>${esc(r.merchant_raw || '')}</td>
                    <td>${esc(r.category_name || r.category_id || '')}</td>
                    <td style="text-align:right">${fmt(r.amount)}</td>
                </tr>`).join('')}</tbody>
           </table>` : '';
    return `<!doctype html><html><head><title>${esc(title)}</title>
        <meta charset="utf-8">
        <style>
            body { font-family: -apple-system, sans-serif; color: #111; margin: 24px; }
            h1 { font-size: 22px; margin: 0 0 4px; border-bottom: 2px solid #000; padding-bottom: 4px; }
            h2 { font-size: 14px; margin: 18px 0 6px; color: #333; text-transform: uppercase; letter-spacing: 0.06em; }
            .pdf-period { color: #555; font-size: 11px; margin-bottom: 16px; }
            .pdf-stat-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; margin-bottom: 16px; }
            .pdf-stat { border: 1px solid #999; padding: 8px; }
            .pdf-stat-label { font-size: 9px; text-transform: uppercase; color: #666; letter-spacing: 0.06em; }
            .pdf-stat-value { font-family: monospace; font-size: 16px; font-weight: 700; margin-top: 4px; }
            table.pdf-tbl { width: 100%; border-collapse: collapse; font-size: 10px; margin-bottom: 12px; }
            table.pdf-tbl th, table.pdf-tbl td { border: 1px solid #ddd; padding: 3px 6px; }
            table.pdf-tbl thead { background: #f0f0f0; }
            .pdf-prop { margin-bottom: 12px; }
            .pdf-prop-name { font-weight: 700; font-size: 11px; margin-bottom: 4px; }
            @media print { @page { size: letter; margin: 0.5in; } }
        </style></head><body>
        <h1>${esc(title)}</h1>
        <div class="pdf-period">${esc(period)} · TraderView Tax Export</div>
        <div class="pdf-stat-grid">
            <div class="pdf-stat"><div class="pdf-stat-label">Income</div><div class="pdf-stat-value">${fmt(income)}</div></div>
            <div class="pdf-stat"><div class="pdf-stat-label">Schedule C</div><div class="pdf-stat-value">${fmt(rollup?.business?.grand_total || 0)}</div></div>
            <div class="pdf-stat"><div class="pdf-stat-label">Schedule E</div><div class="pdf-stat-value">${fmt(rollup?.rental?.grand_total || 0)}</div></div>
            <div class="pdf-stat"><div class="pdf-stat-label">Personal + Unclassified</div><div class="pdf-stat-value">${fmt(personal + uncl)}</div></div>
        </div>
        <h2>Schedule C (Business)</h2>
        ${cCats.length ? `<table class="pdf-tbl">
            <thead><tr><th>Line</th><th>Category</th><th style="text-align:right">Total</th></tr></thead>
            <tbody>${cCats.map(c => `<tr>
                <td>${c.schedule_c_line ? 'C' + esc(c.schedule_c_line) : '—'}</td>
                <td>${esc(t('view.expenses.cat.' + c.category))}</td>
                <td style="text-align:right">${fmt(c.total)}</td>
            </tr>`).join('')}</tbody>
        </table>` : '<p style="color:#777">(no business items in range)</p>'}
        <h2>Schedule E (Rental) — per property</h2>
        ${eProps.length ? eProps.map(p => `
            <div class="pdf-prop">
                <div class="pdf-prop-name">${esc(p.property_name || '(unassigned)')} — ${fmt(p.grand_total)}</div>
                <table class="pdf-tbl">
                    <thead><tr><th>Line</th><th>Category</th><th style="text-align:right">Total</th></tr></thead>
                    <tbody>${(p.categories || []).map(c => `<tr>
                        <td>${c.schedule_e_line ? 'E' + esc(c.schedule_e_line) : '—'}</td>
                        <td>${esc(t('view.expenses.cat.' + c.category))}</td>
                        <td style="text-align:right">${fmt(c.total)}</td>
                    </tr>`).join('')}</tbody>
                </table>
            </div>`).join('') : '<p style="color:#777">(no rental items in range)</p>'}
        ${detailHtml}
        </body></html>`;
}
function openPrintWindow(html) {
    const w = window.open('', '_blank', 'width=900,height=1100');
    if (!w) {
        showToast(t('view.expenses.pdf.popup_blocked'), { level: 'error' });
        return;
    }
    w.document.write(html);
    w.document.close();
    // Defer print() so the document fully renders + the user's pop-up
    // chrome settles. The browser dialog includes "Save as PDF".
    setTimeout(() => { try { w.print(); } catch (_) {} }, 250);
}

async function refresh() {
    const params = {
        limit: 500,
        ...(state.currentAccountId ? { account_id: state.currentAccountId } : {}),
        ...(state.filters.from ? { from: state.filters.from } : {}),
        ...(state.filters.to ? { to: state.filters.to } : {}),
        ...(state.filters.category && state.filters.category !== '__none__'
            ? { category: state.filters.category } : {}),
        ...(state.filters.is_business !== '' ? { is_business: state.filters.is_business } : {}),
        ...(state.filters.search ? { search: state.filters.search } : {}),
    };
    try {
        let rows = await api.expenseTransactions(params);
        if (!viewIsCurrent(state.tok)) return;
        if (state.filters.category === '__none__') {
            rows = rows.filter(r => !r.category_code);
        }
        state.transactions = rows;
        drawTable();
    } catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        const tbl = state.mount.querySelector('#exp-table');
        if (tbl) tbl.innerHTML = `<p class="boot">${esc(t('view.expenses.boot.transactions_failed', { err: e.message }))}</p>`;
    }
}

function drawTable() {
    const host = state.mount.querySelector('#exp-table');
    if (!host) return;
    if (!state.transactions.length) {
        host.innerHTML = `<p data-i18n="view.expenses.hint.no_transactions_upload_a_csv" class="boot">no transactions. upload a CSV.</p>`;
        return;
    }
    const acctNames = Object.fromEntries(state.accounts.map(a => [a.id, a.name]));

    const catOptsBase = '<option data-i18n="view.expenses.opt.uncategorized_2" value="">(uncategorized)</option>' +
        state.categories
            .map(c => `<option value="${c.code}">${c.schedule_c_line}. ${esc(c.label)}</option>`)
            .join('');

    const rows = state.transactions.map(t => {
        const amt = Number(t.amount);
        const cls = amt < 0 ? 'pnl-neg' : 'pnl-pos';
        const xferCls = t.is_transfer ? 'tx-transfer' : '';
        const bizClass = t.is_business ? 'biz-on' : 'biz-off';
        const catSel = state.categories
            .map(c =>
                `<option value="${c.code}"${c.code === t.category_code ? ' selected' : ''}>` +
                `${c.schedule_c_line}. ${esc(c.label)}</option>`)
            .join('');
        return `
        <tr class="${xferCls}" data-tx="${t.id}">
            <td>${t.posted_at.slice(0, 10)}</td>
            <td>${esc(acctNames[t.account_id] || t.account_id.slice(0, 8))}</td>
            <td title="${esc(t.merchant_raw)}">${esc(t.merchant_raw)}</td>
            <td class="${cls}">${amt.toFixed(2)}</td>
            <td>
                <select class="exp-cat" data-tx="${t.id}">
                    <option data-i18n="view.expenses.opt.uncategorized_3" value=""${t.category_code ? '' : ' selected'}>(uncategorized)</option>
                    ${catSel}
                </select>
            </td>
            <td>
                <button class="tx-biz ${bizClass}" data-tx="${t.id}" data-biz="${t.is_business}" data-i18n="view.expenses.btn.${t.is_business ? 'biz' : 'pers'}">
                    ${t.is_business ? 'BIZ' : 'pers'}
                </button>
            </td>
            <td>
                <button class="tx-xfer ${t.is_transfer ? 'biz-on' : ''}"
                        data-tx="${t.id}" data-xfer="${t.is_transfer}"${t.is_transfer ? ' data-i18n="view.expenses.btn.xfer"' : ''}>
                    ${t.is_transfer ? 'XFER' : '—'}
                </button>
            </td>
        </tr>`;
    }).join('');

    host.innerHTML = `
        <table class="trades expense-table">
            <thead><tr>
                <th data-i18n="view.expenses.th.date">Date</th><th data-i18n="view.expenses.th.account">Account</th><th data-i18n="view.expenses.th.merchant">Merchant</th>
                <th data-i18n="view.expenses.th.amount">Amount</th><th data-i18n="view.expenses.th.category_schedule_c">Category (Schedule C)</th><th data-i18n="view.expenses.th.biz">Biz?</th><th data-i18n="view.expenses.th.transfer">Transfer?</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
        <div class="chart-panel">
            <h2 data-i18n="view.expenses.h2.cat_chart">Total spend by category (top 10)</h2>
            <div id="exp-chart" style="width:100%;height:240px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.expenses.h2.month_chart">Monthly spend trend (excluding transfers)</h2>
            <div id="exp-month-chart" style="width:100%;height:220px"></div>
        </div>`;
    renderCategoryChart();
    renderMonthChart();

    host.querySelectorAll('select.exp-cat').forEach(sel => {
        sel.addEventListener('change', async ev => {
            const tx = ev.target.dataset.tx;
            const code = ev.target.value || null;
            try {
                await api.updateExpenseTransaction(tx, { category_code: code });
            } catch (e) {
                showToast(t('view.expenses.alert.update_failed', { err: e.message }), { level: 'error' });
            }
        });
    });
    host.querySelectorAll('button.tx-biz').forEach(btn => {
        btn.addEventListener('click', async () => {
            const tx = btn.dataset.tx;
            const next = btn.dataset.biz !== 'true';
            try {
                await api.updateExpenseTransaction(tx, { is_business: next });
                btn.dataset.biz = String(next);
                btn.textContent = next ? t('view.expenses.btn.biz') : t('view.expenses.btn.pers');
                btn.classList.toggle('biz-on', next);
                btn.classList.toggle('biz-off', !next);
            } catch (e) { showToast(t('view.expenses.alert.update_failed', { err: e.message }), { level: 'error' }); }
        });
    });
    host.querySelectorAll('button.tx-xfer').forEach(btn => {
        btn.addEventListener('click', async () => {
            const tx = btn.dataset.tx;
            const next = btn.dataset.xfer !== 'true';
            try {
                await api.updateExpenseTransaction(tx, { is_transfer: next });
                btn.dataset.xfer = String(next);
                btn.textContent = next ? t('view.expenses.btn.xfer') : '—';
                btn.classList.toggle('biz-on', next);
            } catch (e) { showToast(t('view.expenses.alert.update_failed', { err: e.message }), { level: 'error' }); }
        });
    });
}

async function createAccountFlow() {
    const name = await tPrompt('view.expenses.prompt.account_name', {});
    if (!name) return;
    const kind = await tPrompt('view.expenses.prompt.kind', {}, { defaultValue: 'credit_card' });
    if (!['bank', 'credit_card', 'marketplace'].includes(kind)) {
        showToast(t('view.expenses.alert.invalid_kind'), { level: 'error' });
        return;
    }
    const source = await tPrompt('view.expenses.prompt.source', {}, { defaultValue: 'chase' });
    if (!source) return;
    try {
        const acct = await api.createExpenseAccount({ kind, source, name });
        if (!viewIsCurrent(state.tok)) return;
        state.accounts.push(acct);
        state.currentAccountId = acct.id;
        // Redraw shell so the account picker re-renders with the new option.
        drawShell(state.mount);
        const sel = state.mount.querySelector('#exp-account');
        if (sel) sel.value = acct.id;
        await refresh();
    } catch (e) {
        showToast(t('view.expenses.alert.create_failed', { err: e.message }), { level: 'error' });
    }
}

async function handleUpload(ev) {
    const file = ev.target.files && ev.target.files[0];
    ev.target.value = '';
    if (!file) return;
    if (!state.currentAccountId) {
        showToast(t('view.expenses.alert.pick_account'), { level: 'warning' });
        return;
    }
    const source = state.mount.querySelector('#exp-source').value;
    let mapping = null;
    if (source === 'generic') {
        const num = (id) => Math.max(0, Math.round(Number(state.mount.querySelector(id).value) || 0));
        mapping = {
            date_col: num('#gm-date'),
            amount_col: num('#gm-amount'),
            description_col: num('#gm-desc'),
            has_header: state.mount.querySelector('#gm-header').checked,
            negate_amount: state.mount.querySelector('#gm-negate').checked,
        };
    }
    const status = state.mount.querySelector('#exp-status');
    if (status) status.textContent = t('view.expenses.status.uploading', { name: file.name });
    try {
        const res = await api.importExpense(state.currentAccountId, source, file, mapping);
        if (!viewIsCurrent(state.tok)) return;
        const status2 = state.mount.querySelector('#exp-status');
        if (res.duplicate) {
            if (status2) status2.textContent = t('view.expenses.status.duplicate', { id: res.import_id.slice(0, 8) });
        } else {
            if (status2) status2.textContent = t('view.expenses.status.imported', {
                inserted: res.inserted_count,
                total: res.row_count,
                categorized: res.categorized_count,
                pairs: res.transfer_pairs,
            });
        }
        await refresh();
    } catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        const status2 = state.mount.querySelector('#exp-status');
        if (status2) {
            if (e instanceof ApiError && e.status === 400) {
                status2.textContent = t('view.expenses.status.parser_err', { err: e.message });
            } else {
                status2.textContent = t('view.expenses.status.upload_err', { err: e.message });
            }
        }
    }
}

async function seedRulesFlow() {
    const status = state.mount.querySelector('#exp-status');
    try {
        const res = await api.seedExpenseRules();
        if (!viewIsCurrent(state.tok)) return;
        const status2 = state.mount.querySelector('#exp-status');
        if (status2) status2.textContent = res.skipped_existing
            ? t('view.expenses.status.seed_skipped', { n: res.skipped_existing })
            : t('view.expenses.status.seed_ok', { n: res.inserted });
    } catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        const status2 = state.mount.querySelector('#exp-status');
        if (status2) status2.textContent = t('view.expenses.status.seed_err', { err: e.message });
    }
}

async function openRulesModal() {
    const modal = state.mount.querySelector('#exp-rules-modal');
    if (!modal) return;
    modal.classList.remove('hidden');
    modal.innerHTML = '<div class="modal-inner"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.status.loading">loading…</div></div></div>';
    let rules = [];
    try { rules = await api.expenseRules(); }
    catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        modal.innerHTML = `<div class="modal-inner"><p class="boot">${esc(t('view.expenses.boot.load_failed', { err: e.message }))}</p>
            <button data-i18n="view.expenses.btn.close" id="rules-close">Close</button></div>`;
        modal.querySelector('#rules-close').onclick = () => modal.classList.add('hidden');
        return;
    }
    if (!viewIsCurrent(state.tok)) return;
    const catOpts = state.categories
        .map(c => `<option value="${c.code}">${c.schedule_c_line}. ${esc(c.label)}</option>`)
        .join('');

    const rows = rules.map(r => `
        <tr>
            <td>${r.priority}</td>
            <td>${esc(r.pattern)}</td>
            <td>${r.pattern_kind}</td>
            <td>${r.category_code}</td>
            <td>${r.is_business ? 'biz' : 'pers'}</td>
            <td>${r.hit_count}</td>
            <td><button data-i18n="view.expenses.btn.delete" class="rule-del" data-id="${r.id}">delete</button></td>
        </tr>`).join('');

    modal.innerHTML = `
    <div class="modal-inner wide">
        <h2 data-i18n="view.expenses.h2.merchant_rules">Merchant rules</h2>
        <form id="rule-form" class="rule-form">
            <input name="pattern" placeholder="pattern (e.g. uber)" data-i18n-placeholder="view.expenses.placeholder.pattern" required>
            <select name="pattern_kind">
                <option data-i18n="view.expenses.opt.substring" value="substring">substring</option>
                <option data-i18n="view.expenses.opt.regex" value="regex">regex</option>
            </select>
            <select name="category_code">${catOpts}</select>
            <label><input type="checkbox" name="is_business" checked> <span data-i18n="view.expenses.label.biz">biz</span></label>
            <input name="priority" type="number" value="100" style="width:60px">
            <label><input type="checkbox" name="apply_retroactively" checked> <span data-i18n="view.expenses.label.apply_now">apply now</span></label>
            <button data-i18n="view.expenses.btn.add" class="primary" type="submit">add</button>
        </form>
        <table class="trades">
            <thead><tr><th data-i18n="view.expenses.th.pri">Pri</th><th data-i18n="view.expenses.th.pattern">Pattern</th><th data-i18n="view.expenses.th.kind">Kind</th><th data-i18n="view.expenses.th.cat">Cat</th><th data-i18n="view.expenses.th.biz_2">Biz?</th><th data-i18n="view.expenses.th.hits">Hits</th><th></th></tr></thead>
            <tbody>${rows || `<tr><td colspan="7" class="boot">${esc(t('view.expenses.empty.no_rules'))}</td></tr>`}</tbody>
        </table>
        <button data-i18n="view.expenses.btn.close_2" id="rules-close" style="margin-top:12px">Close</button>
    </div>`;
    modal.querySelector('#rules-close').onclick = () => modal.classList.add('hidden');
    modal.querySelectorAll('.rule-del').forEach(btn => {
        btn.onclick = async () => {
            if (!await tConfirm('view.expenses.confirm.delete_rule', {}, { level: 'danger' })) return;
            try { await api.deleteExpenseRule(btn.dataset.id); }
            catch (e) { showToast(t('view.expenses.alert.delete_failed', { err: e.message }), { level: 'error' }); return; }
            if (!viewIsCurrent(state.tok)) return;
            openRulesModal();
        };
    });
    modal.querySelector('#rule-form').addEventListener('submit', async ev => {
        ev.preventDefault();
        const fd = new FormData(ev.target);
        try {
            await api.createExpenseRule({
                pattern: fd.get('pattern'),
                pattern_kind: fd.get('pattern_kind'),
                category_code: fd.get('category_code'),
                is_business: fd.get('is_business') === 'on',
                priority: Number(fd.get('priority')) || 100,
                apply_retroactively: fd.get('apply_retroactively') === 'on',
            });
            if (!viewIsCurrent(state.tok)) return;
            await refresh();
            openRulesModal();
        } catch (e) { showToast(t('view.expenses.alert.create_failed', { err: e.message }), { level: 'error' }); }
    });
}

function esc(s) {
    return String(s == null ? '' : s)
        .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;').replace(/'/g, '&#39;');
}

// --- receipt drag-drop + OCR poll + match modal --------------------------

// Unified drop zone. Files arriving here get sorted by extension /
// MIME and routed:
//   * receipts (jpg / png / webp / heic / pdf-image) → OCR pipeline
//   * statements (csv / xlsx / xls / ods / pdf-text) → handleUploadFiles
// PDF disambiguation is best-effort — most card-statement PDFs land in
// the statement bucket; OCR'd photo PDFs land in receipts. We default
// PDF to "statement" because that's what 99% of mixed batches contain;
// the user can manually drag a single PDF into the receipts library
// page if it's a photo.
function isStatementFile(file) {
    const ext = fileExt(file);
    if (ext === 'csv' || ext === 'xlsx' || ext === 'xls' || ext === 'ods') return true;
    if (ext === 'pdf') return true;
    const m = (file.type || '').toLowerCase();
    if (m === 'text/csv' || m.startsWith('application/vnd.openxmlformats')) return true;
    return false;
}
function bindReceiptDropzone() {
    const dz = state.mount.querySelector('#receipt-dz');
    const picker = state.mount.querySelector('#receipt-file');
    if (!dz || !picker) return;
    // Click on the dropzone — opens whichever picker matches the
    // currently-active upload tab. `auto` defaults to receipts (same
    // as drop, which auto-routes by file type).
    dz.addEventListener('click', () => {
        const tab = state.uploadTab || 'auto';
        let target;
        if (tab === 'statements') target = state.mount.querySelector('#exp-file');
        else if (tab === 'folder') target = state.mount.querySelector('#receipt-folder');
        else target = picker;       // auto / receipts both go to the image picker
        if (target) target.click();
    });
    ['dragenter', 'dragover'].forEach(ev =>
        dz.addEventListener(ev, e => { e.preventDefault(); dz.classList.add('dragover'); }));
    ['dragleave', 'drop'].forEach(ev =>
        dz.addEventListener(ev, e => { e.preventDefault(); dz.classList.remove('dragover'); }));
    dz.addEventListener('drop', async (e) => {
        const files = Array.from(e.dataTransfer.files || []);
        if (!files.length) return;
        // Sort into receipts vs statements. Statements get fed to the
        // existing statement-import handler one at a time; receipts go
        // through the batch OCR pipeline.
        const receipts = [], statements = [];
        for (const f of files) {
            if (isStatementFile(f) && fileExt(f) !== 'jpg' && fileExt(f) !== 'jpeg' && fileExt(f) !== 'png') {
                statements.push(f);
            } else {
                receipts.push(f);
            }
        }
        if (receipts.length) await receiptUploadAll(receipts);
        for (const f of statements) {
            await handleStatementFile(f);
        }
    });
    picker.addEventListener('change', () => {
        receiptUploadAll(picker.files);
        picker.value = '';
    });
}
async function handleStatementFile(file) {
    // Re-uses the existing #exp-file → #exp-upload handler via a
    // synthetic FileList. Minimal — keeps the import code path single-
    // sourced through handleUpload (which the toolbar still triggers).
    const inp = state.mount.querySelector('#exp-file');
    if (!inp) return;
    // DataTransfer is the canonical way to build a FileList programmatically.
    const dt = new DataTransfer();
    dt.items.add(file);
    inp.files = dt.files;
    inp.dispatchEvent(new Event('change', { bubbles: true }));
}

// Receipt-upload status helpers. `setBusyStatus(msg)` renders an inline
// cyan spinner alongside the message so the user knows work is in flight;
// `setStatus(msg)` is the terminal/idle state with no spinner. Both share
// the `#exp-status` slot so HTML reset is unconditional and never leaks
// the previous frame's spinner element.
function expStatusSlot() {
    return state.mount && state.mount.querySelector('#exp-status');
}
function setStatus(txt) {
    const s = expStatusSlot();
    if (s) s.textContent = txt;
}
function setBusyStatus(txt) {
    const s = expStatusSlot();
    if (!s) return;
    s.innerHTML = '';
    const sp = document.createElement('span');
    sp.className = 'tv-spinner tv-spinner-inline';
    s.appendChild(sp);
    const t = document.createElement('span');
    t.textContent = ' ' + txt;
    s.appendChild(t);
}

// Pre-upload image compressor. Phone receipts are typically 8-15 MB
// JPEG; at 10k receipts that's 100-150 GB on disk. Downscale the long
// side to 2400 px (more than enough for Tesseract — the OCR pipeline
// already upscales SHORT side to 1600 px) at quality 85. Pure-canvas
// JPEG re-encode. Skips PDFs, HEIC (already JPEG'd by heicToJpeg),
// and tiny files where compression wouldn't help.
const COMPRESS_THRESHOLD_BYTES = 600 * 1024;     // ≥600 KB triggers
const COMPRESS_LONG_SIDE_PX = 2400;
const COMPRESS_QUALITY = 0.85;
async function maybeCompress(file) {
    if (file.size < COMPRESS_THRESHOLD_BYTES) return file;
    const m = (file.type || '').toLowerCase();
    if (m === 'application/pdf' || fileExt(file) === 'pdf') return file;
    // After heicToJpeg() the input is a 0.92-quality JPEG already; a
    // second re-encode is harmless but the file's already been
    // produced by us, so re-compress only when it's large.
    if (!m.startsWith('image/')) return file;
    const url = URL.createObjectURL(file);
    try {
        const img = new Image();
        img.src = url;
        await img.decode();
        const w = img.naturalWidth, h = img.naturalHeight;
        if (Math.max(w, h) <= COMPRESS_LONG_SIDE_PX) return file;
        const scale = COMPRESS_LONG_SIDE_PX / Math.max(w, h);
        const nw = Math.round(w * scale), nh = Math.round(h * scale);
        const canvas = document.createElement('canvas');
        canvas.width = nw;
        canvas.height = nh;
        const ctx = canvas.getContext('2d');
        if (!ctx) return file;
        ctx.drawImage(img, 0, 0, nw, nh);
        const blob = await new Promise((resolve) => {
            canvas.toBlob(b => resolve(b), 'image/jpeg', COMPRESS_QUALITY);
        });
        if (!blob || blob.size >= file.size) return file;   // re-encode bigger? bail
        const base = (file.name || 'receipt').replace(/\.[^.]+$/, '');
        return new File([blob], `${base}.jpg`, {
            type: 'image/jpeg',
            lastModified: file.lastModified,
        });
    } catch (_) {
        // Decode failure — pass the original through. Backend will
        // reject it cleanly if it's truly unreadable.
        return file;
    } finally {
        URL.revokeObjectURL(url);
    }
}

async function receiptUploadAll(fileList) {
    if (!fileList || !fileList.length) return;
    for (const file of fileList) {
        let toUpload = file;
        if (isHeicFile(file)) {
            setBusyStatus(t('view.expenses.receipt.converting', { name: file.name }));
            try {
                toUpload = await heicToJpeg(file);
            } catch (e) {
                if (!viewIsCurrent(state.tok)) return;
                setStatus(t('view.expenses.receipt.heic_err', { name: file.name, err: e.message }));
                continue;
            }
        }
        toUpload = await maybeCompress(toUpload);
        setBusyStatus(t('view.expenses.receipt.uploading', { name: toUpload.name }));
        try {
            const r = await api.uploadReceipt(toUpload);
            if (!viewIsCurrent(state.tok)) return;
            setBusyStatus(t('view.expenses.receipt.uploaded_ocr', { name: toUpload.name }));
            pollReceiptUntilReady(r.id);
        } catch (e) {
            if (!viewIsCurrent(state.tok)) return;
            setStatus(t('view.expenses.receipt.upload_err', { err: e.message }));
        }
    }
}

const RECEIPT_EXTS = ['jpg', 'jpeg', 'png', 'webp', 'bmp', 'pdf', 'heic', 'heif'];
const HEIC_EXTS = new Set(['heic', 'heif']);

function fileExt(file) {
    const name = (file.name || '').toLowerCase();
    const dot = name.lastIndexOf('.');
    return dot >= 0 ? name.slice(dot + 1) : '';
}

function isReceiptFile(file) {
    return RECEIPT_EXTS.includes(fileExt(file));
}

function isHeicFile(file) {
    if (HEIC_EXTS.has(fileExt(file))) return true;
    const m = (file.type || '').toLowerCase();
    return m === 'image/heic' || m === 'image/heif'
        || m === 'image/heic-sequence' || m === 'image/heif-sequence';
}

// Convert a HEIC / HEIF File to a JPEG Blob+File via the WebView's native
// image decoder. macOS Tauri uses WebKit, which decodes HEIC natively for
// `<img>` since Safari 17 — no external library required. On Chromium-
// based platforms (Linux Tauri, web mode on Chrome) HEIC decode fails and
// we throw a clear message so the caller can surface it.
async function heicToJpeg(file) {
    const url = URL.createObjectURL(file);
    try {
        const img = new Image();
        img.src = url;
        // `Image.decode()` returns a Promise that rejects if the format
        // isn't supported by the underlying engine. WebKit on macOS
        // resolves for HEIC; Chromium rejects.
        await img.decode();
        const canvas = document.createElement('canvas');
        canvas.width = img.naturalWidth;
        canvas.height = img.naturalHeight;
        const ctx = canvas.getContext('2d');
        if (!ctx) throw new Error('no 2d canvas context');
        ctx.drawImage(img, 0, 0);
        const blob = await new Promise((resolve, reject) => {
            canvas.toBlob(b => b ? resolve(b) : reject(new Error('toBlob returned null')),
                          'image/jpeg', 0.92);
        });
        const baseName = (file.name || 'receipt').replace(/\.(heic|heif)$/i, '');
        return new File([blob], `${baseName}.jpg`, { type: 'image/jpeg', lastModified: file.lastModified });
    } finally {
        URL.revokeObjectURL(url);
    }
}

// Scan a chosen directory: upload every receipt file found (recursively, via
// webkitdirectory) through the OCR pipeline. Non-receipt files are skipped and
// the backend dedups by sha256, so re-scanning a folder is idempotent.
// Per-folder progress persistence keyed by folder name. The session
// tracks the SHAs of files already submitted; on resume we skip those.
// Browser-tab-close-safe — every successful upload writes the sha
// immediately so a crash mid-scan loses zero work.
function scanSessionKey(folder) { return `tv-scan-progress:${folder}`; }
function loadScanSession(folder) {
    try {
        const raw = localStorage.getItem(scanSessionKey(folder));
        if (!raw) return null;
        return JSON.parse(raw);
    } catch (_) { return null; }
}
function saveScanSession(folder, session) {
    try { localStorage.setItem(scanSessionKey(folder), JSON.stringify(session)); }
    catch (_) { /* private mode / quota — degrade to in-memory */ }
}
function clearScanSession(folder) {
    try { localStorage.removeItem(scanSessionKey(folder)); }
    catch (_) {}
}

async function sha256Hex(buf) {
    // SubtleCrypto is the standard browser path; WebKit supports it.
    const digest = await crypto.subtle.digest('SHA-256', buf);
    return [...new Uint8Array(digest)].map(b => b.toString(16).padStart(2, '0')).join('');
}

async function receiptScanFolder(fileList) {
    const all = Array.from(fileList || []);
    if (!all.length) return;
    const receipts = all.filter(isReceiptFile);
    const skipped = all.length - receipts.length;
    if (!receipts.length) {
        setStatus(t('view.expenses.receipt.scan_none', { skipped }));
        return;
    }
    const folder = (receipts[0].webkitRelativePath || '').split('/')[0] || receipts[0].name;
    const total = receipts.length;
    const tok = state.tok;

    // Resume? If we have a prior session for this folder, offer to
    // continue. Otherwise start fresh.
    let session = loadScanSession(folder);
    if (session && Array.isArray(session.uploaded_shas) && session.uploaded_shas.length) {
        const cont = await tConfirm('view.expenses.receipt.resume_prompt', {
            folder, done: session.uploaded_shas.length, total,
        });
        if (cont) {
            session = { ...session, uploaded_shas: new Set(session.uploaded_shas) };
        } else {
            clearScanSession(folder);
            session = null;
        }
    } else {
        session = null;
    }
    if (!session) {
        session = { folder, started_at: Date.now(), uploaded_shas: new Set() };
        saveScanSession(folder, {
            ...session,
            uploaded_shas: [...session.uploaded_shas],
        });
    }

    let uploaded = session.uploaded_shas.size, failed = 0, done = session.uploaded_shas.size;
    setBusyStatus(t('view.expenses.receipt.scanning', { folder, done, total }));

    let idx = 0;
    const persist = () => saveScanSession(folder, {
        folder, started_at: session.started_at,
        uploaded_shas: [...session.uploaded_shas],
    });
    const worker = async () => {
        while (idx < receipts.length) {
            const file = receipts[idx++];
            if (!viewIsCurrent(tok)) return;
            // Hash the original file bytes for resume tracking. Backend
            // dedupes on the COMPRESSED sha — we hash here for client
            // dedupe; even if the compressed sha differs, our resume
            // check is "did we already submit this source file?".
            let sha;
            try {
                sha = await sha256Hex(await file.arrayBuffer());
            } catch (_) { sha = `${file.name}:${file.size}:${file.lastModified}`; }
            if (session.uploaded_shas.has(sha)) {
                done++;
                if (!viewIsCurrent(tok)) return;
                setBusyStatus(t('view.expenses.receipt.scanning', { folder, done, total }));
                continue;
            }
            let toUpload = file;
            if (isHeicFile(file)) {
                try { toUpload = await heicToJpeg(file); }
                catch (_) { failed++; done++; continue; }
            }
            toUpload = await maybeCompress(toUpload);
            try {
                await api.uploadReceipt(toUpload);
                uploaded++;
                session.uploaded_shas.add(sha);
                persist();
            } catch (_) {
                failed++;
            }
            done++;
            if (!viewIsCurrent(tok)) return;
            setBusyStatus(t('view.expenses.receipt.scanning', { folder, done, total }));
        }
    };
    const CONCURRENCY = 3;
    await Promise.all(Array.from({ length: Math.min(CONCURRENCY, total) }, worker));
    if (!viewIsCurrent(tok)) return;

    // Folder finished cleanly — purge the resume session so a future
    // re-scan of the same folder starts fresh (the backend's sha dedup
    // still skips already-uploaded blobs).
    clearScanSession(folder);

    setStatus(t('view.expenses.receipt.scan_done', { folder, uploaded, failed, skipped }));
    showToast(t('view.expenses.receipt.scan_done', { folder, uploaded, failed, skipped }),
        { level: failed ? 'warn' : 'success' });
    openReceiptsModal();
}

// Render an inline action prompt when a receipt failed due to missing
// PaddleOCR model files. One click downloads ~110 MB to the platform's
// app data dir, then re-runs OCR on the same blob.
function renderModelsMissingPrompt(receiptId, err) {
    const slot = state.mount.querySelector('#exp-status');
    if (!slot) return;
    slot.innerHTML = '';
    const note = document.createElement('span');
    note.className = 'muted';
    note.textContent = t('view.expenses.receipt.models_missing_note', { err });
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'btn btn-primary btn-compact';
    btn.style.marginLeft = '8px';
    btn.textContent = t('view.expenses.receipt.btn.download_models');
    btn.addEventListener('click', async () => {
        // Show the spinner immediately + disable the button to prevent
        // double-firing. Re-rendering the slot via setBusyStatus replaces
        // the note + button DOM with the spinner row.
        setBusyStatus(t('view.expenses.receipt.downloading_models'));
        try {
            const r = await api.ocrModelsDownload();
            if (!viewIsCurrent(state.tok)) return;
            const got = (r.downloaded || []).length;
            const mb = (r.bytes_total || 0) / 1024 / 1024;
            setBusyStatus(t('view.expenses.receipt.models_downloaded', {
                files: got, mb: mb.toFixed(1),
            }));
            // Re-queue OCR on the previously-failed receipt without
            // making the user drag-drop the file again.
            await api.retryReceiptOcr(receiptId);
            if (!viewIsCurrent(state.tok)) return;
            pollReceiptUntilReady(receiptId);
        } catch (e) {
            if (!viewIsCurrent(state.tok)) return;
            setStatus(t('view.expenses.receipt.models_download_err', {
                err: e.message || String(e),
            }));
        }
    });
    slot.appendChild(note);
    slot.appendChild(btn);
}

async function pollReceiptUntilReady(receiptId) {
    const maxAttempts = 60;        // 60 * 2s = 2 min ceiling
    // Show the spinner immediately rather than waiting 2s for the first
    // poll — the upload-handler already cleared the previous "uploading"
    // message and we want the user to see something is in flight.
    setBusyStatus(t('view.expenses.receipt.ocr_running'));
    for (let i = 0; i < maxAttempts; i++) {
        await new Promise(r => setTimeout(r, 2000));
        if (!viewIsCurrent(state.tok)) return;
        let meta;
        try { meta = await api.receiptMeta(receiptId); }
        catch (e) {
            if (!viewIsCurrent(state.tok)) return;
            setStatus(t('view.expenses.receipt.poll_err', { err: e.message }));
            return;
        }
        if (!viewIsCurrent(state.tok)) return;
        if (meta.ocr_status === 'pending') {
            // Refresh the spinner each tick so a stray DOM mutation
            // elsewhere can't strip it.
            setBusyStatus(t('view.expenses.receipt.ocr_running'));
            continue;
        }
        if (meta.ocr_status === 'failed') {
            const err = meta.error_message || t('common.status.unknown');
            // PaddleOCR model files weren't on disk. Surface a one-click
            // "Download OCR models (~110 MB)" affordance instead of just
            // dumping the raw error and leaving the user to figure out
            // what to do.
            if (/ocr models not found/i.test(err)) {
                renderModelsMissingPrompt(receiptId, err);
                return;
            }
            setStatus(t('view.expenses.receipt.ocr_failed', { err }));
            return;
        }
        if (meta.ocr_status === 'needs_image') {
            setStatus(t('view.expenses.receipt.ocr_needs_image', { err: meta.error_message }));
            return;
        }
        // done — open match suggestion modal
        setStatus(t('view.expenses.receipt.ocr_done', { merchant: meta.ocr_merchant || '?', total: meta.ocr_total ?? '?', date: meta.ocr_date ?? '?' }));
        openReceiptMatchModal(meta);
        return;
    }
    setStatus(t('view.expenses.receipt.ocr_timeout', { id: receiptId.slice(0, 8) }));
}

export async function openReceiptMatchModal(meta) {
    // Page-agnostic — find the modal anywhere in the document; create
    // one on `<body>` if it's missing (e.g., when called from the
    // receipts library page rather than the expenses page). This lets
    // the same modal flow work cross-page without copying its DOM
    // into every consumer.
    let modal = document.querySelector('#exp-rules-modal');
    if (!modal) {
        modal = document.createElement('div');
        modal.id = 'exp-rules-modal';
        modal.className = 'modal hidden';
        document.body.appendChild(modal);
    }
    // Cross-page entry — `state.tok` and `state.accounts` may be unset
    // (the expenses view's renderer wouldn't have run). Bootstrap a
    // fresh token + lazy-load accounts so the candidate-match table
    // can still display account names.
    if (!state.tok) state.tok = currentViewToken();
    if (!Array.isArray(state.accounts) || !state.accounts.length) {
        try { state.accounts = await api.expenseAccounts(); }
        catch (_) { state.accounts = []; }
    }
    modal.classList.remove('hidden');
    modal.innerHTML = '<div class="modal-inner"><p data-i18n="view.expenses.hint.scoring_candidates" class="boot">scoring candidates…</p></div>';
    let matches = [];
    try { matches = await api.receiptMatches(meta.id); }
    catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        modal.innerHTML = `<div class="modal-inner"><p class="boot">${esc(t('view.expenses.boot.match_failed', { err: e.message }))}</p>
            <button data-i18n="view.expenses.btn.close_3" id="m-close">Close</button></div>`;
        modal.querySelector('#m-close').onclick = () => modal.classList.add('hidden');
        return;
    }
    if (!viewIsCurrent(state.tok)) return;

    const acctNames = Object.fromEntries(state.accounts.map(a => [a.id, a.name]));

    const rows = matches.map(m => `
        <tr>
            <td>${(m.score * 100).toFixed(0)}%</td>
            <td>${m.transaction.posted_at.slice(0, 10)}</td>
            <td>${esc(acctNames[m.transaction.account_id] || '?')}</td>
            <td>${esc(m.transaction.merchant_raw)}</td>
            <td>${Number(m.transaction.amount).toFixed(2)}</td>
            <td><button data-i18n="view.expenses.btn.attach" class="m-pick primary" data-tx="${m.transaction.id}">attach</button></td>
        </tr>`).join('');

    // Structured slice from the JSONB column — items, address, time,
    // subtotal, tax. Falls back to an empty object on older receipts
    // that pre-date migration 0041.
    const ex = (meta.ocr_extracted && typeof meta.ocr_extracted === 'object')
        ? meta.ocr_extracted : {};
    const fmtMoney = (v) => {
        if (v == null || v === '') return '—';
        const n = Number(v);
        return Number.isFinite(n) ? `$${n.toFixed(2)}` : esc(String(v));
    };
    // Lazy-load rental properties for the property picker — small list
    // typically (1-10), cheap to cache for the lifetime of the modal.
    let rentalPropsCache = null;
    const getRentalProps = async () => {
        if (rentalPropsCache) return rentalPropsCache;
        try { rentalPropsCache = await api.rentalProperties(); }
        catch (_) { rentalPropsCache = []; }
        return rentalPropsCache;
    };

    const BUCKETS = ['business', 'rental', 'personal', 'unclassified'];
    const renderBucketSelect = (idx, current) => `
        <select class="item-bucket" data-idx="${idx}">
            ${BUCKETS.map(b => `<option value="${b}"${b === current ? ' selected' : ''}>${esc(t('view.expenses.bucket.' + b))}</option>`).join('')}
        </select>`;
    // Every column is editable. Inputs fire `change` → PATCH; the
    // category tag stays click-to-edit (existing behavior wired
    // below). First column carries a `✕` delete button.
    const editableRow = (it, idx) => `
        <tr data-item-idx="${idx}">
            <td>
                <button type="button" class="item-del-btn" data-del-idx="${idx}"
                        title="${esc(t('view.expenses.items.delete'))}">✕</button>
                <input type="text" class="item-name-input" data-idx="${idx}"
                       value="${esc(it.name || '')}"
                       placeholder="${esc(t('view.expenses.items.name_placeholder'))}">
            </td>
            <td class="num"><input type="number" step="0.01" class="item-num-input"
                       data-field="qty" data-idx="${idx}"
                       value="${it.qty != null ? esc(String(it.qty)) : ''}" placeholder="—"></td>
            <td class="num"><input type="number" step="0.01" class="item-num-input"
                       data-field="unit_price" data-idx="${idx}"
                       value="${it.unit_price != null ? esc(String(it.unit_price)) : ''}" placeholder="—"></td>
            <td class="num"><input type="number" step="0.01" class="item-num-input item-total-input"
                       data-field="line_total" data-idx="${idx}"
                       value="${it.line_total != null ? esc(String(it.line_total)) : ''}" placeholder="0.00"></td>
            <td><span class="cat-tag cat-${esc(it.category || 'other')}">${esc(t('view.expenses.cat.' + (it.category || 'other')))}</span></td>
            <td class="item-bucket-cell">
                ${renderBucketSelect(idx, it.tax_bucket || 'unclassified')}
                <span class="item-rental-slot" data-idx="${idx}"></span>
            </td>
        </tr>`;
    const itemRows = Array.isArray(ex.items) && ex.items.length
        ? ex.items.map(editableRow).join('')
        : '';

    modal.innerHTML = `
    <div class="modal-inner wide">
        <h2 data-i18n="view.expenses.h2.receipt_transaction">Receipt → transaction</h2>
        <div class="receipt-summary">
            <div class="receipt-meta-row">
                <label><strong>${esc(t('view.expenses.receipt.label.merchant'))}:</strong>
                    <input type="text" id="rm-merchant" value="${esc(meta.ocr_merchant || '')}"
                           placeholder="${esc(t('view.expenses.receipt.placeholder.merchant'))}"></label>
            </div>
            ${ex.address ? `<div><strong>${esc(t('view.expenses.receipt.label.address'))}:</strong> ${esc(ex.address)}</div>` : ''}
            <div class="receipt-meta-row">
                <label><strong>${esc(t('view.expenses.receipt.label.date'))}:</strong>
                    <input type="date" id="rm-date" value="${esc(meta.ocr_date || '')}"></label>
                ${ex.time ? ` <span class="muted">${esc(ex.time)}</span>` : ''}
                ${!meta.ocr_date ? `<span class="muted small">${esc(t('view.expenses.receipt.date_unread'))}</span>` : ''}
            </div>
            <div class="receipt-totals">
                ${ex.subtotal != null ? `<span><strong>${esc(t('view.expenses.receipt.label.subtotal'))}:</strong> ${fmtMoney(ex.subtotal)}</span>` : ''}
                ${ex.tax != null ? `<span><strong>${esc(t('view.expenses.receipt.label.tax'))}:</strong> ${fmtMoney(ex.tax)}</span>` : ''}
                <span><strong>${esc(t('view.expenses.receipt.label.total'))}:</strong>
                    <input type="number" id="rm-total" step="0.01" value="${meta.ocr_total ?? ''}" class="rm-total-input"></span>
                <span class="muted"><strong>${esc(t('view.expenses.receipt.label.conf'))}:</strong> ${meta.ocr_confidence != null ? (meta.ocr_confidence * 100).toFixed(0) + '%' : '?'}</span>
                ${(() => {
                    if (!ex.engine) return '';
                    // Ensemble results carry a composite engine string like
                    // "ensemble:apple_vision+tesseract_psm4+tesseract_psm6".
                    // Render as a generic "Ensemble (N)" pill with the
                    // backend list as a tooltip so the UI stays compact
                    // while remaining diagnosable.
                    if (ex.engine.startsWith('ensemble:')) {
                        const backends = ex.engine.slice('ensemble:'.length).split('+');
                        const label = t('view.expenses.receipt.engine.ensemble', { n: backends.length });
                        return `<span class="ocr-engine-pill ocr-engine-ensemble" title="${esc(backends.join(', '))}"><strong>${esc(t('view.expenses.receipt.label.engine'))}:</strong> ${esc(label)}</span>`;
                    }
                    return `<span class="ocr-engine-pill ocr-engine-${esc(ex.engine)}"><strong>${esc(t('view.expenses.receipt.label.engine'))}:</strong> ${esc(t('view.expenses.receipt.engine.' + ex.engine))}</span>`;
                })()}
            </div>
        </div>
        <h3 data-i18n="view.expenses.h3.items">Items</h3>
        ${itemRows ? `
        <div class="bulk-bar">
            <span class="muted small">${esc(t('view.expenses.bulk.apply_all'))}</span>
            <select id="bulk-bucket">
                <option value="">${esc(t('view.expenses.bulk.no_change'))}</option>
                ${BUCKETS.map(b => `<option value="${b}">${esc(t('view.expenses.bucket.' + b))}</option>`).join('')}
            </select>
            <span id="bulk-rental-slot"></span>
            <button type="button" id="bulk-apply" class="btn btn-secondary btn-compact" disabled>${esc(t('view.expenses.bulk.apply'))}</button>
            <span class="bulk-spacer"></span>
            <button type="button" id="m-additem" class="btn btn-secondary btn-compact">+ ${esc(t('view.expenses.items.add'))}</button>
            <button type="button" id="m-reocr" class="btn btn-secondary btn-compact" title="${esc(t('view.expenses.receipt.rerun_ocr'))}">⟳ OCR</button>
        </div>
        <table class="trades receipt-items">
            <thead><tr>
                <th>${esc(t('view.expenses.th.item'))}</th>
                <th class="num">${esc(t('view.expenses.th.qty'))}</th>
                <th class="num">${esc(t('view.expenses.th.unit'))}</th>
                <th class="num">${esc(t('view.expenses.th.line_total'))}</th>
                <th>${esc(t('view.expenses.th.category'))}</th>
                <th>${esc(t('view.expenses.th.tax_bucket'))}</th>
            </tr></thead>
            <tbody>${itemRows}</tbody>
        </table>` : `
        <div class="muted small receipt-items-empty">
            ${esc(t('view.expenses.receipt.no_items'))}
            <button type="button" id="m-additem" class="btn btn-secondary btn-compact" style="margin-left:8px">+ ${esc(t('view.expenses.items.add'))}</button>
            <button type="button" id="m-reocr" class="btn btn-secondary btn-compact" style="margin-left:4px">${esc(t('view.expenses.receipt.rerun_ocr'))}</button>
        </div>`}
        <h3 data-i18n="view.expenses.h3.candidate_transactions">Candidate transactions</h3>
        <table class="trades">
            <thead><tr><th data-i18n="view.expenses.th.score">Score</th><th data-i18n="view.expenses.th.date_2">Date</th><th data-i18n="view.expenses.th.account_2">Account</th><th data-i18n="view.expenses.th.merchant_2">Merchant</th><th data-i18n="view.expenses.th.amount_2">Amount</th><th></th></tr></thead>
            <tbody>${rows || `<tr><td colspan="6" class="boot">${esc(t('view.expenses.empty.no_candidates'))}</td></tr>`}</tbody>
        </table>
        <details class="ocr-raw">
            <summary>${esc(t('view.expenses.receipt.raw_text'))}</summary>
            <pre>${esc(meta.ocr_text || '(empty)')}</pre>
        </details>
        <div style="margin-top:12px;display:flex;gap:8px;align-items:center">
            <a href="${api.receiptBlobUrl(meta.id)}" target="_blank">${esc(t('common.link.view_receipt'))}</a>
            <button type="button" id="m-redownload" class="btn btn-secondary btn-compact" title="${esc(t('view.expenses.receipt.redownload_tip'))}">${esc(t('view.expenses.receipt.redownload_models'))}</button>
            <button data-i18n="view.expenses.btn.close_4" id="m-close" style="margin-left:auto">Close</button>
        </div>
    </div>`;
    modal.querySelector('#m-close').onclick = () => modal.classList.add('hidden');

    // Editable receipt-meta fields — merchant / date / total. On change,
    // PATCH the row so the user's correction sticks without a re-upload.
    // Date input is the most important: OCR routinely misses 1 digit
    // (`05/25/26` came back as `0/25/26`) and the parser refuses to
    // commit a date when every candidate had a reject tag.
    const wireMetaField = (id, transform = (v) => v) => {
        const el = modal.querySelector(`#${id}`);
        if (!el) return;
        el.addEventListener('change', async () => {
            const val = transform(el.value);
            const key = id === 'rm-merchant' ? 'merchant'
                       : id === 'rm-date' ? 'date'
                       : 'total';
            try {
                const updated = await api.patchReceiptMeta(meta.id, { [key]: val });
                if (key === 'merchant') meta.ocr_merchant = updated.ocr_merchant;
                if (key === 'date') meta.ocr_date = updated.ocr_date;
                if (key === 'total') meta.ocr_total = updated.ocr_total;
                showToast(t('view.expenses.receipt.meta_saved'), { level: 'success' });
            } catch (e) {
                showToast(t('view.expenses.bucket.save_err', { err: e.message || String(e) }),
                    { level: 'error' });
            }
        });
    };
    wireMetaField('rm-merchant', (v) => v.trim() || null);
    wireMetaField('rm-date', (v) => v || null);
    wireMetaField('rm-total', (v) => v === '' ? null : Number(v));

    // Re-run OCR on this receipt — used when items are missing because
    // the row was extracted on an older parser / OCR engine. Existing
    // hand-corrected meta + bucket choices live in different columns
    // so they survive the re-OCR.
    const reocrBtn = modal.querySelector('#m-reocr');
    if (reocrBtn) {
        reocrBtn.addEventListener('click', async () => {
            reocrBtn.disabled = true;
            const orig = reocrBtn.textContent;
            reocrBtn.textContent = t('view.expenses.receipt.rerunning_ocr');
            try {
                await api.retryReceiptOcr(meta.id);
                modal.classList.add('hidden');
                pollReceiptUntilReady(meta.id);
            } catch (e) {
                showToast(t('view.expenses.bucket.save_err', { err: e.message }),
                    { level: 'error' });
                reocrBtn.disabled = false;
                reocrBtn.textContent = orig;
            }
        });
    }

    const redlBtn = modal.querySelector('#m-redownload');
    if (redlBtn) {
        redlBtn.addEventListener('click', async () => {
            redlBtn.disabled = true;
            const orig = redlBtn.textContent;
            redlBtn.textContent = t('view.expenses.receipt.downloading_models');
            try {
                const r = await api.ocrModelsDownload({ force: true });
                showToast(t('view.expenses.receipt.models_downloaded', {
                    files: (r.downloaded || []).length,
                    mb: ((r.bytes_total || 0) / 1024 / 1024).toFixed(1),
                }), { level: 'success' });
                await api.retryReceiptOcr(meta.id);
                modal.classList.add('hidden');
                pollReceiptUntilReady(meta.id);
            } catch (e) {
                showToast(t('view.expenses.receipt.models_download_err', {
                    err: e.message || String(e),
                }), { level: 'error' });
                redlBtn.disabled = false;
                redlBtn.textContent = orig;
            }
        });
    }

    // Mount a rental-property picker next to each row whose bucket is
    // already `rental` (server-stored from a prior session). For other
    // buckets, the slot stays empty until the user picks `rental`.
    const mountRentalPicker = async (idx, currentPropId) => {
        const slot = modal.querySelector(`.item-rental-slot[data-idx="${idx}"]`);
        if (!slot) return;
        const props = await getRentalProps();
        if (!props.length) {
            slot.innerHTML = `<span class="muted small">${esc(t('view.expenses.bucket.no_properties'))}</span>`;
            return;
        }
        slot.innerHTML = `
            <select class="item-rental-prop" data-idx="${idx}">
                <option value="">${esc(t('view.expenses.bucket.pick_property'))}</option>
                ${props.map(p => `<option value="${esc(p.id)}"${p.id === currentPropId ? ' selected' : ''}>${esc(p.nickname || p.id)}</option>`).join('')}
            </select>`;
        slot.querySelector('select').addEventListener('change', async (e) => {
            const propId = e.target.value || null;
            try {
                await api.patchReceiptItem(meta.id, idx, { rental_property_id: propId });
                showToast(t('view.expenses.bucket.property_saved'), { level: 'success' });
            } catch (err) {
                showToast(t('view.expenses.bucket.save_err', { err: err.message }), { level: 'error' });
            }
        });
    };
    if (Array.isArray(ex.items)) {
        ex.items.forEach((it, idx) => {
            if ((it.tax_bucket || 'unclassified') === 'rental') {
                void mountRentalPicker(idx, it.rental_property_id || null);
            }
        });
    }
    // Bucket dropdown change → PATCH the server, swap rental picker
    // visibility, optimistic UI.
    modal.querySelectorAll('.item-bucket').forEach(sel => {
        sel.addEventListener('change', async (e) => {
            const idx = Number(e.target.dataset.idx);
            const bucket = e.target.value;
            const slot = modal.querySelector(`.item-rental-slot[data-idx="${idx}"]`);
            if (slot) slot.innerHTML = '';
            try {
                await api.patchReceiptItem(meta.id, idx, {
                    tax_bucket: bucket,
                    // Clear the property pointer whenever the bucket
                    // leaves `rental` so the rollup doesn't keep a stale
                    // link.
                    rental_property_id: bucket === 'rental' ? undefined : null,
                });
                if (bucket === 'rental') mountRentalPicker(idx, null);
            } catch (err) {
                showToast(t('view.expenses.bucket.save_err', { err: err.message }), { level: 'error' });
            }
        });
    });

    // Editable name + qty/unit/line_total inputs. PATCH on change so
    // every OCR-mistake correction lands without a separate save step.
    modal.querySelectorAll('.item-name-input').forEach(inp => {
        inp.addEventListener('change', async () => {
            const idx = Number(inp.dataset.idx);
            try {
                await api.patchReceiptItem(meta.id, idx, { name: inp.value.trim() });
                if (ex.items[idx]) ex.items[idx].name = inp.value.trim();
                showToast(t('view.expenses.items.saved'), { level: 'success' });
            } catch (err) {
                showToast(t('view.expenses.bucket.save_err', { err: err.message }),
                    { level: 'error' });
            }
        });
    });
    modal.querySelectorAll('.item-num-input').forEach(inp => {
        inp.addEventListener('change', async () => {
            const idx = Number(inp.dataset.idx);
            const field = inp.dataset.field;
            // Empty string clears qty/unit; line_total stays required.
            const v = inp.value.trim();
            let patchVal;
            if (v === '') {
                if (field === 'line_total') {
                    showToast(t('view.expenses.items.total_required'), { level: 'error' });
                    inp.value = ex.items[idx][field] != null ? String(ex.items[idx][field]) : '';
                    return;
                }
                patchVal = null;
            } else {
                const n = Number(v);
                if (!Number.isFinite(n)) {
                    showToast(t('view.expenses.items.invalid_number'), { level: 'error' });
                    return;
                }
                patchVal = n;
            }
            try {
                await api.patchReceiptItem(meta.id, idx, { [field]: patchVal });
                if (ex.items[idx]) ex.items[idx][field] = patchVal;

                // Auto-recompute line_total = qty × unit_price whenever
                // either factor changes AND both are now set. The user
                // can still manually override the line_total afterward
                // for cases where the receipt's printed total differs
                // (rounding, discount, weighted item, etc.).
                if (field === 'qty' || field === 'unit_price') {
                    const it = ex.items[idx];
                    if (it && it.qty != null && it.unit_price != null) {
                        const q = Number(it.qty);
                        const u = Number(it.unit_price);
                        if (Number.isFinite(q) && Number.isFinite(u)) {
                            // Two-decimal rounding in cents to dodge
                            // float-multiply drift (`0.1 * 3` = 0.30000000000000004).
                            const newTotal = Math.round(q * u * 100) / 100;
                            const totalInp = modal.querySelector(
                                `.item-total-input[data-idx="${idx}"]`,
                            );
                            if (totalInp) totalInp.value = newTotal.toFixed(2);
                            await api.patchReceiptItem(meta.id, idx, {
                                line_total: newTotal,
                            });
                            it.line_total = newTotal;
                        }
                    }
                }
            } catch (err) {
                showToast(t('view.expenses.bucket.save_err', { err: err.message }),
                    { level: 'error' });
            }
        });
    });
    // Delete row → DELETE → remove the row from the DOM optimistically,
    // re-render the modal so subsequent indices stay aligned with the
    // server's array.
    modal.querySelectorAll('.item-del-btn').forEach(btn => {
        btn.addEventListener('click', async () => {
            const idx = Number(btn.dataset.delIdx);
            const itemName = ex.items[idx]?.name || `#${idx + 1}`;
            const ok = await tConfirm('view.expenses.items.confirm_delete', { item: itemName });
            if (!ok) return;
            try {
                const extracted = await api.deleteReceiptItem(meta.id, idx);
                // Refresh `meta.ocr_extracted` from the server response so
                // the re-render picks up the correct index space.
                meta.ocr_extracted = extracted;
                openReceiptMatchModal(meta);
                showToast(t('view.expenses.items.deleted'), { level: 'success' });
            } catch (err) {
                showToast(t('view.expenses.bucket.save_err', { err: err.message }),
                    { level: 'error' });
            }
        });
    });
    // Add item → POST a minimal blank, then re-render so the new row
    // gets editable inputs the user can fill in.
    const addBtn = modal.querySelector('#m-additem');
    if (addBtn) {
        addBtn.addEventListener('click', async () => {
            try {
                const extracted = await api.addReceiptItem(meta.id, {
                    name: t('view.expenses.items.new_default_name'),
                    line_total: 0,
                });
                meta.ocr_extracted = extracted;
                openReceiptMatchModal(meta);
            } catch (err) {
                showToast(t('view.expenses.bucket.save_err', { err: err.message }),
                    { level: 'error' });
            }
        });
    }

    // Click a category tag → swap it for a <select> so the user can
    // re-pick when the parser miscategorized. On change, PATCH the
    // category and restore the tag visual.
    const CATEGORIES = [
        'advertising','vehicle_fuel','vehicle_maintenance','travel_transport',
        'travel_lodging','meals','office_supplies','office_equipment_software',
        'supplies_cogs','repairs_maintenance','utilities','rent_lease',
        'insurance','professional_services','contract_labor','wages_benefits',
        'bank_fees','taxes_licenses_dues','education_training','groceries','other',
    ];
    modal.querySelectorAll('.cat-tag').forEach((tag) => {
        const row = tag.closest('tr[data-item-idx]');
        if (!row) return;
        const idx = Number(row.dataset.itemIdx);
        const current = (ex.items[idx] && ex.items[idx].category) || 'other';
        tag.style.cursor = 'pointer';
        tag.title = t('view.expenses.cat.click_to_change');
        tag.addEventListener('click', () => {
            const sel = document.createElement('select');
            sel.className = 'item-cat-select';
            sel.innerHTML = CATEGORIES.map(c =>
                `<option value="${c}"${c === current ? ' selected' : ''}>${esc(t('view.expenses.cat.' + c))}</option>`
            ).join('');
            tag.replaceWith(sel);
            sel.focus();
            const restoreTag = (categoryId) => {
                const newTag = document.createElement('span');
                newTag.className = `cat-tag cat-${categoryId}`;
                newTag.textContent = t('view.expenses.cat.' + categoryId);
                newTag.style.cursor = 'pointer';
                newTag.title = t('view.expenses.cat.click_to_change');
                sel.replaceWith(newTag);
                // Re-bind the click handler to the new tag so the user
                // can re-pick again without re-rendering the modal.
                newTag.addEventListener('click', () => {
                    sel.dispatchEvent(new Event('rebind'));
                });
                ex.items[idx].category = categoryId;
            };
            sel.addEventListener('change', async (e) => {
                const newCat = e.target.value;
                try {
                    await api.patchReceiptItem(meta.id, idx, { category: newCat });
                    restoreTag(newCat);
                } catch (err) {
                    showToast(t('view.expenses.bucket.save_err', { err: err.message }), { level: 'error' });
                    restoreTag(current);
                }
            });
            sel.addEventListener('blur', () => {
                if (sel.parentElement) restoreTag(current);
            });
        });
    });

    // Bulk-apply: pick a bucket (and optionally a property for rental)
    // → PATCH every item on the receipt in one click. Useful for
    // property-improvement receipts where the whole basket is rental.
    const bulkBucketSel = modal.querySelector('#bulk-bucket');
    const bulkRentalSlot = modal.querySelector('#bulk-rental-slot');
    const bulkApplyBtn = modal.querySelector('#bulk-apply');
    let bulkPropId = null;
    const refreshBulkApplyState = () => {
        const b = bulkBucketSel ? bulkBucketSel.value : '';
        if (!bulkApplyBtn) return;
        // Block apply when bucket=rental but no property selected — the
        // common UX trap is forgetting to pick the property before
        // hitting apply.
        bulkApplyBtn.disabled = !b || (b === 'rental' && !bulkPropId);
    };
    if (bulkBucketSel) {
        bulkBucketSel.addEventListener('change', async () => {
            const b = bulkBucketSel.value;
            bulkPropId = null;
            if (bulkRentalSlot) bulkRentalSlot.innerHTML = '';
            if (b === 'rental') {
                const props = await getRentalProps();
                if (!props.length) {
                    bulkRentalSlot.innerHTML = `<span class="muted small">${esc(t('view.expenses.bucket.no_properties'))}</span>`;
                } else {
                    bulkRentalSlot.innerHTML = `
                        <select id="bulk-rental-prop">
                            <option value="">${esc(t('view.expenses.bucket.pick_property'))}</option>
                            ${props.map(p => `<option value="${esc(p.id)}">${esc(p.nickname || p.id)}</option>`).join('')}
                        </select>`;
                    bulkRentalSlot.querySelector('select').addEventListener('change', (e) => {
                        bulkPropId = e.target.value || null;
                        refreshBulkApplyState();
                    });
                }
            }
            refreshBulkApplyState();
        });
    }
    if (bulkApplyBtn) {
        bulkApplyBtn.addEventListener('click', async () => {
            const b = bulkBucketSel.value;
            const items = Array.isArray(ex.items) ? ex.items : [];
            bulkApplyBtn.disabled = true;
            bulkApplyBtn.textContent = t('view.expenses.bulk.applying');
            try {
                await Promise.all(items.map((_, idx) =>
                    api.patchReceiptItem(meta.id, idx, {
                        tax_bucket: b,
                        rental_property_id: b === 'rental' ? bulkPropId : null,
                    })
                ));
                showToast(t('view.expenses.bulk.applied', { n: items.length }), { level: 'success' });
                // Re-render the modal to reflect the new state without
                // forcing a full receipt re-fetch (meta is already
                // current — we mutate in place).
                items.forEach(it => {
                    it.tax_bucket = b;
                    it.rental_property_id = b === 'rental' ? bulkPropId : null;
                });
                openReceiptMatchModal(meta);
            } catch (err) {
                showToast(t('view.expenses.bucket.save_err', { err: err.message }), { level: 'error' });
                bulkApplyBtn.disabled = false;
                bulkApplyBtn.textContent = t('view.expenses.bulk.apply');
            }
        });
    }

    modal.querySelectorAll('.m-pick').forEach(btn => {
        btn.onclick = async () => {
            try {
                await api.attachReceipt(meta.id, btn.dataset.tx);
                if (!viewIsCurrent(state.tok)) return;
                modal.classList.add('hidden');
                const s = state.mount.querySelector('#exp-status');
                if (s) s.textContent = t('view.expenses.status.receipt_attached');
                await refresh();
            } catch (e) { showToast(t('view.expenses.alert.attach_failed', { err: e.message }), { level: 'error' }); }
        };
    });
}

// Tax rollup — pulls the server's per-bucket / per-category totals
// across every receipt's items in the active date window, defaults to
// year-to-date. Three top-level cards (Business / Rental / Personal +
// Unclassified) so the user can see at a glance how the uploaded
// receipts split across Schedule C vs Schedule E vs out-of-scope.
async function openTaxRollupModal(yearOverride) {
    const modal = state.mount.querySelector('#exp-rules-modal');
    if (!modal) return;
    modal.classList.remove('hidden');
    modal.innerHTML = '<div class="modal-inner"><p class="boot">' + esc(t('view.expenses.hint.building_rollup')) + '</p></div>';
    const year = Number(yearOverride) || (state.taxPeriod && state.taxPeriod.year)
        || new Date().getFullYear();
    const today = new Date();
    const isCurrent = year === today.getFullYear();
    const from = `${year}-01-01`;
    const to = isCurrent ? today.toISOString().slice(0, 10) : `${year}-12-31`;
    let r;
    try { r = await api.taxRollup({ from, to }); }
    catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        modal.innerHTML = `<div class="modal-inner"><p class="boot">${esc(t('view.expenses.boot.rollup_failed', { err: e.message }))}</p>
            <button id="tr-close">Close</button></div>`;
        modal.querySelector('#tr-close').onclick = () => modal.classList.add('hidden');
        return;
    }
    if (!viewIsCurrent(state.tok)) return;
    const money = (v) => {
        const n = Number(v);
        return Number.isFinite(n) ? `$${n.toFixed(2)}` : '—';
    };
    const catRows = (cats, scheduleKey) => cats.length
        ? cats.map(c => {
            const line = c[scheduleKey];
            const lineCell = line
                ? `<span class="sched-line">${esc((scheduleKey === 'schedule_c_line' ? 'C' : 'E') + line)}</span>`
                : '';
            return `<tr>
                <td>${lineCell}</td>
                <td><span class="cat-tag cat-${esc(c.category)}">${esc(t('view.expenses.cat.' + c.category))}</span></td>
                <td class="num">${money(c.total)}</td>
            </tr>`;
        }).join('')
        : `<tr><td colspan="3" class="muted small">${esc(t('view.expenses.bucket.empty'))}</td></tr>`;
    const bucketCard = (titleKey, data, scheduleKey) => `
        <div class="rollup-card">
            <h3>${esc(t(titleKey))}</h3>
            <div class="rollup-total">${money(data.grand_total || 0)}</div>
            <table class="trades"><tbody>${catRows(data.categories || [], scheduleKey)}</tbody></table>
        </div>`;
    const rentalCard = `
        <div class="rollup-card">
            <h3>${esc(t('view.expenses.bucket.rental'))}</h3>
            <div class="rollup-total">${money(r.rental.grand_total)}</div>
            ${r.rental.properties.length
                ? r.rental.properties.map(p => `
                    <div class="rollup-property">
                        <div class="rollup-property-name">${esc(p.property_name || t('view.expenses.bucket.no_property'))} — ${money(p.grand_total)}</div>
                        <table class="trades"><tbody>${catRows(p.categories, 'schedule_e_line')}</tbody></table>
                    </div>`).join('')
                : `<div class="muted small">${esc(t('view.expenses.bucket.empty'))}</div>`}
        </div>`;

    const currentYear = new Date().getFullYear();
    const yearOpts = Array.from({ length: 7 }, (_, i) => currentYear - i)
        .map(y => `<option value="${y}"${y === year ? ' selected' : ''}>${y}</option>`).join('');
    modal.innerHTML = `
    <div class="modal-inner wide">
        <div style="display:flex;align-items:center;gap:12px;margin-bottom:8px">
            <h2 style="margin:0">${esc(t('view.expenses.h2.tax_rollup', { from: r.from, to: r.to }))}</h2>
            <label class="muted small">${esc(t('view.purchases.filter.year'))}:
                <select id="tr-year">${yearOpts}</select></label>
        </div>
        <p class="muted small">${esc(t('view.expenses.hint.rollup_intro', { receipts: r.receipts_counted, items: r.items_counted }))}</p>
        <div class="rollup-grid">
            ${bucketCard('view.expenses.bucket.business', r.business, 'schedule_c_line')}
            ${rentalCard}
            ${bucketCard('view.expenses.bucket.personal', r.personal, null)}
            ${bucketCard('view.expenses.bucket.unclassified', r.unclassified, null)}
        </div>
        <div style="margin-top:12px;display:flex;gap:8px;align-items:center">
            <a href="${api.taxRollupCsvUrl({ from, to })}" download="tax-rollup-${from}-to-${to}.csv" class="btn btn-secondary btn-compact">${esc(t('view.expenses.btn.download_csv'))}</a>
            <button id="tr-close" style="margin-left:auto">${esc(t('view.expenses.btn.close_4'))}</button>
        </div>
    </div>`;
    modal.querySelector('#tr-close').onclick = () => modal.classList.add('hidden');
    modal.querySelector('#tr-year').addEventListener('change', (e) => {
        openTaxRollupModal(Number(e.target.value));
    });
}

async function openScheduleCModal(year) {
    const modal = state.mount.querySelector('#exp-rules-modal');
    if (!modal) return;
    modal.classList.remove('hidden');
    const initialYear = year || new Date().getFullYear();
    modal.innerHTML = '<div class="modal-inner"><p data-i18n="view.expenses.hint.building_report" class="boot">building report…</p></div>';
    let report;
    try { report = await api.scheduleC(initialYear); }
    catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        modal.innerHTML = `<div class="modal-inner"><p class="boot">${esc(t('view.expenses.boot.report_failed', { err: e.message }))}</p>
            <button data-i18n="view.expenses.btn.close_5" id="sc-close">Close</button></div>`;
        modal.querySelector('#sc-close').onclick = () => modal.classList.add('hidden');
        return;
    }
    if (!viewIsCurrent(state.tok)) return;

    const fmt = n => Number(n).toLocaleString(undefined, {
        minimumFractionDigits: 2, maximumFractionDigits: 2,
    });

    const lines = report.lines.map(l => {
        const ded = Number(l.deduction_pct);
        const pctLabel = ded === 1 ? '100%' : `${(ded * 100).toFixed(0)}%`;
        return `<tr>
            <td>${l.schedule_c_line}</td>
            <td>${esc(l.label)}</td>
            <td class="num">${fmt(l.raw_total)}</td>
            <td>${pctLabel}</td>
            <td class="num"><strong>${fmt(l.deductible_total)}</strong></td>
            <td class="num">${l.txn_count}</td>
        </tr>`;
    }).join('');

    // Year picker: current year ± 4.
    const now = new Date().getFullYear();
    const years = [];
    for (let y = now - 4; y <= now + 1; y++) years.push(y);
    const yearOpts = years.map(y =>
        `<option value="${y}"${y === report.year ? ' selected' : ''}>${y}</option>`).join('');

    modal.innerHTML = `
    <div class="modal-inner wide">
        <h2>${esc(t('view.expenses.h2.schedule_c', { year: report.year }))}</h2>
        <div style="display:flex;gap:12px;margin-bottom:12px;align-items:center">
            <label><span data-i18n="view.expenses.label.year">Year</span>
                <select id="sc-year">${yearOpts}</select></label>
            <span style="color:var(--text-dim);font-size:11px">${esc(t('view.expenses.report.meta', { from: report.from_date, to: report.to_date, transfers: report.excluded_transfers, personal: report.excluded_personal }))}</span>
        </div>
        <table class="trades sc-table">
            <thead><tr>
                <th data-i18n="view.expenses.th.line">Line</th><th data-i18n="view.expenses.th.category">Category</th>
                <th data-i18n="view.expenses.th.raw" class="num">Raw $</th><th data-i18n="view.expenses.th.ded">Ded %</th>
                <th data-i18n="view.expenses.th.deductible" class="num">Deductible $</th><th class="num">#</th>
            </tr></thead>
            <tbody>${lines}</tbody>
            <tfoot>
                <tr>
                    <td colspan="2"><strong data-i18n="view.expenses.summary.grand_total">Grand total (categorized business)</strong></td>
                    <td class="num">${fmt(report.grand_total_raw)}</td>
                    <td></td>
                    <td class="num"><strong>${fmt(report.grand_total_deductible)}</strong></td>
                    <td></td>
                </tr>
                <tr>
                    <td colspan="2" style="color:var(--yellow)" data-i18n="view.expenses.summary.uncategorized">⚠ Uncategorized business expenses</td>
                    <td class="num" style="color:var(--yellow)">${fmt(report.uncategorized_total)}</td>
                    <td></td>
                    <td></td>
                    <td class="num">${report.uncategorized_count}</td>
                </tr>
            </tfoot>
        </table>
        <p style="color:var(--text-dim);font-size:11px;margin-top:12px"
           data-i18n-html="view.expenses.summary.footnote">
            Deductible column applies each category's IRS rate (meals 50%; everything else 100%).
            Uncategorized business expenses do <strong>not</strong> roll into the grand total —
            tag them in the transaction list first.
        </p>
        <button data-i18n="view.expenses.btn.close_6" id="sc-close" style="margin-top:8px">Close</button>
    </div>`;
    modal.querySelector('#sc-close').onclick = () => modal.classList.add('hidden');
    modal.querySelector('#sc-year').addEventListener('change', e => {
        openScheduleCModal(Number(e.target.value));
    });
}

async function openReceiptsModal() {
    const modal = state.mount.querySelector('#exp-rules-modal');
    if (!modal) return;
    modal.classList.remove('hidden');
    modal.innerHTML = '<div class="modal-inner"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.status.loading">loading…</div></div></div>';
    let rs = [];
    try {
        const resp = await api.receipts({ limit: 200 });
        // Response shape changed from bare array → { rows, total, offset, limit }.
        // Old modal capped at 200 anyway; for proper browsing use the
        // new #receipts page which has filters + pagination + bulk
        // actions.
        rs = Array.isArray(resp) ? resp : (resp.rows || []);
    }
    catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        modal.innerHTML = `<div class="modal-inner"><p class="boot">${esc(t('view.expenses.boot.load_failed', { err: e.message }))}</p>
            <button data-i18n="view.expenses.btn.close_7" id="r-close">Close</button></div>`;
        modal.querySelector('#r-close').onclick = () => modal.classList.add('hidden');
        return;
    }
    if (!viewIsCurrent(state.tok)) return;

    const rows = rs.map(r => `
        <tr>
            <td>${r.created_at.slice(0, 10)}</td>
            <td>${esc(r.filename)}</td>
            <td>${r.ocr_status}</td>
            <td>${esc(r.ocr_merchant || '')}</td>
            <td>${r.ocr_total ?? ''}</td>
            <td>${r.ocr_date ?? ''}</td>
            <td>${r.transaction_id ? r.transaction_id.slice(0, 8) : ''}</td>
            <td>
                <a href="${api.receiptBlobUrl(r.id)}" target="_blank">${esc(t('common.link.view'))}</a>
                ${!r.transaction_id && r.ocr_status === 'done' ? ` · <button data-i18n="view.expenses.btn.match" class="r-match" data-id="${r.id}">match</button>` : ''}
            </td>
        </tr>`).join('');

    modal.innerHTML = `
    <div class="modal-inner wide">
        <h2 data-i18n="view.expenses.h2.receipts">Receipts</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.expenses.th.uploaded">Uploaded</th><th data-i18n="view.expenses.th.filename">Filename</th><th data-i18n="view.expenses.th.ocr">OCR</th>
                <th data-i18n="view.expenses.th.merchant_3">Merchant</th><th data-i18n="view.expenses.th.total">Total</th><th data-i18n="view.expenses.th.date_3">Date</th><th data-i18n="view.expenses.th.tx">Tx</th><th></th>
            </tr></thead>
            <tbody>${rows || `<tr><td colspan="8" class="boot">${esc(t('view.expenses.empty.no_receipts'))}</td></tr>`}</tbody>
        </table>
        <button data-i18n="view.expenses.btn.close_8" id="r-close" style="margin-top:12px">Close</button>
    </div>`;
    modal.querySelector('#r-close').onclick = () => modal.classList.add('hidden');
    modal.querySelectorAll('.r-match').forEach(btn => {
        btn.onclick = async () => {
            try {
                const meta = await api.receiptMeta(btn.dataset.id);
                if (!viewIsCurrent(state.tok)) return;
                openReceiptMatchModal(meta);
            } catch (e) { showToast(t('view.expenses.alert.open_failed', { err: e.message }), { level: 'error' }); }
        };
    });
}
