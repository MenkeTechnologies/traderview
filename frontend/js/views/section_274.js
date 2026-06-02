// IRC § 274 — Business meals 50% + entertainment DISALLOWED + client gift $25 cap.
// TCJA 2018: business entertainment 100% disallowed (sports tickets, golf, theater).
// Meals: 50% deductible (was 100% temporarily 2021-2022 for restaurant CARES Act).
// Client gifts: $25/recipient/year (de minimis branded items exempt).
// Self-employed meals: same 50% on Schedule C.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-section-274-v1';
const GIFT_CAP_PER_RECIPIENT = 25;

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    entries: load(),
    year: new Date().getFullYear(),
    marginal_rate: 0.32,
};

export async function renderSection274(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s274.h1.title">// § 274 MEALS / GIFTS / ENTERTAINMENT</span></h1>
        <p class="muted small" data-i18n="view.s274.hint.intro">
            Post-TCJA (2018+): <strong>business entertainment 100% DISALLOWED</strong>
            (sports tickets, golf, theater, concerts). Business meals <strong>50% deductible</strong>
            (briefly 100% restaurants 2021-22 under CAA). Client gifts capped at
            <strong>$25/recipient/year</strong> (de minimis branded items &lt; $4 with logo exempt).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s274.h2.rules">Quick rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s274.rule.meal_50">Business meal with client / prospect: 50% deductible</li>
                <li data-i18n="view.s274.rule.travel_meal">Travel meal (away from tax home): 50% deductible</li>
                <li data-i18n="view.s274.rule.employee">Employee meal at workplace: 50% (was 100% pre-TCJA)</li>
                <li data-i18n="view.s274.rule.party">Employee party / picnic: 100% deductible</li>
                <li data-i18n="view.s274.rule.entertain">Sports tickets / golf / theater: 0% (FULLY DISALLOWED)</li>
                <li data-i18n="view.s274.rule.gift_25">Client gift: $25/recipient/yr (per donor not per gift)</li>
                <li data-i18n="view.s274.rule.de_minimis">De minimis branded items (&lt; $4, logo) exempt from $25 cap</li>
                <li data-i18n="view.s274.rule.advertising">Promotional items (T-shirts, pens): § 162 advertising, 100%</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s274.h2.add">Log entry</h2>
            <form id="s274-form" class="inline-form">
                <label><span data-i18n="view.s274.label.date">Date</span>
                    <input type="date" name="date" value="${new Date().toISOString().slice(0,10)}" required></label>
                <label><span data-i18n="view.s274.label.category">Category</span>
                    <select name="category">
                        <option value="client_meal">Client meal (50%)</option>
                        <option value="travel_meal">Travel meal (50%)</option>
                        <option value="employee_meal">Employee meal (50%)</option>
                        <option value="employee_party">Employee party (100%)</option>
                        <option value="entertainment">Entertainment (0% — disallowed)</option>
                        <option value="client_gift">Client gift ($25 cap)</option>
                        <option value="branded">Branded de-minimis (100%)</option>
                        <option value="advertising">Promotional / advertising (100%)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s274.label.recipient">Recipient</span>
                    <input type="text" name="recipient" placeholder="John Smith / All employees"></label>
                <label><span data-i18n="view.s274.label.amount">Amount ($)</span>
                    <input type="number" step="0.01" name="amount" required></label>
                <label><span data-i18n="view.s274.label.purpose">Business purpose</span>
                    <input type="text" name="purpose" placeholder="Q4 strategy discussion"></label>
                <button class="primary" type="submit" data-i18n="view.s274.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.s274.label.year">Tax year</span>
                    <input type="number" id="s274-year" value="${state.year}"></label>
                <label><span data-i18n="view.s274.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" id="s274-marginal" value="${state.marginal_rate}"></label>
            </div>
        </div>
        <div id="s274-summary"></div>
        <div id="s274-table" class="chart-panel"></div>
    `;
    document.getElementById('s274-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const entry = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            date: fd.get('date'),
            category: fd.get('category'),
            recipient: fd.get('recipient') || '',
            amount: Number(fd.get('amount')) || 0,
            purpose: fd.get('purpose') || '',
        };
        state.entries.push(entry);
        save(state.entries);
        e.target.reset();
        e.target.querySelector('[name="date"]').value = new Date().toISOString().slice(0,10);
        showToast(t('view.s274.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('s274-year').addEventListener('change', e => {
        state.year = Number(e.target.value) || new Date().getFullYear();
        render();
    });
    document.getElementById('s274-marginal').addEventListener('change', e => {
        state.marginal_rate = Number(e.target.value) || 0.32;
        render();
    });
    render();
}

function deductibleAmount(entry, perRecipientGiftSoFar) {
    switch (entry.category) {
        case 'client_meal':
        case 'travel_meal':
        case 'employee_meal': return entry.amount * 0.50;
        case 'employee_party':
        case 'branded':
        case 'advertising': return entry.amount;
        case 'entertainment': return 0;
        case 'client_gift': {
            const remaining = Math.max(0, GIFT_CAP_PER_RECIPIENT - perRecipientGiftSoFar);
            return Math.min(entry.amount, remaining);
        }
        default: return 0;
    }
}

function render() {
    const yearEntries = state.entries.filter(e => (e.date || '').startsWith(String(state.year)));
    renderSummary(yearEntries);
    renderTable(yearEntries);
}

function renderSummary(yearEntries) {
    const el = document.getElementById('s274-summary');
    if (!el) return;
    const sorted = [...yearEntries].sort((a, b) => (a.date || '').localeCompare(b.date || ''));
    const giftPerRecipient = new Map();
    let totalSpent = 0, totalDeductible = 0, totalLost = 0;
    let entertainmentTotal = 0;
    let giftCapped = 0;
    for (const e of sorted) {
        totalSpent += e.amount;
        if (e.category === 'entertainment') {
            entertainmentTotal += e.amount;
            totalLost += e.amount;
            continue;
        }
        if (e.category === 'client_gift') {
            const k = e.recipient || '(unspecified)';
            const used = giftPerRecipient.get(k) || 0;
            const deductible = deductibleAmount(e, used);
            const stillCapped = Math.max(0, e.amount - deductible);
            giftCapped += stillCapped;
            totalLost += stillCapped;
            giftPerRecipient.set(k, used + e.amount);
            totalDeductible += deductible;
            continue;
        }
        const ded = deductibleAmount(e, 0);
        totalDeductible += ded;
        totalLost += (e.amount - ded);
    }
    const taxSavings = totalDeductible * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s274.h2.summary">${state.year} summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s274.card.entries">Entries</div>
                    <div class="value">${yearEntries.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s274.card.total_spent">Total spent</div>
                    <div class="value">$${totalSpent.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s274.card.deductible">Total deductible</div>
                    <div class="value">$${totalDeductible.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s274.card.disallowed">Disallowed</div>
                    <div class="value">$${totalLost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s274.card.tax_savings">Tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${entertainmentTotal > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s274.card.entertainment_lost">Entertainment 100% disallowed</div>
                        <div class="value">$${entertainmentTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                ${giftCapped > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s274.card.gift_capped">Gift over-cap loss</div>
                        <div class="value">$${giftCapped.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
}

function renderTable(yearEntries) {
    const el = document.getElementById('s274-table');
    if (!el) return;
    if (!yearEntries.length) {
        el.innerHTML = `<h2 data-i18n="view.s274.h2.entries">Entries</h2>
            <p class="muted" data-i18n="view.s274.empty">No entries logged for this year.</p>`;
        return;
    }
    const sorted = [...yearEntries].sort((a, b) => (b.date || '').localeCompare(a.date || ''));
    const giftRunning = new Map();
    el.innerHTML = `
        <h2 data-i18n="view.s274.h2.entries">Entries</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s274.th.date">Date</th>
                <th data-i18n="view.s274.th.category">Category</th>
                <th data-i18n="view.s274.th.recipient">Recipient</th>
                <th data-i18n="view.s274.th.amount">Amount</th>
                <th data-i18n="view.s274.th.deductible">Deductible</th>
                <th data-i18n="view.s274.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(e => {
                const used = e.category === 'client_gift' ? (giftRunning.get(e.recipient || '(unspecified)') || 0) : 0;
                const ded = deductibleAmount(e, used);
                if (e.category === 'client_gift') giftRunning.set(e.recipient || '(unspecified)', used + e.amount);
                const cls = ded === e.amount ? 'pos' : (ded === 0 ? 'neg' : 'muted');
                return `<tr>
                    <td class="muted">${esc(e.date || '')}</td>
                    <td class="muted">${esc(e.category)}</td>
                    <td class="muted">${esc(e.recipient || '—')}</td>
                    <td>$${e.amount.toLocaleString(undefined, { maximumFractionDigits: 2 })}</td>
                    <td class="${cls}">$${ded.toLocaleString(undefined, { maximumFractionDigits: 2 })}</td>
                    <td><button class="link neg" data-del="${esc(e.id)}" data-i18n="view.s274.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.entries = state.entries.filter(e => e.id !== btn.dataset.del);
            save(state.entries);
            render();
        });
    });
}
