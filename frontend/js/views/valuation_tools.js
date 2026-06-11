// Valuation tools — Reverse DCF, Dividend Discount, Earnings Power
// Value, and the options Wheel calculator behind one tabbed view. Each
// tool is a server-side pure-compute POST; the tab/form/result chrome
// lives in tool_tabs.js.

import { api } from '../api.js';
import { esc } from '../util.js';
import { renderToolTabs } from './tool_tabs.js';

const TOOLS = {
    'reverse-dcf': {
        label: 'Reverse DCF',
        call: (b) => api.calcReverseDcf(b),
        fields: [
            { key: 'fcf_usd', label: 'Free cash flow (TTM, $)', def: 100_000_000_000 },
            { key: 'growth_years', label: 'Growth years', def: 5, int: true },
            { key: 'terminal_growth_pct', label: 'Terminal growth (%)', def: 2.5 },
            { key: 'discount_rate_pct', label: 'Discount rate (%)', def: 10 },
            { key: 'net_debt_usd', label: 'Net debt ($)', def: 0 },
            { key: 'shares_outstanding', label: 'Shares outstanding', def: 15_000_000_000 },
            { key: 'current_price', label: 'Current price ($)', def: 200 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Implied growth</div>
                    <div class="value">${r.implied_growth_pct.toFixed(2)}%/yr</div>
                    <div class="small muted">market-priced stage-1 FCF growth</div></div>
                <div class="card"><div class="label">Check value</div>
                    <div class="value">$${r.intrinsic_at_implied.toFixed(2)}</div></div>
            </div>
            <p class="muted small">If you believe the business will grow FASTER than the implied
            rate, the stock is cheap by your own assumptions — and vice versa.</p>`,
    },
    ddm: {
        label: 'Dividend Discount',
        call: (b) => api.calcDdm(b),
        fields: [
            { key: 'annual_dividend', label: 'Forward dividend / share ($)', def: 4 },
            { key: 'growth_pct', label: 'Stage-1 growth (%/yr)', def: 6 },
            { key: 'growth_years', label: 'Stage-1 years (0 = Gordon)', def: 5, int: true },
            { key: 'terminal_growth_pct', label: 'Terminal growth (%)', def: 2.5 },
            { key: 'required_return_pct', label: 'Required return (%)', def: 9 },
            { key: 'current_price', label: 'Current price ($, optional)', def: '', optional: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Fair value (${esc(r.model)})</div>
                    <div class="value">$${r.fair_value.toFixed(2)}</div>
                    ${r.upside_pct != null ? `<div class="small ${r.upside_pct >= 0 ? 'pos' : 'neg'}">${(r.upside_pct >= 0 ? '+' : '') + r.upside_pct.toFixed(1)}% vs price</div>` : ''}
                </div>
            </div>`,
    },
    epv: {
        label: 'Earnings Power',
        call: (b) => api.calcEpv(b),
        fields: [
            { key: 'normalized_ebit_usd', label: 'Normalized EBIT ($)', def: 10_000_000_000 },
            { key: 'tax_rate_pct', label: 'Tax rate (%)', def: 21 },
            { key: 'wacc_pct', label: 'WACC (%)', def: 9 },
            { key: 'net_cash_adjustment_usd', label: 'Net cash adj. ($)', def: 0 },
            { key: 'shares_outstanding', label: 'Shares outstanding', def: 1_000_000_000 },
            { key: 'current_price', label: 'Current price ($, optional)', def: '', optional: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">EPV / share</div>
                    <div class="value">$${r.epv_per_share.toFixed(2)}</div>
                    ${r.upside_pct != null ? `<div class="small ${r.upside_pct >= 0 ? 'pos' : 'neg'}">${(r.upside_pct >= 0 ? '+' : '') + r.upside_pct.toFixed(1)}% vs price</div>` : ''}
                </div>
                <div class="card"><div class="label">NOPAT</div><div class="value">$${(r.nopat_usd / 1e9).toFixed(2)}B</div></div>
                <div class="card"><div class="label">EPV (operations)</div><div class="value">$${(r.epv_operations_usd / 1e9).toFixed(2)}B</div></div>
            </div>
            <p class="muted small">Greenwald EPV: value at ZERO growth. Price above EPV means
            the market is paying for growth; below means even no-growth earnings cover it.</p>`,
    },
    'value-averaging': {
        label: 'Value Avg vs DCA',
        call: (b) => api.simValueAveraging(b),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'SPY', text: true },
            { key: 'years', label: 'Backtest years', def: 5, int: true },
            { key: 'monthly_amount', label: 'Monthly amount ($)', def: 1000 },
            { key: 'target_growth_pct_monthly', label: 'VA target growth (%/mo)', def: 0.5 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Value averaging</div>
                    <div class="value ${r.va_return_pct >= 0 ? 'pos' : 'neg'}">${r.va_return_pct.toFixed(1)}%</div>
                    <div class="small muted">$${Math.round(r.va_final_value).toLocaleString()} on $${Math.round(r.va_total_contributed).toLocaleString()} in</div></div>
                <div class="card"><div class="label">DCA</div>
                    <div class="value ${r.dca_return_pct >= 0 ? 'pos' : 'neg'}">${r.dca_return_pct.toFixed(1)}%</div>
                    <div class="small muted">$${Math.round(r.dca_final_value).toLocaleString()} on $${Math.round(r.dca_total_contributed).toLocaleString()} in</div></div>
                <div class="card"><div class="label">VA edge</div>
                    <div class="value ${r.va_edge_pct >= 0 ? 'pos' : 'neg'}">${(r.va_edge_pct >= 0 ? '+' : '') + r.va_edge_pct.toFixed(1)}pp</div>
                    <div class="small muted">${r.months} months on ${esc(r.symbol)}</div></div>
            </div>`,
    },
    cppi: {
        label: 'CPPI',
        call: (b) => api.simCppi(b),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'SPY', text: true },
            { key: 'years', label: 'Backtest years', def: 5, int: true },
            { key: 'initial_capital', label: 'Initial capital ($)', def: 100000 },
            { key: 'floor_fraction', label: 'Floor (fraction, e.g. 0.8)', def: 0.8 },
            { key: 'multiplier', label: 'Multiplier', def: 4 },
            { key: 'cash_rate_pct', label: 'Cash rate (%/yr)', def: 4 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">CPPI return</div>
                    <div class="value ${r.total_return_pct >= 0 ? 'pos' : 'neg'}">${r.total_return_pct.toFixed(1)}%</div>
                    <div class="small muted">$${Math.round(r.final_value).toLocaleString()} final</div></div>
                <div class="card"><div class="label">Buy & hold</div>
                    <div class="value ${r.buy_and_hold_return_pct >= 0 ? 'pos' : 'neg'}">${r.buy_and_hold_return_pct.toFixed(1)}%</div></div>
                <div class="card"><div class="label">Max drawdown</div>
                    <div class="value neg">${r.max_drawdown_pct.toFixed(1)}%</div></div>
                <div class="card"><div class="label">Floor</div>
                    <div class="value ${r.floor_breached ? 'neg' : 'pos'}">$${Math.round(r.floor_value).toLocaleString()}</div>
                    <div class="small ${r.floor_breached ? 'neg' : 'muted'}">${r.floor_breached ? 'BREACHED (gap risk)' : 'held'}</div></div>
            </div>`,
    },
    wheel: {
        label: 'Wheel Calculator',
        call: (b) => api.calcWheel(b),
        fields: [
            { key: 'stock_price', label: 'Stock price ($)', def: 100 },
            { key: 'put_strike', label: 'CSP strike ($)', def: 95 },
            { key: 'put_premium', label: 'CSP premium ($)', def: 2 },
            { key: 'put_dte', label: 'CSP days to expiry', def: 30, int: true },
            { key: 'call_strike', label: 'CC strike ($)', def: 100 },
            { key: 'call_premium', label: 'CC premium ($)', def: 2.5 },
            { key: 'call_dte', label: 'CC days to expiry', def: 30, int: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Put-side return</div>
                    <div class="value">${r.put_side_return_pct.toFixed(2)}%</div>
                    <div class="small pos">${r.put_side_annualized_pct.toFixed(1)}% annualized</div></div>
                <div class="card"><div class="label">Full-cycle return</div>
                    <div class="value">${r.full_cycle_return_pct.toFixed(2)}%</div>
                    <div class="small pos">${r.full_cycle_annualized_pct.toFixed(1)}% ann. / ${r.full_cycle_days}d</div></div>
                <div class="card"><div class="label">Assigned basis</div>
                    <div class="value">$${r.assigned_cost_basis.toFixed(2)}</div>
                    <div class="small muted">${r.breakeven_drop_pct.toFixed(1)}% cushion</div></div>
            </div>`,
    },
};

export async function renderValuationTools(mount) {
    renderToolTabs(mount, {
        titleKey: 'view.valuation_tools.h1',
        title: '// VALUATION TOOLS',
        hintKey: 'view.valuation_tools.hint',
        tools: TOOLS,
        defaultKey: 'reverse-dcf',
    });
}
