import { api } from '../api.js';
import { fmt, fmtMoney, fmtPct, fmtDate, fmtSecs, pnlClass, esc, statCard } from '../util.js';
import { barChart, equityChart } from '../charts.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const TABS = [
    ['overview',       'Overview'],
    ['year-month-day', 'Year / Month / Day'],
    ['win-loss-days',  'Win vs Loss Days'],
    ['by-symbol',      'By Symbol'],
    ['by-side',        'By Side'],
    ['by-asset',       'By Asset'],
    ['by-dow',         'By Day of Week'],
    ['by-hour',        'By Hour'],
    ['by-hold',        'By Hold Time'],
    ['by-month',       'By Month'],
    ['r-dist',         'R-Multiple'],
    ['streaks',        'Streaks'],
    ['comparison',     'Comparison'],
    ['exit-eff',       'Exit Efficiency'],
    ['commissions',    'Commissions'],
    ['liquidity',      'Liquidity'],
    ['risk',           'Risk'],
    ['drawdown',       'Drawdown'],
    ['risk-adjusted',  'Sharpe / Sortino'],
];

export async function renderReports(mount, state, sub) {
    const tok = currentViewToken();
    if (!state.accountId) { mount.innerHTML = '<p data-i18n="view.reports.hint.no_account" class="boot">No account.</p>'; return; }
    if (!TABS.find(t => t[0] === sub)) sub = 'overview';
    mount.innerHTML = `
        <h1 data-i18n="view.reports.h1.reports" class="view-title">// REPORTS</h1>
        <div class="report-tabs">
            ${TABS.map(([k, l]) => `<a class="report-tab ${k === sub ? 'active' : ''}" href="#reports/${k}" data-i18n="view.reports.tab.${k}">${l}</a>`).join('')}
        </div>
        <div id="report-body"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
    `;
    const acct = state.accountId;
    // After each await, re-fetch the body element (caller may have replaced it).
    const setBody = (html) => {
        if (!viewIsCurrent(tok)) return null;
        const el = mount.querySelector('#report-body');
        if (el) el.innerHTML = html;
        return el;
    };
    try {
        if (sub === 'overview') {
            const s = await api.overview(acct); setBody(overviewHtml(s));
        } else if (sub === 'by-symbol')  setBody(bucketTable(await api.bySymbol(acct), t('view.reports.col.symbol')));
        else if (sub === 'by-side')      setBody(bucketTable(await api.bySide(acct), t('view.reports.col.side')));
        else if (sub === 'by-asset')     setBody(bucketTable(await api.byAssetClass(acct), t('view.reports.col.asset')));
        else if (sub === 'by-dow')       setBody(bucketTable(await api.byDow(acct), t('view.reports.col.dow')));
        else if (sub === 'by-hour')      setBody(bucketTable(await api.byHour(acct), t('view.reports.col.hour')));
        else if (sub === 'by-hold')      setBody(bucketTable(await api.byHold(acct), t('view.reports.col.hold')));
        else if (sub === 'by-month')     setBody(bucketTable(await api.byMonth(acct), t('view.reports.col.month')));
        else if (sub === 'year-month-day') {
            const [monthly, cal] = await Promise.all([api.byMonth(acct), api.calendar(acct)]);
            if (!viewIsCurrent(tok)) return;
            const body = mount.querySelector('#report-body');
            if (body) renderYearMonthDay(body, monthly, cal);
        }
        else if (sub === 'win-loss-days') {
            const wld = await api.winLossDays(acct);
            if (!viewIsCurrent(tok)) return;
            const body = mount.querySelector('#report-body');
            if (body) renderWinLossDays(body, wld);
        }
        else if (sub === 'r-dist') {
            const dist = await api.rDist(acct);
            if (!viewIsCurrent(tok)) return;
            const body = mount.querySelector('#report-body');
            if (body) renderRDist(body, dist, mount);
        }
        else if (sub === 'streaks')      setBody(streaksHtml(await api.streaks(acct)));
        else if (sub === 'comparison')   setBody(comparisonHtml(await api.comparison(acct)));
        else if (sub === 'exit-eff')     setBody(exitEffHtml(await api.exitEff(acct)));
        else if (sub === 'commissions')  setBody(commissionsHtml(await api.commissions(acct)));
        else if (sub === 'liquidity')    setBody(liquidityHtml(await api.liquidity(acct)));
        else if (sub === 'risk')         setBody(riskHtml(await api.risk(acct)));
        else if (sub === 'drawdown') {
            const [dd, eq] = await Promise.all([api.drawdown(acct), api.equity(acct)]);
            if (!viewIsCurrent(tok)) return;
            setBody(drawdownHtml(dd));
            const eqMount = mount.querySelector('#eq-mount');
            if (eqMount) equityChart(eqMount, eq);
        } else if (sub === 'risk-adjusted') setBody(riskAdjustedHtml(await api.riskAdjusted(acct)));
    } catch (e) {
        setBody(`<p class="boot">${esc(t('view.reports.error', { msg: e.message }))}</p>`);
    }
}

function overviewHtml(s) {
    return `<div class="cards">
        ${statCard(t('view.dashboard.stat.net_pnl'),   fmtMoney(s.net_pnl), pnlClass(s.net_pnl))}
        ${statCard(t('view.reports.stat.gross_pnl'),   fmtMoney(s.gross_pnl), pnlClass(s.gross_pnl))}
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
        ${statCard(t('view.dashboard.stat.avg_r'),     fmt(s.avg_r))}
        ${statCard(t('view.reports.stat.volume'),      fmtMoney(s.total_volume))}
        ${statCard(t('view.dashboard.stat.fees'),      fmtMoney(s.fees))}
        ${statCard(t('view.reports.stat.open'),        s.open_count)}
    </div>`;
}

function bucketTable(rows, header) {
    if (!rows.length) return '<p data-i18n="view.reports.hint.no_data" class="boot">No data.</p>';
    return `
        <table class="trades">
        <thead><tr><th>${esc(header)}</th><th data-i18n="view.reports.th.trades">Trades</th><th data-i18n="view.reports.th.wins">Wins</th><th data-i18n="view.reports.th.losses">Losses</th>
        <th data-i18n="view.reports.th.win">Win%</th><th data-i18n="view.reports.th.net_p_l">Net P&L</th><th data-i18n="view.reports.th.avg_p_l">Avg P&L</th></tr></thead>
        <tbody>${rows.map(b => `
            <tr><td>${esc(b.key)}</td><td>${b.trades}</td><td>${b.wins}</td><td>${b.losses}</td>
            <td>${fmtPct(b.win_rate)}</td>
            <td class="${pnlClass(b.net_pnl)}">${fmtMoney(b.net_pnl)}</td>
            <td class="${pnlClass(b.avg_pnl)}">${fmtMoney(b.avg_pnl)}</td></tr>
        `).join('')}</tbody></table>`;
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
            <div id="r-uplot" style="width:100%;height:240px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.reports.h2.r_cumcount_chart">R-bin cumulative count</h2>
            <div id="r-cum-chart" style="width:100%;height:200px"></div>
        </div>`;
    const chart = mount.querySelector('#r-chart');
    if (!chart) return;
    barChart(
        chart,
        dist.bins.map(b => b.label),
        dist.bins.map(b => b.count),
        { color: '#00e5ff' }
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
        el.innerHTML = `<div class="muted" data-i18n="view.reports.empty_cum_chart">${esc(t('view.reports.empty_cum_chart'))}</div>`;
        return;
    }
    const labels = rows.map(b => b.label);
    const xs = labels.map((_, i) => i + 1);
    let running = 0;
    const ys = rows.map(b => { running += Number(b.count); return running; });
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.reports.chart.bin') },
            { label: t('view.reports.chart.cum_count'),
              stroke: '#00e5ff', width: 1.5,
              points: { show: true, size: 8, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
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
        el.innerHTML = `<div class="muted" data-i18n="view.reports.empty_chart">${esc(t('view.reports.empty_chart'))}</div>`;
        return;
    }
    const labels = bins.map(b => b.label);
    const ys = bins.map(b => Number(b.count));
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.reports.chart.bin_idx') },
            { label: t('view.reports.chart.count'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
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

// ---------- Year / Month / Day (Tradervue parity) ----------------------------
function renderYearMonthDay(body, monthly, cal) {
    // monthly buckets carry key "YYYY-MM"; aggregate up to year + take a
    // year-picker (default = most recent year that has data).
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
                <div id="ymd-trades-year" style="width:100%;height:240px"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.reports.ymd.perf_year">Performance By Year</h2>
                <div id="ymd-perf-year" style="width:100%;height:240px"></div>
            </div>
        </div>
        <h2 class="view-title" style="margin-top:16px"><span id="ymd-year-label">${esc(selectedYear)}</span></h2>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.reports.ymd.trades_month">Trade Distribution By Month</h2>
                <div id="ymd-trades-month" style="width:100%;height:240px"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.reports.ymd.perf_month">Performance By Month</h2>
                <div id="ymd-perf-month" style="width:100%;height:240px"></div>
            </div>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.reports.ymd.trades_day">Trade Distribution By Day</h2>
                <div id="ymd-trades-day" style="width:100%;height:240px"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.reports.ymd.perf_day">Performance By Day</h2>
                <div id="ymd-perf-day" style="width:100%;height:240px"></div>
            </div>
        </div>
    `;

    barChart(body.querySelector('#ymd-trades-year'),
        yearRows.map(r => r.key),
        yearRows.map(r => r.trades),
        { color: '#39ff14', yKind: 'count', seriesLabel: t('view.reports.ymd.trades_year') });
    barChart(body.querySelector('#ymd-perf-year'),
        yearRows.map(r => r.key),
        yearRows.map(r => Number(r.net_pnl) || 0),
        { color: '#39ff14', yKind: 'money', seriesLabel: t('view.reports.ymd.perf_year') });

    // Monthly rows for the selected year, padded to 12 months.
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
        monthsInYear.map(r => r.key),
        monthsInYear.map(r => r.trades),
        { color: '#39ff14', yKind: 'count', seriesLabel: t('view.reports.ymd.trades_month') });
    barChart(body.querySelector('#ymd-perf-month'),
        monthsInYear.map(r => r.key),
        monthsInYear.map(r => r.net_pnl),
        { color: '#39ff14', yKind: 'money', seriesLabel: t('view.reports.ymd.perf_month') });

    // Daily breakdown comes from the calendar cells, filtered to selected year.
    const days = (cal || []).filter(c => c.day && c.day.startsWith(selectedYear));
    barChart(body.querySelector('#ymd-trades-day'),
        days.map(d => d.day),
        days.map(d => Number(d.trades) || 0),
        { color: '#39ff14', yKind: 'count', seriesLabel: t('view.reports.ymd.trades_day') });
    barChart(body.querySelector('#ymd-perf-day'),
        days.map(d => d.day),
        days.map(d => Number(d.net_pnl) || 0),
        { color: '#39ff14', yKind: 'money', seriesLabel: t('view.reports.ymd.perf_day') });
}

// ---------- Win vs Loss Days (Tradervue parity) ------------------------------
function renderWinLossDays(body, wld) {
    if (!wld) {
        body.innerHTML = '<p class="boot">' + esc(t('view.reports.hint.no_data')) + '</p>';
        return;
    }
    body.innerHTML = `
        <h2 class="view-title" style="margin-top:0">Win vs Loss Days</h2>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2>Trade Distribution By Day Of Week</h2>
                <div id="wld-dow-w" style="width:100%;height:240px"></div>
            </div>
            <div class="chart-panel">
                <h2>Performance By Day Of Week</h2>
                <div id="wld-dow-l" style="width:100%;height:240px"></div>
            </div>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2>Trade Distribution By Hour</h2>
                <div id="wld-hour-w" style="width:100%;height:240px"></div>
            </div>
            <div class="chart-panel">
                <h2>Performance By Hour</h2>
                <div id="wld-hour-l" style="width:100%;height:240px"></div>
            </div>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2>Trade Distribution By Hold Time</h2>
                <div id="wld-hold-w" style="width:100%;height:240px"></div>
            </div>
            <div class="chart-panel">
                <h2>Performance By Hold Time</h2>
                <div id="wld-hold-l" style="width:100%;height:240px"></div>
            </div>
        </div>
    `;
    renderWinLossPair(body.querySelector('#wld-dow-w'), wld.by_dow, 'trades');
    renderWinLossPair(body.querySelector('#wld-dow-l'), wld.by_dow, 'net_pnl');
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
            ctx.fillStyle = '#ffd84a'; // winning days = yellow
            ctx.fillRect(xc - bw - 1, Math.min(yZero, wY), bw, Math.abs(wY - yZero));
            ctx.fillStyle = '#3aa1ff'; // losing days = blue
            ctx.fillRect(xc + 1, Math.min(yZero, lY), bw, Math.abs(lY - yZero));
        }
        ctx.restore();
        return null;
    };
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 240,
        scales: { x: {}, y: { auto: true, range: [-max * 1.1, max * 1.1] } },
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
