// Wage garnishment generator — CCPA Title III cap (lesser of % of disposable or
// amount above 30× minimum wage), via /calc/wage-garnishment.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderWageGarnishment(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.garnish.h1.title">// WAGE GARNISHMENT</span></h1>
        <p class="muted small" data-i18n="view.garnish.hint.intro">
            The maximum a creditor may garnish from a pay period under the federal CCPA (Title III): the
            lesser of a percentage of disposable earnings (25% for ordinary debt, higher for support) and the
            amount by which disposable earnings exceed 30× the minimum wage. It computes both caps, the
            protected floor, and the garnishment. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.garnish.h2.inputs">Garnishment inputs</h2>
            <form id="garnish-form" class="inline-form">
                <label><span data-i18n="view.garnish.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.garnish.label.creditor">Creditor</span>
                    <input type="text" name="creditor_name" value=""></label>
                <label><span data-i18n="view.garnish.label.employee">Employee</span>
                    <input type="text" name="employee_name" value=""></label>
                <label><span data-i18n="view.garnish.label.employer">Employer (garnishee)</span>
                    <input type="text" name="employer_name" value=""></label>
                <label><span data-i18n="view.garnish.label.disposable">Disposable earnings ($)</span>
                    <input type="number" step="10" min="0" name="disposable_earnings_usd" value="600" required></label>
                <label><span data-i18n="view.garnish.label.minwage">Minimum wage ($/hr)</span>
                    <input type="number" step="0.25" min="0" name="min_wage_usd" value="7.25"></label>
                <label><span data-i18n="view.garnish.label.pct">Cap on disposable (%)</span>
                    <input type="number" step="1" min="0" max="100" name="cap_pct" value="25"></label>
                <label><span data-i18n="view.garnish.label.mult">Min-wage multiplier</span>
                    <input type="number" step="1" min="0" name="min_wage_multiplier" value="30"></label>
                <label><span data-i18n="view.garnish.label.period">Pay period</span>
                    <input type="text" name="pay_period" value="weekly"></label>
                <label><span data-i18n="view.garnish.label.date">Date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.garnish.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.garnish.ph.statute'))}"></label>
            </form>
        </div>
        <div id="garnish-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#garnish-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            creditor_name: (fd.get('creditor_name') || '').trim(),
            employee_name: (fd.get('employee_name') || '').trim(),
            employer_name: (fd.get('employer_name') || '').trim(),
            disposable_earnings_usd: Number(fd.get('disposable_earnings_usd')) || 0,
            min_wage_usd: Number(fd.get('min_wage_usd')) || 0,
            cap_pct: Number(fd.get('cap_pct')) || 0,
            min_wage_multiplier: Number(fd.get('min_wage_multiplier')) || 0,
            pay_period: (fd.get('pay_period') || '').trim(),
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcWageGarnishment(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.garnish.toast.error'), { level: 'error' });
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
    const bindKey = {
        percentage: 'view.garnish.bind.percentage',
        floor: 'view.garnish.bind.floor',
        none: 'view.garnish.bind.none',
    }[doc.binding_cap] || 'view.garnish.bind.none';
    const el = mount.querySelector('#garnish-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card neg"><div class="label" data-i18n="view.garnish.card.garnishable">Garnishable</div>
                    <div class="value">${money(doc.garnishable_usd)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.garnish.card.keeps">Employee keeps</div>
                    <div class="value">${money(doc.employee_keeps_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.garnish.card.pctcap">25% cap</div>
                    <div class="value">${money(doc.percentage_cap_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.garnish.card.floor">Protected floor</div>
                    <div class="value">${money(doc.protected_floor_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.garnish.card.binding">Binding cap</div>
                    <div class="value" data-i18n="${bindKey}">${esc(doc.binding_cap)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="garnish-copy" type="button" data-i18n="view.garnish.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="garnish-download" type="button" data-i18n="view.garnish.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#garnish-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.garnish.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.garnish.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#garnish-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'wage-garnishment.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
