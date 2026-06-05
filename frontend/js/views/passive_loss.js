// Passive Activity Loss (PAL) § 469 Tracker.
// Passive losses can ONLY offset passive income unless:
//  - $25,000 special allowance (single/MFJ < $100k MAGI, phased to 0 at $150k)
//    for active-participation rental real estate
//  - Real-estate-professional status: 750 hrs + 50%+ of personal services in real estate
//  - Material participation (7 tests)
// Unused PALs SUSPEND, freed on full disposition of activity.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-pal-v1';
const PAL_25K_ALLOWANCE = 25_000;
const PAL_PHASEOUT_START = 100_000;
const PAL_PHASEOUT_END = 150_000;

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    activities: load(),
    year: new Date().getFullYear(),
    magi: 150_000,
    is_rep: false,
    active_participation: true,
};

export async function renderPassiveLoss(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pal.h1.title">// PASSIVE LOSS § 469</span></h1>
        <p class="muted small" data-i18n="view.pal.hint.intro">
            Passive losses can only offset passive INCOME unless: (1) $25k special
            allowance for active-participation rental real estate (phased to 0 at
            $150k MAGI), (2) Real Estate Professional Status (750 hrs + 50%), or
            (3) material participation (7-test). Unused PALs SUSPEND, released on
            full disposition of activity.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.pal.h2.add">Add passive activity</h2>
            <form id="pal-form" class="inline-form">
                <label><span data-i18n="view.pal.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}" required></label>
                <label><span data-i18n="view.pal.label.activity">Activity</span>
                    <input type="text" name="activity" placeholder="123 Main St rental" required></label>
                <label><span data-i18n="view.pal.label.type">Type</span>
                    <select name="activity_type">
                        <option value="rental_re">Rental real estate</option>
                        <option value="trade_business">Trade/business (limited)</option>
                        <option value="other_passive">Other passive</option>
                    </select>
                </label>
                <label><span data-i18n="view.pal.label.income">Passive income ($)</span>
                    <input type="number" step="0.01" name="income" value="0"></label>
                <label><span data-i18n="view.pal.label.loss">Passive loss ($)</span>
                    <input type="number" step="0.01" name="loss" value="0"></label>
                <label><span data-i18n="view.pal.label.suspended_pal">Prior suspended PAL ($)</span>
                    <input type="number" step="0.01" name="suspended_pal_prior" value="0"></label>
                <label><span data-i18n="view.pal.label.fully_disposed">Fully disposed this year?</span>
                    <input type="checkbox" name="fully_disposed"></label>
                <button class="primary" type="submit" data-i18n="view.pal.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.pal.h2.your_status">Your status</h2>
            <form id="pal-status" class="inline-form">
                <label><span data-i18n="view.pal.label.view_year">View year</span>
                    <input type="number" id="pal-year" value="${state.year}"></label>
                <label><span data-i18n="view.pal.label.magi">MAGI ($)</span>
                    <input type="number" id="pal-magi" step="0.01" value="${state.magi}"></label>
                <label><span data-i18n="view.pal.label.is_rep">Real Estate Professional Status?</span>
                    <input type="checkbox" id="pal-rep" ${state.is_rep ? 'checked' : ''}></label>
                <label><span data-i18n="view.pal.label.active_participation">Active participation in rentals?</span>
                    <input type="checkbox" id="pal-active" ${state.active_participation ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.pal.btn.update">Update</button>
            </form>
        </div>
        <div id="pal-summary"></div>
        <div id="pal-table" class="chart-panel"></div>
    `;
    document.getElementById('pal-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const a = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            year: Number(fd.get('year')),
            activity: fd.get('activity'),
            activity_type: fd.get('activity_type'),
            income: Number(fd.get('income')) || 0,
            loss: Number(fd.get('loss')) || 0,
            suspended_pal_prior: Number(fd.get('suspended_pal_prior')) || 0,
            fully_disposed: !!fd.get('fully_disposed'),
        };
        state.activities.push(a);
        save(state.activities);
        e.target.reset();
        e.target.querySelector('[name="year"]').value = state.year;
        showToast(t('view.pal.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('pal-status').addEventListener('submit', (e) => {
        e.preventDefault();
        state.year = Number(document.getElementById('pal-year').value) || new Date().getFullYear();
        state.magi = Number(document.getElementById('pal-magi').value) || 0;
        state.is_rep = document.getElementById('pal-rep').checked;
        state.active_participation = document.getElementById('pal-active').checked;
        render();
    });
    render();
}

function render() {
    const yearAct = state.activities.filter(a => a.year === state.year);
    const totalPassiveIncome = yearAct.reduce((s, a) => s + a.income, 0);
    const totalPassiveLoss = yearAct.reduce((s, a) => s + a.loss + a.suspended_pal_prior, 0);
    const netLoss = Math.max(0, totalPassiveLoss - totalPassiveIncome);

    // Compute allowance
    let allowance = 0;
    if (state.is_rep) {
        allowance = netLoss;  // unlimited deduction against ordinary income
    } else if (state.active_participation) {
        const fullAllowance = PAL_25K_ALLOWANCE;
        const phasedOut = Math.max(0, (state.magi - PAL_PHASEOUT_START) * 0.50);
        allowance = Math.max(0, Math.min(netLoss, fullAllowance - phasedOut));
    }

    const releasedDisposition = yearAct.filter(a => a.fully_disposed)
        .reduce((s, a) => s + a.suspended_pal_prior + a.loss, 0);
    const totalAllowed = allowance + releasedDisposition;
    const newSuspended = Math.max(0, netLoss - totalAllowed);

    renderSummary({ totalPassiveIncome, totalPassiveLoss, netLoss,
                    allowance, releasedDisposition, totalAllowed, newSuspended });
    renderTable(yearAct);
}

function renderSummary({ totalPassiveIncome, totalPassiveLoss, netLoss, allowance, releasedDisposition, totalAllowed, newSuspended }) {
    const el = document.getElementById('pal-summary');
    if (!el) return;
    const phaseoutAmt = Math.max(0, (state.magi - PAL_PHASEOUT_START) * 0.50);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.pal.h2.summary">${state.year} summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.pal.card.income">Passive income</div>
                    <div class="value">$${totalPassiveIncome.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.pal.card.loss">Passive losses (incl prior)</div>
                    <div class="value">$${totalPassiveLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.pal.card.net_loss">Net loss</div>
                    <div class="value">$${netLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.pal.card.allowance">$25k allowance / REP unlimited</div>
                    <div class="value">$${allowance.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.pal.card.released">Released on disposition</div>
                    <div class="value">$${releasedDisposition.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.pal.card.allowed">Total deductible this year</div>
                    <div class="value">$${totalAllowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.pal.card.suspended">Suspended → next year</div>
                    <div class="value">$${newSuspended.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.pal.card.phaseout">$25k phaseout applied</div>
                    <div class="value">$${phaseoutAmt.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(yearAct) {
    const el = document.getElementById('pal-table');
    if (!el) return;
    if (!yearAct.length) {
        el.innerHTML = `<h2 data-i18n="view.pal.h2.activities">Activities</h2>
            <p class="muted" data-i18n="view.pal.empty">No passive activities for this year.</p>`;
        return;
    }
    el.innerHTML = `
        <h2 data-i18n="view.pal.h2.activities">Activities</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.pal.th.activity">Activity</th>
                <th data-i18n="view.pal.th.type">Type</th>
                <th data-i18n="view.pal.th.income">Income</th>
                <th data-i18n="view.pal.th.loss">Loss</th>
                <th data-i18n="view.pal.th.prior_suspended">Prior suspended</th>
                <th data-i18n="view.pal.th.fully_disposed">Fully disposed</th>
                <th data-i18n="view.pal.th.actions">Actions</th>
            </tr></thead>
            <tbody>${yearAct.map(a => `
                <tr>
                    <td>${esc(a.activity)}</td>
                    <td class="muted">${esc(a.activity_type)}</td>
                    <td class="pos">$${a.income.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="neg">$${a.loss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>$${a.suspended_pal_prior.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>${a.fully_disposed ? '✓' : ''}</td>
                    <td><button class="link neg" data-del="${esc(a.id)}" data-i18n="view.pal.btn.delete">delete</button></td>
                </tr>
            `).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.activities = state.activities.filter(a => a.id !== btn.dataset.del);
            save(state.activities);
            render();
        });
    });
}
