// PTO accrual policy generator — annual accrual (hours/days) + policy clauses,
// via /calc/pto-policy. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderPtoPolicy(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pto.h1.title">// PTO ACCRUAL POLICY</span></h1>
        <p class="muted small" data-i18n="view.pto.hint.intro">
            States how employees accrue paid time off. It computes the annual accrual (rate per pay period
            × periods per year) in both hours and workdays, applies any carryover cap, and assembles the
            policy clauses (eligibility, accrual, carryover, usage, separation payout). Drafting aid, not
            legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.pto.h2.inputs">Policy details</h2>
            <form id="pto-form" class="inline-form">
                <label><span data-i18n="view.pto.label.company">Company</span>
                    <input type="text" name="company_name" value="Acme Inc" required></label>
                <label><span data-i18n="view.pto.label.rate">Accrual (hours / pay period)</span>
                    <input type="number" step="0.01" min="0" name="accrual_rate_hours_per_period" value="4" required></label>
                <label><span data-i18n="view.pto.label.periods">Pay periods / year</span>
                    <input type="number" step="1" min="1" name="pay_periods_per_year" value="26" required></label>
                <label><span data-i18n="view.pto.label.workday">Hours per workday</span>
                    <input type="number" step="0.5" min="1" name="hours_per_workday" value="8" required></label>
                <label><span data-i18n="view.pto.label.carryover">Carryover cap (hours)</span>
                    <input type="number" step="1" min="0" name="carryover_cap_hours" value="40"></label>
                <label><span data-i18n="view.pto.label.waiting">Waiting period (days)</span>
                    <input type="number" step="1" min="0" name="waiting_period_days" value="90"></label>
                <label><span data-i18n="view.pto.label.payout">Payout on separation</span>
                    <input type="checkbox" name="payout_on_separation" checked></label>
                <label><span data-i18n="view.pto.label.state">State</span>
                    <input type="text" name="state" value="California" required></label>
            </form>
        </div>
        <div id="pto-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#pto-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            accrual_rate_hours_per_period: Number(fd.get('accrual_rate_hours_per_period')) || 0,
            pay_periods_per_year: Math.round(Number(fd.get('pay_periods_per_year')) || 0),
            hours_per_workday: Number(fd.get('hours_per_workday')) || 8,
            carryover_cap_hours: Number(fd.get('carryover_cap_hours')) || 0,
            waiting_period_days: Math.round(Number(fd.get('waiting_period_days')) || 0),
            payout_on_separation: fd.get('payout_on_separation') != null,
            state: (fd.get('state') || '').trim(),
        };
        try {
            const doc = await api.calcPtoPolicy(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.pto.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase(), ''];
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#pto-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.pto.card.days">Annual PTO (days)</div>
                    <div class="value">${num(doc.annual_accrual_days)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pto.card.hours">Annual PTO (hours)</div>
                    <div class="value">${num(doc.annual_accrual_hours)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pto.card.cap">Carryover cap (h)</div>
                    <div class="value">${num(doc.carryover_cap_hours)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="pto-copy" type="button" data-i18n="view.pto.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="pto-download" type="button" data-i18n="view.pto.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#pto-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.pto.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.pto.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#pto-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'pto-policy.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
