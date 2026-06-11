// Strategy tools — Grid Trading ladder, Fixed-Ratio position sizing
// (Ryan Jones), and Turn-of-Month seasonality behind one tabbed view.
// Each tool is a server-side compute; the tab/form/result chrome lives
// in tool_tabs.js.

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
