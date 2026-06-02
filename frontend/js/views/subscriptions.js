// Recurring Subscriptions Tracker — detects monthly/annual subscriptions
// from your existing expense transactions, projects annual cost, and
// flags candidates to cancel (low usage, expensive).
//
// Pure client-side: pulls from /expense/transactions, no backend changes.
// Detection: same merchant + same/very-similar amount appearing on a
// roughly monthly cadence at least 2× = recurring subscription.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { months: 12 };

export async function renderSubscriptions(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.subscriptions.h1.title">// RECURRING SUBSCRIPTIONS</span></h1>
        <p class="muted small" data-i18n="view.subscriptions.hint.intro">
            Auto-detects monthly + annual recurring charges from your expense history.
            Trader subscription stacks (data feeds, charting, news) silently bleed P&amp;L —
            audit here every quarter.
        </p>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.subscriptions.label.window">Look back</span>
                    <select id="sub-months">
                        <option value="6"  ${state.months === 6 ? 'selected' : ''}>6 months</option>
                        <option value="12" ${state.months === 12 ? 'selected' : ''}>12 months</option>
                        <option value="24" ${state.months === 24 ? 'selected' : ''}>24 months</option>
                    </select>
                </label>
                <button class="primary" id="sub-refresh" type="button" data-i18n="view.subscriptions.btn.refresh">Refresh</button>
            </div>
            <div id="sub-summary" style="margin-top:10px"></div>
            <div id="sub-table" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('sub-months').addEventListener('change', e => {
        state.months = Number(e.target.value);
        void load(tok);
    });
    document.getElementById('sub-refresh').addEventListener('click', () => void load(tok));
    await load(tok);
}

async function load(tok) {
    const sumEl = document.getElementById('sub-summary');
    const tblEl = document.getElementById('sub-table');
    if (sumEl) sumEl.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const since = new Date();
        since.setMonth(since.getMonth() - state.months);
        const txns = await api.expenseTransactions({
            from: since.toISOString().slice(0, 10),
            limit: 5000,
        });
        if (!viewIsCurrent(tok)) return;
        const rows = Array.isArray(txns) ? txns : (txns?.transactions || []);
        if (!rows.length) {
            sumEl.innerHTML = `<p class="muted" data-i18n="view.subscriptions.empty">No expense transactions found in window.</p>`;
            if (tblEl) tblEl.innerHTML = '';
            return;
        }
        const subs = detectSubscriptions(rows);
        renderSummary(sumEl, subs);
        renderTable(tblEl, subs);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (sumEl) sumEl.innerHTML = `<p class="muted neg">${esc(t('view.subscriptions.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.subscriptions.toast.failed'), { level: 'error' });
    }
}

// Group transactions by (normalized-merchant, ~amount). Anything that shows
// up at least twice on a roughly monthly cadence is flagged as recurring.
function detectSubscriptions(txns) {
    const buckets = new Map();
    for (const t of txns) {
        const merchant = normalizeMerchant(t.merchant || t.description || '');
        if (!merchant) continue;
        const amt = Math.abs(Number(t.amount) || 0);
        if (amt < 1) continue;
        // Bin amount to nearest dollar — subscription prices rarely change cents.
        const amtBin = Math.round(amt);
        const key = `${merchant}|${amtBin}`;
        if (!buckets.has(key)) buckets.set(key, []);
        buckets.get(key).push({
            ts: new Date(t.date || t.transaction_date || t.posted_at).getTime(),
            amount: amt,
            raw: t,
        });
    }
    const subs = [];
    for (const [key, items] of buckets) {
        if (items.length < 2) continue;
        items.sort((a, b) => a.ts - b.ts);
        // Compute median gap between consecutive charges (days).
        const gaps = [];
        for (let i = 1; i < items.length; i++) {
            gaps.push((items[i].ts - items[i - 1].ts) / 86_400_000);
        }
        gaps.sort((a, b) => a - b);
        const median = gaps[Math.floor(gaps.length / 2)];
        // Accept monthly (25-35d), annual (350-380d), quarterly (85-95d), weekly (6-8d).
        const cadence = classifyCadence(median);
        if (!cadence) continue;
        const [merchant, amtBin] = key.split('|');
        const last = items[items.length - 1];
        const monthlyCost = monthlyCostOf(Number(amtBin), cadence);
        subs.push({
            merchant,
            cadence,
            unitAmount: Number(amtBin),
            monthlyCost,
            annualCost: monthlyCost * 12,
            hitCount: items.length,
            firstSeen: new Date(items[0].ts).toISOString().slice(0, 10),
            lastSeen: new Date(last.ts).toISOString().slice(0, 10),
            sampleDescription: last.raw.description || last.raw.merchant || '—',
            category: last.raw.category || '—',
        });
    }
    subs.sort((a, b) => b.monthlyCost - a.monthlyCost);
    return subs;
}

function normalizeMerchant(s) {
    return String(s).toUpperCase()
        .replace(/[0-9]+/g, ' ')
        .replace(/\W+/g, ' ')
        .trim();
}

function classifyCadence(medianGap) {
    if (medianGap >= 6 && medianGap <= 8)   return 'weekly';
    if (medianGap >= 25 && medianGap <= 35) return 'monthly';
    if (medianGap >= 85 && medianGap <= 95) return 'quarterly';
    if (medianGap >= 350 && medianGap <= 380) return 'annual';
    return null;
}

function monthlyCostOf(amount, cadence) {
    switch (cadence) {
        case 'weekly':    return amount * 4.33;
        case 'monthly':   return amount;
        case 'quarterly': return amount / 3;
        case 'annual':    return amount / 12;
        default:          return 0;
    }
}

function renderSummary(el, subs) {
    if (!el) return;
    if (!subs.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.subscriptions.empty_detected">No recurring subscriptions detected in window.</p>`;
        return;
    }
    const monthlyTotal = subs.reduce((s, x) => s + x.monthlyCost, 0);
    const annualTotal = monthlyTotal * 12;
    el.innerHTML = `
        <div class="cards">
            <div class="card">
                <div class="label" data-i18n="view.subscriptions.card.count">Detected subscriptions</div>
                <div class="value">${subs.length}</div>
            </div>
            <div class="card neg">
                <div class="label" data-i18n="view.subscriptions.card.monthly_total">Total monthly</div>
                <div class="value">$${monthlyTotal.toFixed(2)}</div>
            </div>
            <div class="card neg">
                <div class="label" data-i18n="view.subscriptions.card.annual_total">Annualized</div>
                <div class="value">$${annualTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.subscriptions.card.median_cost">Median sub</div>
                <div class="value">$${(subs[Math.floor(subs.length / 2)]?.monthlyCost || 0).toFixed(2)}/mo</div>
            </div>
        </div>
    `;
}

function renderTable(el, subs) {
    if (!el) return;
    if (!subs.length) {
        el.innerHTML = '';
        return;
    }
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.subscriptions.th.merchant">Merchant</th>
            <th data-i18n="view.subscriptions.th.cadence">Cadence</th>
            <th data-i18n="view.subscriptions.th.unit">Per charge</th>
            <th data-i18n="view.subscriptions.th.monthly">Monthly</th>
            <th data-i18n="view.subscriptions.th.annual">Annual</th>
            <th data-i18n="view.subscriptions.th.hits">Hits</th>
            <th data-i18n="view.subscriptions.th.first">First</th>
            <th data-i18n="view.subscriptions.th.last">Last</th>
            <th data-i18n="view.subscriptions.th.category">Category</th>
        </tr></thead>
        <tbody>${subs.map(s => `
            <tr>
                <td><strong>${esc(s.merchant)}</strong>
                    <div class="muted small">${esc((s.sampleDescription || '').slice(0, 60))}</div></td>
                <td class="muted">${esc(t('view.subscriptions.cadence.' + s.cadence))}</td>
                <td>$${s.unitAmount.toFixed(2)}</td>
                <td class="neg">$${s.monthlyCost.toFixed(2)}</td>
                <td>$${s.annualCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                <td class="muted">${s.hitCount}</td>
                <td class="muted">${esc(s.firstSeen)}</td>
                <td class="muted">${esc(s.lastSeen)}</td>
                <td class="muted">${esc(s.category)}</td>
            </tr>
        `).join('')}</tbody>
    </table>`;
}
