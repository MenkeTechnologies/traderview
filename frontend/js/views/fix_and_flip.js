// Fix-and-flip underwriting — the 70% rule max-allowable-offer plus the full
// deal P&L (holding, financing, selling costs → net profit, cash-on-cash,
// annualized ROI), via /calc/fix-and-flip. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['arv_usd', 'After-repair value ($)', 300000],
    ['purchase_price_usd', 'Purchase price ($)', 150000],
    ['rehab_cost_usd', 'Rehab cost ($)', 50000],
    ['rule_pct', 'Rule (% of ARV)', 70],
    ['holding_months', 'Holding period (months)', 6],
    ['monthly_holding_cost_usd', 'Monthly holding cost ($)', 1000],
    ['buying_closing_cost_usd', 'Buying closing cost ($)', 5000],
    ['loan_amount_usd', 'Loan amount ($)', 0],
    ['financing_points_pct', 'Loan points (%)', 0],
    ['annual_interest_rate_pct', 'Loan interest (%/yr)', 0],
    ['selling_cost_pct', 'Selling cost (% of ARV)', 6],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const pct = (n) => Number(n).toFixed(1) + '%';

export async function renderFixAndFlip(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.flip.h1.title">// FIX & FLIP UNDERWRITING</span></h1>
        <p class="muted small" data-i18n="view.flip.hint.intro">
            Two questions every flipper asks. What's the most I can pay? The 70% rule: max
            allowable offer = ARV × 70% − rehab; the 30% haircut covers holding, financing,
            selling, and profit. What do I make? Net profit = sale proceeds (ARV less selling
            costs) minus every cost — purchase, rehab, closing, the monthly carry, and the
            financing (points + interest). Cash-on-cash divides profit by the cash you put
            in (all-in cost less the loan) and annualizes by the hold. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.flip.h2.inputs">The deal</h2>
            <form id="flip-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.flip.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="flip-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#flip-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcFixAndFlip(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.flip.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#flip-result');
    const ruleCls = r.passes_rule ? 'pos' : 'neg';
    const profitCls = r.net_profit_usd >= 0 ? 'pos' : 'neg';
    const annual = r.annualized_roi_pct == null ? '—' : pct(r.annualized_roi_pct);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.flip.h2.result">The numbers</h2>
            <div class="cards">
                <div class="card ${ruleCls}"><div class="label" data-i18n="view.flip.card.mao">Max allowable offer</div>
                    <div class="value">${money(r.max_allowable_offer_usd)}</div></div>
                <div class="card ${ruleCls}"><div class="label" data-i18n="view.flip.card.rule">70% rule</div>
                    <div class="value ${ruleCls}">${r.passes_rule ? t('view.flip.passes') : t('view.flip.fails')}</div></div>
                <div class="card ${profitCls}"><div class="label" data-i18n="view.flip.card.profit">Net profit</div>
                    <div class="value ${profitCls}">${money(r.net_profit_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.flip.card.coc">Cash-on-cash</div>
                    <div class="value">${pct(r.cash_on_cash_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.flip.card.annual">Annualized ROI</div>
                    <div class="value">${annual}</div></div>
                <div class="card"><div class="label" data-i18n="view.flip.card.margin">Profit margin (of ARV)</div>
                    <div class="value">${pct(r.profit_margin_pct)}</div></div>
            </div>
            <table class="data-table">
                <thead><tr>
                    <th data-i18n="view.flip.col.line">Line</th>
                    <th data-i18n="view.flip.col.amount">Amount</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.flip.row.holding">Holding cost</td><td>${money(r.holding_cost_usd)}</td></tr>
                    <tr><td data-i18n="view.flip.row.points">Loan points</td><td>${money(r.financing_points_usd)}</td></tr>
                    <tr><td data-i18n="view.flip.row.interest">Interest</td><td>${money(r.interest_cost_usd)}</td></tr>
                    <tr><td data-i18n="view.flip.row.selling">Selling costs</td><td>${money(r.selling_cost_usd)}</td></tr>
                    <tr><td data-i18n="view.flip.row.total">Total project cost</td><td>${money(r.total_project_cost_usd)}</td></tr>
                    <tr><td data-i18n="view.flip.row.proceeds">Net sale proceeds</td><td>${money(r.net_sale_proceeds_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.flip.row.cash">Cash invested</td><td>${money(r.cash_invested_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
