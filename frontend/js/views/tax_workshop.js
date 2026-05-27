// Tax Workshop — surfaces the 5 IRS calculators in one view.
//
// All five hit pure-compute POST endpoints in expense_routes.rs. The
// underlying math lives in traderview-expense and is unit-tested against
// hand-rolled IRS scenarios (Schedule SE wage-base cap, mileage mid-year
// rate split, simplified home office cap, subscription cadence detection,
// 1040-ES safe harbor selection).

import { api } from '../api.js';
import { esc, fmtMoney, fmtPct } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderTaxWorkshop(mount, _state) {
    if (!mount) return;
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 class="view-title">// TAX WORKSHOP</h1>
        <p class="muted small">
            Trader-as-business calculators. Math lives in <code>traderview-expense</code>
            and is unit-tested against hand-rolled IRS scenarios. Inputs are
            <em>local-only</em> — nothing leaves your machine in desktop mode.
        </p>

        <div class="chart-panel">
            <h2>// Schedule SE — Self-Employment Tax</h2>
            <form id="se-form" class="inline-form">
                <label>Net profit (Schedule C line 31)
                    <input name="net_profit_schedule_c" type="number" step="any" value="50000" required></label>
                <label>W-2 SS wages YTD
                    <input name="w2_ss_wages" type="number" step="any" value="0"></label>
                <label>Filing status
                    <select name="filing_status">
                        <option value="single">Single</option>
                        <option value="married_joint">Married Joint</option>
                        <option value="married_separate">Married Separate</option>
                        <option value="head_of_household">Head of Household</option>
                    </select></label>
                <label>Tax year
                    <input name="year" type="number" value="2026" required></label>
                <button class="primary" type="submit">Compute</button>
            </form>
            <pre id="se-out" class="boot">—</pre>
        </div>

        <div class="chart-panel">
            <h2>// Home Office — Form 8829 vs Simplified</h2>
            <form id="ho-form" class="inline-form">
                <label>Business sqft
                    <input name="business_use_sqft" type="number" step="any" value="200" required></label>
                <label>Total home sqft
                    <input name="total_home_sqft" type="number" step="any" value="2000" required></label>
                <label>Annual mortgage interest
                    <input name="annual_mortgage_interest" type="number" step="any" value="0"></label>
                <label>Annual property tax
                    <input name="annual_property_tax" type="number" step="any" value="0"></label>
                <label>Annual utilities
                    <input name="annual_utilities" type="number" step="any" value="0"></label>
                <label>Annual insurance
                    <input name="annual_insurance" type="number" step="any" value="0"></label>
                <label>Annual repairs
                    <input name="annual_repairs" type="number" step="any" value="0"></label>
                <label>Annual depreciation
                    <input name="annual_depreciation" type="number" step="any" value="0"></label>
                <button class="primary" type="submit">Compute</button>
            </form>
            <pre id="ho-out" class="boot">—</pre>
        </div>

        <div class="chart-panel">
            <h2>// Mileage — Standard Method</h2>
            <p class="muted small">Add trips. IRS standard rate is applied per trip date (handles 2022 mid-year split).</p>
            <form id="mi-form" class="inline-form">
                <label>Date <input name="date" type="date" required></label>
                <label>Miles <input name="miles" type="number" step="any" required></label>
                <label>Purpose
                    <select name="purpose">
                        <option value="business">Business</option>
                        <option value="medical">Medical</option>
                        <option value="moving">Moving</option>
                        <option value="charitable">Charitable</option>
                    </select></label>
                <label>Note <input name="note" type="text" placeholder="e.g. Conference travel"></label>
                <button class="primary" type="submit">Add Trip</button>
            </form>
            <table class="trades" id="mi-table">
                <thead><tr><th>Date</th><th>Miles</th><th>Purpose</th><th>Note</th></tr></thead>
                <tbody><tr><td colspan="4" class="muted">No trips added yet.</td></tr></tbody>
            </table>
            <button id="mi-compute" class="primary" type="button" style="margin-top:8px">Compute Deduction</button>
            <pre id="mi-out" class="boot">—</pre>
        </div>

        <div class="chart-panel">
            <h2>// Quarterly Estimated Tax — Form 1040-ES</h2>
            <form id="qt-form" class="inline-form">
                <label>Tax year
                    <input name="tax_year" type="number" value="2026" required></label>
                <label>Prior year total tax
                    <input name="prior_year_total_tax" type="number" step="any" value="20000" required></label>
                <label>Prior year AGI
                    <input name="prior_year_agi" type="number" step="any" value="100000" required></label>
                <label>YTD net profit
                    <input name="ytd_net_profit" type="number" step="any" value="30000" required></label>
                <label>Days through YTD
                    <input name="days_through_ytd" type="number" value="90" required></label>
                <label>Estimated effective tax rate (decimal, e.g. 0.28)
                    <input name="estimated_effective_tax_rate" type="number" step="any" value="0.28" required></label>
                <label>Withholding YTD
                    <input name="withholding_ytd" type="number" step="any" value="0"></label>
                <button class="primary" type="submit">Forecast</button>
            </form>
            <pre id="qt-out" class="boot">—</pre>
        </div>

        <div class="chart-panel">
            <h2>// Recurring Subscription Detector</h2>
            <p class="muted small">Scans your imported transactions for monthly/quarterly/annual charges with stable amounts. Largest annual leak first.</p>
            <button id="sub-run" class="primary" type="button">Detect from my transactions</button>
            <table class="trades" id="sub-table">
                <thead><tr>
                    <th>Merchant</th><th>Cadence</th><th>Median</th><th>Samples</th><th>Projected/yr</th>
                </tr></thead>
                <tbody><tr><td colspan="5" class="muted">Click detect to scan.</td></tr></tbody>
            </table>
        </div>
    `;

    // ---- Schedule SE ---------------------------------------------------
    mount.querySelector('#se-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const body = formAsJson(e.target, ['net_profit_schedule_c', 'w2_ss_wages', 'year']);
        try {
            const r = await api.calcSelfEmploymentTax(body);
            if (!viewIsCurrent(tok)) return;
            renderSe(mount, r);
        } catch (err) { showError(mount, '#se-out', err); }
    });

    // ---- Home Office ---------------------------------------------------
    mount.querySelector('#ho-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const body = formAsJson(e.target, [
            'business_use_sqft', 'total_home_sqft',
            'annual_mortgage_interest', 'annual_property_tax',
            'annual_utilities', 'annual_insurance',
            'annual_repairs', 'annual_depreciation',
        ]);
        try {
            const r = await api.calcHomeOffice(body);
            if (!viewIsCurrent(tok)) return;
            renderHo(mount, r);
        } catch (err) { showError(mount, '#ho-out', err); }
    });

    // ---- Mileage (in-memory list) --------------------------------------
    const trips = [];
    const refreshMiTable = () => {
        const tb = mount.querySelector('#mi-table tbody');
        if (!tb) return;
        if (!trips.length) {
            tb.innerHTML = '<tr><td colspan="4" class="muted">No trips added yet.</td></tr>';
            return;
        }
        tb.innerHTML = trips.map(t =>
            `<tr><td>${esc(t.date)}</td><td>${esc(String(t.miles))}</td>
             <td>${esc(t.purpose)}</td><td>${esc(t.note)}</td></tr>`
        ).join('');
    };
    mount.querySelector('#mi-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        trips.push({
            date: fd.get('date'),
            miles: Number(fd.get('miles')),
            purpose: fd.get('purpose'),
            note: fd.get('note') || '',
        });
        e.target.reset();
        refreshMiTable();
    });
    mount.querySelector('#mi-compute').addEventListener('click', async () => {
        if (!trips.length) { showError(mount, '#mi-out', new Error('Add at least one trip first.')); return; }
        try {
            const r = await api.calcMileage(trips);
            if (!viewIsCurrent(tok)) return;
            renderMi(mount, r);
        } catch (err) { showError(mount, '#mi-out', err); }
    });

    // ---- Quarterly tax -------------------------------------------------
    mount.querySelector('#qt-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const body = formAsJson(e.target, [
            'tax_year', 'days_through_ytd',
            'prior_year_total_tax', 'prior_year_agi', 'ytd_net_profit',
            'estimated_effective_tax_rate', 'withholding_ytd',
        ]);
        try {
            const r = await api.calcQuarterlyTax(body);
            if (!viewIsCurrent(tok)) return;
            renderQt(mount, r);
        } catch (err) { showError(mount, '#qt-out', err); }
    });

    // ---- Subscriptions -------------------------------------------------
    mount.querySelector('#sub-run').addEventListener('click', async () => {
        const tb = mount.querySelector('#sub-table tbody');
        if (tb) tb.innerHTML = '<tr><td colspan="5" class="muted">Scanning…</td></tr>';
        try {
            const subs = await api.detectSubscriptions();
            if (!viewIsCurrent(tok)) return;
            renderSubs(mount, subs);
        } catch (err) {
            if (tb) tb.innerHTML = `<tr><td colspan="5" class="muted">Error: ${esc(err.message)}</td></tr>`;
        }
    });
}

// ---- helpers ---------------------------------------------------------------

function formAsJson(form, numericFields) {
    const fd = new FormData(form);
    const out = {};
    for (const [k, v] of fd.entries()) {
        out[k] = numericFields.includes(k) ? Number(v) : v;
    }
    return out;
}

function showError(mount, sel, err) {
    const el = mount.querySelector(sel);
    if (el) el.textContent = `Error: ${err.message || err}`;
}

function renderSe(mount, r) {
    const el = mount.querySelector('#se-out');
    if (!el) return;
    el.textContent =
`Net SE earnings (line 4a):     ${fmtMoney(r.net_se_earnings)}
Social Security (12.4%):       ${fmtMoney(r.social_security_tax)}
Medicare (2.9%):               ${fmtMoney(r.medicare_tax)}
Additional Medicare (0.9%):    ${fmtMoney(r.additional_medicare_tax)}
─────────────────────────────────────────
Total SE tax (Sched 2 line 4): ${fmtMoney(r.total_se_tax)}
Half deductible (Sched 1 ln15):${fmtMoney(r.deductible_half)}`;
}

function renderHo(mount, r) {
    const el = mount.querySelector('#ho-out');
    if (!el) return;
    el.textContent =
`Simplified ($5/sqft, cap $1,500):  ${fmtMoney(r.simplified_deduction)}
Actual (Form 8829):                ${fmtMoney(r.actual_deduction)}
Business use %:                    ${fmtPct(r.business_pct)}
─────────────────────────────────────────
Recommended (${r.recommended_method}):     ${fmtMoney(r.recommended_deduction)}`;
}

function renderMi(mount, r) {
    const el = mount.querySelector('#mi-out');
    if (!el) return;
    el.textContent =
`Total miles:        ${r.total_miles}
  Business:         ${r.business_miles}  → ${fmtMoney(r.deduction_business)}
  Medical:          ${r.medical_miles}  → ${fmtMoney(r.deduction_medical)}
  Moving:           ${r.moving_miles}  → ${fmtMoney(r.deduction_moving)}
  Charitable:       ${r.charitable_miles}  → ${fmtMoney(r.deduction_charitable)}
Unrated (out-of-range): ${r.unrated_trips}
─────────────────────────────────────────
Total deduction:    ${fmtMoney(r.deduction_total)}`;
}

function renderQt(mount, r) {
    const el = mount.querySelector('#qt-out');
    if (!el) return;
    const q = r.quarters;
    el.textContent =
`Safe harbor (prior year):   ${fmtMoney(r.safe_harbor_prior_year)}
Safe harbor (current year): ${fmtMoney(r.safe_harbor_current_year)}
Target (the smaller):       ${fmtMoney(r.safe_harbor_target)}
Projected annual profit:    ${fmtMoney(r.projected_annual_net_profit)}
Projected annual tax:       ${fmtMoney(r.projected_annual_tax)}
Remaining after withholding:${fmtMoney(r.remaining_to_pay)}
─────────────────────────────────────────
${q[0].period_label}  due ${q[0].due_date}: ${fmtMoney(q[0].estimated_payment)}
${q[1].period_label}  due ${q[1].due_date}: ${fmtMoney(q[1].estimated_payment)}
${q[2].period_label}  due ${q[2].due_date}: ${fmtMoney(q[2].estimated_payment)}
${q[3].period_label}  due ${q[3].due_date}: ${fmtMoney(q[3].estimated_payment)}`;
}

function renderSubs(mount, subs) {
    const tb = mount.querySelector('#sub-table tbody');
    if (!tb) return;
    if (!subs.length) {
        tb.innerHTML = '<tr><td colspan="5" class="muted">No recurring subscriptions detected.</td></tr>';
        return;
    }
    tb.innerHTML = subs.map(s => `
        <tr>
            <td><strong>${esc(s.merchant)}</strong></td>
            <td>${esc(s.cadence)}</td>
            <td>${fmtMoney(s.median_amount)}</td>
            <td>${s.samples}</td>
            <td>${fmtMoney(s.projected_annual_cost)}</td>
        </tr>
    `).join('');
}
