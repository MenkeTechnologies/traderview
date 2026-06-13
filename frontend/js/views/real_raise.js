// Real raise — whether a pay raise beats inflation, via /calc/real-raise. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '%';

export async function renderRealRaise(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.realraise.h1.title">// REAL RAISE</span></h1>
        <p class="muted small" data-i18n="view.realraise.hint.intro">
            Whether a pay raise actually beats inflation. A nominal raise only buys more if it
            outpaces inflation — a raise equal to inflation is a wash, and below it is a real pay
            cut. Shows the inflation-adjusted break-even salary and the real (purchasing-power)
            change. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.realraise.h2.inputs">The offer</h2>
            <form id="realraise-form" class="inline-form">
                <label><span data-i18n="view.realraise.label.salary">Current salary ($)</span>
                    <input type="number" step="0.01" min="0" name="current_salary_usd" value="80000" required></label>
                <label><span data-i18n="view.realraise.label.raise">Raise (%)</span>
                    <input type="number" step="0.1" name="raise_pct" value="5" required></label>
                <label><span data-i18n="view.realraise.label.inflation">Inflation (%)</span>
                    <input type="number" step="0.1" name="inflation_pct" value="3" required></label>
            </form>
        </div>
        <div id="realraise-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#realraise-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            current_salary_usd: Number(fd.get('current_salary_usd')) || 0,
            raise_pct: Number(fd.get('raise_pct')) || 0,
            inflation_pct: Number(fd.get('inflation_pct')) || 0,
        };
        try {
            const r = await api.calcRealRaise(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.realraise.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#realraise-result');
    const cls = r.is_real_raise ? 'pos' : 'neg';
    const verdictKey = r.is_real_raise ? 'view.realraise.verdict.real' : 'view.realraise.verdict.cut';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.realraise.h2.result">After inflation</h2>
            <div class="cards">
                <div class="card ${cls}"><div class="label" data-i18n="view.realraise.card.real">Real raise</div>
                    <div class="value ${cls}">${pct(r.real_raise_pct)}</div></div>
                <div class="card ${cls}"><div class="label" data-i18n="view.realraise.card.change">Real change</div>
                    <div class="value ${cls}">${money(r.real_change_usd)}</div></div>
                <div class="card ${cls}"><div class="label" data-i18n="view.realraise.card.verdict">Verdict</div>
                    <div class="value ${cls}" data-i18n="${verdictKey}">—</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.realraise.row.new">New salary</td><td>${money(r.new_salary_usd)}</td></tr>
                    <tr><td data-i18n="view.realraise.row.nominal">Nominal change</td><td>${money(r.nominal_change_usd)}</td></tr>
                    <tr><td data-i18n="view.realraise.row.breakeven">Break-even salary (keeps pace)</td><td>${money(r.inflation_adjusted_salary_usd)}</td></tr>
                    <tr><td data-i18n="view.realraise.row.keeppace">Raise to keep pace</td><td>${pct(r.raise_to_keep_pace_pct)}</td></tr>
                    <tr class="emph ${cls}"><td data-i18n="view.realraise.row.real">Real raise</td><td>${pct(r.real_raise_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
