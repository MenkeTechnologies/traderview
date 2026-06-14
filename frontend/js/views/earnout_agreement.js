// Earnout agreement generator — contingent consideration (rate × excess over
// threshold, capped) on a business sale, via /calc/earnout.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

export async function renderEarnout(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.earnout.h1.title">// EARNOUT AGREEMENT</span></h1>
        <p class="muted small" data-i18n="view.earnout.hint.intro">
            The contingent-consideration provision of a business acquisition: part of the price is paid up
            front, and the rest is earned later if the acquired business beats a performance target. The
            earnout pays a rate on the amount the actual metric exceeds a threshold, optionally capped.
            Drafting aid, not legal/tax advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.earnout.h2.inputs">Deal terms</h2>
            <form id="earnout-form" class="inline-form">
                <label><span data-i18n="view.earnout.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.earnout.label.buyer">Buyer</span>
                    <input type="text" name="buyer_name" value=""></label>
                <label><span data-i18n="view.earnout.label.seller">Seller</span>
                    <input type="text" name="seller_name" value=""></label>
                <label><span data-i18n="view.earnout.label.business">Business</span>
                    <input type="text" name="business_name" value=""></label>
                <label><span data-i18n="view.earnout.label.upfront">Upfront cash ($)</span>
                    <input type="number" step="10000" min="0" name="upfront_usd" value="2000000" required></label>
                <label><span data-i18n="view.earnout.label.metric">Performance metric</span>
                    <input type="text" name="metric_label" value="trailing-twelve-month revenue"></label>
                <label><span data-i18n="view.earnout.label.threshold">Threshold ($)</span>
                    <input type="number" step="10000" min="0" name="threshold_usd" value="1000000" required></label>
                <label><span data-i18n="view.earnout.label.actual">Actual achieved ($)</span>
                    <input type="number" step="10000" min="0" name="actual_usd" value="3000000" required></label>
                <label><span data-i18n="view.earnout.label.rate">Earnout rate (%)</span>
                    <input type="number" step="0.1" min="0" name="rate_pct" value="20" required></label>
                <label><span data-i18n="view.earnout.label.cap">Cap ($, 0 = uncapped)</span>
                    <input type="number" step="10000" min="0" name="cap_usd" value="500000"></label>
                <label><span data-i18n="view.earnout.label.period">Earnout period (months)</span>
                    <input type="number" step="1" min="1" name="period_months" value="24" required></label>
                <label><span data-i18n="view.earnout.label.date">Date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.earnout.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.earnout.ph.statute'))}"></label>
            </form>
        </div>
        <div id="earnout-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#earnout-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            buyer_name: (fd.get('buyer_name') || '').trim(),
            seller_name: (fd.get('seller_name') || '').trim(),
            business_name: (fd.get('business_name') || '').trim(),
            upfront_usd: Number(fd.get('upfront_usd')) || 0,
            metric_label: (fd.get('metric_label') || '').trim(),
            threshold_usd: Number(fd.get('threshold_usd')) || 0,
            actual_usd: Number(fd.get('actual_usd')) || 0,
            rate_pct: Number(fd.get('rate_pct')) || 0,
            cap_usd: Number(fd.get('cap_usd')) || 0,
            period_months: Number(fd.get('period_months')) || 0,
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcEarnout(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.earnout.toast.error'), { level: 'error' });
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
    const capNote = doc.cap_applied
        ? ` <span class="muted small" data-i18n="view.earnout.capped">(capped)</span>`
        : '';
    const el = mount.querySelector('#earnout-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.earnout.card.earnout">Earnout payable</div>
                    <div class="value">${money(doc.earnout_usd)}${capNote}</div></div>
                <div class="card"><div class="label" data-i18n="view.earnout.card.excess">Excess over threshold</div>
                    <div class="value">${money(doc.excess_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.earnout.card.total">Total consideration</div>
                    <div class="value">${money(doc.total_consideration_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.earnout.card.pct">Earnout % of total</div>
                    <div class="value">${pct(doc.earnout_pct_of_total)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="earnout-copy" type="button" data-i18n="view.earnout.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="earnout-download" type="button" data-i18n="view.earnout.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#earnout-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.earnout.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.earnout.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#earnout-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'earnout-agreement.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
