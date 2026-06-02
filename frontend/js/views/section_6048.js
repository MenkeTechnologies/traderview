// IRC § 6048 Form 3520 — Foreign Trusts + Large Foreign Gifts.
// Three reportable events: (1) creation / transfer to foreign trust;
// (2) distributions from foreign trust to US person; (3) receipt of > $100,000 gift
// from foreign individual / estate OR > $19,570 (2024) from foreign corp/partnership.
// Penalty: 35% of unreported amount (5% per month, min $10k). Form 3520-A annually.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-form-3520-v1';
const INDIV_GIFT_THRESHOLD = 100_000;
const CORP_GIFT_THRESHOLD_2024 = 19_570;

function load() { try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); } catch { return []; } }
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    events: load(),
};

export async function renderSection6048(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6048.h1.title">// § 6048 FORM 3520 FOREIGN TRUST / GIFT</span></h1>
        <p class="muted small" data-i18n="view.s6048.hint.intro">
            Three reportable events: <strong>(1) transfer to foreign trust</strong>;
            <strong>(2) distribution from foreign trust</strong>; <strong>(3) receipt of
            &gt; $100,000 gift</strong> from foreign individual / estate, OR
            &gt; $19,570 (2024) from foreign corp / partnership. Penalty: <strong>35% of
            unreported amount</strong> (5%/month, min $10k). <strong>Form 3520-A annually</strong>
            by trustee. <strong>U.S. owner of foreign grantor trust</strong> reports trust income
            on 1040 even if no distribution.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6048.h2.add">Log event</h2>
            <form id="s6048-form" class="inline-form">
                <label><span data-i18n="view.s6048.label.date">Date</span>
                    <input type="date" name="date" required></label>
                <label><span data-i18n="view.s6048.label.kind">Kind</span>
                    <select name="kind">
                        <option value="trust_transfer">Transfer TO foreign trust</option>
                        <option value="trust_distribution">Distribution FROM foreign trust</option>
                        <option value="gift_individual">Gift from foreign individual</option>
                        <option value="gift_estate">Gift from foreign estate</option>
                        <option value="gift_corp">Gift from foreign corp / partnership</option>
                        <option value="grantor_income">Foreign grantor trust ongoing income</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6048.label.amount">Amount ($)</span>
                    <input type="number" step="100" name="amount" required></label>
                <label><span data-i18n="view.s6048.label.country">Country</span>
                    <input type="text" name="country" required></label>
                <label><span data-i18n="view.s6048.label.donor">Donor / trust name</span>
                    <input type="text" name="donor" required></label>
                <label><span data-i18n="view.s6048.label.filed">Form 3520 filed?</span>
                    <input type="checkbox" name="filed"></label>
                <label><span data-i18n="view.s6048.label.months_late">Months late filing</span>
                    <input type="number" step="1" name="months_late" value="0"></label>
                <button class="primary" type="submit" data-i18n="view.s6048.btn.add">Add</button>
            </form>
        </div>
        <div id="s6048-summary"></div>
        <div id="s6048-table" class="chart-panel"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6048.h2.exclusions">NOT reportable on Form 3520</h2>
            <ul class="muted small">
                <li data-i18n="view.s6048.excl.us_trust">U.S. trust distributions (use K-1 instead)</li>
                <li data-i18n="view.s6048.excl.charitable">Distributions to U.S. charity</li>
                <li data-i18n="view.s6048.excl.medical_education">Direct tuition / medical payments by foreign donor</li>
                <li data-i18n="view.s6048.excl.canadian_rrsp">Canadian RRSP / RRIF (Rev. Proc. 2014-55 auto exclusion)</li>
                <li data-i18n="view.s6048.excl.below_threshold">Below threshold gift ($100k individual / $19,570 corp)</li>
                <li data-i18n="view.s6048.excl.qualified_employee_trust">Qualified employee benefit trust (separate Form 8891 / FBAR)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6048.h2.penalty_relief">Penalty relief</h2>
            <ul class="muted small">
                <li data-i18n="view.s6048.relief.reasonable">Reasonable cause showing — IRS Notice 97-34</li>
                <li data-i18n="view.s6048.relief.streamlined">Streamlined Foreign Offshore Procedures (non-resident)</li>
                <li data-i18n="view.s6048.relief.streamlined_dom">Streamlined Domestic Offshore (5% Title 26 miscellaneous penalty)</li>
                <li data-i18n="view.s6048.relief.delinquent">Delinquent International Information Return Submission Procedures (DIIRSP)</li>
                <li data-i18n="view.s6048.relief.vdp">Voluntary Disclosure Program if willful</li>
                <li data-i18n="view.s6048.relief.precluded">Quiet disclosure / amended return — NOT penalty relief</li>
            </ul>
        </div>
    `;
    document.getElementById('s6048-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.events.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            date: fd.get('date'),
            kind: fd.get('kind'),
            amount: Number(fd.get('amount')) || 0,
            country: fd.get('country'),
            donor: fd.get('donor'),
            filed: !!fd.get('filed'),
            months_late: Number(fd.get('months_late')) || 0,
        });
        save(state.events);
        e.target.reset();
        showToast(t('view.s6048.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function isReportable(event) {
    switch (event.kind) {
        case 'trust_transfer':
        case 'trust_distribution':
        case 'grantor_income':
            return true;
        case 'gift_individual':
        case 'gift_estate':
            return event.amount > INDIV_GIFT_THRESHOLD;
        case 'gift_corp':
            return event.amount > CORP_GIFT_THRESHOLD_2024;
        default:
            return false;
    }
}

function calculatePenalty(event) {
    if (!isReportable(event) || event.filed) return 0;
    const monthlyPct = 0.05;
    const cap = 0.35;
    const monthlyAccum = Math.min(monthlyPct * event.months_late, cap);
    return Math.max(10_000, event.amount * monthlyAccum);
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('s6048-summary');
    if (!el) return;
    const reportable = state.events.filter(isReportable);
    const unfiled = reportable.filter(e => !e.filed);
    const totalPenalty = state.events.reduce((s, e) => s + calculatePenalty(e), 0);
    const totalReportableValue = reportable.reduce((s, e) => s + e.amount, 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6048.h2.summary">Summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6048.card.count">Events logged</div>
                    <div class="value">${state.events.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6048.card.reportable">Reportable</div>
                    <div class="value">${reportable.length}</div>
                </div>
                <div class="card ${unfiled.length > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6048.card.unfiled">Unfiled (delinquent)</div>
                    <div class="value">${unfiled.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6048.card.value">Total reportable value</div>
                    <div class="value">$${totalReportableValue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${totalPenalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6048.card.penalty">Estimated penalty exposure</div>
                    <div class="value">$${totalPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('s6048-table');
    if (!el) return;
    if (!state.events.length) {
        el.innerHTML = `<h2 data-i18n="view.s6048.h2.events">Events</h2>
            <p class="muted" data-i18n="view.s6048.empty">No events logged.</p>`;
        return;
    }
    const sorted = [...state.events].sort((a, b) => (b.date || '').localeCompare(a.date || ''));
    el.innerHTML = `
        <h2 data-i18n="view.s6048.h2.events">Events</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s6048.th.date">Date</th>
                <th data-i18n="view.s6048.th.kind">Kind</th>
                <th data-i18n="view.s6048.th.amount">Amount</th>
                <th data-i18n="view.s6048.th.country">Country</th>
                <th data-i18n="view.s6048.th.reportable">Reportable</th>
                <th data-i18n="view.s6048.th.filed">Filed</th>
                <th data-i18n="view.s6048.th.penalty">Penalty</th>
                <th data-i18n="view.s6048.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(e => {
                const rep = isReportable(e);
                const pen = calculatePenalty(e);
                return `<tr>
                    <td class="muted">${esc(e.date || '')}</td>
                    <td class="muted">${esc(e.kind)}</td>
                    <td>$${e.amount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">${esc(e.country)}</td>
                    <td class="${rep ? 'neg' : 'pos'}">${rep ? esc(t('view.s6048.status.yes')) : esc(t('view.s6048.status.no'))}</td>
                    <td class="${e.filed ? 'pos' : 'neg'}">${e.filed ? esc(t('view.s6048.status.yes')) : esc(t('view.s6048.status.no'))}</td>
                    <td class="${pen > 0 ? 'neg' : ''}">$${pen.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td><button class="link neg" data-del="${esc(e.id)}" data-i18n="view.s6048.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
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
