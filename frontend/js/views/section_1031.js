// § 1031 Like-Kind Exchange Tracker.
// Real estate only since TCJA 2018. Defer cap gains by exchanging into
// like-kind real estate. Strict deadlines:
//   - 45 days to IDENTIFY replacement property (in writing, to QI)
//   - 180 days to CLOSE on replacement (or tax return due date if earlier)
// Boot (cash, debt relief) = taxable. Basis carries over (new basis =
// old basis + gain recognized + new debt - old debt).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-1031-v1';

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = { exchanges: load() };

export async function renderSection1031(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1031.h1.title">// § 1031 LIKE-KIND EXCHANGE</span></h1>
        <p class="muted small" data-i18n="view.s1031.hint.intro">
            Real estate only (post-TCJA 2018). Strict deadlines: <strong>45 days</strong>
            to identify replacement property in writing, <strong>180 days</strong> to close.
            Boot (cash, debt relief) = taxable. Basis carries over. Death = step-up basis
            wipes out deferred gain. <strong>QI required</strong> — can't touch proceeds.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1031.h2.add">Log exchange</h2>
            <form id="ex-form" class="inline-form">
                <label><span data-i18n="view.s1031.label.relinquished_close">Relinquished property closed</span>
                    <input type="date" name="relinquished_close_date" required></label>
                <label><span data-i18n="view.s1031.label.relinquished_basis">Relinquished basis ($)</span>
                    <input type="number" step="1000" name="relinquished_basis" required></label>
                <label><span data-i18n="view.s1031.label.relinquished_sale_price">Sale price ($)</span>
                    <input type="number" step="1000" name="sale_price" required></label>
                <label><span data-i18n="view.s1031.label.relinquished_debt">Debt paid off ($)</span>
                    <input type="number" step="1000" name="relinquished_debt" value="0"></label>
                <label><span data-i18n="view.s1031.label.replacement_address">Replacement property</span>
                    <input type="text" name="replacement_address" placeholder="Address or TBD"></label>
                <label><span data-i18n="view.s1031.label.replacement_purchase_price">Replacement purchase price ($)</span>
                    <input type="number" step="1000" name="replacement_purchase_price" value="0"></label>
                <label><span data-i18n="view.s1031.label.new_debt">New debt incurred ($)</span>
                    <input type="number" step="1000" name="new_debt" value="0"></label>
                <label><span data-i18n="view.s1031.label.cash_boot">Cash boot received ($)</span>
                    <input type="number" step="1000" name="cash_boot" value="0"></label>
                <label><span data-i18n="view.s1031.label.qi">QI used?</span>
                    <input type="checkbox" name="qi_used" checked></label>
                <button class="primary" type="submit" data-i18n="view.s1031.btn.add">Add</button>
            </form>
        </div>
        <div id="ex-summary"></div>
        <div id="ex-table" class="chart-panel"></div>
    `;
    document.getElementById('ex-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const ex = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            relinquished_close_date: fd.get('relinquished_close_date'),
            relinquished_basis: Number(fd.get('relinquished_basis')),
            sale_price: Number(fd.get('sale_price')),
            relinquished_debt: Number(fd.get('relinquished_debt')) || 0,
            replacement_address: fd.get('replacement_address') || '',
            replacement_purchase_price: Number(fd.get('replacement_purchase_price')) || 0,
            new_debt: Number(fd.get('new_debt')) || 0,
            cash_boot: Number(fd.get('cash_boot')) || 0,
            qi_used: !!fd.get('qi_used'),
        };
        state.exchanges.push(ex);
        save(state.exchanges);
        e.target.reset();
        e.target.querySelector('[name="qi_used"]').checked = true;
        showToast(t('view.s1031.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function analyzeExchange(ex) {
    const closeDate = new Date(ex.relinquished_close_date);
    const day45 = new Date(closeDate.getTime() + 45 * 86_400_000);
    const day180 = new Date(closeDate.getTime() + 180 * 86_400_000);
    const today = new Date();
    const daysFrom45 = Math.floor((day45 - today) / 86_400_000);
    const daysFrom180 = Math.floor((day180 - today) / 86_400_000);
    const realizedGain = ex.sale_price - ex.relinquished_basis;
    const debtBoot = Math.max(0, ex.relinquished_debt - ex.new_debt);
    const totalBoot = ex.cash_boot + debtBoot;
    const recognizedGain = Math.min(realizedGain, totalBoot);
    const deferredGain = Math.max(0, realizedGain - recognizedGain);
    const newBasis = ex.relinquished_basis + recognizedGain - ex.cash_boot + (ex.new_debt - ex.relinquished_debt);
    return {
        closeDate, day45, day180, daysFrom45, daysFrom180,
        realizedGain, totalBoot, recognizedGain, deferredGain, newBasis,
    };
}

function render() {
    const stats = state.exchanges.map(analyzeExchange);
    renderSummary(stats);
    renderTable(stats);
}

function renderSummary(stats) {
    const el = document.getElementById('ex-summary');
    if (!el) return;
    const totalRealized = stats.reduce((s, x) => s + x.realizedGain, 0);
    const totalDeferred = stats.reduce((s, x) => s + x.deferredGain, 0);
    const totalRecognized = stats.reduce((s, x) => s + x.recognizedGain, 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1031.h2.summary">Aggregate</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1031.card.exchanges">Exchanges</div>
                    <div class="value">${state.exchanges.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1031.card.realized">Realized gain</div>
                    <div class="value">$${totalRealized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1031.card.deferred">Deferred gain</div>
                    <div class="value">$${totalDeferred.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1031.card.recognized">Recognized (boot)</div>
                    <div class="value">$${totalRecognized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(stats) {
    const el = document.getElementById('ex-table');
    if (!el) return;
    if (!state.exchanges.length) {
        el.innerHTML = `<h2 data-i18n="view.s1031.h2.exchanges">Exchanges</h2>
            <p class="muted" data-i18n="view.s1031.empty">No exchanges tracked yet.</p>`;
        return;
    }
    el.innerHTML = `
        <h2 data-i18n="view.s1031.h2.exchanges">Exchanges</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s1031.th.close">Close date</th>
                <th data-i18n="view.s1031.th.day_45">Day 45 deadline</th>
                <th data-i18n="view.s1031.th.day_180">Day 180 deadline</th>
                <th data-i18n="view.s1031.th.realized">Realized</th>
                <th data-i18n="view.s1031.th.boot">Total boot</th>
                <th data-i18n="view.s1031.th.recognized">Recognized</th>
                <th data-i18n="view.s1031.th.deferred">Deferred</th>
                <th data-i18n="view.s1031.th.new_basis">New basis</th>
                <th data-i18n="view.s1031.th.replacement">Replacement</th>
                <th data-i18n="view.s1031.th.actions">Actions</th>
            </tr></thead>
            <tbody>${state.exchanges.map((ex, i) => {
                const s = stats[i];
                const cls45 = s.daysFrom45 >= 0 ? 'pos' : 'neg';
                const cls180 = s.daysFrom180 >= 0 ? 'pos' : 'neg';
                return `<tr>
                    <td>${esc(ex.relinquished_close_date)}</td>
                    <td class="${cls45}">${esc(s.day45.toISOString().slice(0, 10))}
                        <span class="muted small">(${s.daysFrom45 >= 0 ? s.daysFrom45 + 'd left' : 'past'})</span></td>
                    <td class="${cls180}">${esc(s.day180.toISOString().slice(0, 10))}
                        <span class="muted small">(${s.daysFrom180 >= 0 ? s.daysFrom180 + 'd left' : 'past'})</span></td>
                    <td>$${s.realizedGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="neg">$${s.totalBoot.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="neg">$${s.recognizedGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="pos">$${s.deferredGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>$${s.newBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">${esc(ex.replacement_address || '—')}</td>
                    <td><button class="link neg" data-del="${esc(ex.id)}" data-i18n="view.s1031.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.exchanges = state.exchanges.filter(ex => ex.id !== btn.dataset.del);
            save(state.exchanges);
            render();
        });
    });
}
