// Mortgage Payoff vs Invest — common dilemma. Given an existing mortgage
// + extra cash/month, compare: (a) put extra against principal (saves
// interest, frees up cash earlier) vs (b) invest the extra in a market
// portfolio at expected return r. Reports end-state net wealth at
// horizon — house equity + investment account in both paths.
// Includes mortgage-interest tax deduction adjustment for itemizers.

import { api } from '../api.js';
import { esc } from '../util.js';
import { applyUiI18n, t } from '../i18n.js';

export async function renderMortgagePayoffVsInvest(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.mortgage_payoff_vs_invest.title">// MORTGAGE PAYOFF vs INVEST</span></h1>
        <p class="muted small" data-i18n-html="view.mortgage_payoff_vs_invest.intro">
            Decision matrix for: <strong>extra cash/mo against mortgage</strong>
            vs <strong>same cash into the market</strong>. Compares end-state
            net wealth (house equity + investment account) across N years.
            If your mortgage rate (after itemization) is below your expected
            return, investing wins on expected value but loses on certainty.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.mortgage_payoff_vs_invest.field.balance">Current mortgage balance $</span>
                    <input type="number" id="mpi-bal" step="1000" min="0" value="350000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_payoff_vs_invest.field.rate">Mortgage rate %</span>
                    <input type="number" id="mpi-rate" step="0.05" min="0" max="20" value="6.5" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_payoff_vs_invest.field.term">Remaining term (months)</span>
                    <input type="number" id="mpi-term" step="1" min="12" max="480" value="300" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_payoff_vs_invest.field.extra">Extra $/month</span>
                    <input type="number" id="mpi-extra" step="50" min="0" value="500" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_payoff_vs_invest.field.return">Expected market return %</span>
                    <input type="number" id="mpi-er" step="0.1" min="-10" max="30" value="7" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_payoff_vs_invest.field.tax">Marginal tax bracket %</span>
                    <input type="number" id="mpi-tax" step="1" min="0" max="50" value="22" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_payoff_vs_invest.field.itemize">Itemize mortgage interest?</span>
                    <select id="mpi-item" style="width:100%">
                        <option value="0" selected data-i18n="view.mortgage_payoff_vs_invest.opt.no">No (std deduction)</option>
                        <option value="1" data-i18n="view.mortgage_payoff_vs_invest.opt.yes">Yes</option>
                    </select>
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_payoff_vs_invest.field.horizon">Horizon (years)</span>
                    <input type="number" id="mpi-horizon" step="1" min="1" max="40" value="25" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="mpi-run" data-i18n="view.mortgage_payoff_vs_invest.btn.run">⚡ Compare</button>
            <div id="mpi-result" style="margin-top:12px"></div>
        </div>
    `;
    applyUiI18n(mount);
    mount.querySelectorAll('#mpi-bal, #mpi-rate, #mpi-term, #mpi-extra, #mpi-er, #mpi-tax, #mpi-item, #mpi-horizon').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#mpi-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

async function compute(mount) {
    const result = mount.querySelector('#mpi-result');
    const body = {
        balance_usd: parseFloat(mount.querySelector('#mpi-bal').value) || 0,
        mortgage_rate_pct: parseFloat(mount.querySelector('#mpi-rate').value) || 0,
        term_months: parseInt(mount.querySelector('#mpi-term').value, 10) || 0,
        extra_monthly_usd: parseFloat(mount.querySelector('#mpi-extra').value) || 0,
        expected_return_pct: parseFloat(mount.querySelector('#mpi-er').value) || 0,
        marginal_tax_pct: parseFloat(mount.querySelector('#mpi-tax').value) || 0,
        itemize: mount.querySelector('#mpi-item').value === '1',
        horizon_years: Math.max(1, parseInt(mount.querySelector('#mpi-horizon').value, 10) || 1),
    };
    try {
        const r = await api.calcMortgagePayoffVsInvest(body);
        renderResult(result, r, body);
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

function renderResult(result, r, body) {
    const winnerCls = r.gap_usd > 0 ? 'pos' : r.gap_usd < 0 ? 'neg' : 'muted';
    const winnerLabel = r.gap_usd > 0 ? t('view.mortgage_payoff_vs_invest.winner.invest')
        : r.gap_usd < 0 ? t('view.mortgage_payoff_vs_invest.winner.payoff')
            : t('view.mortgage_payoff_vs_invest.winner.tie');
    const dash = '—';
    const subA = t('view.mortgage_payoff_vs_invest.sub.payoff', {
        month: r.payoff_month_payoff_path == null ? dash : String(r.payoff_month_payoff_path),
        interest: '$' + fmt(r.interest_paid_payoff_usd, 0),
    });
    const subB = t('view.mortgage_payoff_vs_invest.sub.payoff', {
        month: r.payoff_month_invest_path == null ? dash : String(r.payoff_month_invest_path),
        interest: '$' + fmt(r.interest_paid_invest_usd, 0),
    });
    const gapSub = t('view.mortgage_payoff_vs_invest.sub.gap', {
        amount: '$' + fmt(Math.abs(r.gap_usd), 0),
        pct: fmt(Math.abs(r.gap_usd) / Math.max(1, Math.abs(r.wealth_payoff_usd)) * 100, 2) + '%',
    });
    const effSub = body.itemize
        ? t('view.mortgage_payoff_vs_invest.sub.itemize_yes', { tax: fmt(body.marginal_tax_pct, 0) + '%' })
        : t('view.mortgage_payoff_vs_invest.sub.itemize_no');
    const spread = r.effective_rate_pct == null ? 0 : body.expected_return_pct - r.effective_rate_pct;
    const spreadSub = t('view.mortgage_payoff_vs_invest.sub.spread', { pp: fmt(spread, 2) });
    const note = t('view.mortgage_payoff_vs_invest.note', { saved: '$' + fmt(r.interest_saved_usd, 0) });
    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label" data-i18n="view.mortgage_payoff_vs_invest.card.path_a">Path A: Pay extra → mortgage</div><div class="value">$${fmt(r.wealth_payoff_usd, 0)}</div><div class="muted small">${esc(subA)}</div></div>
            <div class="card"><div class="label" data-i18n="view.mortgage_payoff_vs_invest.card.path_b">Path B: Invest extra</div><div class="value">$${fmt(r.wealth_invest_usd, 0)}</div><div class="muted small">${esc(subB)}</div></div>
            <div class="card"><div class="label" data-i18n="view.mortgage_payoff_vs_invest.card.winner">Winner</div><div class="value ${winnerCls}"><strong>${esc(winnerLabel)}</strong></div><div class="muted small">${esc(gapSub)}</div></div>
            <div class="card"><div class="label" data-i18n="view.mortgage_payoff_vs_invest.card.eff_rate">Effective mortgage rate</div><div class="value">${fmt(r.effective_rate_pct, 2)}%</div><div class="muted small">${esc(effSub)}</div></div>
            <div class="card"><div class="label" data-i18n="view.mortgage_payoff_vs_invest.card.vs_return">Vs expected return</div><div class="value ${spread > 0 ? 'pos' : 'neg'}">${fmt(body.expected_return_pct, 2)}%</div><div class="muted small">${esc(spreadSub)}</div></div>
        </div>
        <p class="muted small">${esc(note)}</p>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
