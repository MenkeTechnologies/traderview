// Side-by-side business comparison view.
//
// Renders one column per business (plus an aggregated column), each
// showing the same KPI strip (total, deductible, biggest, burn rate,
// receipt count, uncategorized) so two or more businesses can be
// scanned against each other without switching tabs.
//
// Backed by parallel calls to /api/receipts/dashboard-bundle, one per
// business + one aggregated.

import { api } from '../api.js';
import { fmtUsd } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { listBusinesses } from '../business_context.js';

const YEAR_KEY = 'biz_compare_year';

function esc(s) {
    return String(s).replace(/[&<>"]/g, (c) => ({
        '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;',
    }[c]));
}

function getYear() {
    const v = Number(localStorage.getItem(YEAR_KEY));
    return Number.isFinite(v) && v >= 2000 ? v : new Date().getFullYear();
}
function setYear(y) {
    if (Number.isFinite(y)) localStorage.setItem(YEAR_KEY, String(y));
}

export async function renderBusinessCompare(mount) {
    const tok = currentViewToken();
    let businesses = [];
    try { businesses = await listBusinesses(); }
    catch { businesses = []; }
    if (!viewIsCurrent(tok)) return;

    const year = getYear();
    const currentYear = new Date().getFullYear();
    const yearOpts = Array.from({ length: 6 }, (_, i) => currentYear - i)
        .map(y => `<option value="${y}"${y === year ? ' selected' : ''}>${y}</option>`).join('');

    mount.innerHTML = `
        <div class="bizc-header">
            <h1 class="view-title">
                <span data-i18n="view.biz_compare.title">// BUSINESS COMPARISON</span>
            </h1>
            <div class="bizc-controls">
                <label>${esc(t('view.biz_compare.year'))}
                    <select id="bizc-year">${yearOpts}</select></label>
            </div>
        </div>
        <div id="bizc-grid" class="bizc-grid">
            <div class="muted small">${esc(t('common.loading'))}</div>
        </div>
    `;
    mount.querySelector('#bizc-year').addEventListener('change', (e) => {
        setYear(Number(e.target.value));
        renderBusinessCompare(mount);
    });
    if (businesses.length === 0) {
        mount.querySelector('#bizc-grid').innerHTML = `
            <div class="bizc-empty">
                <p>${esc(t('view.biz_compare.empty'))}</p>
                <p class="muted small">${esc(t('view.biz_compare.empty_hint'))}</p>
            </div>`;
        return;
    }

    // Fetch one bundle per business + one aggregated. Parallel.
    let bundles;
    try {
        bundles = await Promise.all([
            api.expenseDashboardBundle(year, null),
            ...businesses.map(b => api.expenseDashboardBundle(year, b.id)),
        ]);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.biz_compare.load_failed', { err: e.message }), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;

    const columns = [
        { label: t('view.biz_compare.aggregated'), key: 'all', bundle: bundles[0] },
        ...businesses.map((b, i) => ({
            label: b.name,
            key: b.id,
            bundle: bundles[i + 1],
        })),
    ];

    const kpis = [
        { key: 'total', label: t('view.exp_dash.kpi.total_spend'), fmt: fmtUsd },
        { key: 'schedule_c', label: t('view.exp_dash.kpi.schedule_c'), fmt: fmtUsd },
        { key: 'schedule_e', label: t('view.exp_dash.kpi.schedule_e'), fmt: fmtUsd },
        { key: 'personal', label: t('view.exp_dash.kpi.personal'), fmt: fmtUsd },
        { key: 'deductible_pct', label: t('view.exp_dash.kpi.deductible_pct'),
            fmt: (v) => `${(+v).toFixed(1)}%` },
        { key: 'avg_ticket', label: t('view.exp_dash.kpi.avg_ticket'), fmt: fmtUsd },
        { key: 'avg_daily', label: t('view.exp_dash.kpi.avg_daily'), fmt: fmtUsd },
        { key: 'burn_rate_monthly', label: t('view.exp_dash.kpi.burn_rate'), fmt: fmtUsd },
        { key: 'biggest_receipt', label: t('view.exp_dash.kpi.biggest_receipt'), fmt: fmtUsd },
        { key: 'receipt_count', label: t('view.exp_dash.kpi.total_receipts'), fmt: (v) => String(v) },
        { key: 'uncategorized_total', label: t('view.exp_dash.kpi.uncategorized'), fmt: fmtUsd },
        { key: 'longest_zero_streak_days', label: t('view.exp_dash.kpi.longest_zero_streak'),
            fmt: (v) => `${v} ${t('view.exp_dash.kpi.days')}` },
    ];

    const headerRow = `<thead><tr>
        <th class="bizc-kpi-label"></th>
        ${columns.map(c => `<th class="bizc-col-head" data-key="${esc(c.key)}">${esc(c.label)}</th>`).join('')}
    </tr></thead>`;

    // Per-KPI rows. We highlight the column that's "best" for each KPI
    // (lowest spend for cost rows, highest % deductible — except for
    // total/burn/biggest where higher is "worse").
    const lowerIsBetter = new Set([
        'total', 'schedule_e', 'personal', 'avg_ticket', 'avg_daily',
        'burn_rate_monthly', 'biggest_receipt', 'uncategorized_total',
    ]);

    const bodyRows = kpis.map(k => {
        const vals = columns.map(c => Number(c.bundle?.kpis?.[k.key] ?? 0));
        // Skip the aggregated column for "best" highlighting since it's
        // always the largest by definition for sum-type KPIs.
        const perBizVals = vals.slice(1);
        let bestIdx = -1;
        if (perBizVals.length > 0) {
            const ext = lowerIsBetter.has(k.key)
                ? Math.min(...perBizVals)
                : Math.max(...perBizVals);
            if (Number.isFinite(ext)) bestIdx = perBizVals.indexOf(ext) + 1;
        }
        return `<tr>
            <th class="bizc-kpi-label">${esc(k.label)}</th>
            ${columns.map((c, i) => {
                const v = vals[i];
                const cls = i === bestIdx ? 'bizc-best' : (i === 0 ? 'bizc-agg' : '');
                return `<td class="${cls}"><strong>${esc(k.fmt(v))}</strong></td>`;
            }).join('')}
        </tr>`;
    }).join('');

    mount.querySelector('#bizc-grid').innerHTML = `
        <table class="bizc-table">
            ${headerRow}
            <tbody>${bodyRows}</tbody>
        </table>
        <p class="muted small bizc-footer">
            ${esc(t('view.biz_compare.footer_legend'))}
        </p>
    `;
}
