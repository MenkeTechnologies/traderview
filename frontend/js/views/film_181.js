// IRC § 181 — 100% expensing for qualified film, TV, and live theatrical productions.
// Originally TCJA-limited but restored + extended through 2025 (and now indefinite
// per CAA-21 + IRA extension). Cap: $15M per production ($20M low-income area).
// 75% of compensation must be paid in the US. Owners deduct in year placed.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-film-181-v1';
const PROD_CAP_DEFAULT = 15_000_000;
const PROD_CAP_LOW_INCOME = 20_000_000;
const US_COMP_THRESHOLD = 0.75;

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    productions: load(),
    marginal_rate: 0.37,
};

export async function renderFilm181(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.f181.h1.title">// § 181 FILM / TV / THEATER EXPENSING</span></h1>
        <p class="muted small" data-i18n="view.f181.hint.intro">
            100% year-1 deduction for qualified film, TV, and live theatrical productions —
            up to <strong>$15M per production</strong> ($20M low-income area). Must be a
            qualified production: <strong>75% of compensation paid in the US</strong>. Restored
            by TCJA, extended indefinitely by CAA-21. Owners deduct in year placed in service.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.f181.h2.qualification">Qualification requirements</h2>
            <ul class="muted small">
                <li data-i18n="view.f181.qual.production_type">Qualifying types: motion picture, TV series, live theatrical</li>
                <li data-i18n="view.f181.qual.us_compensation">≥ 75% of total compensation paid for US-located services</li>
                <li data-i18n="view.f181.qual.first_44">For TV: only first 44 episodes eligible</li>
                <li data-i18n="view.f181.qual.cap_15m">$15M cap per production ($20M in low-income census tract)</li>
                <li data-i18n="view.f181.qual.election">Make election on year-1 return; passive activity rules apply</li>
                <li data-i18n="view.f181.qual.no_porn">Sexually explicit content disqualifies</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.f181.h2.add">Log production</h2>
            <form id="f181-form" class="inline-form">
                <label><span data-i18n="view.f181.label.title">Title</span>
                    <input type="text" name="title" required></label>
                <label><span data-i18n="view.f181.label.type">Type</span>
                    <select name="ptype">
                        <option value="film">Motion picture</option>
                        <option value="tv">TV series</option>
                        <option value="theater">Live theatrical</option>
                    </select>
                </label>
                <label><span data-i18n="view.f181.label.placed_year">Year placed in service</span>
                    <input type="number" step="1" name="placed_year" value="${new Date().getFullYear()}" required></label>
                <label><span data-i18n="view.f181.label.cost">Total production cost ($)</span>
                    <input type="number" step="0.01" name="cost" required></label>
                <label><span data-i18n="view.f181.label.us_comp_pct">% US compensation</span>
                    <input type="number" step="0.01" name="us_comp_pct" value="0.85"></label>
                <label><span data-i18n="view.f181.label.your_share">Your ownership %</span>
                    <input type="number" step="0.01" name="ownership_pct" value="1.00"></label>
                <label><span data-i18n="view.f181.label.is_low_income">Low-income census tract?</span>
                    <input type="checkbox" name="is_low_income"></label>
                <button class="primary" type="submit" data-i18n="view.f181.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.f181.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" id="f181-marginal" value="${state.marginal_rate}"></label>
            </div>
        </div>
        <div id="f181-summary"></div>
        <div id="f181-table" class="chart-panel"></div>
    `;
    document.getElementById('f181-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const p = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            title: fd.get('title'),
            ptype: fd.get('ptype'),
            placed_year: Number(fd.get('placed_year')),
            cost: Number(fd.get('cost')) || 0,
            us_comp_pct: Number(fd.get('us_comp_pct')) || 0,
            ownership_pct: Number(fd.get('ownership_pct')) || 1,
            is_low_income: !!fd.get('is_low_income'),
        };
        state.productions.push(p);
        save(state.productions);
        e.target.reset();
        showToast(t('view.f181.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('f181-marginal').addEventListener('change', e => {
        state.marginal_rate = Number(e.target.value) || 0.37;
        render();
    });
    render();
}

function evaluateProduction(p) {
    const qualifiesUs = p.us_comp_pct >= US_COMP_THRESHOLD;
    const cap = p.is_low_income ? PROD_CAP_LOW_INCOME : PROD_CAP_DEFAULT;
    const eligibleCost = qualifiesUs ? Math.min(p.cost, cap) : 0;
    const yourDeduction = eligibleCost * p.ownership_pct;
    return { qualifiesUs, cap, eligibleCost, yourDeduction };
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('f181-summary');
    if (!el) return;
    const summary = state.productions.map(p => ({ p, ev: evaluateProduction(p) }));
    const totalDeduction = summary.reduce((s, x) => s + x.ev.yourDeduction, 0);
    const totalSavings = totalDeduction * state.marginal_rate;
    const eligibleCount = summary.filter(x => x.ev.qualifiesUs).length;
    const disqualifiedCount = summary.length - eligibleCount;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.f181.h2.summary">Portfolio summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.f181.card.productions">Productions</div>
                    <div class="value">${state.productions.length}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.f181.card.eligible">Eligible</div>
                    <div class="value">${eligibleCount}</div>
                </div>
                <div class="card ${disqualifiedCount > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.f181.card.disqualified">Disqualified (US comp &lt; 75%)</div>
                    <div class="value">${disqualifiedCount}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.f181.card.total_deduction">Total § 181 deduction</div>
                    <div class="value">$${totalDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.f181.card.tax_savings">Tax savings</div>
                    <div class="value">$${totalSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('f181-table');
    if (!el) return;
    if (!state.productions.length) {
        el.innerHTML = `<h2 data-i18n="view.f181.h2.productions">Productions</h2>
            <p class="muted" data-i18n="view.f181.empty">No productions logged.</p>`;
        return;
    }
    const sorted = [...state.productions].sort((a, b) => b.cost - a.cost);
    el.innerHTML = `
        <h2 data-i18n="view.f181.h2.productions">Productions</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.f181.th.title">Title</th>
                <th data-i18n="view.f181.th.type">Type</th>
                <th data-i18n="view.f181.th.placed">Year</th>
                <th data-i18n="view.f181.th.cost">Cost</th>
                <th data-i18n="view.f181.th.us_comp">US comp %</th>
                <th data-i18n="view.f181.th.cap">Cap</th>
                <th data-i18n="view.f181.th.deduction">Your deduction</th>
                <th data-i18n="view.f181.th.status">Status</th>
                <th data-i18n="view.f181.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(p => {
                const ev = evaluateProduction(p);
                const cls = ev.qualifiesUs ? 'pos' : 'neg';
                const status = ev.qualifiesUs ? t('view.f181.status.qualified') : t('view.f181.status.disqualified');
                return `<tr>
                    <td>${esc(p.title)}</td>
                    <td class="muted">${esc(p.ptype)}</td>
                    <td class="muted">${p.placed_year}</td>
                    <td>$${p.cost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>${(p.us_comp_pct * 100).toFixed(0)}%</td>
                    <td class="muted">$${ev.cap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>$${ev.yourDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="${cls}">${esc(status)}</td>
                    <td><button class="link neg" data-del="${esc(p.id)}" data-i18n="view.f181.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.productions = state.productions.filter(p => p.id !== btn.dataset.del);
            save(state.productions);
            render();
        });
    });
}
