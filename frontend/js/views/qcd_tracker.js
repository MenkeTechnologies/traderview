// QCD (Qualified Charitable Distribution) Tracker — IRC § 408(d)(8).
// Direct transfer from Traditional IRA to qualified charity. NOT taxable
// income, counts toward RMD. 2024 cap: $105k/yr per taxpayer.
// Age 70½+ required (even though RMD age is 73+). SECURE 2.0 added
// one-time $53,000 (2024) QCD to split-interest trust (CRT, CRUT, CGA).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-qcd-v1';
const ANNUAL_CAP_2024 = 105_000;
const ANNUAL_CAP_2025 = 108_000;
const SPLIT_INTEREST_2024 = 53_000;
const SPLIT_INTEREST_2025 = 54_000;
const MIN_AGE = 70.5;

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    distributions: load(),
    year: new Date().getFullYear(),
    rmd_required_amount: 0,
};

export async function renderQcdTracker(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.qcd.h1.title">// QCD TRACKER</span></h1>
        <p class="muted small" data-i18n="view.qcd.hint.intro">
            Qualified Charitable Distribution: direct IRA → charity transfer. NOT
            taxable income, COUNTS toward RMD. 2024 cap: $105k/yr per taxpayer.
            <strong>Age 70½+ required</strong> (even though RMD age is 73+). Great for
            retirees who don't itemize — gives them charitable benefit without itemizing.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.qcd.h2.add">Log QCD</h2>
            <form id="qcd-form" class="inline-form">
                <label><span data-i18n="view.qcd.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}" required></label>
                <label><span data-i18n="view.qcd.label.date">Date</span>
                    <input type="date" name="date" required value="${new Date().toISOString().slice(0,10)}"></label>
                <label><span data-i18n="view.qcd.label.charity">Charity name</span>
                    <input type="text" name="charity" required></label>
                <label><span data-i18n="view.qcd.label.amount">Amount ($)</span>
                    <input type="number" step="100" name="amount" required></label>
                <label><span data-i18n="view.qcd.label.is_split_interest">Split-interest (CRT/CGA)?</span>
                    <input type="checkbox" name="is_split_interest"></label>
                <label><span data-i18n="view.qcd.label.notes">Notes</span>
                    <input type="text" name="notes"></label>
                <button class="primary" type="submit" data-i18n="view.qcd.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.qcd.label.view_year">View year</span>
                    <input type="number" id="qcd-year" value="${state.year}"></label>
                <label><span data-i18n="view.qcd.label.rmd_amount">${state.year} RMD amount ($)</span>
                    <input type="number" step="100" id="qcd-rmd" value="${state.rmd_required_amount}"></label>
            </div>
        </div>
        <div id="qcd-summary"></div>
        <div id="qcd-table" class="chart-panel"></div>
    `;
    document.getElementById('qcd-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const r = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            year: Number(fd.get('year')),
            date: fd.get('date'),
            charity: fd.get('charity'),
            amount: Number(fd.get('amount')),
            is_split_interest: !!fd.get('is_split_interest'),
            notes: fd.get('notes') || '',
        };
        state.distributions.push(r);
        save(state.distributions);
        e.target.reset();
        e.target.querySelector('[name="year"]').value = state.year;
        e.target.querySelector('[name="date"]').value = new Date().toISOString().slice(0, 10);
        showToast(t('view.qcd.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('qcd-year').addEventListener('change', e => {
        state.year = Number(e.target.value) || new Date().getFullYear();
        render();
    });
    document.getElementById('qcd-rmd').addEventListener('change', e => {
        state.rmd_required_amount = Number(e.target.value) || 0;
        render();
    });
    render();
}

function render() {
    const yearDist = state.distributions.filter(d => d.year === state.year);
    renderSummary(yearDist);
    renderTable(yearDist);
}

function renderSummary(yearDist) {
    const el = document.getElementById('qcd-summary');
    if (!el) return;
    const cap = state.year >= 2025 ? ANNUAL_CAP_2025 : ANNUAL_CAP_2024;
    const splitCap = state.year >= 2025 ? SPLIT_INTEREST_2025 : SPLIT_INTEREST_2024;
    const totalQcd = yearDist.reduce((s, d) => s + d.amount, 0);
    const splitTotal = yearDist.filter(d => d.is_split_interest).reduce((s, d) => s + d.amount, 0);
    const directTotal = totalQcd - splitTotal;
    const overCap = Math.max(0, totalQcd - cap);
    const remaining = Math.max(0, cap - totalQcd);
    const rmdShortfall = Math.max(0, state.rmd_required_amount - totalQcd);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.qcd.h2.summary">${state.year} summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.qcd.card.total">Total QCDs</div>
                    <div class="value">$${totalQcd.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.qcd.card.cap">Annual cap</div>
                    <div class="value">$${cap.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.qcd.card.remaining">Cap remaining</div>
                    <div class="value">$${remaining.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${overCap > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.qcd.card.over_cap">Excess over cap</div>
                    <div class="value">$${overCap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${state.rmd_required_amount > 0 ? `
                    <div class="card ${rmdShortfall > 0 ? 'neg' : 'pos'}">
                        <div class="label" data-i18n="view.qcd.card.rmd_remaining">RMD remaining</div>
                        <div class="value">$${rmdShortfall.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card">
                    <div class="label" data-i18n="view.qcd.card.split_interest">Split-interest used</div>
                    <div class="value">$${splitTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })} / $${splitCap.toLocaleString()}</div>
                </div>
            </div>
            <p class="muted small" style="margin-top:10px" data-i18n="view.qcd.note">
                QCD MUST go directly from IRA to charity (custodian-to-custodian).
                Taking the cash first and donating doesn't qualify — that's regular RMD + itemized deduction.
            </p>
        </div>
    `;
}

function renderTable(yearDist) {
    const el = document.getElementById('qcd-table');
    if (!el) return;
    if (!yearDist.length) {
        el.innerHTML = `<h2 data-i18n="view.qcd.h2.distributions">Distributions</h2>
            <p class="muted" data-i18n="view.qcd.empty">No QCDs recorded for this year.</p>`;
        return;
    }
    const sorted = [...yearDist].sort((a, b) => String(b.date).localeCompare(String(a.date)));
    el.innerHTML = `
        <h2 data-i18n="view.qcd.h2.distributions">Distributions</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.qcd.th.date">Date</th>
                <th data-i18n="view.qcd.th.charity">Charity</th>
                <th data-i18n="view.qcd.th.amount">Amount</th>
                <th data-i18n="view.qcd.th.type">Type</th>
                <th data-i18n="view.qcd.th.notes">Notes</th>
                <th data-i18n="view.qcd.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(d => `
                <tr>
                    <td>${esc(d.date)}</td>
                    <td>${esc(d.charity)}</td>
                    <td class="pos">$${d.amount.toLocaleString()}</td>
                    <td class="muted">${esc(d.is_split_interest ? t('view.qcd.type.split') : t('view.qcd.type.direct'))}</td>
                    <td class="muted">${esc(d.notes || '')}</td>
                    <td><button class="link neg" data-del="${esc(d.id)}" data-i18n="view.qcd.btn.delete">delete</button></td>
                </tr>
            `).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.distributions = state.distributions.filter(d => d.id !== btn.dataset.del);
            save(state.distributions);
            render();
        });
    });
}
