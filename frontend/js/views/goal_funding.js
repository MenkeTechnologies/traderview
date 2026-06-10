// Goal Funding — "how much per month / year do I need to save to
// reach $X target in N years at rate r?" Inverse of FV-with-PMT.
// Three modes: monthly contribution, lump-sum today, or hybrid
// (lump-sum + monthly). Includes sensitivity table across rate × horizon.

import { esc } from '../util.js';

export async function renderGoalFunding(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.goal_funding.title">// GOAL FUNDING CALCULATOR</span></h1>
        <p class="muted small" data-i18n-html="view.goal_funding.intro">
            Given a target dollar amount and a horizon, compute exactly how much
            you need to invest per period to hit it. Three modes: pure monthly
            DCA, single lump-sum today, or hybrid (some now + monthly top-up).
            Sensitivity table shows how the required monthly varies if your
            assumed return or horizon shifts.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Target $</span>
                    <input type="number" id="gf-target" step="1000" min="0" value="200000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Horizon (years)</span>
                    <input type="number" id="gf-years" step="0.5" min="0.5" max="60" value="10" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Expected return %/yr</span>
                    <input type="number" id="gf-rate" step="0.1" min="-10" max="30" value="6" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Starting balance $</span>
                    <input type="number" id="gf-start" step="500" min="0" value="10000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Goal label</span>
                    <input type="text" id="gf-label" value="Down payment" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="gf-run">⚡ Compute</button>
            <div id="gf-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#gf-target, #gf-years, #gf-rate, #gf-start, #gf-label').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#gf-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const target = parseFloat(mount.querySelector('#gf-target').value) || 0;
    const years = parseFloat(mount.querySelector('#gf-years').value) || 0;
    const rate = parseFloat(mount.querySelector('#gf-rate').value) / 100;
    const start = parseFloat(mount.querySelector('#gf-start').value) || 0;
    const label = mount.querySelector('#gf-label').value || 'Goal';
    const result = mount.querySelector('#gf-result');
    if (target <= 0 || years <= 0) {
        result.innerHTML = `<p class="muted">Target > 0 and horizon > 0 required.</p>`;
        return;
    }

    const n = years * 12;
    const r_m = Math.pow(1 + rate, 1/12) - 1;
    const startFv = start * Math.pow(1 + rate, years);
    const gap = Math.max(0, target - startFv);

    // Solve PMT for FV-annuity: gap = PMT * ((1+r)^n − 1) / r
    let pmt;
    if (r_m === 0) pmt = gap / n;
    else pmt = gap * r_m / (Math.pow(1 + r_m, n) - 1);

    // Pure lump-sum-today required.
    const lumpToday = target / Math.pow(1 + rate, years);
    const additionalLumpNeeded = Math.max(0, lumpToday - start);

    // Sensitivity table over rate (4%/6%/8%/10%) × horizon (years ÷ 2, years, years × 1.5).
    const rateGrid = [0.04, 0.06, 0.08, 0.10];
    const yearGrid = [Math.max(1, Math.round(years / 2)), years, Math.round(years * 1.5)];
    const sens = yearGrid.map(yr => ({
        years: yr,
        cells: rateGrid.map(rt => {
            const rm = Math.pow(1 + rt, 1/12) - 1;
            const nm = yr * 12;
            const sFv = start * Math.pow(1 + rt, yr);
            const g = Math.max(0, target - sFv);
            const p = rm === 0 ? g / nm : g * rm / (Math.pow(1 + rm, nm) - 1);
            return { rate: rt, pmt: p };
        }),
    }));

    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
            <div class="card">
                <div class="label">${esc(label)} target</div>
                <div class="value">$${fmt(target, 0)}</div>
                <div class="muted small">in ${years} years</div>
            </div>
            <div class="card">
                <div class="label">From $${fmt(start, 0)} starting balance</div>
                <div class="value">→ $${fmt(startFv, 0)}</div>
                <div class="muted small">Grown @ ${fmt(rate * 100, 1)}%/yr</div>
            </div>
            <div class="card">
                <div class="label">Gap to close</div>
                <div class="value ${gap > 0 ? 'neg' : 'pos'}">$${fmt(gap, 0)}</div>
            </div>
            <div class="card">
                <div class="label">Required monthly</div>
                <div class="value pos"><strong>$${fmt(pmt, 0)}</strong></div>
                <div class="muted small">$${fmt(pmt * 12, 0)}/yr</div>
            </div>
            <div class="card">
                <div class="label">Or lump-sum today</div>
                <div class="value">$${fmt(additionalLumpNeeded, 0)}</div>
                <div class="muted small">Total need today: $${fmt(lumpToday, 0)}</div>
            </div>
        </div>
        <h3 class="section-title">Sensitivity — required monthly $</h3>
        <table class="trades" data-table-key="gf-sens">
            <thead><tr>
                <th>Horizon ↓ / Rate →</th>
                ${rateGrid.map(r => `<th>${fmt(r * 100, 0)}%/yr</th>`).join('')}
            </tr></thead>
            <tbody>${sens.map(row => `<tr>
                <td><strong>${row.years} yr</strong></td>
                ${row.cells.map(c => `<td>$${fmt(c.pmt, 0)}/mo</td>`).join('')}
            </tr>`).join('')}</tbody>
        </table>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
