// IRC § 6038 — Form 5471 CFC reporting.
// US person owning ≥ 10% of a CFC (foreign corp where US owners hold > 50%) must file
// Form 5471 with their 1040. 5 categories of filer (1-5). Penalty: $10,000 per year per
// CFC for non-filing, increasing by $10,000/month up to $50,000. Continuing failure can
// reduce foreign tax credit. Subpart F + GILTI income flow through annually.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-form-5471-v1';
const PENALTY_PER_FAILURE = 10_000;
const MAX_PENALTY = 50_000;
const CFC_THRESHOLD_US_OWNERSHIP = 0.50;
const TEN_PCT_OWNERSHIP = 0.10;

function load() { try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); } catch { return []; } }
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    cfcs: load(),
};

export async function renderSection6038(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6038.h1.title">// § 6038 FORM 5471 CFC REPORTING</span></h1>
        <p class="muted small" data-i18n="view.s6038.hint.intro">
            US person owning ≥ 10% of a CFC (foreign corp where US owners hold &gt; 50%) must file
            <strong>Form 5471</strong> with their return. <strong>Penalty: $10,000 per year per CFC</strong>
            for non-filing, escalating $10k/month up to $50k. Continuing failure reduces foreign tax credit.
            <strong>Subpart F + GILTI</strong> income flows through annually regardless of cash distribution.
            <strong>5 categories of filer</strong> by ownership / control. Common trader trap: foreign
            holding companies.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038.h2.categories">5 Categories of Filer</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6038.th.cat">Category</th>
                    <th data-i18n="view.s6038.th.who">Who</th>
                    <th data-i18n="view.s6038.th.schedules">Schedules required</th>
                </tr></thead>
                <tbody>
                    <tr><td>1</td><td data-i18n="view.s6038.cat.1">US shareholder ≥ 10% in foreign passive investment company (§ 1297 PFIC overlap with 8621)</td><td>FYS, GILTI (§ 951A)</td></tr>
                    <tr><td>2</td><td data-i18n="view.s6038.cat.2">US officer / director of foreign corp + US person owning ≥ 10%</td><td>A</td></tr>
                    <tr><td>3</td><td data-i18n="view.s6038.cat.3">US person acquires ≥ 10% stock OR additional 5% during year</td><td>A, B, C, F</td></tr>
                    <tr><td>4</td><td data-i18n="view.s6038.cat.4">US person controls (&gt; 50%) for ≥ 30 days</td><td>A, B, C, F, G, H, I, J, K, L, M, P, Q, R</td></tr>
                    <tr><td>5</td><td data-i18n="view.s6038.cat.5">US shareholder owning ≥ 10% of CFC at year end</td><td>I, J, P (Subpart F + § 956 + GILTI)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038.h2.add">Log CFC ownership</h2>
            <form id="s6038-form" class="inline-form">
                <label><span data-i18n="view.s6038.label.name">Foreign corp name</span>
                    <input type="text" name="name" required></label>
                <label><span data-i18n="view.s6038.label.country">Country</span>
                    <input type="text" name="country" required></label>
                <label><span data-i18n="view.s6038.label.ownership">Your ownership %</span>
                    <input type="number" step="0.01" min="0" max="100" name="ownership_pct" required></label>
                <label><span data-i18n="view.s6038.label.us_aggregate">Total US ownership %</span>
                    <input type="number" step="0.01" min="0" max="100" name="us_aggregate_pct" required></label>
                <label><span data-i18n="view.s6038.label.subpart_f">Subpart F income (your share) ($)</span>
                    <input type="number" step="100" name="subpart_f_income" value="0"></label>
                <label><span data-i18n="view.s6038.label.gilti">GILTI inclusion (your share) ($)</span>
                    <input type="number" step="100" name="gilti_inclusion" value="0"></label>
                <label><span data-i18n="view.s6038.label.acq_year">Acquisition year</span>
                    <input type="number" step="1" name="acquired_year" value="${new Date().getFullYear()}"></label>
                <label><span data-i18n="view.s6038.label.years_missed">Years missed filing</span>
                    <input type="number" step="1" name="years_missed_filing" value="0"></label>
                <button class="primary" type="submit" data-i18n="view.s6038.btn.add">Add</button>
            </form>
        </div>
        <div id="s6038-summary"></div>
        <div id="s6038-table" class="chart-panel"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038.h2.streamlined">Streamlined Filing Compliance</h2>
            <p class="muted" data-i18n="view.s6038.stream.body">
                If you missed Form 5471 filings: Streamlined Domestic Offshore Procedures may
                allow late filing with reduced/waived penalties IF non-willful. Required:
                3 years amended returns + 6 years FBARs + 5% offshore-account penalty (Domestic version).
                Foreign Streamlined: no penalty if non-resident. <strong>Voluntary Disclosure Program</strong>
                if willful: penalty up to 50% of high balance.
            </p>
        </div>
    `;
    document.getElementById('s6038-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.cfcs.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            name: fd.get('name'),
            country: fd.get('country'),
            ownership_pct: Number(fd.get('ownership_pct')) || 0,
            us_aggregate_pct: Number(fd.get('us_aggregate_pct')) || 0,
            subpart_f_income: Number(fd.get('subpart_f_income')) || 0,
            gilti_inclusion: Number(fd.get('gilti_inclusion')) || 0,
            acquired_year: Number(fd.get('acquired_year')) || new Date().getFullYear(),
            years_missed_filing: Number(fd.get('years_missed_filing')) || 0,
        });
        save(state.cfcs);
        e.target.reset();
        e.target.querySelector('[name="acquired_year"]').value = new Date().getFullYear();
        showToast(t('view.s6038.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function categoryFor(c) {
    const usOwn = c.us_aggregate_pct / 100;
    const myOwn = c.ownership_pct / 100;
    if (usOwn > CFC_THRESHOLD_US_OWNERSHIP) {
        if (myOwn > CFC_THRESHOLD_US_OWNERSHIP) return 4;
        if (myOwn >= TEN_PCT_OWNERSHIP) return 5;
    }
    if (myOwn >= TEN_PCT_OWNERSHIP) return 3;
    return null;
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('s6038-summary');
    if (!el) return;
    const totalCfcs = state.cfcs.length;
    const totalSubpartF = state.cfcs.reduce((s, c) => s + c.subpart_f_income, 0);
    const totalGilti = state.cfcs.reduce((s, c) => s + c.gilti_inclusion, 0);
    const totalMissedFilings = state.cfcs.reduce((s, c) => s + c.years_missed_filing, 0);
    const totalPenalty = state.cfcs.reduce((s, c) => s + Math.min(c.years_missed_filing * PENALTY_PER_FAILURE, MAX_PENALTY * c.years_missed_filing / Math.max(1, c.years_missed_filing)), 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6038.h2.summary">Summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6038.card.count">CFCs / holdings</div>
                    <div class="value">${totalCfcs}</div>
                </div>
                <div class="card ${totalSubpartF > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s6038.card.subpart_f">Total Subpart F</div>
                    <div class="value">$${totalSubpartF.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${totalGilti > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s6038.card.gilti">Total GILTI</div>
                    <div class="value">$${totalGilti.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${totalMissedFilings > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6038.card.missed">Years missed</div>
                    <div class="value">${totalMissedFilings}</div>
                </div>
                <div class="card ${totalPenalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6038.card.penalty">Est. penalty exposure</div>
                    <div class="value">$${totalPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('s6038-table');
    if (!el) return;
    if (!state.cfcs.length) {
        el.innerHTML = `<h2 data-i18n="view.s6038.h2.cfcs">CFCs</h2>
            <p class="muted" data-i18n="view.s6038.empty">No CFCs logged.</p>`;
        return;
    }
    el.innerHTML = `
        <h2 data-i18n="view.s6038.h2.cfcs">CFCs</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s6038.th.name">Name</th>
                <th data-i18n="view.s6038.th.country">Country</th>
                <th data-i18n="view.s6038.th.your_pct">Your %</th>
                <th data-i18n="view.s6038.th.cat_required">Category</th>
                <th data-i18n="view.s6038.th.subpart_f">Subpart F</th>
                <th data-i18n="view.s6038.th.gilti">GILTI</th>
                <th data-i18n="view.s6038.th.missed">Missed</th>
                <th data-i18n="view.s6038.th.actions">Actions</th>
            </tr></thead>
            <tbody>${state.cfcs.map(c => {
                const cat = categoryFor(c);
                return `<tr>
                    <td>${esc(c.name)}</td>
                    <td class="muted">${esc(c.country)}</td>
                    <td>${c.ownership_pct.toFixed(2)}%</td>
                    <td class="${cat ? 'neg' : 'muted'}">${cat ? 'Cat ' + cat : '—'}</td>
                    <td>$${c.subpart_f_income.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>$${c.gilti_inclusion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="${c.years_missed_filing > 0 ? 'neg' : 'muted'}">${c.years_missed_filing}</td>
                    <td><button class="link neg" data-del="${esc(c.id)}" data-i18n="view.s6038.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.cfcs = state.cfcs.filter(c => c.id !== btn.dataset.del);
            save(state.cfcs);
            render();
        });
    });
}
