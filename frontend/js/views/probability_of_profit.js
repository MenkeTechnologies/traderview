// Probability of profit (POP) — lognormal probability that an option position
// finishes in its profit zone, via /calc/probability-of-profit.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const pct = (n) => (n == null ? '—' : (Number(n) * 100).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));

export async function renderProbabilityOfProfit(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pop.h1.title">// PROBABILITY OF PROFIT</span></h1>
        <p class="muted small" data-i18n="view.pop.hint.intro">
            The lognormal probability that the underlying finishes in an option position's profit zone at
            expiration. Choose "profit between" for credit spreads and iron condors, or "profit outside" for
            long straddles and strangles. Zero-drift uses the common retail-screen convention (μ = 0).
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.pop.h2.inputs">Position inputs</h2>
            <form id="pop-form" class="inline-form">
                <label><span data-i18n="view.pop.label.spot">Spot ($)</span>
                    <input type="number" step="0.01" min="0" name="spot" value="100" required></label>
                <label><span data-i18n="view.pop.label.iv">IV (decimal)</span>
                    <input type="number" step="0.01" min="0" name="iv" value="0.25" required></label>
                <label><span data-i18n="view.pop.label.t">Time to expiry (years)</span>
                    <input type="number" step="0.01" min="0" name="time_to_expiry_years" value="0.25" required></label>
                <label><span data-i18n="view.pop.label.rf">Risk-free rate (decimal)</span>
                    <input type="number" step="0.005" name="risk_free_rate" value="0.04"></label>
                <label><span data-i18n="view.pop.label.q">Dividend yield (decimal)</span>
                    <input type="number" step="0.005" name="dividend_yield" value="0"></label>
                <label><span data-i18n="view.pop.label.lower">Lower breakeven ($)</span>
                    <input type="number" step="0.01" name="lower_breakeven" value="95"></label>
                <label><span data-i18n="view.pop.label.upper">Upper breakeven ($)</span>
                    <input type="number" step="0.01" name="upper_breakeven" value="105"></label>
                <label><span data-i18n="view.pop.label.between">Profit zone</span>
                    <select name="profit_between">
                        <option value="true" data-i18n="view.pop.opt.between">Between bounds (credit/condor)</option>
                        <option value="false" data-i18n="view.pop.opt.outside">Outside bounds (long straddle)</option>
                    </select></label>
                <label><span data-i18n="view.pop.label.zerodrift">Zero drift (μ=0)</span>
                    <input type="checkbox" name="zero_drift" checked></label>
            </form>
        </div>
        <div id="pop-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#pop-form');
    const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const generate = async () => {
        const lower = form.querySelector('[name="lower_breakeven"]').value;
        const upper = form.querySelector('[name="upper_breakeven"]').value;
        const body = {
            spot: n('spot'), iv: n('iv'), time_to_expiry_years: n('time_to_expiry_years'),
            risk_free_rate: n('risk_free_rate'), dividend_yield: n('dividend_yield'),
            zero_drift: form.querySelector('[name="zero_drift"]').checked,
            lower_breakeven: lower === '' ? null : Number(lower),
            upper_breakeven: upper === '' ? null : Number(upper),
            profit_between: form.querySelector('[name="profit_between"]').value === 'true',
        };
        try {
            const doc = await api.calcProbabilityOfProfit(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.pop.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, doc) {
    const el = mount.querySelector('#pop-result');
    if (!doc) { el.innerHTML = `<p class="muted" data-i18n="view.pop.invalid">Provide at least one breakeven bound.</p>`; applyUiI18n(el); return; }
    el.innerHTML = `
        <div class="lpv-bar"><div class="cards">
            <div class="card pos"><div class="label" data-i18n="view.pop.card.pop">Probability of profit</div>
                <div class="value">${pct(doc.probability_of_profit)}</div></div>
            <div class="card"><div class="label" data-i18n="view.pop.card.below">P(below lower)</div>
                <div class="value">${doc.prob_below_lower == null ? '—' : pct(doc.prob_below_lower)}</div></div>
            <div class="card"><div class="label" data-i18n="view.pop.card.above">P(above upper)</div>
                <div class="value">${doc.prob_above_upper == null ? '—' : pct(doc.prob_above_upper)}</div></div>
            <div class="card"><div class="label" data-i18n="view.pop.card.exp">Expected spot</div>
                <div class="value">${money(doc.expected_spot)}</div></div>
        </div></div>
    `;
    applyUiI18n(el);
}
