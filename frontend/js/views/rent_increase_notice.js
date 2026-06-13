// Rent increase notice generator — new rent (percent or flat), change, and the
// effective date from the service date + notice period, via
// /calc/rent-increase-notice. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

export async function renderRentIncreaseNotice(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rinc.h1.title">// RENT INCREASE NOTICE</span></h1>
        <p class="muted small" data-i18n="view.rinc.hint.intro">
            The written notice a landlord serves before raising rent on a periodic tenancy. Enter the
            increase as a percent or a flat amount; it computes the new rent, the dollar and percent
            change, and the effective date from the service date plus the required notice period. Many
            states set the minimum notice (and sometimes a cap) by the size of the increase. Drafting
            aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.rinc.h2.inputs">Notice details</h2>
            <form id="rinc-form" class="inline-form">
                <label><span data-i18n="view.rinc.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="California" required></label>
                <label><span data-i18n="view.rinc.label.landlord_name">Landlord name</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.rinc.label.landlord_address">Landlord address</span>
                    <input type="text" name="landlord_address" value=""></label>
                <label><span data-i18n="view.rinc.label.landlord_phone">Landlord phone</span>
                    <input type="text" name="landlord_phone" value=""></label>
                <label><span data-i18n="view.rinc.label.tenant_name">Tenant name</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.rinc.label.premises">Premises address</span>
                    <input type="text" name="premises_address" value=""></label>
                <label><span data-i18n="view.rinc.label.current_rent">Current rent ($)</span>
                    <input type="number" step="0.01" min="0" name="current_rent_usd" value="1500" required></label>
                <label><span data-i18n="view.rinc.label.type">Increase by</span>
                    <select name="increase_type" id="rinc-type">
                        <option value="percent" data-i18n="view.rinc.type.percent">Percent (%)</option>
                        <option value="amount" data-i18n="view.rinc.type.amount">Flat amount ($)</option>
                    </select></label>
                <label><span data-i18n="view.rinc.label.value">Increase value</span>
                    <input type="number" step="0.01" min="0" name="increase_value" value="5" required></label>
                <label><span data-i18n="view.rinc.label.served">Date served</span>
                    <input type="date" name="served_date" value="2026-06-01" required></label>
                <label><span data-i18n="view.rinc.label.notice_days">Notice period (days)</span>
                    <input type="number" step="1" min="1" name="notice_days" value="60" required></label>
                <label><span data-i18n="view.rinc.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.rinc.ph.statute'))}"></label>
            </form>
        </div>
        <div id="rinc-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#rinc-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            landlord_address: (fd.get('landlord_address') || '').trim(),
            landlord_phone: (fd.get('landlord_phone') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            current_rent_usd: Number(fd.get('current_rent_usd')) || 0,
            increase_type: fd.get('increase_type'),
            increase_value: Number(fd.get('increase_value')) || 0,
            served_date: fd.get('served_date'),
            notice_days: Math.round(Number(fd.get('notice_days')) || 0),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcRentIncreaseNotice(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.rinc.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#rinc-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.rinc.card.new_rent">New rent</div>
                    <div class="value">${money(doc.new_rent_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rinc.card.increase">Increase</div>
                    <div class="value">${money(doc.increase_amount_usd)} · ${pct(doc.increase_pct)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.rinc.card.effective">Effective</div>
                    <div class="value">${esc(doc.effective_date || '—')}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="rinc-copy" type="button" data-i18n="view.rinc.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="rinc-download" type="button" data-i18n="view.rinc.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#rinc-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.rinc.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.rinc.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#rinc-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'rent-increase-notice.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
