// IV smile surface — 2D implied-vol grid across moneyness and expiry, via /calc/iv-surface.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const SEED = [
    { label: '30d', years: 0.0822, atm: 20 },
    { label: '60d', years: 0.1644, atm: 21 },
    { label: '90d', years: 0.2466, atm: 22 },
    { label: '180d', years: 0.4932, atm: 23 },
];
function rowHtml(e) {
    return `<div class="mpb-row ivs-row">
        <input type="text" class="ivs-label" value="${esc(e.label)}">
        <input type="number" step="0.01" min="0" class="ivs-years" value="${e.years}">
        <input type="number" step="0.5" min="0" class="ivs-atm" value="${e.atm}">
        <button type="button" class="ivs-del" data-i18n="view.ivs.remove">Remove</button></div>`;
}
function heat(iv, lo, hi) {
    const f = hi > lo ? (iv - lo) / (hi - lo) : 0.5;
    const r = Math.round(20 + f * 200), b = Math.round(220 - f * 180);
    return `background:rgb(${r},${Math.round(60 + (1 - Math.abs(f - 0.5) * 2) * 80)},${b});`;
}
export async function renderIvSurface(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ivs.h1.title">// IV SMILE SURFACE</span></h1>
        <p class="muted small" data-i18n="view.ivs.hint.intro">A 2D implied-volatility grid across moneyness (strike ÷ forward) and expiry. Each expiry's ATM IV sets the term structure; the smile within an expiry is IV = atm × (1 + skew·k + curvature·k²) with k = ln(moneyness). Negative skew makes downside puts richer (the equity smirk).</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.ivs.h2.inputs">Term structure & smile</h2>
        <form id="ivs-form" class="inline-form">
            <label><span data-i18n="view.ivs.label.moneyness">Moneyness levels (K/F)</span><input type="text" name="moneyness" value="0.8, 0.9, 1.0, 1.1, 1.2"></label>
            <label><span data-i18n="view.ivs.label.skew">Skew</span><input type="number" step="0.01" name="skew" value="-0.1"></label>
            <label><span data-i18n="view.ivs.label.curv">Curvature</span><input type="number" step="0.1" name="curvature" value="0.5"></label>
        </form>
        <div class="mpb-head ivs-head"><span data-i18n="view.ivs.col.label">Expiry</span><span data-i18n="view.ivs.col.years">Years</span><span data-i18n="view.ivs.col.atm">ATM IV (%)</span><span></span></div>
        <div id="ivs-rows">${SEED.map(rowHtml).join('')}</div>
        <button type="button" id="ivs-add" class="secondary" data-i18n="view.ivs.add">+ Add expiry</button>
        </div><div id="ivs-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#ivs-form'); const rowsEl = mount.querySelector('#ivs-rows');
    const gen = async () => {
        const expiries = [...rowsEl.querySelectorAll('.ivs-row')].map((r) => ({ label: (r.querySelector('.ivs-label').value || '').trim(), years: Number(r.querySelector('.ivs-years').value) || 0, atm_iv_pct: Number(r.querySelector('.ivs-atm').value) || 0 })).filter((e) => e.label);
        const moneyness = (form.querySelector('[name="moneyness"]').value || '').split(/[\s,]+/).map(Number).filter((x) => x > 0);
        const body = { expiries, moneyness_levels: moneyness, skew: Number(form.querySelector('[name="skew"]').value) || 0, curvature: Number(form.querySelector('[name="curvature"]').value) || 0 };
        if (!expiries.length || !moneyness.length) { mount.querySelector('#ivs-result').innerHTML = ''; return; }
        try { const d = await api.calcIvSurface(body); if (!viewIsCurrent(tok)) return; res(mount, d, moneyness); }
        catch (e) { showToast(e.message || t('view.ivs.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250);
    mount.querySelector('#ivs-add').addEventListener('click', () => { rowsEl.insertAdjacentHTML('beforeend', rowHtml({ label: '', years: 0.25, atm: 22 })); applyUiI18n(rowsEl.lastElementChild); gen(); });
    rowsEl.addEventListener('click', (e) => { if (e.target.classList.contains('ivs-del')) { e.target.closest('.ivs-row').remove(); gen(); } });
    form.addEventListener('input', () => live()); rowsEl.addEventListener('input', () => live()); gen();
}
function res(mount, d, moneyness) {
    const el = mount.querySelector('#ivs-result');
    if (!d.ok) { el.innerHTML = `<p class="muted">No surface.</p>`; return; }
    const head = `<tr><th data-i18n="view.ivs.th.exp">Expiry</th>${moneyness.map((m) => `<th>${m}</th>`).join('')}</tr>`;
    const body = d.rows.map((r) => `<tr><td>${esc(r.label)}</td>${r.cells.map((c) => `<td style="${heat(c.iv_pct, d.min_iv_pct, d.max_iv_pct)}">${c.iv_pct}</td>`).join('')}</tr>`).join('');
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card"><div class="label" data-i18n="view.ivs.card.min">Min IV</div><div class="value">${d.min_iv_pct}%</div></div>
        <div class="card"><div class="label" data-i18n="view.ivs.card.max">Max IV</div><div class="value">${d.max_iv_pct}%</div></div>
    </div></div>
    <table class="data-table ivs-surface"><thead>${head}</thead><tbody>${body}</tbody></table>`;
    applyUiI18n(el);
}
