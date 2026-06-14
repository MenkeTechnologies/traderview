// IV cone — expected 1σ/2σ move bands across the IV term structure, via /calc/iv-cone.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
export async function renderIvCone(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ivcone.h1.title">// IV CONE</span></h1>
        <p class="muted small" data-i18n="view.ivcone.hint.intro">Projects the expected 1σ and 2σ price bands at each expiry from the implied-volatility term structure. Enter the term as "days:IV%" pairs (e.g. 30:18, 60:20, 90:22).</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.ivcone.h2.inputs">Inputs</h2>
        <form id="ivcone-form" class="inline-form">
            <label><span data-i18n="view.ivcone.label.spot">Spot ($)</span><input type="number" step="0.01" min="0" name="spot" value="100" required></label>
            <label class="full"><span data-i18n="view.ivcone.label.term">Term (days:IV%)</span><textarea name="term" rows="3">7:25, 30:22, 60:21, 90:20, 180:19</textarea></label>
        </form></div><div id="ivcone-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#ivcone-form');
    const gen = async () => {
        const term = (form.querySelector('[name="term"]').value || '').split(/[,\n]+/).map((p) => p.split(':').map((x) => Number(x.trim()))).filter((a) => a.length === 2 && a[0] > 0 && a[1] > 0).map(([days, iv_pct]) => ({ days, iv_pct }));
        const body = { spot: Number(form.querySelector('[name="spot"]').value) || 0, term };
        if (!term.length) { mount.querySelector('#ivcone-result').innerHTML = ''; return; }
        try { const d = await api.calcIvCone(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.ivcone.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, rows) {
    const el = mount.querySelector('#ivcone-result');
    if (!rows || !rows.length) { el.innerHTML = `<p class="muted" data-i18n="view.ivcone.invalid">Enter a valid term structure.</p>`; applyUiI18n(el); return; }
    const body = rows.map((r) => `<tr><td>${r.days}</td><td>${pct(r.iv_pct)}</td><td>±${pct(r.move_1s_pct)}</td><td>${money(r.low_1s)} – ${money(r.high_1s)}</td><td>${money(r.low_2s)} – ${money(r.high_2s)}</td></tr>`).join('');
    el.innerHTML = `<table class="data-table"><thead><tr><th data-i18n="view.ivcone.th.days">Days</th><th data-i18n="view.ivcone.th.iv">IV</th><th data-i18n="view.ivcone.th.move">1σ move</th><th data-i18n="view.ivcone.th.b1">1σ band</th><th data-i18n="view.ivcone.th.b2">2σ band</th></tr></thead><tbody>${body}</tbody></table>`;
    applyUiI18n(el);
}
