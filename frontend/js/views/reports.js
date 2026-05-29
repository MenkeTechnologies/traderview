import { api } from '../api.js';
import { fmt, fmtMoney, fmtPct, fmtDate, fmtSecs, pnlClass, esc, statCard } from '../util.js';
import { barChart, equityChart } from '../charts.js';
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
            ${TABS.map(([k, l]) => `<a class="report-tab ${k === sub ? 'active' : ''}" href="#reports/${k}">${l}</a>`).join('')}
        </div>
        <div id="report-body"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text">loading…</div></div></div>
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
        } else if (sub === 'by-symbol')  setBody(bucketTable(await api.bySymbol(acct), 'Symbol'));
        else if (sub === 'by-side')      setBody(bucketTable(await api.bySide(acct), 'Side'));
        else if (sub === 'by-asset')     setBody(bucketTable(await api.byAssetClass(acct), 'Asset'));
        else if (sub === 'by-dow')       setBody(bucketTable(await api.byDow(acct), 'DoW'));
        else if (sub === 'by-hour')      setBody(bucketTable(await api.byHour(acct), 'Hour'));
        else if (sub === 'by-hold')      setBody(bucketTable(await api.byHold(acct), 'Hold'));
        else if (sub === 'by-month')     setBody(bucketTable(await api.byMonth(acct), 'Month'));
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
        setBody(`<p class="boot">Error: ${e.message}</p>`);
    }
}

function overviewHtml(s) {
    return `<div class="cards">
        ${statCard('Net P&L', fmtMoney(s.net_pnl), pnlClass(s.net_pnl))}
        ${statCard('Gross P&L', fmtMoney(s.gross_pnl), pnlClass(s.gross_pnl))}
        ${statCard('Trades', s.trade_count)}
        ${statCard('Wins / Losses / Scratch', `${s.win_count} / ${s.loss_count} / ${s.scratch_count}`)}
        ${statCard('Win rate', fmtPct(s.win_rate))}
        ${statCard('Profit factor', fmt(s.profit_factor))}
        ${statCard('Expectancy', fmtMoney(s.expectancy), pnlClass(s.expectancy))}
        ${statCard('Avg win', fmtMoney(s.avg_win), 'pos')}
        ${statCard('Avg loss', fmtMoney(s.avg_loss), 'neg')}
        ${statCard('Largest win', fmtMoney(s.largest_win), 'pos')}
        ${statCard('Largest loss', fmtMoney(s.largest_loss), 'neg')}
        ${statCard('Max consec wins', s.max_consec_wins)}
        ${statCard('Max consec losses', s.max_consec_losses)}
        ${statCard('Avg hold', fmtSecs(s.avg_hold_seconds))}
        ${statCard('Avg win hold', fmtSecs(s.avg_win_hold_seconds))}
        ${statCard('Avg loss hold', fmtSecs(s.avg_loss_hold_seconds))}
        ${statCard('Avg R', fmt(s.avg_r))}
        ${statCard('Volume', fmtMoney(s.total_volume))}
        ${statCard('Fees', fmtMoney(s.fees))}
        ${statCard('Open', s.open_count)}
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
            ${statCard('Trades with R', dist.trades_with_r)}
            ${statCard('Avg R', fmt(dist.avg_r))}
            ${statCard('Median R', fmt(dist.median_r))}
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
        ${statCard('Long trades', c.long.trades)}
        ${statCard('Long net', fmtMoney(c.long.net_pnl), pnlClass(c.long.net_pnl))}
        ${statCard('Long win%', fmtPct(c.long.win_rate))}
        ${statCard('Short trades', c.short.trades)}
        ${statCard('Short net', fmtMoney(c.short.net_pnl), pnlClass(c.short.net_pnl))}
        ${statCard('Short win%', fmtPct(c.short.win_rate))}
    </div>
    <div class="cards">
        ${statCard('Avg win', fmtMoney(c.wins.avg_pnl), 'pos')}
        ${statCard('Avg loss', fmtMoney(c.losses.avg_pnl), 'neg')}
        ${statCard('Win avg hold', fmtSecs(c.wins.avg_hold_seconds))}
        ${statCard('Loss avg hold', fmtSecs(c.losses.avg_hold_seconds))}
        ${statCard('Win avg qty', fmt(c.wins.avg_qty, 0))}
        ${statCard('Loss avg qty', fmt(c.losses.avg_qty, 0))}
    </div>`;
}

function exitEffHtml(e) {
    return `<div class="cards">
        ${statCard('Avg efficiency', fmtPct(e.avg_efficiency))}
        ${statCard('Trades with data', e.trades_with_data)}
        ${statCard('Missed P&L', fmtMoney(e.missed_pnl), 'neg')}
    </div>
    ${bucketTable(e.by_symbol, 'Symbol')}`;
}

function commissionsHtml(c) {
    return `<div class="cards">
        ${statCard('Total fees', fmtMoney(c.total_fees))}
        ${statCard('Fees % of gross', fmtPct(c.fees_pct_of_gross))}
        ${statCard('Avg fee / trade', fmtMoney(c.avg_fee_per_trade))}
        ${statCard('Avg fee / unit', fmtMoney(c.avg_fee_per_unit))}
    </div>${bucketTable(c.by_symbol, 'Symbol')}`;
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
        ${statCard('Trades with R', r.trades_with_r)}
        ${statCard('Avg R', fmt(r.avg_r))}
        ${statCard('Max R', fmt(r.max_r))}
        ${statCard('Min R', fmt(r.min_r))}
        ${statCard('Expectancy R', fmt(r.expectancy_r))}
    </div>
    <p data-i18n="view.reports.hint.r_multiple_net_p_l_risk_amount_populate_stop_loss_" class="muted">R-multiple = net P&L / risk amount. Populate stop_loss + risk_amount on each trade to get these numbers.</p>`;
}

function drawdownHtml(dd) {
    return `<div class="cards">
        ${statCard('Max drawdown', fmtMoney(dd.max_dd), 'neg')}
        ${statCard('Max drawdown %', fmtPct(dd.max_dd_pct))}
        ${statCard('Peak day', fmtDate(dd.peak_day))}
        ${statCard('Trough day', fmtDate(dd.trough_day))}
    </div>
    <div class="chart-panel">
        <h2 data-i18n="view.reports.h2.equity_drawdown">Equity + Drawdown</h2>
        <div id="eq-mount"></div>
    </div>`;
}

function riskAdjustedHtml(ra) {
    const ann = (v) => v * Math.sqrt(252);
    return `<div class="cards">
        ${statCard('Sharpe (daily)', fmt(ra.sharpe))}
        ${statCard('Sharpe (ann.)', fmt(ann(ra.sharpe)))}
        ${statCard('Sortino (daily)', fmt(ra.sortino))}
        ${statCard('Sortino (ann.)', fmt(ann(ra.sortino)))}
        ${statCard('Mean daily', fmtMoney(ra.mean_daily))}
        ${statCard('Stdev daily', fmtMoney(ra.stdev_daily))}
        ${statCard('Downside stdev', fmtMoney(ra.downside_stdev_daily))}
    </div>
    <p data-i18n="view.reports.hint.annualized_values_assume_252_trading_days_year_and" class="muted">Annualized values assume 252 trading days/year and rf = 0.</p>`;
}
