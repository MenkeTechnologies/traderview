// Rule of 40 — revenue growth + profit margin vs the 40% bar, with the
// growth/margin you'd need to clear it, via /calc/rule-of-40. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%');

export async function renderRuleOf40(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ruleof40.h1.title">// RULE OF 40</span></h1>
        <p class="muted small" data-i18n="view.ruleof40.hint.intro">
            The SaaS/growth-company health check: revenue growth rate plus profit margin should sum
            to at least 40%. It trades growth against profitability — a company can pass by growing
            fast at a loss, or growing slowly with fat margins. The margin can be FCF, EBITDA, or
            net — whatever you pair with growth. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ruleof40.h2.inputs">The numbers</h2>
            <form id="ruleof40-form" class="inline-form">
                <label><span data-i18n="view.ruleof40.label.growth">Revenue growth (%)</span>
                    <input type="number" step="0.1" name="revenue_growth_pct" value="30" required></label>
                <label><span data-i18n="view.ruleof40.label.margin">Profit margin (%)</span>
                    <input type="number" step="0.1" name="profit_margin_pct" value="15" required></label>
                <label><span data-i18n="view.ruleof40.label.target">Target (%)</span>
                    <input type="number" step="1" name="target_pct" value="40"></label>
            </form>
        </div>
        <div id="ruleof40-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ruleof40-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            revenue_growth_pct: Number(fd.get('revenue_growth_pct')) || 0,
            profit_margin_pct: Number(fd.get('profit_margin_pct')) || 0,
            target_pct: Number(fd.get('target_pct')) || 40,
        };
        try {
            const r = await api.calcRuleOf40(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.ruleof40.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#ruleof40-result');
    const cls = r.passes ? 'pos' : 'neg';
    const verdictKey = r.passes ? 'view.ruleof40.verdict.pass' : 'view.ruleof40.verdict.fail';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ruleof40.h2.result">The verdict</h2>
            <div class="cards">
                <div class="card ${cls}"><div class="label" data-i18n="view.ruleof40.card.score">Score</div>
                    <div class="value ${cls}">${pct(r.score_pct)}</div></div>
                <div class="card ${cls}"><div class="label" data-i18n="view.ruleof40.card.verdict">Verdict</div>
                    <div class="value ${cls}" data-i18n="${verdictKey}">—</div></div>
                <div class="card"><div class="label" data-i18n="view.ruleof40.card.surplus">Surplus / gap</div>
                    <div class="value">${pct(r.surplus_pct)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.ruleof40.row.marginneeded">Margin needed at this growth</td><td>${pct(r.margin_needed_pct)}</td></tr>
                    <tr><td data-i18n="view.ruleof40.row.growthneeded">Growth needed at this margin</td><td>${pct(r.growth_needed_pct)}</td></tr>
                    <tr><td data-i18n="view.ruleof40.row.growthshare">Growth share of score</td><td>${pct(r.growth_share_pct)}</td></tr>
                    <tr><td data-i18n="view.ruleof40.row.marginshare">Margin share of score</td><td>${pct(r.margin_share_pct)}</td></tr>
                    <tr class="emph"><td data-i18n="view.ruleof40.row.score">Score</td><td>${pct(r.score_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
