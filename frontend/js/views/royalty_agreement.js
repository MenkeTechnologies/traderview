// Royalty / license agreement generator — earned vs minimum-guarantee royalty
// with recoupable advance, via /calc/royalty.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

export async function renderRoyalty(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.royalty.h1.title">// ROYALTY / LICENSE AGREEMENT</span></h1>
        <p class="muted small" data-i18n="view.royalty.hint.intro">
            A licensor grants the right to use intellectual property in exchange for royalties. The earned
            royalty is a rate on licensed-product revenue; a minimum guarantee floors what is owed for the
            period; and a recoupable advance is credited against amounts due. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.royalty.h2.inputs">License terms</h2>
            <form id="royalty-form" class="inline-form">
                <label><span data-i18n="view.royalty.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.royalty.label.licensor">Licensor</span>
                    <input type="text" name="licensor_name" value=""></label>
                <label><span data-i18n="view.royalty.label.licensee">Licensee</span>
                    <input type="text" name="licensee_name" value=""></label>
                <label><span data-i18n="view.royalty.label.property">Licensed property</span>
                    <input type="text" name="property_label" value="the Patent"></label>
                <label><span data-i18n="view.royalty.label.revenue">Period revenue ($)</span>
                    <input type="number" step="10000" min="0" name="revenue_usd" value="2000000" required></label>
                <label><span data-i18n="view.royalty.label.rate">Royalty rate (%)</span>
                    <input type="number" step="0.1" min="0" name="rate_pct" value="8" required></label>
                <label><span data-i18n="view.royalty.label.minimum">Minimum guarantee ($)</span>
                    <input type="number" step="10000" min="0" name="minimum_guarantee_usd" value="100000"></label>
                <label><span data-i18n="view.royalty.label.advance">Recoupable advance ($)</span>
                    <input type="number" step="10000" min="0" name="advance_usd" value="50000"></label>
                <label><span data-i18n="view.royalty.label.period">Reporting period (months)</span>
                    <input type="number" step="1" min="1" name="period_months" value="12" required></label>
                <label><span data-i18n="view.royalty.label.date">Date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.royalty.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.royalty.ph.statute'))}"></label>
            </form>
        </div>
        <div id="royalty-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#royalty-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            licensor_name: (fd.get('licensor_name') || '').trim(),
            licensee_name: (fd.get('licensee_name') || '').trim(),
            property_label: (fd.get('property_label') || '').trim(),
            revenue_usd: Number(fd.get('revenue_usd')) || 0,
            rate_pct: Number(fd.get('rate_pct')) || 0,
            minimum_guarantee_usd: Number(fd.get('minimum_guarantee_usd')) || 0,
            advance_usd: Number(fd.get('advance_usd')) || 0,
            period_months: Number(fd.get('period_months')) || 0,
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcRoyalty(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.royalty.toast.error'), { level: 'error' });
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
    const minNote = doc.minimum_applied
        ? ` <span class="muted small" data-i18n="view.royalty.minapplied">(minimum)</span>`
        : '';
    const unrecoupedCard = doc.unrecouped_advance_usd > 0
        ? `<div class="card neg"><div class="label" data-i18n="view.royalty.card.unrecouped">Unrecouped advance</div>
               <div class="value">${money(doc.unrecouped_advance_usd)}</div></div>`
        : '';
    const el = mount.querySelector('#royalty-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.royalty.card.payable">Payable now</div>
                    <div class="value">${money(doc.payable_now_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.royalty.card.earned">Earned royalty</div>
                    <div class="value">${money(doc.earned_royalty_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.royalty.card.total">Total due${minNote}</div>
                    <div class="value">${money(doc.total_due_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.royalty.card.effective">Effective rate</div>
                    <div class="value">${pct(doc.effective_rate_pct)}</div></div>
                ${unrecoupedCard}
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="royalty-copy" type="button" data-i18n="view.royalty.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="royalty-download" type="button" data-i18n="view.royalty.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#royalty-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.royalty.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.royalty.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#royalty-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'royalty-agreement.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
