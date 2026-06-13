// LLC operating agreement generator — member ownership-% split from capital
// contributions, via /calc/llc-operating-agreement. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

const SEED = [
    { name: 'Alice', capital: 60000 },
    { name: 'Bob', capital: 40000 },
];

function rowHtml(m) {
    return `
        <div class="mpb-row llc-row">
            <input type="text" class="llc-name" placeholder="${esc(t('view.llc.ph.name'))}" value="${esc(m.name || '')}">
            <input type="number" step="100" min="0" class="llc-capital" value="${m.capital}">
            <button type="button" class="llc-del" data-i18n="view.llc.remove">Remove</button>
        </div>`;
}

export async function renderLlcOperatingAgreement(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.llc.h1.title">// LLC OPERATING AGREEMENT</span></h1>
        <p class="muted small" data-i18n="view.llc.hint.intro">
            Governs a limited liability company among its members. Each member's capital contribution
            determines their ownership percentage (capital ÷ total capital), which drives profit/loss
            allocation and distributions. It computes the splits and assembles the operative clauses.
            Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.llc.h2.inputs">Company details</h2>
            <form id="llc-form" class="inline-form">
                <label><span data-i18n="view.llc.label.name">LLC name</span>
                    <input type="text" name="llc_name" value="Widgets LLC" required></label>
                <label><span data-i18n="view.llc.label.state">Formation state</span>
                    <input type="text" name="formation_state" value="Delaware" required></label>
                <label><span data-i18n="view.llc.label.date">Formation date</span>
                    <input type="date" name="formation_date" value="2026-07-01" required></label>
                <label><span data-i18n="view.llc.label.mgmt">Management</span>
                    <select name="management">
                        <option value="member_managed" data-i18n="view.llc.mgmt.member">Member-managed</option>
                        <option value="manager_managed" data-i18n="view.llc.mgmt.manager">Manager-managed</option>
                    </select></label>
                <label><span data-i18n="view.llc.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.llc.ph.statute'))}"></label>
            </form>
            <div class="mpb-head llc-head">
                <span data-i18n="view.llc.col.name">Member</span>
                <span data-i18n="view.llc.col.capital">Capital ($)</span>
                <span></span>
            </div>
            <div id="llc-rows">${SEED.map(rowHtml).join('')}</div>
            <button type="button" id="llc-add" class="secondary" data-i18n="view.llc.add">+ Add member</button>
        </div>
        <div id="llc-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#llc-form');
    const rowsEl = mount.querySelector('#llc-rows');

    const generate = async () => {
        const fd = new FormData(form);
        const members = [...rowsEl.querySelectorAll('.llc-row')].map((r) => ({
            name: (r.querySelector('.llc-name').value || '').trim(),
            capital_usd: Number(r.querySelector('.llc-capital').value) || 0,
        })).filter((m) => m.name);
        const body = {
            llc_name: (fd.get('llc_name') || '').trim(),
            formation_state: (fd.get('formation_state') || '').trim(),
            formation_date: fd.get('formation_date'),
            management: fd.get('management'),
            members,
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        if (!members.length) { mount.querySelector('#llc-result').innerHTML = ''; return; }
        try {
            const doc = await api.calcLlcOperatingAgreement(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.llc.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);

    mount.querySelector('#llc-add').addEventListener('click', () => {
        rowsEl.insertAdjacentHTML('beforeend', rowHtml({ name: '', capital: 0 }));
        applyUiI18n(rowsEl.lastElementChild);
        generate();
    });
    rowsEl.addEventListener('click', (e) => {
        if (e.target.classList.contains('llc-del')) {
            e.target.closest('.llc-row').remove();
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
    const el = mount.querySelector('#llc-result');
    const rows = doc.members.map((m) => `
        <tr><td>${esc(m.name)}</td><td>${money(m.capital_usd)}</td><td>${pct(m.ownership_pct)}</td></tr>
    `).join('');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.llc.card.members">Members</div>
                    <div class="value">${doc.member_count}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.llc.card.capital">Total capital</div>
                    <div class="value">${money(doc.total_capital_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="llc-copy" type="button" data-i18n="view.llc.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="llc-download" type="button" data-i18n="view.llc.btn.download">Download .txt</button>
            </div>
        </div>
        <table class="data-table">
            <thead><tr>
                <th data-i18n="view.llc.th.name">Member</th>
                <th data-i18n="view.llc.th.capital">Capital</th>
                <th data-i18n="view.llc.th.ownership">Ownership</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#llc-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.llc.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.llc.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#llc-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'llc-operating-agreement.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
