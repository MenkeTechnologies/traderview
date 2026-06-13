// Age-based allocation — the rule-of-N equity glidepath (equity % = N − age),
// dollar split, and a 10-year-step glidepath, via /calc/age-allocation.
// Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['age', 'Age', 35],
    ['rule_n', 'Rule N (100/110/120; 0 = 110)', 110],
    ['portfolio_value_usd', 'Portfolio value ($)', 250000],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const pct = (n) => Number(n).toFixed(0) + '%';

export async function renderAgeAllocation(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.aba.h1.title">// AGE-BASED ALLOCATION</span></h1>
        <p class="muted small" data-i18n="view.aba.hint.intro">
            A simple heuristic for how much to hold in stocks: equity % = N − age. The classic
            N is 100; longer lifespans pushed it to 110 or 120 for more growth. Bonds take the
            rest, and the equity share falls as you age — a built-in glidepath toward capital
            preservation. A rule of thumb, not advice. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.aba.h2.inputs">You</h2>
            <form id="aba-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.aba.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="aba-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#aba-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcAgeAllocation(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.aba.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#aba-result');
    const glide = r.glidepath.map((g) => `
        <tr><td>${g.age}</td><td>${pct(g.equity_pct)}</td><td>${pct(g.bond_pct)}</td></tr>
    `).join('');
    el.innerHTML = `
        <div class="chart-panel">
            <h2>${t('view.aba.h2.result', { n: r.rule_n })}</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.aba.card.equity">Equity</div>
                    <div class="value pos">${pct(r.equity_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.aba.card.bond">Bonds</div>
                    <div class="value">${pct(r.bond_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.aba.card.equity_usd">Equity $</div>
                    <div class="value">${money(r.equity_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.aba.card.bond_usd">Bonds $</div>
                    <div class="value">${money(r.bond_usd)}</div></div>
            </div>
            <table class="data-table">
                <thead><tr>
                    <th data-i18n="view.aba.col.age">Age</th>
                    <th data-i18n="view.aba.col.equity">Equity</th>
                    <th data-i18n="view.aba.col.bond">Bonds</th>
                </tr></thead>
                <tbody>${glide}</tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
