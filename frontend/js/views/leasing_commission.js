// Leasing commission generator — tiered rate per lease-year on an escalating
// rent stream, via /calc/leasing-commission.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 }) + '%');
let LAST_DOC = null;

export async function renderLeasingCommission(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.leasecomm.h1.title">// LEASING COMMISSION</span></h1>
        <p class="muted small" data-i18n="view.leasecomm.hint.intro">
            The brokerage fee on a commercial lease — a tiered percentage applied per lease-year to that
            year's rent, summed over the term. The rate typically steps down after the early years and the
            rent escalates annually. It computes the per-year schedule, aggregate rent, total commission, and
            blended effective rate. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.leasecomm.h2.inputs">Commission terms</h2>
            <form id="leasecomm-form" class="inline-form">
                <label><span data-i18n="view.leasecomm.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.leasecomm.label.broker">Broker</span>
                    <input type="text" name="broker_name" value=""></label>
                <label><span data-i18n="view.leasecomm.label.landlord">Landlord</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.leasecomm.label.tenant">Tenant</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.leasecomm.label.property">Premises</span>
                    <input type="text" name="property_label" value="Suite 700"></label>
                <label><span data-i18n="view.leasecomm.label.rent">Year-1 rent ($)</span>
                    <input type="number" step="1000" min="0" name="year_one_rent_usd" value="100000" required></label>
                <label><span data-i18n="view.leasecomm.label.escalation">Annual escalation (%)</span>
                    <input type="number" step="0.1" min="0" name="annual_escalation_pct" value="3"></label>
                <label><span data-i18n="view.leasecomm.label.term">Term (years)</span>
                    <input type="number" step="1" min="1" name="term_years" value="10" required></label>
                <label><span data-i18n="view.leasecomm.label.tier1rate">Tier-1 rate (%)</span>
                    <input type="number" step="0.1" min="0" name="tier1_rate_pct" value="5" required></label>
                <label><span data-i18n="view.leasecomm.label.tier1years">Tier-1 years</span>
                    <input type="number" step="1" min="0" name="tier1_years" value="5" required></label>
                <label><span data-i18n="view.leasecomm.label.tier2rate">Tier-2 rate (%)</span>
                    <input type="number" step="0.1" min="0" name="tier2_rate_pct" value="2.5"></label>
                <label><span data-i18n="view.leasecomm.label.date">Date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.leasecomm.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.leasecomm.ph.statute'))}"></label>
            </form>
        </div>
        <div id="leasecomm-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#leasecomm-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            broker_name: (fd.get('broker_name') || '').trim(),
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            property_label: (fd.get('property_label') || '').trim(),
            year_one_rent_usd: Number(fd.get('year_one_rent_usd')) || 0,
            annual_escalation_pct: Number(fd.get('annual_escalation_pct')) || 0,
            term_years: Number(fd.get('term_years')) || 0,
            tier1_rate_pct: Number(fd.get('tier1_rate_pct')) || 0,
            tier1_years: Number(fd.get('tier1_years')) || 0,
            tier2_rate_pct: Number(fd.get('tier2_rate_pct')) || 0,
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcLeasingCommission(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.leasecomm.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase()];
    if (doc.statutory_citation) lines.push(doc.statutory_citation);
    lines.push('');
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const rows = doc.schedule.map((r) => `
        <tr><td>${r.year}</td><td>${money(r.rent_usd)}</td><td>${r.rate_pct}%</td><td>${money(r.commission_usd)}</td></tr>
    `).join('');
    const el = mount.querySelector('#leasecomm-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.leasecomm.card.total">Total commission</div>
                    <div class="value">${money(doc.total_commission_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.leasecomm.card.aggregate">Aggregate rent</div>
                    <div class="value">${money(doc.aggregate_rent_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.leasecomm.card.effective">Effective rate</div>
                    <div class="value">${pct(doc.effective_rate_pct)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="leasecomm-copy" type="button" data-i18n="view.leasecomm.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="leasecomm-download" type="button" data-i18n="view.leasecomm.btn.download">Download .txt</button>
            </div>
        </div>
        <table class="data-table">
            <thead><tr>
                <th data-i18n="view.leasecomm.th.year">Year</th>
                <th data-i18n="view.leasecomm.th.rent">Rent</th>
                <th data-i18n="view.leasecomm.th.rate">Rate</th>
                <th data-i18n="view.leasecomm.th.commission">Commission</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#leasecomm-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.leasecomm.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.leasecomm.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#leasecomm-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'leasing-commission.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
