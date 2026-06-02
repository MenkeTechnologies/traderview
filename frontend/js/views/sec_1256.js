// Section 1256 60/40 Calculator — futures, broad-based index options (SPX/NDX),
// foreign currency forwards, dealer equity options.
// Tax treatment: 60% long-term + 40% short-term regardless of holding period,
// PLUS mark-to-market at year-end (open positions treated as sold at FMV).
// Can carry net loss BACK 3 years (only § 1256, not regular cap loss).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-1256-positions-v1';

const QUALIFYING_INSTRUMENTS = [
    'Regulated futures (ES, NQ, CL, GC, etc.)',
    'Broad-based index options (SPX, NDX, RUT, VIX)',
    'Foreign currency forward contracts',
    'Dealer equity options',
    'Non-equity options (commodity)',
];

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(p) { try { localStorage.setItem(LS_KEY, JSON.stringify(p)); } catch { /* ignore */ } }

let state = {
    positions: load(),
    lt_rate: 0.20,
    st_rate: 0.32,
    year: new Date().getFullYear(),
};

export async function renderSec1256(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1256.h1.title">// SECTION 1256 60/40</span></h1>
        <p class="muted small" data-i18n="view.s1256.hint.intro">
            Futures + SPX/NDX/VIX options + forwards: 60% long-term / 40% short-term
            regardless of holding period. Mark-to-market at year-end. Net § 1256 loss
            can carry BACK 3 years (unique benefit vs. regular capital loss).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1256.h2.qualifying">Qualifying instruments</h2>
            <ul class="muted small">
                ${QUALIFYING_INSTRUMENTS.map(i => `<li>${esc(i)}</li>`).join('')}
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1256.h2.add">Add position</h2>
            <form id="s1256-form" class="inline-form">
                <label><span data-i18n="view.s1256.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}" required></label>
                <label><span data-i18n="view.s1256.label.symbol">Symbol</span>
                    <input type="text" name="symbol" placeholder="/ES, SPX 4500 call" required></label>
                <label><span data-i18n="view.s1256.label.realized_pnl">Realized P&L ($)</span>
                    <input type="number" step="0.01" name="realized_pnl" value="0"></label>
                <label><span data-i18n="view.s1256.label.mtm_pnl">Year-end MTM P&L ($)</span>
                    <input type="number" step="0.01" name="mtm_pnl" value="0"></label>
                <label><span data-i18n="view.s1256.label.contracts">Contracts</span>
                    <input type="number" step="1" name="contracts" value="1"></label>
                <button class="primary" type="submit" data-i18n="view.s1256.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1256.h2.rates">Tax rates</h2>
            <form id="s1256-rates" class="inline-form">
                <label><span data-i18n="view.s1256.label.lt_rate">LT cap-gains rate %</span>
                    <input type="number" step="0.5" name="lt_rate" value="${(state.lt_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.s1256.label.st_rate">ST ordinary rate %</span>
                    <input type="number" step="0.5" name="st_rate" value="${(state.st_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.s1256.btn.update_rates">Update rates</button>
            </form>
        </div>
        <div id="s1256-summary"></div>
        <div id="s1256-table" class="chart-panel"></div>
    `;
    document.getElementById('s1256-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const p = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            year: Number(fd.get('year')),
            symbol: fd.get('symbol'),
            realized_pnl: Number(fd.get('realized_pnl')) || 0,
            mtm_pnl: Number(fd.get('mtm_pnl')) || 0,
            contracts: Number(fd.get('contracts')) || 1,
        };
        state.positions.push(p);
        save(state.positions);
        e.target.reset();
        e.target.querySelector('[name="year"]').value = state.year;
        e.target.querySelector('[name="contracts"]').value = 1;
        showToast(t('view.s1256.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('s1256-rates').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
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
    const el = document.getElementById('s1256-summary');
    if (!el) return;
    const totalPnL = yearPos.reduce((s, p) => s + (p.realized_pnl + p.mtm_pnl), 0);
    const longTermPortion = totalPnL * 0.60;
    const shortTermPortion = totalPnL * 0.40;
    const ltTax = longTermPortion > 0 ? longTermPortion * state.lt_rate : 0;
    const stTax = shortTermPortion > 0 ? shortTermPortion * state.st_rate : 0;
    const totalTax = ltTax + stTax;
    const effectiveRate = totalPnL > 0 ? (totalTax / totalPnL) : 0;
    const equivalentSTtax = totalPnL > 0 ? totalPnL * state.st_rate : 0;
    const savings = equivalentSTtax - totalTax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1256.h2.summary">${state.year} 60/40 summary</h2>
            <div class="cards">
                <div class="card ${totalPnL >= 0 ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s1256.card.total_pnl">Total P&L</div>
                    <div class="value">$${totalPnL.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1256.card.lt_portion">60% long-term</div>
                    <div class="value">$${longTermPortion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1256.card.st_portion">40% short-term</div>
                    <div class="value">$${shortTermPortion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1256.card.total_tax">Total tax</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1256.card.effective">Effective rate</div>
                    <div class="value">${(effectiveRate * 100).toFixed(1)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1256.card.savings_vs_st">Savings vs all-ST</div>
                    <div class="value">$${savings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${totalPnL < 0 ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s1256.loss_carryback">
                    Net § 1256 LOSS — can carry BACK up to 3 years to offset prior § 1256 gains.
                    File Form 6781 + amend prior returns (Form 1040-X). Carryforward is also OK.
                </p>
            ` : ''}
        </div>
    `;
}

function renderTable(yearPos) {
    const el = document.getElementById('s1256-table');
    if (!el) return;
    if (!yearPos.length) {
        el.innerHTML = `<h2 data-i18n="view.s1256.h2.positions">Positions</h2>
            <p class="muted" data-i18n="view.s1256.empty">No § 1256 positions for this year.</p>`;
        return;
    }
    const sorted = [...yearPos].sort((a, b) =>
        (b.realized_pnl + b.mtm_pnl) - (a.realized_pnl + a.mtm_pnl));
    el.innerHTML = `
        <h2 data-i18n="view.s1256.h2.positions">Positions</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s1256.th.symbol">Symbol</th>
                <th data-i18n="view.s1256.th.contracts">Contracts</th>
                <th data-i18n="view.s1256.th.realized">Realized P&L</th>
                <th data-i18n="view.s1256.th.mtm">MTM P&L</th>
                <th data-i18n="view.s1256.th.total">Total P&L</th>
                <th data-i18n="view.s1256.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(p => {
                const total = p.realized_pnl + p.mtm_pnl;
                return `<tr>
                    <td>${esc(p.symbol)}</td>
                    <td>${p.contracts}</td>
                    <td class="${p.realized_pnl >= 0 ? 'pos' : 'neg'}">$${p.realized_pnl.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="${p.mtm_pnl >= 0 ? 'pos' : 'neg'}">$${p.mtm_pnl.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="${total >= 0 ? 'pos' : 'neg'}">$${total.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td><button class="link neg" data-del="${esc(p.id)}" data-i18n="view.s1256.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
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
