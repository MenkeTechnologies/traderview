// Crypto Staking + Airdrop Income Tracker.
// Rev. Rul. 2023-14: staking rewards ORDINARY INCOME at FMV when "dominion and control"
// (typically when received in wallet). Basis = FMV at receipt. Future sale: cap gain over basis.
// Airdrops: ORDINARY income at receipt (Rev. Rul. 2019-24). Hard forks: ORDINARY if new coin received.
// Jarrett v. United States (M.D. Tenn 2023) lost — government settled but precedent unclear.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-crypto-income-v1';

function load() { try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); } catch { return []; } }
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    events: load(),
    year: new Date().getFullYear(),
    marginal_rate: 0.32,
    se_active: false,
};

export async function renderCryptoStaking(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.crypto_inc.h1.title">// CRYPTO STAKING + AIRDROP INCOME</span></h1>
        <p class="muted small" data-i18n="view.crypto_inc.hint.intro">
            <strong>Rev. Rul. 2023-14:</strong> staking rewards = ORDINARY income at FMV when
            "dominion and control" obtained (typically when received in wallet). Basis = FMV at
            receipt. Future sale: cap gain over basis. <strong>Rev. Rul. 2019-24:</strong>
            airdrops + hard forks = ORDINARY income at receipt. <strong>Jarrett v. United States
            (M.D. Tenn 2023):</strong> taxpayer argued staking ≠ income (new property created) —
            government settled, precedent unclear.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.crypto_inc.h2.add">Log income event</h2>
            <form id="crypto_inc-form" class="inline-form">
                <label><span data-i18n="view.crypto_inc.label.date">Date received</span>
                    <input type="date" name="date" required></label>
                <label><span data-i18n="view.crypto_inc.label.kind">Kind</span>
                    <select name="kind">
                        <option value="staking">Staking reward</option>
                        <option value="airdrop">Airdrop</option>
                        <option value="hard_fork">Hard fork</option>
                        <option value="mining">Mining reward</option>
                        <option value="defi_yield">DeFi yield (LP / farming)</option>
                        <option value="lending">Lending interest</option>
                        <option value="referral">Referral / promo bonus</option>
                    </select>
                </label>
                <label><span data-i18n="view.crypto_inc.label.symbol">Symbol</span>
                    <input type="text" name="symbol" required></label>
                <label><span data-i18n="view.crypto_inc.label.units">Units received</span>
                    <input type="number" step="0.00000001" name="units" required></label>
                <label><span data-i18n="view.crypto_inc.label.fmv_per_unit">FMV per unit at receipt ($)</span>
                    <input type="number" step="0.01" name="fmv_per_unit" required></label>
                <label><span data-i18n="view.crypto_inc.label.dominion">Dominion + control obtained?</span>
                    <input type="checkbox" name="dominion_obtained" checked></label>
                <label><span data-i18n="view.crypto_inc.label.platform">Platform / source</span>
                    <input type="text" name="platform" placeholder="Coinbase / Kraken / Lido"></label>
                <button class="primary" type="submit" data-i18n="view.crypto_inc.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.crypto_inc.label.year">Year</span>
                    <input type="number" step="1" id="crypto_inc-year" value="${state.year}"></label>
                <label><span data-i18n="view.crypto_inc.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" id="crypto_inc-marginal" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.crypto_inc.label.se_active">Treat as SE income (Sch C)?</span>
                    <input type="checkbox" id="crypto_inc-se" ${state.se_active ? 'checked' : ''}></label>
            </div>
        </div>
        <div id="crypto_inc-summary"></div>
        <div id="crypto_inc-table" class="chart-panel"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.crypto_inc.h2.basis_tracking">Basis tracking</h2>
            <p class="muted small" data-i18n="view.crypto_inc.basis.body">
                Every staking / airdrop / mining receipt creates BASIS = FMV at receipt + ordinary
                income recognized. When you LATER sell those units, gain = sale proceeds − basis.
                <strong>Tax-software trap:</strong> CoinTracker / Koinly / Crypto.com Tax often
                miscalculate basis when staking auto-restakes. <strong>FIFO default</strong>; HIFO
                possible if specific-identification rules followed (Rev. Proc. 2019-19).
            </p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.crypto_inc.h2.wash_sale_loophole">Crypto wash-sale loophole (until repeal)</h2>
            <p class="muted small" data-i18n="view.crypto_inc.wash.body">
                <strong>Current:</strong> § 1091 wash-sale rule applies only to "stock or securities".
                Crypto = property under Notice 2014-21, NOT a security. → Sell at loss + immediately
                rebuy with no disallowance. Multiple Build Back Better / Biden FY25 budget proposals
                to extend wash-sale to crypto — DID NOT PASS. Loophole alive for now.
            </p>
        </div>
    `;
    document.getElementById('crypto_inc-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const units = Number(fd.get('units')) || 0;
        const fmv = Number(fd.get('fmv_per_unit')) || 0;
        state.events.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            date: fd.get('date'),
            kind: fd.get('kind'),
            symbol: String(fd.get('symbol') || '').toUpperCase(),
            units,
            fmv_per_unit: fmv,
            value: units * fmv,
            dominion_obtained: !!fd.get('dominion_obtained'),
            platform: fd.get('platform') || '',
        });
        save(state.events);
        e.target.reset();
        showToast(t('view.crypto_inc.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('crypto_inc-year').addEventListener('change', e => {
        state.year = Number(e.target.value) || new Date().getFullYear();
        render();
    });
    document.getElementById('crypto_inc-marginal').addEventListener('change', e => {
        state.marginal_rate = Number(e.target.value) || 0.32;
        render();
    });
    document.getElementById('crypto_inc-se').addEventListener('change', e => {
        state.se_active = !!e.target.checked;
        render();
    });
    render();
}

function render() {
    const yearEvents = state.events.filter(e => (e.date || '').startsWith(String(state.year)));
    renderSummary(yearEvents);
    renderTable(yearEvents);
}

function renderSummary(yearEvents) {
    const el = document.getElementById('crypto_inc-summary');
    if (!el) return;
    const totalValue = yearEvents.reduce((s, e) => s + e.value, 0);
    const byKind = new Map();
    for (const e of yearEvents) {
        byKind.set(e.kind, (byKind.get(e.kind) || 0) + e.value);
    }
    const incomeTax = totalValue * state.marginal_rate;
    const seTax = state.se_active ? totalValue * 0.9235 * 0.153 : 0;
    const totalTax = incomeTax + seTax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.crypto_inc.h2.summary">${state.year} summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.crypto_inc.card.events">Events</div>
                    <div class="value">${yearEvents.length}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.crypto_inc.card.total_income">Total ordinary income</div>
                    <div class="value">$${totalValue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.crypto_inc.card.income_tax">Income tax</div>
                    <div class="value">$${incomeTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${state.se_active ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.crypto_inc.card.se_tax">SE tax (15.3%)</div>
                        <div class="value">$${seTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card neg">
                    <div class="label" data-i18n="view.crypto_inc.card.total_tax">Total tax</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(yearEvents) {
    const el = document.getElementById('crypto_inc-table');
    if (!el) return;
    if (!yearEvents.length) {
        el.innerHTML = `<h2 data-i18n="view.crypto_inc.h2.events">Events</h2>
            <p class="muted" data-i18n="view.crypto_inc.empty">No events logged for this year.</p>`;
        return;
    }
    const sorted = [...yearEvents].sort((a, b) => (b.date || '').localeCompare(a.date || ''));
    el.innerHTML = `
        <h2 data-i18n="view.crypto_inc.h2.events">Events</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.crypto_inc.th.date">Date</th>
                <th data-i18n="view.crypto_inc.th.kind">Kind</th>
                <th data-i18n="view.crypto_inc.th.symbol">Symbol</th>
                <th data-i18n="view.crypto_inc.th.units">Units</th>
                <th data-i18n="view.crypto_inc.th.fmv">FMV/unit</th>
                <th data-i18n="view.crypto_inc.th.value">Value</th>
                <th data-i18n="view.crypto_inc.th.dominion">Dominion?</th>
                <th data-i18n="view.crypto_inc.th.platform">Platform</th>
                <th data-i18n="view.crypto_inc.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(e => `
                <tr>
                    <td class="muted">${esc(e.date || '')}</td>
                    <td class="muted">${esc(e.kind)}</td>
                    <td>${esc(e.symbol)}</td>
                    <td>${e.units}</td>
                    <td>$${e.fmv_per_unit.toLocaleString(undefined, { maximumFractionDigits: 2 })}</td>
                    <td>$${e.value.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="${e.dominion_obtained ? 'pos' : 'neg'}">${e.dominion_obtained ? esc(t('view.crypto_inc.status.yes')) : esc(t('view.crypto_inc.status.no'))}</td>
                    <td class="muted">${esc(e.platform || '—')}</td>
                    <td><button class="link neg" data-del="${esc(e.id)}" data-i18n="view.crypto_inc.btn.delete">delete</button></td>
                </tr>
            `).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.events = state.events.filter(e => e.id !== btn.dataset.del);
            save(state.events);
            render();
        });
    });
}
