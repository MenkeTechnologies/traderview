// Pet addendum generator — up-front charges total + new monthly rent with pet
// rent, via /calc/pet-addendum. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderPetAddendum(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pet.h1.title">// PET ADDENDUM</span></h1>
        <p class="muted small" data-i18n="view.pet.hint.intro">
            Amends an existing lease to permit a pet on stated terms. It totals the up-front charges
            (refundable deposit + non-refundable fee) and adds any monthly pet rent to the base rent for a
            new monthly total, then assembles the addendum clauses. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.pet.h2.inputs">Addendum details</h2>
            <form id="pet-form" class="inline-form">
                <label><span data-i18n="view.pet.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="Texas" required></label>
                <label><span data-i18n="view.pet.label.landlord">Landlord name</span>
                    <input type="text" name="landlord_name" value=""></label>
                <label><span data-i18n="view.pet.label.tenant">Tenant name</span>
                    <input type="text" name="tenant_name" value=""></label>
                <label><span data-i18n="view.pet.label.premises">Premises address</span>
                    <input type="text" name="premises_address" value=""></label>
                <label><span data-i18n="view.pet.label.lease_date">Original lease date</span>
                    <input type="date" name="lease_date" value="2026-01-01" required></label>
                <label><span data-i18n="view.pet.label.pet">Pet description</span>
                    <input type="text" name="pet_description" value="1 dog, Labrador, 'Rex', 60 lbs"></label>
                <label><span data-i18n="view.pet.label.deposit">Refundable deposit ($)</span>
                    <input type="number" step="0.01" min="0" name="pet_deposit_usd" value="300"></label>
                <label><span data-i18n="view.pet.label.fee">Non-refundable fee ($)</span>
                    <input type="number" step="0.01" min="0" name="pet_fee_usd" value="200"></label>
                <label><span data-i18n="view.pet.label.pet_rent">Monthly pet rent ($)</span>
                    <input type="number" step="0.01" min="0" name="monthly_pet_rent_usd" value="50"></label>
                <label><span data-i18n="view.pet.label.current_rent">Current monthly rent ($)</span>
                    <input type="number" step="0.01" min="0" name="current_monthly_rent_usd" value="1500" required></label>
                <label><span data-i18n="view.pet.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.pet.ph.statute'))}"></label>
            </form>
        </div>
        <div id="pet-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#pet-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            landlord_name: (fd.get('landlord_name') || '').trim(),
            tenant_name: (fd.get('tenant_name') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            lease_date: fd.get('lease_date'),
            pet_description: (fd.get('pet_description') || '').trim(),
            pet_deposit_usd: Number(fd.get('pet_deposit_usd')) || 0,
            pet_fee_usd: Number(fd.get('pet_fee_usd')) || 0,
            monthly_pet_rent_usd: Number(fd.get('monthly_pet_rent_usd')) || 0,
            current_monthly_rent_usd: Number(fd.get('current_monthly_rent_usd')) || 0,
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcPetAddendum(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.pet.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#pet-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.pet.card.new_rent">New monthly rent</div>
                    <div class="value">${money(doc.new_monthly_rent_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pet.card.upfront">Up-front total</div>
                    <div class="value">${money(doc.total_upfront_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pet.card.pet_rent">Monthly pet rent</div>
                    <div class="value">${money(doc.monthly_pet_rent_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="pet-copy" type="button" data-i18n="view.pet.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="pet-download" type="button" data-i18n="view.pet.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#pet-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.pet.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.pet.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#pet-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'pet-addendum.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
