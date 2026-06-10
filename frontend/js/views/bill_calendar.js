// Bill Calendar — recurring bill due-date timeline. Track recurring
// obligations (rent, subscriptions, utilities, debt service), project
// the next N due-dates per bill, and roll up monthly cash-out totals.
// State persists in localStorage; no backend.

import { esc } from '../util.js';
import { tConfirm } from '../dialog.js';

const STORAGE_KEY = 'tv.bill_calendar.v1';

const DEFAULT_BILLS = [
    { id: 'rent', label: 'Rent / Mortgage', amount: 2400, dom: 1, cadence: 'monthly' },
    { id: 'electric', label: 'Electric', amount: 120, dom: 15, cadence: 'monthly' },
    { id: 'internet', label: 'Internet', amount: 80, dom: 7, cadence: 'monthly' },
    { id: 'streaming', label: 'Streaming bundle', amount: 35, dom: 12, cadence: 'monthly' },
    { id: 'auto_ins', label: 'Auto insurance', amount: 540, dom: 5, cadence: 'semiannual' },
    { id: 'amex', label: 'Amex annual fee', amount: 695, dom: 22, cadence: 'annual' },
];

function loadBills() {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) return JSON.parse(raw);
    } catch (_) {}
    return [...DEFAULT_BILLS];
}

function saveBills(bills) {
    try { localStorage.setItem(STORAGE_KEY, JSON.stringify(bills)); } catch (_) {}
}

export async function renderBillCalendar(mount, _state) {
    let bills = loadBills();

    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.bill_calendar.title">// BILL CALENDAR</span></h1>
        <p class="muted small" data-i18n="view.bill_calendar.intro">
            Recurring obligations + projected due-dates for the next 12 months.
            Saved locally — never leaves the browser.
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:8px;flex-wrap:wrap;align-items:end;margin-bottom:12px">
                <label>
                    <div class="muted small">Label</div>
                    <input id="bc-label" type="text" placeholder="Spotify" style="width:160px">
                </label>
                <label>
                    <div class="muted small">Amount $</div>
                    <input id="bc-amount" type="number" step="0.01" min="0" style="width:100px">
                </label>
                <label>
                    <div class="muted small">Day-of-month</div>
                    <input id="bc-dom" type="number" step="1" min="1" max="28" value="1" style="width:80px">
                </label>
                <label>
                    <div class="muted small">Cadence</div>
                    <select id="bc-cadence">
                        <option value="weekly">Weekly</option>
                        <option value="biweekly">Bi-weekly</option>
                        <option value="monthly" selected>Monthly</option>
                        <option value="quarterly">Quarterly</option>
                        <option value="semiannual">Semi-annual</option>
                        <option value="annual">Annual</option>
                    </select>
                </label>
                <button class="btn btn-sm primary" id="bc-add" data-tip="Add this bill to the calendar">+ Add bill</button>
                <button class="btn btn-sm" id="bc-reset" data-tip="Restore the default bill list">↺ Reset to defaults</button>
            </div>
            <div id="bc-totals" class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:8px;margin-bottom:12px"></div>
            <h3 class="section-title">Bills</h3>
            <table class="trades" data-table-key="bc-bills" id="bc-bills">
                <thead><tr>
                    <th>Label</th>
                    <th>Amount</th>
                    <th>Day-of-month</th>
                    <th>Cadence</th>
                    <th>Monthly equiv.</th>
                    <th></th>
                </tr></thead>
                <tbody></tbody>
            </table>
            <h3 class="section-title" style="margin-top:18px">Next 12 months — projected due-dates</h3>
            <div id="bc-timeline" class="muted small"></div>
        </div>
    `;

    const tbody = mount.querySelector('#bc-bills tbody');
    const totals = mount.querySelector('#bc-totals');
    const timeline = mount.querySelector('#bc-timeline');

    function render() {
        tbody.innerHTML = bills.map(b => `
            <tr data-id="${esc(b.id)}">
                <td>${esc(b.label)}</td>
                <td>$${fmt(b.amount, 2)}</td>
                <td>${b.dom}</td>
                <td>${esc(b.cadence)}</td>
                <td>$${fmt(monthlyEquiv(b), 2)}</td>
                <td><button class="btn btn-xs danger" data-action="remove" data-id="${esc(b.id)}">✕</button></td>
            </tr>
        `).join('') || `<tr><td colspan="6" class="muted small">No bills configured.</td></tr>`;

        const monthlyTotal = bills.reduce((s, b) => s + monthlyEquiv(b), 0);
        const annualTotal = monthlyTotal * 12;
        const biggest = bills.slice().sort((a, b) => monthlyEquiv(b) - monthlyEquiv(a))[0];
        totals.innerHTML = `
            <div class="card"><div class="label">Monthly equivalent</div><div class="value">$${fmt(monthlyTotal, 2)}</div></div>
            <div class="card"><div class="label">Annual total</div><div class="value">$${fmt(annualTotal, 2)}</div></div>
            <div class="card"><div class="label">Bill count</div><div class="value">${bills.length}</div></div>
            <div class="card"><div class="label">Largest line</div><div class="value">${biggest ? esc(biggest.label) : '—'}</div><div class="muted small">${biggest ? '$' + fmt(monthlyEquiv(biggest), 2) + '/mo equiv' : ''}</div></div>
        `;

        const months = projectNext12Months(bills);
        timeline.innerHTML = months.map(m => `
            <div style="display:grid;grid-template-columns:120px 100px 1fr;gap:8px;padding:4px 0;border-bottom:1px solid #2228">
                <div><strong>${m.label}</strong></div>
                <div class="${m.total > 0 ? 'neg' : 'muted'}">$${fmt(m.total, 2)}</div>
                <div class="muted small">${m.hits.map(h => `${esc(h.label)} ($${fmt(h.amount, 0)})`).join(' · ')}</div>
            </div>
        `).join('');
    }

    function monthlyEquiv(b) {
        const a = parseFloat(b.amount) || 0;
        switch (b.cadence) {
            case 'weekly':     return a * 52 / 12;
            case 'biweekly':   return a * 26 / 12;
            case 'monthly':    return a;
            case 'quarterly':  return a / 3;
            case 'semiannual': return a / 6;
            case 'annual':     return a / 12;
            default:           return a;
        }
    }

    function projectNext12Months(bills) {
        const now = new Date();
        const out = [];
        for (let i = 0; i < 12; i++) {
            const d = new Date(now.getFullYear(), now.getMonth() + i, 1);
            const monthKey = d.toLocaleString(undefined, { month: 'short', year: 'numeric' });
            const hits = [];
            for (const b of bills) {
                const hit = hitsThisMonth(b, d, i);
                if (hit) hits.push({ label: b.label, amount: parseFloat(b.amount) || 0 });
            }
            const total = hits.reduce((s, h) => s + h.amount, 0);
            out.push({ label: monthKey, total, hits });
        }
        return out;
    }

    function hitsThisMonth(b, dateOfFirst, monthIdx) {
        switch (b.cadence) {
            case 'weekly':
            case 'biweekly':
            case 'monthly':    return true;
            case 'quarterly':  return monthIdx % 3 === 0;
            case 'semiannual': return monthIdx % 6 === 0;
            case 'annual':     return monthIdx === 0;
            default:           return true;
        }
    }

    mount.querySelector('#bc-add').addEventListener('click', () => {
        const label = mount.querySelector('#bc-label').value.trim();
        const amount = parseFloat(mount.querySelector('#bc-amount').value);
        const dom = parseInt(mount.querySelector('#bc-dom').value, 10) || 1;
        const cadence = mount.querySelector('#bc-cadence').value;
        if (!label || !(amount > 0)) return;
        bills.push({ id: 'b_' + Math.floor(performance.now()), label, amount, dom, cadence });
        saveBills(bills);
        mount.querySelector('#bc-label').value = '';
        mount.querySelector('#bc-amount').value = '';
        render();
    });

    mount.querySelector('#bc-reset').addEventListener('click', async () => {
        const ok = await tConfirm('view.bill_calendar.confirm.reset');
        if (!ok) return;
        bills = [...DEFAULT_BILLS];
        saveBills(bills);
        render();
    });

    tbody.addEventListener('click', (ev) => {
        const btn = ev.target.closest('button[data-action="remove"]');
        if (!btn) return;
        const id = btn.dataset.id;
        bills = bills.filter(b => b.id !== id);
        saveBills(bills);
        render();
    });

    render();
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
