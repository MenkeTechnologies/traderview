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
    'risk-of-ruin': {
        label: 'Risk of Ruin',
        call: (b) => api.calcRiskOfRuin(b),
        fields: [
            { key: 'win_probability', label: 'Win probability (0–1)', def: 0.55 },
            { key: 'payoff_ratio', label: 'Payoff ratio (R)', def: 1.5 },
            { key: 'capital', label: 'Capital ($)', def: 10000 },
            { key: 'risk_per_trade', label: 'Risk per trade ($)', def: 250 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Risk of ruin</div>
                    <div class="value ${r.risk_of_ruin > 0.05 ? 'neg' : 'pos'}">${(r.risk_of_ruin * 100).toPrecision(3)}%</div>
                    <div class="small muted">${r.risk_units.toFixed(1)} risk units · z₀ ${r.z0.toFixed(4)}</div></div>
                <div class="card"><div class="label">Expectancy</div>
                    <div class="value ${r.expectancy_r > 0 ? 'pos' : 'neg'}">${r.expectancy_r.toFixed(3)}R</div></div>
                <div class="card"><div class="label">Full Kelly</div>
                    <div class="value">${(r.kelly_fraction * 100).toFixed(1)}%</div>
                    <div class="small muted">of capital per trade</div></div>
            </div>
            <p class="muted small">Closed-form gambler's-ruin: each trade risks 1 unit, wins R units with
            probability p; RoR = z₀^units from the characteristic equation. Non-positive expectancy ⇒
            ruin certain. Complements the Monte Carlo simulator with an analytic answer.</p>`,
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
    heston: {
        label: 'Heston Pricer',
        call: (b) => api.calcHeston(b),
        fields: [
            { key: 'spot', label: 'Spot ($)', def: 100 },
            { key: 'strike', label: 'Strike ($)', def: 100 },
            { key: 'time_to_expiry_years', label: 'Time to expiry (years)', def: 0.5 },
            { key: 'risk_free_rate', label: 'Risk-free rate (decimal)', def: 0.03 },
            { key: 'dividend_yield', label: 'Dividend yield (decimal)', def: 0 },
            { key: 'v0', label: 'Initial variance v₀', def: 0.04 },
            { key: 'kappa', label: 'Mean reversion κ', def: 2 },
            { key: 'theta', label: 'Long-run variance θ', def: 0.04 },
            { key: 'vol_of_vol', label: 'Vol of vol σ', def: 0.3 },
            { key: 'rho', label: 'Correlation ρ', def: -0.7 },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid inputs</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">Call</div>
                    <div class="value pos">$${r.call_price.toFixed(4)}</div></div>
                <div class="card"><div class="label">Put</div>
                    <div class="value">$${r.put_price.toFixed(4)}</div></div>
                <div class="card"><div class="label">Exercise probs</div>
                    <div class="value">${(r.p2 * 100).toFixed(1)}%</div>
                    <div class="small muted">P1 ${(r.p1 * 100).toFixed(1)}% · P2 risk-neutral ITM</div></div>
                <div class="card"><div class="label">Feller 2κθ ≥ σ²</div>
                    <div class="value ${r.feller_satisfied ? 'pos' : 'neg'}">${r.feller_satisfied ? 'satisfied' : 'VIOLATED'}</div>
                    <div class="small muted">${r.feller_satisfied ? 'variance stays positive' : 'variance can hit zero'}</div></div>
            </div>
            <p class="muted small">Heston (1993) stochastic-vol European pricer — semi-analytic
            characteristic-function integration ("little trap" form). Negative ρ produces the
            equity-style downside skew a flat-vol Black-Scholes can't.</p>`;
        },
    },
    'diagonal-spread': {
        label: 'Diagonal Spread',
        call: (b) => api.calcDiagonalSpread({
            spread: {
                front_strike: b.front_strike,
                back_strike: b.back_strike,
                kind: b.kind === 1 ? 'put' : 'call',
                front_premium: b.front_premium,
                back_premium: b.back_premium,
                back_time_after_front_expiry: b.back_time_after_front_expiry,
                risk_free: b.risk_free,
                dividend_yield: 0,
                sigma: b.sigma,
                contracts: 1,
                multiplier: 100,
            },
        }),
        fields: [
            { key: 'front_strike', label: 'Short front strike ($)', def: 105 },
            { key: 'back_strike', label: 'Long back strike ($)', def: 90 },
            { key: 'kind', label: 'Kind (0 = call, 1 = put)', def: 0, int: true },
            { key: 'front_premium', label: 'Front premium ($)', def: 2 },
            { key: 'back_premium', label: 'Back premium ($)', def: 12 },
            { key: 'back_time_after_front_expiry', label: 'Back time after front expiry (years)', def: 0.25 },
            { key: 'risk_free', label: 'Risk-free rate (decimal)', def: 0.05 },
            { key: 'sigma', label: 'Volatility (decimal)', def: 0.25 },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid inputs</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">Net debit</div>
                    <div class="value">$${r.net_debit.toFixed(2)}</div>
                    <div class="small muted">1 contract × 100</div></div>
                <div class="card"><div class="label">Max profit (at front expiry)</div>
                    <div class="value pos">$${r.max_profit.toFixed(2)}</div>
                    <div class="small muted">at $${r.max_profit_at.toFixed(2)}</div></div>
                <div class="card"><div class="label">Max loss (grid)</div>
                    <div class="value neg">$${r.max_loss.toFixed(2)}</div></div>
                <div class="card"><div class="label">Breakevens</div>
                    <div class="value">${r.breakevens.length ? r.breakevens.map(x => '$' + x.toFixed(2)).join(' / ') : '—'}</div></div>
            </div>
            <p class="muted small">Short front-month at one strike, long back-month at another ("poor man's
            covered call" when the back leg is deep ITM). Back leg revalued by Black-Scholes at front expiry.</p>`;
        },
    },
    'santa-rally': {
        label: 'Santa Rally',
        call: (b) => api.santaRally(b.symbol, b.years),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'SPY', text: true },
            { key: 'years', label: 'Lookback years', def: 15, int: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Avg window return</div>
                    <div class="value ${r.rally_avg_return_pct >= 0 ? 'pos' : 'neg'}">${r.rally_avg_return_pct.toFixed(2)}%</div>
                    <div class="small muted">last 5 Dec + first 2 Jan sessions</div></div>
                <div class="card"><div class="label">Hit rate</div>
                    <div class="value">${r.rally_hit_rate_pct.toFixed(0)}%</div>
                    <div class="small muted">${r.years_analyzed} complete windows on ${esc(r.symbol)}</div></div>
            </div>
            <table class="gs-table">
                <thead><tr><th>Year</th><th>Window return</th></tr></thead>
                <tbody>${r.yearly.map(y => `
                    <tr><td>${y.year}→${y.year + 1}</td>
                        <td class="${y.window_return_pct >= 0 ? 'pos' : 'neg'}">${y.window_return_pct.toFixed(2)}%</td></tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Hirsch's Santa Claus rally: "if Santa Claus should fail to call, bears may
            come to Broad and Wall" — a negative window has historically preceded weak Januaries.</p>`,
    },
    'correlation-regime': {
        label: 'Correlation Regime',
        call: (b) => api.correlationRegime(b.a, b.b, b.window, b.years),
        fields: [
            { key: 'a', label: 'Symbol A', def: 'SPY', text: true },
            { key: 'b', label: 'Symbol B', def: 'TLT', text: true },
            { key: 'window', label: 'Rolling window (days)', def: 63, int: true },
            { key: 'years', label: 'Lookback years', def: 5, int: true },
        ],
        render: (r) => {
            const cls = { coupled: 'pos', neutral: '', inverse: 'neg' };
            return `
            <div class="cards">
                <div class="card"><div class="label">Current ρ (${r.window}d)</div>
                    <div class="value ${cls[r.current_regime] || ''}">${r.current.toFixed(2)}</div>
                    <div class="small muted">${esc(r.current_regime)} · mean ${r.mean.toFixed(2)}</div></div>
                <div class="card"><div class="label">Time in regime</div>
                    <div class="value">${r.pct_coupled.toFixed(0)} / ${r.pct_neutral.toFixed(0)} / ${r.pct_inverse.toFixed(0)}%</div>
                    <div class="small muted">coupled / neutral / inverse</div></div>
                <div class="card"><div class="label">Regime breaks</div>
                    <div class="value">${r.breaks.length}</div>
                    <div class="small muted">${r.breaks.length ? 'last: ' + esc(r.breaks[r.breaks.length - 1].from) + ' → ' + esc(r.breaks[r.breaks.length - 1].to) : 'stable'}</div></div>
            </div>
            <p class="muted small">Rolling Pearson correlation of daily log returns; ±0.5 thresholds split
            coupled / neutral / inverse. Breaks mark the bars where a hedge pair stops hedging — the
            SPY/TLT inverse-to-coupled flip is the classic risk-parity pain trade.</p>`;
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
