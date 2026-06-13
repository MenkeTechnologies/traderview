// Roommate agreement generator — weighted rent/deposit split across roommates,
// via /calc/roommate-agreement. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

const SEED = [
    { name: 'Alice', weight: 1 },
    { name: 'Bob', weight: 1 },
    { name: 'Cara', weight: 1 },
];

function rowHtml(r) {
    return `
        <div class="mpb-row rma-row">
            <input type="text" class="rma-name" placeholder="${esc(t('view.rma.ph.name'))}" value="${esc(r.name || '')}">
            <input type="number" step="0.1" min="0" class="rma-weight" value="${r.weight}">
            <button type="button" class="rma-del" data-i18n="view.rma.remove">Remove</button>
        </div>`;
}

export async function renderRoommateAgreement(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rma.h1.title">// ROOMMATE AGREEMENT</span></h1>
        <p class="muted small" data-i18n="view.rma.hint.intro">
            Splits the rent and security deposit among co-tenants and records the house rules. Each
            roommate carries a weight — equal weights split evenly; unequal weights handle different room
            sizes — and the rent and deposit divide in proportion. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.rma.h2.inputs">Agreement details</h2>
            <form id="rma-form" class="inline-form">
                <label><span data-i18n="view.rma.label.premises">Premises address</span>
                    <input type="text" name="premises_address" value=""></label>
                <label><span data-i18n="view.rma.label.rent">Total monthly rent ($)</span>
                    <input type="number" step="0.01" min="0" name="total_monthly_rent_usd" value="2100" required></label>
                <label><span data-i18n="view.rma.label.deposit">Total deposit ($)</span>
                    <input type="number" step="0.01" min="0" name="total_deposit_usd" value="900"></label>
                <label><span data-i18n="view.rma.label.start">Lease start date</span>
                    <input type="date" name="lease_start_date" value="2026-08-01" required></label>
                <label><span data-i18n="view.rma.label.state">State (optional)</span>
                    <input type="text" name="state" value="Oregon"></label>
                <label><span data-i18n="view.rma.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.rma.ph.statute'))}"></label>
            </form>
            <div class="mpb-head rma-head">
                <span data-i18n="view.rma.col.name">Roommate</span>
                <span data-i18n="view.rma.col.weight">Weight</span>
                <span></span>
            </div>
            <div id="rma-rows">${SEED.map(rowHtml).join('')}</div>
            <button type="button" id="rma-add" class="secondary" data-i18n="view.rma.add">+ Add roommate</button>
        </div>
        <div id="rma-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#rma-form');
    const rowsEl = mount.querySelector('#rma-rows');

    const generate = async () => {
        const fd = new FormData(form);
        const roommates = [...rowsEl.querySelectorAll('.rma-row')].map((r) => ({
            name: (r.querySelector('.rma-name').value || '').trim(),
            weight: Number(r.querySelector('.rma-weight').value) || 1,
        })).filter((r) => r.name);
        const body = {
            premises_address: (fd.get('premises_address') || '').trim(),
            total_monthly_rent_usd: Number(fd.get('total_monthly_rent_usd')) || 0,
            total_deposit_usd: Number(fd.get('total_deposit_usd')) || 0,
            lease_start_date: fd.get('lease_start_date'),
            roommates,
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        if (!roommates.length) { mount.querySelector('#rma-result').innerHTML = ''; return; }
        try {
            const doc = await api.calcRoommateAgreement(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.rma.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);

    mount.querySelector('#rma-add').addEventListener('click', () => {
        rowsEl.insertAdjacentHTML('beforeend', rowHtml({ name: '', weight: 1 }));
        applyUiI18n(rowsEl.lastElementChild);
        generate();
    });
    rowsEl.addEventListener('click', (e) => {
        if (e.target.classList.contains('rma-del')) {
            e.target.closest('.rma-row').remove();
            generate();
        }
    });
    form.addEventListener('input', () => { live(); });
    rowsEl.addEventListener('input', () => { live(); });
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
    const el = mount.querySelector('#rma-result');
    const rows = doc.shares.map((s) => `
        <tr><td>${esc(s.name)}</td><td>${money(s.rent_share_usd)}</td><td>${money(s.deposit_share_usd)}</td></tr>
    `).join('');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.rma.card.count">Roommates</div>
                    <div class="value">${doc.roommate_count}</div></div>
                <div class="card"><div class="label" data-i18n="view.rma.card.rent">Total rent</div>
                    <div class="value">${money(doc.total_monthly_rent_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rma.card.deposit">Total deposit</div>
                    <div class="value">${money(doc.total_deposit_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="rma-copy" type="button" data-i18n="view.rma.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="rma-download" type="button" data-i18n="view.rma.btn.download">Download .txt</button>
            </div>
        </div>
        <table class="data-table">
            <thead><tr>
                <th data-i18n="view.rma.th.name">Roommate</th>
                <th data-i18n="view.rma.th.rent">Rent / mo</th>
                <th data-i18n="view.rma.th.deposit">Deposit</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#rma-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.rma.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.rma.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#rma-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'roommate-agreement.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
