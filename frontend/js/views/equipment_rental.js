// Equipment rental agreement generator — rental total + deposit + return date,
// via /calc/equipment-rental. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderEquipmentRental(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.eqr.h1.title">// EQUIPMENT RENTAL</span></h1>
        <p class="muted small" data-i18n="view.eqr.hint.intro">
            Rents personal property — tools, vehicles, AV gear, machinery — for a fixed period. It
            computes the rental total from the rate and duration, adds the security deposit for the total
            due, and computes the return date from the rate period. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.eqr.h2.inputs">Rental details</h2>
            <form id="eqr-form" class="inline-form">
                <label><span data-i18n="view.eqr.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="Colorado" required></label>
                <label><span data-i18n="view.eqr.label.owner">Owner (lessor)</span>
                    <input type="text" name="owner_name" value=""></label>
                <label><span data-i18n="view.eqr.label.renter">Renter (lessee)</span>
                    <input type="text" name="renter_name" value=""></label>
                <label><span data-i18n="view.eqr.label.equipment">Equipment</span>
                    <input type="text" name="equipment_description" value="Mini excavator, model X120"></label>
                <label><span data-i18n="view.eqr.label.rate">Rate ($)</span>
                    <input type="number" step="0.01" min="0" name="rate_usd" value="100" required></label>
                <label><span data-i18n="view.eqr.label.period">Per</span>
                    <select name="rate_period">
                        <option value="day" data-i18n="view.eqr.period.day">Day</option>
                        <option value="week" data-i18n="view.eqr.period.week">Week</option>
                        <option value="month" data-i18n="view.eqr.period.month">Month</option>
                    </select></label>
                <label><span data-i18n="view.eqr.label.duration">Duration (periods)</span>
                    <input type="number" step="1" min="1" name="duration" value="5" required></label>
                <label><span data-i18n="view.eqr.label.deposit">Security deposit ($)</span>
                    <input type="number" step="0.01" min="0" name="security_deposit_usd" value="200"></label>
                <label><span data-i18n="view.eqr.label.start">Start date</span>
                    <input type="date" name="start_date" value="2026-06-01" required></label>
                <label><span data-i18n="view.eqr.label.late">Late fee per day ($)</span>
                    <input type="number" step="0.01" min="0" name="late_fee_per_day_usd" value="25"></label>
                <label><span data-i18n="view.eqr.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.eqr.ph.statute'))}"></label>
            </form>
        </div>
        <div id="eqr-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#eqr-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            owner_name: (fd.get('owner_name') || '').trim(),
            renter_name: (fd.get('renter_name') || '').trim(),
            equipment_description: (fd.get('equipment_description') || '').trim(),
            rate_usd: Number(fd.get('rate_usd')) || 0,
            rate_period: fd.get('rate_period'),
            duration: Math.round(Number(fd.get('duration')) || 0),
            security_deposit_usd: Number(fd.get('security_deposit_usd')) || 0,
            start_date: fd.get('start_date'),
            late_fee_per_day_usd: Number(fd.get('late_fee_per_day_usd')) || 0,
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcEquipmentRental(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.eqr.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#eqr-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.eqr.card.due">Total due</div>
                    <div class="value">${money(doc.total_due_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.eqr.card.rental">Rental total</div>
                    <div class="value">${money(doc.rental_total_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.eqr.card.return">Return by</div>
                    <div class="value">${esc(doc.return_date || '—')}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="eqr-copy" type="button" data-i18n="view.eqr.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="eqr-download" type="button" data-i18n="view.eqr.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#eqr-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.eqr.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.eqr.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#eqr-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'equipment-rental.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
