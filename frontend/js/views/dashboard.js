import { api } from '../api.js';
import { fmtMoney, fmtSecs, pnlClass, applyBarWidths } from '../util.js';
import { equityChart } from '../charts.js';
import { renderWorldMarkets } from './world_map.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { initDragReorder, resetDragReorder } from '../drag_reorder.js';

const INTERVAL_KEY = 'dashboard_interval_days';
const PERIOD_KEY   = 'dashboard_period_key';
const VALID_INTERVALS = [30, 60, 90];
// Calendar-aware quick-pick periods. Each maps to a number of days that
// covers the period; 'all' clears the filter. Computed lazily so YTD
// stretches the right amount each call.
const PERIOD_KEYS = ['today', 'wtd', 'mtd', 'qtd', 'ytd', 'all'];

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

function dayStrip(cal) {
    const map = new Map((cal || []).map(c => [c.day, c]));
    const cells = [];
    const today = new Date();
    for (let i = 6; i >= 0; i--) {
        const d = new Date(today);
        d.setDate(d.getDate() - i);
        const key = d.toISOString().slice(0, 10);
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
                <div class="dash-tv-day-trades">${trades} ${trades === 1 ? 'trade' : 'trades'}</div>
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
            scales: { x: {}, y: { auto: true } },
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

function dailyVolumeChart(elId, daily) {
    if (!daily || !daily.length) return false;
    setTimeout(() => {
        const el = document.getElementById(elId);
        if (!el || !window.uPlot) return;
        const xs = daily.map((_, i) => i);
        const ys = daily.map(d => Number(d.volume) || 0);
        const labels = daily.map(d => shortDay(d.day));
        const barsPath = (u) => {
            const ctx = u.ctx;
            ctx.save();
            const bw = Math.max(2, (u.bbox.width / xs.length) * 0.7);
            const yZero = u.valToPos(0, 'y', true);
            for (let i = 0; i < xs.length; i++) {
                const x = u.valToPos(xs[i], 'x', true);
                const y = u.valToPos(ys[i], 'y', true);
                ctx.fillStyle = '#39ff14';
                ctx.fillRect(x - bw / 2, Math.min(yZero, y), bw, Math.abs(y - yZero));
            }
            ctx.restore();
            return null;
        };
        new window.uPlot({
            title: '', width: el.clientWidth || 600, height: 260,
            scales: { x: {}, y: {} },
            series: [
                { label: 'day' },
                { label: 'volume', stroke: 'transparent', paths: barsPath },
            ],
            axes: [
                // size:60 reserves enough vertical room for the rotated date
                // labels — at size:28 they get clipped to "20-" in release.
                { stroke: '#aab', size: 60, rotate: -45,
                  values: (_u, splits) => splits.map(v => labels[Math.round(v)] || '') },
                { stroke: '#aab', size: 64,
                  values: (_u, ticks) => ticks.map(v => {
                      const a = Math.abs(v);
                      if (a >= 1e6) return `${(v/1e6).toFixed(1)}M`;
                      if (a >= 1e3) return `${(v/1e3).toFixed(0)}K`;
                      return v.toFixed(0);
                  }) },
            ],
            legend: { show: false },
        }, [xs, ys], el);
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
            scales: { x: {}, y: { auto: true } },
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
];
const WIDGETS_BY_ID = new Map(WIDGETS.map(w => [w.id, w]));
const DEFAULT_LAYOUT = WIDGETS.map(w => w.id);

function renderLayoutPanels(layout, data) {
    return layout
        .map(id => WIDGETS_BY_ID.get(id))
        .filter(Boolean)
        .map(w => `
            <div class="chart-panel${w.spans2 ? ' dash-tv-span-2' : ''}" data-widget-id="${w.id}">
                <span class="dash-tv-drag-handle" title="drag to reorder" data-drag-handle>⠿</span>
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
            const id = btn.dataset.delWidget;
            const panel = btn.closest('.chart-panel[data-widget-id]');
            if (panel) panel.remove();
            const next = [...grid.querySelectorAll('.chart-panel[data-widget-id]')]
                .map(el => el.dataset.widgetId);
            persistFn(next).catch((err) => console.warn('layout persist failed', err));
        });
    });
}

let cachedSettings = null;
async function loadLayout() {
    try {
        cachedSettings = await api.settings();
        const stored = cachedSettings?.dashboard_layout;
        if (stored && Array.isArray(stored.order)) {
            // Drop unknown IDs (e.g. widgets removed between versions).
            const known = stored.order.filter(id => WIDGETS_BY_ID.has(id));
            return known.length ? known : DEFAULT_LAYOUT.slice();
        }
    } catch (_) { /* fall through to default */ }
    return DEFAULT_LAYOUT.slice();
}
async function saveLayout(order) {
    // If we never managed to load settings (first-render race, transient
    // network blip), pull them fresh now so the user's drag/delete isn't
    // silently dropped. saveLayout used to early-return here and the drop
    // appeared to do nothing.
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

    const [summary, equity, cal, dow, hold, hour, dd, byPrice, daily, tags,
           byMonth, bySymbol, byDurationCoarse, byRBucket, byOpeningGap,
           byInstrumentVolume, byMovement, openTrades, layout] = await Promise.all([
        api.summary(state.accountId, interval),
        api.equity(state.accountId, undefined, interval),
        api.calendar(state.accountId, interval),
        api.byDow(state.accountId, interval).catch(() => []),
        api.byHold(state.accountId, interval).catch(() => []),
        api.byHour(state.accountId, interval).catch(() => []),
        api.drawdown(state.accountId, undefined, interval).catch(() => null),
        api.byPrice(state.accountId, interval).catch(() => []),
        api.dailySeries(state.accountId, interval).catch(() => []),
        api.byTag(state.accountId, interval).catch(() => []),
        api.byMonth(state.accountId, interval).catch(() => []),
        api.bySymbol(state.accountId, interval).catch(() => []),
        api.byDurationCoarse(state.accountId, interval).catch(() => []),
        api.byRBucket(state.accountId, interval).catch(() => []),
        api.byOpeningGap(state.accountId, interval).catch(() => []),
        api.byInstrumentVolume(state.accountId, interval).catch(() => []),
        api.byMovement(state.accountId, interval).catch(() => []),
        api.trades(state.accountId, { status: 'open', limit: 100 }).catch(() => []),
        loadLayout(),
    ]);
    if (!viewIsCurrent(tok)) return;
    const data = { equity, summary, dow, hold, hour, byPrice, dd, daily, tags, cal,
                   byMonth, bySymbol, byDurationCoarse, byRBucket, byOpeningGap,
                   byInstrumentVolume, byMovement, openTrades };

    mount.innerHTML = `
        <div class="dash-tv-header">
            <h1 class="view-title"><span data-i18n="view.dashboard.h1.dashboard_2">// DASHBOARD</span></h1>
            <button type="button" class="btn btn-secondary btn-compact" id="dashboard-refresh-btn"
                    data-i18n="view.dashboard.btn.refresh"
                    data-tip="view.dashboard.tip.refresh"
                    data-shortcut="dashboard_refresh">⟳ Refresh</button>
            <div class="dash-tv-layout-controls">
                <div class="dash-tv-add-wrap">
                    <button type="button" id="dash-add-widget" class="btn btn-secondary btn-compact">+ ${esc(t('view.dashboard.tv.add_widget'))}</button>
                    <div class="dash-tv-add-menu" id="dash-add-menu" hidden></div>
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
        <div class="dash-tv-day-strip">${dayStrip(cal)}</div>

        <div class="dash-tv-grid" id="dash-tv-grid">
            ${renderLayoutPanels(layout, data)}
        </div>

        <div id="world-markets-mount" class="world-mount"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.dashboard.h2.risk_gate_today">🛡 Risk Gate · today</h2>
            <div id="dash-rg" class="muted small"><span data-i18n="common.loading">loading…</span></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.dashboard.h2.discipline_score_last_7_days">📐 Discipline score · last 7 days</h2>
            <div id="dash-disc"><span data-i18n="common.loading">loading…</span></div>
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
        addBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            const inLayout = new Set(layout);
            const missing = WIDGETS.filter(w => !inLayout.has(w.id));
            addMenu.innerHTML = missing.length
                ? missing.map(w =>
                    `<button type="button" class="dash-tv-add-item" data-add-widget="${w.id}">+ ${esc(t(w.titleKey))}</button>`
                  ).join('')
                : `<div class="dash-tv-add-empty">${esc(t('view.dashboard.tv.all_widgets_shown'))}</div>`;
            addMenu.hidden = !addMenu.hidden;
        });
        mount.addEventListener('click', (e) => {
            if (addMenu.hidden) return;
            if (addMenu.contains(e.target) || e.target === addBtn) return;
            addMenu.hidden = true;
        });
        addMenu.addEventListener('click', async (e) => {
            const btn = e.target.closest('[data-add-widget]');
            if (!btn) return;
            const id = btn.dataset.addWidget;
            if (!WIDGETS_BY_ID.has(id) || layout.includes(id)) return;
            await persistAndRerender([...layout, id]);
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
