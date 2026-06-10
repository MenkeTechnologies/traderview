// Vertical Spread — same expiration, different strikes, opposite legs.
// Four flavors:
//   Bull Call Spread (debit, bullish): buy low strike call, sell high strike call
//   Bear Put Spread (debit, bearish):  buy high strike put,  sell low strike put
//   Bull Put Spread (credit, bullish): sell high strike put, buy low strike put
//   Bear Call Spread (credit, bearish): sell low strike call, buy high strike call
// All cap risk + reward at the spread width. Computes max profit, max loss,
// breakeven, and per-strike P&L curve.

import { esc } from '../util.js';

const STRATEGIES = {
    bull_call:  { dir: 'bullish', kind: 'debit',  desc: 'Buy lower-strike call, sell higher-strike call' },
    bear_put:   { dir: 'bearish', kind: 'debit',  desc: 'Buy higher-strike put, sell lower-strike put' },
    bull_put:   { dir: 'bullish', kind: 'credit', desc: 'Sell higher-strike put, buy lower-strike put' },
    bear_call:  { dir: 'bearish', kind: 'credit', desc: 'Sell lower-strike call, buy higher-strike call' },
};

export async function renderVerticalSpread(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.vertical_spread.title">// VERTICAL SPREAD</span></h1>
        <p class="muted small" data-i18n-html="view.vertical_spread.intro">
            Same expiration, different strikes, opposite legs. Caps risk +
            reward at the spread width. Use a <strong>debit spread</strong>
            (bull call / bear put) when you have directional conviction;
            <strong>credit spread</strong> (bull put / bear call) when you
            want to collect time decay on a non-event in your direction.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Strategy</span>
                    <select id="vs-strat" style="width:100%">
                        <option value="bull_call" selected>Bull Call (debit, ↑)</option>
                        <option value="bear_put">Bear Put (debit, ↓)</option>
                        <option value="bull_put">Bull Put (credit, ↑)</option>
                        <option value="bear_call">Bear Call (credit, ↓)</option>
                    </select>
                </label>
                <label>
                    <span class="muted small">Current price $</span>
                    <input type="number" id="vs-spot" step="0.5" min="0" value="100" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Long strike $</span>
                    <input type="number" id="vs-long-k" step="1" min="0" value="100" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Short strike $</span>
                    <input type="number" id="vs-short-k" step="1" min="0" value="110" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Long premium $</span>
                    <input type="number" id="vs-long-prem" step="0.05" min="0" value="3.50" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Short premium $</span>
                    <input type="number" id="vs-short-prem" step="0.05" min="0" value="1.20" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Contracts</span>
                    <input type="number" id="vs-qty" step="1" min="1" value="10" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="vs-run">⚡ Compute</button>
            <div id="vs-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#vs-strat, #vs-spot, #vs-long-k, #vs-short-k, #vs-long-prem, #vs-short-prem, #vs-qty').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#vs-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const strat = mount.querySelector('#vs-strat').value;
    const spot = parseFloat(mount.querySelector('#vs-spot').value) || 0;
    const longK = parseFloat(mount.querySelector('#vs-long-k').value) || 0;
    const shortK = parseFloat(mount.querySelector('#vs-short-k').value) || 0;
    const longPrem = parseFloat(mount.querySelector('#vs-long-prem').value) || 0;
    const shortPrem = parseFloat(mount.querySelector('#vs-short-prem').value) || 0;
    const qty = parseInt(mount.querySelector('#vs-qty').value, 10) || 1;
    const result = mount.querySelector('#vs-result');

    const meta = STRATEGIES[strat];
    const width = Math.abs(shortK - longK);
    const isCall = strat.endsWith('call');
    const isCredit = meta.kind === 'credit';

    // Per-contract P&L = 100 × (intrinsic_long − intrinsic_short ± net premium).
    // Sign convention: positive net = long pays you; negative = you pay.
    const netCredit = shortPrem - longPrem;     // positive when credit, negative when debit
    const netCost = -netCredit * 100 * qty;     // debit cost (positive) or credit received (negative)

    // Max profit / loss for each strategy:
    let maxProfit, maxLoss, breakeven;
    if (strat === 'bull_call') {
        maxProfit = (width - (longPrem - shortPrem)) * 100 * qty;
        maxLoss   = -(longPrem - shortPrem) * 100 * qty;
        breakeven = longK + (longPrem - shortPrem);
    } else if (strat === 'bear_put') {
        maxProfit = (width - (longPrem - shortPrem)) * 100 * qty;
        maxLoss   = -(longPrem - shortPrem) * 100 * qty;
        breakeven = longK - (longPrem - shortPrem);
    } else if (strat === 'bull_put') {
        // Sell higher-strike put (shortK > longK)
        maxProfit = (shortPrem - longPrem) * 100 * qty;
        maxLoss   = -(width - (shortPrem - longPrem)) * 100 * qty;
        breakeven = shortK - (shortPrem - longPrem);
    } else {     // bear_call
        // Sell lower-strike call (shortK < longK)
        maxProfit = (shortPrem - longPrem) * 100 * qty;
        maxLoss   = -(width - (shortPrem - longPrem)) * 100 * qty;
        breakeven = shortK + (shortPrem - longPrem);
    }

    // P&L curve at expiration across strikes.
    const lo = Math.min(longK, shortK) - width * 1.5;
    const hi = Math.max(longK, shortK) + width * 1.5;
    const steps = 15;
    const curve = [];
    for (let i = 0; i <= steps; i++) {
        const px = lo + (hi - lo) * (i / steps);
        const longIntrinsic = isCall ? Math.max(0, px - longK) : Math.max(0, longK - px);
        const shortIntrinsic = isCall ? Math.max(0, px - shortK) : Math.max(0, shortK - px);
        const payoff = (longIntrinsic - shortIntrinsic + netCredit) * 100 * qty;
        curve.push({ px, payoff });
    }

    const reward_risk = Math.abs(maxLoss) > 0 ? Math.abs(maxProfit / maxLoss) : 0;
    const collateral = isCredit ? Math.abs(maxLoss) : Math.abs(netCost);
    const yieldPct = collateral > 0 ? (maxProfit / collateral) * 100 : 0;

    result.innerHTML = `
        <p class="muted small"><strong>${esc(strat.toUpperCase().replace('_', ' '))}</strong> — ${esc(meta.dir)}, ${esc(meta.kind)}. ${esc(meta.desc)}.</p>
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Width × ${qty} × 100</div><div class="value">$${fmt(width * 100 * qty, 0)}</div><div class="muted small">$${fmt(width, 2)} spread width</div></div>
            <div class="card"><div class="label">Net ${isCredit ? 'credit received' : 'debit paid'}</div><div class="value ${isCredit ? 'pos' : 'neg'}">$${fmt(Math.abs(netCost), 0)}</div></div>
            <div class="card"><div class="label">Max profit</div><div class="value pos">$${fmt(maxProfit, 0)}</div></div>
            <div class="card"><div class="label">Max loss</div><div class="value neg">$${fmt(maxLoss, 0)}</div></div>
            <div class="card"><div class="label">Breakeven</div><div class="value">$${fmt(breakeven, 2)}</div></div>
            <div class="card"><div class="label">Reward:Risk</div><div class="value">${fmt(reward_risk, 2)}:1</div></div>
            <div class="card"><div class="label">Return on collateral</div><div class="value pos">${fmt(yieldPct, 1)}%</div></div>
        </div>
        <h3 class="section-title">P&L at expiration</h3>
        <table class="trades" data-table-key="vs-curve">
            <thead><tr><th>Underlying $</th><th>P&L</th></tr></thead>
            <tbody>${curve.map(c => `<tr>
                <td>${esc(c.px === spot ? '★ ' : '')}$${fmt(c.px, 2)}${esc(c.px === spot ? ' (spot)' : '')}</td>
                <td class="${c.payoff > 0 ? 'pos' : c.payoff < 0 ? 'neg' : 'muted'}"><strong>${c.payoff >= 0 ? '+' : ''}$${fmt(c.payoff, 0)}</strong></td>
            </tr>`).join('')}</tbody>
        </table>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
