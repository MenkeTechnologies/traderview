import { api } from '../api.js';
import { fmt, fmtMoney, fmtPct, fmtDate, fmtSecs, pnlClass, esc, statCard } from '../util.js';
import { barChart, equityChart } from '../charts.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const TABS = [
    ['overview',       'Overview'],
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
        </div>`;
    const chart = mount.querySelector('#r-chart');
    if (!chart) return;
    barChart(
        chart,
        dist.bins.map(b => b.label),
        dist.bins.map(b => b.count),
        { color: '#00e5ff' }
    );
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
