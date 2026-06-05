// State Residency Day-Count Tracker.
// Most states: > 183 days physical presence + domicile = statutory resident.
// NY / CA aggressive: any "permanent place of abode" + > 183 days = stat resident.
// Traders relocating from NY/CA to TX/FL/NV/WA/TN must track meticulously.
// Audit risk: NY DTF historical audit-rate ~12%; California FTB high for high earners.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-residency-v1';
const STATUTORY_THRESHOLD = 183;

const STATE_TAX_INFO = {
    NY: { rate: 0.109, top_bracket: 25_000_000, notes: 'most aggressive auditor; "permanent place of abode" + 183-day stat resident test' },
    CA: { rate: 0.133, top_bracket: 1_000_000, notes: 'FTB very aggressive; mental-intent domicile test + 9 contact factors' },
    NJ: { rate: 0.1075, top_bracket: 1_000_000, notes: 'PA reciprocal NJ-PA agreement' },
    MA: { rate: 0.09, top_bracket: 1_000_000, notes: 'Millionaires Tax 4% surtax > $1M' },
    CT: { rate: 0.0699, top_bracket: 500_000, notes: '' },
    IL: { rate: 0.0495, top_bracket: 0, notes: 'flat tax' },
    OR: { rate: 0.099, top_bracket: 125_000, notes: '' },
    HI: { rate: 0.11, top_bracket: 200_000, notes: '' },
    MN: { rate: 0.0985, top_bracket: 183_340, notes: '' },
    VT: { rate: 0.0875, top_bracket: 235_000, notes: '' },
    DC: { rate: 0.0995, top_bracket: 1_000_000, notes: '' },
    TX: { rate: 0, top_bracket: 0, notes: 'NO state income tax' },
    FL: { rate: 0, top_bracket: 0, notes: 'NO state income tax' },
    TN: { rate: 0, top_bracket: 0, notes: 'NO state income tax (Hall tax repealed 2021)' },
    WA: { rate: 0, top_bracket: 0, notes: '7% LTCG only (Quinn v. WA upheld 2022)' },
    NV: { rate: 0, top_bracket: 0, notes: 'NO state income tax' },
    WY: { rate: 0, top_bracket: 0, notes: 'NO state income tax' },
    SD: { rate: 0, top_bracket: 0, notes: 'NO state income tax' },
    AK: { rate: 0, top_bracket: 0, notes: 'NO state income tax + PFD dividend' },
    NH: { rate: 0, top_bracket: 0, notes: '5% interest/dividend only (sunsetting 2027)' },
};

function load() { try { return JSON.parse(localStorage.getItem(LS_KEY) || '{}'); } catch { return {}; } }
function save(data) { try { localStorage.setItem(LS_KEY, JSON.stringify(data)); } catch { /* ignore */ } }

let state = Object.assign({
    year: new Date().getFullYear(),
    days_by_state: {},  // { 'NY': 100, 'TX': 200, ... }
    has_abode: {},      // { 'NY': true, ... }
    domicile_state: 'TX',
    estimated_income: 0,
}, load());

function persist() { save(state); }

export async function renderResidencyDaycount(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.res.h1.title">// STATE RESIDENCY DAY-COUNT</span></h1>
        <p class="muted small" data-i18n="view.res.hint.intro">
            Most states: <strong>&gt; 183 days physical presence + domicile</strong> = statutory
            resident. <strong>NY / CA aggressive:</strong> any "permanent place of abode" + 183 days
            = stat resident even if domiciled elsewhere. Traders relocating to TX / FL / NV / WA / TN
            must track meticulously. NY DTF audit rate historically ~12% for stat-resident claims;
            CA FTB high for &gt; $1M income filers.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.res.h2.inputs">Setup</h2>
            <form id="res-form" class="inline-form">
                <label><span data-i18n="view.res.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}"></label>
                <label><span data-i18n="view.res.label.domicile">Domicile state</span>
                    <input type="text" name="domicile_state" value="${state.domicile_state}" maxlength="2" required></label>
                <label><span data-i18n="view.res.label.income">Estimated state-taxable income ($)</span>
                    <input type="number" step="0.01" name="estimated_income" value="${state.estimated_income}"></label>
                <button class="primary" type="submit" data-i18n="view.res.btn.update">Update</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.res.h2.add">Log days in state</h2>
            <form id="res-add" class="inline-form">
                <label><span data-i18n="view.res.label.state">State (2-letter)</span>
                    <input type="text" name="state" maxlength="2" required></label>
                <label><span data-i18n="view.res.label.days">Days</span>
                    <input type="number" step="1" name="days" required></label>
                <label><span data-i18n="view.res.label.has_abode">Have permanent place of abode here?</span>
                    <input type="checkbox" name="has_abode"></label>
                <button class="primary" type="submit" data-i18n="view.res.btn.add">Add</button>
            </form>
        </div>
        <div id="res-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.res.h2.domicile">Domicile-change checklist</h2>
            <ul class="muted small">
                <li data-i18n="view.res.dom.physical">Spend &gt; 183 days in new state (track with calendar)</li>
                <li data-i18n="view.res.dom.drivers">Get new state driver's license + register vehicles</li>
                <li data-i18n="view.res.dom.voter">Register to vote + actually vote in new state</li>
                <li data-i18n="view.res.dom.bank">Move primary banking + brokerage to new state</li>
                <li data-i18n="view.res.dom.doctors">Change doctors / dentists / vets to new state</li>
                <li data-i18n="view.res.dom.mail">Update mailing address everywhere + USPS forwarding</li>
                <li data-i18n="view.res.dom.club">Cancel old state social club / gym / country club</li>
                <li data-i18n="view.res.dom.school">Move kids' schools / dependents' addresses</li>
                <li data-i18n="view.res.dom.will">Update will / trust to reference new state</li>
                <li data-i18n="view.res.dom.affidavit">File NY IT-201/CA 540 with partial-year + statement of domicile change</li>
                <li data-i18n="view.res.dom.documentation">Keep AIRLINE TICKETS, hotel receipts, calendar exports — burden of proof on you</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.res.h2.audit_factors">NY DTF "Bona Fide Residence" factors</h2>
            <ul class="muted small">
                <li data-i18n="view.res.audit.home">Home (size, value, time spent)</li>
                <li data-i18n="view.res.audit.active_biz">Active business connections</li>
                <li data-i18n="view.res.audit.time">Time analysis (calendar days)</li>
                <li data-i18n="view.res.audit.items">Items near and dear (heirlooms, pets, valuables)</li>
                <li data-i18n="view.res.audit.family">Family connections (where kids / spouse live)</li>
            </ul>
        </div>
    `;
    document.getElementById('res-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.year = Number(fd.get('year')) || new Date().getFullYear();
        state.domicile_state = String(fd.get('domicile_state') || '').toUpperCase().slice(0, 2);
        state.estimated_income = Number(fd.get('estimated_income')) || 0;
        persist(); render();
    });
    document.getElementById('res-add').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const stKey = String(fd.get('state') || '').toUpperCase().slice(0, 2);
        const days = Number(fd.get('days')) || 0;
        if (!stKey || days <= 0) return;
        state.days_by_state[stKey] = (state.days_by_state[stKey] || 0) + days;
        state.has_abode[stKey] = !!fd.get('has_abode');
        persist();
        e.target.reset();
        showToast(t('view.res.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function render() {
    const el = document.getElementById('res-output');
    if (!el) return;
    const entries = Object.entries(state.days_by_state).sort((a, b) => b[1] - a[1]);
    const totalDays = entries.reduce((s, [, d]) => s + d, 0);
    const overflow = totalDays > 366 ? totalDays - 366 : 0;
    const statutoryResidentRows = entries.map(([st, days]) => {
        const tax = STATE_TAX_INFO[st] || { rate: 0, notes: '' };
        const isOverThreshold = days > STATUTORY_THRESHOLD;
        const hasAbode = !!state.has_abode[st];
        const isStatResident = isOverThreshold && hasAbode && st !== state.domicile_state;
        const aggressive = ['NY', 'CA', 'NJ', 'MA', 'CT', 'OR'].includes(st);
        const exposure = isStatResident ? state.estimated_income * tax.rate : 0;
        return { st, days, isOverThreshold, hasAbode, isStatResident, aggressive, exposure, tax };
    });
    const totalDoubleTaxRisk = statutoryResidentRows.reduce((s, r) => s + r.exposure, 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.res.h2.summary">${state.year} day-count summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.res.card.domicile">Domicile</div>
                    <div class="value">${esc(state.domicile_state)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.res.card.total_days">Total days logged</div>
                    <div class="value">${totalDays}</div>
                </div>
                ${overflow > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.res.card.over_366">Over 366 days logged!</div>
                        <div class="value">${overflow}</div>
                    </div>
                ` : ''}
                <div class="card ${totalDoubleTaxRisk > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.res.card.exposure">Stat-resident tax exposure</div>
                    <div class="value">$${totalDoubleTaxRisk.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.res.h2.states">States</h2>
            ${entries.length === 0 ? `<p class="muted" data-i18n="view.res.empty">No days logged.</p>` : `
                <table class="trades">
                    <thead><tr>
                        <th data-i18n="view.res.th.state">State</th>
                        <th data-i18n="view.res.th.days">Days</th>
                        <th data-i18n="view.res.th.over_183">&gt; 183?</th>
                        <th data-i18n="view.res.th.abode">Abode?</th>
                        <th data-i18n="view.res.th.stat_resident">Stat resident?</th>
                        <th data-i18n="view.res.th.rate">Top rate</th>
                        <th data-i18n="view.res.th.exposure">Exposure</th>
                        <th data-i18n="view.res.th.notes">Notes</th>
                    </tr></thead>
                    <tbody>${statutoryResidentRows.map(r => `
                        <tr>
                            <td>${esc(r.st)}</td>
                            <td class="${r.days > STATUTORY_THRESHOLD ? 'neg' : ''}">${r.days}</td>
                            <td class="${r.isOverThreshold ? 'neg' : 'pos'}">${r.isOverThreshold ? esc(t('view.res.status.yes')) : esc(t('view.res.status.no'))}</td>
                            <td class="${r.hasAbode ? 'neg' : 'pos'}">${r.hasAbode ? esc(t('view.res.status.yes')) : esc(t('view.res.status.no'))}</td>
                            <td class="${r.isStatResident ? 'neg' : 'pos'}">${r.isStatResident ? esc(t('view.res.status.yes')) : esc(t('view.res.status.no'))}</td>
                            <td>${(r.tax.rate * 100).toFixed(2)}%</td>
                            <td class="${r.exposure > 0 ? 'neg' : 'muted'}">$${r.exposure.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                            <td class="muted small">${esc(r.tax.notes || '')}</td>
                        </tr>
                    `).join('')}</tbody>
                </table>
            `}
        </div>
    `;
}
