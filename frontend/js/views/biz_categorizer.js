// Business vs Personal Expense Categorizer — rule-based auto-tag.
// Defines keyword + amount patterns that mark transactions business vs
// personal, runs them across your expense history, and surfaces the matches.
// Pure client-side: pulls from /expense/transactions, doesn't modify them.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-biz-rules-v1';

const DEFAULT_RULES = [
    { id: 'r1',  pattern: 'webull|ibkr|tradestation|fidelity|schwab|tastytrade', tag: 'business', reason: 'Brokerage' },
    { id: 'r2',  pattern: 'finnhub|polygon|alpaca|trade.?ideas|benzinga|warrior|daytradedash', tag: 'business', reason: 'Data feed / education' },
    { id: 'r3',  pattern: 'aws|gcp|google.cloud|digital.ocean|linode|vultr|hetzner|azure', tag: 'business', reason: 'Cloud hosting' },
    { id: 'r4',  pattern: 'github|gitlab|bitbucket|jetbrains|rustrover|claude|openai|anthropic|chatgpt', tag: 'business', reason: 'Developer tools' },
    { id: 'r5',  pattern: 'tradingview|thinkorswim|optionstrat|barchart|stocktwits', tag: 'business', reason: 'Charting / analysis' },
    { id: 'r6',  pattern: 'cpa|accountant|tax pro|bookkeep|gusto|adp', tag: 'business', reason: 'Tax / payroll' },
    { id: 'r7',  pattern: 'staples|office depot|amazon.*supplies|ups store|fedex', tag: 'business', reason: 'Office supplies' },
    { id: 'r8',  pattern: 'wework|regus|coworking|industrious', tag: 'business', reason: 'Co-working' },
    { id: 'r9',  pattern: 'verizon|comcast|att|t.?mobile|spectrum.*business', tag: 'business', reason: 'Business internet/phone' },
    { id: 'r10', pattern: 'kroger|wholefoods|safeway|trader.joe|costco|sam.s club', tag: 'personal', reason: 'Groceries' },
    { id: 'r11', pattern: 'mcdonald|chipotle|starbucks|dunkin|chickfila|panera', tag: 'personal', reason: 'Fast food (likely personal)' },
    { id: 'r12', pattern: 'netflix|hulu|spotify|disney|hbo|paramount|prime video', tag: 'personal', reason: 'Streaming' },
    { id: 'r13', pattern: 'shell|chevron|exxon|bp|sunoco|wawa', tag: 'ambiguous', reason: 'Fuel — split by use' },
    { id: 'r14', pattern: 'uber|lyft|taxi', tag: 'ambiguous', reason: 'Rideshare — split by purpose' },
    { id: 'r15', pattern: 'apple|google|microsoft|adobe', tag: 'ambiguous', reason: 'Software — could be either' },
];

function loadRules() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || 'null') || DEFAULT_RULES; }
    catch { return DEFAULT_RULES; }
}
function saveRules(r) { try { localStorage.setItem(LS_KEY, JSON.stringify(r)); } catch { /* ignore */ } }

let state = {
    rules: loadRules(),
    txns: [],
    months: 12,
};

export async function renderBizCategorizer(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.bizcat.h1.title">// BUSINESS / PERSONAL CATEGORIZER</span></h1>
        <p class="muted small" data-i18n="view.bizcat.hint.intro">
            Regex-based auto-tagging of expense transactions. Edit the rules below
            to match your stack. "Ambiguous" tag flags transactions you need to
            split manually (fuel %, software dual-use, etc.).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.bizcat.h2.rules">Rules</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.bizcat.th.pattern">Pattern (regex)</th>
                    <th data-i18n="view.bizcat.th.tag">Tag</th>
                    <th data-i18n="view.bizcat.th.reason">Reason</th>
                    <th data-i18n="view.bizcat.th.actions">Actions</th>
                </tr></thead>
                <tbody id="bc-rules"></tbody>
            </table>
            <form id="bc-add-rule" class="inline-form" style="margin-top:10px">
                <label><span data-i18n="view.bizcat.label.pattern">Pattern</span>
                    <input type="text" name="pattern" required></label>
                <label><span data-i18n="view.bizcat.label.tag">Tag</span>
                    <select name="tag">
                        <option value="business">business</option>
                        <option value="personal">personal</option>
                        <option value="ambiguous">ambiguous</option>
                    </select>
                </label>
                <label><span data-i18n="view.bizcat.label.reason">Reason</span>
                    <input type="text" name="reason"></label>
                <button class="primary" type="submit" data-i18n="view.bizcat.btn.add_rule">Add rule</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.bizcat.label.window">Window (months)</span>
                    <select id="bc-window">
                        <option value="3"  ${state.months === 3  ? 'selected' : ''}>3 months</option>
                        <option value="6"  ${state.months === 6  ? 'selected' : ''}>6 months</option>
                        <option value="12" ${state.months === 12 ? 'selected' : ''}>12 months</option>
                        <option value="24" ${state.months === 24 ? 'selected' : ''}>24 months</option>
                    </select>
                </label>
                <button class="primary" id="bc-scan" type="button" data-i18n="view.bizcat.btn.scan">Scan expenses</button>
            </div>
        </div>
        <div id="bc-summary"></div>
        <div id="bc-table" class="chart-panel"></div>
    `;
    document.getElementById('bc-add-rule').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.rules.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            pattern: fd.get('pattern'),
            tag: fd.get('tag'),
            reason: fd.get('reason') || '',
        });
        saveRules(state.rules);
        e.target.reset();
        renderRules();
    });
    document.getElementById('bc-window').addEventListener('change', e => {
        state.months = Number(e.target.value);
    });
    document.getElementById('bc-scan').addEventListener('click', () => void scan(tok));
    renderRules();
    await scan(tok);
}

function renderRules() {
    const el = document.getElementById('bc-rules');
    if (!el) return;
    el.innerHTML = state.rules.map(r => `
        <tr>
            <td><code>${esc(r.pattern)}</code></td>
            <td class="${tagCls(r.tag)}">${esc(r.tag)}</td>
            <td class="muted">${esc(r.reason || '')}</td>
            <td><button class="link neg" data-del="${esc(r.id)}" data-i18n="view.bizcat.btn.delete">delete</button></td>
        </tr>
    `).join('');
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.rules = state.rules.filter(r => r.id !== btn.dataset.del);
            saveRules(state.rules);
            renderRules();
        });
    });
}

function tagCls(tag) {
    if (tag === 'business') return 'pos';
    if (tag === 'personal') return 'neg';
    return 'muted';
}

async function scan(tok) {
    const summary = document.getElementById('bc-summary');
    const tableEl = document.getElementById('bc-table');
    if (summary) summary.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const since = new Date();
        since.setMonth(since.getMonth() - state.months);
        const txns = await api.expenseTransactions({
            from: since.toISOString().slice(0, 10),
            limit: 5000,
        });
        if (!viewIsCurrent(tok)) return;
        const rows = Array.isArray(txns) ? txns : (txns?.transactions || []);
        state.txns = rows;
        const tagged = rows.map(t => {
            const haystack = [(t.merchant || ''), (t.description || ''), (t.category || '')]
                .join(' ').toLowerCase();
            const hit = state.rules.find(r => {
                try { return new RegExp(r.pattern, 'i').test(haystack); }
                catch { return false; }
            });
            return { ...t, _tag: hit?.tag || 'untagged', _reason: hit?.reason || '' };
        });
        renderSummary(summary, tagged);
        renderTable(tableEl, tagged);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (summary) summary.innerHTML = `<p class="muted neg">${esc(t('view.bizcat.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.bizcat.toast.failed'), { level: 'error' });
    }
}

function renderSummary(el, tagged) {
    if (!el) return;
    const sum = tagged.reduce((acc, t) => {
        const amt = Math.abs(Number(t.amount) || 0);
        acc[t._tag] = (acc[t._tag] || 0) + amt;
        return acc;
    }, {});
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.bizcat.h2.summary">Spend by tag</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.bizcat.card.business">Business</div>
                    <div class="value">$${(sum.business || 0).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.bizcat.card.personal">Personal</div>
                    <div class="value">$${(sum.personal || 0).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.bizcat.card.ambiguous">Ambiguous</div>
                    <div class="value">$${(sum.ambiguous || 0).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.bizcat.card.untagged">Untagged</div>
                    <div class="value">$${(sum.untagged || 0).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.bizcat.card.transactions">Transactions</div>
                    <div class="value">${tagged.length}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(el, tagged) {
    if (!el) return;
    if (!tagged.length) {
        el.innerHTML = `<h2 data-i18n="view.bizcat.h2.transactions">Tagged transactions</h2>
            <p class="muted" data-i18n="view.bizcat.empty">No transactions in window.</p>`;
        return;
    }
    // Sort untagged + ambiguous to the top so they're easier to triage.
    const order = { untagged: 0, ambiguous: 1, personal: 2, business: 3 };
    const sorted = [...tagged].sort((a, b) =>
        (order[a._tag] - order[b._tag])
        || (Math.abs(Number(b.amount) || 0) - Math.abs(Number(a.amount) || 0)));
    el.innerHTML = `
        <h2 data-i18n="view.bizcat.h2.transactions">Tagged transactions</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.bizcat.th.date">Date</th>
                <th data-i18n="view.bizcat.th.merchant">Merchant</th>
                <th data-i18n="view.bizcat.th.amount">Amount</th>
                <th data-i18n="view.bizcat.th.tag_2">Tag</th>
                <th data-i18n="view.bizcat.th.reason_2">Reason</th>
                <th data-i18n="view.bizcat.th.category">Category</th>
            </tr></thead>
            <tbody>${sorted.slice(0, 500).map(t => `
                <tr>
                    <td class="muted">${esc((t.date || '').slice(0, 10))}</td>
                    <td>${esc(t.merchant || t.description || '—')}</td>
                    <td>$${Math.abs(Number(t.amount) || 0).toFixed(2)}</td>
                    <td class="${tagCls(t._tag)}">${esc(t._tag)}</td>
                    <td class="muted">${esc(t._reason || '')}</td>
                    <td class="muted">${esc(t.category || '')}</td>
                </tr>
            `).join('')}</tbody>
        </table>
    `;
}
