// Covered Call — own 100 shares of a stock + sell 1 OTM call against
// it. Collect premium (income); cap upside at strike + premium.
// Best in flat / mildly bullish markets. Risk: stock surges past
// strike → forced sale at strike (still profitable, just opportunity-
// cost of further upside). Stock crash: premium provides small cushion
// but you still lose on the underlying.

import { esc } from '../util.js';

export async function renderCoveredCall(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.covered_call.title">// COVERED CALL · INCOME STRATEGY</span></h1>
        <p class="muted small" data-i18n-html="view.covered_call.intro">
            Own 100 shares + sell 1 OTM call against each block. Collect
            premium; cap upside at strike + premium. Best in flat / mildly
            bullish markets. Risk: stock surges past strike → assignment at
            strike (still profitable, just opportunity-cost on the rip).
            Premium provides small cushion if stock falls but you still
            own the underlying.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Shares owned</span>
                    <input type="number" id="cc-shares" step="100" min="100" value="100" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Cost basis $/sh</span>
                    <input type="number" id="cc-basis" step="0.5" min="0" value="95" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Current price $</span>
                    <input type="number" id="cc-spot" step="0.5" min="0" value="100" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Strike $</span>
                    <input type="number" id="cc-strike" step="0.5" min="0" value="105" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Call premium $/sh</span>
                    <input type="number" id="cc-prem" step="0.05" min="0" value="1.50" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Days to expiration</span>
                    <input type="number" id="cc-dte" step="1" min="1" max="365" value="30" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="cc-run">⚡ Compute</button>
            <div id="cc-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#cc-shares, #cc-basis, #cc-spot, #cc-strike, #cc-prem, #cc-dte').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#cc-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const shares = parseInt(mount.querySelector('#cc-shares').value, 10) || 0;
    const basis = parseFloat(mount.querySelector('#cc-basis').value) || 0;
    const spot = parseFloat(mount.querySelector('#cc-spot').value) || 0;
    const strike = parseFloat(mount.querySelector('#cc-strike').value) || 0;
    const prem = parseFloat(mount.querySelector('#cc-prem').value) || 0;
    const dte = parseInt(mount.querySelector('#cc-dte').value, 10) || 30;
    const result = mount.querySelector('#cc-result');

    const contracts = Math.floor(shares / 100);
    if (contracts < 1) {
        result.innerHTML = `<p class="muted">Need at least 100 shares for 1 covered call.</p>`;
        return;
    }
    const premiumIncome = prem * 100 * contracts;
    const premYieldVsSpot = (prem / spot) * 100;
    const annualizedYield = premYieldVsSpot * (365 / dte);

    // Scenarios at expiration.
    const scenarios = [
        { label: 'Stock drops -20%', spotEnd: spot * 0.80 },
        { label: 'Stock drops -10%', spotEnd: spot * 0.90 },
        { label: 'Stock flat',        spotEnd: spot },
        { label: 'Stock to strike',   spotEnd: strike },
        { label: 'Stock +10% (called)', spotEnd: spot * 1.10 },
        { label: 'Stock +20% (called)', spotEnd: spot * 1.20 },
    ];

    const positionValueNow = spot * shares;
    const rows = scenarios.map(s => {
        const called = s.spotEnd > strike;
        const stockEnd = called ? strike * shares : s.spotEnd * shares;
        const totalEnd = stockEnd + premiumIncome;
        const initialCost = basis * shares;
        const pl = totalEnd - initialCost;
        const plPct = (pl / initialCost) * 100;
        const oppCost = called ? (s.spotEnd - strike) * shares : 0;
        return { ...s, stockEnd, totalEnd, pl, plPct, oppCost, called };
    });

    const maxProfit = (strike - basis) * shares + premiumIncome;
    const breakeven = basis - prem;
    const downsidePct = (basis - breakeven) / basis * 100;

    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Contracts written</div><div class="value">${contracts}</div><div class="muted small">${shares} shares</div></div>
            <div class="card"><div class="label">Premium collected</div><div class="value pos">$${fmt(premiumIncome, 0)}</div><div class="muted small">$${fmt(prem, 2)}/sh × 100 × ${contracts}</div></div>
            <div class="card"><div class="label">Premium yield (period)</div><div class="value">${fmt(premYieldVsSpot, 2)}%</div><div class="muted small">$${fmt(prem, 2)} / $${fmt(spot, 2)} spot</div></div>
            <div class="card"><div class="label">Annualized yield</div><div class="value pos">${fmt(annualizedYield, 2)}%</div><div class="muted small">${dte} DTE → 365/${dte}</div></div>
            <div class="card"><div class="label">Max profit (called)</div><div class="value pos">$${fmt(maxProfit, 0)}</div><div class="muted small">Strike $${fmt(strike, 2)} − basis $${fmt(basis, 2)} + premium</div></div>
            <div class="card"><div class="label">Breakeven $/sh</div><div class="value">$${fmt(breakeven, 2)}</div><div class="muted small">${fmt(downsidePct, 2)}% cushion before red</div></div>
        </div>
        <h3 class="section-title">Scenario P&L</h3>
        <table class="trades" data-table-key="cc-scenarios">
            <thead><tr>
                <th>Scenario</th>
                <th>Stock end</th>
                <th>Called?</th>
                <th>End value</th>
                <th>P&L vs basis</th>
                <th>Opportunity cost</th>
            </tr></thead>
            <tbody>${rows.map(r => `<tr>
                <td>${esc(r.label)}</td>
                <td>$${fmt(r.spotEnd, 2)}</td>
                <td class="${r.called ? 'neg' : ''}">${r.called ? 'YES' : 'no'}</td>
                <td>$${fmt(r.totalEnd, 0)}</td>
                <td class="${r.pl > 0 ? 'pos' : 'neg'}"><strong>${r.pl >= 0 ? '+' : ''}$${fmt(r.pl, 0)}</strong> (${r.plPct >= 0 ? '+' : ''}${fmt(r.plPct, 1)}%)</td>
                <td class="muted">${r.oppCost > 0 ? '-$' + fmt(r.oppCost, 0) : '—'}</td>
            </tr>`).join('')}</tbody>
        </table>
        <p class="muted small" style="margin-top:8px">
            <strong>Roll strategy:</strong> when ITM and approaching expiration, you
            can "roll up and out" — buy back the current call, sell a higher-strike
            longer-dated call. Captures more upside at cost of additional risk.
        </p>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
