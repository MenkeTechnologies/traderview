// I-Bond Calculator — Series I Treasury savings bonds. Combines a
// fixed rate (set at purchase, locked for life of bond) with an inflation
// rate (resets every 6 months based on CPI-U). Composite rate is
// Treasury's official formula:
//   composite = fixed + 2·inflation + fixed·inflation
// Bonds lock for 1 year (no withdrawal) and lose 3 months' interest if
// redeemed before 5 years. Max $10k/yr/person electronic + $5k paper.
// Tax: federal-taxable, state-exempt; deferrable until redemption.

import { api } from '../api.js';
import { esc } from '../util.js';
import { applyUiI18n, t } from '../i18n.js';

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
                    <span class="muted small" data-i18n="view.ibond_calculator.field.amount">Initial purchase $</span>
                    <input type="number" id="ib-amount" step="100" min="25" max="10000" value="10000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.ibond_calculator.field.fixed">Fixed rate %</span>
                    <input type="number" id="ib-fixed" step="0.05" min="0" max="5" value="1.20" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.ibond_calculator.field.semi">Inflation rate %/semi</span>
                    <input type="number" id="ib-semi" step="0.05" min="-5" max="10" value="0.95" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.ibond_calculator.field.months">Hold months</span>
                    <input type="number" id="ib-months" step="1" min="12" max="360" value="60" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="ib-run" data-i18n="view.ibond_calculator.btn.run">⚡ Compute</button>
            <div id="ib-result" style="margin-top:12px"></div>
            <h3 class="section-title" style="margin-top:18px" data-i18n="view.ibond_calculator.h3.history">Recent composite-rate history</h3>
            <table class="trades" data-table-key="ib-hist">
                <thead><tr><th data-i18n="view.ibond_calculator.th.period">Period</th><th data-i18n="view.ibond_calculator.th.fixed">Fixed %</th><th data-i18n="view.ibond_calculator.th.semi_infl">Semi-annual inflation %</th><th data-i18n="view.ibond_calculator.th.composite">Composite %</th></tr></thead>
                <tbody>${RATE_HISTORY.map(h => `<tr>
                    <td>${esc(h.period)}</td>
                    <td>${fmt(h.fixed, 2)}%</td>
                    <td>${fmt(h.semi_infl, 2)}%</td>
                    <td><strong>${fmt(h.composite, 2)}%</strong></td>
                </tr>`).join('')}</tbody>
            </table>
        </div>
    `;
    applyUiI18n(mount);
    mount.querySelectorAll('#ib-amount, #ib-fixed, #ib-semi, #ib-months').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#ib-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

async function compute(mount) {
    const result = mount.querySelector('#ib-result');
    const body = {
        amount_usd: parseFloat(mount.querySelector('#ib-amount').value) || 0,
        fixed_rate_pct: parseFloat(mount.querySelector('#ib-fixed').value) || 0,
        semi_inflation_pct: parseFloat(mount.querySelector('#ib-semi').value) || 0,
        hold_months: Math.max(1, parseInt(mount.querySelector('#ib-months').value, 10) || 0),
    };
    if (body.amount_usd <= 0) {
        result.innerHTML = `<p class="muted">${esc(t('view.ibond_calculator.empty.invalid'))}</p>`;
        return;
    }
    try {
        const r = await api.calcIbondCalculator(body);
        renderResult(result, r, body);
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

function renderResult(result, r, body) {
    const subComposite = t('view.ibond_calculator.sub.composite', {
        fixed: fmt(body.fixed_rate_pct, 2) + '%', semi: fmt(body.semi_inflation_pct, 2) + '%',
    });
    const subInterest = t('view.ibond_calculator.sub.interest', { pct: fmt(r.total_return_pct, 2) + '%' });
    const redeemSub = r.locked
        ? t('view.ibond_calculator.redeem.locked')
        : r.penalty_applies
            ? t('view.ibond_calculator.redeem.penalty', { amount: '$' + fmt(r.last_three_months_interest_usd, 2) })
            : t('view.ibond_calculator.redeem.nopenalty');
    const rows = r.rows.map((row) => `<tr>
                <td>${esc(t('view.ibond_calculator.ms.' + row.label_key))}</td>
                <td>${row.month}</td>
                <td><strong>$${fmt(row.value_usd, 2)}</strong></td>
                <td class="pos">$${fmt(row.cum_interest_usd, 2)}</td>
            </tr>`).join('');
    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label" data-i18n="view.ibond_calculator.card.composite">Composite rate (annual)</div><div class="value pos">${fmt(r.composite_annual_pct, 2)}%</div><div class="muted small">${esc(subComposite)}</div></div>
            <div class="card"><div class="label" data-i18n="view.ibond_calculator.card.value">Value at hold</div><div class="value">$${fmt(r.value_at_hold_usd, 2)}</div></div>
            <div class="card"><div class="label" data-i18n="view.ibond_calculator.card.interest">Interest earned</div><div class="value pos">$${fmt(r.interest_earned_usd, 2)}</div><div class="muted small">${esc(subInterest)}</div></div>
            <div class="card">
                <div class="label" data-i18n="view.ibond_calculator.card.redeemable">Redeemable today</div>
                <div class="value ${r.penalty_applies ? 'neg' : 'pos'}">$${fmt(r.redeemable_today_usd, 2)}</div>
                <div class="muted small">${esc(redeemSub)}</div>
            </div>
        </div>
        ${rows ? `
        <table class="trades" data-table-key="ib-milestones">
            <thead><tr><th data-i18n="view.ibond_calculator.th.milestone">Milestone</th><th data-i18n="view.ibond_calculator.th.month">Month</th><th data-i18n="view.ibond_calculator.th.value">Value</th><th data-i18n="view.ibond_calculator.th.cum">Cum interest</th></tr></thead>
            <tbody>${rows}</tbody>
        </table>` : ''}
    `;
    applyUiI18n(result);
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
