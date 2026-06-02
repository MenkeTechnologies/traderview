// Gift Tax Exclusion Tracker — IRC § 2503(b) annual + § 2010 lifetime.
// 2024: $18,000/recipient/year annual exclusion. MFJ split-gift = $36,000.
// Lifetime: $13.61M (2024) — unified with estate tax. Any gift > annual
// exclusion triggers Form 709 filing (even if no tax due, eats lifetime).
// 529 plan superfund: $90k single / $180k MFJ in one year, treated as 5×$18k.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-gifts-v1';
const LIMITS = {
    2024: { annual: 18_000, lifetime: 13_610_000 },
    2025: { annual: 19_000, lifetime: 13_990_000 },
    2026: { annual: 19_000, lifetime: 7_000_000 },  // POST-TCJA SUNSET — major cliff
};

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    gifts: load(),
    year: new Date().getFullYear(),
    is_mfj_split: false,
    cumulative_lifetime_used: 0,
};

export async function renderGiftTax(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.gift.h1.title">// GIFT TAX EXCLUSION TRACKER</span></h1>
        <p class="muted small" data-i18n="view.gift.hint.intro">
            2024: $18,000/recipient/year exclusion ($36k MFJ split-gift). Lifetime
            $13.61M unified with estate tax. Gifts &gt; annual exclusion trigger
            <strong>Form 709</strong> (even when no tax due). 529 superfund: $90k single /
            $180k MFJ in one year, 5-year spread. <strong>2026 cliff: lifetime drops to ~$7M.</strong>
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.gift.h2.add">Log gift</h2>
            <form id="gt-form" class="inline-form">
                <label><span data-i18n="view.gift.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}" required></label>
                <label><span data-i18n="view.gift.label.recipient">Recipient</span>
                    <input type="text" name="recipient" required></label>
                <label><span data-i18n="view.gift.label.relationship">Relationship</span>
                    <input type="text" name="relationship" placeholder="Daughter / friend / charity"></label>
                <label><span data-i18n="view.gift.label.amount">Amount ($)</span>
                    <input type="number" step="100" name="amount" required></label>
                <label><span data-i18n="view.gift.label.gift_type">Type</span>
                    <select name="gift_type">
                        <option value="cash">Cash</option>
                        <option value="stock">Appreciated stock</option>
                        <option value="529_superfund">529 superfund (5-yr)</option>
                        <option value="real_estate">Real estate</option>
                        <option value="other">Other</option>
                    </select>
                </label>
                <label><span data-i18n="view.gift.label.is_split">Split with spouse (MFJ)?</span>
                    <input type="checkbox" name="is_split"></label>
                <button class="primary" type="submit" data-i18n="view.gift.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.gift.label.view_year">View year</span>
                    <input type="number" id="gt-year" value="${state.year}"></label>
                <label><span data-i18n="view.gift.label.lifetime_used">Lifetime exemption already used ($)</span>
                    <input type="number" id="gt-lifetime" step="1000" value="${state.cumulative_lifetime_used}"></label>
            </div>
        </div>
        <div id="gt-summary"></div>
        <div id="gt-table" class="chart-panel"></div>
    `;
    document.getElementById('gt-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const g = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            year: Number(fd.get('year')),
            recipient: fd.get('recipient'),
            relationship: fd.get('relationship'),
            amount: Number(fd.get('amount')),
            gift_type: fd.get('gift_type'),
            is_split: !!fd.get('is_split'),
        };
        state.gifts.push(g);
        save(state.gifts);
        e.target.reset();
        e.target.querySelector('[name="year"]').value = state.year;
        showToast(t('view.gift.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('gt-year').addEventListener('change', e => {
        state.year = Number(e.target.value) || new Date().getFullYear();
        render();
    });
    document.getElementById('gt-lifetime').addEventListener('change', e => {
        state.cumulative_lifetime_used = Number(e.target.value) || 0;
        render();
    });
    render();
}

function render() {
    const yearGifts = state.gifts.filter(g => g.year === state.year);
    renderSummary(yearGifts);
    renderTable(yearGifts);
}

function renderSummary(yearGifts) {
    const el = document.getElementById('gt-summary');
    if (!el) return;
    const limits = LIMITS[state.year] || LIMITS[2024];
    // Aggregate by recipient
    const byRecipient = new Map();
    for (const g of yearGifts) {
        const k = g.recipient;
        const effectiveExclusion = g.is_split ? limits.annual * 2 : limits.annual;
        const r = byRecipient.get(k) || { total: 0, exclusion: effectiveExclusion, gifts: 0 };
        r.total += g.amount;
        r.exclusion = effectiveExclusion;
        r.gifts++;
        byRecipient.set(k, r);
    }
    const recipientStats = [...byRecipient.entries()].map(([k, v]) => ({
        recipient: k,
        ...v,
        over: Math.max(0, v.total - v.exclusion),
        form_709_needed: v.total > v.exclusion,
    }));
    const totalOverAnnual = recipientStats.reduce((s, r) => s + r.over, 0);
    const lifetimeRemaining = Math.max(0, limits.lifetime - state.cumulative_lifetime_used - totalOverAnnual);
    const anyForm709 = recipientStats.some(r => r.form_709_needed);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.gift.h2.summary">${state.year} gift summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.gift.card.recipients">Recipients</div>
                    <div class="value">${recipientStats.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.gift.card.total_given">Total given</div>
                    <div class="value">$${yearGifts.reduce((s, g) => s + g.amount, 0).toLocaleString()}</div>
                </div>
                <div class="card ${totalOverAnnual > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.gift.card.over_exclusion">Over annual exclusion</div>
                    <div class="value">$${totalOverAnnual.toLocaleString()}</div>
                </div>
                <div class="card ${anyForm709 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.gift.card.form_709_required">Form 709 needed?</div>
                    <div class="value">${anyForm709 ? esc(t('view.gift.status.yes')) : esc(t('view.gift.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.gift.card.lifetime_remaining">Lifetime exemption remaining</div>
                    <div class="value">$${lifetimeRemaining.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.year >= 2025 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.gift.warning.sunset">
                    2026 SUNSET: lifetime exemption drops from ~$13.6M to ~$7M unless Congress acts.
                    Consider using up exemption pre-2026 if you're estate-tax-exposed.
                </p>
            ` : ''}
        </div>
    `;
}

function renderTable(yearGifts) {
    const el = document.getElementById('gt-table');
    if (!el) return;
    if (!yearGifts.length) {
        el.innerHTML = `<h2 data-i18n="view.gift.h2.gifts">Gifts</h2>
            <p class="muted" data-i18n="view.gift.empty">No gifts recorded for this year.</p>`;
        return;
    }
    const limits = LIMITS[state.year] || LIMITS[2024];
    const sorted = [...yearGifts].sort((a, b) => b.amount - a.amount);
    el.innerHTML = `
        <h2 data-i18n="view.gift.h2.gifts">Gifts</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.gift.th.recipient">Recipient</th>
                <th data-i18n="view.gift.th.relationship">Relationship</th>
                <th data-i18n="view.gift.th.amount">Amount</th>
                <th data-i18n="view.gift.th.type">Type</th>
                <th data-i18n="view.gift.th.split">Split?</th>
                <th data-i18n="view.gift.th.exclusion">Exclusion</th>
                <th data-i18n="view.gift.th.status">Status</th>
                <th data-i18n="view.gift.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(g => {
                const exclusion = g.is_split ? limits.annual * 2 : limits.annual;
                const over = Math.max(0, g.amount - exclusion);
                const cls = over > 0 ? 'neg' : 'pos';
                const status = over > 0
                    ? t('view.gift.status.form_709', { over: over.toLocaleString() })
                    : t('view.gift.status.ok');
                return `<tr>
                    <td>${esc(g.recipient)}</td>
                    <td class="muted">${esc(g.relationship || '—')}</td>
                    <td>$${g.amount.toLocaleString()}</td>
                    <td class="muted">${esc(g.gift_type)}</td>
                    <td>${g.is_split ? '✓' : ''}</td>
                    <td class="muted">$${exclusion.toLocaleString()}</td>
                    <td class="${cls}">${esc(status)}</td>
                    <td><button class="link neg" data-del="${esc(g.id)}" data-i18n="view.gift.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.gifts = state.gifts.filter(g => g.id !== btn.dataset.del);
            save(state.gifts);
            render();
        });
    });
}
