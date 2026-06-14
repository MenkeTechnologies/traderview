// DRIP Simulator — Dividend Reinvestment Plan compounding. Every paid
// dividend buys more shares at the then-current price. Over decades,
// reinvested dividends dwarf the original principal — for many quality
// dividend stocks, total return is ~60% dividends-reinvested and ~40%
// price appreciation. Compares DRIP vs cash dividends side by side.

import { api } from '../api.js';
import { esc } from '../util.js';
import { applyUiI18n, t } from '../i18n.js';

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
                    <span class="muted small" data-i18n="view.drip_simulator.field.shares">Shares purchased</span>
                    <input type="number" id="dr-shares" step="1" min="1" value="100" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.drip_simulator.field.price">Price per share $</span>
                    <input type="number" id="dr-price" step="0.5" min="0" value="100" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.drip_simulator.field.yield">Starting yield %</span>
                    <input type="number" id="dr-yield" step="0.1" min="0" max="20" value="3.0" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.drip_simulator.field.divgrow">Dividend growth %/yr</span>
                    <input type="number" id="dr-divgrow" step="0.1" min="0" max="30" value="5.0" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.drip_simulator.field.pricegrow">Price growth %/yr</span>
                    <input type="number" id="dr-pricegrow" step="0.1" min="-10" max="30" value="5.0" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.drip_simulator.field.years">Holding period (years)</span>
                    <input type="number" id="dr-years" step="1" min="1" max="60" value="30" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.drip_simulator.field.tax">Dividend tax rate % (DRIP only)</span>
                    <input type="number" id="dr-tax" step="1" min="0" max="50" value="15" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="dr-run" data-i18n="view.drip_simulator.btn.run">⚡ Simulate</button>
            <div id="dr-result" style="margin-top:12px"></div>
        </div>
    `;
    applyUiI18n(mount);
    mount.querySelectorAll('#dr-shares, #dr-price, #dr-yield, #dr-divgrow, #dr-pricegrow, #dr-years, #dr-tax').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#dr-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

async function compute(mount) {
    const result = mount.querySelector('#dr-result');
    const body = {
        shares: parseFloat(mount.querySelector('#dr-shares').value) || 0,
        price_usd: parseFloat(mount.querySelector('#dr-price').value) || 0,
        starting_yield_pct: parseFloat(mount.querySelector('#dr-yield').value) || 0,
        dividend_growth_pct: parseFloat(mount.querySelector('#dr-divgrow').value) || 0,
        price_growth_pct: parseFloat(mount.querySelector('#dr-pricegrow').value) || 0,
        years: parseInt(mount.querySelector('#dr-years').value, 10) || 1,
        dividend_tax_pct: parseFloat(mount.querySelector('#dr-tax').value) || 0,
    };
    if (body.shares <= 0 || body.price_usd <= 0) {
        result.innerHTML = `<p class="muted">${esc(t('view.drip_simulator.empty.invalid'))}</p>`;
        return;
    }
    try {
        const r = await api.calcDripSimulator(body);
        renderResult(result, r);
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

function renderResult(result, r) {
    const subDrip = t('view.drip_simulator.sub.drip', {
        shares: fmt(r.drip_shares, 1), price: fmt(r.final_price_usd, 2), cagr: fmt(r.drip_cagr_pct, 2) + '%',
    });
    const subCash = t('view.drip_simulator.sub.cash', {
        shares_val: '$' + fmt(r.cash_share_value_usd, 0), cash: '$' + fmt(r.cash_divs_accum_usd, 0), cagr: fmt(r.cash_cagr_pct, 2) + '%',
    });
    const subAdv = t('view.drip_simulator.sub.advantage', { pct: fmt(r.drip_advantage_pct, 2) + '%' });
    const rows = r.rows.map((row) => `<tr>
                <td><strong>${row.year}</strong></td>
                <td>$${fmt(row.price_usd, 2)}</td>
                <td>$${fmt(row.div_per_share_usd, 2)}</td>
                <td>${fmt(row.drip_shares, 1)}</td>
                <td class="pos">$${fmt(row.drip_value_usd, 0)}</td>
                <td>$${fmt(row.cash_value_usd, 0)}</td>
                <td class="muted">$${fmt(row.cash_total_return_usd, 0)}</td>
            </tr>`).join('');
    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label" data-i18n="view.drip_simulator.card.principal">Initial investment</div><div class="value">$${fmt(r.principal_usd, 0)}</div></div>
            <div class="card"><div class="label" data-i18n="view.drip_simulator.card.drip">DRIP final value</div><div class="value pos">$${fmt(r.drip_final_value_usd, 0)}</div><div class="muted small">${esc(subDrip)}</div></div>
            <div class="card"><div class="label" data-i18n="view.drip_simulator.card.cash">Cash divs final value</div><div class="value">$${fmt(r.cash_total_return_usd, 0)}</div><div class="muted small">${esc(subCash)}</div></div>
            <div class="card"><div class="label" data-i18n="view.drip_simulator.card.advantage">DRIP advantage</div><div class="value pos">+$${fmt(r.drip_advantage_usd, 0)}</div><div class="muted small">${esc(subAdv)}</div></div>
        </div>
        <h3 class="section-title" data-i18n="view.drip_simulator.h3.yearly">Year-by-year</h3>
        <table class="trades" data-table-key="dr-rows">
            <thead><tr>
                <th data-i18n="view.drip_simulator.th.year">Year</th>
                <th data-i18n="view.drip_simulator.th.price">Share price</th>
                <th data-i18n="view.drip_simulator.th.divsh">Div/sh</th>
                <th data-i18n="view.drip_simulator.th.drip_shares">DRIP shares</th>
                <th data-i18n="view.drip_simulator.th.drip_value">DRIP value</th>
                <th data-i18n="view.drip_simulator.th.cash_value">Cash value</th>
                <th data-i18n="view.drip_simulator.th.cash_total">Cash + cash divs</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
    `;
    applyUiI18n(result);
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
