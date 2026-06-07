import { api } from '../api.js';
import { fmt, fmtMoney, fmtPct, fmtDate, fmtSecs, pnlClass, esc, statCard, applyBarWidths } from '../util.js';
import { barChart, equityChart } from '../charts.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { initDragReorder, resetDragReorder } from '../drag_reorder.js';
import { showToast } from '../toast.js';
import { tPrompt } from '../dialog.js';

// Tradervue-style primary tabs (top-level reports). Each maps to a render
// function below. Detailed and Drawdown delegate to the original detailed
// section so we don't lose access to per-dimension bar reports.
const PRIMARY_TABS = [
    ['overview',       'Overview'],
    ['detailed',       'Detailed'],
    ['win-loss-days',  'Win vs Loss Days'],
    ['drawdown',       'Drawdown'],
    ['compare',        'Compare'],
    ['tag-breakdown',  'Tag Breakdown'],
    ['advanced',       'Advanced'],
    ['year-month-day', 'Year / Month / Day'],
    ['calendar',       'Calendar'],
];

// Legacy single-dimension reports live under the Detailed tab as a sub-strip.
const DETAILED_SUBS = [
    ['by-symbol',   'By Symbol'],
    ['by-side',     'By Side'],
    ['by-asset',    'By Asset'],
    ['by-dow',      'By Day of Week'],
    ['by-hour',     'By Hour'],
    ['by-hold',     'By Hold Time'],
    ['by-month',    'By Month'],
    ['by-price',    'By Price'],
    ['r-dist',      'R-Multiple'],
    ['streaks',     'Streaks'],
    ['comparison',  'Long vs Short'],
    ['exit-eff',    'Exit Efficiency'],
    ['commissions', 'Commissions'],
    ['liquidity',   'Liquidity'],
    ['risk',        'Risk'],
    ['risk-adjusted', 'Sharpe / Sortino'],
];

// ----------------------------------------------------------------------------
// Filter + display-style state (persisted in sessionStorage so navigation
// between tabs preserves the user's filters; matches Tradervue's behavior).
// ----------------------------------------------------------------------------
const FILTER_KEY = 'reports_filter_v1';
const STYLE_KEY  = 'reports_style_v1';
const FILTER_DEFAULTS = {
    symbol: '',
    side: '',          // '' | 'long' | 'short'
    asset_class: '',   // '' | 'stock' | 'option' | 'future' | 'forex'
    duration: '',      // '' | 'intraday' | 'multiday'
    date_from: '',
    date_to: '',
    tag_id: '',
    days: '',          // '' | 30 | 60 | 90
};
const STYLE_DEFAULTS = {
    pnl_type: 'net',     // 'net' | 'gross'
    view_mode: 'value',  // 'value' ($) | 'pct' (% of total)
    style:     'aggregate', // 'aggregate' | 'per_trade'
};

function loadFilter() {
    try { return { ...FILTER_DEFAULTS, ...JSON.parse(sessionStorage.getItem(FILTER_KEY) || '{}') }; }
    catch (_) { return { ...FILTER_DEFAULTS }; }
}
function saveFilter(f) { sessionStorage.setItem(FILTER_KEY, JSON.stringify(f)); }
function loadStyle()  {
    try { return { ...STYLE_DEFAULTS, ...JSON.parse(sessionStorage.getItem(STYLE_KEY) || '{}') }; }
    catch (_) { return { ...STYLE_DEFAULTS }; }
}
function saveStyle(s)  { sessionStorage.setItem(STYLE_KEY,  JSON.stringify(s)); }

// Drop empty values when shipping to the server (qs() already skips them but
// being explicit here makes the wire shape obvious in DevTools).
function filterForApi(f) {
    const out = {};
    for (const [k, v] of Object.entries(f)) {
        if (v === '' || v === null || v === undefined) continue;
        out[k] = v;
    }
    return out;
}

// ----------------------------------------------------------------------------
// Main entry point. `sub` is the primary tab (overview, detailed, etc).
// When sub === 'detailed' we read the secondary tab from location.hash.
// ----------------------------------------------------------------------------
export async function renderReports(mount, state, sub) {
    const tok = currentViewToken();
    if (!state.accountId) {
        mount.innerHTML = '<p data-i18n="view.reports.hint.no_account" class="boot">No account.</p>';
        return;
    }

    // Resolve primary tab + (optional) sub-tab for the Detailed section.
    let primary = sub || 'overview';
    let detailedSub = null;
    const isLegacyDetailed = DETAILED_SUBS.find(s => s[0] === primary);
    if (isLegacyDetailed) {
        detailedSub = primary;
        primary = 'detailed';
    }
    if (!PRIMARY_TABS.find(p => p[0] === primary)) primary = 'overview';

    const filter = loadFilter();
    const style  = loadStyle();
    // If sessionStorage carries a stale tag_id (the tag was deleted from the
    // Tags page after this filter was saved), drop it before the next request
    // — otherwise /reports/by-tag?tag_id=X returns 404 / "not found" and the
    // user sees an error toast on a page they didn't change.
    if (filter.tag_id) {
        try {
            const tags = await api.tags();
            if (!tags.some(t => t.id === filter.tag_id)) {
                filter.tag_id = '';
                saveFilter(filter);
            }
        } catch (_) { /* tags fetch failed; leave filter untouched */ }
    }

    mount.innerHTML = `
        <h1 data-i18n="view.reports.h1.reports" class="view-title">// REPORTS</h1>
        ${filterBarHtml(filter)}
        <div class="report-tabs report-primary-tabs">
            ${PRIMARY_TABS.map(([k, l]) => `
                <a class="report-tab ${k === primary ? 'active' : ''}"
                   href="#reports/${k}" data-i18n="view.reports.tab.${k}">${l}</a>
            `).join('')}
        </div>
        ${styleBarHtml(style)}
        ${primary === 'detailed' ? `
            <div class="report-tabs report-secondary-tabs">
                ${DETAILED_SUBS.map(([k, l]) => `
                    <a class="report-tab ${k === detailedSub ? 'active' : ''}"
                       href="#reports/${k}" data-i18n="view.reports.tab.${k}">${l}</a>
                `).join('')}
            </div>
        ` : ''}
        <div id="report-body">
            <div class="tv-spinner-wrap"><div class="tv-spinner"></div>
                <div class="tv-spinner-text" data-i18n="common.loading">loading…</div>
            </div>
        </div>
    `;

    wireFilterBar(mount, state, filter);
    wireStyleBar(mount, state, style);

    // Trello-style reorder on the primary tab strip. Persists order to
    // localStorage so the user's preferred tab arrangement sticks across
    // sessions. Routing still works because each tab keeps its href.
    const primaryTabs = mount.querySelector('.report-primary-tabs');
    if (primaryTabs) {
        resetDragReorder(primaryTabs);
        initDragReorder(primaryTabs, '.report-tab', 'reports_primary_tab_order', {
            direction: 'horizontal',
            getKey: (el) => (el.getAttribute('href') || '').replace('#reports/', '') || el.textContent.trim(),
            toastMessage: t('toast.reordered_tabs'),
        });
    }
    const secondaryTabs = mount.querySelector('.report-secondary-tabs');
    if (secondaryTabs) {
        resetDragReorder(secondaryTabs);
        initDragReorder(secondaryTabs, '.report-tab', 'reports_secondary_tab_order', {
            direction: 'horizontal',
            getKey: (el) => (el.getAttribute('href') || '').replace('#reports/', '') || el.textContent.trim(),
            toastMessage: t('toast.reordered_tabs'),
        });
    }

    const acct = state.accountId;
    const apiF = filterForApi(filter);
    const body = mount.querySelector('#report-body');
    const setBody = (html) => {
        if (!viewIsCurrent(tok)) return null;
        const el = mount.querySelector('#report-body');
        if (el) el.innerHTML = html;
        return el;
    };

    try {
        if (primary === 'overview') {
            const [s, dow, hour, byMonth, byHold, byDurationCoarse, byRBucket] = await Promise.all([
                api.overview(acct, apiF),
                api.byDow(acct, apiF),
                api.byHour(acct, apiF),
                api.byMonth(acct, apiF).catch(() => []),
                api.byHold(acct, apiF).catch(() => []),
                api.byDurationCoarse(acct, apiF).catch(() => []),
                api.byRBucket(acct, apiF).catch(() => []),
            ]);
            if (!viewIsCurrent(tok)) return;
            setBody(overviewHtml(s, style));
            renderDistBars(mount.querySelector('#rep-dist-dow'),       dow,              'trades');
            renderDistBars(mount.querySelector('#rep-perf-dow'),       dow,              'net_pnl');
            renderDistBars(mount.querySelector('#rep-dist-hour'),      hour,             'trades');
            renderDistBars(mount.querySelector('#rep-perf-hour'),      hour,             'net_pnl');
            renderDistBars(mount.querySelector('#rep-dist-month'),     byMonth,          'trades');
            renderDistBars(mount.querySelector('#rep-perf-month'),     byMonth,          'net_pnl');
            renderDistBars(mount.querySelector('#rep-dist-hold'),      byHold,           'trades');
            renderDistBars(mount.querySelector('#rep-perf-hold'),      byHold,           'net_pnl');
            renderDistBars(mount.querySelector('#rep-dist-duration'),  byDurationCoarse, 'trades');
            renderDistBars(mount.querySelector('#rep-perf-duration'),  byDurationCoarse, 'net_pnl');
            renderDistBars(mount.querySelector('#rep-dist-rbucket'),   byRBucket,        'trades');
            renderDistBars(mount.querySelector('#rep-perf-rbucket'),   byRBucket,        'net_pnl');
            applyBarWidths(mount);
        } else if (primary === 'detailed') {
            await renderDetailedTab(mount, state, detailedSub || 'by-symbol', apiF, style, tok);
        } else if (primary === 'win-loss-days') {
            const wld = await api.winLossDays(acct, apiF);
            if (!viewIsCurrent(tok)) return;
            renderWinLossDays(body, wld);
        } else if (primary === 'drawdown') {
            const [dd, eq] = await Promise.all([
                api.drawdown(acct, undefined, apiF),
                api.equity(acct, undefined, apiF),
            ]);
            if (!viewIsCurrent(tok)) return;
            setBody(drawdownHtml(dd));
            const eqMount = mount.querySelector('#eq-mount');
            if (eqMount) equityChart(eqMount, eq);
        } else if (primary === 'compare') {
            await renderCompareTab(mount, state, tok);
        } else if (primary === 'tag-breakdown') {
            const tags = await api.byTag(acct, apiF);
            if (!viewIsCurrent(tok)) return;
            renderTagBreakdown(body, tags, style);
        } else if (primary === 'advanced') {
            const adv = await api.advanced(acct, undefined, apiF);
            if (!viewIsCurrent(tok)) return;
            renderAdvanced(body, adv);
        } else if (primary === 'year-month-day') {
            const [monthly, cal] = await Promise.all([
                api.byMonth(acct, apiF),
                api.calendar(acct, apiF),
            ]);
            if (!viewIsCurrent(tok)) return;
            renderYearMonthDay(body, monthly, cal);
        } else if (primary === 'calendar') {
            const cal = await api.calendar(acct, apiF);
            if (!viewIsCurrent(tok)) return;
            renderCalendarYearGrid(body, cal);
        }
    } catch (e) {
        setBody(`<p class="boot">${esc(t('view.reports.error', { msg: e.message }))}</p>`);
    }
}

// ----------------------------------------------------------------------------
// Filter bar
// ----------------------------------------------------------------------------
function filterBarHtml(f) {
    return `
        <div class="report-filter-bar">
            <label class="rfb-field">
                <span data-i18n="view.reports.filter.symbol">Symbol</span>
                <input type="text" id="rfb-symbol" value="${esc(f.symbol)}" placeholder="${esc(t('view.reports.filter.all'))}" />
            </label>
            <label class="rfb-field">
                <span data-i18n="view.reports.filter.side">Side</span>
                <select id="rfb-side">
                    <option value=""        ${f.side === ''      ? 'selected' : ''}>${esc(t('view.reports.filter.all'))}</option>
                    <option value="long"    ${f.side === 'long'  ? 'selected' : ''}>${esc(t('view.reports.filter.long'))}</option>
                    <option value="short"   ${f.side === 'short' ? 'selected' : ''}>${esc(t('view.reports.filter.short'))}</option>
                </select>
            </label>
            <label class="rfb-field">
                <span data-i18n="view.reports.filter.asset">Asset</span>
                <select id="rfb-asset">
                    <option value=""        ${f.asset_class === ''       ? 'selected' : ''}>${esc(t('view.reports.filter.all'))}</option>
                    <option value="stock"   ${f.asset_class === 'stock'  ? 'selected' : ''}>Stock</option>
                    <option value="option"  ${f.asset_class === 'option' ? 'selected' : ''}>Option</option>
                    <option value="future"  ${f.asset_class === 'future' ? 'selected' : ''}>Future</option>
                    <option value="forex"   ${f.asset_class === 'forex'  ? 'selected' : ''}>Forex</option>
                </select>
            </label>
            <label class="rfb-field">
                <span data-i18n="view.reports.filter.duration">Duration</span>
                <select id="rfb-duration">
                    <option value=""          ${f.duration === ''         ? 'selected' : ''}>${esc(t('view.reports.filter.all'))}</option>
                    <option value="intraday"  ${f.duration === 'intraday' ? 'selected' : ''}>Intraday</option>
                    <option value="multiday"  ${f.duration === 'multiday' ? 'selected' : ''}>Multiday</option>
                </select>
            </label>
            <label class="rfb-field">
                <span data-i18n="view.reports.filter.date_from">From</span>
                <input type="date" id="rfb-from" value="${esc(f.date_from)}" />
            </label>
            <label class="rfb-field">
                <span data-i18n="view.reports.filter.date_to">To</span>
                <input type="date" id="rfb-to" value="${esc(f.date_to)}" />
            </label>
            <label class="rfb-field">
                <span data-i18n="view.reports.filter.rolling">Rolling</span>
                <select id="rfb-days">
                    <option value=""    ${f.days === ''  ? 'selected' : ''}>${esc(t('view.reports.filter.all_time'))}</option>
                    <option value="30"  ${String(f.days) === '30' ? 'selected' : ''}>30 days</option>
                    <option value="60"  ${String(f.days) === '60' ? 'selected' : ''}>60 days</option>
                    <option value="90"  ${String(f.days) === '90' ? 'selected' : ''}>90 days</option>
                    <option value="180" ${String(f.days) === '180' ? 'selected' : ''}>180 days</option>
                    <option value="365" ${String(f.days) === '365' ? 'selected' : ''}>365 days</option>
                </select>
            </label>
            <button type="button" id="rfb-clear" class="btn btn-secondary" data-i18n="view.reports.filter.clear">Clear</button>
            <button type="button" id="rfb-save"  class="btn btn-primary"   data-i18n="view.reports.filter.save_set">Save Set</button>
            <select id="rfb-set" class="rfb-set-picker" title="${esc(t('view.reports.filter.saved_sets'))}">
                <option value="" data-i18n="view.reports.filter.saved_sets">Saved sets…</option>
            </select>
        </div>
    `;
}

function wireFilterBar(mount, state, filter) {
    const get = (id) => mount.querySelector(id);
    const collect = () => ({
        symbol:      get('#rfb-symbol').value.trim().toUpperCase(),
        side:        get('#rfb-side').value,
        asset_class: get('#rfb-asset').value,
        duration:    get('#rfb-duration').value,
        date_from:   get('#rfb-from').value,
        date_to:     get('#rfb-to').value,
        days:        get('#rfb-days').value,
        tag_id:      filter.tag_id, // not in bar; reused as-is
    });
    const apply = () => {
        saveFilter(collect());
        renderReports(mount, state, currentPrimary());
    };
    ['#rfb-symbol', '#rfb-side', '#rfb-asset', '#rfb-duration',
     '#rfb-from',   '#rfb-to',   '#rfb-days'].forEach(sel => {
        const el = get(sel);
        if (!el) return;
        if (el.tagName === 'INPUT' && el.type === 'text') {
            el.addEventListener('change', apply);
        } else {
            el.addEventListener('change', apply);
        }
    });
    const clear = get('#rfb-clear');
    if (clear) clear.addEventListener('click', () => {
        saveFilter({ ...FILTER_DEFAULTS });
        renderReports(mount, state, currentPrimary());
    });
    const save = get('#rfb-save');
    if (save) save.addEventListener('click', () => saveCurrentFilterSet(mount, state, collect()));
    populateSavedSets(mount, state);
}

function currentPrimary() {
    const h = (location.hash || '').replace(/^#/, '');
    const parts = h.split('/');
    if (parts[0] !== 'reports') return 'overview';
    const sub = parts[1] || 'overview';
    return sub;
}

// ----------------------------------------------------------------------------
// Style bar
// ----------------------------------------------------------------------------
function styleBarHtml(s) {
    const opt = (current, value, label, key) => `
        <button type="button" data-style-key="${key}" data-style-val="${value}"
                class="${current === value ? 'active' : ''}">${esc(label)}</button>
    `;
    return `
        <div class="report-style-bar">
            <div class="rsb-group">
                <span class="rsb-label" data-i18n="view.reports.style.pnl_type">P&L Type</span>
                ${opt(s.pnl_type, 'gross', t('view.reports.style.gross'), 'pnl_type')}
                ${opt(s.pnl_type, 'net',   t('view.reports.style.net'),   'pnl_type')}
            </div>
            <div class="rsb-group">
                <span class="rsb-label" data-i18n="view.reports.style.view_mode">View Mode</span>
                ${opt(s.view_mode, 'value', t('view.reports.style.dollar'), 'view_mode')}
                ${opt(s.view_mode, 'pct',   t('view.reports.style.percent'), 'view_mode')}
            </div>
            <div class="rsb-group">
                <span class="rsb-label" data-i18n="view.reports.style.style">Report Style</span>
                ${opt(s.style, 'aggregate', t('view.reports.style.aggregate'), 'style')}
                ${opt(s.style, 'per_trade', t('view.reports.style.per_trade'), 'style')}
            </div>
        </div>
    `;
}

function wireStyleBar(mount, state, style) {
    mount.querySelectorAll('.report-style-bar [data-style-key]').forEach(btn => {
        btn.addEventListener('click', () => {
            const key = btn.dataset.styleKey;
            const val = btn.dataset.styleVal;
            style[key] = val;
            saveStyle(style);
            renderReports(mount, state, currentPrimary());
        });
    });
}

// ----------------------------------------------------------------------------
// Overview (style-aware)
// ----------------------------------------------------------------------------
function overviewHtml(s, style) {
    const pnl = style.pnl_type === 'gross' ? s.gross_pnl : s.net_pnl;
    // Render Option<f64> coming back as null/undefined as em-dash so the
    // grid stays uniform when the metric is undefined (e.g. SQN with
    // zero r-spread, kelly with no losers).
    const optF = (v, digits = 2) =>
        v === null || v === undefined || !Number.isFinite(Number(v)) ? '—' : fmt(v, digits);
    const optPct = (v) =>
        v === null || v === undefined || !Number.isFinite(Number(v)) ? '—' : fmtPct(v);
    return `<div class="cards">
        ${statCard(t(style.pnl_type === 'gross' ? 'view.reports.stat.gross_pnl' : 'view.dashboard.stat.net_pnl'),
                   fmtMoney(pnl), pnlClass(pnl))}
        ${statCard(t('view.dashboard.stat.trades'),    s.trade_count)}
        ${statCard(t('view.reports.stat.wls'),         `${s.win_count} / ${s.loss_count} / ${s.scratch_count}`)}
        ${statCard(t('view.dashboard.stat.win_rate'),  fmtPct(s.win_rate))}
        ${statCard(t('view.dashboard.stat.profit_factor'), fmt(s.profit_factor))}
        ${statCard(t('view.dashboard.stat.expectancy'), fmtMoney(s.expectancy), pnlClass(s.expectancy))}
        ${statCard(t('view.reports.stat.avg_win'),     fmtMoney(s.avg_win), 'pos')}
        ${statCard(t('view.reports.stat.avg_loss'),    fmtMoney(s.avg_loss), 'neg')}
        ${statCard(t('view.dashboard.stat.largest_win'), fmtMoney(s.largest_win), 'pos')}
        ${statCard(t('view.dashboard.stat.largest_loss'), fmtMoney(s.largest_loss), 'neg')}
        ${statCard(t('view.dashboard.stat.max_consec_wins'),   s.max_consec_wins)}
        ${statCard(t('view.dashboard.stat.max_consec_losses'), s.max_consec_losses)}
        ${statCard(t('view.dashboard.stat.avg_hold'),  fmtSecs(s.avg_hold_seconds))}
        ${statCard(t('view.reports.stat.avg_win_hold'),  fmtSecs(s.avg_win_hold_seconds))}
        ${statCard(t('view.reports.stat.avg_loss_hold'), fmtSecs(s.avg_loss_hold_seconds))}
        ${statCard(t('view.reports.stat.avg_scratch_hold'), fmtSecs(s.avg_scratch_hold_seconds))}
        ${statCard(t('view.dashboard.stat.avg_r'),     fmt(s.avg_r))}
        ${statCard(t('view.reports.stat.volume'),      fmtMoney(s.total_volume))}
        ${statCard(t('view.reports.stat.commissions'), fmtMoney(s.commissions))}
        ${statCard(t('view.dashboard.stat.fees'),      fmtMoney(s.fees))}
        ${statCard(t('view.reports.stat.open'),        s.open_count)}
        ${statCard(t('view.reports.stat.trading_days'), s.trading_days)}
        ${statCard(t('view.reports.stat.avg_daily_pnl'), fmtMoney(s.avg_daily_pnl), pnlClass(s.avg_daily_pnl))}
        ${statCard(t('view.reports.stat.avg_daily_volume'), fmtMoney(s.avg_daily_volume))}
        ${statCard(t('view.reports.stat.avg_per_share_pnl'), fmtMoney(s.avg_per_share_pnl), pnlClass(s.avg_per_share_pnl))}
        ${statCard(t('view.reports.stat.net_pnl_stddev'), fmtMoney(s.net_pnl_stddev))}
        ${statCard(t('view.reports.stat.avg_mae'), fmtMoney(s.avg_mae), 'neg')}
        ${statCard(t('view.reports.stat.avg_mfe'), fmtMoney(s.avg_mfe), 'pos')}
        ${statCard(t('view.reports.stat.sqn'), optF(s.sqn))}
        ${statCard(t('view.reports.stat.k_ratio'), optF(s.k_ratio))}
        ${statCard(t('view.reports.stat.kelly_pct'), optPct(s.kelly_pct))}
        ${statCard(t('view.reports.stat.random_chance'), optPct(s.random_chance_prob))}
    </div>
    <div class="panel-grid">
        <div class="chart-panel">
            <h2 data-i18n="view.reports.dist.by_dow">Trade distribution by day of week</h2>
            <div id="rep-dist-dow" class="dist-bars"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.perf.by_dow">Performance by day of week</h2>
            <div id="rep-perf-dow" class="dist-bars"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.dist.by_hour">Trade distribution by hour of day</h2>
            <div id="rep-dist-hour" class="dist-bars"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.perf.by_hour">Performance by hour of day</h2>
            <div id="rep-perf-hour" class="dist-bars"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.dist.by_month">Trade distribution by month of year</h2>
            <div id="rep-dist-month" class="dist-bars"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.perf.by_month">Performance by month of year</h2>
            <div id="rep-perf-month" class="dist-bars"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.dist.by_hold">Trade distribution by intraday duration</h2>
            <div id="rep-dist-hold" class="dist-bars"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.perf.by_hold">Performance by intraday duration</h2>
            <div id="rep-perf-hold" class="dist-bars"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.dist.by_duration">Trade distribution by duration</h2>
            <div id="rep-dist-duration" class="dist-bars"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.perf.by_duration">Performance by duration</h2>
            <div id="rep-perf-duration" class="dist-bars"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.dist.by_rbucket">Trade distribution by R</h2>
            <div id="rep-dist-rbucket" class="dist-bars"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.perf.by_rbucket">Performance by R</h2>
            <div id="rep-perf-rbucket" class="dist-bars"></div>
        </div>
    </div>`;
}

// Horizontal bar chart with one row per bucket. Used by Overview to render
// trade-count distributions and net-P&L performance by day-of-week / hour.
// DOM-based (no canvas / chart lib) so it survives Tauri release WebKit
// compositing and avoids the inline-style stripping pitfall (widths come
// from data-bar-pct via applyBarWidths). `field` is 'trades' (unsigned,
// cyan) or 'net_pnl' (signed, green/red).
function renderDistBars(el, rows, field) {
    if (!el) return;
    if (!Array.isArray(rows) || rows.length === 0) {
        el.innerHTML = `<div class="muted">${esc(t('view.reports.hint.no_data'))}</div>`;
        return;
    }
    const signed = field === 'net_pnl';
    const vals = rows.map(r => Number(r[field]) || 0);
    const max  = Math.max(...vals.map(Math.abs), 1);
    const fmtVal = signed ? fmtMoney : (n) => String(n);
    el.innerHTML = rows.map((r, i) => {
        const v = vals[i];
        const pct = Math.round(Math.abs(v) / max * 100);
        const cls = signed
            ? (v > 0 ? 'pos' : v < 0 ? 'neg' : 'zero')
            : 'neutral';
        return `<div class="dist-row">
            <div class="dist-label">${esc(String(r.key))}</div>
            <div class="dist-track">
                <div class="dist-fill ${cls}" data-bar-pct="${pct}"></div>
            </div>
            <div class="dist-value ${signed ? pnlClass(v) : ''}">${esc(fmtVal(v))}</div>
        </div>`;
    }).join('');
}

// ----------------------------------------------------------------------------
// Detailed tab — chooses one of the single-dimension reports.
// ----------------------------------------------------------------------------
async function renderDetailedTab(mount, state, sub, apiF, style, tok) {
    const acct = state.accountId;
    const body = mount.querySelector('#report-body');
    const setBody = (html) => {
        if (!viewIsCurrent(tok)) return null;
        const el = mount.querySelector('#report-body');
        if (el) el.innerHTML = html;
        return el;
    };
    if (sub === 'by-symbol')        setBody(bucketTable(await api.bySymbol(acct, apiF), t('view.reports.col.symbol'), style));
    else if (sub === 'by-side')     setBody(bucketTable(await api.bySide(acct, apiF), t('view.reports.col.side'), style));
    else if (sub === 'by-asset')    setBody(bucketTable(await api.byAssetClass(acct, apiF), t('view.reports.col.asset'), style));
    else if (sub === 'by-dow')      setBody(bucketTable(await api.byDow(acct, apiF), t('view.reports.col.dow'), style));
    else if (sub === 'by-hour')     setBody(bucketTable(await api.byHour(acct, apiF), t('view.reports.col.hour'), style));
    else if (sub === 'by-hold')     setBody(bucketTable(await api.byHold(acct, apiF), t('view.reports.col.hold'), style));
    else if (sub === 'by-month')    setBody(bucketTable(await api.byMonth(acct, apiF), t('view.reports.col.month'), style));
    else if (sub === 'by-price')    setBody(bucketTable(await api.byPrice(acct, apiF), t('view.reports.col.price'), style));
    else if (sub === 'r-dist') {
        const dist = await api.rDist(acct, apiF);
        if (!viewIsCurrent(tok)) return;
        const b = mount.querySelector('#report-body');
        if (b) renderRDist(b, dist, mount);
    }
    else if (sub === 'streaks')      setBody(streaksHtml(await api.streaks(acct, apiF)));
    else if (sub === 'comparison')   setBody(comparisonHtml(await api.comparison(acct, apiF)));
    else if (sub === 'exit-eff')     setBody(exitEffHtml(await api.exitEff(acct, apiF)));
    else if (sub === 'commissions')  setBody(commissionsHtml(await api.commissions(acct, apiF)));
    else if (sub === 'liquidity')    setBody(liquidityHtml(await api.liquidity(acct)));
    else if (sub === 'risk')         setBody(riskHtml(await api.risk(acct, apiF)));
    else if (sub === 'risk-adjusted') setBody(riskAdjustedHtml(await api.riskAdjusted(acct, undefined, apiF)));
}

// ----------------------------------------------------------------------------
// Compare tab — A vs B side-by-side using two filter sets.
// ----------------------------------------------------------------------------
const COMPARE_KEY_A = 'reports_compare_a_v1';
const COMPARE_KEY_B = 'reports_compare_b_v1';
function loadCompare(key, fallback) {
    try { return { ...FILTER_DEFAULTS, ...JSON.parse(sessionStorage.getItem(key) || '{}'), ...fallback }; }
    catch (_) { return { ...FILTER_DEFAULTS, ...fallback }; }
}
function saveCompare(key, f) { sessionStorage.setItem(key, JSON.stringify(f)); }

async function renderCompareTab(mount, state, tok) {
    const body = mount.querySelector('#report-body');
    if (!body) return;
    const fa = loadCompare(COMPARE_KEY_A, { side: 'long'  });
    const fb = loadCompare(COMPARE_KEY_B, { side: 'short' });
    body.innerHTML = `
        <div class="cmp-filter-pair">
            ${cmpFilterColumn('A', '#ffd84a', fa)}
            ${cmpFilterColumn('B', '#3aa1ff', fb)}
        </div>
        <div id="cmp-summary" class="panel-grid"></div>
        <div class="panel-grid">
            <div class="chart-panel"><h2 data-i18n="view.reports.cmp.by_dow">By Day of Week</h2><div id="cmp-dow"   class="chart-h-240"></div></div>
            <div class="chart-panel"><h2 data-i18n="view.reports.cmp.by_hour">By Hour</h2>      <div id="cmp-hour"  class="chart-h-240"></div></div>
            <div class="chart-panel"><h2 data-i18n="view.reports.cmp.by_hold">By Hold Time</h2> <div id="cmp-hold"  class="chart-h-240"></div></div>
            <div class="chart-panel"><h2 data-i18n="view.reports.cmp.by_price">By Price</h2>    <div id="cmp-price" class="chart-h-240"></div></div>
        </div>
    `;
    wireCompareFilters(mount, state, 'A', fa, COMPARE_KEY_A);
    wireCompareFilters(mount, state, 'B', fb, COMPARE_KEY_B);

    const [oA, oB, dowA, dowB, hourA, hourB, holdA, holdB, priceA, priceB] = await Promise.all([
        api.overview(state.accountId, filterForApi(fa)),
        api.overview(state.accountId, filterForApi(fb)),
        api.byDow(state.accountId, filterForApi(fa)),
        api.byDow(state.accountId, filterForApi(fb)),
        api.byHour(state.accountId, filterForApi(fa)),
        api.byHour(state.accountId, filterForApi(fb)),
        api.byHold(state.accountId, filterForApi(fa)),
        api.byHold(state.accountId, filterForApi(fb)),
        api.byPrice(state.accountId, filterForApi(fa)),
        api.byPrice(state.accountId, filterForApi(fb)),
    ]);
    if (!viewIsCurrent(tok)) return;

    const summary = body.querySelector('#cmp-summary');
    if (summary) summary.innerHTML = cmpSummaryCards(oA, oB);
    renderComparePair(body.querySelector('#cmp-dow'),   dowA, dowB);
    renderComparePair(body.querySelector('#cmp-hour'),  hourA, hourB);
    renderComparePair(body.querySelector('#cmp-hold'),  holdA, holdB);
    renderComparePair(body.querySelector('#cmp-price'), priceA, priceB);
}

function cmpFilterColumn(label, color, f) {
    const id = (k) => `cmp${label}-${k}`;
    return `
        <div class="cmp-filter-col cmp-filter-col-${label.toLowerCase()}">
            <h3>Filter ${label}</h3>
            <label><span>Symbol</span>
                <input type="text" id="${id('symbol')}" value="${esc(f.symbol)}" /></label>
            <label><span>Side</span>
                <select id="${id('side')}">
                    <option value=""        ${f.side === ''      ? 'selected' : ''}>All</option>
                    <option value="long"    ${f.side === 'long'  ? 'selected' : ''}>Long</option>
                    <option value="short"   ${f.side === 'short' ? 'selected' : ''}>Short</option>
                </select>
            </label>
            <label><span>Duration</span>
                <select id="${id('duration')}">
                    <option value=""          ${f.duration === ''         ? 'selected' : ''}>All</option>
                    <option value="intraday"  ${f.duration === 'intraday' ? 'selected' : ''}>Intraday</option>
                    <option value="multiday"  ${f.duration === 'multiday' ? 'selected' : ''}>Multiday</option>
                </select>
            </label>
            <label><span>From</span><input type="date" id="${id('from')}" value="${esc(f.date_from)}" /></label>
            <label><span>To</span>  <input type="date" id="${id('to')}"   value="${esc(f.date_to)}" /></label>
        </div>
    `;
}

function wireCompareFilters(mount, state, label, f, key) {
    const id = (k) => `#cmp${label}-${k}`;
    const apply = () => {
        const next = { ...f,
            symbol:    mount.querySelector(id('symbol')).value.trim().toUpperCase(),
            side:      mount.querySelector(id('side')).value,
            duration:  mount.querySelector(id('duration')).value,
            date_from: mount.querySelector(id('from')).value,
            date_to:   mount.querySelector(id('to')).value,
        };
        saveCompare(key, next);
        renderReports(mount, state, 'compare');
    };
    ['symbol','side','duration','from','to'].forEach(k => {
        const el = mount.querySelector(id(k));
        if (el) el.addEventListener('change', apply);
    });
}

function cmpSummaryCards(a, b) {
    const card = (label, va, vb, fmt, klass = (x) => '') => `
        <div class="cmp-stat-card">
            <div class="cmp-stat-label">${esc(label)}</div>
            <div class="cmp-stat-row"><span class="cmp-a-pip"></span><span class="${klass(va)}">${fmt(va)}</span></div>
            <div class="cmp-stat-row"><span class="cmp-b-pip"></span><span class="${klass(vb)}">${fmt(vb)}</span></div>
        </div>
    `;
    return [
        card(t('view.dashboard.stat.net_pnl'), a.net_pnl, b.net_pnl, fmtMoney, pnlClass),
        card(t('view.dashboard.stat.trades'),  a.trade_count, b.trade_count, x => x),
        card(t('view.dashboard.stat.win_rate'), a.win_rate, b.win_rate, fmtPct),
        card(t('view.dashboard.stat.profit_factor'), a.profit_factor, b.profit_factor, fmt),
        card(t('view.dashboard.stat.expectancy'), a.expectancy, b.expectancy, fmtMoney, pnlClass),
        card(t('view.dashboard.stat.avg_r'), a.avg_r, b.avg_r, fmt),
    ].join('');
}

function renderComparePair(el, aRows, bRows) {
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const keys = Array.from(new Set([
        ...(aRows || []).map(r => r.key),
        ...(bRows || []).map(r => r.key),
    ]));
    if (!keys.length) {
        el.innerHTML = `<div class="muted">${esc(t('view.reports.hint.no_data'))}</div>`;
        return;
    }
    const aMap = new Map((aRows || []).map(r => [r.key, Number(r.net_pnl) || 0]));
    const bMap = new Map((bRows || []).map(r => [r.key, Number(r.net_pnl) || 0]));
    const aY = keys.map(k => aMap.get(k) ?? 0);
    const bY = keys.map(k => bMap.get(k) ?? 0);
    const xs = keys.map((_, i) => i);
    const max = Math.max(...aY.map(Math.abs), ...bY.map(Math.abs), 1);
    const drawPair = (u) => {
        const ctx = u.ctx; ctx.save();
        const bw = Math.max(2, (u.bbox.width / xs.length) * 0.32);
        const yZero = u.valToPos(0, 'y', true);
        for (let i = 0; i < xs.length; i++) {
            const xc = u.valToPos(xs[i], 'x', true);
            const aPos = u.valToPos(aY[i], 'y', true);
            const bPos = u.valToPos(bY[i], 'y', true);
            ctx.fillStyle = '#ffd84a';
            ctx.fillRect(xc - bw - 1, Math.min(yZero, aPos), bw, Math.abs(aPos - yZero));
            ctx.fillStyle = '#3aa1ff';
            ctx.fillRect(xc + 1, Math.min(yZero, bPos), bw, Math.abs(bPos - yZero));
        }
        ctx.restore();
        return null;
    };
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 240,
        scales: { x: { time: false,}, y: { auto: true, range: [-max * 1.1, max * 1.1] } },
        series: [
            { label: 'idx' },
            { label: 'A',  stroke: 'transparent', paths: drawPair },
            { label: 'B',  stroke: 'transparent' },
        ],
        axes: [
            { stroke: '#aab', rotate: -45, size: 60,
              values: (_u, splits) => splits.map(v => keys[Math.round(v)] || '') },
            { stroke: '#aab', size: 64,
              values: (_u, ticks) => ticks.map(v => {
                  const ab = Math.abs(v); const sgn = v < 0 ? '-' : '';
                  if (ab >= 1e6) return `${sgn}$${(ab/1e6).toFixed(1)}M`;
                  if (ab >= 1e3) return `${sgn}$${(ab/1e3).toFixed(1)}K`;
                  return `${sgn}$${ab.toFixed(0)}`;
              }) },
        ],
        legend: { show: false },
    }, [xs, aY, bY], el);
}

// ----------------------------------------------------------------------------
// Tag Breakdown
// ----------------------------------------------------------------------------
function renderTagBreakdown(body, tags, style) {
    if (!tags || !tags.length) {
        body.innerHTML = `<p class="boot">${esc(t('view.reports.tag_break.empty'))}</p>`;
        return;
    }
    const sorted = [...tags].sort((a, b) => Number(b.net_pnl) - Number(a.net_pnl));
    const maxAbs = Math.max(...sorted.map(b => Math.abs(Number(b.net_pnl) || 0)), 1);
    body.innerHTML = `
        <div class="cards">
            ${statCard(t('view.reports.tag_break.tags'),  tags.length)}
            ${statCard(t('view.reports.tag_break.trades'), tags.reduce((s, t) => s + (t.trades||0), 0))}
            ${statCard(t('view.reports.tag_break.net_pnl'),
                       fmtMoney(tags.reduce((s, t) => s + Number(t.net_pnl||0), 0)),
                       pnlClass(tags.reduce((s, t) => s + Number(t.net_pnl||0), 0)))}
        </div>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.reports.tag_break.col.tag">Tag</th>
                <th data-i18n="view.reports.th.trades">Trades</th>
                <th data-i18n="view.reports.th.wins">Wins</th>
                <th data-i18n="view.reports.th.losses">Losses</th>
                <th data-i18n="view.reports.th.win">Win %</th>
                <th data-i18n="view.reports.th.net_p_l">Net P&L</th>
                <th data-i18n="view.reports.th.avg_p_l">Avg P&L</th>
                <th></th>
            </tr></thead>
            <tbody>${sorted.map(b => {
                const v = Number(b.net_pnl) || 0;
                const pct = (Math.abs(v) / maxAbs * 100).toFixed(0);
                return `<tr>
                    <td>${esc(b.key)}</td>
                    <td>${b.trades}</td>
                    <td>${b.wins}</td>
                    <td>${b.losses}</td>
                    <td>${fmtPct(b.win_rate)}</td>
                    <td class="${pnlClass(v)}">${fmtMoney(v)}</td>
                    <td class="${pnlClass(b.avg_pnl)}">${fmtMoney(b.avg_pnl)}</td>
                    <td><div class="tag-bar-track">
                        <div class="tag-bar-fill ${v >= 0 ? 'pos' : 'neg'}" data-bar-pct="${pct}"></div>
                    </div></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    applyBarWidths(body);
}

// ----------------------------------------------------------------------------
// Advanced — equity curve + per-trade scatter
// ----------------------------------------------------------------------------
function renderAdvanced(body, adv) {
    if (!adv || (!adv.cum_curve?.length && !adv.scatter?.length)) {
        body.innerHTML = `<p class="boot">${esc(t('view.reports.hint.no_data'))}</p>`;
        return;
    }
    body.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.reports.adv.cum_pnl">Cumulative P&L</h2>
            <div id="adv-eq" class="chart-h-280"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.adv.scatter">Per-Trade Scatter</h2>
            <div id="adv-scatter" class="chart-h-320"></div>
        </div>
    `;
    const eq = body.querySelector('#adv-eq');
    if (eq) equityChart(eq, adv.cum_curve);

    const sc = body.querySelector('#adv-scatter');
    if (sc && window.uPlot && adv.scatter?.length) {
        const xs = adv.scatter.map((_, i) => i);
        const ys = adv.scatter.map(p => Number(p.net_pnl) || 0);
        const points = adv.scatter;
        const drawDots = (u) => {
            const ctx = u.ctx; ctx.save();
            const yZero = u.valToPos(0, 'y', true);
            ctx.strokeStyle = 'rgba(255,255,255,0.04)';
            ctx.beginPath(); ctx.moveTo(0, yZero); ctx.lineTo(u.bbox.left + u.bbox.width, yZero); ctx.stroke();
            for (let i = 0; i < xs.length; i++) {
                const x = u.valToPos(xs[i], 'x', true);
                const y = u.valToPos(ys[i], 'y', true);
                ctx.fillStyle = points[i].win === false ? '#ff3860'
                              : points[i].win === true  ? '#39ff14'
                              : '#aab';
                ctx.beginPath();
                ctx.arc(x, y, 3, 0, Math.PI * 2);
                ctx.fill();
            }
            ctx.restore();
            return null;
        };
        new window.uPlot({
            title: '', width: sc.clientWidth || 800, height: 320,
            scales: { x: { time: false,}, y: { auto: true } },
            series: [
                { label: 'idx' },
                { label: 'P&L', stroke: 'transparent', paths: drawDots },
            ],
            axes: [
                { stroke: '#aab', size: 28,
                  values: (_u, splits) => splits.map(v => {
                      const p = points[Math.round(v)];
                      return p ? p.day.slice(5) : '';
                  }) },
                { stroke: '#aab', size: 64,
                  values: (_u, ticks) => ticks.map(v => {
                      const a = Math.abs(v); const sgn = v < 0 ? '-' : '';
                      if (a >= 1e6) return `${sgn}$${(a/1e6).toFixed(1)}M`;
                      if (a >= 1e3) return `${sgn}$${(a/1e3).toFixed(1)}K`;
                      return `${sgn}$${a.toFixed(0)}`;
                  }) },
            ],
            legend: { show: false },
        }, [xs, ys], sc);
    }
}

// ----------------------------------------------------------------------------
// Calendar year-grid (12 mini-cals on one page)
// ----------------------------------------------------------------------------
function renderCalendarYearGrid(body, cal) {
    if (!cal || !cal.length) {
        body.innerHTML = `<p class="boot">${esc(t('view.calendar.hint.no_data_yet'))}</p>`;
        return;
    }
    const byDay = new Map(cal.map(c => [c.day, c]));
    const years = [...new Set(cal.map(c => Number(c.day.slice(0, 4))))].sort();
    const year = years[years.length - 1];
    const MONTHS = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
    const DOW = ['S','M','T','W','T','F','S'];
    const monthHtml = (m) => {
        const first = new Date(year, m, 1);
        const daysInMo = new Date(year, m + 1, 0).getDate();
        const start = first.getDay();
        let monthPnl = 0;
        let cells = '';
        for (let i = 0; i < start; i++) cells += `<div class="ycal-cell empty"></div>`;
        for (let d = 1; d <= daysInMo; d++) {
            const key = `${year}-${String(m + 1).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
            const c = byDay.get(key);
            const v = Number(c?.net_pnl) || 0;
            const trades = Number(c?.trades) || 0;
            monthPnl += v;
            const cls = trades === 0 ? '' : v > 0 ? 'pos' : v < 0 ? 'neg' : '';
            cells += `<div class="ycal-cell ${cls}" data-day="${key}" role="button" tabindex="0"
                            title="${esc(key)}: ${fmtMoney(v)}">${d}</div>`;
        }
        return `
            <div class="ycal-month">
                <div class="ycal-month-head">
                    <strong>${MONTHS[m]}, ${year}</strong>
                    <span class="${pnlClass(monthPnl)}">${fmtMoney(monthPnl)}</span>
                </div>
                <div class="ycal-grid">
                    ${DOW.map(d => `<div class="ycal-dow">${d}</div>`).join('')}
                    ${cells}
                </div>
            </div>
        `;
    };
    body.innerHTML = `
        <div class="ycal-year-header">${year}</div>
        <div class="ycal-row">${[0,1,2,3,4,5,6,7,8,9,10,11].map(monthHtml).join('')}</div>
    `;
    body.querySelectorAll('.ycal-cell[data-day]').forEach(el => {
        const go = () => {
            const day = el.getAttribute('data-day');
            if (day) window.location.hash = `journal/${day}`;
        };
        el.addEventListener('click', go);
        el.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                go();
            }
        });
    });
}

// ----------------------------------------------------------------------------
// Saved filter sets (server-side via /filter-sets endpoints)
// ----------------------------------------------------------------------------
async function populateSavedSets(mount, state) {
    try {
        const sets = await api.listFilters?.();
        if (!Array.isArray(sets)) return;
        const sel = mount.querySelector('#rfb-set');
        if (!sel) return;
        for (const s of sets) {
            const opt = document.createElement('option');
            opt.value = s.id;
            opt.textContent = s.name;
            opt.dataset.payload = JSON.stringify(s.payload || {});
            sel.appendChild(opt);
        }
        sel.addEventListener('change', () => {
            const opt = sel.selectedOptions[0];
            if (!opt || !opt.value) return;
            try {
                const payload = JSON.parse(opt.dataset.payload || '{}');
                saveFilter({ ...FILTER_DEFAULTS, ...payload });
                renderReports(mount, state, currentPrimary());
            } catch (_) { /* noop */ }
        });
    } catch (_) { /* noop */ }
}

async function saveCurrentFilterSet(mount, state, filter) {
    const name = await tPrompt('view.reports.filter.save_prompt');
    if (!name) return;
    try {
        await api.saveFilter?.(name, filter, false);
        // Re-render so the set picker shows the new entry.
        renderReports(mount, state, currentPrimary());
    } catch (e) {
        showToast(t('view.reports.filter.save_err', { msg: e.message }), { level: 'error' });
    }
}

// ----------------------------------------------------------------------------
// Helpers that didn't change
// ----------------------------------------------------------------------------
function bucketTable(rows, header, style) {
    if (!rows.length) return '<p data-i18n="view.reports.hint.no_data" class="boot">No data.</p>';
    const useGross = style && style.pnl_type === 'gross';
    const asPct    = style && style.view_mode === 'pct';
    const total    = rows.reduce((s, b) => s + Math.abs(Number(useGross ? b.gross_pnl : b.net_pnl) || 0), 0) || 1;
    return `
        <table class="trades">
        <thead><tr><th>${esc(header)}</th><th data-i18n="view.reports.th.trades">Trades</th><th data-i18n="view.reports.th.wins">Wins</th><th data-i18n="view.reports.th.losses">Losses</th>
        <th data-i18n="view.reports.th.win">Win%</th><th>${esc(useGross ? t('view.reports.stat.gross_pnl') : t('view.dashboard.stat.net_pnl'))}</th><th data-i18n="view.reports.th.avg_p_l">Avg P&L</th></tr></thead>
        <tbody>${rows.map(b => {
            const pnl = Number(useGross ? b.gross_pnl : b.net_pnl) || 0;
            const cell = asPct ? `${(Math.abs(pnl) / total * 100).toFixed(2)}%` : fmtMoney(pnl);
            return `<tr><td>${esc(b.key)}</td><td>${b.trades}</td><td>${b.wins}</td><td>${b.losses}</td>
                <td>${fmtPct(b.win_rate)}</td>
                <td class="${pnlClass(pnl)}">${cell}</td>
                <td class="${pnlClass(b.avg_pnl)}">${fmtMoney(b.avg_pnl)}</td></tr>`;
        }).join('')}</tbody></table>`;
}

function renderRDist(body, dist, mount) {
    body.innerHTML = `
        <div class="cards">
            ${statCard(t('view.reports.stat.trades_with_r'), dist.trades_with_r)}
            ${statCard(t('view.dashboard.stat.avg_r'), fmt(dist.avg_r))}
            ${statCard(t('view.reports.stat.median_r'), fmt(dist.median_r))}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.h2.r_multiple_distribution">R-Multiple Distribution</h2>
            <div id="r-chart"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.h2.r_chart">R-bin count (uPlot)</h2>
            <div id="r-uplot" class="chart-h-240"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.h2.r_cumcount_chart">R-bin cumulative count</h2>
            <div id="r-cum-chart" class="chart-h-200"></div>
        </div>`;
    const chart = mount.querySelector('#r-chart');
    if (!chart) return;
    barChart(
        chart,
        dist.bins.map(b => b.label),
        dist.bins.map(b => b.count),
        { color: '#00e5ff', yKind: 'count' }
    );
    renderRBinsChart(dist.bins);
    renderRCumChart(dist.bins);
}

function renderRCumChart(bins) {
    const el = document.getElementById('r-cum-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (bins || []).filter(b => Number.isFinite(Number(b.count)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted">${esc(t('view.reports.empty_cum_chart'))}</div>`;
        return;
    }
    const labels = rows.map(b => b.label);
    const xs = labels.map((_, i) => i + 1);
    let running = 0;
    const ys = rows.map(b => { running += Number(b.count); return running; });
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.reports.chart.bin') },
            { label: t('view.reports.chart.cum_count'),
              stroke: '#00e5ff', width: 1.5,
              points: { show: true, size: 8, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderRBinsChart(bins) {
    const el = document.getElementById('r-uplot');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!bins || !bins.length) {
        el.innerHTML = `<div class="muted">${esc(t('view.reports.empty_chart'))}</div>`;
        return;
    }
    const labels = bins.map(b => b.label);
    const ys = bins.map(b => Number(b.count));
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.reports.chart.bin_idx') },
            { label: t('view.reports.chart.count'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function streaksHtml(streaks) {
    if (!streaks.length) return '<p data-i18n="view.reports.hint.no_streaks_yet" class="boot">No streaks yet.</p>';
    return `<table class="trades">
        <thead><tr><th data-i18n="view.reports.th.kind">Kind</th><th data-i18n="view.reports.th.length">Length</th><th data-i18n="view.reports.th.net_p_l_2">Net P&L</th><th data-i18n="view.reports.th.start">Start</th><th data-i18n="view.reports.th.end">End</th></tr></thead>
        <tbody>${streaks.map(s => `
            <tr><td class="${s.kind === 'win' ? 'pos' : 'neg'}">${s.kind}</td>
            <td>${s.length}</td>
            <td class="${pnlClass(s.net_pnl)}">${fmtMoney(s.net_pnl)}</td>
            <td>${fmtDate(s.start)}</td><td>${fmtDate(s.end)}</td></tr>
        `).join('')}</tbody></table>`;
}

function comparisonHtml(c) {
    return `<div class="cards">
        ${statCard(t('view.reports.cmp.long_trades'),  c.long.trades)}
        ${statCard(t('view.reports.cmp.long_net'),     fmtMoney(c.long.net_pnl), pnlClass(c.long.net_pnl))}
        ${statCard(t('view.reports.cmp.long_win_pct'), fmtPct(c.long.win_rate))}
        ${statCard(t('view.reports.cmp.short_trades'), c.short.trades)}
        ${statCard(t('view.reports.cmp.short_net'),    fmtMoney(c.short.net_pnl), pnlClass(c.short.net_pnl))}
        ${statCard(t('view.reports.cmp.short_win_pct'), fmtPct(c.short.win_rate))}
    </div>
    <div class="cards">
        ${statCard(t('view.reports.stat.avg_win'),  fmtMoney(c.wins.avg_pnl), 'pos')}
        ${statCard(t('view.reports.stat.avg_loss'), fmtMoney(c.losses.avg_pnl), 'neg')}
        ${statCard(t('view.reports.cmp.win_avg_hold'),  fmtSecs(c.wins.avg_hold_seconds))}
        ${statCard(t('view.reports.cmp.loss_avg_hold'), fmtSecs(c.losses.avg_hold_seconds))}
        ${statCard(t('view.reports.cmp.win_avg_qty'), fmt(c.wins.avg_qty, 0))}
        ${statCard(t('view.reports.cmp.loss_avg_qty'), fmt(c.losses.avg_qty, 0))}
    </div>`;
}

function exitEffHtml(e) {
    return `<div class="cards">
        ${statCard(t('view.reports.exit.avg_efficiency'),   fmtPct(e.avg_efficiency))}
        ${statCard(t('view.reports.exit.trades_with_data'), e.trades_with_data)}
        ${statCard(t('view.reports.exit.missed_pnl'),       fmtMoney(e.missed_pnl), 'neg')}
    </div>
    ${bucketTable(e.by_symbol, t('view.reports.col.symbol'))}`;
}

function commissionsHtml(c) {
    return `<div class="cards">
        ${statCard(t('view.reports.fee.total'),         fmtMoney(c.total_fees))}
        ${statCard(t('view.reports.fee.pct_of_gross'),  fmtPct(c.fees_pct_of_gross))}
        ${statCard(t('view.reports.fee.avg_per_trade'), fmtMoney(c.avg_fee_per_trade))}
        ${statCard(t('view.reports.fee.avg_per_unit'),  fmtMoney(c.avg_fee_per_unit))}
    </div>${bucketTable(c.by_symbol, t('view.reports.col.symbol'))}`;
}

function liquidityHtml(l) {
    const r = l.report;
    return `<div class="cards">
        ${l.report.buckets.map(b => statCard(b.label, `${b.trades} · ${fmtPct(b.win_rate)}`,
            pnlClass(b.net_pnl))).join('')}
    </div>
    <table class="trades">
        <thead><tr><th data-i18n="view.reports.th.symbol">Symbol</th><th data-i18n="view.reports.th.trades_2">Trades</th><th data-i18n="view.reports.th.avg_qty">Avg qty</th>
        <th data-i18n="view.reports.th.avg_daily_vol">Avg daily vol</th><th data-i18n="view.reports.th.avg_of_adv">Avg % of ADV</th><th data-i18n="view.reports.th.net_p_l_3">Net P&L</th></tr></thead>
        <tbody>${r.rows.map(row => `
            <tr><td>${esc(row.symbol)}</td><td>${row.trades}</td>
            <td>${fmt(row.avg_qty_per_trade, 0)}</td>
            <td>${row.avg_daily_volume !== null ? fmt(row.avg_daily_volume, 0) : '—'}</td>
            <td>${row.avg_pct_of_adv !== null ? fmtPct(row.avg_pct_of_adv) : '—'}</td>
            <td class="${pnlClass(row.net_pnl)}">${fmtMoney(row.net_pnl)}</td></tr>
        `).join('')}</tbody></table>
    <p class="muted" data-i18n-html="view.reports.adv_hint">Pass <code>?adv=AAPL:50000000,TSLA:80000000</code> to populate ADV columns.</p>`;
}

function riskHtml(r) {
    return `<div class="cards">
        ${statCard(t('view.reports.stat.trades_with_r'), r.trades_with_r)}
        ${statCard(t('view.dashboard.stat.avg_r'),       fmt(r.avg_r))}
        ${statCard(t('view.reports.risk.max_r'),         fmt(r.max_r))}
        ${statCard(t('view.reports.risk.min_r'),         fmt(r.min_r))}
        ${statCard(t('view.reports.risk.expectancy_r'),  fmt(r.expectancy_r))}
    </div>
    <p data-i18n="view.reports.hint.r_multiple_net_p_l_risk_amount_populate_stop_loss_" class="muted">R-multiple = net P&L / risk amount. Populate stop_loss + risk_amount on each trade to get these numbers.</p>`;
}

function drawdownHtml(dd) {
    return `<div class="cards">
        ${statCard(t('view.reports.dd.max'),     fmtMoney(dd.max_dd), 'neg')}
        ${statCard(t('view.reports.dd.max_pct'), fmtPct(dd.max_dd_pct))}
        ${statCard(t('view.reports.dd.peak'),    fmtDate(dd.peak_day))}
        ${statCard(t('view.reports.dd.trough'),  fmtDate(dd.trough_day))}
    </div>
    <div class="chart-panel">
        <h2 data-i18n="view.reports.h2.equity_drawdown">Equity + Drawdown</h2>
        <div id="eq-mount"></div>
    </div>`;
}

function riskAdjustedHtml(ra) {
    const ann = (v) => v * Math.sqrt(252);
    return `<div class="cards">
        ${statCard(t('view.reports.ra.sharpe_daily'),  fmt(ra.sharpe))}
        ${statCard(t('view.reports.ra.sharpe_ann'),    fmt(ann(ra.sharpe)))}
        ${statCard(t('view.reports.ra.sortino_daily'), fmt(ra.sortino))}
        ${statCard(t('view.reports.ra.sortino_ann'),   fmt(ann(ra.sortino)))}
        ${statCard(t('view.reports.ra.mean_daily'),    fmtMoney(ra.mean_daily))}
        ${statCard(t('view.reports.ra.stdev_daily'),   fmtMoney(ra.stdev_daily))}
        ${statCard(t('view.reports.ra.downside_stdev'), fmtMoney(ra.downside_stdev_daily))}
    </div>
    <p data-i18n="view.reports.hint.annualized_values_assume_252_trading_days_year_and" class="muted">Annualized values assume 252 trading days/year and rf = 0.</p>`;
}

// ---------- Year / Month / Day (unchanged) ----------------------------------
function renderYearMonthDay(body, monthly, cal) {
    const years = new Map();
    for (const m of monthly || []) {
        const y = String(m.key || '').slice(0, 4);
        if (!y || y.length !== 4) continue;
        const acc = years.get(y) || { key: y, trades: 0, wins: 0, losses: 0, net_pnl: 0 };
        acc.trades += Number(m.trades) || 0;
        acc.wins   += Number(m.wins)   || 0;
        acc.losses += Number(m.losses) || 0;
        acc.net_pnl += Number(m.net_pnl) || 0;
        years.set(y, acc);
    }
    const yearRows = [...years.values()].sort((a, b) => a.key.localeCompare(b.key));
    if (!yearRows.length) {
        body.innerHTML = '<p class="boot">' + esc(t('view.reports.hint.no_data')) + '</p>';
        return;
    }
    const selectedYear = yearRows[yearRows.length - 1].key;

    body.innerHTML = `
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.reports.ymd.trades_year">Trade Distribution By Year</h2>
                <div id="ymd-trades-year" class="chart-h-240"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.reports.ymd.perf_year">Performance By Year</h2>
                <div id="ymd-perf-year" class="chart-h-240"></div>
            </div>
        </div>
        <h2 class="view-title vt-mt-16"><span id="ymd-year-label">${esc(selectedYear)}</span></h2>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.reports.ymd.trades_month">Trade Distribution By Month</h2>
                <div id="ymd-trades-month" class="chart-h-240"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.reports.ymd.perf_month">Performance By Month</h2>
                <div id="ymd-perf-month" class="chart-h-240"></div>
            </div>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.reports.ymd.trades_day">Trade Distribution By Day</h2>
                <div id="ymd-trades-day" class="chart-h-240"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.reports.ymd.perf_day">Performance By Day</h2>
                <div id="ymd-perf-day" class="chart-h-240"></div>
            </div>
        </div>
    `;
    barChart(body.querySelector('#ymd-trades-year'),
        yearRows.map(r => r.key), yearRows.map(r => r.trades),
        { color: '#39ff14', yKind: 'count', seriesLabel: t('view.reports.ymd.trades_year') });
    barChart(body.querySelector('#ymd-perf-year'),
        yearRows.map(r => r.key), yearRows.map(r => Number(r.net_pnl) || 0),
        { color: '#39ff14', yKind: 'money', seriesLabel: t('view.reports.ymd.perf_year') });
    const monthsInYear = Array.from({ length: 12 }, (_, i) => {
        const key = `${selectedYear}-${String(i + 1).padStart(2, '0')}`;
        const row = (monthly || []).find(m => m.key === key);
        return {
            key: ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'][i],
            trades: row ? Number(row.trades) : 0,
            net_pnl: row ? Number(row.net_pnl) : 0,
        };
    });
    barChart(body.querySelector('#ymd-trades-month'),
        monthsInYear.map(r => r.key), monthsInYear.map(r => r.trades),
        { color: '#39ff14', yKind: 'count', seriesLabel: t('view.reports.ymd.trades_month') });
    barChart(body.querySelector('#ymd-perf-month'),
        monthsInYear.map(r => r.key), monthsInYear.map(r => r.net_pnl),
        { color: '#39ff14', yKind: 'money', seriesLabel: t('view.reports.ymd.perf_month') });
    const days = (cal || []).filter(c => c.day && c.day.startsWith(selectedYear));
    barChart(body.querySelector('#ymd-trades-day'),
        days.map(d => d.day), days.map(d => Number(d.trades) || 0),
        { color: '#39ff14', yKind: 'count', seriesLabel: t('view.reports.ymd.trades_day') });
    barChart(body.querySelector('#ymd-perf-day'),
        days.map(d => d.day), days.map(d => Number(d.net_pnl) || 0),
        { color: '#39ff14', yKind: 'money', seriesLabel: t('view.reports.ymd.perf_day') });
}

// ---------- Win vs Loss Days (unchanged) ------------------------------------
function renderWinLossDays(body, wld) {
    if (!wld) {
        body.innerHTML = '<p class="boot">' + esc(t('view.reports.hint.no_data')) + '</p>';
        return;
    }
    body.innerHTML = `
        <h2 class="view-title vt-mt-0">Win vs Loss Days</h2>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2>Trade Distribution By Day Of Week</h2>
                <div id="wld-dow-w" class="chart-h-240"></div>
            </div>
            <div class="chart-panel">
                <h2>Performance By Day Of Week</h2>
                <div id="wld-dow-l" class="chart-h-240"></div>
            </div>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2>Trade Distribution By Hour</h2>
                <div id="wld-hour-w" class="chart-h-240"></div>
            </div>
            <div class="chart-panel">
                <h2>Performance By Hour</h2>
                <div id="wld-hour-l" class="chart-h-240"></div>
            </div>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2>Trade Distribution By Hold Time</h2>
                <div id="wld-hold-w" class="chart-h-240"></div>
            </div>
            <div class="chart-panel">
                <h2>Performance By Hold Time</h2>
                <div id="wld-hold-l" class="chart-h-240"></div>
            </div>
        </div>
    `;
    renderWinLossPair(body.querySelector('#wld-dow-w'),  wld.by_dow,  'trades');
    renderWinLossPair(body.querySelector('#wld-dow-l'),  wld.by_dow,  'net_pnl');
    renderWinLossPair(body.querySelector('#wld-hour-w'), wld.by_hour, 'trades');
    renderWinLossPair(body.querySelector('#wld-hour-l'), wld.by_hour, 'net_pnl');
    renderWinLossPair(body.querySelector('#wld-hold-w'), wld.by_hold, 'trades');
    renderWinLossPair(body.querySelector('#wld-hold-l'), wld.by_hold, 'net_pnl');
}

function renderWinLossPair(el, split, valKey) {
    if (!el || !split) return;
    const keys = Array.from(new Set([
        ...split.winning_days.map(b => b.key),
        ...split.losing_days.map(b => b.key),
    ]));
    const winMap  = new Map(split.winning_days.map(b => [b.key, Number(b[valKey]) || 0]));
    const lossMap = new Map(split.losing_days.map(b => [b.key, Number(b[valKey]) || 0]));
    const isMoney = valKey === 'net_pnl';
    el.innerHTML = '';
    if (!window.uPlot) { el.textContent = 'chart unavailable'; return; }
    const xs = keys.map((_, i) => i);
    const winY  = keys.map(k => winMap.get(k)  ?? 0);
    const lossY = keys.map(k => lossMap.get(k) ?? 0);
    const max = Math.max(...winY.map(Math.abs), ...lossY.map(Math.abs), 1);
    const drawPair = (u) => {
        const ctx = u.ctx; ctx.save();
        const bw = Math.max(2, (u.bbox.width / xs.length) * 0.32);
        const yZero = u.valToPos(0, 'y', true);
        for (let i = 0; i < xs.length; i++) {
            const xc = u.valToPos(xs[i], 'x', true);
            const wY = u.valToPos(winY[i], 'y', true);
            const lY = u.valToPos(lossY[i], 'y', true);
            ctx.fillStyle = '#ffd84a';
            ctx.fillRect(xc - bw - 1, Math.min(yZero, wY), bw, Math.abs(wY - yZero));
            ctx.fillStyle = '#3aa1ff';
            ctx.fillRect(xc + 1, Math.min(yZero, lY), bw, Math.abs(lY - yZero));
        }
        ctx.restore();
        return null;
    };
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 240,
        scales: { x: { time: false,}, y: { auto: true, range: [-max * 1.1, max * 1.1] } },
        series: [
            { label: 'idx' },
            { label: 'win days',  stroke: 'transparent', paths: drawPair },
            { label: 'loss days', stroke: 'transparent' },
        ],
        axes: [
            { stroke: '#aab', rotate: -45, size: 60,
              values: (_u, splits) => splits.map(v => keys[Math.round(v)] || '') },
            { stroke: '#aab', size: 64,
              values: (_u, ticks) => ticks.map(v => {
                  if (!isMoney) return v.toFixed(0);
                  const a = Math.abs(v); const sgn = v < 0 ? '-' : '';
                  if (a >= 1e6) return `${sgn}$${(a/1e6).toFixed(1)}M`;
                  if (a >= 1e3) return `${sgn}$${(a/1e3).toFixed(1)}K`;
                  return `${sgn}$${a.toFixed(0)}`;
              }) },
        ],
        legend: { show: false },
    }, [xs, winY, lossY], el);
}
