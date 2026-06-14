// Roth Conversion Ladder — early-retirement bridge. Convert $X/yr from a
// Traditional IRA to Roth; each conversion seasons 5 years before its principal
// is penalty-free. Computation runs server-side via /calc/roth-conversion-ladder
// (traderview-core::roth_conversion_ladder) — a faithful port of the former
// client-side bracket/ladder math, Python-pinned and unit-tested. Class-based
// styling for release-WebKit correctness.

import { api } from '../api.js';
import { applyUiI18n } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const fmt = (n, d) => (n == null || !Number.isFinite(Number(n)) ? '—' : Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d }));
const VIEW = 'roth-conversion-ladder';
let lastReport = null;
let lastBody = null;

export async function renderRothConversionLadder(mount, _state) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.roth_conversion_ladder.title">// ROTH CONVERSION LADDER</span></h1>
        <p class="muted small" data-i18n-html="view.roth_conversion_ladder.intro">
            Early-retirement bridge. Each $X moved from Traditional → Roth IRA
            becomes principal that can be withdrawn tax- and penalty-free
            after a <strong>5-year season-out period</strong> (per conversion).
            Stagger conversions to build a Roth "ladder" you can draw from
            before age 59½ without the 10% early-withdrawal penalty.
            Conversions count as ordinary income — size them to fill the
            <strong>12% or 22% brackets</strong> (whichever is your seam).
        </p>
        <div class="chart-panel">
            <div class="re-grid">
                <label><span class="muted small">Filing status</span>
                    <select id="rcl-status">
                        <option value="single">Single</option>
                        <option value="mfj" selected>MFJ</option>
                        <option value="mfs">MFS</option>
                        <option value="hoh">HoH</option>
                    </select></label>
                <label><span class="muted small">Other ordinary income / yr $</span>
                    <input type="number" id="rcl-other" step="500" min="0" value="0"></label>
                <label><span class="muted small">Annual conversion $</span>
                    <input type="number" id="rcl-amount" step="1000" min="0" value="60000"></label>
                <label><span class="muted small">Years to ladder</span>
                    <input type="number" id="rcl-years" step="1" min="1" max="30" value="10"></label>
                <label><span class="muted small">Current age</span>
                    <input type="number" id="rcl-age" step="1" min="18" max="65" value="45"></label>
                <label><span class="muted small">Trad IRA balance $</span>
                    <input type="number" id="rcl-balance" step="10000" min="0" value="1200000"></label>
            </div>
            <button class="btn btn-sm primary" id="rcl-run">⚡ Build ladder</button>
            <div id="rcl-tools" class="ce-toolbar"></div>
            <div id="rcl-result" class="re-result"></div>
        </div>
    `;
    applyUiI18n(mount);

    const num = (id) => parseFloat(mount.querySelector(id).value) || 0;
    const readBody = () => ({
        filing_status: mount.querySelector('#rcl-status').value,
        other_income_usd: num('#rcl-other'),
        annual_conversion_usd: num('#rcl-amount'),
        years: Math.max(1, Math.min(30, parseInt(mount.querySelector('#rcl-years').value, 10) || 1)),
        current_age: parseInt(mount.querySelector('#rcl-age').value, 10) || 45,
        traditional_balance_usd: num('#rcl-balance'),
    });
    const compute = async () => {
        const body = readBody();
        if (body.annual_conversion_usd <= 0) {
            mount.querySelector('#rcl-result').innerHTML = `<p class="muted">Conversion amount &gt; 0 required.</p>`;
            lastReport = null; return;
        }
        try {
            const r = await api.calcRothConversionLadder(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            await renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || 'Could not build the ladder.', { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#rcl-tools'), {
        viewId: VIEW, link: false, filename: 'roth-conversion-ladder.csv',
        getRows: () => reportRows(lastReport),
    });
    mount.querySelectorAll('#rcl-status, #rcl-other, #rcl-amount, #rcl-years, #rcl-age, #rcl-balance').forEach(el => {
        el.addEventListener('input', debounce(compute, 250));
    });
    mount.querySelector('#rcl-run').addEventListener('click', compute);
    compute();
}

function reportRows(r) {
    if (!r || !r.valid) return [];
    const rows = [['convert_year', 'age', 'converted_usd', 'tax_usd', 'effective_pct', 'marginal_pct', 'access_age', 'trad_balance_after_usd']];
    for (const x of r.rows) rows.push([x.convert_year, x.age, x.converted_usd, x.tax_on_conversion_usd, x.effective_rate_pct, x.marginal_rate_pct, x.access_age, x.traditional_balance_after_usd]);
    return rows;
}

async function renderResult(mount, r, body, tok) {
    const result = mount.querySelector('#rcl-result');
    if (!r.valid) { result.innerHTML = ''; return; }
    // Line chart: average effective conversion-tax rate as the annual amount sweeps
    // 0 → 3× current (rises as conversions spill into higher brackets).
    const amt = body.annual_conversion_usd || 60000;
    const xs = enh.linspace(amt * 0.25, amt * 3, 12);
    const pts = await Promise.all(xs.map(async (a) => {
        const rr = await api.calcRothConversionLadder({ ...body, annual_conversion_usd: a });
        return { x: a / 1000, y: rr && rr.valid ? rr.avg_effective_rate_pct : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'conversion $k', ylabel: 'avg eff %' });
    result.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label">Total converted</div><div class="value pos">$${fmt(r.total_converted_usd, 0)}</div></div>
            <div class="card"><div class="label">Total fed tax cost</div><div class="value neg">$${fmt(r.total_tax_usd, 0)}</div></div>
            <div class="card"><div class="label">Avg effective rate</div><div class="value">${fmt(r.avg_effective_rate_pct, 2)}%</div></div>
            <div class="card"><div class="label">Trad IRA remaining</div><div class="value">$${fmt(r.traditional_remaining_usd, 0)}</div></div>
            <div class="card"><div class="label">First access at age</div><div class="value">${r.first_access_age ?? '—'}</div></div>
        </div>
        ${chart}
        <table class="trades" data-table-key="rcl-rows">
            <thead><tr>
                <th>Year</th><th>Age</th><th>Convert $</th><th>Tax $</th><th>Effective</th><th>Marginal</th><th>Access age</th><th>Trad balance after</th>
            </tr></thead>
            <tbody>${r.rows.map(x => `<tr>
                <td>${x.convert_year}</td>
                <td>${x.age}</td>
                <td>$${fmt(x.converted_usd, 0)}</td>
                <td class="neg">$${fmt(x.tax_on_conversion_usd, 0)}</td>
                <td>${fmt(x.effective_rate_pct, 2)}%</td>
                <td>${fmt(x.marginal_rate_pct, 0)}%</td>
                <td><strong>${x.access_age}</strong> <span class="muted small">(${x.access_year})</span></td>
                <td class="muted">$${fmt(x.traditional_balance_after_usd, 0)}</td>
            </tr>`).join('')}</tbody>
        </table>
    `;
}
