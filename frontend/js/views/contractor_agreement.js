// Independent contractor agreement (1099) generator — fee (fixed or hourly with
// estimate) and the operative clauses, via /calc/contractor-agreement. Previews
// live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderContractorAgreement(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ica.h1.title">// CONTRACTOR AGREEMENT (1099)</span></h1>
        <p class="muted small" data-i18n="view.ica.hint.intro">
            Engages a contractor for services as a non-employee. State the fee as fixed or hourly (it
            projects an estimated total from your hours) with net-N payment terms; it assembles the
            operative clauses — services, compensation, term, independent-contractor status (own taxes,
            1099, no benefits), confidentiality, work-product assignment, termination, and governing law.
            Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ica.h2.inputs">Agreement details</h2>
            <form id="ica-form" class="inline-form">
                <label><span data-i18n="view.ica.label.state">Governing state</span>
                    <input type="text" name="governing_state" value="Delaware" required></label>
                <label><span data-i18n="view.ica.label.client">Client name</span>
                    <input type="text" name="client_name" value=""></label>
                <label><span data-i18n="view.ica.label.client_address">Client address</span>
                    <input type="text" name="client_address" value=""></label>
                <label><span data-i18n="view.ica.label.contractor">Contractor name</span>
                    <input type="text" name="contractor_name" value=""></label>
                <label><span data-i18n="view.ica.label.contractor_address">Contractor address</span>
                    <input type="text" name="contractor_address" value=""></label>
                <label><span data-i18n="view.ica.label.services">Services</span>
                    <input type="text" name="services_description" value="Design and build a marketing website"></label>
                <label><span data-i18n="view.ica.label.fee_type">Fee type</span>
                    <select name="fee_type" id="ica-fee-type">
                        <option value="fixed" data-i18n="view.ica.fee.fixed">Fixed fee ($)</option>
                        <option value="hourly" data-i18n="view.ica.fee.hourly">Hourly rate ($/hr)</option>
                    </select></label>
                <label><span data-i18n="view.ica.label.fee_amount">Fee amount ($)</span>
                    <input type="number" step="0.01" min="0" name="fee_amount_usd" value="5000" required></label>
                <label id="ica-hours-wrap"><span data-i18n="view.ica.label.hours">Estimated hours</span>
                    <input type="number" step="0.5" min="0" name="estimated_hours" value="0"></label>
                <label><span data-i18n="view.ica.label.terms">Payment terms (net days)</span>
                    <input type="number" step="1" min="0" name="payment_terms_days" value="30" required></label>
                <label><span data-i18n="view.ica.label.start">Start date</span>
                    <input type="date" name="start_date" value="2026-07-01" required></label>
                <label><span data-i18n="view.ica.label.end">End date (blank = ongoing)</span>
                    <input type="date" name="end_date" value=""></label>
                <label><span data-i18n="view.ica.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.ica.ph.statute'))}"></label>
            </form>
        </div>
        <div id="ica-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ica-form');
    const feeType = mount.querySelector('#ica-fee-type');
    const hoursWrap = mount.querySelector('#ica-hours-wrap');
    const syncFields = () => { hoursWrap.style.display = feeType.value === 'hourly' ? '' : 'none'; };

    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            client_name: (fd.get('client_name') || '').trim(),
            client_address: (fd.get('client_address') || '').trim(),
            contractor_name: (fd.get('contractor_name') || '').trim(),
            contractor_address: (fd.get('contractor_address') || '').trim(),
            services_description: (fd.get('services_description') || '').trim(),
            fee_type: fd.get('fee_type'),
            fee_amount_usd: Number(fd.get('fee_amount_usd')) || 0,
            estimated_hours: Number(fd.get('estimated_hours')) || 0,
            payment_terms_days: Math.round(Number(fd.get('payment_terms_days')) || 0),
            start_date: fd.get('start_date'),
            end_date: fd.get('end_date') || '',
            governing_state: (fd.get('governing_state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcContractorAgreement(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.ica.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    feeType.addEventListener('change', () => { syncFields(); generate(); });
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    syncFields();
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
    const el = mount.querySelector('#ica-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.ica.card.total">Estimated total</div>
                    <div class="value">${doc.estimated_total_usd > 0 ? money(doc.estimated_total_usd) : '—'}</div></div>
                <div class="card"><div class="label" data-i18n="view.ica.card.fee">Fee</div>
                    <div class="value">${money(doc.fee_amount_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ica.card.terms">Payment terms</div>
                    <div class="value">${t('view.ica.netdays', { n: doc.payment_terms_days })}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="ica-copy" type="button" data-i18n="view.ica.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="ica-download" type="button" data-i18n="view.ica.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#ica-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.ica.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.ica.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#ica-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'contractor-agreement.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
