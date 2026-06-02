// § 988 Forex Ordinary Loss Tracker — IRC § 988(a)(1).
// Spot forex (EUR/USD, GBP/USD, etc.) defaults to ORDINARY income/loss.
// Big advantage for losers: 100% deductible vs. only $3k/yr cap-loss limit.
// Election out to § 1256 60/40 available for major-pair forwards/futures but
// must be filed before the trade is placed (annual capital-account election).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-988-positions-v1';
const MAJOR_PAIRS = ['EUR/USD', 'GBP/USD', 'USD/JPY', 'USD/CHF', 'AUD/USD', 'USD/CAD', 'NZD/USD'];

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(p) { try { localStorage.setItem(LS_KEY, JSON.stringify(p)); } catch { /* ignore */ } }

let state = {
    positions: load(),
    year: new Date().getFullYear(),
    marginal_rate: 0.32,
    lt_rate: 0.20,
    st_rate: 0.32,
};

export async function renderForex988(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.f988.h1.title">// § 988 FOREX TRACKER</span></h1>
        <p class="muted small" data-i18n="view.f988.hint.intro">
            Spot forex defaults to <strong>ordinary income</strong> (§ 988). Loss-side
            advantage: 100% deductible vs only $3k/yr capital-loss limit. Gain-side
            disadvantage: taxed at marginal rate vs. potential LT cap-gain. Annual
            <strong>election out to § 1256</strong> available for major-pair forwards.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.f988.h2.add">Add position</h2>
            <form id="f988-form" class="inline-form">
                <label><span data-i18n="view.f988.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}" required></label>
                <label><span data-i18n="view.f988.label.pair">Pair</span>
                    <input type="text" name="pair" placeholder="EUR/USD" required></label>
                <label><span data-i18n="view.f988.label.realized">Realized P&L ($)</span>
                    <input type="number" step="0.01" name="realized" required></label>
                <label><span data-i18n="view.f988.label.elected_1256">§ 1256 election active?</span>
                    <input type="checkbox" name="elected_1256"></label>
                <button class="primary" type="submit" data-i18n="view.f988.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.f988.h2.rates">Tax rates</h2>
            <form id="f988-rates" class="inline-form">
                <label><span data-i18n="view.f988.label.marginal_rate">Marginal rate %</span>
                    <input type="number" step="0.5" name="marginal_rate" value="${(state.marginal_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.f988.label.lt_rate">LT cap-gains %</span>
                    <input type="number" step="0.5" name="lt_rate" value="${(state.lt_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.f988.label.st_rate">ST ordinary %</span>
                    <input type="number" step="0.5" name="st_rate" value="${(state.st_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.f988.btn.update_rates">Update</button>
            </form>
        </div>
        <div id="f988-summary"></div>
        <div id="f988-table" class="chart-panel"></div>
    `;
    document.getElementById('f988-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const p = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            year: Number(fd.get('year')),
            pair: (fd.get('pair') || '').toUpperCase(),
            realized: Number(fd.get('realized')),
            elected_1256: !!fd.get('elected_1256'),
        };
        state.positions.push(p);
        save(state.positions);
        e.target.reset();
        e.target.querySelector('[name="year"]').value = state.year;
        showToast(t('view.f988.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('f988-rates').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.marginal_rate = (Number(fd.get('marginal_rate')) || 32) / 100;
        state.lt_rate = (Number(fd.get('lt_rate')) || 20) / 100;
        state.st_rate = (Number(fd.get('st_rate')) || 32) / 100;
        render();
    });
    render();
}

function render() {
    const yearPos = state.positions.filter(p => p.year === state.year);
    renderSummary(yearPos);
    renderTable(yearPos);
}

function renderSummary(yearPos) {
    const el = document.getElementById('f988-summary');
    if (!el) return;
    const totalRealized = yearPos.reduce((s, p) => s + p.realized, 0);
    const positions988 = yearPos.filter(p => !p.elected_1256);
    const positions1256 = yearPos.filter(p => p.elected_1256);
    const pnl988 = positions988.reduce((s, p) => s + p.realized, 0);
    const pnl1256 = positions1256.reduce((s, p) => s + p.realized, 0);
    const tax988 = pnl988 > 0 ? pnl988 * state.marginal_rate : pnl988 * state.marginal_rate;  // loss reduces income at marginal
    const tax1256 = pnl1256 > 0
        ? (pnl1256 * 0.60 * state.lt_rate + pnl1256 * 0.40 * state.st_rate)
        : (pnl1256 * 0.60 * state.lt_rate + pnl1256 * 0.40 * state.st_rate);
    const totalTax = tax988 + tax1256;
    const allDefault = totalRealized > 0
        ? totalRealized * state.marginal_rate
        : totalRealized * state.marginal_rate;
    const allElected = totalRealized > 0
        ? (totalRealized * 0.60 * state.lt_rate + totalRealized * 0.40 * state.st_rate)
        : (totalRealized * 0.60 * state.lt_rate + totalRealized * 0.40 * state.st_rate);
    const better = allDefault < allElected ? 'default' : 'elected_1256';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.f988.h2.summary">${state.year} summary</h2>
            <div class="cards">
                <div class="card ${totalRealized >= 0 ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.f988.card.total_pnl">Total realized P&L</div>
                    <div class="value">$${totalRealized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.f988.card.pnl_988">§ 988 P&L</div>
                    <div class="value">$${pnl988.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.f988.card.pnl_1256">§ 1256-elected P&L</div>
                    <div class="value">$${pnl1256.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.f988.card.tax">Total tax</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.f988.card.better">Hypothetically better</div>
                    <div class="value">${esc(t('view.f988.label.' + better))}</div>
                </div>
            </div>
            ${totalRealized < 0 ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.f988.loss_note">
                    Net forex LOSS — § 988 ordinary treatment fully deductible against ordinary
                    income (no $3k cap). Election out to § 1256 would have made it a capital
                    loss capped at $3k/yr against ordinary income.
                </p>
            ` : ''}
        </div>
    `;
}

function renderTable(yearPos) {
    const el = document.getElementById('f988-table');
    if (!el) return;
    if (!yearPos.length) {
        el.innerHTML = `<h2 data-i18n="view.f988.h2.positions">Positions</h2>
            <p class="muted" data-i18n="view.f988.empty">No forex positions for this year.</p>`;
        return;
    }
    const sorted = [...yearPos].sort((a, b) => Math.abs(b.realized) - Math.abs(a.realized));
    el.innerHTML = `
        <h2 data-i18n="view.f988.h2.positions">Positions</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.f988.th.pair">Pair</th>
                <th data-i18n="view.f988.th.realized">Realized P&L</th>
                <th data-i18n="view.f988.th.treatment">Treatment</th>
                <th data-i18n="view.f988.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(p => `
                <tr>
                    <td>${esc(p.pair)}</td>
                    <td class="${p.realized >= 0 ? 'pos' : 'neg'}">$${p.realized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>${p.elected_1256 ? esc(t('view.f988.treat.elected_1256')) : esc(t('view.f988.treat.default'))}</td>
                    <td><button class="link neg" data-del="${esc(p.id)}" data-i18n="view.f988.btn.delete">delete</button></td>
                </tr>
            `).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.positions = state.positions.filter(p => p.id !== btn.dataset.del);
            save(state.positions);
            render();
        });
    });
}
