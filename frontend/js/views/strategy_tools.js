// Strategy tools — Grid Trading ladder, Fixed-Ratio position sizing
// (Ryan Jones), Anti-Martingale streak sizing, Dual Momentum (GEM),
// and Turn-of-Month seasonality behind one tabbed view. Each tool is a
// server-side compute; the tab/form/result chrome lives in tool_tabs.js.

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
