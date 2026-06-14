// Fixed-asset disposal generator — gain/loss with §1245 ordinary recapture and
// §1231 split, via /calc/asset-disposal.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderAssetDisposal(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.dispose.h1.title">// FIXED-ASSET DISPOSAL</span></h1>
        <p class="muted small" data-i18n="view.dispose.hint.intro">
            The gain or loss when a business sells or scraps a depreciable asset. Net book value is cost less
            accumulated depreciation; gain or loss is proceeds less net book value. For §1245 property, gain is
            ordinary income to the extent of prior depreciation (recapture), any amount above the original cost
            is §1231 capital gain, and a sale below book value is an ordinary §1231 loss. Drafting aid, not tax advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.dispose.h2.inputs">Disposal inputs</h2>
            <form id="dispose-form" class="inline-form">
                <label><span data-i18n="view.dispose.label.company">Company</span>
                    <input type="text" name="company_name" value="Acme Co" required></label>
                <label><span data-i18n="view.dispose.label.asset">Asset</span>
                    <input type="text" name="asset_label" value="CNC machine" required></label>
                <label><span data-i18n="view.dispose.label.cost">Original cost ($)</span>
                    <input type="number" step="100" min="0" name="cost_usd" value="10000" required></label>
                <label><span data-i18n="view.dispose.label.accum">Accumulated depreciation ($)</span>
                    <input type="number" step="100" min="0" name="accumulated_depreciation_usd" value="6000" required></label>
                <label><span data-i18n="view.dispose.label.proceeds">Sale proceeds ($)</span>
                    <input type="number" step="100" min="0" name="proceeds_usd" value="7000"></label>
                <label><span data-i18n="view.dispose.label.disposaldate">Disposal date</span>
                    <input type="date" name="disposal_date" value="2026-06-15" required></label>
                <label><span data-i18n="view.dispose.label.date">Statement date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.dispose.label.note">Note (optional)</span>
                    <input type="text" name="note" value="" placeholder="${esc(t('view.dispose.ph.note'))}"></label>
            </form>
        </div>
        <div id="dispose-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#dispose-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            asset_label: (fd.get('asset_label') || '').trim(),
            cost_usd: Number(fd.get('cost_usd')) || 0,
            accumulated_depreciation_usd: Number(fd.get('accumulated_depreciation_usd')) || 0,
            proceeds_usd: Number(fd.get('proceeds_usd')) || 0,
            disposal_date: fd.get('disposal_date'),
            date: fd.get('date'),
            note: (fd.get('note') || '').trim(),
        };
        try {
            const doc = await api.calcAssetDisposal(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.dispose.toast.error'), { level: 'error' });
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
    const glCls = doc.gain_loss_usd >= 0 ? 'pos' : 'neg';
    const glLabel = doc.is_loss ? t('view.dispose.card.loss') : t('view.dispose.card.gain');
    const el = mount.querySelector('#dispose-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card ${glCls}"><div class="label">${esc(glLabel)}</div>
                    <div class="value">${money(Math.abs(doc.gain_loss_usd))}</div></div>
                <div class="card"><div class="label" data-i18n="view.dispose.card.book">Net book value</div>
                    <div class="value">${money(doc.net_book_value_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.dispose.card.ordinary">Ordinary (§1245)</div>
                    <div class="value">${money(doc.ordinary_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.dispose.card.capital">§1231 capital gain</div>
                    <div class="value">${money(doc.section_1231_gain_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="dispose-copy" type="button" data-i18n="view.dispose.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="dispose-download" type="button" data-i18n="view.dispose.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#dispose-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.dispose.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.dispose.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#dispose-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'asset-disposal.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
