// Break-even ROAS — ad profitability, via /calc/break-even-roas.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const x = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '×');
export async function renderBreakEvenRoas(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.roas.h1.title">// BREAK-EVEN ROAS</span></h1>
        <p class="muted small" data-i18n="view.roas.hint.intro">For a marketing campaign, the return on ad spend (revenue ÷ ad spend) versus the break-even ROAS at which the gross profit on the revenue exactly covers the ad spend (1 ÷ gross-margin ratio). A campaign is profitable when its ROAS exceeds break-even. Not advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.roas.h2.inputs">Campaign</h2>
        <form id="roas-form" class="inline-form">
            <label><span data-i18n="view.roas.label.label">Campaign</span><input type="text" name="campaign_label" value="Spring sale"></label>
            <label><span data-i18n="view.roas.label.rev">Revenue ($)</span><input type="number" step="100" min="0" name="revenue_usd" value="10000" required></label>
            <label><span data-i18n="view.roas.label.ad">Ad spend ($)</span><input type="number" step="50" min="0" name="ad_spend_usd" value="2500" required></label>
            <label><span data-i18n="view.roas.label.margin">Gross margin (%)</span><input type="number" step="1" min="0" max="100" name="gross_margin_pct" value="40" required></label>
        </form></div><div id="roas-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#roas-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { campaign_label: (form.querySelector('[name="campaign_label"]').value || '').trim(), revenue_usd: n('revenue_usd'), ad_spend_usd: n('ad_spend_usd'), gross_margin_pct: n('gross_margin_pct') };
        try { const d = await api.calcBreakEvenRoas(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.roas.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#roas-result');
    if (!d.valid) { el.innerHTML = `<p class="muted" data-i18n="view.roas.invalid">Ad spend and gross margin must be positive.</p>`; applyUiI18n(el); return; }
    const statusKey = d.profitable ? 'view.roas.profitable' : 'view.roas.unprofitable';
    const cls = d.profitable ? 'pos' : 'neg';
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card ${cls}"><div class="label" data-i18n="view.roas.card.status">Status</div><div class="value" data-i18n="${statusKey}">${d.profitable ? 'Profitable' : 'Unprofitable'}</div></div>
        <div class="card"><div class="label" data-i18n="view.roas.card.roas">ROAS</div><div class="value">${x(d.roas)}</div></div>
        <div class="card"><div class="label" data-i18n="view.roas.card.be">Break-even ROAS</div><div class="value">${x(d.break_even_roas)}</div></div>
        <div class="card ${cls}"><div class="label" data-i18n="view.roas.card.contrib">Contribution after ads</div><div class="value">${money(d.contribution_after_ads_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.roas.card.ppad">Profit / ad \$</div><div class="value">${money(d.profit_per_ad_dollar)}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
