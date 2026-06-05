// § 1202 QSBS (Qualified Small Business Stock) Tracker.
// 100% federal exclusion of gain (up to $10M or 10× basis, whichever greater)
// on QSBS held > 5 years. Acquired 2010-09-28+. C-corp, < $50M gross assets at
// issue, 80% active business, NOT services/finance/farming/extraction.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-qsbs-v1';
const PER_ISSUER_CAP_BASE = 10_000_000;
const BASIS_MULTIPLIER = 10;

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = { holdings: load() };

export async function renderQsbs1202(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.qsbs.h1.title">// § 1202 QSBS TRACKER</span></h1>
        <p class="muted small" data-i18n="view.qsbs.hint.intro">
            <strong>100% federal exclusion</strong> of gain on Qualified Small Business
            Stock held &gt; 5 years (acquired 2010-09-28+). Per-issuer cap: greater of
            $10M or 10× cost basis. Excluded gain also escapes NIIT 3.8% and (for some)
            state tax.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.qsbs.h2.add">Add holding</h2>
            <form id="qs-form" class="inline-form">
                <label><span data-i18n="view.qsbs.label.company">Company name</span>
                    <input type="text" name="company" required></label>
                <label><span data-i18n="view.qsbs.label.acquisition_date">Acquisition date</span>
                    <input type="date" name="acquisition_date" required></label>
                <label><span data-i18n="view.qsbs.label.cost_basis">Cost basis ($)</span>
                    <input type="number" step="0.01" name="cost_basis" required></label>
                <label><span data-i18n="view.qsbs.label.current_value">Current FMV ($)</span>
                    <input type="number" step="0.01" name="current_value" required></label>
                <label><span data-i18n="view.qsbs.label.gross_assets_at_issue">Gross assets at issue ($)</span>
                    <input type="number" step="0.01" name="gross_assets_at_issue" placeholder="< 50M required"></label>
                <label><span data-i18n="view.qsbs.label.business_type">Business type</span>
                    <select name="business_type">
                        <option value="qualified">Qualified (SaaS / hardware / biotech / consumer)</option>
                        <option value="excluded_services">EXCLUDED — professional services</option>
                        <option value="excluded_finance">EXCLUDED — banking / finance / brokerage</option>
                        <option value="excluded_farming">EXCLUDED — farming / extraction</option>
                        <option value="excluded_hospitality">EXCLUDED — hotel / restaurant</option>
                    </select>
                </label>
                <label><span data-i18n="view.qsbs.label.c_corp">C-corp at issue?</span>
                    <input type="checkbox" name="is_c_corp" checked></label>
                <label><span data-i18n="view.qsbs.label.original_issue">Original issue (not secondary)?</span>
                    <input type="checkbox" name="original_issue" checked></label>
                <button class="primary" type="submit" data-i18n="view.qsbs.btn.add">Add</button>
            </form>
        </div>
        <div id="qs-summary"></div>
        <div id="qs-table" class="chart-panel"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.qsbs.h2.requirements">Qualification requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.qsbs.req.c_corp">Issued by US C-corp (or eligible entity)</li>
                <li data-i18n="view.qsbs.req.assets">Gross assets ≤ $50M at issue and immediately after</li>
                <li data-i18n="view.qsbs.req.original">Acquired at ORIGINAL ISSUE (not secondary market)</li>
                <li data-i18n="view.qsbs.req.active">80%+ of assets used in active business throughout holding period</li>
                <li data-i18n="view.qsbs.req.qualified">Qualified trade or business (NOT services, finance, farming, extraction, hospitality)</li>
                <li data-i18n="view.qsbs.req.cash_payment">Paid cash, property, or service (not stock-for-stock)</li>
                <li data-i18n="view.qsbs.req.5_year">Hold &gt; 5 years before sale</li>
            </ol>
        </div>
    `;
    document.getElementById('qs-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const h = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            company: fd.get('company'),
            acquisition_date: fd.get('acquisition_date'),
            cost_basis: Number(fd.get('cost_basis')),
            current_value: Number(fd.get('current_value')),
            gross_assets_at_issue: Number(fd.get('gross_assets_at_issue')) || 0,
            business_type: fd.get('business_type'),
            is_c_corp: !!fd.get('is_c_corp'),
            original_issue: !!fd.get('original_issue'),
        };
        state.holdings.push(h);
        save(state.holdings);
        e.target.reset();
        e.target.querySelector('[name="is_c_corp"]').checked = true;
        e.target.querySelector('[name="original_issue"]').checked = true;
        showToast(t('view.qsbs.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function analyzeHolding(h) {
    const today = new Date();
    const acq = new Date(h.acquisition_date);
    const yearsHeld = (today.getTime() - acq.getTime()) / (365.25 * 86_400_000);
    const gain = h.current_value - h.cost_basis;
    const perIssuerCap = Math.max(PER_ISSUER_CAP_BASE, h.cost_basis * BASIS_MULTIPLIER);
    const excludableGain = Math.min(Math.max(0, gain), perIssuerCap);
    const qualifyingChecks = {
        acq_date_ok: acq >= new Date('2010-09-28'),
        c_corp: h.is_c_corp,
        original_issue: h.original_issue,
        gross_assets_ok: h.gross_assets_at_issue > 0 && h.gross_assets_at_issue <= 50_000_000,
        qualified_business: h.business_type === 'qualified',
        five_year_held: yearsHeld >= 5,
    };
    const allOk = Object.values(qualifyingChecks).every(Boolean);
    return { yearsHeld, gain, perIssuerCap, excludableGain, qualifyingChecks, allOk };
}

function render() {
    const yearTotals = state.holdings.reduce((acc, h) => {
        const a = analyzeHolding(h);
        if (a.allOk) {
            acc.totalGain += a.gain;
            acc.totalExcludable += a.excludableGain;
        }
        return acc;
    }, { totalGain: 0, totalExcludable: 0 });
    renderSummary(yearTotals);
    renderTable();
}

function renderSummary({ totalGain, totalExcludable }) {
    const el = document.getElementById('qs-summary');
    if (!el) return;
    const federalSavings = totalExcludable * 0.20;
    const niitSavings = totalExcludable * 0.038;
    const totalSavings = federalSavings + niitSavings;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.qsbs.h2.summary">Portfolio summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.qsbs.card.holdings">Holdings</div>
                    <div class="value">${state.holdings.length}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.qsbs.card.total_gain">Total qualifying gain</div>
                    <div class="value">$${totalGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.qsbs.card.excludable">Excludable gain (§ 1202)</div>
                    <div class="value">$${totalExcludable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.qsbs.card.fed_savings">Federal cap-gains saved</div>
                    <div class="value">$${federalSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.qsbs.card.niit_savings">NIIT saved</div>
                    <div class="value">$${niitSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.qsbs.card.total_savings">Total tax saved</div>
                    <div class="value">$${totalSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('qs-table');
    if (!el) return;
    if (!state.holdings.length) {
        el.innerHTML = `<h2 data-i18n="view.qsbs.h2.holdings">Holdings</h2>
            <p class="muted" data-i18n="view.qsbs.empty">No QSBS holdings tracked yet.</p>`;
        return;
    }
    const sorted = [...state.holdings].sort((a, b) => b.current_value - a.current_value);
    el.innerHTML = `
        <h2 data-i18n="view.qsbs.h2.holdings">Holdings</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.qsbs.th.company">Company</th>
                <th data-i18n="view.qsbs.th.held">Years held</th>
                <th data-i18n="view.qsbs.th.basis">Cost basis</th>
                <th data-i18n="view.qsbs.th.fmv">Current FMV</th>
                <th data-i18n="view.qsbs.th.gain">Embedded gain</th>
                <th data-i18n="view.qsbs.th.cap">Per-issuer cap</th>
                <th data-i18n="view.qsbs.th.excludable">Excludable</th>
                <th data-i18n="view.qsbs.th.status">Status</th>
                <th data-i18n="view.qsbs.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(h => {
                const a = analyzeHolding(h);
                const cls = a.allOk ? 'pos' : 'neg';
                const status = a.allOk
                    ? t('view.qsbs.status.qualified')
                    : !a.qualifyingChecks.five_year_held
                        ? t('view.qsbs.status.wait_5yr', { yrs: a.yearsHeld.toFixed(1) })
                        : t('view.qsbs.status.disqualified');
                return `<tr>
                    <td>${esc(h.company)}</td>
                    <td>${a.yearsHeld.toFixed(1)}</td>
                    <td>$${h.cost_basis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>$${h.current_value.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="${a.gain >= 0 ? 'pos' : 'neg'}">$${a.gain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">$${a.perIssuerCap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="${cls}">$${a.excludableGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="${cls}">${esc(status)}</td>
                    <td><button class="link neg" data-del="${esc(h.id)}" data-i18n="view.qsbs.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.holdings = state.holdings.filter(h => h.id !== btn.dataset.del);
            save(state.holdings);
            render();
        });
    });
}
