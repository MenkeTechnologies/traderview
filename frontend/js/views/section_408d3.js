// IRC § 408(d)(3) — Once-per-12-months IRA Indirect Rollover.
// Bobrow v. Comm'r (2014): the per-account interpretation was rejected. Now applies
// PER TAXPAYER across ALL IRAs (Traditional + Roth + SEP + SIMPLE).
// Second rollover within 12 mo: amount taxable + 10% penalty if < 59½ + may be excess
// contribution + 6% excise. Direct trustee-to-trustee transfers do NOT count.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-408d3-v1';

function load() { try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); } catch { return []; } }
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    rollovers: load(),
};

export async function renderSection408d3(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s408d3.h1.title">// § 408(d)(3) BOBROW IRA ROLLOVER</span></h1>
        <p class="muted small" data-i18n="view.s408d3.hint.intro">
            <strong>Bobrow v. Comm'r (2014)</strong>: one indirect rollover per 12 months
            applies <strong>PER TAXPAYER across ALL your IRAs</strong> (Traditional + Roth +
            SEP + SIMPLE). Pre-Bobrow misreading allowed per-account. Second indirect rollover
            within 12 mo: amount taxable + 10% penalty if &lt; 59½ + excess contribution + 6%
            excise. <strong>Trustee-to-trustee transfers do NOT count</strong> — unlimited.
            <strong>Roth conversions don't count.</strong>
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s408d3.h2.add">Log rollover</h2>
            <form id="s408d3-form" class="inline-form">
                <label><span data-i18n="view.s408d3.label.distribution_date">Distribution date</span>
                    <input type="date" name="distribution_date" required></label>
                <label><span data-i18n="view.s408d3.label.rollback_date">Rollover deposit date</span>
                    <input type="date" name="rollback_date"></label>
                <label><span data-i18n="view.s408d3.label.kind">Kind</span>
                    <select name="kind">
                        <option value="indirect">Indirect rollover (60-day, counts)</option>
                        <option value="direct">Trustee-to-trustee (does NOT count)</option>
                        <option value="conversion">Roth conversion (does NOT count)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s408d3.label.amount">Distribution amount ($)</span>
                    <input type="number" step="100" name="amount" required></label>
                <label><span data-i18n="view.s408d3.label.source_account">Source IRA</span>
                    <input type="text" name="source_account" required></label>
                <label><span data-i18n="view.s408d3.label.dest_account">Destination IRA</span>
                    <input type="text" name="dest_account"></label>
                <button class="primary" type="submit" data-i18n="view.s408d3.btn.add">Add</button>
            </form>
        </div>
        <div id="s408d3-summary"></div>
        <div id="s408d3-table" class="chart-panel"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s408d3.h2.workarounds">Workarounds</h2>
            <ul class="muted small">
                <li data-i18n="view.s408d3.work.direct">Always do <strong>direct trustee-to-trustee transfer</strong> — unlimited per year</li>
                <li data-i18n="view.s408d3.work.conversions">Roth conversions don't count toward the limit</li>
                <li data-i18n="view.s408d3.work.qualified">401(k)→IRA distributions are not subject (employer plan)</li>
                <li data-i18n="view.s408d3.work.60_days">If you do indirect: 60 days strict; banks must withhold 20% (less for IRA)</li>
                <li data-i18n="view.s408d3.work.self_certify">Late rollover relief: Rev. Proc. 2016-47 self-certification (11 reasons)</li>
                <li data-i18n="view.s408d3.work.private_letter">PLR for other reasons — $10k+ user fee</li>
                <li data-i18n="view.s408d3.work.spousal_separate">Spouse's IRA rollovers count separately (each spouse has own limit)</li>
            </ul>
        </div>
    `;
    document.getElementById('s408d3-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.rollovers.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            distribution_date: fd.get('distribution_date'),
            rollback_date: fd.get('rollback_date') || null,
            kind: fd.get('kind'),
            amount: Number(fd.get('amount')) || 0,
            source_account: fd.get('source_account'),
            dest_account: fd.get('dest_account') || '',
        });
        save(state.rollovers);
        e.target.reset();
        showToast(t('view.s408d3.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('s408d3-summary');
    if (!el) return;
    const indirect = state.rollovers.filter(r => r.kind === 'indirect')
        .sort((a, b) => (a.distribution_date || '').localeCompare(b.distribution_date || ''));
    let violations = 0, lastIndirect = null;
    let blockedUntil = null;
    for (const r of indirect) {
        if (lastIndirect) {
            const distDate = new Date(r.distribution_date);
            const lastDate = new Date(lastIndirect.distribution_date);
            const daysApart = (distDate - lastDate) / (24 * 3600 * 1000);
            if (daysApart < 365) violations++;
        }
        lastIndirect = r;
    }
    if (lastIndirect) {
        const next = new Date(lastIndirect.distribution_date);
        next.setDate(next.getDate() + 365);
        blockedUntil = next.toISOString().slice(0, 10);
    }
    const todayBlocked = blockedUntil && new Date() < new Date(blockedUntil);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s408d3.h2.summary">12-month window status</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s408d3.card.indirect_count">Indirect rollovers logged</div>
                    <div class="value">${indirect.length}</div>
                </div>
                <div class="card ${violations > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s408d3.card.violations">Bobrow violations</div>
                    <div class="value">${violations}</div>
                </div>
                <div class="card ${todayBlocked ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s408d3.card.blocked_until">Next allowed</div>
                    <div class="value">${blockedUntil || esc(t('view.s408d3.status.now'))}</div>
                </div>
            </div>
            ${violations > 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s408d3.warning.violation">
                    Multiple indirect rollovers within 12 months — second + later treated as
                    TAXABLE DISTRIBUTION + possible 10% penalty + 6% excess contribution excise.
                    File Form 5329 + consider self-certification.
                </p>
            ` : ''}
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('s408d3-table');
    if (!el) return;
    if (!state.rollovers.length) {
        el.innerHTML = `<h2 data-i18n="view.s408d3.h2.rollovers">Rollovers</h2>
            <p class="muted" data-i18n="view.s408d3.empty">No rollovers logged.</p>`;
        return;
    }
    const sorted = [...state.rollovers].sort((a, b) => (b.distribution_date || '').localeCompare(a.distribution_date || ''));
    el.innerHTML = `
        <h2 data-i18n="view.s408d3.h2.rollovers">Rollovers</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s408d3.th.dist_date">Dist date</th>
                <th data-i18n="view.s408d3.th.deposit">Deposit date</th>
                <th data-i18n="view.s408d3.th.kind">Kind</th>
                <th data-i18n="view.s408d3.th.amount">Amount</th>
                <th data-i18n="view.s408d3.th.source">Source</th>
                <th data-i18n="view.s408d3.th.dest">Dest</th>
                <th data-i18n="view.s408d3.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(r => {
                const kindCls = r.kind === 'indirect' ? 'neg' : 'pos';
                return `<tr>
                    <td class="muted">${esc(r.distribution_date || '')}</td>
                    <td class="muted">${esc(r.rollback_date || '—')}</td>
                    <td class="${kindCls}">${esc(r.kind)}</td>
                    <td>$${r.amount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">${esc(r.source_account)}</td>
                    <td class="muted">${esc(r.dest_account || '—')}</td>
                    <td><button class="link neg" data-del="${esc(r.id)}" data-i18n="view.s408d3.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.rollovers = state.rollovers.filter(r => r.id !== btn.dataset.del);
            save(state.rollovers);
            render();
        });
    });
}
