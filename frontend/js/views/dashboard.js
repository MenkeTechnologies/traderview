import { api } from '../api.js';
import { fmtMoney, fmtPct, fmtSecs, pnlClass } from '../util.js';
import { equityChart } from '../charts.js';
import { renderWorldMarkets } from './world_map.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const INTERVAL_KEY = 'dashboard_interval_days';
const VALID_INTERVALS = [30, 60, 90];

function getInterval() {
    const v = Number(localStorage.getItem(INTERVAL_KEY));
    return VALID_INTERVALS.includes(v) ? v : 90;
}
function setInterval(days) {
    if (VALID_INTERVALS.includes(days)) localStorage.setItem(INTERVAL_KEY, String(days));
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
            <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${kind}" style="width:${pct}%"></div></div>
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

function dayOfWeekWidget(dow) {
    if (!Array.isArray(dow) || !dow.length) {
        return `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`;
    }
    const order = ['Sun','Mon','Tue','Wed','Thu','Fri','Sat'];
    const byKey = new Map(dow.map(b => [b.key, b]));
    const maxAbs = Math.max(...dow.map(b => Math.abs(Number(b.net_pnl) || 0)), 1);
    const total  = dow.reduce((a, b) => a + Math.abs(Number(b.net_pnl) || 0), 0) || 1;
    return compareWidget(order.map(k => {
        const b = byKey.get(k) || { net_pnl: 0, trades: 0 };
        const v = Number(b.net_pnl) || 0;
        const pct = (Math.abs(v) / total) * 100;
        return `
            <div class="dash-tv-compare-row">
                <div class="dash-tv-compare-label">${k}</div>
                <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${v >= 0 ? 'pos' : 'neg'}" style="width:${(Math.abs(v) / maxAbs * 100).toFixed(0)}%"></div></div>
                <div class="dash-tv-compare-value ${pnlClass(v)}">${fmtMoney(v)} <span class="muted" style="font-weight:400">${pct.toFixed(2)}%</span></div>
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
                <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${v >= 0 ? 'pos' : 'neg'}" style="width:${(Math.abs(v) / maxAbs * 100).toFixed(0)}%"></div></div>
                <div class="dash-tv-compare-value ${pnlClass(v)}">${fmtMoney(v)} <span class="muted" style="font-weight:400">${pct.toFixed(2)}%</span></div>
            </div>
        `;
    }));
}

function profitFactorGauge(pf) {
    const v = Number(pf) || 0;
    // Normalize: 0 → empty arc, 1 → half arc, 2+ → full arc
    const clamped = Math.min(Math.max(v / 2, 0), 1);
    const angle = 180 * clamped; // 0..180 deg semicircle sweep
    const color = v >= 1.5 ? '#39ff14' : v >= 1.0 ? '#ffd84a' : '#ff3860';
    const r = 70;
    const cx = 90, cy = 90;
    const rad = (angle - 180) * Math.PI / 180;
    const endX = cx + r * Math.cos(rad);
    const endY = cy + r * Math.sin(rad);
    const largeArc = angle > 180 ? 1 : 0;
    return `
        <div style="display:flex;flex-direction:column;align-items:center;justify-content:center;padding:12px 0;gap:8px">
            <div style="font-size:32px;font-weight:700;color:${color};font-family:'Orbitron',sans-serif">${v.toFixed(2)}</div>
            <svg width="180" height="100" viewBox="0 0 180 100">
                <path d="M 20 90 A 70 70 0 0 1 160 90" stroke="rgba(255,255,255,0.08)" stroke-width="10" fill="none"/>
                <path d="M 20 90 A 70 70 0 ${largeArc} 1 ${endX.toFixed(2)} ${endY.toFixed(2)}" stroke="${color}" stroke-width="10" fill="none"/>
            </svg>
        </div>
    `;
}

function drawdownChart(elId, dd) {
    if (!dd || !Array.isArray(dd.series) || !dd.series.length) return false;
    setTimeout(() => {
        const el = document.getElementById(elId);
        if (!el || !window.uPlot) return;
        const xs = dd.series.map((_, i) => i);
        const ys = dd.series.map(p => Number(p.value) || 0);
        new window.uPlot({
            title: '', width: el.clientWidth || 600, height: 220,
            scales: { x: {}, y: { auto: true } },
            series: [
                { label: 'idx' },
                { label: 'drawdown', stroke: '#ff3860', width: 2,
                  fill: 'rgba(255,56,96,0.18)' },
            ],
            axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 56 }],
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
                <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${v >= 0 ? 'pos' : 'neg'}" style="width:${(Math.abs(v) / maxAbs * 100).toFixed(0)}%"></div></div>
                <div class="dash-tv-compare-value ${pnlClass(v)}">${fmtMoney(v)} <span class="muted" style="font-weight:400">${pct.toFixed(2)}%</span></div>
            </div>
        `;
    }));
}

function dailyVolumeChart(elId, daily) {
    if (!daily || !daily.length) return false;
    setTimeout(() => {
        const el = document.getElementById(elId);
        if (!el || !window.uPlot) return;
        const xs = daily.map((_, i) => i);
        const ys = daily.map(d => Number(d.volume) || 0);
        const labels = daily.map(d => d.day);
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
            title: '', width: el.clientWidth || 600, height: 220,
            scales: { x: {}, y: {} },
            series: [
                { label: 'day' },
                { label: 'volume', stroke: 'transparent', paths: barsPath },
            ],
            axes: [
                { stroke: '#aab', size: 28, rotate: -45,
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
        const labels = daily.map(d => d.day);
        new window.uPlot({
            title: '', width: el.clientWidth || 600, height: 220,
            scales: { x: {}, y: { auto: true } },
            series: [
                { label: 'day' },
                { label: valueKey, stroke: color, width: 2 },
            ],
            axes: [
                { stroke: '#aab', size: 28, rotate: -45,
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
            <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${intraday >= 0 ? 'pos' : 'neg'}" style="width:${(Math.abs(intraday)/max*100).toFixed(0)}%"></div></div>
            <div class="dash-tv-compare-value ${pnlClass(intraday)}">${fmtMoney(intraday)}</div>
        </div>`,
        `<div class="dash-tv-compare-row">
            <div class="dash-tv-compare-label">${esc(t('view.dashboard.tv.multiday'))}</div>
            <div class="dash-tv-compare-track"><div class="dash-tv-compare-fill ${multiday >= 0 ? 'pos' : 'neg'}" style="width:${(Math.abs(multiday)/max*100).toFixed(0)}%"></div></div>
            <div class="dash-tv-compare-value ${pnlClass(multiday)}">${fmtMoney(multiday)}</div>
        </div>`,
    ]);
}

function winPctSummary(s) {
    const rate = Number(s.win_rate) || 0;
    const pct = (rate * 100).toFixed(1);
    return `
        <div style="display:flex;flex-direction:column;align-items:center;justify-content:center;padding:32px 0;gap:8px">
            <div style="font-size:48px;font-weight:700;color:var(--cyan);font-family:'Orbitron',sans-serif">${pct}%</div>
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
        html: (d) => `<div id="dash-win-pct-chart" style="width:100%;height:220px">${d.daily && d.daily.length ? '' : `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`}</div>`,
        mount: (d) => { if (d.daily && d.daily.length) lineChart('dash-win-pct-chart', d.daily, 'running_win_rate', '#39ff14'); } },
    { id: 'perf_dow', titleKey: 'view.dashboard.tv.perf_dow',
        html: (d) => dayOfWeekWidget(d.dow) },
    { id: 'mfe_mae', titleKey: 'view.dashboard.tv.mfe_mae',
        html: () => `<div class="dash-tv-na">${esc(t('view.dashboard.tv.mfe_mae_unavailable'))}</div>` },
    { id: 'perf_duration', titleKey: 'view.dashboard.tv.perf_duration',
        html: (d) => durationWidget(d.hold) },
    { id: 'perf_hour', titleKey: 'view.dashboard.tv.perf_hour',
        html: (d) => hourOfDayWidget(d.hour) },
    { id: 'profit_factor', titleKey: 'view.dashboard.tv.profit_factor',
        html: (d) => profitFactorGauge(d.summary.profit_factor) },
    { id: 'total_fees', titleKey: 'view.dashboard.tv.total_fees',
        html: (d) => `<div style="display:flex;align-items:center;justify-content:center;padding:36px 0"><div style="font-size:32px;font-weight:700;color:#ffd84a;font-family:'JetBrains Mono',monospace">${esc(fmtMoney(d.summary.fees))}</div></div>` },
    { id: 'cumulative_drawdown', titleKey: 'view.dashboard.tv.cumulative_drawdown', spans2: true,
        html: (d) => `<div id="dash-drawdown-chart" style="width:100%;height:220px">${d.dd ? '' : `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`}</div>`,
        mount: (d) => { if (d.dd) drawdownChart('dash-drawdown-chart', d.dd); } },
    { id: 'perf_price', titleKey: 'view.dashboard.tv.perf_price',
        html: (d) => byPriceWidget(d.byPrice) },
    { id: 'daily_volume', titleKey: 'view.dashboard.tv.daily_volume', spans2: true,
        html: (d) => `<div id="dash-daily-volume-chart" style="width:100%;height:220px">${d.daily && d.daily.length ? '' : `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`}</div>`,
        mount: (d) => { if (d.daily && d.daily.length) dailyVolumeChart('dash-daily-volume-chart', d.daily); } },
    { id: 'avg_trade_pnl', titleKey: 'view.dashboard.tv.avg_trade_pnl', spans2: true,
        html: (d) => `<div id="dash-avg-pnl-chart" style="width:100%;height:220px">${d.daily && d.daily.length ? '' : `<div class="dash-tv-na">${esc(t('view.dashboard.empty.no_data'))}</div>`}</div>`,
        mount: (d) => { if (d.daily && d.daily.length) lineChart('dash-avg-pnl-chart', d.daily, 'running_avg_pnl', '#00e5ff'); } },
];
const WIDGETS_BY_ID = new Map(WIDGETS.map(w => [w.id, w]));
const DEFAULT_LAYOUT = WIDGETS.map(w => w.id);

function renderLayoutPanels(layout, data) {
    return layout
        .map(id => WIDGETS_BY_ID.get(id))
        .filter(Boolean)
        .map(w => `
            <div class="chart-panel${w.spans2 ? ' dash-tv-span-2' : ''}" draggable="true" data-widget-id="${w.id}">
                <span class="dash-tv-drag-handle" title="drag to reorder">⠿</span>
                <span class="dash-tv-del-btn" title="remove from layout" data-del-widget="${w.id}">✕</span>
                <h2 data-i18n="${w.titleKey}">${esc(t(w.titleKey))}</h2>
                ${w.html(data)}
            </div>
        `).join('');
}

function attachLayoutHandlers(mount, layout, data, persistFn) {
    const grid = mount.querySelector('#dash-tv-grid');
    if (!grid) return;
    let dragged = null;

    grid.addEventListener('dragstart', (e) => {
        const panel = e.target.closest('.chart-panel[data-widget-id]');
        if (!panel) return;
        dragged = panel;
        panel.classList.add('dragging');
        e.dataTransfer.effectAllowed = 'move';
        e.dataTransfer.setData('text/plain', panel.dataset.widgetId);
    });
    grid.addEventListener('dragend', (e) => {
        const panel = e.target.closest('.chart-panel[data-widget-id]');
        if (panel) panel.classList.remove('dragging');
        grid.querySelectorAll('.drop-target').forEach(el => el.classList.remove('drop-target'));
        dragged = null;
    });
    grid.addEventListener('dragover', (e) => {
        const over = e.target.closest('.chart-panel[data-widget-id]');
        if (!over || over === dragged) return;
        e.preventDefault();
        e.dataTransfer.dropEffect = 'move';
        grid.querySelectorAll('.drop-target').forEach(el => el.classList.remove('drop-target'));
        over.classList.add('drop-target');
    });
    grid.addEventListener('drop', async (e) => {
        e.preventDefault();
        const over = e.target.closest('.chart-panel[data-widget-id]');
        if (!over || !dragged || over === dragged) return;
        const order = [...grid.querySelectorAll('.chart-panel[data-widget-id]')];
        const srcIdx = order.indexOf(dragged);
        const dstIdx = order.indexOf(over);
        if (srcIdx < 0 || dstIdx < 0) return;
        order.splice(srcIdx, 1);
        order.splice(dstIdx, 0, dragged);
        const newLayout = order.map(el => el.dataset.widgetId);
        await persistFn(newLayout);
    });

    grid.querySelectorAll('[data-del-widget]').forEach(btn => {
        btn.addEventListener('click', async (e) => {
            e.stopPropagation();
            const id = btn.dataset.delWidget;
            const next = layout.filter(w => w !== id);
            await persistFn(next);
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
    if (!cachedSettings) return;
    cachedSettings.dashboard_layout = { order };
    try {
        await api.updateSettings(cachedSettings);
    } catch (e) {
        console.warn('layout save failed', e);
    }
}

export async function renderDashboard(mount, state) {
    const tok = currentViewToken();
    const interval = getInterval();

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

    const [summary, equity, cal, dow, hold, hour, dd, byPrice, daily, layout] = await Promise.all([
        api.summary(state.accountId, interval),
        api.equity(state.accountId, undefined, interval),
        api.calendar(state.accountId, interval),
        api.byDow(state.accountId, interval).catch(() => []),
        api.byHold(state.accountId, interval).catch(() => []),
        api.byHour(state.accountId, interval).catch(() => []),
        api.drawdown(state.accountId, undefined, interval).catch(() => null),
        api.byPrice(state.accountId, interval).catch(() => []),
        api.dailySeries(state.accountId, interval).catch(() => []),
        loadLayout(),
    ]);
    if (!viewIsCurrent(tok)) return;
    const data = { equity, summary, dow, hold, hour, byPrice, dd, daily };

    mount.innerHTML = `
        <div class="dash-tv-header">
            <h1 class="view-title"><span data-i18n="view.dashboard.h1.dashboard_2">// DASHBOARD</span></h1>
            <button type="button" class="btn btn-secondary" id="dashboard-refresh-btn"
                    data-i18n="view.dashboard.btn.refresh"
                    data-tip="view.dashboard.tip.refresh"
                    data-shortcut="dashboard_refresh"
                    style="font-size:11px;padding:4px 10px">⟳ Refresh</button>
            <div class="dash-tv-layout-controls">
                <button type="button" id="dash-add-widget" class="btn btn-secondary" style="font-size:11px;padding:4px 10px">+ ${esc(t('view.dashboard.tv.add_widget'))}</button>
                <button type="button" id="dash-reset-layout" class="btn btn-secondary" style="font-size:11px;padding:4px 10px">⟲ ${esc(t('view.dashboard.tv.reset_layout'))}</button>
            </div>
            <div class="dash-tv-toggle" role="tablist">
                ${VALID_INTERVALS.map(d => `<button type="button" data-interval="${d}" class="${d === interval ? 'active' : ''}">${d} Days</button>`).join('')}
            </div>
        </div>
        <div class="dash-tv-range">${esc(rangeLabel(interval))}</div>
        <div class="dash-tv-day-strip">${dayStrip(cal)}</div>

        <div class="dash-tv-add-menu" id="dash-add-menu" hidden></div>

        <div class="dash-tv-grid" id="dash-tv-grid">
            ${renderLayoutPanels(layout, data)}
        </div>

        <div id="world-markets-mount" style="margin-top:14px"></div>

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
            renderDashboard(mount, state);
        });
    });

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

    const persist = async (newLayout) => {
        await saveLayout(newLayout);
        renderDashboard(mount, state);
    };
    attachLayoutHandlers(mount, layout, data, persist);

    const resetBtn = mount.querySelector('#dash-reset-layout');
    if (resetBtn) resetBtn.addEventListener('click', () => persist(DEFAULT_LAYOUT.slice()));

    const addBtn = mount.querySelector('#dash-add-widget');
    const addMenu = mount.querySelector('#dash-add-menu');
    if (addBtn && addMenu) {
        addBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            const inLayout = new Set(layout);
            const missing = WIDGETS.filter(w => !inLayout.has(w.id));
            if (!missing.length) {
                addMenu.innerHTML = `<div class="dash-tv-add-empty">${esc(t('view.dashboard.tv.all_widgets_shown'))}</div>`;
            } else {
                addMenu.innerHTML = missing.map(w =>
                    `<button type="button" class="dash-tv-add-item" data-add-widget="${w.id}">+ ${esc(t(w.titleKey))}</button>`
                ).join('');
            }
            addMenu.hidden = !addMenu.hidden;
        });
        document.addEventListener('click', (e) => {
            if (!addMenu.hidden && !addMenu.contains(e.target) && e.target !== addBtn) addMenu.hidden = true;
        }, { once: true });
        addMenu.addEventListener('click', async (e) => {
            const btn = e.target.closest('[data-add-widget]');
            if (!btn) return;
            const id = btn.dataset.addWidget;
            if (!WIDGETS_BY_ID.has(id) || layout.includes(id)) return;
            await persist([...layout, id]);
        });
    }
}

async function loadDisciplineScore(el, accountId) {
    try {
        const s = await api.disciplineScore(accountId, 7);
        const color = s.score >= 90 ? '#39ff14'
                    : s.score >= 75 ? '#ffb800'
                                    : '#ff2a6d';
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
            <div style="display:flex;align-items:center;gap:20px;flex-wrap:wrap">
                <div style="font-size:48px;font-weight:700;color:${color};line-height:1">${s.score}</div>
                <div style="font-size:24px;color:${color}">${esc(s.grade)}</div>
                <div class="muted small" style="flex:1;min-width:200px">
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
            blocks_html: `<strong style="color:#ff2a6d">${blocks}</strong>`,
            warns_html:  `<strong style="color:#ffb800">${warns}</strong>`,
            audit_link:  `<a href="#risk-gate">${esc(t('view.dashboard.risk_gate.audit_log'))}</a>`,
        });
    } catch (_) {
        el.textContent = t('view.dashboard.risk_gate.unavailable');
    }
}
