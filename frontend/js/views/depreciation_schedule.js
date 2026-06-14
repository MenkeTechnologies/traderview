// Depreciation schedule generator — straight-line or double-declining-balance
// period-by-period book value, via /calc/depreciation-schedule.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderDepreciationSchedule(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.depsch.h1.title">// DEPRECIATION SCHEDULE</span></h1>
        <p class="muted small" data-i18n="view.depsch.hint.intro">
            The period-by-period book-value table for a fixed asset. Straight-line spreads (cost − salvage)
            evenly over the life; double-declining-balance applies twice the straight-line rate to the
            declining book value, floored so the book value never falls below salvage; sum-of-years-digits
            weights the base by remaining life; units-of-production charges each year its share of the base
            equal to that year's units over total estimated units. Drafting aid, not accounting/tax advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.depsch.h2.inputs">Asset details</h2>
            <form id="depsch-form" class="inline-form">
                <label><span data-i18n="view.depsch.label.company">Company</span>
                    <input type="text" name="company_name" value="Acme Co" required></label>
                <label><span data-i18n="view.depsch.label.asset">Asset</span>
                    <input type="text" name="asset_label" value="Delivery van" required></label>
                <label><span data-i18n="view.depsch.label.cost">Cost ($)</span>
                    <input type="number" step="100" min="0" name="cost_usd" value="10000" required></label>
                <label><span data-i18n="view.depsch.label.salvage">Salvage value ($)</span>
                    <input type="number" step="100" min="0" name="salvage_usd" value="1000"></label>
                <label><span data-i18n="view.depsch.label.life">Life (years)</span>
                    <input type="number" step="1" min="1" name="life_years" value="5" required></label>
                <label><span data-i18n="view.depsch.label.method">Method</span>
                    <select name="method">
                        <option value="straight_line" data-i18n="view.depsch.opt.sl">Straight-line</option>
                        <option value="ddb" data-i18n="view.depsch.opt.ddb">Double-declining-balance</option>
                        <option value="syd" data-i18n="view.depsch.opt.syd">Sum-of-years-digits</option>
                        <option value="uop" data-i18n="view.depsch.opt.uop">Units-of-production</option>
                    </select></label>
                <label><span data-i18n="view.depsch.label.total_units">Total estimated units (units-of-production)</span>
                    <input type="number" step="100" min="0" name="total_estimated_units" value="100000"></label>
                <label><span data-i18n="view.depsch.label.units_per_period">Units per year, comma-separated (units-of-production)</span>
                    <input type="text" name="units_per_period" value="30000, 25000, 20000, 15000, 10000"></label>
                <label><span data-i18n="view.depsch.label.start">Placed in service (year)</span>
                    <input type="number" step="1" min="0" name="start_year" value="2026"></label>
                <label><span data-i18n="view.depsch.label.date">Date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.depsch.label.note">Note (optional)</span>
                    <input type="text" name="note" value="" placeholder="${esc(t('view.depsch.ph.note'))}"></label>
            </form>
        </div>
        <div id="depsch-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#depsch-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            asset_label: (fd.get('asset_label') || '').trim(),
            cost_usd: Number(fd.get('cost_usd')) || 0,
            salvage_usd: Number(fd.get('salvage_usd')) || 0,
            life_years: Number(fd.get('life_years')) || 0,
            method: fd.get('method') || 'straight_line',
            total_estimated_units: Number(fd.get('total_estimated_units')) || 0,
            units_per_period: (fd.get('units_per_period') || '')
                .split(',').map((s) => Number(s.trim())).filter((n) => Number.isFinite(n)),
            start_year: Number(fd.get('start_year')) || 0,
            date: fd.get('date'),
            note: (fd.get('note') || '').trim(),
        };
        try {
            const doc = await api.calcDepreciationSchedule(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.depsch.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase(), ''];
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const rows = doc.schedule.map((r) => `
        <tr><td>${r.year}</td><td>${money(r.depreciation_usd)}</td><td>${money(r.accumulated_usd)}</td><td>${money(r.book_value_usd)}</td></tr>
    `).join('');
    const el = mount.querySelector('#depsch-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.depsch.card.total">Total depreciation</div>
                    <div class="value">${money(doc.total_depreciation_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.depsch.card.base">Depreciable base</div>
                    <div class="value">${money(doc.depreciable_base_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.depsch.card.method">Method</div>
                    <div class="value">${esc(doc.method_label)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="depsch-copy" type="button" data-i18n="view.depsch.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="depsch-download" type="button" data-i18n="view.depsch.btn.download">Download .txt</button>
            </div>
        </div>
        <table class="data-table">
            <thead><tr>
                <th data-i18n="view.depsch.th.year">Year</th>
                <th data-i18n="view.depsch.th.dep">Depreciation</th>
                <th data-i18n="view.depsch.th.accum">Accumulated</th>
                <th data-i18n="view.depsch.th.book">Book value</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#depsch-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.depsch.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.depsch.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#depsch-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'depreciation-schedule.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
