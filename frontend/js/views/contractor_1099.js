// 1099-NEC contractor payment summary generator — totals a year's payments,
// applies the reporting threshold and backup withholding, via /calc/contractor-1099.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

const SEED = [
    { date: '2025-01-15', description: 'Design retainer', amount: 5000 },
    { date: '2025-04-10', description: 'Phase 2 build', amount: 5000 },
    { date: '2025-08-01', description: 'Final delivery', amount: 5000 },
];

function rowHtml(p) {
    return `
        <div class="mpb-row nec-row">
            <input type="text" class="nec-desc" placeholder="${esc(t('view.nec.ph.desc'))}" value="${esc(p.description || '')}">
            <input type="date" class="nec-date" value="${esc(p.date || '')}">
            <input type="number" step="0.01" min="0" class="nec-amount" value="${p.amount}">
            <button type="button" class="nec-del" data-i18n="view.nec.remove">Remove</button>
        </div>`;
}

export async function renderContractor1099(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.nec.h1.title">// 1099-NEC PAYMENT SUMMARY</span></h1>
        <p class="muted small" data-i18n="view.nec.hint.intro">
            A payer's year-end summary of nonemployee compensation paid to an independent contractor. It
            totals the year's payments, applies the $600 reporting threshold (a 1099-NEC is required at or
            above it), and computes backup withholding (24% when the payer must withhold) and net paid.
            Drafting aid, not tax advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.nec.h2.inputs">Summary details</h2>
            <form id="nec-form" class="inline-form">
                <label><span data-i18n="view.nec.label.payer">Payer (business)</span>
                    <input type="text" name="payer_name" value="Acme Co" required></label>
                <label><span data-i18n="view.nec.label.contractor">Contractor</span>
                    <input type="text" name="contractor_name" value="Jane Freelancer" required></label>
                <label><span data-i18n="view.nec.label.year">Tax year</span>
                    <input type="text" name="tax_year" value="2025" required></label>
                <label><span data-i18n="view.nec.label.threshold">Reporting threshold ($)</span>
                    <input type="number" step="50" min="0" name="reporting_threshold_usd" value="600"></label>
                <label><span data-i18n="view.nec.label.backup">Subject to backup withholding</span>
                    <input type="checkbox" name="subject_to_backup_withholding"></label>
                <label><span data-i18n="view.nec.label.rate">Backup rate (%)</span>
                    <input type="number" step="0.1" min="0" name="backup_rate_pct" value="24"></label>
                <label><span data-i18n="view.nec.label.date">Statement date</span>
                    <input type="date" name="date" value="2026-01-31" required></label>
                <label><span data-i18n="view.nec.label.note">Note (optional)</span>
                    <input type="text" name="note" value="" placeholder="${esc(t('view.nec.ph.note'))}"></label>
            </form>
            <div class="mpb-head nec-head">
                <span data-i18n="view.nec.col.desc">Description</span>
                <span data-i18n="view.nec.col.date">Date</span>
                <span data-i18n="view.nec.col.amount">Amount ($)</span>
                <span></span>
            </div>
            <div id="nec-rows">${SEED.map(rowHtml).join('')}</div>
            <button type="button" id="nec-add" class="secondary" data-i18n="view.nec.add">+ Add payment</button>
        </div>
        <div id="nec-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#nec-form');
    const rowsEl = mount.querySelector('#nec-rows');

    const generate = async () => {
        const fd = new FormData(form);
        const payments = [...rowsEl.querySelectorAll('.nec-row')].map((r) => ({
            description: (r.querySelector('.nec-desc').value || '').trim(),
            date: r.querySelector('.nec-date').value,
            amount_usd: Number(r.querySelector('.nec-amount').value) || 0,
        })).filter((p) => p.date);
        const body = {
            payer_name: (fd.get('payer_name') || '').trim(),
            contractor_name: (fd.get('contractor_name') || '').trim(),
            tax_year: (fd.get('tax_year') || '').trim(),
            payments,
            subject_to_backup_withholding: fd.get('subject_to_backup_withholding') === 'on',
            backup_rate_pct: Number(fd.get('backup_rate_pct')) || 0,
            reporting_threshold_usd: Number(fd.get('reporting_threshold_usd')) || 0,
            date: fd.get('date'),
            note: (fd.get('note') || '').trim(),
        };
        try {
            const doc = await api.calcContractor1099(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.nec.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);

    mount.querySelector('#nec-add').addEventListener('click', () => {
        rowsEl.insertAdjacentHTML('beforeend', rowHtml({ date: '', description: '', amount: 0 }));
        applyUiI18n(rowsEl.lastElementChild);
        generate();
    });
    rowsEl.addEventListener('click', (e) => {
        if (e.target.classList.contains('nec-del')) {
            e.target.closest('.nec-row').remove();
            generate();
        }
    });
    form.addEventListener('input', () => { live(); });
    rowsEl.addEventListener('input', () => { live(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase(), ''];
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const reportable = doc.reportable
        ? `<span class="pos" data-i18n="view.nec.reportable">1099 required</span>`
        : `<span class="muted" data-i18n="view.nec.not_reportable">Below threshold</span>`;
    const bwCard = doc.backup_withholding_usd > 0
        ? `<div class="card neg"><div class="label" data-i18n="view.nec.card.backup">Backup withholding</div>
               <div class="value">${money(doc.backup_withholding_usd)}</div></div>`
        : '';
    const el = mount.querySelector('#nec-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.nec.card.total">Total paid</div>
                    <div class="value">${money(doc.total_paid_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.nec.card.net">Net to contractor</div>
                    <div class="value">${money(doc.net_paid_usd)}</div></div>
                ${bwCard}
                <div class="card"><div class="label" data-i18n="view.nec.card.status">Status</div>
                    <div class="value">${reportable}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="nec-copy" type="button" data-i18n="view.nec.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="nec-download" type="button" data-i18n="view.nec.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#nec-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.nec.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.nec.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#nec-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'contractor-1099.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
