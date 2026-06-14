// SPAN-style portfolio margin — worst-case loss across 16 risk scenarios, via /calc/span-margin.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
export async function renderSpanMargin(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.span.h1.title">// SPAN MARGIN</span></h1>
        <p class="muted small" data-i18n="view.span.hint.intro">A scenario-based portfolio-margin estimate modeled on CME SPAN. It revalues the portfolio under 16 risk scenarios — seven price moves each with an up/down vol shift (14), plus two extreme moves at a 35% loss fraction — and takes the worst-case loss as the scan risk. Portfolio P&L = delta·dP + ½·gamma·dP² + vega·dV. Faithful simplification, not the full exchange algorithm.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.span.h2.inputs">Portfolio risk</h2>
        <form id="span-form" class="inline-form">
            <label><span data-i18n="view.span.label.price">Underlying price ($)</span><input type="number" step="0.01" min="0" name="underlying_price" value="100" required></label>
            <label><span data-i18n="view.span.label.scan">Price scan range (%)</span><input type="number" step="1" min="0" name="price_scan_pct" value="15" required></label>
            <label><span data-i18n="view.span.label.vol">Vol scan (points)</span><input type="number" step="0.5" min="0" name="vol_scan_points" value="5"></label>
            <label><span data-i18n="view.span.label.delta">Portfolio delta</span><input type="number" step="1" name="portfolio_delta" value="10"></label>
            <label><span data-i18n="view.span.label.gamma">Portfolio gamma</span><input type="number" step="0.5" name="portfolio_gamma" value="-2"></label>
            <label><span data-i18n="view.span.label.vega">Portfolio vega</span><input type="number" step="1" name="portfolio_vega" value="50"></label>
        </form></div><div id="span-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#span-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { underlying_price: n('underlying_price'), price_scan_pct: n('price_scan_pct'), vol_scan_points: n('vol_scan_points'), portfolio_delta: n('portfolio_delta'), portfolio_gamma: n('portfolio_gamma'), portfolio_vega: n('portfolio_vega') };
        try { const d = await api.calcSpanMargin(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.span.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#span-result');
    const rows = d.scenarios.map((s) => `<tr class="${s.index === d.worst_scenario ? 'span-worst' : ''}"><td>${s.index}${s.extreme ? '*' : ''}</td><td>${money(s.price_move)}</td><td>${s.vol_move}</td><td>${money(s.pnl)}</td><td>${money(s.loss)}</td></tr>`).join('');
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card neg"><div class="label" data-i18n="view.span.card.margin">SPAN margin</div><div class="value">${money(d.margin_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.span.card.worst">Worst scenario</div><div class="value">#${d.worst_scenario}</div></div>
    </div></div>
    <table class="data-table"><thead><tr><th data-i18n="view.span.th.idx">#</th><th data-i18n="view.span.th.price">Price move</th><th data-i18n="view.span.th.vol">Vol</th><th data-i18n="view.span.th.pnl">P&L</th><th data-i18n="view.span.th.loss">Loss</th></tr></thead><tbody>${rows}</tbody></table>
    <p class="muted small" data-i18n="view.span.note">* = extreme scenario (35% loss fraction)</p>`;
    applyUiI18n(el);
}
