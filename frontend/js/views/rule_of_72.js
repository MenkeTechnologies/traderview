// Rule of 72 / 114 / 144 — back-of-envelope compound growth estimators.
//   72 / r% ≈ years to DOUBLE
//   114 / r% ≈ years to TRIPLE
//   144 / r% ≈ years to QUADRUPLE
// All three are mental-math shortcuts to log_2 / log_3 / log_4 of (1+r).
// Accuracy: best in the 6-12%/yr range; degrades outside it. Compared
// against the exact closed-form solution below for honest error reporting.

import { esc } from '../util.js';

export async function renderRuleOf72(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rule_of_72.title">// RULE OF 72 / 114 / 144</span></h1>
        <p class="muted small" data-i18n-html="view.rule_of_72.intro">
            Mental-math shortcuts: <strong>72 ÷ rate</strong> ≈ years to double,
            <strong>114 ÷ rate</strong> ≈ years to triple, <strong>144 ÷ rate</strong> ≈
            years to quadruple. Most accurate at 6-12%/yr; outside that range the
            rule-of-N drifts. Exact closed-form: <code>n = ln(k) / ln(1+r)</code> for
            multiplier <code>k</code> and rate <code>r</code>. Both shown side by side
            so you can see when the shortcut breaks down.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Annual rate %</span>
                    <input type="number" id="r72-rate" step="0.25" min="-20" max="100" value="7" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Compounding</span>
                    <select id="r72-compound" style="width:100%">
                        <option value="annual" selected>Annual</option>
                        <option value="monthly">Monthly</option>
                        <option value="daily">Daily</option>
                        <option value="continuous">Continuous</option>
                    </select>
                </label>
                <label>
                    <span class="muted small">Starting amount $</span>
                    <input type="number" id="r72-amount" step="100" min="0" value="10000" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="r72-run">⚡ Compute</button>
            <div id="r72-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#r72-rate, #r72-compound, #r72-amount').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#r72-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const r = parseFloat(mount.querySelector('#r72-rate').value) / 100;
    const compound = mount.querySelector('#r72-compound').value;
    const amount = parseFloat(mount.querySelector('#r72-amount').value) || 0;
    const result = mount.querySelector('#r72-result');
    if (r === 0) {
        result.innerHTML = `<p class="muted">Rate must be non-zero.</p>`;
        return;
    }
    // Compute effective annual rate per compounding.
    let r_eff;
    const r_pct = r * 100;
    if (compound === 'monthly')     r_eff = Math.pow(1 + r/12, 12) - 1;
    else if (compound === 'daily')  r_eff = Math.pow(1 + r/365, 365) - 1;
    else if (compound === 'continuous') r_eff = Math.exp(r) - 1;
    else                            r_eff = r;

    const ln1pr = Math.log(1 + r_eff);

    const rows = [
        { k: 2, label: 'Double (×2)',     shortcutN: 72,  rule: '72 / r' },
        { k: 3, label: 'Triple (×3)',     shortcutN: 114, rule: '114 / r' },
        { k: 4, label: 'Quadruple (×4)',  shortcutN: 144, rule: '144 / r' },
        { k: 5, label: 'Quintuple (×5)',  shortcutN: 167, rule: '167 / r' },
        { k: 10,label: '10× growth',      shortcutN: 240, rule: '240 / r' },
    ];

    result.innerHTML = `
        <p class="muted small">
            Effective annual rate: <strong>${fmt(r_eff * 100, 4)}%</strong>
            (${esc(compound)} compounding of ${fmt(r_pct, 2)}% nominal).
        </p>
        <table class="trades" data-table-key="r72-rows">
            <thead><tr>
                <th>Goal</th>
                <th>Shortcut</th>
                <th>Years (shortcut)</th>
                <th>Years (exact)</th>
                <th>Error</th>
                <th>$${fmt(amount, 0)} becomes</th>
            </tr></thead>
            <tbody>${rows.map(row => {
                const shortcut = row.shortcutN / r_pct;
                const exact = Math.log(row.k) / ln1pr;
                const err = ((shortcut - exact) / exact) * 100;
                const errCls = Math.abs(err) < 2 ? 'pos' : Math.abs(err) < 5 ? '' : 'neg';
                return `<tr>
                    <td><strong>${esc(row.label)}</strong></td>
                    <td class="muted small">${esc(row.rule)}</td>
                    <td>${fmt(shortcut, 2)}</td>
                    <td><strong>${fmt(exact, 2)}</strong></td>
                    <td class="${errCls}">${err >= 0 ? '+' : ''}${fmt(err, 2)}%</td>
                    <td class="muted">$${fmt(amount * row.k, 0)} in yr ${fmt(exact, 1)}</td>
                </tr>`;
            }).join('')}</tbody>
        </table>
        <p class="muted small" style="margin-top:8px">
            <strong>Half-life of debt:</strong> Rule of 72 also works in reverse.
            At ${fmt(r_pct, 1)}%/yr inflation, money loses half its purchasing
            power in <strong>${fmt(72 / Math.max(0.01, r_pct), 1)}</strong> years.
        </p>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
