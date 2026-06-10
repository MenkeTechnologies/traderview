// I-Bond Calculator — Series I Treasury savings bonds. Combines a
// fixed rate (set at purchase, locked for life of bond) with an inflation
// rate (resets every 6 months based on CPI-U). Composite rate is
// Treasury's official formula:
//   composite = fixed + 2·inflation + fixed·inflation
// Bonds lock for 1 year (no withdrawal) and lose 3 months' interest if
// redeemed before 5 years. Max $10k/yr/person electronic + $5k paper.
// Tax: federal-taxable, state-exempt; deferrable until redemption.

import { esc } from '../util.js';
import { t } from '../i18n.js';

// Recent rate history (Treasury Direct, May 2022 → Nov 2024). Used as
// "what if you'd held through these resets" historical replay.
// Format: [period, fixed_at_issue (if purchased that period), composite_at_period]
const RATE_HISTORY = [
    { period: 'Nov 2024',  composite: 3.11, fixed: 1.20, semi_infl: 0.95 },
    { period: 'May 2024',  composite: 4.28, fixed: 1.30, semi_infl: 1.48 },
    { period: 'Nov 2023',  composite: 5.27, fixed: 1.30, semi_infl: 1.97 },
    { period: 'May 2023',  composite: 4.30, fixed: 0.90, semi_infl: 1.69 },
    { period: 'Nov 2022',  composite: 6.89, fixed: 0.40, semi_infl: 3.24 },
    { period: 'May 2022',  composite: 9.62, fixed: 0.00, semi_infl: 4.81 },
    { period: 'Nov 2021',  composite: 7.12, fixed: 0.00, semi_infl: 3.56 },
];

export async function renderIbondCalculator(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ibond_calculator.title">// I-BOND CALCULATOR</span></h1>
        <p class="muted small" data-i18n-html="view.ibond_calculator.intro">
            U.S. Treasury Series I Savings Bonds. Composite rate =
            <code>fixed + 2·inflation + fixed·inflation</code>. Fixed rate
            locks at purchase for the bond's life; inflation resets every
            6 months. <strong>1-year lockup</strong> (no redemption).
            <strong>3 months' interest penalty</strong> if redeemed before 5 years.
            Federal taxable, state-exempt. $10k/yr/person electronic cap.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Initial purchase $</span>
                    <input type="number" id="ib-amount" step="100" min="25" max="10000" value="10000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Fixed rate %</span>
                    <input type="number" id="ib-fixed" step="0.05" min="0" max="5" value="1.20" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Inflation rate %/semi</span>
                    <input type="number" id="ib-semi" step="0.05" min="-5" max="10" value="0.95" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Hold months</span>
                    <input type="number" id="ib-months" step="1" min="12" max="360" value="60" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="ib-run">⚡ Compute</button>
            <div id="ib-result" style="margin-top:12px"></div>
            <h3 class="section-title" style="margin-top:18px">Recent composite-rate history</h3>
            <table class="trades" data-table-key="ib-hist">
                <thead><tr><th>Period</th><th>Fixed %</th><th>Semi-annual inflation %</th><th>Composite %</th></tr></thead>
                <tbody>${RATE_HISTORY.map(h => `<tr>
                    <td>${esc(h.period)}</td>
                    <td>${fmt(h.fixed, 2)}%</td>
                    <td>${fmt(h.semi_infl, 2)}%</td>
                    <td><strong>${fmt(h.composite, 2)}%</strong></td>
                </tr>`).join('')}</tbody>
            </table>
        </div>
    `;
    mount.querySelectorAll('#ib-amount, #ib-fixed, #ib-semi, #ib-months').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#ib-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const amount = parseFloat(mount.querySelector('#ib-amount').value) || 0;
    const fixed_ann = parseFloat(mount.querySelector('#ib-fixed').value) / 100;
    const semi_infl = parseFloat(mount.querySelector('#ib-semi').value) / 100;
    const months = Math.max(1, parseInt(mount.querySelector('#ib-months').value, 10) || 0);
    const result = mount.querySelector('#ib-result');
    if (amount <= 0) {
        result.innerHTML = `<p class="muted">Enter a positive amount.</p>`;
        return;
    }

    // Treasury's official composite formula uses semi-annual inflation
    // rate directly (NOT annualized) — composite = fixed + 2·semi + fixed·semi.
    // We'll assume the semi-annual rate is held constant across all
    // resets; the user can experiment.
    const fixed_semi = fixed_ann / 2;
    const composite_ann = fixed_ann + 2 * semi_infl + fixed_ann * semi_infl;
    const r_month = Math.pow(1 + composite_ann, 1/12) - 1;

    let value = amount;
    let lastThreeMonths = 0;
    const rows = [];
    let cumInterest = 0;
    let redeemable = 0;
    for (let m = 1; m <= months; m++) {
        const interest = value * r_month;
        value += interest;
        cumInterest += interest;
        if (m >= months - 2) lastThreeMonths += interest;
        if (m === 12)  rows.push({ month: m, value, cum: cumInterest, redeemable: m >= 12 ? value - lastThreeMonthsAt(value, r_month) : 0, label: '1-year mark (lockup ends)' });
        if (m === 60)  rows.push({ month: m, value, cum: cumInterest, redeemable: value, label: '5-year mark (penalty drops)' });
        if (m === 120) rows.push({ month: m, value, cum: cumInterest, redeemable: value, label: '10-year mark' });
        if (m === 240) rows.push({ month: m, value, cum: cumInterest, redeemable: value, label: '20-year mark (orig issue)' });
        if (m === 360) rows.push({ month: m, value, cum: cumInterest, redeemable: value, label: '30-year final maturity' });
    }

    // Redeemable today (after penalty if before 60 months).
    const penaltyApplies = months < 60;
    if (penaltyApplies && months >= 12) {
        redeemable = value - lastThreeMonths;
    } else if (months >= 60) {
        redeemable = value;
    } else {
        redeemable = 0;
    }

    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Composite rate (annual)</div><div class="value pos">${fmt(composite_ann * 100, 2)}%</div><div class="muted small">${fmt(fixed_ann * 100, 2)}% fixed + ${fmt(semi_infl * 100, 2)}%/semi infl</div></div>
            <div class="card"><div class="label">Value at hold</div><div class="value">$${fmt(value, 2)}</div></div>
            <div class="card"><div class="label">Interest earned</div><div class="value pos">$${fmt(cumInterest, 2)}</div><div class="muted small">${fmt(cumInterest / amount * 100, 2)}% total return</div></div>
            <div class="card">
                <div class="label">Redeemable today</div>
                <div class="value ${penaltyApplies ? 'neg' : 'pos'}">$${fmt(redeemable, 2)}</div>
                <div class="muted small">${months < 12 ? 'Locked — &lt; 1yr' : penaltyApplies ? `Penalty −$${fmt(lastThreeMonths, 2)} (3mo interest)` : 'No penalty (≥5yr)'}</div>
            </div>
        </div>
        ${rows.length ? `
        <table class="trades" data-table-key="ib-milestones">
            <thead><tr><th>Milestone</th><th>Month</th><th>Value</th><th>Cum interest</th></tr></thead>
            <tbody>${rows.map(r => `<tr>
                <td>${esc(r.label)}</td>
                <td>${r.month}</td>
                <td><strong>$${fmt(r.value, 2)}</strong></td>
                <td class="pos">$${fmt(r.cum, 2)}</td>
            </tr>`).join('')}</tbody>
        </table>` : ''}
    `;
}

function lastThreeMonthsAt(value, r_month) {
    // Crude reverse-estimate of last 3 months of interest.
    return value - value / Math.pow(1 + r_month, 3);
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
