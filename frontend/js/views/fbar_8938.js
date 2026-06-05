// FBAR + Form 8938 Tracker — foreign account reporting compliance.
// - FBAR (FinCEN 114): aggregate foreign balances > $10,000 at any point in year.
// - Form 8938: foreign assets > $50k single ($100k MFJ) year-end, OR > $75k single
//   ($150k MFJ) at any point in year. Higher thresholds if living abroad.
// Both required for many US traders using IBKR-UK, Saxo, IG Markets, etc.
// FBAR penalty: $10k/non-willful, $100k+/willful per year. NOT optional.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const LS_KEY = 'tv-foreign-accounts-v1';

const FBAR_THRESHOLD = 10_000;
const F8938_SINGLE_YE = 50_000;
const F8938_SINGLE_PEAK = 75_000;
const F8938_MFJ_YE = 100_000;
const F8938_MFJ_PEAK = 150_000;

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    accounts: load(),
    filing: 'single',
    year: new Date().getFullYear(),
};

export async function renderFbar8938(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.fbar.h1.title">// FBAR + FORM 8938 TRACKER</span></h1>
        <p class="muted small" data-i18n="view.fbar.hint.intro">
            US persons with foreign financial accounts must report them. FBAR triggers at
            $10k aggregate any-point. Form 8938 has higher thresholds. Many traders trip
            FBAR by holding IBKR-UK, Saxo, IG Markets — IBKR US-only doesn't count.
            <strong>FBAR penalty: $10k non-willful, $100k+ willful per year.</strong>
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.fbar.h2.add">Add foreign account</h2>
            <form id="fb-form" class="inline-form">
                <label><span data-i18n="view.fbar.label.year">Tax year</span>
                    <input type="number" step="1" name="year" value="${state.year}" required></label>
                <label><span data-i18n="view.fbar.label.institution">Institution</span>
                    <input type="text" name="institution" placeholder="IBKR UK / Saxo / IG Markets" required></label>
                <label><span data-i18n="view.fbar.label.country">Country</span>
                    <input type="text" name="country" placeholder="United Kingdom" required></label>
                <label><span data-i18n="view.fbar.label.account_number">Account number</span>
                    <input type="text" name="account_number" placeholder="U1234567" required></label>
                <label><span data-i18n="view.fbar.label.account_type">Type</span>
                    <select name="account_type">
                        <option value="securities">Securities</option>
                        <option value="bank">Bank</option>
                        <option value="other">Other</option>
                    </select>
                </label>
                <label><span data-i18n="view.fbar.label.max_value">Maximum value during year (USD)</span>
                    <input type="number" step="0.01" name="max_value_usd" required></label>
                <label><span data-i18n="view.fbar.label.year_end_value">Year-end value (USD)</span>
                    <input type="number" step="0.01" name="year_end_value_usd" required></label>
                <button class="primary" type="submit" data-i18n="view.fbar.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.fbar.label.filing">Filing status</span>
                    <select id="fb-filing">
                        <option value="single" ${state.filing === 'single' ? 'selected' : ''}>Single</option>
                        <option value="mfj"    ${state.filing === 'mfj' ? 'selected' : ''}>MFJ</option>
                    </select>
                </label>
                <label><span data-i18n="view.fbar.label.view_year">View year</span>
                    <input type="number" id="fb-view-year" value="${state.year}" min="2010"></label>
            </div>
        </div>
        <div id="fb-summary"></div>
        <div id="fb-table" class="chart-panel"></div>
    `;
    document.getElementById('fb-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const a = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            year: Number(fd.get('year')),
            institution: fd.get('institution'),
            country: fd.get('country'),
            account_number: fd.get('account_number'),
            account_type: fd.get('account_type'),
            max_value_usd: Number(fd.get('max_value_usd')),
            year_end_value_usd: Number(fd.get('year_end_value_usd')),
        };
        state.accounts.push(a);
        save(state.accounts);
        e.target.reset();
        e.target.querySelector('[name="year"]').value = state.year;
        render();
    });
    document.getElementById('fb-filing').addEventListener('change', e => {
        state.filing = e.target.value;
        render();
    });
    document.getElementById('fb-view-year').addEventListener('change', e => {
        state.year = Number(e.target.value) || new Date().getFullYear();
        render();
    });
    render();
}

function render() {
    const yearAccts = state.accounts.filter(a => a.year === state.year);
    renderSummary(yearAccts);
    renderTable(yearAccts);
}

function renderSummary(yearAccts) {
    const el = document.getElementById('fb-summary');
    if (!el) return;
    const aggregateMax = yearAccts.reduce((s, a) => s + a.max_value_usd, 0);
    const aggregateYE = yearAccts.reduce((s, a) => s + a.year_end_value_usd, 0);
    const fbarRequired = aggregateMax >= FBAR_THRESHOLD;
    const f8938YE_threshold = state.filing === 'mfj' ? F8938_MFJ_YE : F8938_SINGLE_YE;
    const f8938Peak_threshold = state.filing === 'mfj' ? F8938_MFJ_PEAK : F8938_SINGLE_PEAK;
    const f8938Required = aggregateYE >= f8938YE_threshold || aggregateMax >= f8938Peak_threshold;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.fbar.h2.compliance">${state.year} compliance summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.fbar.card.accounts">Accounts</div>
                    <div class="value">${yearAccts.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.fbar.card.aggregate_max">Aggregate max (any point)</div>
                    <div class="value">$${aggregateMax.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.fbar.card.aggregate_ye">Aggregate year-end</div>
                    <div class="value">$${aggregateYE.toLocaleString()}</div>
                </div>
                <div class="card ${fbarRequired ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.fbar.card.fbar_required">FBAR required</div>
                    <div class="value">${fbarRequired ? esc(t('view.fbar.status.yes')) : esc(t('view.fbar.status.no'))}</div>
                </div>
                <div class="card ${f8938Required ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.fbar.card.f8938_required">Form 8938 required</div>
                    <div class="value">${f8938Required ? esc(t('view.fbar.status.yes')) : esc(t('view.fbar.status.no'))}</div>
                </div>
            </div>
            <p class="muted small" style="margin-top:10px" data-i18n="view.fbar.deadlines">
                FBAR (FinCEN 114): April 15 with automatic Oct 15 extension. Form 8938: attaches to your 1040.
            </p>
        </div>
    `;
}

function renderTable(yearAccts) {
    const el = document.getElementById('fb-table');
    if (!el) return;
    if (!yearAccts.length) {
        el.innerHTML = `<h2 data-i18n="view.fbar.h2.accounts">Accounts</h2>
            <p class="muted" data-i18n="view.fbar.empty">No foreign accounts for this year.</p>`;
        return;
    }
    const sorted = [...yearAccts].sort((a, b) => b.max_value_usd - a.max_value_usd);
    el.innerHTML = `
        <h2 data-i18n="view.fbar.h2.accounts">Accounts</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.fbar.th.institution">Institution</th>
                <th data-i18n="view.fbar.th.country">Country</th>
                <th data-i18n="view.fbar.th.account_number">Account #</th>
                <th data-i18n="view.fbar.th.type">Type</th>
                <th data-i18n="view.fbar.th.max_value">Max value</th>
                <th data-i18n="view.fbar.th.year_end">Year-end</th>
                <th data-i18n="view.fbar.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(a => `
                <tr>
                    <td>${esc(a.institution)}</td>
                    <td class="muted">${esc(a.country)}</td>
                    <td class="muted">${esc(a.account_number)}</td>
                    <td class="muted">${esc(a.account_type)}</td>
                    <td>$${a.max_value_usd.toLocaleString()}</td>
                    <td>$${a.year_end_value_usd.toLocaleString()}</td>
                    <td><button class="link neg" data-del="${esc(a.id)}" data-i18n="view.fbar.btn.delete">delete</button></td>
                </tr>
            `).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.accounts = state.accounts.filter(a => a.id !== btn.dataset.del);
            save(state.accounts);
            render();
        });
    });
}
