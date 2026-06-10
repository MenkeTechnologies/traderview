// Cash Flow Forecast — projects a bank balance N months out given
// starting balance, monthly income, recurring bills (from Bill Calendar
// localStorage if present), and ad-hoc upcoming inflows/outflows.

import { esc } from '../util.js';

const ADHOC_KEY = 'tv.cash_flow.adhoc.v1';
const SETTINGS_KEY = 'tv.cash_flow.settings.v1';
const BILLS_KEY = 'tv.bill_calendar.v1';

function loadSettings() {
    try {
        const raw = localStorage.getItem(SETTINGS_KEY);
        if (raw) return JSON.parse(raw);
    } catch (_) {}
    return { starting: 25000, monthlyIncome: 8500, monthlyDiscretionary: 1200, horizon: 12 };
}

function saveSettings(s) {
    try { localStorage.setItem(SETTINGS_KEY, JSON.stringify(s)); } catch (_) {}
}

function loadAdhoc() {
    try {
        const raw = localStorage.getItem(ADHOC_KEY);
        if (raw) return JSON.parse(raw);
    } catch (_) {}
    return [];
}

function saveAdhoc(rows) {
    try { localStorage.setItem(ADHOC_KEY, JSON.stringify(rows)); } catch (_) {}
}

function loadBills() {
    try {
        const raw = localStorage.getItem(BILLS_KEY);
        if (raw) return JSON.parse(raw);
    } catch (_) {}
    return [];
}

export async function renderCashFlowForecast(mount, _state) {
    let settings = loadSettings();
    let adhoc = loadAdhoc();

    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cash_flow_forecast.title">// CASH-FLOW FORECAST</span></h1>
        <p class="muted small" data-i18n-html="view.cash_flow_forecast.intro">
            Projects bank balance N months out. Pulls recurring bills from the
            <strong>Bill Calendar</strong> view if you've configured it, adds
            monthly income, and lets you stack one-off inflows/outflows.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Starting balance $</span>
                    <input type="number" id="cf-starting" step="100" value="${settings.starting}" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Monthly income (net) $</span>
                    <input type="number" id="cf-income" step="100" value="${settings.monthlyIncome}" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Discretionary / mo $</span>
                    <input type="number" id="cf-disc" step="50" value="${settings.monthlyDiscretionary}" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Horizon (months)</span>
                    <input type="number" id="cf-horizon" step="1" min="1" max="60" value="${settings.horizon}" style="width:100%">
                </label>
            </div>
            <h3 class="section-title">One-off events</h3>
            <div style="display:flex;gap:8px;flex-wrap:wrap;align-items:end;margin-bottom:8px">
                <label>
                    <div class="muted small">Month-offset (0=this month)</div>
                    <input id="cf-ah-month" type="number" step="1" min="0" max="60" value="1" style="width:120px">
                </label>
                <label>
                    <div class="muted small">Amount $ (negative = outflow)</div>
                    <input id="cf-ah-amount" type="number" step="100" placeholder="-4500" style="width:140px">
                </label>
                <label>
                    <div class="muted small">Label</div>
                    <input id="cf-ah-label" type="text" placeholder="Property tax" style="width:200px">
                </label>
                <button class="btn btn-sm primary" id="cf-ah-add">+ Add event</button>
            </div>
            <table class="trades" data-table-key="cf-adhoc" id="cf-adhoc">
                <thead><tr><th>Month</th><th>Label</th><th>Amount</th><th></th></tr></thead>
                <tbody></tbody>
            </table>

            <h3 class="section-title" style="margin-top:18px">Forecast</h3>
            <div id="cf-totals" class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px"></div>
            <table class="trades" data-table-key="cf-projection">
                <thead><tr>
                    <th>Month</th>
                    <th>Open</th>
                    <th>Income</th>
                    <th>Bills (recurring)</th>
                    <th>Discretionary</th>
                    <th>One-offs</th>
                    <th>Net</th>
                    <th>Close</th>
                </tr></thead>
                <tbody id="cf-projection"></tbody>
            </table>
        </div>
    `;

    const tbody = mount.querySelector('#cf-adhoc tbody');
    const projTbody = mount.querySelector('tbody#cf-projection');
    const totals = mount.querySelector('#cf-totals');

    function renderAdhoc() {
        tbody.innerHTML = adhoc.length
            ? adhoc.map(e => `
                <tr>
                    <td>+${e.month}</td>
                    <td>${esc(e.label || '')}</td>
                    <td class="${e.amount >= 0 ? 'pos' : 'neg'}">${e.amount >= 0 ? '+' : ''}$${fmt(e.amount, 2)}</td>
                    <td><button class="btn btn-xs danger" data-id="${esc(e.id)}">✕</button></td>
                </tr>
            `).join('')
            : `<tr><td colspan="4" class="muted small">No one-offs.</td></tr>`;
    }

    function recurringMonthlyBills(monthIdx) {
        const bills = loadBills();
        let sum = 0;
        for (const b of bills) {
            const a = parseFloat(b.amount) || 0;
            switch (b.cadence) {
                case 'weekly':     sum += a * 52 / 12; break;
                case 'biweekly':   sum += a * 26 / 12; break;
                case 'monthly':    sum += a; break;
                case 'quarterly':  if (monthIdx % 3 === 0) sum += a; break;
                case 'semiannual': if (monthIdx % 6 === 0) sum += a; break;
                case 'annual':     if (monthIdx === 0) sum += a; break;
                default:           sum += a; break;
            }
        }
        return sum;
    }

    function renderProjection() {
        settings = {
            starting: parseFloat(mount.querySelector('#cf-starting').value) || 0,
            monthlyIncome: parseFloat(mount.querySelector('#cf-income').value) || 0,
            monthlyDiscretionary: parseFloat(mount.querySelector('#cf-disc').value) || 0,
            horizon: Math.max(1, Math.min(60, parseInt(mount.querySelector('#cf-horizon').value, 10) || 12)),
        };
        saveSettings(settings);

        let balance = settings.starting;
        let minBalance = balance;
        let minMonth = 'now';
        const rows = [];
        const now = new Date();
        for (let i = 0; i < settings.horizon; i++) {
            const open = balance;
            const income = settings.monthlyIncome;
            const bills = recurringMonthlyBills(i);
            const disc = settings.monthlyDiscretionary;
            const oneOffs = adhoc.filter(a => a.month === i).reduce((s, a) => s + (parseFloat(a.amount) || 0), 0);
            const net = income - bills - disc + oneOffs;
            balance = open + net;
            const monthLabel = new Date(now.getFullYear(), now.getMonth() + i, 1)
                .toLocaleString(undefined, { month: 'short', year: 'numeric' });
            if (balance < minBalance) { minBalance = balance; minMonth = monthLabel; }
            rows.push({ monthLabel, open, income, bills, disc, oneOffs, net, close: balance });
        }
        projTbody.innerHTML = rows.map(r => `
            <tr>
                <td>${esc(r.monthLabel)}</td>
                <td>$${fmt(r.open, 0)}</td>
                <td class="pos">+$${fmt(r.income, 0)}</td>
                <td class="neg">-$${fmt(r.bills, 0)}</td>
                <td class="neg">-$${fmt(r.disc, 0)}</td>
                <td class="${r.oneOffs >= 0 ? 'pos' : 'neg'}">${r.oneOffs >= 0 ? '+' : ''}$${fmt(r.oneOffs, 0)}</td>
                <td class="${r.net >= 0 ? 'pos' : 'neg'}"><strong>${r.net >= 0 ? '+' : ''}$${fmt(r.net, 0)}</strong></td>
                <td class="${r.close < 0 ? 'neg' : ''}"><strong>$${fmt(r.close, 0)}</strong></td>
            </tr>
        `).join('');

        const ending = rows[rows.length - 1]?.close ?? settings.starting;
        const totalNet = ending - settings.starting;
        const monthsToZero = rows.findIndex(r => r.close < 0);
        totals.innerHTML = `
            <div class="card"><div class="label">Starting balance</div><div class="value">$${fmt(settings.starting, 0)}</div></div>
            <div class="card"><div class="label">Projected ending (mo ${settings.horizon})</div><div class="value ${ending >= 0 ? '' : 'neg'}">$${fmt(ending, 0)}</div></div>
            <div class="card"><div class="label">Net change</div><div class="value ${totalNet >= 0 ? 'pos' : 'neg'}">${totalNet >= 0 ? '+' : ''}$${fmt(totalNet, 0)}</div></div>
            <div class="card"><div class="label">Min balance</div><div class="value ${minBalance < 0 ? 'neg' : ''}">$${fmt(minBalance, 0)}</div><div class="muted small">${esc(minMonth)}</div></div>
            <div class="card"><div class="label">Cash crunch?</div><div class="value ${monthsToZero >= 0 ? 'neg' : 'pos'}">${monthsToZero >= 0 ? 'YES — month ' + (monthsToZero + 1) : 'no'}</div></div>
        `;
    }

    function rerender() { renderAdhoc(); renderProjection(); }

    mount.querySelectorAll('#cf-starting, #cf-income, #cf-disc, #cf-horizon').forEach(el => {
        el.addEventListener('input', renderProjection);
    });

    mount.querySelector('#cf-ah-add').addEventListener('click', () => {
        const month = parseInt(mount.querySelector('#cf-ah-month').value, 10);
        const amount = parseFloat(mount.querySelector('#cf-ah-amount').value);
        const label = mount.querySelector('#cf-ah-label').value.trim();
        if (!Number.isFinite(month) || !Number.isFinite(amount)) return;
        adhoc.push({ id: 'e_' + Math.floor(performance.now()), month, amount, label });
        saveAdhoc(adhoc);
        mount.querySelector('#cf-ah-amount').value = '';
        mount.querySelector('#cf-ah-label').value = '';
        rerender();
    });

    tbody.addEventListener('click', (ev) => {
        const btn = ev.target.closest('button[data-id]');
        if (!btn) return;
        adhoc = adhoc.filter(e => e.id !== btn.dataset.id);
        saveAdhoc(adhoc);
        rerender();
    });

    rerender();
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
