// IRC § 165(g) Worthless Securities + § 1234A Capital Loss on Terminated Contracts.
// § 165(g): security held by individual becomes worthless → capital loss treated as
// sold for $0 on Dec 31 of year of worthlessness. § 165(g)(3): securities of affiliate
// → ORDINARY loss for parent owning ≥ 80%. § 1234A: terminated rights / cancelled options /
// expired forward contracts treated as capital loss on date of termination.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-worthless-v1';

function load() { try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); } catch { return []; } }
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    items: load(),
    year: new Date().getFullYear(),
    marginal_ordinary: 0.32,
    ltcg_rate: 0.20,
};

export async function renderSection165g(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s165g.h1.title">// § 165(g) + § 1234A WORTHLESS / TERMINATED</span></h1>
        <p class="muted small" data-i18n="view.s165g.hint.intro">
            <strong>§ 165(g):</strong> security becomes worthless in year → capital loss treated
            as sold for $0 on <strong>Dec 31</strong>. Hold period extended to year-end:
            > 1 yr held = LT. <strong>§ 165(g)(3) affiliate exception:</strong> parent owning ≥ 80%
            takes <strong>ORDINARY</strong> loss. <strong>§ 1234A:</strong> terminated rights,
            cancelled options, expired forward contracts → capital loss on TERMINATION date,
            preserving holding period.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s165g.h2.add">Log worthless / terminated</h2>
            <form id="s165g-form" class="inline-form">
                <label><span data-i18n="view.s165g.label.year">Tax year</span>
                    <input type="number" step="1" name="year" value="${state.year}" required></label>
                <label><span data-i18n="view.s165g.label.symbol">Symbol / description</span>
                    <input type="text" name="symbol" required></label>
                <label><span data-i18n="view.s165g.label.kind">Kind</span>
                    <select name="kind">
                        <option value="worthless_stock">§ 165(g) worthless stock</option>
                        <option value="worthless_bond">§ 165(g) worthless bond</option>
                        <option value="affiliate_ordinary">§ 165(g)(3) affiliate (ORDINARY)</option>
                        <option value="expired_option_long">§ 1234A expired long option</option>
                        <option value="expired_option_short">§ 1234A short option closed at $0</option>
                        <option value="terminated_contract">§ 1234A terminated forward / swap</option>
                    </select>
                </label>
                <label><span data-i18n="view.s165g.label.basis">Cost basis ($)</span>
                    <input type="number" step="0.01" name="basis" required></label>
                <label><span data-i18n="view.s165g.label.purchase_date">Purchase date</span>
                    <input type="date" name="purchase_date" required></label>
                <label><span data-i18n="view.s165g.label.worthless_date">Date of worthlessness / termination</span>
                    <input type="date" name="worthless_date" required></label>
                <button class="primary" type="submit" data-i18n="view.s165g.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.s165g.label.year_view">View year</span>
                    <input type="number" id="s165g-year" value="${state.year}"></label>
                <label><span data-i18n="view.s165g.label.ordinary">Ordinary marginal %</span>
                    <input type="number" step="0.01" id="s165g-ord" value="${state.marginal_ordinary}"></label>
                <label><span data-i18n="view.s165g.label.ltcg">LTCG %</span>
                    <input type="number" step="0.01" id="s165g-ltcg" value="${state.ltcg_rate}"></label>
            </div>
        </div>
        <div id="s165g-summary"></div>
        <div id="s165g-table" class="chart-panel"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s165g.h2.proof">Proving worthlessness</h2>
            <ul class="muted small">
                <li data-i18n="view.s165g.proof.bankruptcy">Bankruptcy Chapter 7 confirms (not Chapter 11 reorg)</li>
                <li data-i18n="view.s165g.proof.no_liability">Delisting alone insufficient — need NO LIABILITY value remaining</li>
                <li data-i18n="view.s165g.proof.abandon">Voluntary abandonment + § 1.165-5 statement to broker</li>
                <li data-i18n="view.s165g.proof.identifiable_event">"Identifiable event" doctrine — bankruptcy filing OR cessation of trading</li>
                <li data-i18n="view.s165g.proof.documentation">Document: news, SEC filings, broker confirmation, expert opinion</li>
                <li data-i18n="view.s165g.proof.refund_claim">If wrong year identified: § 6511(d)(1) extends SOL to 7 yrs</li>
            </ul>
        </div>
    `;
    document.getElementById('s165g-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const i = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            year: Number(fd.get('year')),
            symbol: fd.get('symbol'),
            kind: fd.get('kind'),
            basis: Number(fd.get('basis')) || 0,
            purchase_date: fd.get('purchase_date'),
            worthless_date: fd.get('worthless_date'),
        };
        state.items.push(i);
        save(state.items);
        e.target.reset();
        e.target.querySelector('[name="year"]').value = state.year;
        showToast(t('view.s165g.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('s165g-year').addEventListener('change', e => {
        state.year = Number(e.target.value) || new Date().getFullYear();
        render();
    });
    document.getElementById('s165g-ord').addEventListener('change', e => {
        state.marginal_ordinary = Number(e.target.value) || 0.32;
        render();
    });
    document.getElementById('s165g-ltcg').addEventListener('change', e => {
        state.ltcg_rate = Number(e.target.value) || 0.20;
        render();
    });
    render();
}

function classify(item) {
    const isOrdinary = item.kind === 'affiliate_ordinary';
    const purchaseDate = new Date(item.purchase_date);
    let yearEnd;
    if (item.kind.startsWith('worthless_') || item.kind === 'affiliate_ordinary') {
        yearEnd = new Date(item.year, 11, 31);
    } else {
        yearEnd = new Date(item.worthless_date);
    }
    const yearsHeld = (yearEnd - purchaseDate) / (365.25 * 24 * 3600 * 1000);
    const isLongTerm = yearsHeld > 1;
    return { isOrdinary, isLongTerm, yearsHeld };
}

function render() {
    const yearItems = state.items.filter(i => i.year === state.year);
    renderSummary(yearItems);
    renderTable(yearItems);
}

function renderSummary(yearItems) {
    const el = document.getElementById('s165g-summary');
    if (!el) return;
    let totalOrdinary = 0, totalLT = 0, totalST = 0, taxSavings = 0;
    for (const i of yearItems) {
        const { isOrdinary, isLongTerm } = classify(i);
        if (isOrdinary) {
            totalOrdinary += i.basis;
            taxSavings += i.basis * state.marginal_ordinary;
        } else if (isLongTerm) {
            totalLT += i.basis;
            taxSavings += i.basis * state.ltcg_rate;
        } else {
            totalST += i.basis;
            taxSavings += i.basis * state.marginal_ordinary;
        }
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s165g.h2.summary">${state.year} summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s165g.card.items">Items</div>
                    <div class="value">${yearItems.length}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s165g.card.ordinary_loss">§ 165(g)(3) ordinary loss</div>
                    <div class="value">$${totalOrdinary.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s165g.card.lt_cap_loss">LT capital loss</div>
                    <div class="value">$${totalLT.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s165g.card.st_cap_loss">ST capital loss</div>
                    <div class="value">$${totalST.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s165g.card.tax_savings">Total tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(yearItems) {
    const el = document.getElementById('s165g-table');
    if (!el) return;
    if (!yearItems.length) {
        el.innerHTML = `<h2 data-i18n="view.s165g.h2.items">Items</h2>
            <p class="muted" data-i18n="view.s165g.empty">No worthless / terminated items for this year.</p>`;
        return;
    }
    const sorted = [...yearItems].sort((a, b) => b.basis - a.basis);
    el.innerHTML = `
        <h2 data-i18n="view.s165g.h2.items">Items</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s165g.th.symbol">Symbol</th>
                <th data-i18n="view.s165g.th.kind">Kind</th>
                <th data-i18n="view.s165g.th.basis">Basis</th>
                <th data-i18n="view.s165g.th.character">Character</th>
                <th data-i18n="view.s165g.th.holding">Holding</th>
                <th data-i18n="view.s165g.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(i => {
                const c = classify(i);
                const character = c.isOrdinary
                    ? t('view.s165g.char.ordinary')
                    : (c.isLongTerm ? t('view.s165g.char.lt') : t('view.s165g.char.st'));
                const charCls = c.isOrdinary ? 'pos' : 'muted';
                return `<tr>
                    <td>${esc(i.symbol)}</td>
                    <td class="muted">${esc(i.kind)}</td>
                    <td>$${i.basis.toLocaleString(undefined, { maximumFractionDigits: 2 })}</td>
                    <td class="${charCls}">${esc(character)}</td>
                    <td class="muted">${c.yearsHeld.toFixed(1)} yr</td>
                    <td><button class="link neg" data-del="${esc(i.id)}" data-i18n="view.s165g.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.items = state.items.filter(i => i.id !== btn.dataset.del);
            save(state.items);
            render();
        });
    });
}
