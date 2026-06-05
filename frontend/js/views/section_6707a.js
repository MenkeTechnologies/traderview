// IRC § 6707A — Reportable Transaction + Listed Transaction penalty.
// 5 categories: Listed (most aggressive), Confidential, Loss, Transaction-of-Interest, Contractual Protection.
// Penalty: 75% of decrease in tax claimed (min $10k individual / $50k entity, max $100k / $200k).
// Reportable on Form 8886 with return. Common: SCEs (syndicated conservation easements),
// captive insurance § 831(b), monetized installment sales, Puerto Rico Act 60.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-6707a-v1';
const MIN_INDIV = 10_000;
const MIN_ENTITY = 50_000;
const MAX_INDIV = 100_000;
const MAX_ENTITY = 200_000;
const PENALTY_RATE = 0.75;

function load() { try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); } catch { return []; } }
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    transactions: load(),
};

export async function renderSection6707a(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6707a.h1.title">// § 6707A REPORTABLE TRANSACTION</span></h1>
        <p class="muted small" data-i18n="view.s6707a.hint.intro">
            <strong>5 categories:</strong> Listed (most aggressive) / Confidential / Loss
            / Transaction-of-Interest / Contractual Protection. Penalty: <strong>75% of decrease
            in tax claimed</strong> (min $10k / $50k entity, max $100k / $200k entity). Reportable
            on Form 8886. Strict liability — no reasonable cause defense.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6707a.h2.add">Log reportable transaction</h2>
            <form id="s6707a-form" class="inline-form">
                <label><span data-i18n="view.s6707a.label.name">Transaction name</span>
                    <input type="text" name="name" required></label>
                <label><span data-i18n="view.s6707a.label.category">Category</span>
                    <select name="category">
                        <option value="listed">Listed Transaction</option>
                        <option value="confidential">Confidential</option>
                        <option value="loss">Loss (&gt; $2M individual / $10M entity)</option>
                        <option value="toi">Transaction-of-Interest</option>
                        <option value="contractual_protection">Contractual Protection</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6707a.label.tax_savings">Tax decrease claimed ($)</span>
                    <input type="number" step="0.01" name="tax_savings_claimed" required></label>
                <label><span data-i18n="view.s6707a.label.entity_type">Entity type</span>
                    <select name="entity_type">
                        <option value="individual">Individual / Sched K-1 owner</option>
                        <option value="entity">C-corp / S-corp / partnership entity</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6707a.label.form_8886_filed">Form 8886 filed?</span>
                    <input type="checkbox" name="form_8886_filed"></label>
                <label><span data-i18n="view.s6707a.label.years_unreported">Years unreported</span>
                    <input type="number" step="1" name="years_unreported" value="0"></label>
                <button class="primary" type="submit" data-i18n="view.s6707a.btn.add">Add</button>
            </form>
        </div>
        <div id="s6707a-summary"></div>
        <div id="s6707a-table" class="chart-panel"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6707a.h2.listed_transactions">Currently identified Listed Transactions</h2>
            <ul class="muted small">
                <li data-i18n="view.s6707a.listed.scl">Syndicated Conservation Easement (Notice 2017-10, 2023-30)</li>
                <li data-i18n="view.s6707a.listed.captive">Micro-captive § 831(b) insurance (Notice 2016-66, Final Regs 2024)</li>
                <li data-i18n="view.s6707a.listed.misc">CRT-CRAT zero-out abuse (Notice 2024-37)</li>
                <li data-i18n="view.s6707a.listed.section_267">"Distressed asset / debt" deduction (Notice 2008-34)</li>
                <li data-i18n="view.s6707a.listed.option_strategies">Cross-option holding (Notice 2002-50)</li>
                <li data-i18n="view.s6707a.listed.straddle_abuse">Straddle-tax shelter (Notice 2002-65)</li>
                <li data-i18n="view.s6707a.listed.notional">Notional contract abuse</li>
                <li data-i18n="view.s6707a.listed.monetized">Monetized installment sales (Notice 2023-34)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6707a.h2.material_advisor">§ 6707 / § 6708 Material Advisor</h2>
            <p class="muted small" data-i18n="view.s6707a.material.body">
                Material Advisors who promote / draft / sell Reportable Transactions face their own
                penalties: <strong>§ 6707: $50k flat (or $200k for Listed)</strong> for failing to
                file Form 8918. <strong>§ 6708: $10k per day</strong> (max $50k) for not maintaining
                investor / advisee list. Form 13976 required to disclose advisor identity to investors.
            </p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6707a.h2.misc_penalties">Stacked § 6662A penalties</h2>
            <ul class="muted small">
                <li data-i18n="view.s6707a.stack.6662a">20% accuracy-related (30% if non-disclosed)</li>
                <li data-i18n="view.s6707a.stack.6694">Return preparer penalties for advising</li>
                <li data-i18n="view.s6707a.stack.7203">Criminal § 7203 willful failure to file (1-yr misdemeanor)</li>
                <li data-i18n="view.s6707a.stack.7206">Criminal § 7206 false statements (3-yr felony)</li>
                <li data-i18n="view.s6707a.stack.7201">Criminal § 7201 tax evasion (5-yr felony)</li>
                <li data-i18n="view.s6707a.stack.civil_fraud">§ 6663 75% civil fraud penalty</li>
            </ul>
        </div>
    `;
    document.getElementById('s6707a-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.transactions.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            name: fd.get('name'),
            category: fd.get('category'),
            tax_savings_claimed: Number(fd.get('tax_savings_claimed')) || 0,
            entity_type: fd.get('entity_type'),
            form_8886_filed: !!fd.get('form_8886_filed'),
            years_unreported: Number(fd.get('years_unreported')) || 0,
        });
        save(state.transactions);
        e.target.reset();
        showToast(t('view.s6707a.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function penaltyFor(tx) {
    if (tx.form_8886_filed) return 0;
    const calc = tx.tax_savings_claimed * PENALTY_RATE;
    const min = tx.entity_type === 'entity' ? MIN_ENTITY : MIN_INDIV;
    const max = tx.entity_type === 'entity' ? MAX_ENTITY : MAX_INDIV;
    return Math.min(max, Math.max(min, calc)) * Math.max(1, tx.years_unreported);
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('s6707a-summary');
    if (!el) return;
    const totalPenalty = state.transactions.reduce((s, t) => s + penaltyFor(t), 0);
    const totalSavings = state.transactions.reduce((s, t) => s + t.tax_savings_claimed, 0);
    const unfiled = state.transactions.filter(t => !t.form_8886_filed).length;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6707a.h2.summary">Summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6707a.card.count">Transactions</div>
                    <div class="value">${state.transactions.length}</div>
                </div>
                <div class="card ${unfiled > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6707a.card.unfiled">Unfiled Form 8886</div>
                    <div class="value">${unfiled}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6707a.card.savings">Claimed tax savings</div>
                    <div class="value">$${totalSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${totalPenalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6707a.card.penalty">§ 6707A penalty exposure</div>
                    <div class="value">$${totalPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('s6707a-table');
    if (!el) return;
    if (!state.transactions.length) {
        el.innerHTML = `<h2 data-i18n="view.s6707a.h2.transactions">Transactions</h2>
            <p class="muted" data-i18n="view.s6707a.empty">No transactions logged.</p>`;
        return;
    }
    const sorted = [...state.transactions].sort((a, b) => penaltyFor(b) - penaltyFor(a));
    el.innerHTML = `
        <h2 data-i18n="view.s6707a.h2.transactions">Transactions</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s6707a.th.name">Name</th>
                <th data-i18n="view.s6707a.th.category">Category</th>
                <th data-i18n="view.s6707a.th.savings">Tax savings</th>
                <th data-i18n="view.s6707a.th.entity">Entity</th>
                <th data-i18n="view.s6707a.th.filed">Filed?</th>
                <th data-i18n="view.s6707a.th.penalty">Penalty</th>
                <th data-i18n="view.s6707a.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(t => {
                const pen = penaltyFor(t);
                return `<tr>
                    <td>${esc(t.name)}</td>
                    <td class="muted">${esc(t.category)}</td>
                    <td>$${t.tax_savings_claimed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">${esc(t.entity_type)}</td>
                    <td class="${t.form_8886_filed ? 'pos' : 'neg'}">${t.form_8886_filed ? '✓' : '✗'}</td>
                    <td class="${pen > 0 ? 'neg' : ''}">$${pen.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td><button class="link neg" data-del="${esc(t.id)}" data-i18n="view.s6707a.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.transactions = state.transactions.filter(t => t.id !== btn.dataset.del);
            save(state.transactions);
            render();
        });
    });
}
