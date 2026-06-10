// Iron Condor — sell OTM call spread + sell OTM put spread, same exp.
// Four legs, two credit spreads. Profit if underlying stays between
// the inner strikes through expiration. Max loss capped at the wider
// of the two spread widths minus net credit collected. Classic
// theta-positive strategy for range-bound expectations / low IV
// realized vs IV implied.

import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderIronCondor(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.iron_condor.title">// IRON CONDOR</span></h1>
        <p class="muted small" data-i18n-html="view.iron_condor.intro">
            Sell OTM put spread + sell OTM call spread, same expiration.
            Theta-positive, vega-negative. Max profit if price stays
            between the two short strikes through expiration. Max loss
            capped at the wider spread width minus net credit. Use when
            you expect <strong>realized vol &lt; implied vol</strong>
            (range-bound expectation, post-earnings IV crush, etc.).
        </p>
        <div class="chart-panel">
            <h3 class="section-title">Short put spread (lower side)</h3>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Long put strike $</span>
                    <input type="number" id="ic-lp-k" step="1" min="0" value="90" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Long put premium $</span>
                    <input type="number" id="ic-lp-prem" step="0.05" min="0" value="0.85" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Short put strike $</span>
                    <input type="number" id="ic-sp-k" step="1" min="0" value="95" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Short put premium $</span>
                    <input type="number" id="ic-sp-prem" step="0.05" min="0" value="1.85" style="width:100%">
                </label>
            </div>
            <h3 class="section-title">Short call spread (upper side)</h3>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Short call strike $</span>
                    <input type="number" id="ic-sc-k" step="1" min="0" value="105" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Short call premium $</span>
                    <input type="number" id="ic-sc-prem" step="0.05" min="0" value="1.95" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Long call strike $</span>
                    <input type="number" id="ic-lc-k" step="1" min="0" value="110" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Long call premium $</span>
                    <input type="number" id="ic-lc-prem" step="0.05" min="0" value="0.90" style="width:100%">
                </label>
            </div>
            <h3 class="section-title">Position</h3>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Current price $</span>
                    <input type="number" id="ic-spot" step="0.5" min="0" value="100" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Contracts (per spread)</span>
                    <input type="number" id="ic-qty" step="1" min="1" value="5" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="ic-run">⚡ Compute</button>
            <div id="ic-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#ic-lp-k, #ic-lp-prem, #ic-sp-k, #ic-sp-prem, #ic-sc-k, #ic-sc-prem, #ic-lc-k, #ic-lc-prem, #ic-spot, #ic-qty').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#ic-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const Lp_k = parseFloat(mount.querySelector('#ic-lp-k').value) || 0;
    const Lp = parseFloat(mount.querySelector('#ic-lp-prem').value) || 0;
    const Sp_k = parseFloat(mount.querySelector('#ic-sp-k').value) || 0;
    const Sp = parseFloat(mount.querySelector('#ic-sp-prem').value) || 0;
    const Sc_k = parseFloat(mount.querySelector('#ic-sc-k').value) || 0;
    const Sc = parseFloat(mount.querySelector('#ic-sc-prem').value) || 0;
    const Lc_k = parseFloat(mount.querySelector('#ic-lc-k').value) || 0;
    const Lc = parseFloat(mount.querySelector('#ic-lc-prem').value) || 0;
    const spot = parseFloat(mount.querySelector('#ic-spot').value) || 0;
    const qty = parseInt(mount.querySelector('#ic-qty').value, 10) || 1;
    const result = mount.querySelector('#ic-result');

    const putWidth = Sp_k - Lp_k;
    const callWidth = Lc_k - Sc_k;
    const netCredit = (Sp + Sc) - (Lp + Lc);    // per share
    const maxProfit = netCredit * 100 * qty;
    const maxLossPerShare = Math.max(putWidth, callWidth) - netCredit;
    const maxLoss = -maxLossPerShare * 100 * qty;
    const beLower = Sp_k - netCredit;
    const beUpper = Sc_k + netCredit;
    const profitZoneWidth = beUpper - beLower;
    const collateral = Math.max(putWidth, callWidth) * 100 * qty - maxProfit;
    const returnOnCollat = collateral > 0 ? (maxProfit / collateral) * 100 : 0;

    // Payoff curve at expiration.
    const lo = Lp_k - Math.max(putWidth, callWidth);
    const hi = Lc_k + Math.max(putWidth, callWidth);
    const steps = 18;
    const curve = [];
    for (let i = 0; i <= steps; i++) {
        const px = lo + (hi - lo) * (i / steps);
        const Lp_intrinsic = Math.max(0, Lp_k - px);
        const Sp_intrinsic = Math.max(0, Sp_k - px);
        const Sc_intrinsic = Math.max(0, px - Sc_k);
        const Lc_intrinsic = Math.max(0, px - Lc_k);
        const payoff = (Lp_intrinsic - Sp_intrinsic - Sc_intrinsic + Lc_intrinsic + netCredit) * 100 * qty;
        curve.push({ px, payoff });
    }

    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Net credit collected</div><div class="value pos">$${fmt(maxProfit, 0)}</div><div class="muted small">$${fmt(netCredit, 2)}/share × ${qty * 100}</div></div>
            <div class="card"><div class="label">Max profit</div><div class="value pos">$${fmt(maxProfit, 0)}</div><div class="muted small">(if expires between $${fmt(Sp_k, 0)} and $${fmt(Sc_k, 0)})</div></div>
            <div class="card"><div class="label">Max loss</div><div class="value neg">$${fmt(maxLoss, 0)}</div><div class="muted small">(width − credit) × ${qty * 100}</div></div>
            <div class="card"><div class="label">Profit zone</div><div class="value">$${fmt(beLower, 2)} → $${fmt(beUpper, 2)}</div><div class="muted small">$${fmt(profitZoneWidth, 2)} wide (${fmt(profitZoneWidth / spot * 100, 1)}% of spot)</div></div>
            <div class="card"><div class="label">Collateral required</div><div class="value">$${fmt(collateral, 0)}</div></div>
            <div class="card"><div class="label">Return on collateral</div><div class="value pos">${fmt(returnOnCollat, 1)}%</div><div class="muted small">If maxprofit hit</div></div>
        </div>
        <h3 class="section-title">P&L at expiration</h3>
        <table class="trades" data-table-key="ic-curve">
            <thead><tr><th>Underlying $</th><th>P&L</th><th>Zone</th></tr></thead>
            <tbody>${curve.map(c => {
                const zone = c.px < Lp_k ? 'Max loss (below long put)'
                           : c.px < Sp_k ? 'Put spread loss zone'
                           : c.px <= Sc_k ? 'Max profit zone'
                           : c.px <= Lc_k ? 'Call spread loss zone'
                           : 'Max loss (above long call)';
                return `<tr>
                    <td>${esc(Math.abs(c.px - spot) < 0.01 ? '★ ' : '')}$${fmt(c.px, 2)}${esc(Math.abs(c.px - spot) < 0.01 ? ' (spot)' : '')}</td>
                    <td class="${c.payoff > 0 ? 'pos' : c.payoff < 0 ? 'neg' : 'muted'}"><strong>${c.payoff >= 0 ? '+' : ''}$${fmt(c.payoff, 0)}</strong></td>
                    <td class="muted small">${esc(zone)}</td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
