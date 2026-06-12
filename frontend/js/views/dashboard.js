import { api } from '../api.js';
import { fmtMoney, fmtSecs, pnlClass, applyBarWidths } from '../util.js';
import { equityChart, barChart, zoomPlugin } from '../charts.js';
import { renderWorldMarkets } from './world_map.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { initDragReorder, resetDragReorder } from '../drag_reorder.js';
import * as dashStore from '../_dashboards_storage.js';
import { showToast } from '../toast.js';
import { localToday } from '../local_date.js';

const INTERVAL_KEY = 'dashboard_interval_days';
const PERIOD_KEY   = 'dashboard_period_key';
const VALID_INTERVALS = [30, 60, 90];
// Calendar-aware quick-pick periods. Each maps to a number of days that
// covers the period; 'all' clears the filter. Computed lazily so YTD
// stretches the right amount each call.
const PERIOD_KEYS = ['today', 'wtd', 'mtd', 'qtd', 'ytd', 'all'];

// Fetch every analytics-dashboard slice in parallel. Returns the same
// `data` shape the in-page widgets expect, plus the saved widget order.
// `failedFetches` (optional) accumulates `{name, msg}` for failed
// endpoints so the caller can surface a banner without us coupling to
// that UI here. Exported so views/dashboards.js can render the same
// graph widgets in pinned-tile context.
export async function loadAnalyticsBundle(accountId, interval, failedFetches = []) {
    const swallow = (name, fallback) => (e) => {
        const msg = e?.message || String(e);
        failedFetches.push({ name, msg });
        // eslint-disable-next-line no-console
        console.warn(`[dashboard] ${name} failed:`, msg);
        return fallback;
    };
    const [summary, equity, cal, dow, hold, hour, dd, byPrice, daily, tags,
           byMonth, bySymbol, byDurationCoarse, byRBucket, byOpeningGap,
           byInstrumentVolume, byMovement, byBroker, openTrades, layout] = await Promise.all([
        api.summary(accountId, interval),
        api.equity(accountId, undefined, interval),
        api.calendar(accountId, interval),
        api.byDow(accountId, interval).catch(swallow('byDow', [])),
        api.byHold(accountId, interval).catch(swallow('byHold', [])),
        api.byHour(accountId, interval).catch(swallow('byHour', [])),
        api.drawdown(accountId, undefined, interval).catch(swallow('drawdown', null)),
        api.byPrice(accountId, interval).catch(swallow('byPrice', [])),
        api.dailySeries(accountId, interval).catch(swallow('dailySeries', [])),
        api.byTag(accountId, interval).catch(swallow('byTag', [])),
        api.byMonth(accountId, interval).catch(swallow('byMonth', [])),
        api.bySymbol(accountId, interval).catch(swallow('bySymbol', [])),
        api.byDurationCoarse(accountId, interval).catch(swallow('byDurationCoarse', [])),
        api.byRBucket(accountId, interval).catch(swallow('byRBucket', [])),
        api.byOpeningGap(accountId, interval).catch(swallow('byOpeningGap', [])),
        api.byInstrumentVolume(accountId, interval).catch(swallow('byInstrumentVolume', [])),
        api.byMovement(accountId, interval).catch(swallow('byMovement', [])),
        api.byBroker(accountId, interval).catch(swallow('byBroker', [])),
        api.trades(accountId, { status: 'open', limit: 100 }).catch(swallow('openTrades', [])),
        loadLayout(),
    ]);
    const data = { equity, summary, dow, hold, hour, byPrice, dd, daily, tags, cal,
                   byMonth, bySymbol, byDurationCoarse, byRBucket, byOpeningGap,
                   byInstrumentVolume, byMovement, byBroker, openTrades };
    return { data, layout };
}

function getInterval() {
    const v = Number(localStorage.getItem(INTERVAL_KEY));
    return VALID_INTERVALS.includes(v) ? v : 90;
}
function setInterval(days) {
    if (VALID_INTERVALS.includes(days)) localStorage.setItem(INTERVAL_KEY, String(days));
}
function getPeriod() {
    const v = localStorage.getItem(PERIOD_KEY);
    return PERIOD_KEYS.includes(v) ? v : null;
}
function setPeriod(p) {
    if (p === null || p === '') localStorage.removeItem(PERIOD_KEY);
    else if (PERIOD_KEYS.includes(p)) localStorage.setItem(PERIOD_KEY, p);
}

/**
 * Convert a calendar period ('today', 'wtd', etc.) into a rolling `days`
 * count from today back to the start of that period. `null` means no filter.
 */
function periodToDays(period) {
    if (!period || period === 'all') return null;
    const now = new Date();
    const start = new Date(now);
    if (period === 'today') start.setHours(0, 0, 0, 0);
    else if (period === 'wtd') {
        const dow = now.getDay();
        start.setDate(start.getDate() - dow);  // Sunday-anchored week
        start.setHours(0, 0, 0, 0);
    } else if (period === 'mtd') {
        start.setDate(1); start.setHours(0, 0, 0, 0);
    } else if (period === 'qtd') {
        const m = now.getMonth();
        const qStart = m - (m % 3);
        start.setMonth(qStart, 1); start.setHours(0, 0, 0, 0);
    } else if (period === 'ytd') {
        start.setMonth(0, 1); start.setHours(0, 0, 0, 0);
    }
    const ms = now - start;
    return Math.max(1, Math.ceil(ms / (1000 * 60 * 60 * 24)));
}

function esc(s) { return String(s).replace(/[&<>"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c])); }

const DAY_NAMES = ['Sun','Mon','Tue','Wed','Thu','Fri','Sat'];
const MONTH_NAMES = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];

function rangeLabel(days) {
    const end = new Date();
    const start = new Date(end);
    start.setDate(start.getDate() - days);
    const startStr = `${MONTH_NAMES[start.getMonth()]}`;
    const endStr   = `${MONTH_NAMES[end.getMonth()]} ${end.getFullYear()}`;
    return startStr === endStr.split(' ')[0] ? endStr : `${startStr} - ${endStr}`;
}

// One tile per calendar day across the selected window, in a horizontally
// scrollable strip (renderDashboard scrolls it to today on mount). 'All
// time' (no rolling window) renders a full year of tiles regardless of how
// far back the data goes — sizing it to the earliest calendar entry left a
// young account with 7 tiles and nothing to scroll. 365 is also the cap so
// a years-deep account doesn't render thousands of nodes.
function dayStrip(cal, windowDays) {
    const map = new Map((cal || []).map(c => [c.day, c]));
    const cells = [];
    const today = new Date();
    const days = Math.max(7, Math.min(windowDays || 365, 365));
    for (let i = days - 1; i >= 0; i--) {
        const d = new Date(today);
        d.setDate(d.getDate() - i);
        // Use LOCAL date for the lookup key — `toISOString().slice(0,10)`
        // is UTC, so after ~5pm PT / 8pm ET the cell labeled "today"
        // (built from local d.getDate()) would query the next day's UTC
        // key and miss every trade that landed today.
        const key = localToday(d);
        const c = map.get(key);
        const pnl = Number(c?.net_pnl) || 0;
        const trades = Number(c?.trades) || 0;
        const cls = trades === 0 ? '' : pnl > 0 ? 'pos' : pnl < 0 ? 'neg' : '';
        cells.push(`
            <div class="dash-tv-day ${cls}">
                <div class="dash-tv-day-head">
                    <span class="dash-tv-day-num">${d.getDate()}</span>
                    <span class="dash-tv-day-name">${DAY_NAMES[d.getDay()]}</span>
                </div>
                <div class="dash-tv-day-pnl ${pnlClass(pnl)}">${pnl === 0 ? '$0' : fmtMoney(pnl)}</div>
                <div class="dash-tv-day-trades">${trades} ${esc(t(trades === 1 ? 'view.dashboard.day_strip.trade_singular' : 'view.dashboard.day_strip.trade_plural'))}</div>
            </div>
        `);
    }
    return cells.join('');
}

function compareRow(label, value, ratio, kind) {
    const pct = Math.max(0, Math.min(100, ratio * 100));
    return `
        <div class="dash-tv-compare-row">
            <div class="dash-tv-compare-label">${esc(label)}</div>
            <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${kind}" data-bar-pct="${pct}"></div></div>
            <div class="dash-tv-compare-value ${kind}">${esc(value)}</div>
        </div>
    `;
}

function compareWidget(rows) {
    return `<div class="dash-tv-compare">${rows.join('')}</div>`;
}

function winLossCount(s) {
    const max = Math.max(s.win_count, s.loss_count, 1);
    return compareWidget([
        compareRow(t('view.dashboard.tv.winning'), String(s.win_count), s.win_count / max, 'pos'),
        compareRow(t('view.dashboard.tv.losing'),  String(s.loss_count), s.loss_count / max, 'neg'),
    ]);
}

function holdTimeWinLoss(s) {
    const max = Math.max(s.avg_win_hold_seconds, s.avg_loss_hold_seconds, 1);
    return compareWidget([
        compareRow(t('view.dashboard.tv.winning'), fmtSecs(s.avg_win_hold_seconds),  s.avg_win_hold_seconds  / max, 'pos'),
        compareRow(t('view.dashboard.tv.losing'),  fmtSecs(s.avg_loss_hold_seconds), s.avg_loss_hold_seconds / max, 'neg'),
    ]);
}

function avgWinLoss(s) {
    const aw = Number(s.avg_win) || 0;
    const al = Math.abs(Number(s.avg_loss) || 0);
    const max = Math.max(aw, al, 1);
    return compareWidget([
        compareRow(t('view.dashboard.tv.winning'), fmtMoney(aw),    aw / max, 'pos'),
        compareRow(t('view.dashboard.tv.losing'),  fmtMoney(-al),   al / max, 'neg'),
    ]);
}

function largestGainLoss(s) {
    const lw = Number(s.largest_win)  || 0;
    const ll = Math.abs(Number(s.largest_loss) || 0);
    const max = Math.max(lw, ll, 1);
    return compareWidget([
        compareRow(t('view.dashboard.tv.gain'), fmtMoney(lw),  lw / max, 'pos'),
        compareRow(t('view.dashboard.tv.loss'), fmtMoney(-ll), ll / max, 'neg'),
    ]);
}

function avgMfeMae(s) {
    const mfe = Number(s.avg_mfe) || 0;
    const mae = Math.abs(Number(s.avg_mae) || 0);
    if (mfe === 0 && mae === 0) {
        return `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`;
    }
    const max = Math.max(mfe, mae, 1);
    return compareWidget([
        compareRow(t('view.dashboard.tv.mfe'), fmtMoney(mfe),  mfe / max, 'pos'),
        compareRow(t('view.dashboard.tv.mae'), fmtMoney(-mae), mae / max, 'neg'),
    ]);
}

function tagBreakdownWidget(tags) {
    if (!Array.isArray(tags) || !tags.length) {
        return `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`;
    }
    const top = [...tags]
        .map(t => ({ ...t, net: Number(t.net_pnl) || 0 }))
        .sort((a, b) => Math.abs(b.net) - Math.abs(a.net))
        .slice(0, 8);
    const maxAbs = Math.max(...top.map(t => Math.abs(t.net)), 1);
    return compareWidget(top.map(b => `
        <div class="dash-tv-compare-row">
            <div class="dash-tv-compare-label">${esc(String(b.key))}</div>
            <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${b.net >= 0 ? 'pos' : 'neg'}" data-bar-pct="${(Math.abs(b.net) / maxAbs * 100).toFixed(0)}"></div></div>
            <div class="dash-tv-compare-value ${pnlClass(b.net)}">${fmtMoney(b.net)}</div>
        </div>
    `));
}

function perfByBucketsWidget(rows, options = {}) {
    if (!Array.isArray(rows) || !rows.length) {
        return `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`;
    }
    const limit = options.limit || 12;
    const sorter = options.preserveOrder
        ? (rs) => rs
        : (rs) => [...rs].sort((a, b) => Math.abs(Number(b.net_pnl) || 0) - Math.abs(Number(a.net_pnl) || 0));
    const top = sorter(rows).slice(0, limit);
    const maxAbs = Math.max(...top.map(r => Math.abs(Number(r.net_pnl) || 0)), 1);
    return compareWidget(top.map(r => {
        const v = Number(r.net_pnl) || 0;
        return `
            <div class="dash-tv-compare-row">
                <div class="dash-tv-compare-label">${esc(String(r.key))}</div>
                <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${v >= 0 ? 'pos' : 'neg'}" data-bar-pct="${(Math.abs(v) / maxAbs * 100).toFixed(0)}"></div></div>
                <div class="dash-tv-compare-value ${pnlClass(v)}">${fmtMoney(v)}</div>
            </div>
        `;
    }));
}

function openTradesWidget(trades) {
    if (!Array.isArray(trades) || trades.length === 0) {
        return `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_open_trades'))}</div>`;
    }
    return `
        <table class="open-trades">
            <thead><tr>
                <th>${esc(t('view.dashboard.tv.open.symbol'))}</th>
                <th>${esc(t('view.dashboard.tv.open.side'))}</th>
                <th>${esc(t('view.dashboard.tv.open.qty'))}</th>
                <th>${esc(t('view.dashboard.tv.open.entry'))}</th>
                <th>${esc(t('view.dashboard.tv.open.opened'))}</th>
            </tr></thead>
            <tbody>${trades.map(o => `
                <tr>
                    <td><a href="#trade/${esc(o.id)}">${esc(o.symbol)}</a></td>
                    <td>${esc(o.side)}</td>
                    <td>${Number(o.qty || 0).toLocaleString()}</td>
                    <td>${fmtMoney(o.entry_avg)}</td>
                    <td>${(o.opened_at || '').slice(0, 10)}</td>
                </tr>
            `).join('')}</tbody>
        </table>
    `;
}

function heroStat(value, klass = 'hero-num-cyan', sub = '') {
    return `
        <div class="hero-stat hero-stat-band">
            <div class="hero-num hero-num-md ${klass}">${esc(String(value))}</div>
            ${sub ? `<div class="muted small">${esc(sub)}</div>` : ''}
        </div>
    `;
}

function perfDayTypeWidget(cal) {
    if (!Array.isArray(cal) || !cal.length) {
        return `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`;
    }
    // Bucket each calendar day by sign of net P&L; sum per bucket. "Trading
    // day" classification matches how Tradervue's "Performance by Day Type"
    // groups results.
    let winSum = 0, lossSum = 0, scratchSum = 0;
    let winN = 0, lossN = 0, scratchN = 0;
    for (const c of cal) {
        const v = Number(c.net_pnl) || 0;
        const tr = Number(c.trades) || 0;
        if (tr === 0) continue;
        if (v > 0) { winSum += v; winN++; }
        else if (v < 0) { lossSum += v; lossN++; }
        else { scratchSum += v; scratchN++; }
    }
    const maxAbs = Math.max(Math.abs(winSum), Math.abs(lossSum), Math.abs(scratchSum), 1);
    const row = (label, sum, n, sign) => `
        <div class="dash-tv-compare-row">
            <div class="dash-tv-compare-label">${esc(label)}</div>
            <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${sign}" data-bar-pct="${(Math.abs(sum) / maxAbs * 100).toFixed(0)}"></div></div>
            <div class="dash-tv-compare-value ${pnlClass(sum)}">${fmtMoney(sum)} <span class="muted cmp-pct-muted">${n} ${n === 1 ? 'day' : 'days'}</span></div>
        </div>
    `;
    return compareWidget([
        row(t('view.dashboard.tv.winning_days'), winSum, winN, 'pos'),
        row(t('view.dashboard.tv.losing_days'),  lossSum, lossN, 'neg'),
        row(t('view.dashboard.tv.scratch_days'), scratchSum, scratchN, 'pos'),
    ]);
}

function dayOfWeekWidget(dow) {
    if (!Array.isArray(dow) || !dow.length) {
        return `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`;
    }
    // Backend bucket keys are "1_mon", "2_tue", ..., "7_sun" — preserving
    // chronological week order. Map them to display labels Mon..Sun.
    // (Previously this widget looked up "Mon", "Tue", ... directly against
    // the bucket map, missed every key, and showed every bar as $0.)
    const DOW = [
        ['1_mon', 'Mon'], ['2_tue', 'Tue'], ['3_wed', 'Wed'],
        ['4_thu', 'Thu'], ['5_fri', 'Fri'], ['6_sat', 'Sat'], ['7_sun', 'Sun'],
    ];
    const byKey = new Map(dow.map(b => [b.key, b]));
    const maxAbs = Math.max(...dow.map(b => Math.abs(Number(b.net_pnl) || 0)), 1);
    const total  = dow.reduce((a, b) => a + Math.abs(Number(b.net_pnl) || 0), 0) || 1;
    return compareWidget(DOW.map(([key, label]) => {
        const b = byKey.get(key) || { net_pnl: 0, trades: 0 };
        const v = Number(b.net_pnl) || 0;
        const pct = (Math.abs(v) / total) * 100;
        return `
            <div class="dash-tv-compare-row">
                <div class="dash-tv-compare-label">${label}</div>
                <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${v >= 0 ? 'pos' : 'neg'}" data-bar-pct="${(Math.abs(v) / maxAbs * 100).toFixed(0)}"></div></div>
                <div class="dash-tv-compare-value ${pnlClass(v)}">${fmtMoney(v)} <span class="muted cmp-pct-muted">${pct.toFixed(2)}%</span></div>
            </div>
        `;
    }));
}

function hourOfDayWidget(hour) {
    if (!Array.isArray(hour) || !hour.length) {
        return `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`;
    }
    const maxAbs = Math.max(...hour.map(b => Math.abs(Number(b.net_pnl) || 0)), 1);
    const total  = hour.reduce((a, b) => a + Math.abs(Number(b.net_pnl) || 0), 0) || 1;
    return compareWidget(hour.map(b => {
        const v = Number(b.net_pnl) || 0;
        const pct = (Math.abs(v) / total) * 100;
        return `
            <div class="dash-tv-compare-row">
                <div class="dash-tv-compare-label">${esc(String(b.key))}</div>
                <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${v >= 0 ? 'pos' : 'neg'}" data-bar-pct="${(Math.abs(v) / maxAbs * 100).toFixed(0)}"></div></div>
                <div class="dash-tv-compare-value ${pnlClass(v)}">${fmtMoney(v)} <span class="muted cmp-pct-muted">${pct.toFixed(2)}%</span></div>
            </div>
        `;
    }));
}

function profitFactorGauge(pf) {
    const v = Number(pf) || 0;
    // Normalize: 0 → empty arc, 1 → half arc, 2+ → full arc
    const clamped = Math.min(Math.max(v / 2, 0), 1);
    const angle = 180 * clamped; // 0..180 deg semicircle sweep
    const cls = v >= 1.5 ? 'pos' : v >= 1.0 ? 'warn' : 'neg';
    const r = 70;
    const cx = 90, cy = 90;
    const rad = (angle - 180) * Math.PI / 180;
    const endX = cx + r * Math.cos(rad);
    const endY = cy + r * Math.sin(rad);
    const largeArc = angle > 180 ? 1 : 0;
    return `
        <div class="hero-stat hero-stat-tight">
            <div class="hero-num hero-num-md pf-num-${cls}">${v.toFixed(2)}</div>
            <svg width="180" height="100" viewBox="0 0 180 100">
                <path d="M 20 90 A 70 70 0 0 1 160 90" stroke="rgba(255,255,255,0.08)" stroke-width="10" fill="none"/>
                <path d="M 20 90 A 70 70 0 ${largeArc} 1 ${endX.toFixed(2)} ${endY.toFixed(2)}"
                      class="pf-arc-${cls}" stroke-width="10" fill="none"/>
            </svg>
        </div>
    `;
}

/**
 * Plot the running drawdown (≤ 0) from the equity curve. We pass `equity`
 * (Vec<EquityPoint>) here, not the MaxDrawdown summary — EquityPoint already
 * has `drawdown` per day, and the dashboard already fetches equity. Earlier
 * the widget tried to read `dd.series` from MaxDrawdown which doesn't carry
 * a series field, so the chart silently bailed out.
 */
function drawdownChart(elId, equity) {
    if (!Array.isArray(equity) || !equity.length) return false;
    setTimeout(() => {
        const el = document.getElementById(elId);
        if (!el || !window.uPlot) return;
        const xs = equity.map((_, i) => i);
        const ys = equity.map(p => Number(p.drawdown) || 0);
        const labels = equity.map(p => shortDay(p.day));
        new window.uPlot({
            title: '', width: el.clientWidth || 600, height: 260,
            scales: { x: { time: false,}, y: { auto: true } },
            series: [
                { label: 'day' },
                { label: 'drawdown', stroke: '#ff3860', width: 2,
                  fill: 'rgba(255,56,96,0.18)' },
            ],
            axes: [
                { stroke: '#aab', size: 60, rotate: -45,
                  values: (_u, splits) => splits.map(v => labels[Math.round(v)] || '') },
                { stroke: '#aab', size: 64,
                  values: (_u, ticks) => ticks.map(v => {
                      const a = Math.abs(v); const sgn = v < 0 ? '-' : '';
                      if (a >= 1e6) return `${sgn}$${(a/1e6).toFixed(1)}M`;
                      if (a >= 1e3) return `${sgn}$${(a/1e3).toFixed(1)}K`;
                      return `${sgn}$${a.toFixed(0)}`;
                  }) },
            ],
            legend: { show: false },
            plugins: [zoomPlugin()],
        }, [xs, ys], el);
    }, 0);
    return true;
}

function byPriceWidget(rows) {
    if (!Array.isArray(rows) || !rows.length) {
        return `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`;
    }
    const maxAbs = Math.max(...rows.map(b => Math.abs(Number(b.net_pnl) || 0)), 1);
    const total  = rows.reduce((a, b) => a + Math.abs(Number(b.net_pnl) || 0), 0) || 1;
    return compareWidget(rows.map(b => {
        const v = Number(b.net_pnl) || 0;
        const pct = (Math.abs(v) / total) * 100;
        return `
            <div class="dash-tv-compare-row">
                <div class="dash-tv-compare-label">${esc(b.key)}</div>
                <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${v >= 0 ? 'pos' : 'neg'}" data-bar-pct="${(Math.abs(v) / maxAbs * 100).toFixed(0)}"></div></div>
                <div class="dash-tv-compare-value ${pnlClass(v)}">${fmtMoney(v)} <span class="muted cmp-pct-muted">${pct.toFixed(2)}%</span></div>
            </div>
        `;
    }));
}

// Render dashboard date labels as MM/DD — the dashboard is always a rolling
// window (30/60/90 days), so YYYY- is dead weight that just gets clipped.
function shortDay(iso) {
    if (!iso || typeof iso !== 'string') return '';
    const parts = iso.slice(0, 10).split('-');
    return parts.length >= 3 ? `${parts[1]}/${parts[2]}` : iso.slice(5, 10);
}

// Shared barChart, not a local uPlot build: the old hand-rolled version
// sized bars as width/n with no cap, so a 1-day window drew one bar 70%
// of the panel wide, spilling left across the y-axis. barChart caps bar
// width at 18% and insets the x range half a slot, so the single bar
// stays inside the axes.
function dailyVolumeChart(elId, daily) {
    if (!daily || !daily.length) return false;
    setTimeout(() => {
        const el = document.getElementById(elId);
        if (!el) return;
        barChart(el,
            daily.map(d => shortDay(d.day)),
            daily.map(d => Number(d.volume) || 0),
            { color: '#39ff14', yKind: 'count', height: 260 });
    }, 0);
    return true;
}

function lineChart(elId, daily, valueKey, color) {
    if (!daily || !daily.length) return false;
    setTimeout(() => {
        const el = document.getElementById(elId);
        if (!el || !window.uPlot) return;
        const xs = daily.map((_, i) => i);
        const ys = daily.map(d => Number(d[valueKey]) || 0);
        const labels = daily.map(d => shortDay(d.day));
        new window.uPlot({
            title: '', width: el.clientWidth || 600, height: 260,
            scales: { x: { time: false,}, y: { auto: true } },
            series: [
                { label: 'day' },
                { label: valueKey, stroke: color, width: 2 },
            ],
            axes: [
                { stroke: '#aab', size: 60, rotate: -45,
                  values: (_u, splits) => splits.map(v => labels[Math.round(v)] || '') },
                { stroke: '#aab', size: 64,
                  values: (_u, ticks) => ticks.map(v => {
                      if (valueKey === 'running_win_rate') return `${(v * 100).toFixed(0)}%`;
                      const a = Math.abs(v); const sgn = v < 0 ? '-' : '';
                      if (a >= 1e6) return `${sgn}$${(a/1e6).toFixed(1)}M`;
                      if (a >= 1e3) return `${sgn}$${(a/1e3).toFixed(1)}K`;
                      return `${sgn}$${a.toFixed(0)}`;
                  }) },
            ],
            legend: { show: false },
            plugins: [zoomPlugin()],
        }, [xs, ys], el);
    }, 0);
    return true;
}

function durationWidget(hold) {
    if (!Array.isArray(hold) || !hold.length) {
        return `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`;
    }
    // by_hold returns buckets keyed by hold-time. Split into intraday (≤1 day) vs multiday.
    let intraday = 0, multiday = 0;
    for (const b of hold) {
        const v = Number(b.net_pnl) || 0;
        const k = String(b.key || '').toLowerCase();
        if (k.includes('day') && !k.includes('<') && !k.includes('intra')) multiday += v;
        else intraday += v;
    }
    const max = Math.max(Math.abs(intraday), Math.abs(multiday), 1);
    return compareWidget([
        `<div class="dash-tv-compare-row">
            <div class="dash-tv-compare-label">${esc(t('view.dashboard.tv.intraday'))}</div>
            <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${intraday >= 0 ? 'pos' : 'neg'}" data-bar-pct="${(Math.abs(intraday)/max*100).toFixed(0)}"></div></div>
            <div class="dash-tv-compare-value ${pnlClass(intraday)}">${fmtMoney(intraday)}</div>
        </div>`,
        `<div class="dash-tv-compare-row">
            <div class="dash-tv-compare-label">${esc(t('view.dashboard.tv.multiday'))}</div>
            <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${multiday >= 0 ? 'pos' : 'neg'}" data-bar-pct="${(Math.abs(multiday)/max*100).toFixed(0)}"></div></div>
            <div class="dash-tv-compare-value ${pnlClass(multiday)}">${fmtMoney(multiday)}</div>
        </div>`,
    ]);
}

function winPctSummary(s) {
    const rate = Number(s.win_rate) || 0;
    const pct = (rate * 100).toFixed(1);
    return `
        <div class="hero-stat">
            <div class="hero-num hero-num-cyan">${pct}%</div>
            <div class="muted small">${esc(t('view.dashboard.tv.win_rate_subtitle', {wins: s.win_count, total: s.win_count + s.loss_count}))}</div>
        </div>
    `;
}

// ===========================================================================
// Widget catalog + layout persistence
// ===========================================================================

// Each entry: { id, titleKey, spans2, html(data), mount(data) }
// `spans2` makes the panel span 2 grid columns (used by line/area charts).
// `html` returns the inner contents (the chart-panel wrapper, drag handle,
// and delete button are added by renderLayoutPanels around it).
// `mount` runs after the panel is in the DOM — for uPlot init etc.
const WIDGETS = [
    { id: 'cumulative_pnl', titleKey: 'view.dashboard.tv.cumulative_pnl', spans2: true,
        html: () => `<div id="equity-chart"></div>`,
        mount: (data, mount) => { const el = mount.querySelector('#equity-chart'); if (el) equityChart(el, data.equity); } },
    { id: 'win_loss', titleKey: 'view.dashboard.tv.win_loss',
        html: (d) => winLossCount(d.summary) },
    { id: 'hold_win_loss', titleKey: 'view.dashboard.tv.hold_win_loss',
        html: (d) => holdTimeWinLoss(d.summary) },
    { id: 'avg_win_loss', titleKey: 'view.dashboard.tv.avg_win_loss',
        html: (d) => avgWinLoss(d.summary) },
    { id: 'largest_gain_loss', titleKey: 'view.dashboard.tv.largest_gain_loss',
        html: (d) => largestGainLoss(d.summary) },
    { id: 'win_pct', titleKey: 'view.dashboard.tv.win_pct',
        html: (d) => `<div id="dash-win-pct-chart" class="chart-h-260">${d.daily && d.daily.length ? '' : `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`}</div>`,
        mount: (d) => { if (d.daily && d.daily.length) lineChart('dash-win-pct-chart', d.daily, 'running_win_rate', '#39ff14'); } },
    { id: 'perf_dow', titleKey: 'view.dashboard.tv.perf_dow',
        html: (d) => dayOfWeekWidget(d.dow) },
    { id: 'mfe_mae', titleKey: 'view.dashboard.tv.mfe_mae',
        html: (d) => avgMfeMae(d.summary) },
    { id: 'perf_duration', titleKey: 'view.dashboard.tv.perf_duration',
        html: (d) => durationWidget(d.hold) },
    { id: 'perf_hour', titleKey: 'view.dashboard.tv.perf_hour',
        html: (d) => hourOfDayWidget(d.hour) },
    { id: 'profit_factor', titleKey: 'view.dashboard.tv.profit_factor',
        html: (d) => profitFactorGauge(d.summary.profit_factor) },
    { id: 'total_fees', titleKey: 'view.dashboard.tv.total_fees',
        html: (d) => `<div class="hero-stat hero-stat-band"><div class="hero-num-md hero-num-warn">${esc(fmtMoney(d.summary.fees))}</div></div>` },
    { id: 'cumulative_drawdown', titleKey: 'view.dashboard.tv.cumulative_drawdown', spans2: true,
        // Drives off the equity curve (which has per-day drawdown) — the
        // MaxDrawdown summary in `d.dd` carries only peak/trough values, no
        // series, so it can't feed a line chart.
        html: (d) => `<div id="dash-drawdown-chart" class="chart-h-260">${d.equity && d.equity.length ? '' : `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`}</div>`,
        mount: (d) => { if (d.equity && d.equity.length) drawdownChart('dash-drawdown-chart', d.equity); } },
    { id: 'perf_price', titleKey: 'view.dashboard.tv.perf_price',
        html: (d) => byPriceWidget(d.byPrice) },
    { id: 'daily_volume', titleKey: 'view.dashboard.tv.daily_volume', spans2: true,
        html: (d) => `<div id="dash-daily-volume-chart" class="chart-h-260">${d.daily && d.daily.length ? '' : `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`}</div>`,
        mount: (d) => { if (d.daily && d.daily.length) dailyVolumeChart('dash-daily-volume-chart', d.daily); } },
    { id: 'avg_trade_pnl', titleKey: 'view.dashboard.tv.avg_trade_pnl', spans2: true,
        html: (d) => `<div id="dash-avg-pnl-chart" class="chart-h-260">${d.daily && d.daily.length ? '' : `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`}</div>`,
        mount: (d) => { if (d.daily && d.daily.length) lineChart('dash-avg-pnl-chart', d.daily, 'running_avg_pnl', '#00e5ff'); } },
    { id: 'tag_breakdown', titleKey: 'view.dashboard.tv.tag_breakdown',
        html: (d) => tagBreakdownWidget(d.tags) },
    { id: 'perf_day_type', titleKey: 'view.dashboard.tv.perf_day_type',
        html: (d) => perfDayTypeWidget(d.cal) },
    { id: 'perf_month', titleKey: 'view.dashboard.tv.perf_month',
        html: (d) => perfByBucketsWidget(d.byMonth, { preserveOrder: true }) },
    { id: 'perf_symbol', titleKey: 'view.dashboard.tv.perf_symbol',
        html: (d) => perfByBucketsWidget(d.bySymbol, { limit: 10 }) },
    // Per-broker P&L breakdown — the brokers-side analog of perf_symbol.
    // Mapped to the {key, net_pnl} shape perfByBucketsWidget expects.
    { id: 'perf_broker', titleKey: 'view.dashboard.tv.perf_broker',
        html: (d) => perfByBucketsWidget(
            (d.byBroker || []).map(b => ({ key: b.label, net_pnl: b.net_pnl })),
            { limit: 10, preserveOrder: true },
        ) },
    { id: 'total_trades', titleKey: 'view.dashboard.tv.total_trades',
        html: (d) => heroStat(d.summary.trade_count, 'hero-num-cyan') },
    { id: 'avg_daily_volume', titleKey: 'view.dashboard.tv.avg_daily_volume',
        html: (d) => heroStat(fmtMoney(d.summary.avg_daily_volume), 'hero-num-cyan',
                              `${d.summary.trading_days || 0} ${t('view.dashboard.tv.trading_days_suffix')}`) },
    { id: 'avg_position_mae', titleKey: 'view.dashboard.tv.avg_position_mae',
        html: (d) => heroStat(fmtMoney(d.summary.avg_mae), 'hero-num-warn') },
    { id: 'avg_position_mfe', titleKey: 'view.dashboard.tv.avg_position_mfe',
        html: (d) => heroStat(fmtMoney(d.summary.avg_mfe), 'hero-num-cyan') },
    { id: 'max_consec_wins', titleKey: 'view.dashboard.tv.max_consec_wins_widget',
        html: (d) => heroStat(d.summary.max_consec_wins, 'hero-num-cyan') },
    { id: 'max_consec_losses', titleKey: 'view.dashboard.tv.max_consec_losses_widget',
        html: (d) => heroStat(d.summary.max_consec_losses, 'hero-num-warn') },
    { id: 'perf_opening_gap', titleKey: 'view.dashboard.tv.perf_opening_gap',
        html: (d) => perfByBucketsWidget(d.byOpeningGap, { preserveOrder: true }) },
    { id: 'perf_instrument_volume', titleKey: 'view.dashboard.tv.perf_instrument_volume',
        html: (d) => perfByBucketsWidget(d.byInstrumentVolume, { preserveOrder: true }) },
    { id: 'perf_movement', titleKey: 'view.dashboard.tv.perf_movement',
        html: (d) => perfByBucketsWidget(d.byMovement, { preserveOrder: true }) },
    { id: 'perf_r_bucket', titleKey: 'view.dashboard.tv.perf_r_bucket',
        html: (d) => perfByBucketsWidget(d.byRBucket, { preserveOrder: true }) },
    { id: 'perf_duration_coarse', titleKey: 'view.dashboard.tv.perf_duration_coarse',
        html: (d) => perfByBucketsWidget(d.byDurationCoarse, { preserveOrder: true }) },
    { id: 'open_trades', titleKey: 'view.dashboard.tv.open_trades', spans2: true,
        html: (d) => openTradesWidget(d.openTrades) },
    // Budget status — this month's income/expense/savings rate + over-budget flag.
    { id: 'budget_status', titleKey: 'view.dashboard.tv.budget_status',
        html: () => `<div id="dash-budget-status" class="dash-budget-status"><div class="muted small"><div class="tv-spinner-wrap"><div class="tv-spinner"></div></div></div></div>`,
        mount: async () => {
            const el = document.getElementById('dash-budget-status');
            if (!el) return;
            try {
                const s = await api.budgetSnapshot();
                const netCls = (+s.net || 0) >= 0 ? 'tw-refund' : 'tw-owed';
                const rateCls = (+s.savings_rate || 0) >= 0 ? 'tw-refund' : 'tw-owed';
                const overBlock = s.over_budget_categories > 0
                    ? `<span class="tw-owed">${esc(t('view.dashboard.tv.budget_widget.over', { n: s.over_budget_categories }))}</span>`
                    : `<span class="tw-refund">${esc(t('view.dashboard.tv.budget_widget.on_track'))}</span>`;
                el.innerHTML = `
                    <div class="dash-budget-row">
                        <span class="muted small">${esc(t('view.dashboard.tv.budget_widget.month', { year: s.year, month: String(s.month).padStart(2, '0') }))}</span>
                        ${overBlock}
                    </div>
                    <div class="dash-budget-grid">
                        <div><span>${esc(t('view.dashboard.tv.budget_widget.income'))}</span><strong>${esc(fmtMoney(+s.income || 0))}</strong></div>
                        <div><span>${esc(t('view.dashboard.tv.budget_widget.expense'))}</span><strong>${esc(fmtMoney(+s.expense || 0))}</strong></div>
                        <div><span>${esc(t('view.dashboard.tv.budget_widget.net'))}</span><strong class="${netCls}">${esc(fmtMoney(+s.net || 0))}</strong></div>
                        <div><span>${esc(t('view.dashboard.tv.budget_widget.savings_rate'))}</span><strong class="${rateCls}">${(+s.savings_rate || 0).toFixed(1)}%</strong></div>
                    </div>
                    <a href="#budget" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.budget_widget.open'))}</a>
                `;
            } catch (_) {
                el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.budget_widget.start'))}</div>
                    <a href="#budget" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.budget_widget.open'))}</a>`;
            }
        } },
    // Trades — recent trades list with a link to the full table.
    { id: 'recent_trades', titleKey: 'view.dashboard.tv.recent_trades',
        html: () => `<div id="dash-recent-trades" class="dash-manage-list"><div class="muted small"><div class="tv-spinner-wrap"><div class="tv-spinner"></div></div></div></div>`,
        mount: async () => {
            const el = document.getElementById('dash-recent-trades');
            if (!el) return;
            try {
                // GET /trades requires an account_id (axum query reject is
                // hard 400 if missing). Read the currently-active account
                // from the global app state — if none picked yet, surface
                // the prompt instead of failing the fetch.
                const { state } = await import('../app.js');
                const accountId = state?.accountId;
                if (!accountId) {
                    el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.recent_trades_no_account'))}</div>
                        <a href="#accounts" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.recent_trades_pick_account'))}</a>`;
                    return;
                }
                const trades = await api.trades(accountId, { limit: 8 });
                if (!trades || !trades.length) {
                    el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.recent_trades_empty'))}</div>
                        <a href="#trades" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.recent_trades_open'))}</a>`;
                    return;
                }
                el.innerHTML = `<ul class="dash-manage-rows">${
                    trades.slice(0, 6).map(tr => {
                        const pnl = +(tr.net_pnl || 0);
                        const cls = pnl > 0 ? 'tw-refund' : pnl < 0 ? 'tw-owed' : 'muted';
                        return `<li>
                            <span class="dash-manage-spacer"></span>
                            <strong>${esc(tr.symbol || '—')}</strong>
                            <span class="${cls} small">${esc(fmtMoney(pnl))}</span>
                        </li>`;
                    }).join('')
                }</ul>
                    <div class="dash-manage-meta muted small">${esc(t('view.dashboard.tv.recent_trades_count', { n: trades.length }))}</div>
                    <a href="#trades" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.recent_trades_open'))}</a>`;
            } catch (e) {
                el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.recent_trades_failed', { err: e.message || String(e) }))}</div>
                    <a href="#trades" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.recent_trades_open'))}</a>`;
            }
        } },
    // Expense dashboard summary tile.
    { id: 'expense_dashboard_card', titleKey: 'view.dashboard.tv.expense_dashboard',
        html: () => `<div id="dash-expense-dashboard" class="dash-manage-list"><div class="muted small"><div class="tv-spinner-wrap"><div class="tv-spinner"></div></div></div></div>`,
        mount: async () => {
            const el = document.getElementById('dash-expense-dashboard');
            if (!el) return;
            try {
                const year = new Date().getFullYear();
                const bundle = await api.expenseDashboardBundle(year, null);
                const total = +(bundle?.totals?.total || bundle?.totals?.gross || 0);
                el.innerHTML = `<div class="dash-tax-amount">${esc(fmtMoney(total))}</div>
                    <div class="dash-manage-meta muted small">${esc(t('view.dashboard.tv.expense_dashboard_window', { year }))}</div>
                    <a href="#expense-dashboard" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.expense_dashboard_open'))}</a>`;
            } catch (e) {
                el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.expense_dashboard_failed', { err: e.message || String(e) }))}</div>
                    <a href="#expense-dashboard" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.expense_dashboard_open'))}</a>`;
            }
        } },
    // Expense calendar — current-month total + opener.
    { id: 'expense_calendar_card', titleKey: 'view.dashboard.tv.expense_calendar',
        html: () => `<div id="dash-expense-calendar" class="dash-manage-list"><div class="muted small"><div class="tv-spinner-wrap"><div class="tv-spinner"></div></div></div></div>`,
        mount: async () => {
            const el = document.getElementById('dash-expense-calendar');
            if (!el) return;
            try {
                const now = new Date();
                const year = now.getFullYear();
                const month = now.getMonth() + 1;
                const cal = await api.receiptsMonthCalendar(year, month, null);
                const sum = Array.isArray(cal?.days)
                    ? cal.days.reduce((acc, d) => acc + (+d.total || 0), 0)
                    : 0;
                el.innerHTML = `<div class="dash-tax-amount">${esc(fmtMoney(sum))}</div>
                    <div class="dash-manage-meta muted small">${esc(t('view.dashboard.tv.expense_calendar_window', { year, month }))}</div>
                    <a href="#expense-calendar" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.expense_calendar_open'))}</a>`;
            } catch (e) {
                el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.expense_calendar_failed', { err: e.message || String(e) }))}</div>
                    <a href="#expense-calendar" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.expense_calendar_open'))}</a>`;
            }
        } },
    // Categorize queue size — sum of receipts pending item-level
    // category assignment.
    { id: 'categorize_card', titleKey: 'view.dashboard.tv.categorize',
        html: () => `<div id="dash-categorize" class="dash-manage-list"><div class="muted small"><div class="tv-spinner-wrap"><div class="tv-spinner"></div></div></div></div>`,
        mount: async () => {
            const el = document.getElementById('dash-categorize');
            if (!el) return;
            try {
                // The categorize page groups receipts by merchant — show the
                // group count as the headline so the user sees roughly how
                // much triage work is queued.
                const groups = await api.receiptsByMerchant({ uncategorized_only: true, limit: 1 }).catch(() => null);
                const count = Array.isArray(groups?.merchants)
                    ? groups.merchants.length
                    : (groups?.total || 0);
                el.innerHTML = `<div class="dash-tax-amount">${esc(String(count))}</div>
                    <div class="dash-manage-meta muted small">${esc(t('view.dashboard.tv.categorize_label'))}</div>
                    <a href="#categorize" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.categorize_open'))}</a>`;
            } catch (e) {
                el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.categorize_failed', { err: e.message || String(e) }))}</div>
                    <a href="#categorize" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.categorize_open'))}</a>`;
            }
        } },
    // Note templates — count + link.
    { id: 'note_templates_card', titleKey: 'view.dashboard.tv.note_templates',
        html: () => `<div id="dash-note-templates" class="dash-manage-list"><div class="muted small"><div class="tv-spinner-wrap"><div class="tv-spinner"></div></div></div></div>`,
        mount: async () => {
            const el = document.getElementById('dash-note-templates');
            if (!el) return;
            try {
                const tmpls = await api.noteTemplates().catch(() => []);
                el.innerHTML = `<div class="dash-tax-amount">${esc(String((tmpls || []).length))}</div>
                    <div class="dash-manage-meta muted small">${esc(t('view.dashboard.tv.note_templates_label'))}</div>
                    <a href="#note-templates" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.note_templates_open'))}</a>`;
            } catch (e) {
                el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.note_templates_failed', { err: e.message || String(e) }))}</div>
                    <a href="#note-templates" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.note_templates_open'))}</a>`;
            }
        } },
    // Side-by-side broker comparison (net P&L, win rate, profit factor,
    // expectancy) condensed into a sparkline-style tile. Full view at
    // `#broker-compare`.
    { id: 'broker_compare_card', titleKey: 'view.dashboard.tv.broker_compare',
        html: () => `<div id="dash-broker-compare" class="dash-manage-list"><div class="muted small"><div class="tv-spinner-wrap"><div class="tv-spinner"></div></div></div></div>`,
        mount: async () => {
            const el = document.getElementById('dash-broker-compare');
            if (!el) return;
            try {
                const brokers = await api.brokersList();
                if (!brokers.length) {
                    el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.broker_compare_empty'))}</div>
                        <a href="#broker-compare" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.broker_compare_open'))}</a>`;
                    return;
                }
                const summaries = await Promise.all(
                    brokers.slice(0, 6).map(b =>
                        api.summaryRaw({ days: 90, broker_id: b.id }).catch(() => null)),
                );
                const rows = brokers.slice(0, 6).map((b, i) => {
                    const s = summaries[i];
                    const pnl = +(s?.net_pnl || 0);
                    const cls = pnl > 0 ? 'tw-refund' : pnl < 0 ? 'tw-owed' : 'muted';
                    return `<li>
                        ${b.is_default ? '<span class="brk-default">★</span>' : '<span class="dash-manage-spacer"></span>'}
                        <strong>${esc(b.display_name)}</strong>
                        <span class="${cls} small">${esc(fmtMoney(pnl))}</span>
                    </li>`;
                }).join('');
                el.innerHTML = `<ul class="dash-manage-rows">${rows}</ul>
                    <div class="dash-manage-meta muted small">${esc(t('view.dashboard.tv.broker_compare_window'))}</div>
                    <a href="#broker-compare" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.broker_compare_open'))}</a>`;
            } catch (e) {
                el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.broker_compare_failed', { err: e.message || String(e) }))}</div>
                    <a href="#broker-compare" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.broker_compare_open'))}</a>`;
            }
        } },
    // Side-by-side business comparison (expense totals).
    { id: 'business_compare_card', titleKey: 'view.dashboard.tv.business_compare',
        html: () => `<div id="dash-business-compare" class="dash-manage-list"><div class="muted small"><div class="tv-spinner-wrap"><div class="tv-spinner"></div></div></div></div>`,
        mount: async () => {
            const el = document.getElementById('dash-business-compare');
            if (!el) return;
            try {
                const businesses = await api.businessesList();
                if (!businesses.length) {
                    el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.business_compare_empty'))}</div>
                        <a href="#business-compare" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.business_compare_open'))}</a>`;
                    return;
                }
                const year = new Date().getFullYear();
                const bundles = await Promise.all(
                    businesses.slice(0, 6).map(b =>
                        api.expenseDashboardBundle(year, b.id).catch(() => null)),
                );
                const rows = businesses.slice(0, 6).map((b, i) => {
                    const total = +(bundles[i]?.totals?.gross || bundles[i]?.totals?.total || 0);
                    return `<li>
                        ${b.is_default ? '<span class="brk-default">★</span>' : '<span class="dash-manage-spacer"></span>'}
                        <strong>${esc(b.name)}</strong>
                        <span class="muted small">${esc(fmtMoney(total))}</span>
                    </li>`;
                }).join('');
                el.innerHTML = `<ul class="dash-manage-rows">${rows}</ul>
                    <div class="dash-manage-meta muted small">${esc(t('view.dashboard.tv.business_compare_window', { year }))}</div>
                    <a href="#business-compare" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.business_compare_open'))}</a>`;
            } catch (e) {
                el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.business_compare_failed', { err: e.message || String(e) }))}</div>
                    <a href="#business-compare" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.business_compare_open'))}</a>`;
            }
        } },
    // Brokers — list + quick actions. Mirrors the budget/tax tile
    // pattern: self-fetching tile that doubles as a dashboard widget
    // AND as the full-page management view at `#brokers`.
    { id: 'manage_brokers', titleKey: 'view.dashboard.tv.manage_brokers',
        html: () => `<div id="dash-manage-brokers" class="dash-manage-list"><div class="muted small"><div class="tv-spinner-wrap"><div class="tv-spinner"></div></div></div></div>`,
        mount: async () => {
            const el = document.getElementById('dash-manage-brokers');
            if (!el) return;
            try {
                const rows = await api.brokersList();
                if (!rows.length) {
                    el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.manage_brokers_empty'))}</div>
                        <a href="#brokers" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.manage_brokers_open'))}</a>`;
                    return;
                }
                el.innerHTML = `
                    <ul class="dash-manage-rows">
                        ${rows.slice(0, 6).map(b => `
                            <li>
                                ${b.is_default ? '<span class="brk-default">★</span>' : '<span class="dash-manage-spacer"></span>'}
                                <strong>${esc(b.display_name)}</strong>
                                <code class="muted small">${esc(b.slug)}</code>
                            </li>`).join('')}
                    </ul>
                    <div class="dash-manage-meta muted small">${esc(t('view.dashboard.tv.manage_brokers_count', { n: rows.length }))}</div>
                    <a href="#brokers" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.manage_brokers_open'))}</a>
                `;
            } catch (e) {
                el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.manage_brokers_failed', { err: e.message || String(e) }))}</div>
                    <a href="#brokers" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.manage_brokers_open'))}</a>`;
            }
        } },
    // Businesses — Schedule C entities list + quick actions.
    { id: 'manage_businesses', titleKey: 'view.dashboard.tv.manage_businesses',
        html: () => `<div id="dash-manage-businesses" class="dash-manage-list"><div class="muted small"><div class="tv-spinner-wrap"><div class="tv-spinner"></div></div></div></div>`,
        mount: async () => {
            const el = document.getElementById('dash-manage-businesses');
            if (!el) return;
            try {
                const rows = await api.businessesList();
                if (!rows.length) {
                    el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.manage_businesses_empty'))}</div>
                        <a href="#businesses" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.manage_businesses_open'))}</a>`;
                    return;
                }
                el.innerHTML = `
                    <ul class="dash-manage-rows">
                        ${rows.slice(0, 6).map(b => `
                            <li>
                                ${b.is_default ? '<span class="brk-default">★</span>' : '<span class="dash-manage-spacer"></span>'}
                                <strong>${esc(b.name)}</strong>
                                <code class="muted small">${esc(b.entity_type || 'sole_prop')}</code>
                            </li>`).join('')}
                    </ul>
                    <div class="dash-manage-meta muted small">${esc(t('view.dashboard.tv.manage_businesses_count', { n: rows.length }))}</div>
                    <a href="#businesses" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.manage_businesses_open'))}</a>
                `;
            } catch (e) {
                el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.manage_businesses_failed', { err: e.message || String(e) }))}</div>
                    <a href="#businesses" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.manage_businesses_open'))}</a>`;
            }
        } },
    // Tax wizard status. Self-fetching — doesn't read from `d`.
    { id: 'tax_filing_status', titleKey: 'view.dashboard.tv.tax_filing_status',
        html: () => `<div id="dash-tax-filing" class="dash-tax-filing"><div class="muted small"><div class="tv-spinner-wrap"><div class="tv-spinner"></div></div></div></div>`,
        mount: async () => {
            const el = document.getElementById('dash-tax-filing');
            if (!el) return;
            const year = new Date().getFullYear() - 1;
            try {
                const r = await api.taxReturn(year);
                const refund = +(r.result.refund_due || 0);
                const owed   = +(r.result.tax_owed   || 0);
                const label  = refund > 0
                    ? `<strong class="tw-refund">${esc(fmtMoney(refund))}</strong>`
                    : `<strong class="tw-owed">${esc(fmtMoney(owed))}</strong>`;
                const verdict = refund > 0
                    ? esc(t('view.dashboard.tv.tax_widget.refund'))
                    : esc(t('view.dashboard.tv.tax_widget.owed'));
                el.innerHTML = `
                    <div class="dash-tax-row">
                        <span class="muted small">${esc(t('view.dashboard.tv.tax_widget.year', { year }))}</span>
                        <span class="muted small">${esc(r.status || 'personal')}</span>
                    </div>
                    <div class="dash-tax-amount">${verdict}: ${label}</div>
                    <div class="dash-tax-row">
                        <span class="muted small">AGI: ${esc(fmtMoney(+r.result.agi || 0))}</span>
                        <span class="muted small">Tax: ${esc(fmtMoney(+r.result.tax_after_credits || 0))}</span>
                    </div>
                    <a href="#file-taxes?year=${year}" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.tax_widget.open'))}</a>
                `;
            } catch (_) {
                el.innerHTML = `<div class="muted small">${esc(t('view.dashboard.tv.tax_widget.start'))}</div>
                    <a href="#file-taxes" class="btn btn-secondary btn-compact">${esc(t('view.dashboard.tv.tax_widget.open'))}</a>`;
            }
        } },
];
export const WIDGETS_BY_ID = new Map(WIDGETS.map(w => [w.id, w]));
const DEFAULT_LAYOUT = WIDGETS.map(w => w.id);

function renderLayoutPanels(layout, data) {
    return layout
        .map(id => WIDGETS_BY_ID.get(id))
        .filter(Boolean)
        .map(w => `
            <div class="chart-panel${w.spans2 ? ' dash-tv-span-2' : ''}" data-widget-id="${w.id}">
                <span class="dash-tv-drag-handle" title="drag to reorder" data-drag-handle>⠿</span>
                <span class="dash-tv-pin-btn" title="pin to a saved board" data-pin-widget="${w.id}">📌</span>
                <span class="dash-tv-del-btn" title="remove from layout" data-del-widget="${w.id}">✕</span>
                <h2 data-i18n="${w.titleKey}">${esc(t(w.titleKey))}</h2>
                ${w.html(data)}
            </div>
        `).join('');
}

/**
 * Wire the dashboard grid into the global Trello-style drag engine.
 * Pointer-based — no HTML5 dragstart/dragover; bypasses Tauri and uPlot
 * canvas interception entirely. Persists the new order to the user's
 * server-side dashboard_layout setting via `persistFn`.
 */
function attachLayoutHandlers(mount, layout, _data, persistFn) {
    const grid = mount.querySelector('#dash-tv-grid');
    if (!grid) return;
    resetDragReorder(grid);  // allow re-wire after re-render
    initDragReorder(grid, '.chart-panel[data-widget-id]', null, {
        direction: 'vertical',
        handleSelector: '[data-drag-handle], .chart-panel > h2',
        getKey: (el) => el.dataset.widgetId,
        persist: (newOrder) => persistFn(newOrder),
        toastMessage: t('view.dashboard.tv.reordered'),
    });

    // Delete button — optimistic remove then persist.
    grid.querySelectorAll('[data-del-widget]').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            const panel = btn.closest('.chart-panel[data-widget-id]');
            if (panel) panel.remove();
            const next = [...grid.querySelectorAll('.chart-panel[data-widget-id]')]
                .map(el => el.dataset.widgetId);
            persistFn(next).catch((err) => console.warn('layout persist failed', err));
        });
    });

    // Pin button — adds this graph widget as a tile on the user's active
    // saved board (see views/dashboards.js). The graph keeps rendering
    // here too; pin is additive, not a move.
    grid.querySelectorAll('[data-pin-widget]').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            const id = btn.dataset.pinWidget;
            const widget = WIDGETS_BY_ID.get(id);
            if (!widget) return;
            let s = dashStore.loadState();
            const active = dashStore.getActiveDashboard(s);
            if (!active) {
                showToast(t('toast.no_active_dashboard'), { level: 'warning' });
                return;
            }
            s = dashStore.addGraphTile(s, active.id, id);
            dashStore.saveState(s);
            btn.textContent = '✓';
            setTimeout(() => { btn.textContent = '📌'; }, 1200);
            showToast(
                t('toast.graph_pinned', {
                    graph: t(widget.titleKey),
                    dashboard: active.name || active.id,
                }),
                { level: 'success' });
            window.dispatchEvent(new CustomEvent('tv:dashboards-changed'));
        });
    });
}

let cachedSettings = null;
// Per-broker layout key — the server-side `dashboard_layout` setting
// remains the default/aggregated. A localStorage override stores
// per-broker tweaks so each broker can have a different widget order.
function brokerLayoutKey() {
    try {
        const bid = globalThis.__tvActiveBroker?.();
        return bid ? `dashboard_layout_broker_${bid}` : null;
    } catch { return null; }
}
/**
 * Merge any widget ID present in DEFAULT_LAYOUT but missing from
 * `saved`. New widgets (manage_brokers, manage_businesses,
 * broker_compare, business_compare, etc.) ship enabled by default —
 * users with a saved layout from before the addition would otherwise
 * never see them without manually opening the "+ Add widget" picker.
 */
function mergeMissingDefaults(saved) {
    const have = new Set(saved);
    const additions = DEFAULT_LAYOUT.filter(id => !have.has(id));
    return additions.length ? [...saved, ...additions] : saved;
}

async function loadLayout() {
    const bkey = brokerLayoutKey();
    if (bkey) {
        try {
            const raw = localStorage.getItem(bkey);
            if (raw) {
                const arr = JSON.parse(raw);
                if (Array.isArray(arr)) {
                    const known = arr.filter((id) => WIDGETS_BY_ID.has(id));
                    if (known.length) return mergeMissingDefaults(known);
                }
            }
        } catch {}
    }
    try {
        cachedSettings = await api.settings();
        const stored = cachedSettings?.dashboard_layout;
        if (stored && Array.isArray(stored.order)) {
            // Drop unknown IDs (e.g. widgets removed between versions).
            const known = stored.order.filter(id => WIDGETS_BY_ID.has(id));
            return known.length ? mergeMissingDefaults(known) : DEFAULT_LAYOUT.slice();
        }
    } catch (_) { /* fall through to default */ }
    return DEFAULT_LAYOUT.slice();
}
async function saveLayout(order) {
    // Per-broker overrides stay in localStorage; the aggregated layout
    // is persisted server-side.
    const bkey = brokerLayoutKey();
    if (bkey) {
        try { localStorage.setItem(bkey, JSON.stringify(order)); } catch {}
        return;
    }
    if (!cachedSettings) {
        try { cachedSettings = await api.settings(); }
        catch (e) { console.warn('layout save: fetch settings failed', e); return; }
    }
    cachedSettings.dashboard_layout = { order };
    await api.updateSettings(cachedSettings);
}

export async function renderDashboard(mount, state) {
    const tok = currentViewToken();
    const period = getPeriod();
    // Calendar-aware periods take precedence over rolling 30/60/90 — that's
    // how Tradervue's dashboard period strip works.
    const interval = period ? periodToDays(period) : getInterval();

    if (!state.accountId) {
        mount.innerHTML = `
            <h1 data-i18n="view.dashboard.h1.dashboard" class="view-title">// DASHBOARD</h1>
            <div id="world-markets-mount"></div>
            <p data-i18n="view.dashboard.hint.no_account_yet_add_one_via_accounts_then_import_or" class="boot">No account yet. Add one via Accounts, then import or add trades.</p>
        `;
        const wm = mount.querySelector('#world-markets-mount');
        if (wm) renderWorldMarkets(wm);
        return;
    }

    const failedFetches = [];
    const { data, layout } = await loadAnalyticsBundle(state.accountId, interval, failedFetches);
    if (!viewIsCurrent(tok)) return;

    mount.innerHTML = `
        <div class="dash-tv-header">
            <h1 class="view-title"><span data-i18n="view.dashboard.h1.dashboard_2">// DASHBOARD</span></h1>
            <button type="button" class="btn btn-secondary btn-compact" id="dashboard-refresh-btn"
                    data-i18n="view.dashboard.btn.refresh"
                    data-tip="view.dashboard.tip.refresh"
                    data-shortcut="dashboard_refresh">⟳ Refresh</button>
            <div class="dash-tv-layout-controls">
                <div class="dash-tv-add-wrap">
                    <button type="button" id="dash-add-widget" class="btn btn-secondary btn-compact">⊞ ${esc(t('view.dashboard.tv.add_widget'))}</button>
                    <div class="dash-tv-add-menu" id="dash-add-menu" hidden role="menu" aria-label="${esc(t('view.dashboard.tv.add_widget'))}"></div>
                </div>
                <button type="button" id="dash-reset-layout" class="btn btn-secondary btn-compact">⟲ ${esc(t('view.dashboard.tv.reset_layout'))}</button>
            </div>
            <div class="dash-tv-toggle" role="tablist">
                ${VALID_INTERVALS.map(d => `<button type="button" data-interval="${d}" class="${!period && d === interval ? 'active' : ''}">${d} Days</button>`).join('')}
            </div>
            <div class="dash-tv-period-bar" role="tablist">
                ${PERIOD_KEYS.map(k => `<button type="button" data-period="${k}" class="${k === period ? 'active' : ''}">${esc(t('view.dashboard.period.' + k))}</button>`).join('')}
            </div>
        </div>
        <div class="dash-tv-range">${esc(rangeLabel(interval))}</div>
        ${failedFetches.length > 0 ? `<div class="dash-tv-banner warn" role="alert">${esc(t('view.dashboard.banner.partial_data', { names: failedFetches.map(f => f.name).join(', ') }))}</div>` : ''}
        <div class="dash-tv-day-strip">${dayStrip(data.cal, interval)}</div>

        <div class="dash-tv-grid" id="dash-tv-grid">
            ${renderLayoutPanels(layout, data)}
        </div>

        <div id="world-markets-mount" class="world-mount"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.dashboard.h2.risk_gate_today">🛡 Risk Gate · today</h2>
            <div id="dash-rg" class="muted small"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.dashboard.h2.discipline_score_last_7_days">📐 Discipline score · last 7 days</h2>
            <div id="dash-disc"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
        </div>
    `;

    const refreshBtn = mount.querySelector('#dashboard-refresh-btn');
    if (refreshBtn) refreshBtn.addEventListener('click', () =>
        window.dispatchEvent(new HashChangeEvent('hashchange')));

    mount.querySelectorAll('.dash-tv-toggle button[data-interval]').forEach(btn => {
        btn.addEventListener('click', () => {
            const d = Number(btn.dataset.interval);
            setInterval(d);
            setPeriod(null);  // clear calendar period when user picks rolling-days
            renderDashboard(mount, state);
        });
    });
    mount.querySelectorAll('.dash-tv-period-bar button[data-period]').forEach(btn => {
        btn.addEventListener('click', () => {
            setPeriod(btn.dataset.period);
            renderDashboard(mount, state);
        });
    });

    // Apply data-bar-pct widths via rAF — Tauri release WebKit strips
    // inline style="width:X%" from innerHTML, so widths land via JS.
    applyBarWidths(mount);

    // Start the day strip at its right end (today). rAF so release WebKit
    // has resolved layout and scrollWidth is real.
    const strip = mount.querySelector('.dash-tv-day-strip');
    if (strip) requestAnimationFrame(() => { strip.scrollLeft = strip.scrollWidth; });

    // Mount widgets that need post-DOM init (uPlot, etc). Walk the actual
    // layout — never assume a widget is present.
    for (const id of layout) {
        const w = WIDGETS_BY_ID.get(id);
        if (w && typeof w.mount === 'function') w.mount(data, mount);
    }

    const wmEl  = mount.querySelector('#world-markets-mount');
    const rgEl  = mount.querySelector('#dash-rg');
    const discEl = mount.querySelector('#dash-disc');
    if (wmEl)   renderWorldMarkets(wmEl);
    if (rgEl)   loadRiskGateBadge(rgEl);
    if (discEl) loadDisciplineScore(discEl, state.accountId);

    // persist = write to server in background. We rely on the optimistic
    // DOM reorder in attachLayoutHandlers for the visible change so the
    // user doesn't experience a flash of re-rendered widgets on every drag.
    const persist = async (newLayout) => { await saveLayout(newLayout); };
    // Full re-render is only needed when widgets are *added* (the DOM
    // doesn't have their nodes yet) — Add-Widget calls this.
    const persistAndRerender = async (newLayout) => {
        await saveLayout(newLayout);
        renderDashboard(mount, state);
    };
    attachLayoutHandlers(mount, layout, data, persist);

    const resetBtn = mount.querySelector('#dash-reset-layout');
    if (resetBtn) resetBtn.addEventListener('click', () => persistAndRerender(DEFAULT_LAYOUT.slice()));

    const addBtn = mount.querySelector('#dash-add-widget');
    const addMenu = mount.querySelector('#dash-add-menu');
    if (addBtn && addMenu) {
        // Build (or rebuild) the menu body as a single checklist of every
        // widget — checked = currently in layout, unchecked = available to
        // add. The previous design only listed "missing" widgets, which on
        // a fresh dashboard (DEFAULT_LAYOUT = every widget) made the menu
        // permanently render the "all widgets already shown" empty state
        // and look like the button was broken.
        const renderMenu = () => {
            const inLayout = new Set(layout);
            addMenu.innerHTML = WIDGETS.map(w => {
                const on = inLayout.has(w.id);
                return `<button type="button" class="dash-tv-add-item${on ? ' on' : ''}"
                                data-toggle-widget="${w.id}"
                                role="menuitemcheckbox" aria-checked="${on}">
                            <span class="dash-tv-add-check">${on ? '✓' : ''}</span>${esc(t(w.titleKey))}
                        </button>`;
            }).join('');
        };
        addBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            renderMenu();
            addMenu.hidden = !addMenu.hidden;
        });
        mount.addEventListener('click', (e) => {
            if (addMenu.hidden) return;
            if (addMenu.contains(e.target) || e.target === addBtn) return;
            addMenu.hidden = true;
        });
        addMenu.addEventListener('click', async (e) => {
            const btn = e.target.closest('[data-toggle-widget]');
            if (!btn) return;
            e.stopPropagation();
            const id = btn.dataset.toggleWidget;
            if (!WIDGETS_BY_ID.has(id)) return;
            // Toggle: present → remove; absent → append. Re-renders the
            // dashboard so the panel grid + the menu's check marks
            // reflect the new layout in one pass.
            const next = layout.includes(id)
                ? layout.filter(x => x !== id)
                : [...layout, id];
            await persistAndRerender(next);
        });
    }
}

async function loadDisciplineScore(el, accountId) {
    try {
        const s = await api.disciplineScore(accountId, 7);
        const cls = s.score >= 90 ? 'pos' : s.score >= 75 ? 'warn' : 'neg';
        const body = t('view.dashboard.discipline.body', {
            stop_set:       s.component_stop_set,
            stop_honored:   s.component_stop_honored,
            plan:           s.component_plan_adherence,
            gate_restraint: s.component_gate_restraint,
        });
        const win = t('view.dashboard.discipline.window', {
            blocks:        s.gate_blocks,
            block_label:   t(s.gate_blocks === 1 ? 'view.dashboard.discipline.block_singular' : 'view.dashboard.discipline.block_plural'),
            warnings:      s.gate_warnings,
            warning_label: t(s.gate_warnings === 1 ? 'view.dashboard.discipline.warning_singular' : 'view.dashboard.discipline.warning_plural'),
        });
        el.innerHTML = `
            <div class="discipline-strip">
                <div class="discipline-num discipline-${cls}">${s.score}</div>
                <div class="discipline-grade discipline-${cls}">${esc(s.grade)}</div>
                <div class="muted small discipline-detail">
                    ${esc(body)}
                    <br>${esc(win)}
                </div>
            </div>
        `;
    } catch (_) {
        el.textContent = t('view.dashboard.discipline.unavailable');
    }
}

async function loadRiskGateBadge(el) {
    try {
        const fires = await api.riskFires(200);
        const today = new Date().toISOString().slice(0, 10);
        const todays = fires.filter(f => f.fired_at.slice(0, 10) === today);
        const blocks = todays.filter(f => f.blocked).length;
        const warns  = todays.length - blocks;
        if (!todays.length) {
            el.innerHTML = `<span class="muted">${t('view.dashboard.empty.no_fires_today')}</span>`;
            return;
        }
        el.innerHTML = t('view.dashboard.risk_gate.body', {
            blocks_html: `<strong class="discipline-neg">${blocks}</strong>`,
            warns_html:  `<strong class="discipline-warn">${warns}</strong>`,
            audit_link:  `<a href="#risk-gate">${esc(t('view.dashboard.risk_gate.audit_log'))}</a>`,
        });
    } catch (_) {
        el.textContent = t('view.dashboard.risk_gate.unavailable');
    }
}
