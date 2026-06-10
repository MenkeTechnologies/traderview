// Bond Ladder Builder — staggered-maturity fixed-income ladder.
// Splits a starting principal across N rungs, each with a different
// maturity, so a rung matures every period and rolls into the longest
// rung at the back of the ladder. Smooths reinvestment risk (rolling
// the whole portfolio at the same time = locking in whatever rate is
// available that day) and provides predictable cash flow.
// Supports Treasuries (semi-annual coupons), CDs (compound interest),
// and "agency / muni-equivalent" generic mode.

import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderBondLadder(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.bond_ladder.title">// BOND LADDER BUILDER</span></h1>
        <p class="muted small" data-i18n-html="view.bond_ladder.intro">
            Stagger maturity dates so one rung matures every period. Each
            maturing rung rolls into a fresh long-dated rung at the back of
            the ladder. Smooths reinvestment risk vs putting everything in a
            single maturity. Default 5-year ladder uses current Treasury
            curve approximations — replace with the actual quote when laddering.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Total to ladder $</span>
                    <input type="number" id="bl-total" step="1000" min="0" value="100000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Number of rungs</span>
                    <input type="number" id="bl-rungs" step="1" min="2" max="20" value="5" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Months between rungs</span>
                    <input type="number" id="bl-gap" step="1" min="3" max="60" value="12" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Instrument</span>
                    <select id="bl-kind" style="width:100%">
                        <option value="treasury" selected>Treasury (semi-annual coupon)</option>
                        <option value="cd">CD (compound, no coupons)</option>
                        <option value="muni">Muni / agency (semi-annual, tax-free toggle)</option>
                    </select>
                </label>
                <label>
                    <span class="muted small">Yield curve %/yr (comma-separated, ascending maturity)</span>
                    <input type="text" id="bl-curve" value="4.2,4.0,3.95,3.92,3.95" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Marginal tax bracket %</span>
                    <input type="number" id="bl-tax" step="1" min="0" max="50" value="22" style="width:100%">
                </label>
                <label>
                    <span class="muted small">State tax %</span>
                    <input type="number" id="bl-state" step="0.1" min="0" max="15" value="5" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="bl-run">⚡ Build ladder</button>
            <div id="bl-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#bl-total, #bl-rungs, #bl-gap, #bl-kind, #bl-curve, #bl-tax, #bl-state').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#bl-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const total = parseFloat(mount.querySelector('#bl-total').value) || 0;
    const rungs = Math.max(2, Math.min(20, parseInt(mount.querySelector('#bl-rungs').value, 10) || 5));
    const gap = parseInt(mount.querySelector('#bl-gap').value, 10) || 12;
    const kind = mount.querySelector('#bl-kind').value;
    const curveStr = mount.querySelector('#bl-curve').value;
    const taxFed = parseFloat(mount.querySelector('#bl-tax').value) / 100;
    const taxSt = parseFloat(mount.querySelector('#bl-state').value) / 100;
    const result = mount.querySelector('#bl-result');

    const yields = curveStr.split(',').map(s => parseFloat(s.trim()) / 100).filter(n => Number.isFinite(n));
    if (yields.length < rungs) {
        result.innerHTML = `<p class="muted">Provide at least ${rungs} yields in the curve (got ${yields.length}).</p>`;
        return;
    }
    const perRung = total / rungs;
    const now = new Date();

    const ladder = [];
    let totalAnnualIncome = 0;
    for (let i = 0; i < rungs; i++) {
        const months = (i + 1) * gap;
        const yrs = months / 12;
        const y = yields[i];
        const maturity = new Date(now.getFullYear(), now.getMonth() + months, 1);
        let annualIncome = 0;
        let endValue = 0;
        if (kind === 'cd') {
            annualIncome = 0;
            endValue = perRung * Math.pow(1 + y, yrs);
        } else {
            annualIncome = perRung * y;
            endValue = perRung;
        }
        totalAnnualIncome += annualIncome;
        // Tax treatment.
        let afterTaxAnnual;
        if (kind === 'treasury') {
            // Treasury: federal taxable, state exempt.
            afterTaxAnnual = annualIncome * (1 - taxFed);
        } else if (kind === 'muni') {
            // Muni: federal + state exempt (in-state).
            afterTaxAnnual = annualIncome;
        } else {
            // CD: fully taxable, no coupon (recognized at maturity).
            afterTaxAnnual = 0;
        }
        ladder.push({
            rung: i + 1,
            months,
            maturity,
            principal: perRung,
            yld: y,
            annualIncome,
            afterTaxAnnual,
            endValue,
        });
    }

    const totalAfterTax = ladder.reduce((s, r) => s + r.afterTaxAnnual, 0);
    const avgYield = yields.slice(0, rungs).reduce((s, y) => s + y, 0) / rungs;
    const tey = kind === 'muni' ? avgYield / (1 - taxFed - taxSt) : avgYield;     // tax-equivalent for muni

    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Principal per rung</div><div class="value">$${fmt(perRung, 0)}</div></div>
            <div class="card"><div class="label">Avg yield (ladder)</div><div class="value pos">${fmt(avgYield * 100, 2)}%</div></div>
            ${kind === 'muni' ? `<div class="card"><div class="label">Tax-equivalent yield</div><div class="value pos">${fmt(tey * 100, 2)}%</div><div class="muted small">@ ${fmt((taxFed + taxSt) * 100, 1)}% combined bracket</div></div>` : ''}
            <div class="card"><div class="label">Annual income (pre-tax)</div><div class="value pos">$${fmt(totalAnnualIncome, 0)}</div></div>
            <div class="card"><div class="label">Annual income (after-tax)</div><div class="value pos">$${fmt(totalAfterTax, 0)}</div></div>
        </div>
        <table class="trades" data-table-key="bl-rungs">
            <thead><tr>
                <th>Rung</th>
                <th>Months</th>
                <th>Maturity</th>
                <th>Principal $</th>
                <th>Yield</th>
                <th>${kind === 'cd' ? 'End value' : 'Annual income'}</th>
                <th>After-tax</th>
            </tr></thead>
            <tbody>${ladder.map(r => `<tr>
                <td><strong>${r.rung}</strong></td>
                <td>${r.months}</td>
                <td class="muted">${r.maturity.toLocaleString(undefined, { month:'short', year:'numeric' })}</td>
                <td>$${fmt(r.principal, 0)}</td>
                <td class="pos">${fmt(r.yld * 100, 2)}%</td>
                <td>${kind === 'cd' ? '$' + fmt(r.endValue, 0) : '$' + fmt(r.annualIncome, 0) + '/yr'}</td>
                <td class="muted">$${fmt(r.afterTaxAnnual, 0)}/yr</td>
            </tr>`).join('')}</tbody>
        </table>
        <p class="muted small" style="margin-top:8px">
            Rolling rule: when rung 1 matures, reinvest the principal at the longest tenor
            (${rungs * gap} months out) at the rate then prevailing. Repeat each period.
        </p>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
