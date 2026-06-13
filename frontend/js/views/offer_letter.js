// Employment offer letter generator — per-paycheck breakdown + offer clauses,
// via /calc/offer-letter. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderOfferLetter(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.offer.h1.title">// EMPLOYMENT OFFER LETTER</span></h1>
        <p class="muted small" data-i18n="view.offer.hint.intro">
            Extends a job offer with title, compensation, and start date. It breaks the annual salary into
            the per-paycheck amount for the chosen pay frequency and assembles the offer clauses
            (position, compensation, equity, benefits, at-will, contingencies). Drafting aid, not legal
            advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.offer.h2.inputs">Offer details</h2>
            <form id="offer-form" class="inline-form">
                <label><span data-i18n="view.offer.label.company">Company</span>
                    <input type="text" name="company_name" value=""></label>
                <label><span data-i18n="view.offer.label.candidate">Candidate</span>
                    <input type="text" name="candidate_name" value=""></label>
                <label><span data-i18n="view.offer.label.title">Job title</span>
                    <input type="text" name="job_title" value="Software Engineer"></label>
                <label><span data-i18n="view.offer.label.salary">Annual salary ($)</span>
                    <input type="number" step="100" min="0" name="annual_salary_usd" value="120000" required></label>
                <label><span data-i18n="view.offer.label.freq">Pay frequency</span>
                    <select name="pay_frequency">
                        <option value="weekly" data-i18n="view.offer.freq.weekly">Weekly</option>
                        <option value="biweekly" selected data-i18n="view.offer.freq.biweekly">Biweekly</option>
                        <option value="semimonthly" data-i18n="view.offer.freq.semimonthly">Semimonthly</option>
                        <option value="monthly" data-i18n="view.offer.freq.monthly">Monthly</option>
                    </select></label>
                <label><span data-i18n="view.offer.label.start">Start date</span>
                    <input type="date" name="start_date" value="2026-08-01" required></label>
                <label><span data-i18n="view.offer.label.bonus">Signing bonus ($)</span>
                    <input type="number" step="100" min="0" name="signing_bonus_usd" value="10000"></label>
                <label><span data-i18n="view.offer.label.equity">Equity (optional)</span>
                    <input type="text" name="equity_description" value="5,000 ISOs vesting over 4 years"></label>
                <label><span data-i18n="view.offer.label.exempt">FLSA-exempt (salaried)</span>
                    <input type="checkbox" name="exempt" checked></label>
                <label><span data-i18n="view.offer.label.state">State</span>
                    <input type="text" name="state" value="California" required></label>
            </form>
        </div>
        <div id="offer-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#offer-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            candidate_name: (fd.get('candidate_name') || '').trim(),
            job_title: (fd.get('job_title') || '').trim(),
            annual_salary_usd: Number(fd.get('annual_salary_usd')) || 0,
            pay_frequency: fd.get('pay_frequency'),
            start_date: fd.get('start_date'),
            signing_bonus_usd: Number(fd.get('signing_bonus_usd')) || 0,
            equity_description: (fd.get('equity_description') || '').trim(),
            exempt: fd.get('exempt') != null,
            state: (fd.get('state') || '').trim(),
        };
        try {
            const doc = await api.calcOfferLetter(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.offer.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#offer-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.offer.card.paycheck">Per paycheck</div>
                    <div class="value">${money(doc.per_paycheck_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.offer.card.salary">Annual salary</div>
                    <div class="value">${money(doc.annual_salary_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.offer.card.periods">Pay periods/yr</div>
                    <div class="value">${doc.periods_per_year}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="offer-copy" type="button" data-i18n="view.offer.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="offer-download" type="button" data-i18n="view.offer.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#offer-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.offer.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.offer.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#offer-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'offer-letter.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
