// Inflation calculator — purchasing-power adjuster. Given a present-day
// dollar amount and a horizon, computes what that amount needs to grow to
// in nominal terms to preserve real purchasing power at a target average
// inflation rate. Also runs the inverse: what $X today equals in real
// dollars after N years of inflation.
//
// All math is closed-form, no backend hop. Default rate: 3.2% (long-run
// US CPI-U average ~1960-2024 per BLS).

import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderInflationCalculator(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.inflation_calculator.title">// INFLATION CALCULATOR</span></h1>
        <p class="muted small" data-i18n-html="view.inflation_calculator.intro">
            Two-way calculator: <strong>future nominal $</strong> needed to preserve
            today's purchasing power across a horizon, and the <strong>real value</strong>
            of a future amount discounted back to today. Default 3.2% is the long-run
            US CPI-U average. Use 2% for the Fed target, 4-5% for the
            2021-2024 spike, or your own assumption for a stress test.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.inflation_calculator.field.amount">Amount $ (today)</span>
                    <input type="number" id="ic-amount" step="100" min="0" value="100000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.inflation_calculator.field.years">Years horizon</span>
                    <input type="number" id="ic-years" step="1" min="1" max="100" value="30" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.inflation_calculator.field.rate">Annual inflation rate %</span>
                    <input type="number" id="ic-rate" step="0.1" min="-5" max="20" value="3.2" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="ic-run" data-shortcut="r" data-i18n="view.inflation_calculator.btn.run">⚡ Compute</button>
            <div id="ic-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelector('#ic-run').addEventListener('click', () => runCompute(mount));
    runCompute(mount);
}

function runCompute(mount) {
    const result = mount.querySelector('#ic-result');
    const A = parseFloat(mount.querySelector('#ic-amount').value) || 0;
    const Y = parseInt(mount.querySelector('#ic-years').value, 10) || 0;
    const R = parseFloat(mount.querySelector('#ic-rate').value) / 100;
    if (A <= 0 || Y <= 0) {
        result.innerHTML = `<p class="muted">${esc(t('view.inflation_calculator.empty.invalid') || 'Enter a positive amount and horizon.')}</p>`;
        return;
    }
    const factor = Math.pow(1 + R, Y);
    const future_nominal = A * factor;
    const future_real = A / factor;
    // Year-by-year erosion of $A's real value.
    const series = [];
    for (let y = 0; y <= Y; y++) {
        const f = Math.pow(1 + R, y);
        series.push({
            year: y,
            nominal_target: A * f,        // what you need in year y to equal $A today
            real_value: A / f,            // what $A today is worth in year y
            cumulative_pct: (f - 1) * 100,
        });
    }
    const summaryCls = R > 0 ? 'neg' : R < 0 ? 'pos' : 'muted';
    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
            <div class="card">
                <div class="label">${esc(t('view.inflation_calculator.card.future_nominal') || 'Future $ to preserve power')}</div>
                <div class="value ${summaryCls}">$${fmt(future_nominal, 0)}</div>
                <div class="muted small">In year ${Y} = $${fmt(A, 0)} today</div>
            </div>
            <div class="card">
                <div class="label">${esc(t('view.inflation_calculator.card.real_value') || 'Today $ → real value in year ' + Y)}</div>
                <div class="value ${summaryCls}">$${fmt(future_real, 0)}</div>
                <div class="muted small">${fmt(future_real / A * 100, 1)}% of original purchasing power</div>
            </div>
            <div class="card">
                <div class="label">${esc(t('view.inflation_calculator.card.cumulative') || 'Cumulative inflation')}</div>
                <div class="value ${summaryCls}">${fmt((factor - 1) * 100, 1)}%</div>
                <div class="muted small">over ${Y} years @ ${fmt(R * 100, 2)}%/yr</div>
            </div>
        </div>
        <table class="trades" data-table-key="inflation-series">
            <thead><tr>
                <th>Year</th>
                <th>Nominal $ to equal $${fmt(A, 0)}</th>
                <th>$${fmt(A, 0)} → real value</th>
                <th>Cumulative %</th>
            </tr></thead>
            <tbody>${series.map(s => `<tr>
                <td>${s.year}</td>
                <td><strong>$${fmt(s.nominal_target, 0)}</strong></td>
                <td class="muted">$${fmt(s.real_value, 0)}</td>
                <td class="${s.cumulative_pct >= 0 ? 'neg' : 'pos'}">${s.cumulative_pct >= 0 ? '+' : ''}${fmt(s.cumulative_pct, 1)}%</td>
            </tr>`).join('')}</tbody>
        </table>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
