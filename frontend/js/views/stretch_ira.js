// Stretch IRA — SECURE Act 1.0 (2019) killed the lifetime "stretch"
// for most non-spouse inherited IRAs. Now the 10-year rule applies:
// the entire account must be emptied within 10 years of the original
// owner's death. SECURE Act 2.0 (2022) added Eligible Designated
// Beneficiaries who can still stretch (surviving spouse, minor child,
// disabled, chronically ill, person <10 years younger than decedent).
//
// This tool models: even-distribution, back-loaded, RMD-style, and
// year-10-everything strategies, with year-by-year tax cost projection
// based on user's expected ordinary-income bracket.

import { api } from '../api.js';
import { esc } from '../util.js';
import { applyUiI18n, t } from '../i18n.js';

export async function renderStretchIra(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.stretch_ira.title">// STRETCH IRA · SECURE 10-YEAR RULE</span></h1>
        <p class="muted small" data-i18n-html="view.stretch_ira.intro">
            SECURE Act 1.0 (effective 2020) killed the lifetime stretch for most
            non-spouse inherited IRAs. The <strong>entire inherited account
            must be distributed within 10 years</strong> of the original owner's
            death. RMDs required years 1-9 if the decedent had started RMDs;
            year 10 must bring the balance to zero. Comparing 4 distribution
            strategies for total tax cost.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.stretch_ira.field.balance">Inherited IRA balance $</span>
                    <input type="number" id="si-balance" step="10000" min="0" value="500000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.stretch_ira.field.growth">Expected growth %/yr</span>
                    <input type="number" id="si-growth" step="0.5" min="-10" max="20" value="6" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.stretch_ira.field.other">Your other ordinary income $</span>
                    <input type="number" id="si-other" step="1000" min="0" value="120000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.stretch_ira.field.status">Filing status</span>
                    <select id="si-status" style="width:100%">
                        <option value="single" data-i18n="view.stretch_ira.opt.single">Single</option>
                        <option value="mfj" selected data-i18n="view.stretch_ira.opt.mfj">MFJ</option>
                    </select>
                </label>
            </div>
            <button class="btn btn-sm primary" id="si-run" data-i18n="view.stretch_ira.btn.run">⚡ Compare strategies</button>
            <div id="si-result" style="margin-top:12px"></div>
        </div>
    `;
    applyUiI18n(mount);
    mount.querySelectorAll('#si-balance, #si-growth, #si-other, #si-status').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#si-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

async function compute(mount) {
    const result = mount.querySelector('#si-result');
    const body = {
        balance_usd: parseFloat(mount.querySelector('#si-balance').value) || 0,
        growth_pct: parseFloat(mount.querySelector('#si-growth').value) || 0,
        other_income_usd: parseFloat(mount.querySelector('#si-other').value) || 0,
        filing_status: mount.querySelector('#si-status').value,
    };
    let r;
    try {
        r = await api.calcStretchIra(body);
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
        return;
    }
    const runs = r.strategies;
    const name = (key) => t('view.stretch_ira.strat.' + key);
    const winnerKey = r.winner_key;
    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:8px;margin-bottom:12px">
            ${runs.map(run => {
                const isWinner = run.key === winnerKey;
                const sub = esc(t('view.stretch_ira.card.sub', { tax: '$' + fmt(run.total_tax_usd, 0) }))
                    + (isWinner ? ' · <strong>' + esc(t('view.stretch_ira.card.winner')) + '</strong>' : '');
                return `<div class="card">
                <div class="label">${esc(name(run.key))}</div>
                <div class="value ${isWinner ? 'pos' : ''}">$${fmt(run.total_received_usd, 0)}</div>
                <div class="muted small">${sub}</div>
            </div>`;
            }).join('')}
        </div>
        <h3 class="section-title" data-i18n="view.stretch_ira.h3.yearly">Year-by-year distributions</h3>
        <table class="trades" data-table-key="si-rows">
            <thead><tr>
                <th data-i18n="view.stretch_ira.th.year">Year</th>
                ${runs.map(run => `<th>${esc(name(run.key))}<br><span class="muted small">${esc(t('view.stretch_ira.th.aftertax'))}</span></th>`).join('')}
            </tr></thead>
            <tbody>${[0,1,2,3,4,5,6,7,8,9].map(y => `<tr>
                <td>${y + 1}</td>
                ${runs.map(run => {
                    const cell = run.rows[y];
                    const c = esc(t('view.stretch_ira.cell', { dist: '$' + fmt(cell.distribution_usd, 0), tax: '$' + fmt(cell.tax_on_dist_usd, 0) }));
                    return `<td><span class="muted small">${c}</span><br><strong>$${fmt(cell.after_tax_usd, 0)}</strong></td>`;
                }).join('')}
            </tr>`).join('')}</tbody>
        </table>
        <p class="muted small" style="margin-top:8px" data-i18n-html="view.stretch_ira.note"></p>
    `;
    applyUiI18n(result);
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
