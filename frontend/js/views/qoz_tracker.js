// Qualified Opportunity Zone (QOZ) Investment Tracker — IRC § 1400Z-2.
// Defer cap gains by reinvesting in QOF within 180 days.
//   - Hold 5 years: 10% basis step-up (eliminates 10% of deferred gain)
//   - Hold 7 years: ADDITIONAL 5% step-up (15% total) — REQUIRED BY 2026
//   - Hold 10+ years: 100% step-up on QOF gains (deferred original gain still taxed)
// Original gain recognized Dec 31, 2026 regardless.
// PROGRAM CLIFF: 10-yr basis step-up for new investments expires 2026.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-qoz-v1';
const RECOGNITION_DATE = new Date('2026-12-31');

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    investments: load(),
    marginal_lt_rate: 0.20,
    niit: 0.038,
    current_market_growth: 0.08,
};

export async function renderQozTracker(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.qoz.h1.title">// OPPORTUNITY ZONE TRACKER</span></h1>
        <p class="muted small" data-i18n="view.qoz.hint.intro">
            <strong>§ 1400Z-2:</strong> defer cap gains by reinvesting in a QOF within
            180 days. Hold 5y: 10% step-up. Hold 7y: 15% (BY 2026). Hold 10+y: 100% step-up
            on the QOF gain (deferred gain still taxed Dec 31, 2026). After Dec 31, 2026,
            the deferred gain is recognized regardless of whether you've sold.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.qoz.h2.add">Log QOF investment</h2>
            <form id="qoz-form" class="inline-form">
                <label><span data-i18n="view.qoz.label.original_gain_date">Original gain date</span>
                    <input type="date" name="original_gain_date" required></label>
                <label><span data-i18n="view.qoz.label.qof_invested_date">QOF invested date</span>
                    <input type="date" name="qof_invested_date" required></label>
                <label><span data-i18n="view.qoz.label.deferred_gain">Deferred cap gain ($)</span>
                    <input type="number" step="0.01" name="deferred_gain" required></label>
                <label><span data-i18n="view.qoz.label.qof_name">QOF name</span>
                    <input type="text" name="qof_name" required></label>
                <label><span data-i18n="view.qoz.label.current_value">Current QOF value ($)</span>
                    <input type="number" step="0.01" name="current_value"></label>
                <button class="primary" type="submit" data-i18n="view.qoz.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.qoz.h2.rates">Tax rates</h2>
            <form id="qoz-rates" class="inline-form">
                <label><span data-i18n="view.qoz.label.marginal_lt_rate">Marginal LT cap-gains rate %</span>
                    <input type="number" step="0.5" name="marginal_lt_rate" value="${(state.marginal_lt_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.qoz.label.niit">NIIT %</span>
                    <input type="number" step="0.1" name="niit" value="${(state.niit * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.qoz.btn.update">Update</button>
            </form>
        </div>
        <div id="qoz-summary"></div>
        <div id="qoz-table" class="chart-panel"></div>
    `;
    document.getElementById('qoz-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const i = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            original_gain_date: fd.get('original_gain_date'),
            qof_invested_date: fd.get('qof_invested_date'),
            deferred_gain: Number(fd.get('deferred_gain')),
            qof_name: fd.get('qof_name'),
            current_value: Number(fd.get('current_value')) || Number(fd.get('deferred_gain')),
        };
        state.investments.push(i);
        save(state.investments);
        e.target.reset();
        showToast(t('view.qoz.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('qoz-rates').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.marginal_lt_rate = (Number(fd.get('marginal_lt_rate')) || 20) / 100;
        state.niit = (Number(fd.get('niit')) || 3.8) / 100;
        render();
    });
    render();
}

function computeStepUp(i) {
    const investedDate = new Date(i.qof_invested_date);
    const fiveYr = new Date(investedDate); fiveYr.setFullYear(fiveYr.getFullYear() + 5);
    const sevenYr = new Date(investedDate); sevenYr.setFullYear(sevenYr.getFullYear() + 7);
    if (fiveYr > RECOGNITION_DATE) return 0;
    if (sevenYr > RECOGNITION_DATE) return 0.10;
    return 0.15;
}

function render() {
    const stats = state.investments.map(i => {
        const investedDate = new Date(i.qof_invested_date);
        const tenYr = new Date(investedDate); tenYr.setFullYear(tenYr.getFullYear() + 10);
        const stepUpPct = computeStepUp(i);
        const recognizedGain = i.deferred_gain * (1 - stepUpPct);
        const taxOnDeferred = recognizedGain * (state.marginal_lt_rate + state.niit);
        const qofAppreciation = Math.max(0, i.current_value - i.deferred_gain);
        const taxOn10YrGain = 0;  // 100% exclusion if 10+ year hold
        return { ...i, stepUpPct, recognizedGain, taxOnDeferred,
                 qofAppreciation, ten_year_date: tenYr };
    });
    renderSummary(stats);
    renderTable(stats);
}

function renderSummary(stats) {
    const el = document.getElementById('qoz-summary');
    if (!el) return;
    const totalDeferred = stats.reduce((s, i) => s + i.deferred_gain, 0);
    const totalStepUp = stats.reduce((s, i) => s + i.deferred_gain * i.stepUpPct, 0);
    const taxOnDeferredTotal = stats.reduce((s, i) => s + i.taxOnDeferred, 0);
    const appreciation = stats.reduce((s, i) => s + i.qofAppreciation, 0);
    const taxAvoidedOnAppreciation = appreciation * (state.marginal_lt_rate + state.niit);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.qoz.h2.summary">QOF portfolio summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.qoz.card.investments">Investments</div>
                    <div class="value">${stats.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.qoz.card.deferred_gain">Total deferred gain</div>
                    <div class="value">$${totalDeferred.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.qoz.card.step_up_total">Total step-up earned</div>
                    <div class="value">$${totalStepUp.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.qoz.card.tax_on_deferred">Tax due Dec 31, 2026</div>
                    <div class="value">$${taxOnDeferredTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.qoz.card.qof_appreciation">QOF appreciation</div>
                    <div class="value">$${appreciation.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.qoz.card.tax_avoided">Tax avoided if held 10+y</div>
                    <div class="value">$${taxAvoidedOnAppreciation.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            <p class="muted small" style="margin-top:10px" data-i18n="view.qoz.cliff_note">
                CLIFF: original deferred gain recognized Dec 31, 2026 regardless of hold.
                New QOF investments after 2021 can no longer earn the 7-year 15% step-up.
                10-year 100% exclusion still available for new investments through 2026.
            </p>
        </div>
    `;
}

function renderTable(stats) {
    const el = document.getElementById('qoz-table');
    if (!el) return;
    if (!stats.length) {
        el.innerHTML = `<h2 data-i18n="view.qoz.h2.investments">Investments</h2>
            <p class="muted" data-i18n="view.qoz.empty">No QOF investments tracked yet.</p>`;
        return;
    }
    const sorted = [...stats].sort((a, b) => b.deferred_gain - a.deferred_gain);
    el.innerHTML = `
        <h2 data-i18n="view.qoz.h2.investments">Investments</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.qoz.th.qof_name">QOF</th>
                <th data-i18n="view.qoz.th.invested_date">Invested</th>
                <th data-i18n="view.qoz.th.deferred_gain">Deferred gain</th>
                <th data-i18n="view.qoz.th.step_up">Step-up %</th>
                <th data-i18n="view.qoz.th.recognized">Recognized (2026)</th>
                <th data-i18n="view.qoz.th.tax_due">Tax due</th>
                <th data-i18n="view.qoz.th.current_value">Current value</th>
                <th data-i18n="view.qoz.th.10yr_date">10-yr exclusion date</th>
                <th data-i18n="view.qoz.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(i => `
                <tr>
                    <td>${esc(i.qof_name)}</td>
                    <td>${esc(i.qof_invested_date)}</td>
                    <td>$${i.deferred_gain.toLocaleString()}</td>
                    <td class="pos">${(i.stepUpPct * 100).toFixed(0)}%</td>
                    <td>$${i.recognizedGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="neg">$${i.taxOnDeferred.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>$${i.current_value.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">${esc(i.ten_year_date.toISOString().slice(0, 10))}</td>
                    <td><button class="link neg" data-del="${esc(i.id)}" data-i18n="view.qoz.btn.delete">delete</button></td>
                </tr>
            `).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.investments = state.investments.filter(i => i.id !== btn.dataset.del);
            save(state.investments);
            render();
        });
    });
}
