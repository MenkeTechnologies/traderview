// Business-expense dashboard — widget-catalog port of the trading
// dashboard architecture (views/dashboard.js). One backend bundle call
// → 30+ widgets render from local data slices. Layout is draggable and
// persisted to localStorage.
//
// Each widget definition:
//   { id, titleKey, spans2?, html(data), mount?(data, mount) }
//
// `html()` returns the inner HTML string (sync). `mount()` runs after
// the panel is in the DOM — used for uPlot init or canvas drawing.

import { api } from '../api.js';
import { fmtUsd } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { initDragReorder, resetDragReorder } from '../drag_reorder.js';
import { showToast } from '../toast.js';
import {
    mountBusinessSelector,
    onChange as onBusinessChange,
    activeBusinessId,
} from '../business_context.js';

const LAYOUT_BASE_KEY = 'expense_dashboard_layout_v1';
function layoutKey() {
    const bid = activeBusinessId();
    return bid ? `${LAYOUT_BASE_KEY}_${bid}` : LAYOUT_BASE_KEY;
}
const YEAR_KEY = 'expense_dashboard_year';

function esc(s) {
    return String(s).replace(/[&<>"]/g, (c) => ({
        '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;',
    }[c]));
}

// Hero stat helper — large number + optional subtitle. Mirrors the
// trade-dashboard `heroStat` pattern so widgets feel native.
function heroStat(value, cls = '', sub = '') {
    return `<div class="hero-stat hero-stat-band">
        <div class="hero-num-md ${cls}">${esc(value)}</div>
        ${sub ? `<div class="muted small">${esc(sub)}</div>` : ''}
    </div>`;
}

// ── Catalog of expense-dashboard widgets ──────────────────────────────────
// Each one renders from one or more slices of the bundle.

const WIDGETS = [
    // KPI hero strip ────────────────────────────────────────────────────
    { id: 'kpi_total_spend', titleKey: 'view.exp_dash.kpi.total_spend',
        html: (d) => heroStat(fmtUsd(+d.kpis.total),
            'hero-num-cyan',
            t('view.exp_dash.kpi.from_n_receipts', { n: d.kpis.receipt_count })) },

    { id: 'kpi_schedule_c', titleKey: 'view.exp_dash.kpi.schedule_c',
        html: (d) => heroStat(fmtUsd(+d.kpis.schedule_c),
            'hero-num-cyan',
            t('view.exp_dash.kpi.business_share')) },

    { id: 'kpi_schedule_e', titleKey: 'view.exp_dash.kpi.schedule_e',
        html: (d) => heroStat(fmtUsd(+d.kpis.schedule_e),
            'hero-num-cyan',
            t('view.exp_dash.kpi.rental_share')) },

    { id: 'kpi_personal', titleKey: 'view.exp_dash.kpi.personal',
        html: (d) => heroStat(fmtUsd(+d.kpis.personal),
            '',
            t('view.exp_dash.kpi.personal_share')) },

    { id: 'kpi_deductible_pct', titleKey: 'view.exp_dash.kpi.deductible_pct',
        html: (d) => {
            const pct = +d.kpis.deductible_pct || 0;
            const cls = pct >= 70 ? 'tw-refund' : pct >= 40 ? '' : 'tw-owed';
            return heroStat(`${pct.toFixed(1)}%`, cls, t('view.exp_dash.kpi.biz_plus_rental'));
        }},

    { id: 'kpi_avg_ticket', titleKey: 'view.exp_dash.kpi.avg_ticket',
        html: (d) => heroStat(fmtUsd(+d.kpis.avg_ticket),
            'hero-num-cyan',
            `${d.kpis.receipt_count} ${t('view.exp_dash.kpi.receipts')}`) },

    { id: 'kpi_avg_daily', titleKey: 'view.exp_dash.kpi.avg_daily',
        html: (d) => heroStat(fmtUsd(+d.kpis.avg_daily),
            'hero-num-cyan',
            t('view.exp_dash.kpi.ytd_pace')) },

    { id: 'kpi_burn_rate', titleKey: 'view.exp_dash.kpi.burn_rate',
        html: (d) => heroStat(fmtUsd(+d.kpis.burn_rate_monthly),
            'hero-num-warn',
            t('view.exp_dash.kpi.month_pace')) },

    { id: 'kpi_biggest_receipt', titleKey: 'view.exp_dash.kpi.biggest_receipt',
        html: (d) => heroStat(fmtUsd(+d.kpis.biggest_receipt), 'hero-num-warn') },

    { id: 'kpi_smallest_receipt', titleKey: 'view.exp_dash.kpi.smallest_receipt',
        html: (d) => heroStat(fmtUsd(+d.kpis.smallest_receipt), '') },

    { id: 'kpi_total_receipts', titleKey: 'view.exp_dash.kpi.total_receipts',
        html: (d) => heroStat(d.kpis.receipt_count, 'hero-num-cyan',
            `${d.kpis.item_count} ${t('view.exp_dash.kpi.items')}`) },

    { id: 'kpi_uncategorized', titleKey: 'view.exp_dash.kpi.uncategorized',
        html: (d) => heroStat(fmtUsd(+d.kpis.uncategorized_total),
            +d.kpis.uncategorized_total > 0 ? 'tw-owed' : 'tw-refund',
            `${d.kpis.uncategorized_count} ${t('view.exp_dash.kpi.items')}`) },

    { id: 'kpi_longest_zero_streak', titleKey: 'view.exp_dash.kpi.longest_zero_streak',
        html: (d) => heroStat(d.kpis.longest_zero_streak_days,
            'hero-num-cyan', t('view.exp_dash.kpi.days')) },

    { id: 'kpi_longest_consec_spending', titleKey: 'view.exp_dash.kpi.longest_consec_spending',
        html: (d) => heroStat(d.kpis.longest_consec_spending_days,
            'hero-num-warn', t('view.exp_dash.kpi.days')) },

    // Time-series chart widgets ────────────────────────────────────────
    { id: 'cumulative_spend', titleKey: 'view.exp_dash.chart.cumulative', spans2: true,
        html: () => `<div id="expd-cumulative" class="chart-h-260"></div>`,
        mount: (d) => mountCumulative(d.daily) },

    { id: 'daily_volume', titleKey: 'view.exp_dash.chart.daily_volume', spans2: true,
        html: () => `<div id="expd-daily-volume" class="chart-h-260"></div>`,
        mount: (d) => mountDailyVolume(d.daily) },

    { id: 'monthly_spend', titleKey: 'view.exp_dash.chart.monthly', spans2: true,
        html: () => `<canvas id="expd-monthly" width="900" height="240"></canvas>`,
        mount: (d) => drawBars('expd-monthly', d.by_month) },

    { id: 'quarterly_spend', titleKey: 'view.exp_dash.chart.quarterly',
        html: () => `<canvas id="expd-quarterly" width="380" height="220"></canvas>`,
        mount: (d) => drawBars('expd-quarterly', d.by_quarter) },

    { id: 'spend_by_dow', titleKey: 'view.exp_dash.chart.dow',
        html: () => `<canvas id="expd-dow" width="380" height="220"></canvas>`,
        mount: (d) => drawBars('expd-dow', orderDow(d.by_dow)) },

    { id: 'spend_by_hour', titleKey: 'view.exp_dash.chart.hour',
        html: () => `<canvas id="expd-hour" width="900" height="220"></canvas>`,
        mount: (d) => drawBars('expd-hour', d.by_hour) },

    { id: 'weekday_vs_weekend', titleKey: 'view.exp_dash.chart.weekday_vs_weekend',
        html: (d) => {
            const wd = d.weekday_vs_weekend?.find(x => x.label === 'weekday') || { total: 0, count: 0 };
            const we = d.weekday_vs_weekend?.find(x => x.label === 'weekend') || { total: 0, count: 0 };
            const wdT = +wd.total, weT = +we.total;
            const total = wdT + weT;
            const wdPct = total > 0 ? (wdT / total * 100).toFixed(1) : '0';
            const wePct = total > 0 ? (weT / total * 100).toFixed(1) : '0';
            return `<div class="expd-split">
                <div class="expd-split-row"><span>${esc(t('view.exp_dash.chart.weekday'))}</span>
                    <strong>${esc(fmtUsd(wdT))} (${wdPct}%)</strong></div>
                <div class="expd-split-row"><span>${esc(t('view.exp_dash.chart.weekend'))}</span>
                    <strong>${esc(fmtUsd(weT))} (${wePct}%)</strong></div>
                <div class="expd-split-bar">
                    <div class="expd-split-bar-wd" data-bar-pct="${wdPct}"></div>
                    <div class="expd-split-bar-we" data-bar-pct="${wePct}"></div>
                </div>
            </div>`;
        }},

    // Distribution widgets ─────────────────────────────────────────────
    { id: 'amount_distribution', titleKey: 'view.exp_dash.chart.amount_dist',
        html: () => `<canvas id="expd-amount" width="380" height="220"></canvas>`,
        mount: (d) => drawBars('expd-amount', d.by_amount_bucket) },

    { id: 'category_pie', titleKey: 'view.exp_dash.chart.category_pie',
        html: () => `<canvas id="expd-cat-pie" width="380" height="220"></canvas>`,
        mount: (d) => drawPie('expd-cat-pie', d.by_category) },

    { id: 'tax_bucket_pie', titleKey: 'view.exp_dash.chart.tax_bucket_pie',
        html: () => `<canvas id="expd-bucket-pie" width="380" height="220"></canvas>`,
        mount: (d) => drawPie('expd-bucket-pie', d.by_tax_bucket) },

    // Leaderboards ─────────────────────────────────────────────────────
    { id: 'top_merchants_by_total', titleKey: 'view.exp_dash.chart.top_merchants_total', spans2: true,
        html: (d) => leaderboardHtml(d.top_merchants_by_total, 'total') },

    { id: 'top_merchants_by_count', titleKey: 'view.exp_dash.chart.top_merchants_count', spans2: true,
        html: (d) => leaderboardHtml(d.top_merchants_by_count, 'count') },

    { id: 'top_categories', titleKey: 'view.exp_dash.chart.top_categories', spans2: true,
        html: (d) => categoryLeaderboardHtml(d.by_category) },

    // Calendar heatmap ─────────────────────────────────────────────────
    { id: 'calendar_heatmap', titleKey: 'view.exp_dash.chart.calendar', spans2: true,
        html: () => `<canvas id="expd-calendar" width="980" height="160"></canvas>
            <div class="tax-calendar-legend">
                <span>${esc(t('view.expenses.tax.chart.calendar_less'))}</span>
                <span class="tax-cal-swatch s0"></span>
                <span class="tax-cal-swatch s1"></span>
                <span class="tax-cal-swatch s2"></span>
                <span class="tax-cal-swatch s3"></span>
                <span class="tax-cal-swatch s4"></span>
                <span>${esc(t('view.expenses.tax.chart.calendar_more'))}</span>
                <span class="tax-calendar-hover" id="expd-cal-hover"></span>
            </div>`,
        mount: (d) => drawCalendar(d.calendar) },

    // ── New gap-fill widgets (audit phase 2) ─────────────────────────
    { id: 'subscriptions', titleKey: 'view.exp_dash.widget.subscriptions', spans2: true,
        html: () => `<div id="expd-subs" class="muted small">${esc(t('common.loading'))}</div>`,
        mount: () => mountSubscriptions() },

    { id: 'pareto_merchants', titleKey: 'view.exp_dash.widget.pareto_merchants', spans2: true,
        html: () => `<canvas id="expd-pareto-merch" width="900" height="260"></canvas>`,
        mount: (d) => drawPareto('expd-pareto-merch', d.top_merchants_by_total) },

    { id: 'pareto_categories', titleKey: 'view.exp_dash.widget.pareto_categories', spans2: true,
        html: () => `<canvas id="expd-pareto-cat" width="900" height="260"></canvas>`,
        mount: (d) => drawPareto('expd-pareto-cat', d.by_category) },

    { id: 'yoy_monthly', titleKey: 'view.exp_dash.widget.yoy_monthly', spans2: true,
        html: () => `<div id="expd-yoy-monthly" class="chart-h-260"></div>`,
        mount: (d) => mountYoyMonthly(d.year) },

    { id: 'aging_buckets', titleKey: 'view.exp_dash.widget.aging',
        html: () => `<div id="expd-aging" class="muted small">${esc(t('common.loading'))}</div>`,
        mount: () => mountAging() },

    { id: 'per_property', titleKey: 'view.exp_dash.widget.per_property', spans2: true,
        html: () => `<div id="expd-property" class="muted small">${esc(t('common.loading'))}</div>`,
        mount: (d) => mountPerProperty(d.year) },

    { id: 'anomalies', titleKey: 'view.exp_dash.widget.anomalies', spans2: true,
        html: () => `<div id="expd-anomalies" class="muted small">${esc(t('common.loading'))}</div>`,
        mount: () => mountAnomalies() },

    { id: 'drawdown', titleKey: 'view.exp_dash.widget.drawdown', spans2: true,
        html: () => `<div id="expd-drawdown" class="chart-h-260"></div>`,
        mount: (d) => mountDrawdown(d.daily, d.kpis.avg_daily) },

    { id: 'box_plot_categories', titleKey: 'view.exp_dash.widget.box_plot', spans2: true,
        html: () => `<canvas id="expd-boxplot" width="900" height="320"></canvas>`,
        mount: (d) => mountBoxPlot(d.year) },

    { id: 'tax_planner_calcs', titleKey: 'view.exp_dash.widget.tax_planner', spans2: true,
        html: () => taxPlannerHtml(),
        mount: () => wireTaxPlanner() },
];

const WIDGETS_BY_ID = new Map(WIDGETS.map((w) => [w.id, w]));
const DEFAULT_LAYOUT = WIDGETS.map((w) => w.id);

// ── Phase-2 widget mounts (subscriptions, Pareto, YoY, aging, etc.) ──────

async function mountSubscriptions() {
    const el = document.getElementById('expd-subs');
    if (!el) return;
    let rows = [];
    try { rows = await api.receiptsRecurring({ min_occurrences: 3 }); }
    catch { rows = []; }
    if (!rows.length) {
        el.innerHTML = `<p class="muted small">${esc(t('view.exp_dash.widget.subs_empty'))}</p>`;
        return;
    }
    const monthlyTotal = rows.reduce((s, r) =>
        s + (Number(r.average_amount) || 0) * (30 / Math.max(1, r.median_gap_days)), 0);
    const annualTotal = rows.reduce((s, r) => s + (Number(r.annualized_cost) || 0), 0);
    el.innerHTML = `
        <div class="expd-subs-strip">
            <div><span>${esc(t('view.exp_dash.widget.subs_monthly'))}</span>
                <strong class="tw-owed">${esc(fmtUsd(monthlyTotal))}</strong></div>
            <div><span>${esc(t('view.exp_dash.widget.subs_annual'))}</span>
                <strong class="tw-owed">${esc(fmtUsd(annualTotal))}</strong></div>
            <div><span>${esc(t('view.exp_dash.widget.subs_count'))}</span>
                <strong>${rows.length}</strong></div>
        </div>
        <table class="tax-merchants-table">
            <thead>
                <tr>
                    <th>${esc(t('view.exp_dash.widget.subs_merchant'))}</th>
                    <th class="num">${esc(t('view.exp_dash.widget.subs_avg'))}</th>
                    <th class="num">${esc(t('view.exp_dash.widget.subs_cadence'))}</th>
                    <th class="num">${esc(t('view.exp_dash.widget.subs_annualized'))}</th>
                    <th class="num">${esc(t('view.exp_dash.widget.subs_next'))}</th>
                    <th class="num">${esc(t('view.exp_dash.widget.subs_confidence'))}</th>
                </tr>
            </thead>
            <tbody>
                ${rows.slice(0, 30).map(r => `<tr>
                    <td>${esc(r.canonical_merchant)}</td>
                    <td class="num">${esc(fmtUsd(+r.average_amount))}</td>
                    <td class="num">${r.median_gap_days}d</td>
                    <td class="num">${esc(fmtUsd(+r.annualized_cost))}</td>
                    <td class="num">${esc(r.predicted_next_date)}</td>
                    <td class="num">${Math.round((r.confidence || 0) * 100)}%</td>
                </tr>`).join('')}
            </tbody>
        </table>`;
}

function drawPareto(canvasId, rows) {
    const c = document.getElementById(canvasId);
    if (!c || !rows?.length) return;
    const ctx = c.getContext('2d');
    const W = c.width, H = c.height;
    ctx.clearRect(0, 0, W, H);
    const sorted = [...rows].sort((a, b) => (+b.total) - (+a.total)).slice(0, 30);
    const totals = sorted.map(r => +r.total || 0);
    const grand = totals.reduce((s, v) => s + v, 0) || 1;
    let acc = 0;
    const cumPcts = totals.map(v => { acc += v; return acc / grand * 100; });

    const padL = 56, padR = 50, padT = 16, padB = 44;
    const plotW = W - padL - padR, plotH = H - padT - padB;
    const barW = Math.floor(plotW / sorted.length) - 3;
    const max = totals[0] || 1;
    ctx.font = '10px monospace';
    // Horizontal gridlines (left axis = $).
    ctx.fillStyle = '#33424f';
    [0, 0.25, 0.5, 0.75, 1].forEach(p => {
        ctx.fillRect(padL, padT + plotH * (1 - p), plotW, 1);
    });
    ctx.fillStyle = '#7a8ba8';
    ctx.textAlign = 'right';
    [0, 0.5, 1].forEach(p => {
        ctx.fillText(fmtUsd(max * p), padL - 6, padT + plotH * (1 - p) + 3);
    });
    // 80% line on right axis.
    const y80 = padT + plotH * (1 - 0.80);
    ctx.strokeStyle = '#ffd84a';
    ctx.setLineDash([4, 4]);
    ctx.beginPath(); ctx.moveTo(padL, y80); ctx.lineTo(padL + plotW, y80); ctx.stroke();
    ctx.setLineDash([]);
    ctx.fillStyle = '#ffd84a';
    ctx.textAlign = 'left';
    ctx.fillText('80%', padL + plotW + 4, y80 + 3);
    // Bars (left axis = $).
    ctx.textAlign = 'center';
    for (let i = 0; i < sorted.length; i++) {
        const x = padL + i * (barW + 3) + 2;
        const h = Math.round((totals[i] / max) * plotH);
        ctx.fillStyle = '#36c8d4';
        ctx.fillRect(x, padT + plotH - h, barW, h);
    }
    // Cumulative line (right axis = %).
    ctx.strokeStyle = '#ff7a3d';
    ctx.lineWidth = 2;
    ctx.beginPath();
    for (let i = 0; i < sorted.length; i++) {
        const x = padL + i * (barW + 3) + 2 + barW / 2;
        const y = padT + plotH * (1 - cumPcts[i] / 100);
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
    }
    ctx.stroke();
    ctx.lineWidth = 1;
    // Labels (rotated text would be ideal; abbreviating to first 6 chars).
    ctx.fillStyle = '#aab';
    ctx.font = '9px monospace';
    for (let i = 0; i < sorted.length; i++) {
        const x = padL + i * (barW + 3) + 2 + barW / 2;
        const label = String(sorted[i].label || sorted[i].canonical_merchant || '').slice(0, 8);
        ctx.save();
        ctx.translate(x, H - padB + 8);
        ctx.rotate(-Math.PI / 4);
        ctx.textAlign = 'right';
        ctx.fillText(label, 0, 0);
        ctx.restore();
    }
    // Right-axis legend (cum %).
    ctx.fillStyle = '#ff7a3d';
    ctx.textAlign = 'left';
    ctx.fillText('100%', padL + plotW + 4, padT + 8);
    ctx.fillText('0%', padL + plotW + 4, padT + plotH + 3);
}

async function mountYoyMonthly(year) {
    const el = document.getElementById('expd-yoy-monthly');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    let rows = [];
    try { rows = await api.receiptsYoyMonthly(year, activeBusinessId()); } catch { rows = []; }
    if (!rows.length) {
        el.innerHTML = `<p class="muted small">${esc(t('view.expenses.tax.chart.empty'))}</p>`;
        return;
    }
    const xs = rows.map(r => r.month);
    const cur = rows.map(r => +r.current);
    const prior = rows.map(r => +r.prior);
    const labels = ['','J','F','M','A','M','J','J','A','S','O','N','D'];
    new uPlot({
        width: el.clientWidth || 600,
        height: 260,
        scales: { x: { time: false }, y: { auto: true } },
        series: [
            {},
            { label: String(year), stroke: '#36c8d4', width: 2, fill: 'rgba(54,200,212,0.10)' },
            { label: String(year - 1), stroke: '#ff7a3d', width: 2, dash: [4, 4] },
        ],
        axes: [
            { stroke: '#aab', values: (_u, splits) => splits.map(v => labels[v] || '') },
            { stroke: '#aab', size: 64, values: (_u, splits) => splits.map(v => fmtUsd(v)) },
        ],
    }, [xs, cur, prior], el);
}

async function mountAging() {
    const el = document.getElementById('expd-aging');
    if (!el) return;
    let rows = [];
    try { rows = await api.receiptsAging(activeBusinessId()); } catch { rows = []; }
    const colors = { '0-7d': '#50ff80', '8-30d': '#ffd84a', '31-90d': '#ff7a3d', '90+d': '#ff3860' };
    const totalCount = rows.reduce((s, r) => s + (r.count || 0), 0);
    if (totalCount === 0) {
        el.innerHTML = `<p class="muted small">${esc(t('view.exp_dash.widget.aging_clean'))}</p>`;
        return;
    }
    el.innerHTML = `
        <div class="expd-aging-strip">
            ${rows.map(r => {
                const cls = (r.bucket === '90+d') ? 'tw-owed' : '';
                return `<div class="expd-aging-cell" style="border-left: 3px solid ${colors[r.bucket] || '#36c8d4'}">
                    <div class="expd-aging-bucket">${esc(r.bucket)}</div>
                    <div class="expd-aging-count ${cls}">${r.count}</div>
                    <div class="muted small">${esc(fmtUsd(+r.total))}</div>
                </div>`;
            }).join('')}
        </div>
        ${rows.find(r => r.bucket === '90+d' && r.count > 0)
            ? `<p class="muted small" style="margin-top:8px">${esc(t('view.exp_dash.widget.aging_warn'))}</p>`
            : ''}
    `;
}

async function mountPerProperty(year) {
    const el = document.getElementById('expd-property');
    if (!el) return;
    let rows = [];
    try { rows = await api.receiptsByProperty(year, activeBusinessId()); } catch { rows = []; }
    if (!rows.length) {
        el.innerHTML = `<p class="muted small">${esc(t('view.exp_dash.widget.property_empty'))}</p>`;
        return;
    }
    const max = rows.reduce((m, r) => Math.max(m, +r.total || 0), 0) || 1;
    el.innerHTML = `<div class="expd-property-list">
        ${rows.map(r => {
            const pct = Math.round((+r.total / max) * 100);
            const catRows = (r.top_categories || []).map(c => {
                const catLabel = t('view.expenses.cat.' + c.category, {}, c.category);
                return `<div class="expd-property-cat">
                    <span>${esc(catLabel)}</span>
                    <strong>${esc(fmtUsd(+c.total))}</strong>
                </div>`;
            }).join('');
            return `<div class="expd-property-row">
                <div class="expd-property-header">
                    <div>
                        <strong>${esc(r.property_name || t('view.exp_dash.widget.property_unassigned'))}</strong>
                        <span class="muted small">· ${r.item_count} ${esc(t('view.exp_dash.kpi.items'))}</span>
                    </div>
                    <strong class="tw-refund">${esc(fmtUsd(+r.total))}</strong>
                </div>
                <div class="expd-property-bar" data-bar-pct="${pct}"></div>
                <div class="expd-property-cats">${catRows}</div>
            </div>`;
        }).join('')}
    </div>`;
    // Apply bar widths
    el.querySelectorAll('[data-bar-pct]').forEach(b => {
        b.style.width = Math.max(0, Math.min(100, Number(b.dataset.barPct) || 0)) + '%';
    });
}

async function mountAnomalies() {
    const el = document.getElementById('expd-anomalies');
    if (!el) return;
    let rows = [];
    try { rows = await api.receiptsAnomalies(activeBusinessId()); } catch { rows = []; }
    if (!rows.length) {
        el.innerHTML = `<p class="muted small">${esc(t('view.exp_dash.widget.anomalies_clean'))}</p>`;
        return;
    }
    const iconFor = (k) =>
        k === 'subscription_jump' ? '⚠' :
        k === 'new_merchant' ? '✦' :
        k === 'outlier_receipt' ? '↑' : '·';
    const labelFor = (a) => {
        if (a.kind === 'subscription_jump')
            return t('view.exp_dash.widget.anom_jump', { label: a.label, pct: (+a.secondary).toFixed(0) });
        if (a.kind === 'new_merchant')
            return t('view.exp_dash.widget.anom_new', { label: a.label, when: a.when || '' });
        return t('view.exp_dash.widget.anom_outlier', { label: a.label });
    };
    el.innerHTML = `<div class="expd-anomalies-grid">
        ${rows.slice(0, 12).map(a => `<div class="expd-anomaly-card expd-anomaly-${a.kind}">
            <div class="expd-anomaly-icon">${iconFor(a.kind)}</div>
            <div class="expd-anomaly-body">
                <div class="expd-anomaly-label">${esc(labelFor(a))}</div>
                <strong class="expd-anomaly-value">${esc(fmtUsd(+a.value))}</strong>
            </div>
        </div>`).join('')}
    </div>`;
}

function mountDrawdown(daily, avgDaily) {
    const el = document.getElementById('expd-drawdown');
    if (!el || !window.uPlot || !daily?.length) return;
    el.innerHTML = '';
    const xs = daily.map(p => Math.floor(new Date(p.day + 'T00:00:00').getTime() / 1000));
    // Treat avg-daily-pace × day-of-year as the "target". Drawdown =
    // actual cumulative − target. Negative = under budget, positive = over.
    const avg = +avgDaily || 0;
    const ys = daily.map((p, i) => (+p.cumulative) - avg * (i + 1));
    new uPlot({
        width: el.clientWidth || 600,
        height: 260,
        scales: { x: { time: true }, y: { auto: true } },
        series: [
            {},
            { label: t('view.exp_dash.widget.drawdown_label'),
              stroke: '#ff3860', width: 2, fill: 'rgba(255,56,96,0.10)' },
        ],
        axes: [
            { stroke: '#aab' },
            { stroke: '#aab', size: 64, values: (_u, splits) => splits.map(v => fmtUsd(v)) },
        ],
    }, [xs, ys], el);
}

async function mountBoxPlot(year) {
    const canvas = document.getElementById('expd-boxplot');
    if (!canvas) return;
    let rows = [];
    try { rows = await api.receiptsCategoryDistribution(year, activeBusinessId()); } catch { rows = []; }
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
    const max = rows.reduce((m, r) => Math.max(m, +r.max || 0), 0) || 1;
    const padL = 130, padR = 30, padT = 14, padB = 30;
    const plotW = W - padL - padR, plotH = H - padT - padB;
    const rowH = Math.floor(plotH / rows.length);
    ctx.font = '10px monospace';
    // x grid
    ctx.fillStyle = '#33424f';
    [0, 0.25, 0.5, 0.75, 1].forEach(p => {
        ctx.fillRect(padL + plotW * p, padT, 1, plotH);
    });
    ctx.fillStyle = '#7a8ba8';
    ctx.textAlign = 'center';
    [0, 0.5, 1].forEach(p => {
        ctx.fillText(fmtUsd(max * p), padL + plotW * p, H - 12);
    });
    // box plots
    ctx.textAlign = 'right';
    rows.forEach((r, i) => {
        const y = padT + i * rowH + rowH / 2;
        const x = (v) => padL + (Math.max(0, +v) / max) * plotW;
        // whisker
        ctx.strokeStyle = '#7a8ba8';
        ctx.beginPath(); ctx.moveTo(x(+r.min), y); ctx.lineTo(x(+r.max), y); ctx.stroke();
        // box
        const boxH = Math.max(10, rowH * 0.45);
        ctx.fillStyle = '#36c8d4';
        ctx.fillRect(x(+r.q1), y - boxH / 2, Math.max(2, x(+r.q3) - x(+r.q1)), boxH);
        // median
        ctx.strokeStyle = '#0a0e14';
        ctx.lineWidth = 2;
        ctx.beginPath(); ctx.moveTo(x(+r.median), y - boxH / 2); ctx.lineTo(x(+r.median), y + boxH / 2); ctx.stroke();
        ctx.lineWidth = 1;
        // label
        ctx.fillStyle = '#aab';
        const catLabel = t('view.expenses.cat.' + r.category, {}, r.category);
        ctx.fillText(`${catLabel.slice(0, 20)} (${r.count})`, padL - 6, y + 4);
    });
}

function taxPlannerHtml() {
    return `<div class="expd-planner">
        <div class="expd-planner-card">
            <h3>${esc(t('view.exp_dash.widget.mileage'))}</h3>
            <label>${esc(t('view.exp_dash.widget.mileage_biz'))}
                <input type="number" id="expd-mi-biz" min="0" /></label>
            <label>${esc(t('view.exp_dash.widget.mileage_med'))}
                <input type="number" id="expd-mi-med" min="0" /></label>
            <label>${esc(t('view.exp_dash.widget.mileage_char'))}
                <input type="number" id="expd-mi-char" min="0" /></label>
            <button type="button" id="expd-mi-run" class="btn btn-secondary btn-compact">${esc(t('common.btn.compute'))}</button>
            <div id="expd-mi-out" class="expd-planner-out"></div>
        </div>
        <div class="expd-planner-card">
            <h3>${esc(t('view.exp_dash.widget.home_office'))}</h3>
            <label>${esc(t('view.exp_dash.widget.ho_office_sqft'))}
                <input type="number" id="expd-ho-office" min="0" /></label>
            <label>${esc(t('view.exp_dash.widget.ho_home_sqft'))}
                <input type="number" id="expd-ho-home" min="0" /></label>
            <label>${esc(t('view.exp_dash.widget.ho_expenses'))}
                <input type="number" id="expd-ho-exp" min="0" step="0.01" /></label>
            <label>${esc(t('view.exp_dash.widget.ho_income'))}
                <input type="number" id="expd-ho-inc" min="0" step="0.01" /></label>
            <button type="button" id="expd-ho-run" class="btn btn-secondary btn-compact">${esc(t('common.btn.compute'))}</button>
            <div id="expd-ho-out" class="expd-planner-out"></div>
        </div>
        <div class="expd-planner-card">
            <h3>${esc(t('view.exp_dash.widget.section_179'))}</h3>
            <label>${esc(t('view.exp_dash.widget.s179_cost'))}
                <input type="number" id="expd-s179-cost" min="0" step="0.01" /></label>
            <label>${esc(t('view.exp_dash.widget.s179_income'))}
                <input type="number" id="expd-s179-inc" min="0" step="0.01" /></label>
            <button type="button" id="expd-s179-run" class="btn btn-secondary btn-compact">${esc(t('common.btn.compute'))}</button>
            <div id="expd-s179-out" class="expd-planner-out"></div>
        </div>
    </div>`;
}

function wireTaxPlanner() {
    const miBtn = document.getElementById('expd-mi-run');
    if (miBtn) {
        miBtn.addEventListener('click', async () => {
            const out = document.getElementById('expd-mi-out');
            try {
                const r = await api.taxPlannerMileage({
                    business_miles: Number(document.getElementById('expd-mi-biz').value) || 0,
                    medical_miles: Number(document.getElementById('expd-mi-med').value) || 0,
                    charitable_miles: Number(document.getElementById('expd-mi-char').value) || 0,
                    active_duty_moving_miles: 0,
                });
                out.innerHTML = `<div><span>${esc(t('view.exp_dash.widget.mi_business_ded'))}</span>
                    <strong>${esc(fmtUsd(+r.business_deduction))}</strong></div>
                    <div><span>${esc(t('view.exp_dash.widget.mi_total'))}</span>
                    <strong class="tw-refund">${esc(fmtUsd(+r.total))}</strong></div>`;
            } catch (e) {
                out.innerHTML = `<div class="err">${esc(e.message)}</div>`;
            }
        });
    }
    const hoBtn = document.getElementById('expd-ho-run');
    if (hoBtn) {
        hoBtn.addEventListener('click', async () => {
            const out = document.getElementById('expd-ho-out');
            try {
                const r = await api.taxPlannerHomeOffice({
                    office_sqft: Number(document.getElementById('expd-ho-office').value) || 0,
                    home_sqft: Number(document.getElementById('expd-ho-home').value) || 0,
                    allocable_home_expenses: String(Number(document.getElementById('expd-ho-exp').value) || 0),
                    business_income_after_other_expenses: String(Number(document.getElementById('expd-ho-inc').value) || 0),
                });
                out.innerHTML = `<div><span>${esc(t('view.exp_dash.widget.ho_simplified'))}</span>
                    <strong>${esc(fmtUsd(+r.simplified_deduction))}</strong></div>
                    <div><span>${esc(t('view.exp_dash.widget.ho_actual'))}</span>
                    <strong>${esc(fmtUsd(+r.actual_deduction))}</strong></div>
                    <div><span>${esc(t('view.exp_dash.widget.ho_better'))}</span>
                    <strong class="tw-refund">${esc(fmtUsd(+r.max_deduction))} (${esc(r.better_method)})</strong></div>`;
            } catch (e) {
                out.innerHTML = `<div class="err">${esc(e.message)}</div>`;
            }
        });
    }
    const s179Btn = document.getElementById('expd-s179-run');
    if (s179Btn) {
        s179Btn.addEventListener('click', async () => {
            const out = document.getElementById('expd-s179-out');
            try {
                const r = await api.taxPlannerSection179({
                    total_qualifying_cost: String(Number(document.getElementById('expd-s179-cost').value) || 0),
                    trade_or_business_income: String(Number(document.getElementById('expd-s179-inc').value) || 0),
                    auto_cap: null,
                });
                out.innerHTML = `<div><span>§ 179</span>
                    <strong>${esc(fmtUsd(+r.s179_deduction))}</strong></div>
                    <div><span>${esc(t('view.exp_dash.widget.s179_bonus'))}</span>
                    <strong>${esc(fmtUsd(+r.bonus_depreciation))}</strong></div>
                    <div><span>${esc(t('view.exp_dash.widget.s179_first_year'))}</span>
                    <strong class="tw-refund">${esc(fmtUsd(+r.first_year_write_off))}</strong></div>`;
            } catch (e) {
                out.innerHTML = `<div class="err">${esc(e.message)}</div>`;
            }
        });
    }
}

// ── Render helpers (uPlot/canvas implementations) ─────────────────────────

function orderDow(dow) {
    // Backend returns Sun-first. Display Mon-Sun.
    const order = [1, 2, 3, 4, 5, 6, 0];
    return order.map((i) => ({ ...dow[i], label: ['Mon','Tue','Wed','Thu','Fri','Sat','Sun'][order.indexOf(i)] }));
}

function mountCumulative(daily) {
    const el = document.getElementById('expd-cumulative');
    if (!el || !window.uPlot || !daily?.length) return;
    el.innerHTML = '';
    const xs = daily.map((p) => Math.floor(new Date(p.day + 'T00:00:00').getTime() / 1000));
    const ys = daily.map((p) => +p.cumulative);
    new uPlot({
        width: el.clientWidth || 600,
        height: 260,
        scales: { x: { time: true }, y: { auto: true } },
        series: [{}, { label: t('view.exp_dash.chart.cumulative'),
            stroke: '#36c8d4', width: 2, fill: 'rgba(54,200,212,0.15)' }],
        axes: [
            { stroke: '#aab' },
            { stroke: '#aab', size: 64,
              values: (_u, splits) => splits.map((v) => fmtUsd(v)) },
        ],
    }, [xs, ys], el);
}

function mountDailyVolume(daily) {
    const el = document.getElementById('expd-daily-volume');
    if (!el || !window.uPlot || !daily?.length) return;
    el.innerHTML = '';
    const xs = daily.map((p) => Math.floor(new Date(p.day + 'T00:00:00').getTime() / 1000));
    const ys = daily.map((p) => +p.total);
    const barsPath = window.uPlot.paths.bars
        ? window.uPlot.paths.bars({ size: [0.6] })
        : null;
    new uPlot({
        width: el.clientWidth || 600,
        height: 260,
        scales: { x: { time: true }, y: { auto: true } },
        series: [
            {},
            { label: t('view.exp_dash.chart.daily_volume'),
              stroke: 'transparent', fill: '#36c8d4',
              ...(barsPath ? { paths: barsPath } : { width: 1, stroke: '#36c8d4' }) },
        ],
        axes: [
            { stroke: '#aab' },
            { stroke: '#aab', size: 64,
              values: (_u, splits) => splits.map((v) => fmtUsd(v)) },
        ],
    }, [xs, ys], el);
}

function drawBars(canvasId, rows) {
    const c = document.getElementById(canvasId);
    if (!c || !rows?.length) return;
    const ctx = c.getContext('2d');
    const W = c.width, H = c.height;
    ctx.clearRect(0, 0, W, H);
    const max = rows.reduce((m, r) => Math.max(m, +r.total || 0), 0) || 1;
    const padL = 50, padR = 14, padT = 14, padB = 30;
    const plotW = W - padL - padR;
    const plotH = H - padT - padB;
    const barW = Math.floor(plotW / rows.length) - 4;
    ctx.font = '10px monospace';
    ctx.fillStyle = '#33424f';
    for (let p = 0; p <= 4; p++) {
        const y = padT + (plotH * p / 4);
        ctx.fillRect(padL, y, plotW, 1);
    }
    ctx.fillStyle = '#7a8ba8';
    ctx.textAlign = 'right';
    [0, 0.5, 1].forEach((p) => {
        const v = max * (1 - p);
        const y = padT + plotH * p + 3;
        ctx.fillText(fmtUsd(v), padL - 6, y);
    });
    ctx.textAlign = 'center';
    for (let i = 0; i < rows.length; i++) {
        const x = padL + i * (barW + 4) + 2;
        const h = Math.round((+rows[i].total / max) * plotH);
        ctx.fillStyle = '#36c8d4';
        ctx.fillRect(x, padT + plotH - h, barW, h);
        ctx.fillStyle = '#aab';
        ctx.fillText(String(rows[i].label || ''), x + barW / 2, H - 14);
        if (rows[i].count != null) {
            ctx.fillStyle = '#7a8ba8';
            ctx.fillText(String(rows[i].count), x + barW / 2, H - 2);
        }
    }
}

function drawPie(canvasId, rows) {
    const c = document.getElementById(canvasId);
    if (!c) return;
    const ctx = c.getContext('2d');
    const W = c.width, H = c.height;
    ctx.clearRect(0, 0, W, H);
    if (!rows?.length) {
        ctx.fillStyle = '#7a8ba8';
        ctx.font = '11px monospace';
        ctx.textAlign = 'center';
        ctx.fillText(t('view.expenses.tax.chart.empty'), W / 2, H / 2);
        return;
    }
    const top = rows.slice(0, 8);
    const grand = top.reduce((a, r) => a + (+r.total || 0), 0) || 1;
    const palette = ['#36c8d4','#ff7a3d','#7fff8a','#b86bff','#ff5fa7','#ffd84a','#79c8ff','#ff3860'];
    const cx = H / 2 + 6, cy = H / 2;
    const radius = H / 2 - 14;
    let acc = -Math.PI / 2;
    top.forEach((r, i) => {
        const slice = ((+r.total) / grand) * Math.PI * 2;
        ctx.beginPath();
        ctx.moveTo(cx, cy);
        ctx.arc(cx, cy, radius, acc, acc + slice);
        ctx.closePath();
        ctx.fillStyle = palette[i % palette.length];
        ctx.fill();
        acc += slice;
    });
    ctx.font = '10px monospace';
    ctx.textAlign = 'left';
    let ly = 12;
    const legendX = H + 20;
    top.forEach((r, i) => {
        const pct = ((+r.total / grand) * 100).toFixed(1);
        ctx.fillStyle = palette[i % palette.length];
        ctx.fillRect(legendX, ly - 8, 10, 10);
        ctx.fillStyle = '#e0f0ff';
        ctx.fillText(`${String(r.label).slice(0, 20)} ${pct}%`, legendX + 14, ly);
        ly += 14;
        if (ly > H - 6) return;
    });
}

function leaderboardHtml(rows, mode) {
    if (!rows?.length) {
        return `<p class="muted small">${esc(t('view.expenses.tax.chart.empty'))}</p>`;
    }
    const key = mode === 'count' ? 'count' : 'total';
    const max = rows.reduce((m, r) => Math.max(m, +r[key] || 0), 0) || 1;
    return `<table class="tax-merchants-table"><tbody>
        ${rows.map((r) => {
            const pct = Math.round(((+r[key]) / max) * 100);
            return `<tr>
                <td class="tax-merchant-name">
                    <div class="tax-merchant-bar" data-bar-pct="${pct}"></div>
                    <span>${esc(r.label)}</span>
                </td>
                <td class="num">${esc(fmtUsd(+r.total))}</td>
                <td class="num">${r.count}</td>
            </tr>`;
        }).join('')}
    </tbody></table>`;
}

function categoryLeaderboardHtml(rows) {
    if (!rows?.length) {
        return `<p class="muted small">${esc(t('view.expenses.tax.chart.empty'))}</p>`;
    }
    const max = rows.reduce((m, r) => Math.max(m, +r.total || 0), 0) || 1;
    return `<table class="tax-merchants-table"><tbody>
        ${rows.map((r) => {
            const pct = Math.round((+r.total / max) * 100);
            const labelKey = 'view.expenses.cat.' + r.label;
            const human = t(labelKey, {}, r.label);
            return `<tr>
                <td class="tax-merchant-name">
                    <div class="tax-merchant-bar" data-bar-pct="${pct}"></div>
                    <span>${esc(human)}</span>
                </td>
                <td class="num">${esc(fmtUsd(+r.total))}</td>
                <td class="num">${r.count}</td>
            </tr>`;
        }).join('')}
    </tbody></table>`;
}

function drawCalendar(days) {
    const canvas = document.getElementById('expd-calendar');
    const hover = document.getElementById('expd-cal-hover');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const W = canvas.width, H = canvas.height;
    ctx.clearRect(0, 0, W, H);
    if (!days?.length) return;
    const positives = days.filter((d) => +d.total > 0).map((d) => +d.total).sort((a, b) => a - b);
    const q = (p) => positives[Math.floor(positives.length * p)] || 0;
    const t1 = q(0.25), t2 = q(0.50), t3 = q(0.75), t4 = q(0.93);
    const bucketOf = (v) => v <= 0 ? 0 : v <= t1 ? 1 : v <= t2 ? 2 : v <= t3 ? 3 : v <= t4 ? 4 : 5;
    const COL = ['#16202c', '#0d3a4d', '#0d556e', '#108a9f', '#36c8d4', '#7af0ff'];

    const TOP = 28;
    const cell = Math.floor(Math.min((W - 30) / 53, (H - TOP) / 7));
    const pad = Math.max(1, Math.floor(cell * 0.12));
    const cells = [];
    const firstDay = new Date(days[0].day + 'T00:00:00');
    const firstColOffset = firstDay.getDay();
    let prevMonth = -1;
    ctx.font = '10px monospace';
    ctx.textAlign = 'left';
    for (let i = 0; i < days.length; i++) {
        const d = days[i];
        const dt = new Date(d.day + 'T00:00:00');
        const dayOfWeek = dt.getDay();
        const col = Math.floor((i + firstColOffset) / 7);
        const x = 24 + col * cell;
        const y = TOP + dayOfWeek * cell;
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

// ── Layout persistence (localStorage; mirrors trade dashboard's pattern) ──

function loadLayout() {
    try {
        const raw = localStorage.getItem(layoutKey());
        if (!raw) return DEFAULT_LAYOUT.slice();
        const arr = JSON.parse(raw);
        if (!Array.isArray(arr)) return DEFAULT_LAYOUT.slice();
        // Append any new widgets added since this layout was saved.
        const known = new Set(arr);
        const missing = DEFAULT_LAYOUT.filter((id) => !known.has(id));
        return [...arr.filter((id) => WIDGETS_BY_ID.has(id)), ...missing];
    } catch {
        return DEFAULT_LAYOUT.slice();
    }
}

function persistLayout(order) {
    try { localStorage.setItem(layoutKey(), JSON.stringify(order)); } catch {}
}

function getYear() {
    const v = Number(localStorage.getItem(YEAR_KEY));
    return Number.isFinite(v) && v >= 2000 ? v : new Date().getFullYear();
}
function setYear(y) {
    if (Number.isFinite(y)) localStorage.setItem(YEAR_KEY, String(y));
}

// ── Top-level render entry ───────────────────────────────────────────────

export async function renderExpenseDashboard(mount) {
    const tok = currentViewToken();
    const year = getYear();
    mount.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div>
        <div class="tv-spinner-text">${esc(t('common.loading'))}</div></div>`;

    let data;
    try {
        data = await api.expenseDashboardBundle(year, activeBusinessId());
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        mount.innerHTML = `<p class="boot">${esc(t('view.exp_dash.load_failed', { err: e.message }))}</p>`;
        return;
    }
    if (!viewIsCurrent(tok)) return;

    const layout = loadLayout();
    const currentYear = new Date().getFullYear();
    const yearOpts = Array.from({ length: 6 }, (_, i) => currentYear - i)
        .map((y) => `<option value="${y}"${y === year ? ' selected' : ''}>${y}</option>`).join('');

    mount.innerHTML = `
        <div class="expd-header">
            <h1 class="view-title">
                <span data-i18n="view.exp_dash.title">// BUSINESS EXPENSE DASHBOARD</span>
            </h1>
            <div class="expd-controls">
                <span id="expd-biz-selector"></span>
                <label>${esc(t('view.exp_dash.year'))}
                    <select id="expd-year">${yearOpts}</select></label>
                <button type="button" id="expd-reset-layout" class="btn btn-secondary btn-compact">
                    ${esc(t('view.exp_dash.reset_layout'))}
                </button>
            </div>
        </div>
        <div id="expd-grid" class="expd-grid">
            ${renderLayoutPanels(layout, data)}
        </div>
    `;

    // Business selector — populates async, re-renders the dashboard
    // whenever the user picks a different business.
    const selHost = mount.querySelector('#expd-biz-selector');
    if (selHost) mountBusinessSelector(selHost);
    const unsubBiz = onBusinessChange(() => renderExpenseDashboard(mount));
    mount.__expdUnsubBiz = unsubBiz;

    // Mount-phase pass for widgets that need post-DOM init (uPlot etc.)
    requestAnimationFrame(() => {
        layout.forEach((id) => {
            const widget = WIDGETS_BY_ID.get(id);
            if (widget?.mount) {
                try { widget.mount(data, mount.querySelector(`[data-widget-id="${id}"]`)); }
                catch (e) { console.warn(`widget ${id} mount failed`, e); }
            }
        });
        applyBarPctWidths(mount);
        attachLayoutHandlers(mount, layout);
    });

    // Year selector — reloads.
    mount.querySelector('#expd-year')?.addEventListener('change', (e) => {
        setYear(Number(e.target.value));
        renderExpenseDashboard(mount);
    });
    mount.querySelector('#expd-reset-layout')?.addEventListener('click', () => {
        persistLayout(DEFAULT_LAYOUT);
        renderExpenseDashboard(mount);
    });
}

function renderLayoutPanels(layout, data) {
    return layout
        .map((id) => WIDGETS_BY_ID.get(id))
        .filter(Boolean)
        .map((w) => `
            <div class="chart-panel${w.spans2 ? ' expd-span-2' : ''}" data-widget-id="${w.id}">
                <span class="dash-tv-drag-handle" title="drag to reorder" data-drag-handle>⠿</span>
                <span class="dash-tv-del-btn" title="hide" data-del-widget="${w.id}">✕</span>
                <h2 data-i18n="${w.titleKey}">${esc(t(w.titleKey))}</h2>
                ${w.html(data)}
            </div>
        `).join('');
}

function applyBarPctWidths(mount) {
    mount.querySelectorAll('[data-bar-pct]').forEach((el) => {
        const pct = Math.max(0, Math.min(100, Number(el.dataset.barPct) || 0));
        el.style.width = pct + '%';
    });
}

function attachLayoutHandlers(mount, layout) {
    const grid = mount.querySelector('#expd-grid');
    if (!grid) return;
    resetDragReorder(grid);
    initDragReorder(grid, '.chart-panel[data-widget-id]', null, {
        direction: 'vertical',
        handleSelector: '[data-drag-handle], .chart-panel > h2',
        getKey: (el) => el.dataset.widgetId,
        persist: (newOrder) => persistLayout(newOrder),
        toastMessage: t('view.exp_dash.reordered'),
    });
    grid.querySelectorAll('[data-del-widget]').forEach((btn) => {
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            const panel = btn.closest('.chart-panel[data-widget-id]');
            if (panel) panel.remove();
            const next = [...grid.querySelectorAll('.chart-panel[data-widget-id]')]
                .map((el) => el.dataset.widgetId);
            persistLayout(next);
            try { showToast(t('toast.widget_removed'), { level: 'success' }); } catch (_) {}
        });
    });
}
