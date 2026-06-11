// Strategy tools — grid trading, fixed-ratio and anti-martingale
// sizing, dual momentum (GEM), vol cone, conversion/reversal arb,
// seagull spread, and turn-of-month / day-of-week seasonality behind
// one tabbed view. Each tool is a server-side compute; the
// tab/form/result chrome lives in tool_tabs.js.

import { api } from '../api.js';
import { esc } from '../util.js';
import { renderToolTabs } from './tool_tabs.js';

const TOOLS = {
    'grid-trading': {
        label: 'Grid Trading',
        call: (b) => api.calcGridTrading({ ...b, geometric: b.geometric === 1 }),
        fields: [
            { key: 'lower_price', label: 'Lower price ($)', def: 90 },
            { key: 'upper_price', label: 'Upper price ($)', def: 110 },
            { key: 'grid_count', label: 'Grid count', def: 20, int: true },
            { key: 'total_capital', label: 'Total capital ($)', def: 10000 },
            { key: 'fee_pct', label: 'Fee per side (%)', def: 0.1 },
            { key: 'geometric', label: 'Geometric spacing (1 = yes)', def: 0, int: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Profit per grid</div>
                    <div class="value ${r.profit_per_grid_min_pct > 0 ? 'pos' : 'neg'}">${r.profit_per_grid_min_pct.toFixed(2)}–${r.profit_per_grid_max_pct.toFixed(2)}%</div>
                    <div class="small muted">$${r.profit_per_grid_min_usd.toFixed(2)}–$${r.profit_per_grid_max_usd.toFixed(2)} per round trip</div></div>
                <div class="card"><div class="label">Capital per grid</div>
                    <div class="value">$${r.capital_per_grid.toFixed(2)}</div>
                    <div class="small muted">${r.levels.length} levels over ${r.range_pct.toFixed(1)}% range</div></div>
                <div class="card"><div class="label">Fee check</div>
                    <div class="value ${r.any_grid_unprofitable ? 'neg' : 'pos'}">${r.any_grid_unprofitable ? 'FEES EAT A GRID' : 'all grids clear fees'}</div></div>
            </div>
            <p class="muted small">Levels: ${r.levels.map(l => l.toFixed(2)).join(' · ')}</p>`,
    },
    'fixed-ratio': {
        label: 'Fixed Ratio Sizing',
        call: (b) => api.calcFixedRatio(b),
        fields: [
            { key: 'starting_capital', label: 'Starting capital ($)', def: 10000 },
            { key: 'delta', label: 'Delta per contract ($)', def: 5000 },
            { key: 'max_contracts', label: 'Max contracts', def: 10, int: true },
            { key: 'profit_per_trade_per_contract', label: 'Avg profit/trade/contract ($, optional)', def: 0 },
        ],
        render: (r) => `
            <table class="gs-table">
                <thead><tr><th>Contracts</th><th>Equity required</th><th>Gain from prev</th><th>Est. trades</th></tr></thead>
                <tbody>${r.rows.map(row => `
                    <tr>
                        <td>${row.contracts}</td>
                        <td>$${Math.round(row.equity_required).toLocaleString()}</td>
                        <td>${row.gain_from_prev ? '$' + Math.round(row.gain_from_prev).toLocaleString() : '—'}</td>
                        <td>${row.est_trades_from_prev != null ? row.est_trades_from_prev.toFixed(1) : '—'}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Total gain to max size: $${Math.round(r.total_gain_to_max).toLocaleString()}.
            Fixed ratio (Ryan Jones): each added contract requires delta × current contracts of NEW profit —
            growth is conservative early, accelerating only as equity compounds.</p>`,
    },
    'anti-martingale': {
        label: 'Anti-Martingale',
        call: (b) => api.calcAntiMartingale(b),
        fields: [
            { key: 'starting_capital', label: 'Starting capital ($)', def: 10000 },
            { key: 'base_risk_pct', label: 'Base risk (%/trade)', def: 1 },
            { key: 'win_factor', label: 'Risk × after win', def: 1.5 },
            { key: 'loss_factor', label: 'Risk × after loss', def: 0.5 },
            { key: 'max_risk_pct', label: 'Max risk (%)', def: 4 },
            { key: 'win_payoff_r', label: 'Win payoff (R)', def: 1.5 },
            { key: 'sequence', label: 'Outcomes (e.g. WWLWL)', def: 'WWLWLLWWW', text: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Anti-martingale</div>
                    <div class="value ${r.total_return_pct >= 0 ? 'pos' : 'neg'}">${r.total_return_pct.toFixed(2)}%</div>
                    <div class="small muted">$${Math.round(r.final_equity).toLocaleString()} final</div></div>
                <div class="card"><div class="label">Fixed risk (control)</div>
                    <div class="value ${r.fixed_risk_return_pct >= 0 ? 'pos' : 'neg'}">${r.fixed_risk_return_pct.toFixed(2)}%</div>
                    <div class="small muted">$${Math.round(r.fixed_risk_final_equity).toLocaleString()} final</div></div>
                <div class="card"><div class="label">Max drawdown</div>
                    <div class="value neg">${r.max_drawdown_pct.toFixed(2)}%</div></div>
            </div>
            <table class="gs-table">
                <thead><tr><th>#</th><th>Outcome</th><th>Risk</th><th>P&amp;L</th><th>Equity</th></tr></thead>
                <tbody>${r.rows.map(row => `
                    <tr>
                        <td>${row.trade}</td>
                        <td class="${row.outcome === 'W' ? 'pos' : 'neg'}">${row.outcome}</td>
                        <td>${row.risk_pct.toFixed(2)}%</td>
                        <td class="${row.pnl >= 0 ? 'pos' : 'neg'}">$${row.pnl.toFixed(2)}</td>
                        <td>$${Math.round(row.equity).toLocaleString()}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Press risk after wins (capped), cut back toward base after losses —
            the opposite of a martingale. The control row trades the same sequence at flat base risk.</p>`,
    },
    'dual-momentum': {
        label: 'Dual Momentum',
        call: (b) => api.simDualMomentum(b),
        fields: [
            { key: 'years', label: 'Backtest years', def: 10, int: true },
            { key: 'lookback_months', label: 'Lookback (months)', def: 12, int: true },
            { key: 'equity_us', label: 'US equity', def: 'SPY', text: true },
            { key: 'equity_intl', label: 'Intl equity', def: 'EFA', text: true },
            { key: 'tbill', label: 'T-bill proxy', def: 'BIL', text: true },
            { key: 'bonds', label: 'Risk-off sleeve', def: 'AGG', text: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Signal now</div>
                    <div class="value">${esc(r.current_signal)}</div>
                    <div class="small muted">${r.switches} switches over ${r.months} months</div></div>
                <div class="card"><div class="label">GEM return</div>
                    <div class="value ${r.gem_return_pct >= 0 ? 'pos' : 'neg'}">${r.gem_return_pct.toFixed(1)}%</div>
                    <div class="small muted">max DD ${r.gem_max_drawdown_pct.toFixed(1)}%</div></div>
                <div class="card"><div class="label">Buy &amp; hold</div>
                    <div class="value ${r.buy_hold_return_pct >= 0 ? 'pos' : 'neg'}">${r.buy_hold_return_pct.toFixed(1)}%</div>
                    <div class="small muted">max DD ${r.buy_hold_max_drawdown_pct.toFixed(1)}%</div></div>
            </div>
            <p class="muted small">Antonacci GEM: each month-end, hold the stronger of US/intl equities by
            trailing lookback return when either beats T-bills; otherwise retreat to bonds.
            Recent holdings: ${esc(r.rows.slice(-6).map(x => x.holding).join(' → '))}.</p>`,
    },
    'vol-cone': {
        label: 'Vol Cone',
        call: (b) => api.volCone(b.symbol, b.years),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'SPY', text: true },
            { key: 'years', label: 'Lookback years', def: 5, int: true },
        ],
        render: (r) => `
            <table class="gs-table">
                <thead><tr><th>Horizon</th><th>Min</th><th>P25</th><th>Median</th><th>P75</th><th>Max</th><th>Current</th><th>Rank</th></tr></thead>
                <tbody>${r.rows.map(row => `
                    <tr>
                        <td>${row.horizon_days}d</td>
                        <td>${row.min_pct.toFixed(1)}%</td>
                        <td>${row.p25_pct.toFixed(1)}%</td>
                        <td>${row.median_pct.toFixed(1)}%</td>
                        <td>${row.p75_pct.toFixed(1)}%</td>
                        <td>${row.max_pct.toFixed(1)}%</td>
                        <td class="${row.current_rank_pct >= 75 ? 'neg' : row.current_rank_pct <= 25 ? 'pos' : ''}">${row.current_pct.toFixed(1)}%</td>
                        <td>${row.current_rank_pct.toFixed(0)}%</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Burghardt-Lane realized-vol cone over ${r.days_analyzed} daily closes of
            ${esc(r.symbol)}. Compare an option's implied vol against the row at its horizon: current RV in
            the bottom quartile (green) flags cheap realized regimes, top quartile (red) rich ones.</p>`,
    },
    'day-of-week': {
        label: 'Day of Week',
        call: (b) => api.dayOfWeekSeasonality(b.symbol, b.years),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'SPY', text: true },
            { key: 'years', label: 'Lookback years', def: 10, int: true },
        ],
        render: (r) => {
            const NAMES = { 1: 'Mon', 2: 'Tue', 3: 'Wed', 4: 'Thu', 5: 'Fri', 6: 'Sat', 7: 'Sun' };
            const rows = r.by_weekday.filter(x => x.sample_count > 0);
            return `
            <table class="gs-table">
                <thead><tr><th>Day</th><th>Mean return</th><th>Std dev</th><th>Hit rate</th><th>N</th></tr></thead>
                <tbody>${rows.map(row => `
                    <tr>
                        <td>${NAMES[row.day_of_week] || row.day_of_week}</td>
                        <td class="${row.mean_return >= 0 ? 'pos' : 'neg'}">${(row.mean_return * 100).toFixed(3)}%</td>
                        <td>${(row.std_return * 100).toFixed(2)}%</td>
                        <td>${(row.hit_rate * 100).toFixed(0)}%</td>
                        <td>${row.sample_count}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Per-weekday log-return stats over ${r.days_analyzed} daily closes of
            ${esc(r.symbol)} — the classic Monday-effect / Friday-strength study.</p>`;
        },
    },
    'conversion-reversal': {
        label: 'Conversion/Reversal',
        call: (b) => api.calcConversionReversal(b),
        fields: [
            { key: 'spot', label: 'Spot ($)', def: 100 },
            { key: 'strike', label: 'Strike ($)', def: 100 },
            { key: 'call_price', label: 'Call price ($)', def: 4.25 },
            { key: 'put_price', label: 'Put price ($)', def: 3 },
            { key: 'pv_dividends', label: 'PV dividends ($)', def: 0 },
            { key: 'time_to_expiry_years', label: 'Time to expiry (years)', def: 0.25 },
            { key: 'market_risk_free_rate', label: 'Market rate (decimal)', def: 0.05 },
            { key: 'arbitrage_threshold_bps', label: 'Arb threshold (bps)', def: 50 },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid inputs</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">Implied rate</div>
                    <div class="value">${isFinite(r.implied_continuous_rate) ? (r.implied_continuous_rate * 100).toFixed(2) + '%' : '∞'}</div>
                    <div class="small muted">vs ${(r.market_rate * 100).toFixed(2)}% market</div></div>
                <div class="card"><div class="label">Deviation</div>
                    <div class="value ${r.is_arbitrage_opportunity ? 'neg' : 'pos'}">${isFinite(r.deviation_basis_points) ? r.deviation_basis_points.toFixed(0) + ' bps' : '∞'}</div>
                    <div class="small muted">${esc(r.arbitrage_side)} ${r.is_arbitrage_opportunity ? '— ARB' : ''}</div></div>
                <div class="card"><div class="label">Edge / share</div>
                    <div class="value ${r.edge_per_share >= 0 ? 'pos' : 'neg'}">$${r.edge_per_share.toFixed(3)}</div>
                    <div class="small muted">conversion cost $${r.conversion_cost.toFixed(2)} → $${r.strike.toFixed(0)} locked</div></div>
            </div>
            <p class="muted small">Put-call parity check: conversion (+stock +put −call) locks the strike at
            expiry. Implied rate above market = do the conversion; below = do the reversal.</p>`;
        },
    },
    seagull: {
        label: 'Seagull Spread',
        call: (b) => api.calcSeagull(b),
        fields: [
            { key: 'put_strike', label: 'Short put strike ($)', def: 90 },
            { key: 'call_low_strike', label: 'Long call strike ($)', def: 100 },
            { key: 'call_high_strike', label: 'Short call strike ($)', def: 110 },
            { key: 'put_price', label: 'Put premium ($)', def: 2 },
            { key: 'call_low_price', label: 'Long call premium ($)', def: 3 },
            { key: 'call_high_price', label: 'Short call premium ($)', def: 1 },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid inputs (need K_put < K_call_low < K_call_high)</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">Net ${r.net_debit >= 0 ? 'debit' : 'credit'}</div>
                    <div class="value">$${Math.abs(r.net_debit).toFixed(2)}</div>
                    <div class="small ${r.is_zero_cost ? 'pos' : 'muted'}">${r.is_zero_cost ? 'zero-cost structure' : ''}</div></div>
                <div class="card"><div class="label">Max profit</div>
                    <div class="value pos">$${r.max_profit.toFixed(2)}</div>
                    <div class="small muted">above $${r.call_high_strike.toFixed(0)}</div></div>
                <div class="card"><div class="label">Loss at zero</div>
                    <div class="value neg">$${r.max_loss_at_zero.toFixed(2)}</div></div>
                <div class="card"><div class="label">Breakevens</div>
                    <div class="value">${r.downside_breakeven != null ? '$' + r.downside_breakeven.toFixed(2) : ''}${r.upside_breakeven != null ? ' $' + r.upside_breakeven.toFixed(2) : ''}</div></div>
            </div>
            <table class="gs-table">
                <thead><tr><th>Price</th><th>P&amp;L</th></tr></thead>
                <tbody>${r.payoff_points.map(([s, p]) => `
                    <tr><td>$${s.toFixed(2)}</td>
                        <td class="${p >= 0 ? 'pos' : 'neg'}">$${p.toFixed(2)}</td></tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Bullish seagull: short put finances a call spread — upside participation
            ${esc(String(r.call_low_strike))}→${esc(String(r.call_high_strike))} paid for with downside risk below ${esc(String(r.put_strike))}.</p>`;
        },
    },
    'turn-of-month': {
        label: 'Turn of Month',
        call: (b) => api.turnOfMonth(b.symbol, b.years),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'SPY', text: true },
            { key: 'years', label: 'Lookback years', def: 10, int: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">TOM window avg</div>
                    <div class="value ${r.tom_avg_return_pct >= 0 ? 'pos' : 'neg'}">${r.tom_avg_return_pct.toFixed(3)}%/day</div></div>
                <div class="card"><div class="label">All other days</div>
                    <div class="value ${r.rest_avg_return_pct >= 0 ? 'pos' : 'neg'}">${r.rest_avg_return_pct.toFixed(3)}%/day</div></div>
                <div class="card"><div class="label">TOM edge</div>
                    <div class="value ${r.edge_pct >= 0 ? 'pos' : 'neg'}">${(r.edge_pct >= 0 ? '+' : '') + r.edge_pct.toFixed(3)}pp</div>
                    <div class="small muted">${r.days_analyzed} days on ${esc(r.symbol)}</div></div>
            </div>
            <table class="gs-table">
                <thead><tr><th>Offset</th><th>Avg return</th><th>Hit rate</th><th>N</th></tr></thead>
                <tbody>${r.rows.map(row => `
                    <tr>
                        <td>${row.offset > 0 ? '+' + row.offset : row.offset}</td>
                        <td class="${row.avg_return_pct >= 0 ? 'pos' : 'neg'}">${row.avg_return_pct.toFixed(3)}%</td>
                        <td>${row.hit_rate_pct.toFixed(0)}%</td>
                        <td>${row.n}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Offset −1 = last trading day of the month, +1 = first. The classic
            turn-of-month effect concentrates equity returns in the −1..+3 window.</p>`,
    },
};

export async function renderStrategyTools(mount) {
    renderToolTabs(mount, {
        titleKey: 'view.strategy_tools.h1',
        title: '// STRATEGY TOOLS',
        hintKey: 'view.strategy_tools.hint',
        tools: TOOLS,
        defaultKey: 'grid-trading',
    });
}
