// CAPE valuation & CAPE-adjusted SWR, via /calc/cape-valuation.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '%');
export async function renderCapeValuation(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.capev.h1.title">// CAPE VALUATION & SWR</span></h1>
        <p class="muted small" data-i18n="view.capev.hint.intro">Uses the Shiller CAPE (P/E10) to gauge valuation and temper a withdrawal rate. The CAPE earnings yield (1 ÷ CAPE) proxies the expected real return; CAPE vs its historical mean flags over/under-valuation; the CAPE-adjusted SWR (heuristic 1.0% + 0.5 × earnings yield) lowers the starting draw when valuations are stretched. Not financial advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.capev.h2.inputs">Inputs</h2>
        <form id="capev-form" class="inline-form">
            <label><span data-i18n="view.capev.label.cape">Current CAPE</span><input type="number" step="0.5" min="0" name="cape" value="30" required></label>
            <label><span data-i18n="view.capev.label.mean">Historical mean CAPE</span><input type="number" step="0.1" min="0" name="historical_mean_cape" value="16.4"></label>
        </form></div><div id="capev-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#capev-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { cape: n('cape'), historical_mean_cape: n('historical_mean_cape') || 16.4 };
        try { const d = await api.calcCapeValuation(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.capev.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#capev-result');
    if (!d.valid) { el.innerHTML = `<p class="muted" data-i18n="view.capev.invalid">CAPE must be positive.</p>`; applyUiI18n(el); return; }
    const valKey = { overvalued: 'view.capev.over', fair: 'view.capev.fair', undervalued: 'view.capev.under' }[d.valuation] || 'view.capev.fair';
    const cls = d.valuation === 'overvalued' ? 'neg' : d.valuation === 'undervalued' ? 'pos' : '';
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card ${cls}"><div class="label" data-i18n="view.capev.card.val">Valuation</div><div class="value" data-i18n="${valKey}">${d.valuation}</div></div>
        <div class="card pos"><div class="label" data-i18n="view.capev.card.swr">CAPE-adjusted SWR</div><div class="value">${pct(d.cape_adjusted_swr_pct)}</div></div>
        <div class="card"><div class="label" data-i18n="view.capev.card.caey">Earnings yield</div><div class="value">${pct(d.cape_earnings_yield_pct)}</div></div>
        <div class="card"><div class="label" data-i18n="view.capev.card.ratio">CAPE / mean</div><div class="value">${num(d.valuation_ratio)}×</div></div>
    </div></div>`;
    applyUiI18n(el);
}
