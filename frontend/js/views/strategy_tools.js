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
    'seasonality-screen': {
        label: 'Seasonality Screen',
        call: (b) => api.seasonalityScreen({
            symbols: String(b.symbols).split(/[\s,]+/).filter(Boolean),
            years: b.years,
        }),
        fields: [
            { key: 'symbols', label: 'Symbols (comma-sep, ≤30)', def: 'SPY, QQQ, IWM, XLE, XLF, TLT, GLD', text: true },
            { key: 'years', label: 'Lookback years', def: 10, int: true },
        ],
        render: (r) => {
            const DOW = { 1: 'Mon', 2: 'Tue', 3: 'Wed', 4: 'Thu', 5: 'Fri' };
            return `
            <table class="gs-table">
                <thead><tr><th>Symbol</th><th>TOM edge</th><th>Overnight</th><th>Intraday</th><th>Santa avg</th><th>Best day</th></tr></thead>
                <tbody>${r.rows.map(row => `
                    <tr>
                        <td>${esc(row.symbol)}</td>
                        <td class="${row.tom_edge_pp >= 0 ? 'pos' : 'neg'}">${(row.tom_edge_pp >= 0 ? '+' : '') + row.tom_edge_pp.toFixed(3)}pp</td>
                        <td class="${row.overnight_total_pct >= 0 ? 'pos' : 'neg'}">${row.overnight_total_pct.toFixed(0)}%</td>
                        <td class="${row.intraday_total_pct >= 0 ? 'pos' : 'neg'}">${row.intraday_total_pct.toFixed(0)}%</td>
                        <td>${row.santa_avg_pct != null ? row.santa_avg_pct.toFixed(2) + '%' : '—'}</td>
                        <td>${row.best_weekday != null ? DOW[row.best_weekday] || row.best_weekday : '—'}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            ${r.skipped.length ? `<p class="muted small neg">skipped (insufficient history): ${r.skipped.map(esc).join(', ')}</p>` : ''}
            <p class="muted small">Calendar edges ranked by |TOM edge| — the same per-symbol studies as
            the character sheet, side by side across the list.</p>`;
        },
    },
    'character-sheet': {
        label: 'Symbol Character',
        call: (b) => api.characterSheet(b.symbol, b.years),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'SPY', text: true },
            { key: 'years', label: 'Lookback years', def: 10, int: true },
        ],
        render: (r) => {
            const on = r.overnight;
            const tom = r.turn_of_month;
            const dd = r.drawdowns;
            const cc = r.concentration;
            const cone21 = r.vol_cone.find(x => x.horizon_days === 21);
            return `
            <div class="cards">
                ${tom ? `<div class="card"><div class="label">Turn-of-month edge</div>
                    <div class="value ${tom.edge_pct >= 0 ? 'pos' : 'neg'}">${(tom.edge_pct >= 0 ? '+' : '') + tom.edge_pct.toFixed(3)}pp/day</div></div>` : ''}
                ${on ? `<div class="card"><div class="label">Overnight vs intraday</div>
                    <div class="value">${on.overnight_total_pct.toFixed(0)}% / ${on.intraday_total_pct.toFixed(0)}%</div></div>` : ''}
                ${r.santa ? `<div class="card"><div class="label">Santa rally</div>
                    <div class="value ${r.santa.rally_avg_return_pct >= 0 ? 'pos' : 'neg'}">${r.santa.rally_avg_return_pct.toFixed(2)}%</div>
                    <div class="small muted">hit ${r.santa.rally_hit_rate_pct.toFixed(0)}% × ${r.santa.years_analyzed}y</div></div>` : ''}
                ${cone21 ? `<div class="card"><div class="label">21d realized vol</div>
                    <div class="value">${cone21.current_pct.toFixed(1)}%</div>
                    <div class="small muted">rank ${cone21.current_rank_pct.toFixed(0)}% of own history</div></div>` : ''}
                ${dd && dd.episodes.length ? `<div class="card"><div class="label">Worst drawdown</div>
                    <div class="value neg">${dd.episodes[0].depth_pct.toFixed(1)}%</div>
                    <div class="small muted">${dd.currently_underwater ? r.symbol + ' underwater now' : 'recovered'}</div></div>` : ''}
                ${cc ? `<div class="card"><div class="label">Missing 10 best days</div>
                    <div class="value neg">${cc.missing_best_pct.toFixed(0)}%</div>
                    <div class="small muted">vs ${cc.total_return_pct.toFixed(0)}% buy-and-hold</div></div>` : ''}
            </div>
            <p class="muted small">One bar fetch, every study in the toolbox: ${r.days_analyzed} sessions
            of ${esc(r.symbol)}. Missing cards mean not enough history for that leg — partial, never faked.</p>`;
        },
    },
    'report-card': {
        label: 'Trade Report Card',
        call: (b) => api.tradeReportCard({
            starting_equity: b.starting_equity,
            trade_pnls: String(b.trade_pnls).split(/[\s,]+/).map(Number).filter(x => isFinite(x)),
        }),
        fields: [
            { key: 'starting_equity', label: 'Starting equity ($)', def: 10000 },
            { key: 'trade_pnls', label: 'Trade P/Ls (comma-sep, oldest first)', def: '300,300,-200,300,-200,300,300,-200,-200,300', text: true },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">need ≥2 finite trades and positive equity</span>';
            const q = r.quality;
            const wr = r.win_rate;
            const ror = r.risk_of_ruin;
            const dd = r.drawdowns;
            const ecf = r.equity_filter;
            return `
            <div class="cards">
                <div class="card"><div class="label">Profit factor / PRR</div>
                    <div class="value">${q.profit_factor != null ? q.profit_factor.toFixed(2) : '∞'} / ${q.pessimistic_return_ratio != null ? q.pessimistic_return_ratio.toFixed(2) : '—'}</div>
                    <div class="small muted">${q.win_count}W / ${q.loss_count}L · net $${Math.round(q.net_profit).toLocaleString()}</div></div>
                ${wr ? `<div class="card"><div class="label">Edge is real?</div>
                    <div class="value ${wr.statistically_significant ? 'pos' : 'neg'}">${wr.statistically_significant ? 'YES' : 'NOT YET'}</div>
                    <div class="small muted">Wilson [${wr.wilson_low_pct.toFixed(0)}%, ${wr.wilson_high_pct.toFixed(0)}%] vs ${wr.breakeven_win_rate_pct.toFixed(0)}% breakeven${wr.trades_needed != null ? ' · need ' + wr.trades_needed : ''}</div></div>` : ''}
                ${ror ? `<div class="card"><div class="label">Risk of ruin</div>
                    <div class="value ${ror.risk_of_ruin > 0.05 ? 'neg' : 'pos'}">${(ror.risk_of_ruin * 100).toPrecision(3)}%</div>
                    <div class="small muted">at observed p, R=${r.observed_payoff_ratio.toFixed(2)}, risk $${r.observed_avg_loss.toFixed(0)}/trade</div></div>` : ''}
                ${dd ? `<div class="card"><div class="label">Worst drawdown</div>
                    <div class="value neg">${dd.episodes.length ? dd.episodes[0].depth_pct.toFixed(1) + '%' : 'none'}</div>
                    <div class="small muted">${dd.currently_underwater ? 'currently underwater' : 'at highs'}</div></div>` : ''}
                ${ecf ? `<div class="card"><div class="label">Equity-curve filter</div>
                    <div class="value ${ecf.filter_helped ? 'pos' : ''}">${ecf.filter_helped ? 'WOULD HAVE HELPED' : 'not needed'}</div>
                    <div class="small muted">$${Math.round(ecf.filtered_final).toLocaleString()} vs $${Math.round(ecf.unfiltered_final).toLocaleString()}</div></div>` : ''}
            </div>
            <p class="muted small">One P/L list, five verdicts cross-read — PF beside the Wilson interval
            that says whether it's proven, and ruin odds at the sizing you actually used.</p>`;
        },
    },
    'win-rate-confidence': {
        label: 'Win-Rate Confidence',
        call: (b) => api.calcWinRateConfidence(b),
        fields: [
            { key: 'wins', label: 'Wins', def: 30, int: true },
            { key: 'losses', label: 'Losses', def: 20, int: true },
            { key: 'payoff_ratio', label: 'Payoff ratio (R)', def: 1 },
            { key: 'z', label: 'z (1.96 = 95%)', def: 1.96 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Win rate (Wilson)</div>
                    <div class="value">${r.observed_win_rate_pct.toFixed(1)}%</div>
                    <div class="small muted">[${r.wilson_low_pct.toFixed(1)}%, ${r.wilson_high_pct.toFixed(1)}%]</div></div>
                <div class="card"><div class="label">Edge is real?</div>
                    <div class="value ${r.statistically_significant ? 'pos' : 'neg'}">${r.statistically_significant ? 'YES' : 'NOT YET'}</div>
                    <div class="small muted">breakeven ${r.breakeven_win_rate_pct.toFixed(1)}% at this payoff</div></div>
                <div class="card"><div class="label">Trades needed</div>
                    <div class="value">${r.trades_needed != null ? r.trades_needed : '∞'}</div>
                    <div class="small muted">${r.trades_needed != null ? 'for significance at this rate' : 'observed rate ≤ breakeven'}</div></div>
            </div>
            <p class="muted small">Wilson interval vs the 1/(1+R) breakeven — a 60% win rate over 50
            trades still straddles breakeven at 95% confidence.</p>`,
    },
    'equity-curve-filter': {
        label: 'Equity Curve Filter',
        call: (b) => api.calcEquityCurveFilter({
            starting_equity: b.starting_equity,
            trade_pnls: String(b.trade_pnls).split(/[\s,]+/).map(Number).filter(x => isFinite(x)),
            ma_length: b.ma_length,
        }),
        fields: [
            { key: 'starting_equity', label: 'Starting equity ($)', def: 10000 },
            { key: 'trade_pnls', label: 'Trade P/Ls (comma-sep, oldest first)', def: '100,100,100,100,100,100,100,100,100,100,-100,-100,-100,-100,-100,-100,-100,-100,-100,-100,100,100,100', text: true },
            { key: 'ma_length', label: 'MA length (trades)', def: 5, int: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Filtered</div>
                    <div class="value ${r.filter_helped ? 'pos' : ''}">$${Math.round(r.filtered_final).toLocaleString()}</div>
                    <div class="small muted">max DD $${Math.round(r.filtered_max_dd).toLocaleString()}</div></div>
                <div class="card"><div class="label">Unfiltered</div>
                    <div class="value">$${Math.round(r.unfiltered_final).toLocaleString()}</div>
                    <div class="small muted">max DD $${Math.round(r.unfiltered_max_dd).toLocaleString()}</div></div>
                <div class="card"><div class="label">Verdict</div>
                    <div class="value ${r.filter_helped ? 'pos' : 'neg'}">${r.filter_helped ? 'FILTER HELPED' : 'filter cost money'}</div>
                    <div class="small muted">${r.trades_taken} taken · ${r.trades_skipped} skipped</div></div>
            </div>
            <p class="muted small">Trade only while the system's own equity sits at/above its N-trade SMA;
            skipped trades accrue on paper. Shortens decaying-edge losing streaks, pays whipsaw tax on
            choppy ones — the verdict tells you which system you have.</p>`,
    },
    'asset-location': {
        label: 'Asset Location',
        call: (b) => api.calcAssetLocation(b),
        fields: [
            { key: 'growth_pct', label: 'Price growth (%/yr)', def: 6 },
            { key: 'yield_pct', label: 'Income yield (%/yr)', def: 0 },
            { key: 'ordinary_rate_pct', label: 'Ordinary tax rate (%)', def: 35 },
            { key: 'cap_gains_rate_pct', label: 'Cap-gains rate (%)', def: 15 },
            { key: 'years', label: 'Holding years', def: 20, int: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Tax drag in taxable</div>
                    <div class="value neg">−${r.tax_drag_pp.toFixed(2)}pp/yr</div>
                    <div class="small muted">rank assets by this to fill the IRA</div></div>
                <div class="card"><div class="label">CAGR</div>
                    <div class="value">${r.taxable_after_tax_cagr_pct.toFixed(2)}%</div>
                    <div class="small muted">vs ${r.pre_tax_cagr_pct.toFixed(2)}% sheltered</div></div>
                <div class="card"><div class="label">Per $1 at horizon</div>
                    <div class="value">$${r.final_value_taxable.toFixed(3)}</div>
                    <div class="small muted">vs $${r.final_value_pre_tax.toFixed(3)} pre-tax</div></div>
            </div>
            <p class="muted small">Annual ordinary tax on the yield, terminal cap-gains on the growth (basis
            tracked). High-yield assets drag multiples of what deferred growth drags — bonds go in the IRA.</p>`,
    },
    'alpha-horizon': {
        label: 'Alpha vs Cost',
        call: (b) => api.calcAlphaHorizon(b),
        fields: [
            { key: 'alpha_bp_per_day', label: 'Alpha (bp/day on)', def: 2 },
            { key: 'round_trip_cost_bp', label: 'Round-trip cost (bp)', def: 10 },
            { key: 'holding_days', label: 'Intended holding (days)', def: 10 },
            { key: 'days_per_year', label: 'Trading days/yr', def: 252 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Breakeven holding</div>
                    <div class="value">${r.breakeven_days.toFixed(1)} days</div>
                    <div class="small ${r.viable ? 'pos' : 'neg'}">${r.viable ? 'viable at your horizon' : 'NOT VIABLE — costs exceed alpha'}</div></div>
                <div class="card"><div class="label">Net per trade</div>
                    <div class="value ${r.net_alpha_per_trade_bp >= 0 ? 'pos' : 'neg'}">${r.net_alpha_per_trade_bp.toFixed(1)}bp</div>
                    <div class="small muted">costs eat ${r.cost_share_pct.toFixed(0)}% of gross</div></div>
                <div class="card"><div class="label">Net annualized</div>
                    <div class="value ${r.net_alpha_annual_bp >= 0 ? 'pos' : 'neg'}">${r.net_alpha_annual_bp.toFixed(0)}bp/yr</div></div>
            </div>
            <p class="muted small">Cost says nothing until divided by alpha velocity — the same signal is
            profitable at a 10-day hold and a donation to market makers at a 1-day hold.</p>`,
    },
    'leveraged-etf-decay': {
        label: 'Leveraged ETF Decay',
        call: (b) => api.calcLeveragedEtfDecay(b),
        fields: [
            { key: 'leverage', label: 'Leverage (2, 3, −1, −2…)', def: 3 },
            { key: 'index_return_pct', label: 'Index return over period (%)', def: 0 },
            { key: 'index_vol_pct', label: 'Index vol (%/yr)', def: 40 },
            { key: 'years', label: 'Period (years)', def: 1 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Expected ETF return</div>
                    <div class="value ${r.letf_return_pct >= 0 ? 'pos' : 'neg'}">${r.letf_return_pct.toFixed(1)}%</div>
                    <div class="small muted">naive k×R says ${r.naive_return_pct.toFixed(1)}%</div></div>
                <div class="card"><div class="label">Vol drag</div>
                    <div class="value neg">−${r.vol_drag_pp.toFixed(1)}pp</div>
                    <div class="small muted">vs ${r.compounded_no_vol_pct.toFixed(1)}% levered compounding alone</div></div>
            </div>
            <p class="muted small">(1+R)^k · e^(−k(k−1)σ²T/2) — smooth trends compound in your favor,
            chop bleeds: a 3× on a flat 40%-vol index loses ~38% in a year.</p>`,
    },
    'short-carry': {
        label: 'Short Carry',
        call: (b) => api.calcShortCarry(b),
        fields: [
            { key: 'short_price', label: 'Short price ($)', def: 50 },
            { key: 'shares', label: 'Shares', def: 100 },
            { key: 'borrow_fee_pct', label: 'Borrow fee (%/yr)', def: 8 },
            { key: 'cash_rate_pct', label: 'Rebate on proceeds (%/yr)', def: 5 },
            { key: 'annual_dividend', label: 'Dividend ($/sh/yr)', def: 2 },
            { key: 'holding_days', label: 'Holding days', def: 90 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Net carry</div>
                    <div class="value ${r.positive_carry ? 'pos' : 'neg'}">${r.net_carry_rate_pct.toFixed(2)}%/yr</div>
                    <div class="small muted">fee − rebate + dividend yield</div></div>
                <div class="card"><div class="label">Breakeven decline</div>
                    <div class="value">${r.breakeven_decline_pct.toFixed(2)}%</div>
                    <div class="small muted">over the holding period</div></div>
                <div class="card"><div class="label">Cost</div>
                    <div class="value">$${r.carry_cost.toFixed(0)}</div>
                    <div class="small muted">$${r.daily_cost.toFixed(2)}/day</div></div>
            </div>
            <p class="muted small">${r.positive_carry ? 'Positive carry — the rebate pays you to be short.' : 'The stock must fall this much just to cover carry — squeeze-name fees compound daily.'}</p>`,
    },
    'impermanent-loss': {
        label: 'Impermanent Loss',
        call: (b) => api.calcImpermanentLoss(b),
        fields: [
            { key: 'price_ratio', label: 'Price ratio (end ÷ entry)', def: 2 },
            { key: 'holding_days', label: 'Holding period (days)', def: 90 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Impermanent loss</div>
                    <div class="value neg">${r.il_pct.toFixed(2)}%</div>
                    <div class="small muted">LP worth ${r.lp_vs_hodl_pct.toFixed(1)}% of HODL</div></div>
                <div class="card"><div class="label">Breakeven fee APR</div>
                    <div class="value">${r.breakeven_fee_apr_pct.toFixed(1)}%</div>
                    <div class="small muted">pool fees must beat this</div></div>
            </div>
            <table class="gs-table">
                <thead><tr><th>Ratio</th><th>IL</th></tr></thead>
                <tbody>${r.curve.map(p => `
                    <tr><td>${p.price_ratio}×</td>
                        <td class="${p.il_pct < -1 ? 'neg' : ''}">${p.il_pct.toFixed(2)}%</td></tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">50/50 constant-product pool: IL = 2√r/(1+r) − 1, symmetric in r and
            1/r. The ±4× waypoint costs exactly 20% vs holding.</p>`,
    },
    'average-down': {
        label: 'Average Down',
        call: (b) => api.calcAverageDown(b),
        fields: [
            { key: 'current_shares', label: 'Current shares', def: 100 },
            { key: 'current_avg_cost', label: 'Current avg cost ($)', def: 50 },
            { key: 'add_shares', label: 'Shares to add', def: 100 },
            { key: 'add_price', label: 'Add price ($)', def: 40 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">New average</div>
                    <div class="value">$${r.new_avg_cost.toFixed(2)}</div>
                    <div class="small muted">${r.new_shares} shares · $${Math.round(r.total_capital).toLocaleString()} in</div></div>
                <div class="card"><div class="label">Bounce to breakeven</div>
                    <div class="value">${r.bounce_needed_after_pct.toFixed(1)}%</div>
                    <div class="small muted">was ${r.bounce_needed_before_pct.toFixed(1)}% before the add</div></div>
                <div class="card"><div class="label">Exposure added</div>
                    <div class="value neg">+${r.exposure_increase_pct.toFixed(0)}%</div>
                    <div class="small muted">$${Math.round(r.capital_added).toLocaleString()} · sitting ${r.unrealized_at_add < 0 ? '−' : '+'}$${Math.abs(Math.round(r.unrealized_at_add)).toLocaleString()} unrealized</div></div>
            </div>
            <p class="muted small">The add always lowers the breakeven bounce — the table prices what that
            costs in fresh capital at risk.</p>`,
    },
    'futures-sizing': {
        label: 'Futures Sizing',
        call: (b) => api.calcFuturesSizing(b),
        fields: [
            { key: 'account', label: 'Account ($)', def: 100000 },
            { key: 'risk_pct', label: 'Risk per trade (%)', def: 1 },
            { key: 'entry', label: 'Entry', def: 5000 },
            { key: 'stop', label: 'Stop', def: 4990 },
            { key: 'tick_size', label: 'Tick size', def: 0.25 },
            { key: 'tick_value', label: 'Tick value ($)', def: 12.5 },
            { key: 'initial_margin', label: 'Initial margin ($/contract)', def: 15000 },
            { key: 'margin_cap_pct', label: 'Margin cap (% of account)', def: 50 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Contracts</div>
                    <div class="value ${r.contracts > 0 ? 'pos' : 'neg'}">${r.contracts}</div>
                    <div class="small muted">${r.contracts_by_risk} by risk · ${r.contracts_by_margin} by margin — ${esc(r.binding_constraint)} binds</div></div>
                <div class="card"><div class="label">Risk</div>
                    <div class="value">$${r.total_risk.toFixed(0)}</div>
                    <div class="small muted">$${r.risk_per_contract.toFixed(0)}/contract · ${r.stop_distance_points} pts × $${r.dollars_per_point.toFixed(0)}/pt</div></div>
                <div class="card"><div class="label">Margin / notional</div>
                    <div class="value">${r.margin_utilization_pct.toFixed(0)}%</div>
                    <div class="small muted">$${Math.round(r.margin_used).toLocaleString()} margin · $${Math.round(r.notional).toLocaleString()} notional</div></div>
            </div>
            <p class="muted small">Size = min(risk budget ÷ stop cost, margin cap ÷ initial margin). When
            exchanges hike margin on vol spikes, the binding constraint flips — the report says which.</p>`,
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
    'iv-cone': {
        label: 'IV Cone',
        call: (b) => api.calcIvCone({
            spot: b.spot,
            term: String(b.term).split(';').map(s => s.trim()).filter(Boolean).map(s => {
                const [d, iv] = s.split(',').map(x => Number(x.trim()));
                return { days: d, iv_pct: iv };
            }),
        }),
        fields: [
            { key: 'spot', label: 'Spot ($)', def: 100 },
            { key: 'term', label: 'Term: days,IV%; …', def: '5,25; 21,22; 63,20; 252,18', text: true },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid term structure</span>';
            return `
            <table class="gs-table">
                <thead><tr><th>Horizon</th><th>IV</th><th>±1σ move</th><th>1σ range</th><th>2σ range</th></tr></thead>
                <tbody>${r.map(row => `
                    <tr>
                        <td>${row.days}d</td>
                        <td>${row.iv_pct.toFixed(1)}%</td>
                        <td>±${row.move_1s_pct.toFixed(2)}%</td>
                        <td>$${row.low_1s.toFixed(2)} – $${row.high_1s.toFixed(2)}</td>
                        <td>$${row.low_2s.toFixed(2)} – $${row.high_2s.toFixed(2)}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Where the options market PRICES the underlying to live (σ√T bands per
            term point) — pair with the realized Vol Cone to spot rich/cheap horizons.</p>`;
        },
    },
    'fund-fees': {
        label: '2-and-20 Fees',
        call: (b) => api.calcFundFees({
            initial_investment: b.initial_investment,
            gross_returns_pct: String(b.gross_returns_pct).split(/[\s,]+/).map(Number).filter(x => isFinite(x)),
            management_fee_pct: b.management_fee_pct,
            incentive_fee_pct: b.incentive_fee_pct,
            hurdle_pct: b.hurdle_pct,
        }),
        fields: [
            { key: 'initial_investment', label: 'Investment ($)', def: 100000 },
            { key: 'gross_returns_pct', label: 'Gross returns %/yr (comma-sep)', def: '20, -10, 15, 12, 8', text: true },
            { key: 'management_fee_pct', label: 'Management fee (%)', def: 2 },
            { key: 'incentive_fee_pct', label: 'Incentive fee (%)', def: 20 },
            { key: 'hurdle_pct', label: 'Hurdle (%/yr)', def: 0 },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid fee inputs</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">Net CAGR</div>
                    <div class="value ${r.net_cagr_pct >= 0 ? 'pos' : 'neg'}">${r.net_cagr_pct.toFixed(2)}%</div>
                    <div class="small muted">vs ${r.gross_cagr_pct.toFixed(2)}% gross</div></div>
                <div class="card"><div class="label">Fee drag</div>
                    <div class="value neg">−${r.fee_drag_pp.toFixed(2)}pp/yr</div>
                    <div class="small muted">$${Math.round(r.total_fees).toLocaleString()} total fees</div></div>
                <div class="card"><div class="label">Final NAV</div>
                    <div class="value">$${Math.round(r.final_net_nav).toLocaleString()}</div>
                    <div class="small muted">vs $${Math.round(r.final_gross_nav).toLocaleString()} gross</div></div>
            </div>
            <table class="gs-table">
                <thead><tr><th>Year</th><th>Gross</th><th>Mgmt</th><th>Incentive</th><th>Net NAV</th></tr></thead>
                <tbody>${r.years.map(y => `
                    <tr>
                        <td>${y.year}</td>
                        <td class="${y.gross_return_pct >= 0 ? 'pos' : 'neg'}">${y.gross_return_pct.toFixed(1)}%</td>
                        <td>$${y.management_fee.toFixed(0)}</td>
                        <td>$${y.incentive_fee.toFixed(0)}</td>
                        <td>$${Math.round(y.net_nav).toLocaleString()}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Mgmt on beginning NAV; incentive only above max(high-water mark,
            hurdle) — the HWM blocks double-charging after a down year.</p>`;
        },
    },
    'vol-rich-cheap': {
        label: 'Vol Rich/Cheap',
        call: (b) => api.volRichCheap(b.symbol, {
            term: String(b.term).split(';').map(s => s.trim()).filter(Boolean).map(s => {
                const [d, iv] = s.split(',').map(x => Number(x.trim()));
                return [d, iv];
            }),
            years: b.years,
        }),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'SPY', text: true },
            { key: 'term', label: 'IV term: days,IV%; …', def: '5,18; 21,17; 63,16', text: true },
            { key: 'years', label: 'Realized lookback (years)', def: 5, int: true },
        ],
        render: (r) => `
            <table class="gs-table">
                <thead><tr><th>Horizon</th><th>IV</th><th>Realized</th><th>RV rank</th><th>Spread</th><th>Regime</th></tr></thead>
                <tbody>${r.rows.map(row => `
                    <tr>
                        <td>${row.days}d</td>
                        <td>${row.iv_pct.toFixed(1)}%</td>
                        <td>${row.realized_pct.toFixed(1)}%</td>
                        <td>${row.realized_rank_pct.toFixed(0)}%</td>
                        <td class="${row.vol_spread_pct >= 0 ? 'pos' : 'neg'}">${row.vol_spread_pct.toFixed(1)}pp</td>
                        <td class="${row.premium_regime === 'rich' ? 'pos' : row.premium_regime === 'cheap' ? 'neg' : ''}">${esc(row.premium_regime)}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Live IV term vs the realized cone's current reading per horizon —
            "rich" premium AND a low realized rank is the short-vol setup; "cheap" against an elevated
            rank says the market is underpricing motion. ${r.days_analyzed} closes on ${esc(r.symbol)}.</p>`,
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
    'rebalance-bands': {
        label: '5/25 Bands',
        call: (b) => api.calcRebalanceBands({
            assets: String(b.assets).split(';').map(s => s.trim()).filter(Boolean).map(s => {
                const i = s.lastIndexOf(',');
                const j = s.lastIndexOf(',', i - 1);
                return {
                    name: s.slice(0, j).trim(),
                    target_weight_pct: Number(s.slice(j + 1, i)),
                    current_weight_pct: Number(s.slice(i + 1)),
                };
            }),
            absolute_band_pp: b.absolute_band_pp,
            relative_band_pct: b.relative_band_pct,
        }),
        fields: [
            { key: 'assets', label: 'Assets: name,target%,current%; …', def: 'US equity,40,46; Intl,20,18; Bonds,30,29; EM small value,4,5.2', text: true },
            { key: 'absolute_band_pp', label: 'Absolute band (pp)', def: 5 },
            { key: 'relative_band_pct', label: 'Relative band (% of target)', def: 25 },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid assets</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">Action</div>
                    <div class="value ${r.any_breach ? 'neg' : 'pos'}">${r.any_breach ? 'REBALANCE' : 'hold'}</div></div>
            </div>
            <table class="gs-table">
                <thead><tr><th>Asset</th><th>Target</th><th>Current</th><th>Drift</th><th>Band</th><th>Trade</th></tr></thead>
                <tbody>${r.rows.map(row => `
                    <tr>
                        <td>${esc(row.name)}</td>
                        <td>${row.target_weight_pct.toFixed(1)}%</td>
                        <td>${row.current_weight_pct.toFixed(1)}%</td>
                        <td class="${row.breached ? 'neg' : ''}">${(row.drift_pp >= 0 ? '+' : '') + row.drift_pp.toFixed(1)}pp</td>
                        <td>±${row.band_pp.toFixed(1)}pp (${esc(row.binding_rule)})</td>
                        <td>${row.breached ? (row.trade_pp >= 0 ? '+' : '') + row.trade_pp.toFixed(1) + 'pp' : '—'}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Swedroe 5/25: trade only past the narrower of 5pp absolute or 25%-of-
            target relative — big sleeves bind absolute, small sleeves relative.</p>`;
        },
    },
    'curve-trade': {
        label: 'Curve Trade',
        call: (b) => api.calcCurveTrade({
            // "label,yield%,dv01_per_1M; ..." — 2 legs = spread, 3 = fly.
            legs: String(b.legs).split(';').map(s => s.trim()).filter(Boolean).map(s => {
                const [label, y, d] = s.split(',').map(x => x.trim());
                return { label, yield_pct: Number(y), dv01_per_million: Number(d) };
            }),
            anchor_notional_mm: b.anchor_notional_mm,
        }),
        fields: [
            { key: 'legs', label: 'Legs: label,yield%,DV01/$1M; … (2 = spread, 3 = fly)', def: '2y,4.6,190; 5y,4.35,440; 10y,4.2,880', text: true },
            { key: 'anchor_notional_mm', label: 'Anchor notional ($M, first leg / belly)', def: 10 },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">need 2 or 3 valid legs</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">${r.kind === 'butterfly' ? 'Fly level' : 'Spread'}</div>
                    <div class="value">${r.level_bp.toFixed(1)}bp</div></div>
                <div class="card"><div class="label">P&amp;L per bp</div>
                    <div class="value">$${r.pl_per_bp.toFixed(0)}</div>
                    <div class="small muted">DV01-neutral ${esc(r.kind)}</div></div>
            </div>
            <table class="gs-table">
                <thead><tr><th>Leg</th><th>Notional</th><th>DV01</th></tr></thead>
                <tbody>${r.legs.map(l => `
                    <tr><td>${esc(l.label)}</td><td>$${l.notional_mm.toFixed(2)}M</td>
                        <td>$${l.dv01_total.toFixed(0)}/bp</td></tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Spread legs carry equal DV01 (pure curve, no outright duration); the fly
            puts half the belly DV01 on each wing, immunizing level and slope.</p>`;
        },
    },
    'cheapest-to-deliver': {
        label: 'Cheapest to Deliver',
        call: (b) => api.calcCheapestToDeliver({
            futures_price: b.futures_price,
            days_to_delivery: b.days_to_delivery,
            // "name,clean,CF[,accrued_now,accrued_delivery]; ..."
            basket: String(b.basket).split(';').map(s => s.trim()).filter(Boolean).map(s => {
                const [name, clean, cf, an, ad] = s.split(',').map(x => x.trim());
                return {
                    name,
                    clean_price: Number(clean),
                    conversion_factor: Number(cf),
                    accrued_now: Number(an) || 0,
                    accrued_at_delivery: Number(ad) || 0,
                };
            }),
        }),
        fields: [
            { key: 'futures_price', label: 'Futures price', def: 110 },
            { key: 'days_to_delivery', label: 'Days to delivery', def: 90 },
            { key: 'basket', label: 'Basket: name,clean,CF[,accr_now,accr_dlv]; …', def: 'T 4.25 2034,99,0.9; T 3.875 2033,98,0.9', text: true },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid basket</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">CTD</div>
                    <div class="value pos">${esc(r.ctd_name)}</div>
                    <div class="small muted">implied repo ${r.ctd_implied_repo_pct.toFixed(2)}%</div></div>
            </div>
            <table class="gs-table">
                <thead><tr><th>Bond</th><th>Gross basis</th><th>Implied repo</th><th></th></tr></thead>
                <tbody>${r.rows.map(row => `
                    <tr>
                        <td>${esc(row.name)}</td>
                        <td>${row.gross_basis.toFixed(3)}</td>
                        <td class="${row.implied_repo_pct >= 0 ? 'pos' : 'neg'}">${row.implied_repo_pct.toFixed(2)}%</td>
                        <td>${row.is_ctd ? '★ CTD' : ''}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">The short delivers whatever finances best — highest implied repo wins;
            everything else in the basket trades as an option off the CTD.</p>`;
        },
    },
    'crack-spread': {
        label: 'Crack Spread',
        call: (b) => api.calcCrackSpread(b),
        fields: [
            { key: 'crude', label: 'Crude ($/bbl)', def: 80 },
            { key: 'gasoline', label: 'Gasoline ($/gal)', def: 2.5 },
            { key: 'distillate', label: 'Distillate/ULSD ($/gal)', def: 2.8 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">3-2-1 crack</div>
                    <div class="value ${r.crack_321 >= 0 ? 'pos' : 'neg'}">$${r.crack_321.toFixed(2)}/bbl</div>
                    <div class="small muted">${r.margin_pct.toFixed(1)}% of crude</div></div>
                <div class="card"><div class="label">Gasoline crack</div>
                    <div class="value">$${r.gasoline_crack.toFixed(2)}</div></div>
                <div class="card"><div class="label">Distillate crack</div>
                    <div class="value">$${r.distillate_crack.toFixed(2)}</div></div>
            </div>
            <p class="muted small">3 bbl crude → 2 gasoline + 1 distillate at 42 gal/bbl — the refiner
            margin behind XLE/CRAK earnings and the classic CL/RB/HO futures spread.</p>`,
    },
    'crush-spread': {
        label: 'Crush Spread',
        call: (b) => api.calcCrushSpread(b),
        fields: [
            { key: 'beans', label: 'Soybeans ($/bu)', def: 12 },
            { key: 'meal', label: 'Soybean meal ($/short ton)', def: 350 },
            { key: 'oil', label: 'Soybean oil ($/lb)', def: 0.45 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Board crush</div>
                    <div class="value ${r.crush >= 0 ? 'pos' : 'neg'}">$${r.crush.toFixed(2)}/bu</div>
                    <div class="small muted">${r.margin_pct.toFixed(1)}% of beans</div></div>
                <div class="card"><div class="label">Product value</div>
                    <div class="value">$${(r.meal_value_per_bu + r.oil_value_per_bu).toFixed(2)}</div>
                    <div class="small muted">meal $${r.meal_value_per_bu.toFixed(2)} + oil $${r.oil_value_per_bu.toFixed(2)}</div></div>
            </div>
            <p class="muted small">CME yields: 1 bu (60 lb) → 44 lb meal (0.022 ton) + 11 lb oil — the
            ZS/ZM/ZL board crush.</p>`,
    },
    'spark-spread': {
        label: 'Spark Spread',
        call: (b) => api.calcSparkSpread(b),
        fields: [
            { key: 'power', label: 'Power ($/MWh)', def: 50 },
            { key: 'fuel', label: 'Fuel ($/MMBtu)', def: 4 },
            { key: 'heat_rate', label: 'Heat rate (MMBtu/MWh; gas ~7, coal ~10)', def: 7.5 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Spread</div>
                    <div class="value ${r.spread >= 0 ? 'pos' : 'neg'}">$${r.spread.toFixed(2)}/MWh</div>
                    <div class="small muted">fuel cost $${r.fuel_cost_per_mwh.toFixed(2)}</div></div>
                <div class="card"><div class="label">Breakeven power</div>
                    <div class="value">$${r.breakeven_power.toFixed(2)}</div></div>
                <div class="card"><div class="label">Market heat rate</div>
                    <div class="value">${r.market_implied_heat_rate.toFixed(1)}</div>
                    <div class="small muted">above plant HR = in the money</div></div>
            </div>
            <p class="muted small">Generation margin: power − fuel × heat rate. Gas fuel = spark spread;
            coal fuel at ~10 heat rate = dark spread.</p>`,
    },
    'sum-of-parts': {
        label: 'Sum of Parts',
        call: (b) => api.calcSumOfParts({
            // "Name,value; ..." segment syntax.
            segments: String(b.segments).split(';').map(s => s.trim()).filter(Boolean).map(s => {
                const i = s.lastIndexOf(',');
                return { name: s.slice(0, i).trim(), value: Number(s.slice(i + 1)) };
            }),
            net_debt: b.net_debt,
            conglomerate_discount_pct: b.conglomerate_discount_pct,
            market_cap: b.market_cap,
        }),
        fields: [
            { key: 'segments', label: 'Segments: name,value; …', def: 'Cloud,60; Ads,30; Hardware,10', text: true },
            { key: 'net_debt', label: 'Net debt ($, negative = cash)', def: 20 },
            { key: 'conglomerate_discount_pct', label: 'Conglomerate discount (%)', def: 15 },
            { key: 'market_cap', label: 'Market cap ($)', def: 60 },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid SOTP inputs</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">SOTP equity value</div>
                    <div class="value">$${r.sotp_equity_value.toFixed(1)}</div>
                    <div class="small muted">NAV $${r.nav.toFixed(1)} less discount</div></div>
                <div class="card"><div class="label">Upside</div>
                    <div class="value ${r.upside_pct >= 0 ? 'pos' : 'neg'}">${(r.upside_pct >= 0 ? '+' : '') + r.upside_pct.toFixed(1)}%</div></div>
                <div class="card"><div class="label">Market-implied discount</div>
                    <div class="value">${r.market_implied_discount_pct.toFixed(1)}%</div>
                    <div class="small muted">what the market charges vs NAV</div></div>
            </div>
            <table class="gs-table">
                <thead><tr><th>Segment</th><th>Value</th><th>Weight</th></tr></thead>
                <tbody>${r.rows.map(s => `
                    <tr><td>${esc(s.name)}</td><td>$${s.value.toFixed(1)}</td>
                        <td>${s.weight_pct.toFixed(1)}%</td></tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Undiscounted SOTP upside is the oldest value trap going — holdcos sit
            10–30% below NAV persistently, so the discount is priced in up front.</p>`;
        },
    },
    'odd-lot-tender': {
        label: 'Odd-Lot Tender',
        call: (b) => api.calcOddLotTender({ ...b, accounts: b.accounts > 0 ? b.accounts : null }),
        fields: [
            { key: 'market_price', label: 'Market price ($)', def: 19 },
            { key: 'tender_price', label: 'Tender price ($)', def: 20 },
            { key: 'shares', label: 'Shares (≤99 for priority)', def: 99, int: true },
            { key: 'fees', label: 'Fees per account ($)', def: 1 },
            { key: 'days_to_payment', label: 'Days to payment', def: 36.5 },
            { key: 'accounts', label: 'Accounts (0 = 1)', def: 1, int: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Profit / account</div>
                    <div class="value ${r.profit_per_account >= 0 ? 'pos' : 'neg'}">$${r.profit_per_account.toFixed(2)}</div>
                    <div class="small muted">on $${r.capital_per_account.toFixed(0)} capital</div></div>
                <div class="card"><div class="label">Return</div>
                    <div class="value">${r.return_pct.toFixed(2)}%</div>
                    <div class="small muted">${r.annualized_pct.toFixed(1)}% annualized</div></div>
                <div class="card"><div class="label">Odd-lot priority</div>
                    <div class="value ${r.odd_lot_priority ? 'pos' : 'neg'}">${r.odd_lot_priority ? 'YES — no proration' : 'NO — proration risk'}</div>
                    <div class="small muted">total $${r.total_profit.toFixed(0)} across accounts</div></div>
            </div>
            <p class="muted small">Tenders with odd-lot priority take ≤99-share holders in full ahead of
            proration — tiny in dollars, large annualized, and it scales per account.</p>`,
    },
    'cef-discount': {
        label: 'CEF Discount',
        call: (b) => api.calcCefDiscount({
            price: b.price,
            nav: b.nav,
            annual_distribution: b.annual_distribution,
            mean_discount_pct: b.std_discount_pct > 0 ? b.mean_discount_pct : null,
            std_discount_pct: b.std_discount_pct > 0 ? b.std_discount_pct : null,
        }),
        fields: [
            { key: 'price', label: 'Market price ($)', def: 9.2 },
            { key: 'nav', label: 'NAV ($)', def: 10 },
            { key: 'annual_distribution', label: 'Annual distribution ($/sh)', def: 0.92 },
            { key: 'mean_discount_pct', label: 'Historical mean discount (%)', def: -5 },
            { key: 'std_discount_pct', label: 'Discount std dev (%, 0 = skip z)', def: 2 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">${r.discount_pct < 0 ? 'Discount' : 'Premium'}</div>
                    <div class="value ${r.discount_pct < 0 ? 'pos' : 'neg'}">${r.discount_pct.toFixed(2)}%</div>
                    ${r.z_score != null ? `<div class="small ${r.z_score <= -1 ? 'pos' : 'muted'}">z = ${r.z_score.toFixed(2)} vs own history</div>` : ''}</div>
                ${r.yield_on_price_pct != null ? `<div class="card"><div class="label">Yield pickup</div>
                    <div class="value">${r.yield_on_price_pct.toFixed(2)}%</div>
                    <div class="small muted">vs ${r.yield_on_nav_pct.toFixed(2)}% on NAV</div></div>` : ''}
            </div>
            <p class="muted small">CEF discounts mean-revert — the screen wants deep-negative z (cheap vs
            its OWN history), not just a wide absolute discount.</p>`,
    },
    'adr-premium': {
        label: 'ADR Premium',
        call: (b) => api.calcAdrPremium(b),
        fields: [
            { key: 'adr_price', label: 'ADR price ($)', def: 73.5 },
            { key: 'ordinary_price_local', label: 'Ordinary price (local ccy)', def: 1000 },
            { key: 'usd_per_local', label: 'USD per local unit', def: 0.007 },
            { key: 'ordinaries_per_adr', label: 'Ordinaries per ADR', def: 10 },
            { key: 'conversion_fee_pct', label: 'Round-trip fee (%)', def: 1 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Parity</div>
                    <div class="value">$${r.parity_usd.toFixed(2)}</div></div>
                <div class="card"><div class="label">${r.premium_pct >= 0 ? 'Premium' : 'Discount'}</div>
                    <div class="value ${Math.abs(r.premium_pct) > 2 ? 'neg' : ''}">${r.premium_pct.toFixed(2)}%</div></div>
                <div class="card"><div class="label">Capturable after fees</div>
                    <div class="value ${r.arb_exists ? 'pos' : 'neg'}">${r.capturable_pct.toFixed(2)}%</div>
                    <div class="small muted">${r.arb_exists ? 'ARB — gap clears friction' : 'friction eats the gap'}</div></div>
            </div>
            <p class="muted small">ADR vs ordinary × FX × ratio. Persistent premiums signal conversion
            restrictions; fleeting ones are the cross-listing arb.</p>`,
    },
    'sbc-dilution': {
        label: 'SBC Dilution',
        call: (b) => api.calcSbcDilution(b),
        fields: [
            { key: 'market_cap', label: 'Market cap ($M)', def: 100000 },
            { key: 'annual_sbc', label: 'Annual SBC ($M)', def: 3000 },
            { key: 'annual_buybacks', label: 'Annual buybacks ($M)', def: 5000 },
            { key: 'annual_revenue', label: 'Annual revenue ($M, 0 = skip)', def: 40000 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">SBC dilution</div>
                    <div class="value neg">−${r.sbc_dilution_pct.toFixed(2)}%/yr</div>
                    ${r.sbc_to_revenue_pct != null ? `<div class="small muted">${r.sbc_to_revenue_pct.toFixed(1)}% of revenue</div>` : ''}</div>
                <div class="card"><div class="label">Net buyback yield</div>
                    <div class="value ${r.net_buyback_yield_pct >= 0 ? 'pos' : 'neg'}">${r.net_buyback_yield_pct.toFixed(2)}%</div>
                    <div class="small muted">${r.gross_buyback_yield_pct.toFixed(2)}% gross</div></div>
                ${r.buyback_consumed_by_sbc_pct != null ? `<div class="card"><div class="label">Buyback consumed by SBC</div>
                    <div class="value ${r.buyback_consumed_by_sbc_pct >= 100 ? 'neg' : ''}">${r.buyback_consumed_by_sbc_pct.toFixed(0)}%</div></div>` : ''}
            </div>
            <p class="muted small">Gross shareholder-yield screens miss SBC re-issuing shares out the back
            door — the net row is the real return of capital.</p>`,
    },
    'merger-arb': {
        label: 'Merger Arb',
        call: (b) => api.calcMergerArb(b),
        fields: [
            { key: 'current_price', label: 'Current price ($)', def: 98 },
            { key: 'deal_price', label: 'Deal price ($)', def: 100 },
            { key: 'break_price', label: 'Break price ($)', def: 90 },
            { key: 'days_to_close', label: 'Days to close', def: 73 },
            { key: 'estimated_probability', label: 'Your completion prob (0–1)', def: 0.9 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Spread</div>
                    <div class="value ${r.gross_spread_pct >= 0 ? 'pos' : 'neg'}">${r.gross_spread_pct.toFixed(2)}%</div>
                    <div class="small muted">${r.annualized_spread_pct.toFixed(1)}% annualized</div></div>
                <div class="card"><div class="label">Market-implied prob</div>
                    <div class="value">${(r.implied_probability * 100).toFixed(0)}%</div></div>
                <div class="card"><div class="label">EV at your prob</div>
                    <div class="value ${r.expected_return_pct >= 0 ? 'pos' : 'neg'}">${r.expected_return_pct.toFixed(2)}%</div>
                    <div class="small muted">$${r.expected_price.toFixed(2)} expected</div></div>
                <div class="card"><div class="label">Reward : risk</div>
                    <div class="value">${r.reward_risk != null ? r.reward_risk.toFixed(2) : '—'}</div>
                    <div class="small neg">${r.downside_pct.toFixed(1)}% on break</div></div>
            </div>
            <p class="muted small">P = p·deal + (1−p)·break solves the completion probability the market is
            charging — the edge is the gap between that and your estimate.</p>`,
    },
    'buyback-accretion': {
        label: 'Buyback Accretion',
        call: (b) => api.calcBuybackAccretion(b),
        fields: [
            { key: 'net_income', label: 'Net income ($M)', def: 1000 },
            { key: 'shares_outstanding', label: 'Shares outstanding (M)', def: 100 },
            { key: 'share_price', label: 'Share price ($)', def: 200 },
            { key: 'buyback_amount', label: 'Buyback ($M)', def: 2000 },
            { key: 'funding_rate', label: 'Funding rate (decimal)', def: 0.05 },
            { key: 'tax_rate', label: 'Tax rate (decimal)', def: 0.21 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">EPS accretion</div>
                    <div class="value ${r.is_accretive ? 'pos' : 'neg'}">${(r.accretion_pct >= 0 ? '+' : '') + r.accretion_pct.toFixed(2)}%</div>
                    <div class="small muted">$${r.old_eps.toFixed(2)} → $${r.new_eps.toFixed(2)}</div></div>
                <div class="card"><div class="label">Breakeven P/E</div>
                    <div class="value">${r.breakeven_pe != null ? r.breakeven_pe.toFixed(1) : '∞'}</div>
                    <div class="small ${r.is_accretive ? 'pos' : 'neg'}">current ${r.current_pe.toFixed(1)} — ${r.is_accretive ? 'accretive' : 'DILUTIVE'}</div></div>
                <div class="card"><div class="label">Shares retired</div>
                    <div class="value">${r.shares_retired.toFixed(1)}M</div></div>
            </div>
            <p class="muted small">A buyback is accretive only while the earnings yield beats the after-tax
            funding rate — breakeven P/E = 1 / (rate × (1 − tax)).</p>`,
    },
    'scale-out': {
        label: 'Scale-Out Ladder',
        call: (b) => api.calcScaleOut({
            entry: b.entry,
            stop: b.stop,
            total_shares: b.total_shares,
            breakeven_after_first: b.breakeven_after_first === 1,
            // "price:shares; ..." nearest target first.
            legs: String(b.legs).split(';').map(s => s.trim()).filter(Boolean).map(s => {
                const [t, sh] = s.split(',').map(x => Number(x.trim()));
                return { target_price: t, shares: sh };
            }),
        }),
        fields: [
            { key: 'entry', label: 'Entry ($)', def: 100 },
            { key: 'stop', label: 'Stop ($)', def: 95 },
            { key: 'total_shares', label: 'Total shares', def: 300 },
            { key: 'legs', label: 'Targets: price,shares; …', def: '105,100; 110,100; 115,100', text: true },
            { key: 'breakeven_after_first', label: 'Stop to breakeven after T1 (1 = yes)', def: 1, int: true },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid ladder</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">Total risk</div>
                    <div class="value">$${r.total_risk.toFixed(0)}</div>
                    <div class="small muted">$${r.risk_per_share.toFixed(2)}/share · ${r.is_long ? 'long' : 'short'}</div></div>
                <div class="card"><div class="label">Max R (all targets)</div>
                    <div class="value pos">${r.max_r.toFixed(2)}R</div></div>
            </div>
            <table class="gs-table">
                <thead><tr><th>Scenario</th><th>P&amp;L</th><th>R</th></tr></thead>
                <tbody>${r.scenarios.map(s => `
                    <tr>
                        <td>${esc(s.label.replace(/_/g, ' '))}</td>
                        <td class="${s.pnl >= 0 ? 'pos' : 'neg'}">$${s.pnl.toFixed(0)}</td>
                        <td class="${s.r_multiple >= 0 ? 'pos' : 'neg'}">${s.r_multiple.toFixed(2)}R</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Every stop-out path priced before entry — ladders feel safe but cap the
            right tail; the scenario table makes the tradeoff concrete.</p>`;
        },
    },
    'variance-risk-premium': {
        label: 'Variance Risk Premium',
        call: (b) => api.calcVarianceRiskPremium(b),
        fields: [
            { key: 'implied_vol_pct', label: 'Implied vol (%)', def: 20 },
            { key: 'realized_vol_pct', label: 'Realized vol (%)', def: 16 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">VRP</div>
                    <div class="value ${r.vrp_variance_points >= 0 ? 'pos' : 'neg'}">${r.vrp_variance_points.toFixed(0)}</div>
                    <div class="small muted">variance points (IV² − RV²)</div></div>
                <div class="card"><div class="label">Vol spread</div>
                    <div class="value">${r.vol_spread_pct.toFixed(1)}pp</div>
                    <div class="small muted">ratio ${r.iv_rv_ratio.toFixed(2)} · ${esc(r.premium_regime)}</div></div>
            </div>
            <p class="muted small">Carr-Wu variance risk premium — persistently positive on equity indexes;
            pair the realized leg with the Vol Cone tab's current reading.</p>`,
    },
    'options-quick-math': {
        label: 'Options Quick Math',
        call: (b) => api.calcOptionsQuickMath(b),
        fields: [
            { key: 'spot', label: 'Spot ($)', def: 100 },
            { key: 'iv_pct', label: 'IV (%)', def: 25 },
            { key: 'days_to_expiry', label: 'Days to expiry', def: 21 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Daily move</div>
                    <div class="value">±${r.daily_move_rule16_pct.toFixed(2)}%</div>
                    <div class="small muted">rule of 16 · exact ${r.daily_move_exact_pct.toFixed(2)}% · weekly ${r.weekly_move_exact_pct.toFixed(2)}%</div></div>
                <div class="card"><div class="label">ATM straddle</div>
                    <div class="value">$${r.straddle_approx.toFixed(2)}</div>
                    <div class="small muted">0.8·S·σ√T · exact $${r.straddle_exact.toFixed(2)} (${r.straddle_approx_error_pct >= 0 ? '+' : ''}${r.straddle_approx_error_pct.toFixed(2)}%)</div></div>
                <div class="card"><div class="label">ATM call</div>
                    <div class="value">$${r.atm_call_approx.toFixed(2)}</div>
                    <div class="small muted">0.4·S·σ√T · exact $${r.atm_call_exact.toFixed(2)}</div></div>
            </div>
            <p class="muted small">The desk shortcuts with their exact Black-Scholes counterparts beside
            them — the 0.8 constant is really √(2/π)·2 ≈ 0.7979.</p>`,
    },
    'lynch-fair-value': {
        label: 'Lynch Fair Value',
        call: (b) => api.calcLynchFairValue(b),
        fields: [
            { key: 'eps_growth_pct', label: 'EPS growth (%/yr)', def: 20 },
            { key: 'dividend_yield_pct', label: 'Dividend yield (%)', def: 2 },
            { key: 'pe_ratio', label: 'P/E', def: 11 },
            { key: 'eps', label: 'EPS ($, 0 = skip fair price)', def: 5 },
            { key: 'price', label: 'Price ($, 0 = skip upside)', def: 55 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Adjusted PEG</div>
                    <div class="value ${r.adjusted_peg >= 1.5 ? 'pos' : r.adjusted_peg < 1 ? 'neg' : ''}">${r.adjusted_peg.toFixed(2)}</div>
                    <div class="small muted">${esc(r.band)} — (growth + yield) ÷ P/E</div></div>
                ${r.fair_price != null ? `<div class="card"><div class="label">Fair price (P/E = growth)</div>
                    <div class="value">$${r.fair_price.toFixed(2)}</div>
                    ${r.upside_pct != null ? `<div class="small ${r.upside_pct >= 0 ? 'pos' : 'neg'}">${(r.upside_pct >= 0 ? '+' : '') + r.upside_pct.toFixed(1)}% vs price</div>` : ''}</div>` : ''}
            </div>
            <p class="muted small">Lynch's bands: <1 poor · 1–1.5 fair · 1.5–2 good · ≥2 attractive.
            Heuristics from One Up On Wall Street, not a DCF.</p>`,
    },
    'early-assignment': {
        label: 'Early Assignment',
        call: (b) => api.calcEarlyAssignment(b),
        fields: [
            { key: 'spot', label: 'Spot ($)', def: 104 },
            { key: 'strike', label: 'Strike ($)', def: 100 },
            { key: 'call_price', label: 'Call price ($)', def: 5.2 },
            { key: 'dividend', label: 'Dividend ($/sh)', def: 1.5 },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid inputs</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">Assignment risk</div>
                    <div class="value ${r.assignment_likely ? 'neg' : 'pos'}">${r.assignment_likely ? 'LIKELY TONIGHT' : 'unlikely'}</div>
                    <div class="small muted">edge ${(r.exercise_edge >= 0 ? '+' : '') + r.exercise_edge.toFixed(2)}</div></div>
                <div class="card"><div class="label">Extrinsic vs dividend</div>
                    <div class="value">$${r.extrinsic.toFixed(2)} / $${r.dividend.toFixed(2)}</div>
                    <div class="small muted">intrinsic $${r.intrinsic.toFixed(2)}</div></div>
            </div>
            <p class="muted small">Calls exercise ahead of ex-div when the dividend beats remaining
            extrinsic — short ITM calls with thin extrinsic get assigned the night before.</p>`;
        },
    },
    'event-vol': {
        label: 'Event Vol / IV Crush',
        call: (b) => api.calcEventVol(b),
        fields: [
            { key: 'total_iv_pct', label: 'Total IV (%)', def: 60 },
            { key: 'ambient_iv_pct', label: 'Ambient IV (%)', def: 30 },
            { key: 'days_to_expiry', label: 'Trading days to expiry', def: 5 },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid inputs</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">Implied event move</div>
                    <div class="value">±${r.implied_event_move_pct.toFixed(2)}%</div>
                    <div class="small muted">one-day, from the vol decomposition</div></div>
                <div class="card"><div class="label">Expected crush</div>
                    <div class="value neg">−${r.iv_crush_pct_points.toFixed(1)}pp</div>
                    <div class="small muted">to ${r.post_event_iv_pct.toFixed(0)}% post-event</div></div>
            </div>
            <p class="muted small">σ²_tot·T = σ²_amb·(T−1d) + move² — the straddle is only cheap if you
            expect a bigger move than the one the decomposition prices.</p>`;
        },
    },
    'gamma-theta': {
        label: 'Gamma-Theta Breakeven',
        call: (b) => api.calcGammaThetaBreakeven(b),
        fields: [
            { key: 'gamma', label: 'Gamma (per $)', def: 0.05 },
            { key: 'theta_daily', label: 'Theta ($/day)', def: -2.5 },
            { key: 'spot', label: 'Spot ($)', def: 200 },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid inputs</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">Breakeven daily move</div>
                    <div class="value">±$${r.breakeven_move.toFixed(2)}</div>
                    <div class="small muted">${r.breakeven_move_pct.toFixed(2)}% of spot</div></div>
                <div class="card"><div class="label">Implied breakeven vol</div>
                    <div class="value">${r.implied_breakeven_vol_pct.toFixed(1)}%</div>
                    <div class="small muted">long gamma pays above this realized vol</div></div>
            </div>
            <p class="muted small">½·Γ·ΔS² = |θ| ⇒ ΔS = √(2|θ|/Γ) — the daily move where the gamma
            scalp covers the decay.</p>`;
        },
    },
    warrant: {
        label: 'Warrant Pricer',
        call: (b) => api.calcWarrant(b),
        fields: [
            { key: 'spot', label: 'Spot ($)', def: 100 },
            { key: 'strike', label: 'Strike ($)', def: 100 },
            { key: 'time_to_expiry_years', label: 'Time to expiry (years)', def: 1 },
            { key: 'risk_free_rate', label: 'Risk-free rate (decimal)', def: 0.05 },
            { key: 'volatility', label: 'Volatility (decimal)', def: 0.2 },
            { key: 'shares_outstanding', label: 'Shares outstanding (M)', def: 100 },
            { key: 'warrants_outstanding', label: 'Warrants outstanding (M)', def: 20 },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid inputs</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">Warrant value</div>
                    <div class="value">$${r.warrant_value.toFixed(4)}</div>
                    <div class="small muted">converged in ${r.iterations} iterations</div></div>
                <div class="card"><div class="label">Plain call</div>
                    <div class="value">$${r.plain_call_value.toFixed(4)}</div>
                    <div class="small muted">dilution-free upper bound</div></div>
                <div class="card"><div class="label">Dilution discount</div>
                    <div class="value neg">−${r.dilution_discount_pct.toFixed(2)}%</div>
                    <div class="small muted">factor N/(N+M) = ${r.dilution_factor.toFixed(3)}</div></div>
            </div>
            <p class="muted small">Galai-Schneller: W = N/(N+M) · BS(S + M·W/N, K) — exercise mints new
            shares but the strike proceeds raise per-share asset value, so the warrant lands between
            the naive factor × call and the plain call.</p>`;
        },
    },
    'implied-dividend': {
        label: 'Implied Dividend',
        call: (b) => api.calcImpliedDividend(b),
        fields: [
            { key: 'spot', label: 'Spot ($)', def: 100 },
            { key: 'strike', label: 'Strike ($)', def: 100 },
            { key: 'call_price', label: 'Call price ($)', def: 4.5 },
            { key: 'put_price', label: 'Put price ($)', def: 4 },
            { key: 'time_to_expiry_years', label: 'Time to expiry (years)', def: 0.5 },
            { key: 'market_risk_free_rate', label: 'Risk-free rate (decimal)', def: 0.04 },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid inputs</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">PV of dividends</div>
                    <div class="value">$${r.pv_dividends.toFixed(3)}</div>
                    <div class="small muted">to expiry, per share</div></div>
                <div class="card"><div class="label">Implied yield</div>
                    <div class="value ${r.negative_implies_borrow_cost ? 'neg' : 'pos'}">${r.implied_annual_yield_pct.toFixed(2)}%/yr</div>
                    <div class="small ${r.negative_implies_borrow_cost ? 'neg' : 'muted'}">${r.negative_implies_borrow_cost ? 'negative — hard-to-borrow fee in parity' : 'option-market dividend forecast'}</div></div>
            </div>
            <p class="muted small">From put-call parity: PV(div) = S − (C − P) − K·e^(−rT). Often sharper
            than analyst estimates around cut/raise events.</p>`;
        },
    },
    'valuation-gauges': {
        label: 'Valuation Gauges',
        call: (b) => api.calcValuationGauges({
            total_market_cap: b.total_market_cap > 0 ? b.total_market_cap : null,
            gdp: b.gdp > 0 ? b.gdp : null,
            market_value: b.market_value > 0 ? b.market_value : null,
            replacement_cost: b.replacement_cost > 0 ? b.replacement_cost : null,
            pe_ratio: b.pe_ratio > 0 ? b.pe_ratio : null,
            treasury_yield_pct: b.pe_ratio > 0 ? b.treasury_yield_pct : null,
            cape: b.cape > 0 ? b.cape : null,
            real_yield_pct: b.cape > 0 ? b.real_yield_pct : null,
        }),
        fields: [
            { key: 'total_market_cap', label: 'Total market cap ($T, 0 = skip)', def: 55 },
            { key: 'gdp', label: 'GDP ($T)', def: 28 },
            { key: 'market_value', label: "Tobin market value ($T, 0 = skip)", def: 0 },
            { key: 'replacement_cost', label: 'Replacement cost ($T)', def: 0 },
            { key: 'pe_ratio', label: 'Market P/E (0 = skip)', def: 22 },
            { key: 'treasury_yield_pct', label: '10y treasury (%)', def: 4.3 },
            { key: 'cape', label: 'Shiller CAPE (0 = skip)', def: 32 },
            { key: 'real_yield_pct', label: 'Real 10y yield (%)', def: 1.8 },
        ],
        render: (r) => `
            <div class="cards">
                ${r.buffett ? `<div class="card"><div class="label">Buffett indicator</div>
                    <div class="value ${r.buffett.ratio_pct >= 115 ? 'neg' : 'pos'}">${r.buffett.ratio_pct.toFixed(0)}%</div>
                    <div class="small muted">${esc(r.buffett.band)}</div></div>` : ''}
                ${r.tobin ? `<div class="card"><div class="label">Tobin's Q</div>
                    <div class="value ${r.tobin.q >= 1 ? 'neg' : 'pos'}">${r.tobin.q.toFixed(2)}</div>
                    <div class="small muted">${esc(r.tobin.band)}</div></div>` : ''}
                ${r.erp ? `<div class="card"><div class="label">Equity risk premium</div>
                    <div class="value ${r.erp.favors_equities ? 'pos' : 'neg'}">${r.erp.equity_risk_premium_pct.toFixed(2)}pp</div>
                    <div class="small muted">${r.erp.earnings_yield_pct.toFixed(2)}% EY − ${r.erp.treasury_yield_pct.toFixed(2)}% 10y</div></div>` : ''}
                ${r.ecy ? `<div class="card"><div class="label">Excess CAPE yield</div>
                    <div class="value ${r.ecy.excess_cape_yield_pct >= 2 ? 'pos' : 'neg'}">${r.ecy.excess_cape_yield_pct.toFixed(2)}pp</div>
                    <div class="small muted">1/${r.ecy.cape.toFixed(0)} − ${r.ecy.real_yield_pct.toFixed(1)}% real</div></div>` : ''}
            </div>
            <p class="muted small">Whole-market valuation gauges. Bands are the commonly cited heuristics
            (Buffett 2001 Fortune thresholds, ~0.7 long-run Tobin mean, Shiller's ECY) — context, not signals.</p>`,
    },
    'taylor-rule': {
        label: 'Taylor Rule',
        call: (b) => api.calcTaylorRule(b),
        fields: [
            { key: 'neutral_real_rate', label: 'Neutral real rate r* (%)', def: 2 },
            { key: 'inflation', label: 'Inflation (%)', def: 3 },
            { key: 'inflation_target', label: 'Inflation target (%)', def: 2 },
            { key: 'output_gap', label: 'Output gap (% of potential)', def: 0.5 },
            { key: 'actual_rate', label: 'Actual policy rate (%)', def: 4.5 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Prescribed rate</div>
                    <div class="value">${r.prescribed_rate.toFixed(2)}%</div>
                    <div class="small muted">vs ${r.actual_rate.toFixed(2)}% actual</div></div>
                <div class="card"><div class="label">Stance</div>
                    <div class="value ${r.stance === 'tight' ? 'neg' : r.stance === 'loose' ? 'pos' : ''}">${esc(r.stance.toUpperCase())}</div>
                    <div class="small muted">${(r.gap >= 0 ? '+' : '') + r.gap.toFixed(2)}pp vs rule</div></div>
            </div>
            <p class="muted small">Taylor (1993): i = r* + π + ½(π − π*) + ½·gap. Policy looser than the
            rule has historically been an inflation tailwind for real assets, tighter a headwind.</p>`,
    },
    'sahm-rule': {
        label: 'Sahm Rule',
        call: (b) => api.calcSahmRule({
            monthly_unemployment: String(b.monthly_unemployment).split(/[\s,]+/).map(Number).filter(x => isFinite(x)),
        }),
        fields: [
            { key: 'monthly_unemployment', label: 'Monthly unemployment %, oldest first (≥15)', def: '3.6,3.6,3.5,3.5,3.6,3.6,3.7,3.7,3.8,3.8,3.9,4.0,4.1,4.2,4.2', text: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Sahm indicator</div>
                    <div class="value ${r.triggered ? 'neg' : 'pos'}">${r.sahm_value.toFixed(2)}pp</div>
                    <div class="small ${r.triggered ? 'neg' : 'muted'}">${r.triggered ? 'TRIGGERED — recession signal' : 'below 0.50 trigger'}</div></div>
                <div class="card"><div class="label">3-month averages</div>
                    <div class="value">${r.current_3mo_avg.toFixed(2)}%</div>
                    <div class="small muted">vs ${r.min_prior_12mo.toFixed(2)}% 12-month low</div></div>
            </div>
            <p class="muted small">Sahm (2019): a 0.50pp rise in the 3-month average unemployment rate above
            its 12-month low has flagged every US recession since 1970 with no false positives in-sample.</p>`,
    },
    'misery-index': {
        label: 'Misery Index',
        call: (b) => api.calcMiseryIndex(b),
        fields: [
            { key: 'inflation', label: 'Inflation (%)', def: 3.2 },
            { key: 'unemployment', label: 'Unemployment (%)', def: 4.1 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Misery index</div>
                    <div class="value ${r.misery_index > 10 ? 'neg' : 'pos'}">${r.misery_index.toFixed(1)}</div>
                    <div class="small muted">${r.inflation.toFixed(1)}% inflation + ${r.unemployment.toFixed(1)}% unemployment</div></div>
            </div>
            <p class="muted small">Okun's misery index — single-number macro pain gauge; readings above ~10
            historically coincide with hostile equity regimes.</p>`,
    },
    'double-barrier': {
        label: 'Target vs Stop Odds',
        call: (b) => api.calcDoubleBarrier(b),
        fields: [
            { key: 'spot', label: 'Spot ($)', def: 100 },
            { key: 'lower', label: 'Stop ($)', def: 90 },
            { key: 'upper', label: 'Target ($)', def: 110 },
            { key: 'drift', label: 'Annual drift (decimal)', def: 0 },
            { key: 'vol', label: 'Annual vol (decimal)', def: 0.2 },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">need stop < spot < target</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">Target hit first</div>
                    <div class="value pos">${(r.p_upper_first * 100).toFixed(1)}%</div></div>
                <div class="card"><div class="label">Stop hit first</div>
                    <div class="value neg">${(r.p_lower_first * 100).toFixed(1)}%</div>
                    <div class="small muted">log drift ν = ${r.log_drift.toFixed(4)}</div></div>
            </div>
            <p class="muted small">GBM exit-split — which bracket leg fires first, in closed form. A tight
            target over a wide stop posts a seductive win rate with the skew hidden in the loser.</p>`;
        },
    },
    'opex-week': {
        label: 'OpEx Week',
        call: (b) => api.opexWeek(b.symbol, b.years, b.quarterly === 1),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'SPY', text: true },
            { key: 'years', label: 'Lookback years', def: 10, int: true },
            { key: 'quarterly', label: 'Triple witching only (1 = yes)', def: 0, int: true },
        ],
        render: (r) => renderEventStudy(r, 'third-Friday expiration'),
    },
    'ex-div-study': {
        label: 'Ex-Div Behavior',
        call: (b) => api.exDivStudy(b.symbol, b.years),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'KO', text: true },
            { key: 'years', label: 'Lookback years', def: 10, int: true },
        ],
        render: (r) => renderEventStudy(r, 'ex-dividend'),
    },
    'pre-holiday': {
        label: 'Pre-Holiday Effect',
        call: (b) => api.preHoliday(b.symbol, b.years),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'SPY', text: true },
            { key: 'years', label: 'Lookback years (calendar covers 2024+)', def: 5, int: true },
        ],
        render: (r) => renderEventStudy(r, 'market-holiday'),
    },
    'event-study': {
        label: 'Event Study',
        call: (b) => api.eventStudy(b.symbol, {
            dates: String(b.dates).split(/[\s,;]+/).filter(Boolean),
            years: b.years,
            window_before: b.window_before,
            window_after: b.window_after,
        }),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'SPY', text: true },
            { key: 'dates', label: 'Event dates YYYY-MM-DD (comma-sep)', def: '2025-01-29, 2025-03-19, 2025-05-07, 2025-06-18, 2025-07-30, 2025-09-17', text: true },
            { key: 'years', label: 'Lookback years', def: 10, int: true },
            { key: 'window_before', label: 'Window before (days)', def: 3, int: true },
            { key: 'window_after', label: 'Window after (days)', def: 3, int: true },
        ],
        render: (r) => renderEventStudy(r, 'supplied event'),
    },
    'best-days': {
        label: 'Miss the Best Days',
        call: (b) => api.bestDays(b.symbol, b.years, b.n),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'SPY', text: true },
            { key: 'years', label: 'Lookback years', def: 10, int: true },
            { key: 'n', label: 'Days to exclude (N)', def: 10, int: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Buy &amp; hold</div>
                    <div class="value ${r.total_return_pct >= 0 ? 'pos' : 'neg'}">${r.total_return_pct.toFixed(1)}%</div>
                    <div class="small muted">${r.days} sessions on ${esc(r.symbol)}</div></div>
                <div class="card"><div class="label">Missing ${r.n_excluded} best</div>
                    <div class="value neg">${r.missing_best_pct.toFixed(1)}%</div></div>
                <div class="card"><div class="label">Missing ${r.n_excluded} worst</div>
                    <div class="value pos">${r.missing_worst_pct.toFixed(1)}%</div></div>
                <div class="card"><div class="label">Missing both</div>
                    <div class="value">${r.missing_both_pct.toFixed(1)}%</div></div>
            </div>
            <p class="muted small">Best days: ${r.best_days_pct.map(x => '+' + x.toFixed(1) + '%').join(', ')} ·
            Worst: ${r.worst_days_pct.map(x => x.toFixed(1) + '%').join(', ')} — they cluster together,
            which is why the timing argument cuts both ways.</p>`,
    },
    'drawdown-episodes': {
        label: 'Drawdown Episodes',
        call: (b) => api.drawdownEpisodes(b.symbol, b.years, b.n),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'SPY', text: true },
            { key: 'years', label: 'Lookback years', def: 10, int: true },
            { key: 'n', label: 'Top N episodes', def: 5, int: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Status</div>
                    <div class="value ${r.currently_underwater ? 'neg' : 'pos'}">${r.currently_underwater ? r.current_drawdown_pct.toFixed(1) + '% underwater' : 'at highs'}</div></div>
            </div>
            <table class="gs-table">
                <thead><tr><th>Peak</th><th>Trough</th><th>Depth</th><th>Decline</th><th>Recovery</th></tr></thead>
                <tbody>${r.rows.map(e => `
                    <tr>
                        <td>${esc(e.peak_date)}</td>
                        <td>${esc(e.trough_date)}</td>
                        <td class="neg">${e.depth_pct.toFixed(1)}%</td>
                        <td>${e.decline_bars} bars</td>
                        <td>${e.recovery_bars != null ? e.recovery_bars + ' bars' : 'still underwater'}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Max-DD hides the frequency and the time underwater — the episode table
            shows how often it hurts and how long the holes last.</p>`,
    },
    'overnight-split': {
        label: 'Overnight vs Intraday',
        call: (b) => api.overnightSplit(b.symbol, b.years),
        fields: [
            { key: 'symbol', label: 'Symbol', def: 'SPY', text: true },
            { key: 'years', label: 'Lookback years', def: 10, int: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Overnight (close→open)</div>
                    <div class="value ${r.overnight_total_pct >= 0 ? 'pos' : 'neg'}">${r.overnight_total_pct.toFixed(1)}%</div>
                    <div class="small muted">avg ${r.overnight_avg_pct.toFixed(3)}%/night · hit ${r.overnight_hit_rate_pct.toFixed(0)}%</div></div>
                <div class="card"><div class="label">Intraday (open→close)</div>
                    <div class="value ${r.intraday_total_pct >= 0 ? 'pos' : 'neg'}">${r.intraday_total_pct.toFixed(1)}%</div>
                    <div class="small muted">avg ${r.intraday_avg_pct.toFixed(3)}%/day · hit ${r.intraday_hit_rate_pct.toFixed(0)}%</div></div>
                <div class="card"><div class="label">Close-to-close</div>
                    <div class="value">${r.close_to_close_total_pct.toFixed(1)}%</div>
                    <div class="small muted">${r.sessions} sessions on ${esc(r.symbol)}</div></div>
            </div>
            <p class="muted small">The two legs compound back to close-to-close exactly. Equity-index
            returns famously concentrate overnight — decisive for holding day-trades past the bell.</p>`,
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
    'pair-sheet': {
        label: 'Pair Sheet',
        call: (b) => api.pairSheet(b.a, b.b, b.years),
        fields: [
            { key: 'a', label: 'Symbol A (long leg)', def: 'XLE', text: true },
            { key: 'b', label: 'Symbol B (short leg)', def: 'XOP', text: true },
            { key: 'years', label: 'Lookback years', def: 5, int: true },
        ],
        render: (r) => {
            const p = r.pair;
            const c = r.correlation;
            const sig = p ? String(p.signal).replace(/_/g, ' ') : '';
            return `
            <div class="cards">
                ${p ? `<div class="card"><div class="label">Spread z-score</div>
                    <div class="value ${Math.abs(p.current_z) >= 2 ? 'neg' : ''}">${p.current_z.toFixed(2)}</div>
                    <div class="small muted">β ${p.hedge_ratio.toFixed(3)} · signal: ${esc(sig)}</div></div>` : ''}
                ${c ? `<div class="card"><div class="label">Correlation (63d)</div>
                    <div class="value">${c.current.toFixed(2)}</div>
                    <div class="small muted">${esc(c.current_regime)} · ${c.breaks.length} regime breaks</div></div>` : ''}
                <div class="card"><div class="label">Relative performance</div>
                    <div class="value ${r.relative_pct >= 0 ? 'pos' : 'neg'}">${(r.relative_pct >= 0 ? '+' : '') + r.relative_pct.toFixed(1)}pp</div>
                    <div class="small muted">${esc(r.symbol_a)} ${r.return_a_pct.toFixed(1)}% vs ${esc(r.symbol_b)} ${r.return_b_pct.toFixed(1)}%</div></div>
            </div>
            <p class="muted small">One aligned fetch over ${r.sessions} joint sessions: OLS hedge + spread
            z + signal, the rolling-correlation regime, and who's been winning. A stretched z is only a
            trade while the correlation regime that created it still holds.</p>`;
        },
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
    'heston-calibrate': {
        label: 'Heston Calibrate',
        call: (b) => api.calcHestonCalibrate({
            spot: b.spot,
            risk_free_rate: b.risk_free_rate,
            dividend_yield: 0,
            // "strike,years,C|P,price; ..." → Quote objects.
            quotes: String(b.quotes).split(';').map(s => s.trim()).filter(Boolean).map(s => {
                const [k, t, cp, px] = s.split(',').map(x => x.trim());
                return {
                    strike: Number(k),
                    time_to_expiry_years: Number(t),
                    is_call: String(cp).toUpperCase().startsWith('C'),
                    mid_price: Number(px),
                };
            }),
        }),
        fields: [
            { key: 'spot', label: 'Spot ($)', def: 100 },
            { key: 'risk_free_rate', label: 'Risk-free rate (decimal)', def: 0.03 },
            { key: 'quotes', label: 'Quotes: strike,years,C|P,price; …', def: '90,0.25,P,2.1; 100,0.25,C,4.8; 110,0.25,C,1.2; 100,0.75,C,8.4', text: true },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid quotes</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">v₀ / θ</div>
                    <div class="value">${r.v0.toFixed(4)} / ${r.theta.toFixed(4)}</div>
                    <div class="small muted">spot vol ${(Math.sqrt(r.v0) * 100).toFixed(1)}% · long-run ${(Math.sqrt(r.theta) * 100).toFixed(1)}%</div></div>
                <div class="card"><div class="label">κ / σ / ρ</div>
                    <div class="value">${r.kappa.toFixed(2)} / ${r.vol_of_vol.toFixed(2)} / ${r.rho.toFixed(2)}</div>
                    <div class="small ${r.feller_satisfied ? 'muted' : 'neg'}">${r.feller_satisfied ? 'Feller satisfied' : 'Feller VIOLATED'}</div></div>
                <div class="card"><div class="label">RMSE</div>
                    <div class="value ${r.rmse < 0.1 ? 'pos' : 'neg'}">$${r.rmse.toFixed(4)}</div></div>
            </div>
            <table class="gs-table">
                <thead><tr><th>Strike</th><th>T</th><th>Market</th><th>Model</th><th>Error</th></tr></thead>
                <tbody>${r.fits.map(f => `
                    <tr>
                        <td>$${f.strike.toFixed(0)}</td>
                        <td>${f.time_to_expiry_years.toFixed(2)}y</td>
                        <td>$${f.market.toFixed(2)}</td>
                        <td>$${f.model.toFixed(2)}</td>
                        <td class="${Math.abs(f.error) < 0.1 ? 'pos' : 'neg'}">${f.error >= 0 ? '+' : ''}${f.error.toFixed(3)}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Nelder-Mead least-squares over the quote prices in an unconstrained
            parameter transform; fits re-priced on the full integration grid.</p>`;
        },
    },
    'probability-of-profit': {
        label: 'POP Calculator',
        call: (b) => api.calcProbabilityOfProfit({
            spot: b.spot,
            iv: b.iv,
            time_to_expiry_years: b.time_to_expiry_years,
            risk_free_rate: b.risk_free_rate,
            zero_drift: b.zero_drift === 1,
            lower_breakeven: b.lower_breakeven > 0 ? b.lower_breakeven : null,
            upper_breakeven: b.upper_breakeven > 0 ? b.upper_breakeven : null,
            profit_between: b.profit_between === 1,
        }),
        fields: [
            { key: 'spot', label: 'Spot ($)', def: 100 },
            { key: 'iv', label: 'Implied vol (decimal)', def: 0.25 },
            { key: 'time_to_expiry_years', label: 'Time to expiry (years)', def: 0.25 },
            { key: 'risk_free_rate', label: 'Risk-free rate (decimal)', def: 0.05 },
            { key: 'zero_drift', label: 'Zero drift (1 = retail convention)', def: 1, int: true },
            { key: 'lower_breakeven', label: 'Lower breakeven ($, 0 = none)', def: 90 },
            { key: 'upper_breakeven', label: 'Upper breakeven ($, 0 = none)', def: 110 },
            { key: 'profit_between', label: 'Profit between (1) / outside (0)', def: 1, int: true },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid inputs (need at least one breakeven)</span>';
            return `
            <div class="cards">
                <div class="card"><div class="label">Probability of profit</div>
                    <div class="value ${r.probability_of_profit >= 0.5 ? 'pos' : 'neg'}">${(r.probability_of_profit * 100).toFixed(1)}%</div></div>
                <div class="card"><div class="label">Tail probabilities</div>
                    <div class="value">${r.prob_below_lower != null ? (r.prob_below_lower * 100).toFixed(1) + '%' : '—'} / ${r.prob_above_upper != null ? (r.prob_above_upper * 100).toFixed(1) + '%' : '—'}</div>
                    <div class="small muted">below lower / above upper</div></div>
                <div class="card"><div class="label">Expected spot</div>
                    <div class="value">$${r.expected_spot.toFixed(2)}</div></div>
            </div>
            <p class="muted small">Lognormal terminal-price probabilities. Credit spreads/condors profit
            BETWEEN breakevens; long straddles/strangles OUTSIDE.</p>`;
        },
    },
    'zero-curve': {
        label: 'Zero Curve Bootstrap',
        call: (b) => api.calcBootstrapZeroCurve({
            par_rates: String(b.par_rates).split(/[\s,]+/).map(Number).filter(x => isFinite(x)).map(x => x / 100),
        }),
        fields: [
            { key: 'par_rates', label: 'Par rates %, year 1..N (comma-sep)', def: '4.5, 4.3, 4.2, 4.2, 4.3, 4.4, 4.5', text: true },
        ],
        render: (r) => {
            if (!r) return '<span class="neg">invalid par curve</span>';
            return `
            <table class="gs-table">
                <thead><tr><th>Tenor</th><th>Par</th><th>Zero</th><th>1y Fwd</th><th>DF</th></tr></thead>
                <tbody>${r.map(p => `
                    <tr>
                        <td>${p.tenor_years}y</td>
                        <td>${(p.par_rate * 100).toFixed(2)}%</td>
                        <td>${(p.zero_rate * 100).toFixed(3)}%</td>
                        <td>${(p.forward_rate * 100).toFixed(3)}%</td>
                        <td>${p.discount_factor.toFixed(5)}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Annual-coupon par bonds bootstrap one discount factor per tenor; zero
            rates are annually compounded, forwards are the implied 1-year rates between tenors.</p>`;
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

function renderEventStudy(r, anchorLabel) {
    return `
        <div class="cards">
            <div class="card"><div class="label">Event-day return</div>
                <div class="value ${r.event_day_avg_pct >= 0 ? 'pos' : 'neg'}">${r.event_day_avg_pct.toFixed(3)}%</div>
                <div class="small muted">hit ${r.event_day_hit_rate_pct.toFixed(0)}% · ${r.events_used}/${r.events_supplied} events in sample</div></div>
        </div>
        <table class="gs-table">
            <thead><tr><th>Offset</th><th>Mean (log)</th><th>Std</th><th>Hit</th><th>N</th></tr></thead>
            <tbody>${r.offsets.by_offset.map(o => `
                <tr>
                    <td>${o.offset > 0 ? '+' + o.offset : o.offset}</td>
                    <td class="${o.mean_return >= 0 ? 'pos' : 'neg'}">${(o.mean_return * 100).toFixed(3)}%</td>
                    <td>${(o.std_return * 100).toFixed(2)}%</td>
                    <td>${(o.hit_rate * 100).toFixed(0)}%</td>
                    <td>${o.sample_count}</td>
                </tr>`).join('')}
            </tbody>
        </table>
        <p class="muted small">Per-offset stats around each ${anchorLabel} day (offset 0 = the day) —
        same engine as the Santa rally study.</p>`;
}

export async function renderStrategyTools(mount) {
    renderToolTabs(mount, {
        titleKey: 'view.strategy_tools.h1',
        title: '// STRATEGY TOOLS',
        hintKey: 'view.strategy_tools.hint',
        tools: TOOLS,
        defaultKey: 'grid-trading',
    });
}
