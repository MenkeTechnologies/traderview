// DRIP Simulator — Dividend Reinvestment Plan compounding. Every paid
// dividend buys more shares at the then-current price. Over decades,
// reinvested dividends dwarf the original principal — for many quality
// dividend stocks, total return is ~60% dividends-reinvested and ~40%
// price appreciation. Compares DRIP vs cash dividends side by side.

import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderDripSimulator(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.drip_simulator.title">// DRIP SIMULATOR</span></h1>
        <p class="muted small" data-i18n-html="view.drip_simulator.intro">
            Reinvested dividends compound additional share growth. Compares
            <strong>DRIP</strong> (auto-reinvest) vs <strong>cash dividends</strong>
            (pocketed and unspent) on the same starting position. Default uses
            a 3% yield growing 5%/yr with 5% price appreciation — roughly
            matching a quality dividend grower like JNJ or PG.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Shares purchased</span>
                    <input type="number" id="dr-shares" step="1" min="1" value="100" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Price per share $</span>
                    <input type="number" id="dr-price" step="0.5" min="0" value="100" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Starting yield %</span>
                    <input type="number" id="dr-yield" step="0.1" min="0" max="20" value="3.0" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Dividend growth %/yr</span>
                    <input type="number" id="dr-divgrow" step="0.1" min="0" max="30" value="5.0" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Price growth %/yr</span>
                    <input type="number" id="dr-pricegrow" step="0.1" min="-10" max="30" value="5.0" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Holding period (years)</span>
                    <input type="number" id="dr-years" step="1" min="1" max="60" value="30" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Dividend tax rate % (DRIP only)</span>
                    <input type="number" id="dr-tax" step="1" min="0" max="50" value="15" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="dr-run">⚡ Simulate</button>
            <div id="dr-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#dr-shares, #dr-price, #dr-yield, #dr-divgrow, #dr-pricegrow, #dr-years, #dr-tax').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#dr-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const initShares = parseFloat(mount.querySelector('#dr-shares').value) || 0;
    const initPrice = parseFloat(mount.querySelector('#dr-price').value) || 0;
    const yield0 = parseFloat(mount.querySelector('#dr-yield').value) / 100;
    const divGrow = parseFloat(mount.querySelector('#dr-divgrow').value) / 100;
    const priceGrow = parseFloat(mount.querySelector('#dr-pricegrow').value) / 100;
    const years = parseInt(mount.querySelector('#dr-years').value, 10) || 1;
    const taxRate = parseFloat(mount.querySelector('#dr-tax').value) / 100;
    const result = mount.querySelector('#dr-result');
    if (initShares <= 0 || initPrice <= 0) {
        result.innerHTML = `<p class="muted">Shares and price > 0 required.</p>`;
        return;
    }

    const principal = initShares * initPrice;
    let dripShares = initShares;
    let cashShares = initShares;
    let cashDivsAccum = 0;
    let dripDivsAccum = 0;
    let dividendPerShare = initPrice * yield0;       // year-0 dividend per share
    let price = initPrice;
    const rows = [];

    for (let y = 1; y <= years; y++) {
        const yoyPrice = price * (1 + priceGrow);
        const yoyDiv = dividendPerShare * (1 + divGrow);
        // Approximate avg-price for DRIP purchase = mid-year price
        const dripPurchasePrice = (price + yoyPrice) / 2;
        const totalDripDiv = dripShares * yoyDiv;
        const afterTaxDripDiv = totalDripDiv * (1 - taxRate);
        dripShares += afterTaxDripDiv / dripPurchasePrice;
        dripDivsAccum += totalDripDiv;
        const cashDiv = cashShares * yoyDiv;
        cashDivsAccum += cashDiv;
        price = yoyPrice;
        dividendPerShare = yoyDiv;
        if ([1, 5, 10, 15, 20, 25, 30, 40, 50, 60].includes(y) || y === years) {
            const dripValue = dripShares * price;
            const cashValue = cashShares * price;
            rows.push({
                year: y,
                price,
                divPerShare: yoyDiv,
                dripShares,
                dripValue,
                cashShares,
                cashValue,
                cashDivsAccum,
                cashTotalReturn: cashValue + cashDivsAccum,
            });
        }
    }

    const finalRow = rows[rows.length - 1];
    const dripCagr = years > 0 ? Math.pow(finalRow.dripValue / principal, 1/years) - 1 : 0;
    const cashTotalReturn = finalRow.cashTotalReturn;
    const cashCagr = years > 0 ? Math.pow(cashTotalReturn / principal, 1/years) - 1 : 0;
    const dripAdvantage = finalRow.dripValue - cashTotalReturn;

    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Initial investment</div><div class="value">$${fmt(principal, 0)}</div></div>
            <div class="card"><div class="label">DRIP final value</div><div class="value pos">$${fmt(finalRow.dripValue, 0)}</div><div class="muted small">${fmt(finalRow.dripShares, 1)} sh @ $${fmt(finalRow.price, 2)} · CAGR ${fmt(dripCagr * 100, 2)}%</div></div>
            <div class="card"><div class="label">Cash divs final value</div><div class="value">$${fmt(cashTotalReturn, 0)}</div><div class="muted small">$${fmt(finalRow.cashValue, 0)} shares + $${fmt(finalRow.cashDivsAccum, 0)} cash · CAGR ${fmt(cashCagr * 100, 2)}%</div></div>
            <div class="card"><div class="label">DRIP advantage</div><div class="value pos">+$${fmt(dripAdvantage, 0)}</div><div class="muted small">${fmt(dripAdvantage / cashTotalReturn * 100, 2)}% better end-state</div></div>
        </div>
        <h3 class="section-title">Year-by-year</h3>
        <table class="trades" data-table-key="dr-rows">
            <thead><tr>
                <th>Year</th>
                <th>Share price</th>
                <th>Div/sh</th>
                <th>DRIP shares</th>
                <th>DRIP value</th>
                <th>Cash value</th>
                <th>Cash + cash divs</th>
            </tr></thead>
            <tbody>${rows.map(r => `<tr>
                <td><strong>${r.year}</strong></td>
                <td>$${fmt(r.price, 2)}</td>
                <td>$${fmt(r.divPerShare, 2)}</td>
                <td>${fmt(r.dripShares, 1)}</td>
                <td class="pos">$${fmt(r.dripValue, 0)}</td>
                <td>$${fmt(r.cashValue, 0)}</td>
                <td class="muted">$${fmt(r.cashTotalReturn, 0)}</td>
            </tr>`).join('')}</tbody>
        </table>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
