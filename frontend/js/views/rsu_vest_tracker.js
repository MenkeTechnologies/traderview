// RSU (Restricted Stock Unit) Vest Tracker.
// Each vest event: FMV at vest = ordinary income (W-2 box 1), supplemental
// W/H 22% (37% > $1M). Default: sell-to-cover at vest. New basis = FMV at
// vest. Subsequent sale: gain/loss = sale - vest FMV, ST or LT by hold.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-rsu-v1';

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    vests: load(),
    year: new Date().getFullYear(),
    marginal_rate: 0.32,
    lt_cap_gains_rate: 0.20,
    niit: 0.038,
};

export async function renderRsuVestTracker(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rsu.h1.title">// RSU VEST TRACKER</span></h1>
        <p class="muted small" data-i18n="view.rsu.hint.intro">
            Each vest: FMV at vest = ordinary income (W-2 box 1), supplemental W/H 22%
            (37% over $1M YTD). Default: sell-to-cover at vest, you keep the rest.
            Basis = FMV at vest. Subsequent sale: gain or loss from vest FMV.
            <strong>The 22% supplemental W/H is often insufficient</strong> at high
            incomes — make estimated payments to avoid April surprise.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.rsu.h2.add">Log vest</h2>
            <form id="rsu-form" class="inline-form">
                <label><span data-i18n="view.rsu.label.vest_date">Vest date</span>
                    <input type="date" name="vest_date" required></label>
                <label><span data-i18n="view.rsu.label.shares_vested">Shares vested</span>
                    <input type="number" step="1" name="shares_vested" required></label>
                <label><span data-i18n="view.rsu.label.fmv_per_share">FMV at vest ($/share)</span>
                    <input type="number" step="0.01" name="fmv_per_share" required></label>
                <label><span data-i18n="view.rsu.label.shares_withheld">Shares withheld (sell-to-cover)</span>
                    <input type="number" step="1" name="shares_withheld" value="0"></label>
                <label><span data-i18n="view.rsu.label.fed_w_h">Federal W/H ($)</span>
                    <input type="number" step="100" name="fed_w_h" value="0"></label>
                <label><span data-i18n="view.rsu.label.fica_w_h">FICA W/H ($)</span>
                    <input type="number" step="100" name="fica_w_h" value="0"></label>
                <label><span data-i18n="view.rsu.label.state_w_h">State W/H ($)</span>
                    <input type="number" step="100" name="state_w_h" value="0"></label>
                <button class="primary" type="submit" data-i18n="view.rsu.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.rsu.h2.context">Tax context</h2>
            <form id="rsu-tax" class="inline-form">
                <label><span data-i18n="view.rsu.label.year">View year</span>
                    <input type="number" id="rsu-year" value="${state.year}"></label>
                <label><span data-i18n="view.rsu.label.marginal_rate">Marginal federal %</span>
                    <input type="number" step="0.5" name="marginal_rate" value="${(state.marginal_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.rsu.btn.update">Update</button>
            </form>
        </div>
        <div id="rsu-summary"></div>
        <div id="rsu-table" class="chart-panel"></div>
    `;
    document.getElementById('rsu-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const v = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            vest_date: fd.get('vest_date'),
            shares_vested: Number(fd.get('shares_vested')),
            fmv_per_share: Number(fd.get('fmv_per_share')),
            shares_withheld: Number(fd.get('shares_withheld')) || 0,
            fed_w_h: Number(fd.get('fed_w_h')) || 0,
            fica_w_h: Number(fd.get('fica_w_h')) || 0,
            state_w_h: Number(fd.get('state_w_h')) || 0,
        };
        state.vests.push(v);
        save(state.vests);
        e.target.reset();
        showToast(t('view.rsu.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('rsu-tax').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.year = Number(document.getElementById('rsu-year').value) || new Date().getFullYear();
        state.marginal_rate = (Number(fd.get('marginal_rate')) || 32) / 100;
        render();
    });
    render();
}

function render() {
    const yearVests = state.vests.filter(v => new Date(v.vest_date).getFullYear() === state.year);
    renderSummary(yearVests);
    renderTable(yearVests);
}

function renderSummary(yearVests) {
    const el = document.getElementById('rsu-summary');
    if (!el) return;
    const ordinaryIncome = yearVests.reduce((s, v) => s + v.shares_vested * v.fmv_per_share, 0);
    const totalFedWh = yearVests.reduce((s, v) => s + v.fed_w_h, 0);
    const totalFicaWh = yearVests.reduce((s, v) => s + v.fica_w_h, 0);
    const totalStateWh = yearVests.reduce((s, v) => s + v.state_w_h, 0);
    const totalWh = totalFedWh + totalFicaWh + totalStateWh;
    const actualFedTax = ordinaryIncome * state.marginal_rate;
    const fedShortfall = Math.max(0, actualFedTax - totalFedWh);
    const sharesNet = yearVests.reduce((s, v) => s + (v.shares_vested - v.shares_withheld), 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.rsu.h2.summary">${state.year} summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.rsu.card.events">Vest events</div>
                    <div class="value">${yearVests.length}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.rsu.card.ordinary_income">Ordinary income</div>
                    <div class="value">$${ordinaryIncome.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.rsu.card.shares_net">Shares net (after withhold)</div>
                    <div class="value">${sharesNet.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.rsu.card.fed_wh">Federal W/H</div>
                    <div class="value">$${totalFedWh.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.rsu.card.actual_fed_tax">Actual federal tax @ ${(state.marginal_rate * 100).toFixed(0)}%</div>
                    <div class="value">$${actualFedTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${fedShortfall > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.rsu.card.fed_shortfall">Federal shortfall</div>
                    <div class="value">$${fedShortfall.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.rsu.card.total_wh">Total W/H (Fed+FICA+State)</div>
                    <div class="value">$${totalWh.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            <p class="muted small" style="margin-top:10px" data-i18n="view.rsu.note">
                If federal shortfall &gt; $1,000, you may owe Q4 estimated tax or underpayment
                penalty. Adjust W-4 ("additional withholding") or pay 1040-ES Q4.
            </p>
        </div>
    `;
}

function renderTable(yearVests) {
    const el = document.getElementById('rsu-table');
    if (!el) return;
    if (!yearVests.length) {
        el.innerHTML = `<h2 data-i18n="view.rsu.h2.vests">Vests</h2>
            <p class="muted" data-i18n="view.rsu.empty">No vests recorded for this year.</p>`;
        return;
    }
    const sorted = [...yearVests].sort((a, b) => String(b.vest_date).localeCompare(String(a.vest_date)));
    el.innerHTML = `
        <h2 data-i18n="view.rsu.h2.vests">Vests</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.rsu.th.date">Date</th>
                <th data-i18n="view.rsu.th.shares">Shares</th>
                <th data-i18n="view.rsu.th.fmv">FMV / share</th>
                <th data-i18n="view.rsu.th.ordinary">Ordinary income</th>
                <th data-i18n="view.rsu.th.withheld_shares">Withheld shares</th>
                <th data-i18n="view.rsu.th.fed_wh">Fed W/H</th>
                <th data-i18n="view.rsu.th.fica_wh">FICA W/H</th>
                <th data-i18n="view.rsu.th.state_wh">State W/H</th>
                <th data-i18n="view.rsu.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(v => {
                const ord = v.shares_vested * v.fmv_per_share;
                return `<tr>
                    <td>${esc(v.vest_date)}</td>
                    <td>${v.shares_vested}</td>
                    <td>$${v.fmv_per_share.toFixed(2)}</td>
                    <td class="pos">$${ord.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>${v.shares_withheld}</td>
                    <td>$${v.fed_w_h.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>$${v.fica_w_h.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>$${v.state_w_h.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td><button class="link neg" data-del="${esc(v.id)}" data-i18n="view.rsu.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.vests = state.vests.filter(v => v.id !== btn.dataset.del);
            save(state.vests);
            render();
        });
    });
}
