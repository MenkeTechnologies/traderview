// Workers' comp premium generator — per class-code manual premium (payroll/100 ×
// rate), experience modifier, via /calc/workers-comp-premium.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

const SEED = [
    { code: '8810', description: 'Clerical', payroll: 300000, rate: 2.00 },
    { code: '5403', description: 'Carpentry', payroll: 200000, rate: 5.00 },
];

function rowHtml(c) {
    return `
        <div class="mpb-row wcp-row">
            <input type="text" class="wcp-code" placeholder="${esc(t('view.wcp.ph.code'))}" value="${esc(c.code || '')}">
            <input type="text" class="wcp-desc" placeholder="${esc(t('view.wcp.ph.desc'))}" value="${esc(c.description || '')}">
            <input type="number" step="1000" min="0" class="wcp-payroll" value="${c.payroll}">
            <input type="number" step="0.01" min="0" class="wcp-rate" value="${c.rate}">
            <button type="button" class="wcp-del" data-i18n="view.wcp.remove">Remove</button>
        </div>`;
}

export async function renderWorkersComp(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.wcp.h1.title">// WORKERS' COMP PREMIUM</span></h1>
        <p class="muted small" data-i18n="view.wcp.hint.intro">
            The standard premium estimate for a workers' compensation policy. Each class code carries a rate
            per $100 of payroll; the manual premium is payroll ÷ 100 × rate, summed across class codes, and the
            modified premium applies the employer's experience modifier (below 1.0 is a credit, above 1.0 a
            surcharge). Drafting aid, not insurance/legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.wcp.h2.inputs">Policy details</h2>
            <form id="wcp-form" class="inline-form">
                <label><span data-i18n="view.wcp.label.insurer">Insurer</span>
                    <input type="text" name="insurer_name" value="State Fund" required></label>
                <label><span data-i18n="view.wcp.label.employer">Employer</span>
                    <input type="text" name="employer_name" value="Acme Co" required></label>
                <label><span data-i18n="view.wcp.label.period">Policy period</span>
                    <input type="text" name="policy_period" value="2026" required></label>
                <label><span data-i18n="view.wcp.label.emod">Experience modifier</span>
                    <input type="number" step="0.01" min="0" name="experience_mod" value="0.90"></label>
                <label><span data-i18n="view.wcp.label.date">Date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.wcp.label.note">Note (optional)</span>
                    <input type="text" name="note" value="" placeholder="${esc(t('view.wcp.ph.note'))}"></label>
            </form>
            <div class="mpb-head wcp-head">
                <span data-i18n="view.wcp.col.code">Class</span>
                <span data-i18n="view.wcp.col.desc">Description</span>
                <span data-i18n="view.wcp.col.payroll">Payroll ($)</span>
                <span data-i18n="view.wcp.col.rate">Rate/100</span>
                <span></span>
            </div>
            <div id="wcp-rows">${SEED.map(rowHtml).join('')}</div>
            <button type="button" id="wcp-add" class="secondary" data-i18n="view.wcp.add">+ Add class code</button>
        </div>
        <div id="wcp-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#wcp-form');
    const rowsEl = mount.querySelector('#wcp-rows');

    const generate = async () => {
        const fd = new FormData(form);
        const classes = [...rowsEl.querySelectorAll('.wcp-row')].map((r) => ({
            code: (r.querySelector('.wcp-code').value || '').trim(),
            description: (r.querySelector('.wcp-desc').value || '').trim(),
            payroll_usd: Number(r.querySelector('.wcp-payroll').value) || 0,
            rate_per_100_usd: Number(r.querySelector('.wcp-rate').value) || 0,
        })).filter((c) => c.code);
        const body = {
            insurer_name: (fd.get('insurer_name') || '').trim(),
            employer_name: (fd.get('employer_name') || '').trim(),
            policy_period: (fd.get('policy_period') || '').trim(),
            classes,
            experience_mod: Number(fd.get('experience_mod')) || 0,
            date: fd.get('date'),
            note: (fd.get('note') || '').trim(),
        };
        try {
            const doc = await api.calcWorkersComp(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.wcp.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);

    mount.querySelector('#wcp-add').addEventListener('click', () => {
        rowsEl.insertAdjacentHTML('beforeend', rowHtml({ code: '', description: '', payroll: 0, rate: 0 }));
        applyUiI18n(rowsEl.lastElementChild);
        generate();
    });
    rowsEl.addEventListener('click', (e) => {
        if (e.target.classList.contains('wcp-del')) {
            e.target.closest('.wcp-row').remove();
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
    const rows = doc.rows.map((r) => `
        <tr><td>${esc(r.code)}</td><td>${esc(r.description)}</td><td>${money(r.payroll_usd)}</td>
            <td>${r.rate_per_100_usd}</td><td>${money(r.manual_premium_usd)}</td></tr>
    `).join('');
    const el = mount.querySelector('#wcp-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.wcp.card.modified">Modified premium</div>
                    <div class="value">${money(doc.modified_premium_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.wcp.card.manual">Manual premium</div>
                    <div class="value">${money(doc.manual_premium_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.wcp.card.emod">Exp. modifier</div>
                    <div class="value">${doc.experience_mod}</div></div>
                <div class="card"><div class="label" data-i18n="view.wcp.card.payroll">Total payroll</div>
                    <div class="value">${money(doc.total_payroll_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="wcp-copy" type="button" data-i18n="view.wcp.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="wcp-download" type="button" data-i18n="view.wcp.btn.download">Download .txt</button>
            </div>
        </div>
        <table class="data-table">
            <thead><tr>
                <th data-i18n="view.wcp.th.code">Class</th>
                <th data-i18n="view.wcp.th.desc">Description</th>
                <th data-i18n="view.wcp.th.payroll">Payroll</th>
                <th data-i18n="view.wcp.th.rate">Rate/100</th>
                <th data-i18n="view.wcp.th.premium">Premium</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#wcp-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.wcp.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.wcp.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#wcp-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'workers-comp-premium.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
